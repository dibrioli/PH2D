//! `PencilParams` sub-struct + curves. Brush Studio §1.3.10.
//! Cap ≤ 14 fields (v1 = 9).

use serde::{Deserialize, Serialize};

/// `pressure_targets` / `tilt_targets` / `barrel_targets` bitmask — which brush
/// attributes the input axis modulates (spec §1.3.10). The `Size` + `Opacity`
/// bits are the canonical Procreate pressure response (default
/// `pressure_targets = SIZE | OPACITY`).
pub const PRESSURE_TARGET_SIZE: u32 = 1;
/// Pressure modulates per-stamp opacity (deposit rate). See [`PRESSURE_TARGET_SIZE`].
pub const PRESSURE_TARGET_OPACITY: u32 = 2;
/// Pressure modulates per-stamp flow.
pub const PRESSURE_TARGET_FLOW: u32 = 4;
/// Pressure modulates wet-mix bleed (reserved, wet-mix subsystem).
pub const PRESSURE_TARGET_BLEED: u32 = 8;

/// Evaluate an 8-control-point response curve (piecewise-linear) at `x ∈ [0,1]`.
///
/// Control points are `(input, output)` pairs with **ascending** `input`
/// (the default identity curve is `(t, t)` for `t = i/7`). `x` is clamped to
/// `[0,1]`; below the first / above the last control point the curve holds the
/// endpoint value. Output is clamped to `[0,1]`.
///
/// **HR-5 determinism:** pure IEEE arithmetic (`+`, `-`, `*`, `/`) — no
/// transcendentals, no clock, no PRNG — so it is bit-identical cross-OS and
/// replay-stable. This is the canonical Apple-Pencil / tablet pressure (and
/// tilt / barrel) response evaluator used by the [`StampScheduler`](crate::StampScheduler).
#[must_use]
pub fn eval_curve8(curve: &[(f32, f32); 8], x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= curve[0].0 {
        return curve[0].1.clamp(0.0, 1.0);
    }
    for w in curve.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        if x <= x1 {
            let span = x1 - x0;
            let t = if span.abs() < 1e-9 {
                0.0
            } else {
                (x - x0) / span
            };
            return (y0 + (y1 - y0) * t).clamp(0.0, 1.0);
        }
    }
    curve[7].1.clamp(0.0, 1.0)
}

/// Estilo do cursor durante painting.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CursorOutline {
    None,
    /// Contraste (default).
    #[default]
    Contrast,
    /// Cor ativa do brush.
    ActiveColor,
}

/// Apple Pencil / Tablet Pen curves + targets per-brush. §1.3.10.
///
/// Curves são serializadas como 8-control-point arrays inline (sem sub-struct
/// `Curve` nominal — evita sub-sub-structs ADR-0044 §2.2). Total 24 floats por
/// curve, flat. Contam como 3 fields (cada curve é UM array).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PencilParams {
    /// Curva pressão input → output normalizado. 8 control points × (in, out).
    pub pressure_curve: [(f32, f32); 8],
    /// Curva tilt (radianos, 0 = vertical, π/2 = horizontal).
    pub tilt_curve: [(f32, f32); 8],
    /// Curva barrel roll (radianos).
    pub barrel_roll_curve: [(f32, f32); 8],
    /// Bitmask: Size=1, Opacity=2, Flow=4, Bleed=8.
    pub pressure_targets: u32,
    /// Bitmask: Size=1, Opacity=2, Gradation=4, Bleed=8, SizeCompression=16.
    pub tilt_targets: u32,
    /// Bitmask: Size=1, Opacity=2, Bleed=4.
    pub barrel_targets: u32,
    pub cursor_outline: CursorOutline,
    /// Mostra preview de pressão estimada no hover (M2+, Wacom hover).
    pub hover_estimated_pressure: bool,
    /// Fill do shape no hover (vs apenas outline).
    pub hover_fill: bool,
    // 9 fields v1, 5 slots de headroom (cap 14).
}

impl Default for PencilParams {
    fn default() -> Self {
        // Identity curves: y = x.
        let identity_curve = {
            let mut c = [(0.0_f32, 0.0_f32); 8];
            for (i, entry) in c.iter_mut().enumerate() {
                let t = i as f32 / 7.0;
                *entry = (t, t);
            }
            c
        };
        Self {
            pressure_curve: identity_curve,
            tilt_curve: identity_curve,
            barrel_roll_curve: identity_curve,
            pressure_targets: 1 | 2, // Size + Opacity
            tilt_targets: 0,
            barrel_targets: 0,
            cursor_outline: CursorOutline::Contrast,
            hover_estimated_pressure: true,
            hover_fill: false,
        }
    }
}
