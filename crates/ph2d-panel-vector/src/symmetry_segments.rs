//! **A escala dos SEGMENTOS da simetria** — o mapa `track ↔ contagem` que o `populate` (o slider
//! ligado ao campo) e o `event_symmetry` (a escrita) partilham.
//!
//! ⚠️ **A SECÇÃO saiu do painel com a D10 (07/10)**: os controlos da simetria vivem na fila ② do
//! vetor (`crate::bars`), oferecidos onde se desenha e em toda ferramenta quando ela está LIGADA. As
//! leis da secção viajaram com eles e têm gate em `tests/it/the_bars_carry_the_gesture.rs`: o
//! *Enable* gateia tudo (os eixos só com ela ligada), `Segments` só no radial e `Fuse` só nos
//! espelhos, `Apply` só com simetria viva.

use ph2d_symmetry::{MAX_SEGMENTS, MIN_SEGMENTS};

/// O `scale` do slider de Segments: o track anda `0..=1` e a contagem `MIN..=MAX`.
pub(crate) const SEGMENTS_SCALE: f32 = (MAX_SEGMENTS - MIN_SEGMENTS) as f32;
/// O `offset` do mesmo slider.
pub(crate) const SEGMENTS_OFFSET: f32 = MIN_SEGMENTS as f32;

/// O track `0..=1` que corresponde a `n` segmentos.
#[must_use]
pub(crate) fn segments_to_track(n: u32) -> f32 {
    // Os dois limites são consts `u32` do vocabulário: NaN é impossível por TIPO, e a ordem
    // deles é a MESMA que o `SymmetrySpec::segments()` honra — as duas leem a mesma faixa.
    let n = n.clamp(MIN_SEGMENTS, MAX_SEGMENTS); // CLAMP-OK: bounds são consts `u32`
    (n - MIN_SEGMENTS) as f32 / SEGMENTS_SCALE
}

/// **Quantas cópias o track `0..=1` pede** — a INVERSA de [`segments_to_track`], e a porta única
/// por onde um arrasto vira contagem.
///
/// ⚠️⚠️ **O arredondamento mora NESTA fronteira, e não no consumidor.** O track é contínuo e uma
/// rosácea de `7,4` cópias não existe: quem lê a resposta (o kernel radial, via
/// `SymmetrySpec::copy_count`) recebe um número de cópias. Duas conversões — uma aqui e outra na
/// tool — divergiriam no dia em que uma delas arredondasse para o outro lado, e o sintoma seria a
/// contagem escrita no chip a discordar das cópias desenhadas.
///
/// ⚠️ A faixa é a MESMA que o `populate_symmetry` dá ao par slider↔chip (`SEGMENTS_SCALE` /
/// `SEGMENTS_OFFSET`): é por isso que a ida-e-volta fecha.
#[must_use]
pub(crate) fn track_to_segments(t: f32) -> u32 {
    let n = t.mul_add(SEGMENTS_SCALE, SEGMENTS_OFFSET);
    if !n.is_finite() {
        // Um track degenerado não escolhe nada — o piso do vocabulário é a resposta honesta.
        return MIN_SEGMENTS;
    }
    let (lo, hi) = (SEGMENTS_OFFSET, SEGMENTS_OFFSET + SEGMENTS_SCALE);
    n.round().clamp(lo, hi) as u32 // CLAMP-OK: consts `u32`, e `n` é finito pela guarda acima
}
