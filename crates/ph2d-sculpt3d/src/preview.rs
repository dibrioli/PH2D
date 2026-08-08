//! **O PADRÃO VISTO NO BARRO** — a máscara de preview, por vértice.
//!
//! O swatch do painel responde *"que densidade?"*; ele desenha uma FATIA plana.
//! A pergunta que sobra é a que só a peça pode responder — ***como este padrão
//! se deita na MINHA forma?*** — e a resposta tem de ser desenhada onde a forma
//! está.
//!
//! ⚠️ **É a MESMA lei que o dab escreve, e não uma segunda.** O valor é o
//! [`Brush::alpha_weight`] na posição do vértice, freado pela máscara **pelo
//! mesmo predicado** que freia o dab ([`Verb::paints_mask`]) — um preview que
//! pintasse o padrão sobre barro protegido estaria prometendo um depósito que o
//! traço recusa, e o artista descobriria a mentira só depois de esculpir.
//!
//! ⚠️ **O que NÃO entra é o FALLOFF nem a força.** Eles são do *dab*, e um dab
//! tem centro; aqui não há gesto nenhum — o que se vê é o campo, que é o que o
//! pincel vai amostrar onde quer que ele caia. Multiplicar pela `strength`
//! faria um pincel legitimamente fraco desenhar um preview quase invisível, que
//! se lê como *"o preview quebrou"*; a intensidade do tinto é do RENDER, e é
//! constante, exatamente como a da máscara.
//!
//! ## Por que ele não mora na malha
//!
//! Um plano na [`Mesh`] atravessaria a subdivisão, o remesh, o fechamento de
//! buraco, a fusão e o documento — **cinco lugares que teriam de decidir o que
//! fazer com ele**, e esquecer um é um preview que descreve a malha anterior.
//! Ele é DERIVADO: o certo é recomputá-lo, nunca carregá-lo.

pub use ph2d_mesh::DEFAULT_PREVIEW as NO_PREVIEW;
use ph2d_mesh::{DEFAULT_MASK, Mesh};

use crate::{AlphaFrame, Brush};

/// O peso do preview num vértice, dado o frame já derivado.
///
/// ⚠️ **O `frame` chega pronto de propósito.** Derivá-lo aqui o faria nascer uma
/// vez por VÉRTICE, e ele sai de um rotor de um grau acumulado (`O(graus)`, até
/// 359 voltas) — mais caro que o padrão inteiro que ele orienta. É a mesma razão
/// pela qual o `dab_core` o iça para fora do laço, e a assinatura é o que
/// garante as duas.
#[inline]
fn weight(brush: &Brush, frame: &AlphaFrame, pos: [f32; 3], mask: f32) -> f32 {
    let keep = if brush.verb.paints_mask() {
        1.0
    } else {
        1.0 - mask
    };
    brush.alpha_weight(pos, frame) * keep
}

/// A máscara do vértice `v`, ou o default de uma malha que ninguém mascarou.
#[inline]
fn mask_of(mesh: &Mesh, v: usize) -> f32 {
    mesh.masks().map_or(DEFAULT_MASK, |m| m[v])
}

/// **O preview da malha INTEIRA.** `out` sai com um valor por vértice.
///
/// Sem padrão armado ele sai VAZIO, e vazio é o que o renderizador lê como
/// *desarmado* — não um vetor de zeros, que custaria 4 B por vértice para dizer
/// a mesma coisa.
pub fn preview_into(mesh: &Mesh, brush: &Brush, out: &mut Vec<f32>) {
    out.clear();
    if brush.alpha.is_none() {
        return;
    }
    let frame = brush.alpha_frame();
    out.reserve(mesh.vert_count());
    for (v, p) in mesh.positions().iter().enumerate() {
        out.push(weight(brush, &frame, *p, mask_of(mesh, v)));
    }
}

/// **O preview de uma PEGADA** — os vértices que um dab acabou de mover.
///
/// ⚠️ É esta porta que mantém a lei do módulo de pé: *o custo de um gesto é
/// função da pegada, nunca do documento* (`upload.rs`). Um dab move alguns
/// milhares de vértices numa malha de centenas de milhares, e recomputar o
/// preview inteiro a cada dab faria o traço engasgar quanto maior a peça — o
/// defeito exato que o upload parcial existe para evitar, um canal ao lado.
///
/// Não faz nada se `out` não mede a malha: um preview de outro comprimento é de
/// outra topologia, e escrever nele poria valores válidos nos vértices errados.
pub fn preview_verts(mesh: &Mesh, brush: &Brush, verts: &[u32], out: &mut [f32]) {
    if brush.alpha.is_none() || out.len() != mesh.vert_count() {
        return;
    }
    let frame = brush.alpha_frame();
    let pos = mesh.positions();
    for &v in verts {
        let v = v as usize;
        if v < out.len() {
            out[v] = weight(brush, &frame, pos[v], mask_of(mesh, v));
        }
    }
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
