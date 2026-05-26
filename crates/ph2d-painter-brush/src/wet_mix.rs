//! `WetMixParams` sub-struct. Brush Studio §1.3.7.
//! Cap ≤ 12 fields (v1 = 9).
//!
//! Subsistema próprio. Ativo se `wet_mix_enabled = true`; caso contrário todos
//! os params são ignorados (zero custo no shader).

use serde::{Deserialize, Serialize};

/// Simulação de mídia úmida (matemática — fluid sim real vive em
/// ph2d-painter-fluid opt-in W15). §1.3.7.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WetMixParams {
    /// Master toggle.
    pub wet_mix_enabled: bool,
    /// Quanto de água mistura com a tinta (0..=1; alta = transparente).
    pub dilution: f32,
    /// Tinta carregada no início do stroke (0..=1).
    pub load: f32,
    /// Taxa de deposição (0..=1; alta = deposita rápido).
    pub attack: f32,
    /// Força com que o brush puxa pintura já no canvas (0..=1).
    pub pull: f32,
    /// Espessura/contraste de textura úmida (0..=1).
    pub grade: f32,
    /// Gaussian blur aplicado ao stamp antes de blend (0..=1).
    pub blur: f32,
    pub blur_jitter: f32,
    pub wetness_jitter: f32,
    // 9 fields v1, 3 slots de headroom (cap 12).
}

impl Default for WetMixParams {
    fn default() -> Self {
        Self {
            wet_mix_enabled: false,
            dilution: 0.0,
            load: 1.0,
            attack: 0.5,
            pull: 0.0,
            grade: 0.5,
            blur: 0.0,
            blur_jitter: 0.0,
            wetness_jitter: 0.0,
        }
    }
}
