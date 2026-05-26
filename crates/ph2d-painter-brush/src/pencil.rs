//! `PencilParams` sub-struct + curves. Brush Studio §1.3.10.
//! Cap ≤ 14 fields (v1 = 9).

use serde::{Deserialize, Serialize};

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
