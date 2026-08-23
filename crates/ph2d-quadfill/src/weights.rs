//! ⭐⭐ **OS PESOS DO ACHATAMENTO** — valor médio e cotangente, lado a lado.
//!
//! ⚠️ **Irmão do [`crate::param`] pelo teto de LOC (HR-18, 700) e por ASSUNTO:** lá
//! *como* o patch é achatado; aqui **com que operador**. ⭐ Os dois ficam juntos de
//! propósito — a escolha entre eles é uma decisão medida, e a tabela que a decide
//! precisa dos dois docs à vista um do outro.

use std::collections::BTreeMap;

/// **AS COORDENADAS DE VALOR MÉDIO** (Floater, 2003) — por vértice, a lista de
/// `(vizinho, peso)`.
///
/// `w_ij = (tan(α/2) + tan(β/2)) / |pᵢ − pⱼ|`, onde `α` e `β` são os ângulos em
/// `pᵢ` nos dois triângulos que partilham a aresta `ij`. ⭐ **Sempre positivo**, o
/// que preserva a garantia de Tutte, e ⭐ **reproduz funções lineares**, que é o que
/// o peso uniforme não faz e é por isso que ele distorce.
///
/// ⚠️ **A soma é por CANTO e não por aresta.** A mesma aresta aparece nos dois
/// triângulos vizinhos, e cada um contribui com o seu `tan(α/2)`; somar por aresta
/// contaria o ângulo errado.
pub(crate) fn mean_value_weights(tris: &[[u32; 3]], pos: &[[f32; 3]]) -> Vec<Vec<(u32, f32)>> {
    let mut acc: Vec<BTreeMap<u32, f32>> = vec![BTreeMap::new(); pos.len()];
    for t in tris {
        for k in 0..3 {
            let (i, a, b) = (
                t[k] as usize,
                t[(k + 1) % 3] as usize,
                t[(k + 2) % 3] as usize,
            );
            let (p, q, r) = (pos[i], pos[a], pos[b]);
            // O ângulo em `p` entre `q` e `r` — o `α` do canto.
            let (u, v) = (sub(q, p), sub(r, p));
            let (lu, lv) = (norm3(u), norm3(v));
            if lu <= 0.0 || lv <= 0.0 {
                continue;
            }
            let cos = (dot3(u, v) / (lu * lv)).clamp(-1.0, 1.0);
            // `tan(θ/2) = (1 − cos θ) / sin θ`, e a forma `sin/(1+cos)` é a estável
            // perto de `θ = 0`, que é o caso comum num triângulo fino.
            let half = (1.0 - cos * cos).max(0.0).sqrt() / (1.0 + cos).max(1.0e-12);
            *acc[i].entry(t[(k + 1) % 3]).or_insert(0.0) += half / lu;
            *acc[i].entry(t[(k + 2) % 3]).or_insert(0.0) += half / lv;
        }
    }
    acc.into_iter()
        .map(|m| m.into_iter().filter(|&(_, w)| w > 0.0).collect())
        .collect()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn norm3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// ⭐⭐⭐ **OS PESOS COTANGENTES — o mapa que preserva ÂNGULO.**
///
/// # ⛔ A cerca que esta função reexamina, com um número
///
/// O [`mean_value_weights`] foi escolhido, e a nota ao lado dele diz porquê:
/// *«cotangente seria harmónico e admite peso NEGATIVO num triângulo obtuso — e é aí
/// que a garantia de Tutte se perde»*. ⚠️ **A afirmação é verdadeira e responde à
/// pergunta errada.** Ela troca **conformalidade** por **validade garantida**, e o
/// preço dessa troca — em quanto a malha final fica ENVIESADA — nunca foi medido.
///
/// ⭐ **O que a medição de 2026-08-23 mostrou:** numa esfera **lisa**, sem relevo
/// nenhum, a nossa saída tem aspecto `1,26` contra `1,22` do oráculo — as células têm
/// as **proporções certas** — e enviesamento `18°` contra `6°`. *As células estão
/// certas de tamanho e tortas de ângulo*, que é exactamente a assinatura de um mapa
/// não-conforme.
///
/// ⛔⛔ **A DEDUÇÃO QUE ESTAVA AQUI FOI REFUTADA, e ela era boa:** *«num patch de
/// quatro lados a grade é um rectângulo no domínio por construção, então o
/// enviesamento só pode nascer no MAPA — e o mapa é este peso.»* ⚠️ **O passo errado
/// é o «por construção»:** a grade do domínio é uma interpolação de Coons sobre os
/// pontos de bordo, e ela só sai rectangular se os lados **opostos** puserem o ponto
/// `k` na mesma fracção. Com a fronteira pregada por `τ` isso quase acontece
/// (`1,0°`); com ela a deslizar, não acontece de todo (`12,4°`) — e é o segundo
/// número que é honesto. ⇒ **o enviesamento pode nascer na fronteira**, e nasce.
///
/// ⭐ **E a medição confirma-o:** trocar SÓ o operador (este ficheiro) dá `12° → 12°`
/// porque os dois operadores obedecem à mesma fronteira pregada. Trocar a **condição
/// de fronteira** — o [`crate::rectangle`] — dá `16° → 14°` e move `15°` de «sem
/// nome» para `1,6°` de folga. *O operador é inocente; a fronteira não.*
///
/// # A lei
///
/// `w_ij = ½ (cot α + cot β)`, com `α` e `β` os ângulos opostos à aresta `ij` nos dois
/// triângulos que a partilham. É o gradiente exacto da energia de Dirichlet — o mapa
/// que ele produz é o **harmónico**, e num domínio convexo ele é a melhor
/// aproximação linear de um mapa conforme.
///
/// ⚠️ **O peso é NEGATIVO quando o ângulo oposto é obtuso**, e é isso que revoga a
/// garantia de Tutte. ⛔ *Não se finge que o risco não existe:* quem chama esta
/// função tem de contar os triângulos virados no domínio
/// ([`crate::aligned::flipped`]) e recuar quando eles aparecem — a mesma rede que o
/// interior alinhado já usa.
pub(crate) fn cotangent_weights(tris: &[[u32; 3]], pos: &[[f32; 3]]) -> Vec<Vec<(u32, f32)>> {
    let mut acc: Vec<BTreeMap<u32, f32>> = vec![BTreeMap::new(); pos.len()];
    for t in tris {
        for k in 0..3 {
            // O ângulo em `t[k]`; ele é OPOSTO à aresta `(t[k+1], t[k+2])`.
            let (o, a, b) = (
                t[k] as usize,
                t[(k + 1) % 3] as usize,
                t[(k + 2) % 3] as usize,
            );
            let (p, q, r) = (pos[o], pos[a], pos[b]);
            let (u, v) = (sub(q, p), sub(r, p));
            let c = dot3(u, v);
            // `cot θ = cos/sin`, e `|u×v|` é `|u||v| sin θ` — a forma estável.
            let s = norm3(cross3(u, v));
            if s <= 1.0e-20 {
                continue;
            }
            let w = 0.5 * c / s;
            *acc[a].entry(t[(k + 2) % 3]).or_insert(0.0) += w;
            *acc[b].entry(t[(k + 1) % 3]).or_insert(0.0) += w;
        }
    }
    // ⚠️ **Os pesos negativos FICAM.** Filtrá-los daria um operador que não é o
    // Laplaciano de ninguém — nem harmónico, nem de valor médio — e a malha sairia
    // de uma lei que não está escrita em lado nenhum. *A rede é a contagem de
    // virados, não uma censura ao sinal.*
    acc.into_iter().map(|m| m.into_iter().collect()).collect()
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}
