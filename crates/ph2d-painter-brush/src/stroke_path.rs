//! `StrokePathParams` sub-struct. Brush Studio §1.3.1.
//! Cap ≤ 6 fields (v1 = 4).

use serde::{Deserialize, Serialize};

/// Distribui stamps ao longo do path. §1.3.1.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrokePathParams {
    /// Distância entre stamps em frações do diameter (0.01..=1.0). Default 0.10.
    pub spacing: f32,
    /// Variação aleatória do espaçamento (0..=1).
    pub spacing_jitter: f32,
    /// Desloca stamps perpendicular ao stroke direction (0..=1, frações do diameter).
    pub jitter_lateral: f32,
    /// Fade-out de opacity ao longo do stroke (0..=1; 1 = desvanece até o fim).
    pub falloff: f32,
    // 4 fields v1, 2 slots de headroom (cap 6).
}

impl Default for StrokePathParams {
    fn default() -> Self {
        Self {
            spacing: 0.10,
            spacing_jitter: 0.0,
            jitter_lateral: 0.0,
            falloff: 0.0,
        }
    }
}
