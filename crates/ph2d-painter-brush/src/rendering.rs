//! `RenderingParams` sub-struct. Brush Studio §1.3.6.
//! Cap ≤ 14 fields (v1 = 10 = 9 brush originais + `fluid_enabled` ADR-0049).

use crate::pigment::PigmentMode;
use crate::rendering_mode::RenderingMode;
use serde::{Deserialize, Serialize};

/// Intensidade do burnt edges effect.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BurntEdgesMode {
    Subtle,
    #[default]
    Normal,
    Heavy,
}

/// Modos de blending intrínsecos do brush. §1.3.6.
///
/// `RenderingParams: 10 v1 (9 brush originais + fluid_enabled de ADR-0049)
/// sub-cap ≤ 14` per ADR-0044 §2.2.1.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RenderingParams {
    pub rendering_mode: RenderingMode,
    /// Axis ortogonal — Mixbox pigment mixing (ADR-0044 §2.5).
    pub pigment_mode: PigmentMode,
    /// Multiplica o stamp alpha antes de blend (0..=1).
    pub flow: f32,
    /// Acumula tinta nas bordas (aquarela).
    pub wet_edges: bool,
    /// Escurece bordas (carvão queimado / sumi-e seco).
    pub burnt_edges: bool,
    pub burnt_edges_mode: BurntEdgesMode,
    /// Habilita blending baseado em luminance (useful para layers de luz).
    pub luminance_blending: bool,
    /// Cap mínimo de alpha — abaixo disso pixel não escreve (0..=1).
    pub alpha_threshold: f32,
    /// Pull factor para wet mix (smudge). Mantido aqui pois afeta rendering
    /// (não brush mechanics); compositor lê.
    pub stroke_blend_mode_index: u32,
    /// **ADR-0049 §2.3 (regra perfeição):** opt-in fluid sim per-brush.
    /// `false` = wet_mix matemático (default); `true` = Shallow Water solver
    /// se device tier capable (`fluid_capable()` true).
    pub fluid_enabled: bool,
    /// **W5 — stroke accumulation (orthogonal to `pigment_mode`).** `false` =
    /// *wash*: opacity caps the stroke's coverage (overlapping dabs within one
    /// stroke build up to `opacity`, no further — a single firm pass is a stable
    /// mix). `true` = *build-up*: dabs accumulate unbounded over the live canvas
    /// (scrubbing drives the stroke toward the brush colour). Default `false`
    /// (wash) — the conventional Photoshop-brush behaviour.
    #[serde(default)]
    pub accumulate: bool,
    /// Intensity of the `wet_edges` / `burnt_edges` settle (0..1). Scales the
    /// edge-darkening strength so the artist can dial a subtle bloom or a heavy
    /// rim. Default 0.6. `#[serde(default)]` keeps pre-field brush files loading
    /// (they fall back to [`default_edge_intensity`]).
    #[serde(default = "default_edge_intensity")]
    pub edge_intensity: f32,
    /// **ADR-0079 — per-brush watercolor controls.** The 15 fluid-solver knobs (diffusion +
    /// deposition + shallow-water flow) the artist drives via the Brush Studio "Watercolor"
    /// subsection. Only consumed when `fluid_enabled` (and the device is fluid-capable);
    /// `#[serde(default)]` (= the validated preset) keeps pre-field brush files loading.
    #[serde(default)]
    pub watercolor: crate::watercolor::WatercolorParams,
    /// **ADR-0087 — minimal watercolor core (`ph2d-painter-wash`), opt-in per-brush.**
    /// `true` = the simplified GPU wash (gated diffusion + FlowOutward + Beer–Lambert),
    /// selected as a PARALLEL mode to `fluid_enabled` (mutually exclusive — enabling one
    /// disables the other). `false` = unaffected. `#[serde(default)]` loads pre-field brushes.
    #[serde(default)]
    pub wash_enabled: bool,
    // 14 fields (cap 14 — FULL).
}

/// Default `edge_intensity` (also the serde fallback for old brush files).
fn default_edge_intensity() -> f32 {
    0.6
}

impl Default for RenderingParams {
    fn default() -> Self {
        Self {
            rendering_mode: RenderingMode::LightGlaze,
            pigment_mode: PigmentMode::Linear,
            flow: 1.0,
            wet_edges: false,
            burnt_edges: false,
            burnt_edges_mode: BurntEdgesMode::Normal,
            luminance_blending: false,
            alpha_threshold: 0.0,
            stroke_blend_mode_index: 0, // Normal (layer blend mode index in spec §2.2)
            fluid_enabled: false,
            accumulate: false, // wash (opacity-capped) by default
            edge_intensity: default_edge_intensity(),
            watercolor: crate::watercolor::WatercolorParams::default(),
            wash_enabled: false,
        }
    }
}
