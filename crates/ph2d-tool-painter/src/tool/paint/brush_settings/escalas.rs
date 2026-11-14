//! O snap de 45° e as escalas dos sliders (tamanho, contagem) — filho de `brush_settings` por LOC.

use super::*;
use crate::tool::paint::brush_ranges::TRILHO_JOELHO;

/// Snap `cursor` to the nearest 45° ray from `anchor` (Blender Line Alt-constrain), projecting onto it.
/// Transcendental-free (abs/signum/mul + the `tan(22.5°)`/`tan(67.5°)`/`√½` constants) for HR-5.
pub(in crate::tool::paint) fn snap_to_45(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    const TAN_22_5: f32 = 0.414_213_56; // tan(22.5°)
    const TAN_67_5: f32 = 2.414_213_5; // tan(67.5°)
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2; // √½
    let dx = cursor[0] - anchor[0];
    let dy = cursor[1] - anchor[1];
    let (adx, ady) = (dx.abs(), dy.abs());
    // The snapped unit direction (one of the 8 rays).
    let (ux, uy) = if ady <= adx * TAN_22_5 {
        (dx.signum(), 0.0) // horizontal
    } else if ady >= adx * TAN_67_5 {
        (0.0, dy.signum()) // vertical
    } else {
        (dx.signum() * DIAG, dy.signum() * DIAG) // diagonal
    };
    // Project the cursor onto the ray (dot product = signed distance along the unit direction).
    let proj = dx * ux + dy * uy;
    [anchor[0] + ux * proj, anchor[1] + uy * proj]
}

/// Max ramp stops the panel snapshot carries (a ramp may hold up to `MAX_RAMP_STOPS = 32`; the editor
/// shows the first this-many — more than enough for hand-authored gradients).
pub const PANEL_RAMP_STOPS: usize = 16;

/// Map a radius in pixels onto the size slider's `0..1` track (inverse of [`size_norm_to_px`]).
/// Squared track → finer control at small sizes.
pub(in crate::tool::paint) fn size_px_to_norm(px: f32) -> f32 {
    let span = BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX;
    ((px - BRUSH_SIZE_MIN_PX) / span).clamp(0.0, 1.0).sqrt()
}

/// Map the size slider's `0..1` track onto a radius in pixels.
pub(in crate::tool::paint) fn size_norm_to_px(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    BRUSH_SIZE_MIN_PX + t * t * (BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX)
}

/// O trilho do Size sobre o tecto do MOTOR (B42.5): até [`TRILHO_JOELHO`] é o trilho de 512 ao bit (o mesmo
/// curso dá o mesmo raio em todo motor); acima, um motor de tecto maior soma `(tecto − 512)·u³`
/// (`u = (t − joelho)/(1 − joelho)`: valor, 1.ª e 2.ª derivada nulas no joelho) e chega ao tecto em `t = 1`.
pub(in crate::tool::paint) fn size_norm_to_px_em(t: f32, tecto: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let base = size_norm_to_px(t);
    let extra = tecto - BRUSH_SIZE_MAX_PX;
    if extra <= 0.0 || t <= TRILHO_JOELHO {
        return base;
    }
    let u = (t - TRILHO_JOELHO) / (1.0 - TRILHO_JOELHO);
    base + extra * u * u * u
}

/// Inverso de [`size_norm_to_px_em`]: fórmula fechada até ao joelho (ao bit com [`size_px_to_norm`]), bissecção
/// acima (a curva é monótona).
pub(in crate::tool::paint) fn size_px_to_norm_em(px: f32, tecto: f32) -> f32 {
    if tecto <= BRUSH_SIZE_MAX_PX || px <= size_norm_to_px(TRILHO_JOELHO) {
        return size_px_to_norm(px);
    }
    let (mut lo, mut hi) = (TRILHO_JOELHO, 1.0f32);
    for _ in 0..32 {
        let mid = 0.5 * (lo + hi);
        if size_norm_to_px_em(mid, tecto) < px {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    hi
}

/// Map a slider's `0..1` track onto a count in `1..=BRUSH_COUNT_SLIDER_MAX` (Input Samples /
/// Dash Length). Inverse of `count_to_norm` in the panel.
pub(in crate::tool::paint) fn count_from_norm(t: f32) -> u32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    1 + (t.clamp(0.0, 1.0) * span).round() as u32
}
