//! ⭐⭐⭐⭐ **AS DUAS LEIS DE COR QUE LEEM O ANEL** — o [`Verb::Blur`] e o
//! [`Verb::SmearColor`].
//!
//! # ⛔⛔ A PROVENIÊNCIA, dita antes da lei
//!
//! O [`Verb::Paint`] é um **porte T0** do `Paint.js` do SculptGL (MIT). Estes
//! dois **não são porte de nada**: a triagem foi corrida e o SculptGL **não
//! tem** um `Blur.js` nem um `Smear.js` — a pasta de ferramentas dele tem
//! catorze ficheiros e nenhum é destes. O alvo que os tem é copyleft, e
//! **nenhuma linha dele foi lida**.
//!
//! ⇒ **Estes dois são COMPOSIÇÃO de leis que esta casa já possui**, aplicadas
//! a um canal diferente, e é isso que os torna escrevíveis sem parede:
//!
//! * o **Blur** é a média do anel — a mesma relaxação laplaciana que o
//!   [`Verb::Smooth`] faz com POSIÇÕES desde que este módulo existe;
//! * o **Smear** é a lei de transporte que o [`Verb::SmearMultires`] já traz
//!   medida contra o oráculo dele (§23 desta linha): o peso de um vizinho é a
//!   parte **NEGATIVA** do cosseno entre a direcção do gesto e a aresta que
//!   leva até ele, `g = max(0, −(d̂·ê))` — *só o montante contribui*, que é o
//!   que TRANSPORTA em vez de borrar.
//!
//! ⚠️ **DIVERGÊNCIA DECLARADA:** não há lado aprovado a comparar. Se um dia o
//! dono quiser paridade com o alvo, isso é uma obra de oráculo (correr o
//! programa sem interface sobre uma malha NOSSA e gravar as cores), e não uma
//! afinação destas constantes — porque não há constante nenhuma para afinar:
//! os dois pesos saem das leis acima e o `w` é o do dab.
//!
//! # ⚠️ O BUFFER É DUPLO, e não é arrumação
//!
//! As duas leis leem a cor dos VIZINHOS e escrevem a do vértice. Escrever no
//! mesmo plano em que se lê faz metade da pegada ler o valor novo e metade o
//! velho — um Gauss-Seidel cuja saída depende da **ORDEM** em que a pegada foi
//! percorrida, que é exactamente o que o §24 desta linha já pagou na média do
//! anel das posições. ⇒ todos os alvos são calculados **antes** de qualquer
//! escrita.

use super::SculptStroke;
use crate::{Brush, Dab, Verb};
use ph2d_mesh::{DEFAULT_COLOR, Mesh};

/// A cor viva de um vértice — `DEFAULT_COLOR` quando ninguém pintou a peça.
fn cor(mesh: &Mesh, v: usize) -> [f32; 3] {
    mesh.colors().map_or(DEFAULT_COLOR, |c| c[v])
}

/// O unitário, ou `None` quando o vector **não tem direcção**. Irmão do da
/// [`super::stroke_smear`], e pela mesma razão: o limiar é o zero EXACTO.
fn unit(v: [f32; 3]) -> Option<[f32; 3]> {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q == 0.0 {
        return None;
    }
    let inv = 1.0 / q.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}

impl SculptStroke {
    /// **A MÉDIA DO ANEL, em cor** — o alvo do [`Verb::Blur`].
    ///
    /// ⚠️ **O próprio vértice entra com peso `1`**, como na irmã das posições:
    /// isto é uma relaxação *para a vizinhança*, não uma substituição por ela.
    /// Sem o próprio, um dab a peso cheio apagaria a cor do vértice de uma vez
    /// e o pincel deixaria de ter gradação.
    ///
    /// ⚠️ **Um vértice sem vizinhos devolve a própria cor** — o no-op honesto.
    fn media_do_anel_em_cor(mesh: &Mesh, v: u32) -> [f32; 3] {
        let vi = v as usize;
        let mut soma = cor(mesh, vi);
        let mut n = 1.0f32;
        for &nb in mesh.adjacency().vert_verts.neighbours(vi) {
            let c = cor(mesh, nb as usize);
            for k in 0..3 {
                soma[k] += c[k];
            }
            n += 1.0;
        }
        // ⛔⛔ **DIVIDIR, e não multiplicar pelo recíproco** — a diferença é a
        // única coisa que torna esta lei um NO-OP AO BIT numa peça de cor
        // uniforme: ali `soma` e `n` são a MESMA sequência de somas, logo
        // `soma/n` é exactamente `1`, enquanto `soma × (1/n)` erra um ulp para
        // todo `n` que não seja potência de dois. *Um pincel que muda a peça
        // onde não há nada a mudar é um passo de undo, um upload de GPU e um
        // ficheiro diferente por nada* — e foi um CONTROLO vermelho que o
        // apanhou.
        [soma[0] / n, soma[1] / n, soma[2] / n]
    }

    /// **O TRANSPORTE DA COR** — o alvo do [`Verb::SmearColor`].
    ///
    /// ⭐ É a lei do [`Verb::SmearMultires`] com a geometria em que a cor vive
    /// (as POSIÇÕES) no lugar da superfície de referência: o peso de um vizinho
    /// é `g = max(0, −(d̂·ê))`, ou seja **só quem está a montante do gesto
    /// contribui**. Com `g` escrito como `|cos|` isto borra em vez de
    /// transportar — a diferença entre as duas ferramentas cabe nesse sinal, e
    /// é a medição que o §23 desta linha registou.
    ///
    /// ⚠️ **Com o cursor parado ele é INERTE ao bit** no modo de arrasto: `d̂`
    /// é nulo, nenhum vizinho passa o teste, e o alvo é a própria cor. Os
    /// outros dois modos não têm essa degenerescência — a direcção deles nasce
    /// da geometria.
    fn transporte_da_cor(mesh: &Mesh, brush: &Brush, dab: &Dab, v: u32) -> [f32; 3] {
        let vi = v as usize;
        let own = cor(mesh, vi);
        let p_v = mesh.positions()[vi];
        let Some(dir) = unit(brush.smear_mode.direction(dab.path, dab.center, p_v)) else {
            return own;
        };
        let mut num = own;
        let mut den = 1.0f32;
        for &nb in mesh.adjacency().vert_verts.neighbours(vi) {
            let p_w = mesh.positions()[nb as usize];
            let Some(e) = unit([p_w[0] - p_v[0], p_w[1] - p_v[1], p_w[2] - p_v[2]]) else {
                continue;
            };
            let g = -(dir[0] * e[0] + dir[1] * e[1] + dir[2] * e[2]);
            if g <= 0.0 {
                continue;
            }
            let c = cor(mesh, nb as usize);
            for k in 0..3 {
                num[k] += g * c[k];
            }
            den += g;
        }
        // ⛔ **DIVIDIR pela mesma razão do irmão acima** (o no-op ao bit) — e é
        // ESTE o lado onde a mutação sangra: aqui o `den` é uma soma de pesos
        // ARBITRÁRIOS, e `x × (1/den)` erra o último bit; no irmão o `n` é uma
        // contagem, e a fixtura só produz contagens para as quais `n × (1/n)`
        // arredonda de volta a `1,0` exacto. *A mesma lei, com uma metade
        // observável e a outra não — e a que não é fica escrita.*
        [num[0] / den, num[1] / den, num[2] / den]
    }

    /// **O ALVO DE COR de cada vértice movido, calculado ANTES de escrever.**
    ///
    /// ⛔ É aqui que o buffer duplo vive — ver o cabeçalho deste módulo.
    pub(super) fn alvos_de_cor(&self, mesh: &Mesh, brush: &Brush, dab: &Dab) -> Vec<[f32; 3]> {
        self.moved
            .iter()
            .map(|&v| match brush.verb {
                Verb::Blur => Self::media_do_anel_em_cor(mesh, v),
                _ => Self::transporte_da_cor(mesh, brush, dab, v),
            })
            .collect()
    }
}

/// **AS DUAS LEIS, medidas no barro** — ver [`tests`].
#[cfg(test)]
#[path = "stroke_cor_tests.rs"]
mod tests;
