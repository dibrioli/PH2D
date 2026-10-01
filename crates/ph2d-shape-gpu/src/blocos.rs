//! **OS BLOCOS DE SEGMENTOS** (doc 121 §9.3) — o que tira o passe de `pixels × segmentos`.
//!
//! O fragmento soma a contribuição de cada segmento para a área do pixel, e numa forma GRANDE isso
//! são centenas de segmentos em milhares de pixels: no proxy de telemóvel (a placa integrada, 2 CU)
//! `72` estrelas de `115 × 38 px` só PREENCHIDAS custavam `2,90 ms` contra `0,58` do Vello, que corta
//! a forma em ladrilhos. ⇒ os segmentos vão em blocos de [`SEGS_POR_BLOCO`] com a caixa LOCAL deles,
//! e o shader pergunta primeiro ao bloco:
//!
//! - **acima, abaixo ou à direita do pixel** ⇒ soma ZERO (cada segmento do bloco tem `dy = 0` no
//!   pixel, ou fica à direita dele) — salta-se;
//! - **todo à ESQUERDA e ENCADEADO** (cada segmento começa onde o anterior acabou) ⇒ a contribuição de
//!   um segmento todo à esquerda é `clamp(y₀) − clamp(y₁)` (o ramo `xmax ≤ 0` da `contribuicao`), e
//!   numa corrente ela TELESCOPA: a soma do bloco é a do primeiro ponto menos a do último — duas
//!   leituras em vez de oito;
//! - **o resto** ⇒ segmento a segmento, como antes.
//!
//! ⚠️ **Os blocos não atravessam um TRECHO** (o preenchimento · as marcas · o contorno): cada trecho é
//! completado até um múltiplo de [`SEGS_POR_BLOCO`] com segmentos de comprimento ZERO no último ponto
//! — eles somam `0` (`dy = 0`) e mantêm a corrente ligada. Assim o bloco `k` é `segs[8k .. 8k+8]` em
//! qualquer geometria e depois do upload (que concatena geometrias de comprimento múltiplo de 8).

use bytemuck::{Pod, Zeroable};

/// Quantos segmentos por bloco. ⚠️ O mesmo número do shader (`SEGS_POR_BLOCO` em `shape.wgsl`).
pub const SEGS_POR_BLOCO: usize = 8;

/// Um bloco, como a placa o lê (`shape.wgsl`, `Bloco`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct BlocoDeSegmentos {
    /// A caixa LOCAL dos segmentos do bloco: `x0, y0, x1, y1`.
    pub caixa: [f32; 4],
    /// `1` quando cada segmento começa EXACTAMENTE onde o anterior acabou — a condição do telescópio.
    pub encadeado: u32,
    pub _pad: [u32; 3],
}

/// Completa o trecho que acaba no fim de `out` até um múltiplo de [`SEGS_POR_BLOCO`], com segmentos
/// de comprimento zero no último ponto. Um trecho VAZIO fica vazio.
pub(crate) fn completa(out: &mut Vec<[f32; 4]>) {
    let Some(&[_, _, x, y]) = out.last() else {
        return;
    };
    while !out.len().is_multiple_of(SEGS_POR_BLOCO) {
        out.push([x, y, x, y]);
    }
}

/// Os blocos de uma lista de segmentos JÁ completada (comprimento múltiplo de [`SEGS_POR_BLOCO`]).
pub(crate) fn blocos_de(segs: &[[f32; 4]]) -> Vec<BlocoDeSegmentos> {
    debug_assert!(segs.len().is_multiple_of(SEGS_POR_BLOCO));
    segs.chunks(SEGS_POR_BLOCO)
        .map(|b| {
            let mut c = [
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ];
            for s in b {
                c[0] = c[0].min(s[0]).min(s[2]);
                c[1] = c[1].min(s[1]).min(s[3]);
                c[2] = c[2].max(s[0]).max(s[2]);
                c[3] = c[3].max(s[1]).max(s[3]);
            }
            #[expect(
                clippy::float_cmp,
                reason = "a corrente é ao BIT: o telescópio pede o mesmo ponto"
            )]
            let encadeado = b
                .windows(2)
                .all(|w| w[0][2] == w[1][0] && w[0][3] == w[1][1]);
            BlocoDeSegmentos {
                caixa: c,
                encadeado: u32::from(encadeado),
                _pad: [0; 3],
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "blocos_tests.rs"]
mod tests;
