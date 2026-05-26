//! `DynamicsParams` sub-struct. Brush Studio §1.3.9.
//! Cap ≤ 8 fields (v1 = 5).

use serde::{Deserialize, Serialize};

/// Modulação por velocidade do stroke. §1.3.9.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DynamicsParams {
    /// Velocidade modula size. -1.0 = rápido → menor; +1.0 = rápido → maior.
    pub speed_size: f32,
    pub speed_opacity: f32,
    pub speed_spacing: f32,
    /// Jitter puramente aleatório de size por stamp (0..=1).
    pub jitter_size: f32,
    pub jitter_opacity: f32,
    // 5 fields v1, 3 slots de headroom (cap 8).
}
