//! **B25.A1 — uma banda do carimbo que entra em pânico derruba o lote.** O `join().unwrap_or(None)` engolia
//! o pânico e o lote devolvia «nada tocado»: a tinta daquela banda sumia em silêncio, enquanto os irmãos
//! (`stamp_color_cache`, `stamp_color_dynamic`) gritam.

use super::stamp_plain_dabs_banded_with;
use ph2d_painter_brush::{BrushSpec, Dab};

/// A altura de tela em que a banda entra em pânico — a injecção, só sob teste (no `band_work`).
pub(super) const PANICO_NA_ALTURA: u32 = 509;
const W: u32 = 512;

/// Um arco de 200 dabs de raio 12 sobre um círculo de raio 160 — o lote de figura de `stamp_banded_tests`.
#[allow(clippy::cast_precision_loss)]
fn arco() -> Vec<Dab> {
    (0..200)
        .map(|i| {
            let t = (i as f32) / 200.0 * std::f32::consts::TAU;
            let (s, c) = (t.sin(), t.cos());
            Dab {
                center: [256.0 + c * 160.0, 254.0 + s * 160.0],
                radius_px: 12.0,
                coverage: 0.6,
                color: [0.1, 0.3, 0.9],
                color_shift: [0.0; 3],
                rotation: [1.0, 0.0],
                dir: [c, s],
                arc_len: 0.0,
                stroke_radius_px: 12.0,
                step: [0.0, 0.0],
                ante: None,
                tilt: None,
            }
        })
        .collect()
}

#[test]
#[should_panic(expected = "stamp band panicked")]
fn uma_banda_que_entra_em_panico_derruba_o_lote() {
    let mut buf = vec![255u8; (W as usize) * (PANICO_NA_ALTURA as usize) * 4];
    let brush = BrushSpec {
        radius_px: 12.0,
        ..BrushSpec::default()
    };
    // Piso 0: o lote vai à rota em banda (o controlo é o próprio pânico — a rota serial não o injecta).
    let _ = stamp_plain_dabs_banded_with(
        &mut buf,
        W,
        PANICO_NA_ALTURA,
        &arco(),
        &brush,
        false,
        None,
        0,
    );
}
