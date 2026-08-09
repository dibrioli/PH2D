//! **As quatro operações de máscara** — o `Masking.js` do original, menos o
//! *extract* (que cria malha, e é a W9).
//!
//! Elas agem na malha INTEIRA, e é isso que as separa do verbo `Mask`: o verbo
//! pinta *onde a mão passou* e estas respondem a *o que já está pintado*. Nenhuma
//! delas é um pincel, então nenhuma passa pela lei do traço — não há `pre` a
//! congelar, não há envelope, não há dab.
//!
//! ⚠️ **Convenção INVERTIDA em relação ao SculptGL:** aqui `0 = livre` e
//! `1 = protegido`; lá é o contrário. É a armadilha nº 1 de todo port desta área,
//! e é por isso que [`invert`] é `1 − m` e não `−m`.
//!
//! ⚠️ **Uma malha que ninguém mascarou não é materializada por engano.** O plano
//! é `Option` (uma esfera intocada não paga 4 B/vértice), e três das quatro
//! operações sobre *nenhuma máscara* devolvem *nenhuma máscara* — chamar
//! `masks_mut` nelas alocaria o plano inteiro para escrever zeros sobre zeros.
//! O `clear` é o caso em que isso importa e ele **remove** o plano.

use ph2d_mesh::{DEFAULT_MASK, Mesh};

/// **Quanto deste vértice está LIVRE para se mexer** — a convenção acima, numa
/// porta só.
///
/// A pergunta *"que fração deste vértice o gesto pode mover?"* tem **dois**
/// perguntadores: o traço (todo verbo pesa o dab por isto) e o
/// [`crate::MaskTransform`]. Duas cópias de `1 − m` não divergem por aritmética
/// — divergem no dia em que alguém portar mais um pedaço do SculptGL e escrever
/// `m` porque *lá* é assim.
///
/// ⚠️ É `1 − m` e não um clamp: a máscara já vive em `[0, 1]` por construção
/// (quem a escreve são [`blur`], [`sharpen`], [`invert`] e o verbo `Mask`, e os
/// quatro clampam), e um clamp aqui esconderia uma máscara envenenada em vez de
/// a deixar aparecer.
#[inline]
#[must_use]
pub fn free_weight(mask: f32) -> f32 {
    1.0 - mask
}

/// Quanto um passo de [`blur`] mistura com a vizinhança.
///
/// ⚠️ **Meio, e não um: um laplaciano cheio (`m = média dos vizinhos`) apaga a
/// própria borda em um passo** — a metade que suaviza é a que o artista pediu, e
/// a outra metade é o que ele já tinha. É o mesmo `0.5` do Smooth do relevo, um
/// canal ao lado.
const BLUR_MIX: f32 = 0.5;

/// Quanto um passo de [`sharpen`] afasta da vizinhança.
///
/// A operação é a do blur com o sinal trocado (`m + k·(m − média)`), e o `k` é o
/// mesmo por simetria: sharpen tem de desfazer um blur, não brigar com ele.
const SHARPEN_MIX: f32 = 0.5;

/// **Limpa a máscara.** Devolve `true` se havia o que limpar.
///
/// ⚠️ Ele **remove o plano** em vez de o preencher com zeros: uma malha limpa é
/// uma malha sem máscara, e é assim que a esfera recém-aberta nasce. Preencher
/// deixaria 4 B/vértice pagos para sempre por um gesto que existe para não
/// deixar rastro.
pub fn clear(mesh: &mut Mesh) -> bool {
    mesh.take_masks().is_some()
}

/// **Inverte:** o que estava protegido fica livre e vice-versa.
///
/// ⚠️ Ela é a única das quatro que **materializa** o plano numa malha sem
/// máscara, e tem de ser: o inverso de *nada protegido* é *tudo protegido*, e
/// esse é justamente o gesto de quem quer esculpir só um pedaço (mascara o
/// pedaço, inverte, esculpe).
pub fn invert(mesh: &mut Mesh) {
    let n = mesh.vert_count();
    let out = mesh.masks_mut();
    for m in &mut out[..n] {
        *m = 1.0 - *m;
    }
}

/// **Borra:** cada vértice caminha `BLUR_MIX` na direção da média do próprio
/// anel. Suaviza a borda de uma máscara pintada à mão.
///
/// `passes` é quantas vezes — o gesto é *"borra mais"*, e um passo é pequeno de
/// propósito (ver [`BLUR_MIX`]).
pub fn blur(mesh: &mut Mesh, passes: u32) {
    relax(mesh, passes, BLUR_MIX);
}

/// **Afia:** o oposto do [`blur`], pela mesma aritmética com o sinal trocado.
/// Uma borda borrada volta a ser uma fronteira.
pub fn sharpen(mesh: &mut Mesh, passes: u32) {
    relax(mesh, passes, -SHARPEN_MIX);
}

/// O motor dos dois: `m ← clamp(m + k·(média_do_anel − m))`.
///
/// ⚠️ **Um passo lê o estado do INÍCIO do passo** (o buffer duplo), e não o que o
/// laço acabou de escrever. A alternativa (Gauss-Seidel) faz o resultado depender
/// da ORDEM dos vértices — que é a ordem do arquivo, não uma decisão de ninguém —
/// e a mesma máscara borrada num OBJ reordenado sairia diferente. É a lição que
/// o `ph2d-wet-paint` pagou no solver (ADR-0147), um módulo ao lado.
fn relax(mesh: &mut Mesh, passes: u32, k: f32) {
    if passes == 0 || mesh.masks().is_none() {
        return;
    }
    let n = mesh.vert_count();
    // O plano SAI da malha: sem isso a adjacência (imutável) e a máscara
    // (mutável) não cabem no mesmo escopo, e a alternativa seria clonar milhões
    // de `u32` por passo. Ver `Mesh::take_masks`.
    let mut out = mesh.take_masks().expect("conferido acima");
    let mut src = vec![DEFAULT_MASK; n];
    for _ in 0..passes {
        src.copy_from_slice(&out[..n]);
        for v in 0..n {
            let ring = mesh.adjacency().vert_verts.neighbours(v);
            if ring.is_empty() {
                continue;
            }
            let sum: f32 = ring.iter().map(|&w| src[w as usize]).sum();
            let avg = sum / ring.len() as f32;
            out[v] = (src[v] + k * (avg - src[v])).clamp(0.0, 1.0);
        }
    }
    mesh.put_masks(out);
}

#[cfg(test)]
#[path = "mask_ops_tests.rs"]
mod tests;
