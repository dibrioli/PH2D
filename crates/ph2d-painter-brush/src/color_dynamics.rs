//! `ColorDynamicsParams` sub-struct. Brush Studio §1.3.8.
//! Cap ≤ 36 fields (v1 = 28).
//!
//! Variação de cor dentro do pincel. Stamp-level + Stroke-level + Pressure +
//! Tilt + Barrel modulation.

use serde::{Deserialize, Serialize};

/// Variação de cor dentro do pincel. §1.3.8.
///
/// Maior sub-struct do Brush. Os 28 fields ficam estruturados em 5 grupos
/// semânticos (stamp/stroke/pressure/tilt/barrel) — todos flat para Serde
/// transparência (sub-cap ≤ 36 ADR-0044 §2.2.1).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorDynamicsParams {
    // === Stamp-level jitter (desvio aleatório por carimbo) — 6 fields ===
    pub stamp_hue_jitter: f32,
    pub stamp_saturation_jitter: f32,
    pub stamp_lightness_jitter: f32,
    pub stamp_darkness_jitter: f32,
    pub stamp_secondary_color: bool,
    pub stamp_secondary_amount: f32,
    // === Stroke-level jitter (fixado quando começa) — 6 fields ===
    pub stroke_hue_jitter: f32,
    pub stroke_saturation_jitter: f32,
    pub stroke_lightness_jitter: f32,
    pub stroke_darkness_jitter: f32,
    pub stroke_secondary_color: bool,
    pub stroke_secondary_amount: f32,
    // === Pressure modulation — 4 fields ===
    pub color_pressure_hue: f32,
    pub color_pressure_saturation: f32,
    pub color_pressure_brightness: f32,
    pub color_pressure_secondary: f32,
    // === Tilt modulation — 4 fields ===
    pub color_tilt_hue: f32,
    pub color_tilt_saturation: f32,
    pub color_tilt_brightness: f32,
    pub color_tilt_secondary: f32,
    // === Barrel-roll modulation (Pencil Pro / Wacom barrel sensors) — 4 fields ===
    pub color_barrel_hue: f32,
    pub color_barrel_saturation: f32,
    pub color_barrel_brightness: f32,
    pub color_barrel_secondary: f32,
    // === Curve scalars compartilhados — 4 fields ===
    pub hue_dynamic_amount: f32,
    pub saturation_dynamic_amount: f32,
    pub brightness_dynamic_amount: f32,
    pub secondary_dynamic_amount: f32,
    // 28 fields v1, 8 slots de headroom (cap 36).
}
