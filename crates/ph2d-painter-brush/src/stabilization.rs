//! `StabilizationParams` sub-struct. Brush Studio §1.3.2.
//! Cap ≤ 8 fields (v1 = 5).

use serde::{Deserialize, Serialize};

/// Suaviza o input pre-stamp emission. §1.3.2.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StabilizationParams {
    /// "Lazy mouse" — desloca cursor de painting atrás do input real (0..=1).
    pub streamline_amount: f32,
    /// Prolonga aplicação suave de pressão (taper interno) (0..=1).
    pub streamline_pressure: f32,
    /// Média móvel sobre últimos N pontos; N = `1 + stabilization * 16` (0..=1).
    pub stabilization: f32,
    /// Remove extremidades de oscilação (0..=1).
    pub motion_filtering_amount: f32,
    /// Reinjeta expressividade depois do filtering (compensa over-smoothing) (0..=1).
    pub motion_filtering_expression: f32,
    // 5 fields v1, 3 slots de headroom (cap 8).
}
