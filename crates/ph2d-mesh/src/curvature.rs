//! **A CURVATURA por vértice** — o canal que faz uma escultura ser LIDA.
//!
//! `docs/3D/05.1` §4 nomeia este como *o canal que mais melhora a leitura de uma
//! escultura por unidade de custo*, e é o mesmo dado que o SSS pré-integrado vai
//! consultar (§2a: *"um dado, dois usos"*). Aqui mora só a lei; quem a acende é
//! o shader.
//!
//! # A grandeza, e por que ela é ADIMENSIONAL
//!
//! ```text
//! k(v) = dot( centroide(anel(v)) − p(v) , n(v) ) / raio_medio(anel(v))
//! ```
//!
//! O numerador é o laplaciano uniforme projetado na normal — *para que lado a
//! superfície dobra, e quanto*. O denominador é o raio médio do anel, e ele é o
//! que torna o número **invariante de escala**: `dot(...)` tem unidade de
//! comprimento e a divisão a cancela.
//!
//! ⚠️ **Sem essa divisão a wave inteira estaria errada de um jeito difícil de
//! nomear:** curvatura média tem unidade `1/comprimento`, então a MESMA forma
//! esculpida no dobro do tamanho devolveria metade do número, e a cavidade
//! clarearia quando o artista escalasse a peça. Ele leria isso como *"o
//! sombreamento mudou sozinho"*, que é a classe de bug que ninguém consegue
//! reportar.
//!
//! Numa esfera de raio `R` amostrada com aresta `h` o valor é `−h/(2R)` — ou
//! seja, **metade do ângulo de virada através de uma aresta**. É exatamente a
//! quantidade que um realce de forma quer: uma ruga de uma aresta de largura
//! lê igual numa peça grande e numa pequena.
//!
//! **Sinal:** `k > 0` é CÔNCAVO (o anel está do lado da normal — um vale, uma
//! fresta) e `k < 0` é CONVEXO (uma crista, uma quina). É a convenção que
//! deixa *cavidade* ser o lado positivo do nome.
//!
//! # Por que o laplaciano UNIFORME basta aqui
//!
//! O estimador uniforme tem má fama, e merecida: numa malha irregular ele
//! confunde curvatura com *distribuição de vértices*, e é por isso que suavizar
//! com ele encolhe a forma. ⚠️ **Essa objeção não alcança este uso, e o motivo é
//! verificável:** a parte dependente da distribuição é **tangencial**, e nós a
//! jogamos fora ao projetar na normal. Num plano — irregular como for — todo
//! vizinho está no plano, logo `centroide − p` está no plano, logo o produto
//! interno com a normal é **exatamente zero**. Há gate.
//!
//! O que sobra de dependência é de segunda ordem (num anel assimétrico o número
//! é uma média das curvaturas normais *pesada pelas direções que o anel tem*, e
//! não a curvatura média verdadeira). Para um realce de forma isso é fiel; para
//! uma medida geométrica seria o cotangente de Meyer et al. 2003 — que custa
//! dois ângulos por aresta e uma área mista por vértice, e não foi construído
//! porque **o caso que o quebraria — o plano irregular — é exatamente o caso que
//! a projeção resolve de graça**.
//!
//! # Paralelismo
//!
//! Mesma forma das normais (`crate::normals`), e pelo mesmo motivo: é um
//! **gather** sobre um CSR de ordem fixa, cada vértice escreve só o seu, e a
//! ordem da soma dentro de um vértice não muda com o escalonamento ⇒
//! byte-idêntico por construção, que é o que o ADR-0109 exige. Reusa o
//! [`crate::normals::PAR_MIN`] em vez de cunhar um piso próprio: a varredura é a
//! mesma, sobre a mesma estrutura, no mesmo laço.

use rayon::prelude::*;

use crate::adjacency::Csr;
use crate::normals::PAR_MIN;

/// A curvatura de um vértice — ver o módulo.
///
/// `0.0` é a resposta para tudo que não tem forma definida: vértice solto (anel
/// vazio) e anel colapsado (raio médio zero). ⚠️ **Zero e não um sentinela:** o
/// consumidor é um multiplicador de sombreamento, e `0` já quer dizer *"esta
/// superfície não dobra"*, que é a leitura honesta de *"não dá para saber"*.
#[must_use]
pub fn curvature_at(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    vert_verts: &Csr,
    v: usize,
) -> f32 {
    let ring = vert_verts.neighbours(v);
    if ring.is_empty() {
        return 0.0;
    }
    let p = positions[v];
    let mut acc = [0.0f32; 3];
    let mut radius = 0.0f32;
    for &j in ring {
        let q = positions[j as usize];
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        acc[0] += d[0];
        acc[1] += d[1];
        acc[2] += d[2];
        radius += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    // A soma acumula os DESLOCAMENTOS e não as posições. ⚠️ **E a vantagem disso
    // é MENOR do que eu escrevi antes de medir:** somar coordenadas absolutas
    // longe da origem e só então subtrair `p` cancela dígitos, sim, mas o erro
    // que DOMINA já está no `positions` — a coordenada `1000,05` perdeu precisão
    // ao ser armazenada, antes de qualquer soma minha, e `q − p` herda isso pelas
    // duas rotas. Medido, contra a mesma esfera na origem:
    //
    // | distância | deslocamentos | posições absolutas |
    // |---|---|---|
    // | 0 | 0 (exato) | 5,1e−7 |
    // | 1 000 | 2,8e−4 | 7,0e−4 |
    // | 100 000 | 2,9e−2 | 5,7e−2 |
    //
    // Ou seja **2 a 3×**, não ordens de grandeza. A forma fica porque é
    // estritamente melhor e não custa nada — e porque na origem ela é EXATA,
    // que é onde toda escultura de fato vive.
    if radius <= f32::MIN_POSITIVE {
        return 0.0;
    }
    let nrm = normals[v];
    // ⚠️ **A contagem do anel CANCELA e por isso não aparece:** o centroide
    // relativo é `acc / n` e o raio médio é `radius / n`, então a razão entre os
    // dois é `acc · n̂ / radius`. Dividir os dois por `n` seria trabalho que a
    // álgebra já fez — e um `n` escrito num lado só é como esta função passaria
    // a medir valência em vez de curvatura.
    let dot = acc[0] * nrm[0] + acc[1] * nrm[1] + acc[2] * nrm[2];
    dot / radius
}

/// Recalcula a curvatura de TODOS os vértices.
///
/// ⚠️ **Roda DEPOIS das normais**, sempre: ela lê `normals[v]`, e com a normal
/// velha o sinal do vértice que acabou de dobrar sai invertido — a crista
/// aparece como fresta, no exato lugar onde o artista está olhando.
pub fn recompute_curvature(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    vert_verts: &Csr,
    out: &mut Vec<f32>,
) {
    out.clear();
    out.resize(positions.len(), 0.0);
    if out.len() < PAR_MIN {
        for (v, o) in out.iter_mut().enumerate() {
            *o = curvature_at(positions, normals, vert_verts, v);
        }
    } else {
        out.par_iter_mut()
            .enumerate()
            .for_each(|(v, o)| *o = curvature_at(positions, normals, vert_verts, v));
    }
}

/// As curvaturas dos vértices listados, **na ordem de `which`** — o chamador
/// espalha. Irmã da [`crate::normals::vertex_normals_of`], pelo mesmo motivo:
/// escrever direto nos índices esparsos seria escrita concorrente sobre o mesmo
/// `Vec`.
pub fn curvature_of(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    vert_verts: &Csr,
    which: &[u32],
    out: &mut Vec<f32>,
) {
    out.clear();
    out.resize(which.len(), 0.0);
    if which.len() < PAR_MIN {
        for (o, &v) in out.iter_mut().zip(which) {
            *o = curvature_at(positions, normals, vert_verts, v as usize);
        }
    } else {
        out.par_iter_mut()
            .zip(which.par_iter())
            .for_each(|(o, &v)| *o = curvature_at(positions, normals, vert_verts, v as usize));
    }
}

#[cfg(test)]
#[path = "curvature_tests.rs"]
mod curvature_tests;
