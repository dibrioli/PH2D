//! As linhas do seletor de fontes do Arrange da CENA (onda 4, item 5): os clips de TODAS as
//! animações agrupados por dono — irmão de [`super::timeline`], que está no tecto de LOC.
//!
//! ⚠️ **Uma faixa DERIVADA, não um array**: o número de linhas é o do documento, não um pool.

use ph2d_a11y::NodeId;

use super::TIMELINE_CLIP_DD;

/// ⭐ **Quantas linhas a lista agrupada pode ter** — e é um recurso, não um número escolhido:
///
/// | parcela | tecto | de onde (medido no código) |
/// |---|---|---|
/// | clips | `65 536` | um strip guarda o clip num `u16` (`StripSource::Clip`), e o documento recusa o clip `u16::MAX + 2` (`TimelineDoc::clip_room` → `ClipRefusal::DocumentFull`) |
/// | cabeçalhos | `≤ 65 536` | um por dono, e todo dono tem ≥ 1 clip |
///
/// O gate `a_faixa_das_fontes_e_unica` prova que os `131 072` ids da faixa são distintos entre si
/// e dos ids fixos deste painel. Só as linhas VISÍVEIS chegam ao store (registo no `paint`), por
/// isso a faixa não custa nada ao arranque.
pub const TIMELINE_SOURCE_ROWS: usize = 2 * (u16::MAX as usize + 1);

/// SplitMix64 — mistura estável (os ids sintéticos têm de sobreviver entre quadros).
const fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A linha `i` da lista agrupada (cabeçalho ou clip), derivada do chip ▾.
#[must_use]
pub const fn timeline_source_opt(i: usize) -> NodeId {
    let h = mix(TIMELINE_CLIP_DD.0 ^ mix(i as u64 + 0x5_0C));
    NodeId(if h == 0 { 1 } else { h })
}
