//! `TaperParams` sub-struct. Brush Studio §1.3.3.
//! Cap ≤ 12 fields (v1 = 8).

use serde::{Deserialize, Serialize};

/// Estreitamento de início/fim do stroke. Pressure Taper (Pencil/tablet) +
/// Touch Taper (finger). §1.3.3.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaperParams {
    pub taper_size_start: f32,
    pub taper_size_end: f32,
    pub taper_length_start: f32,
    pub taper_length_end: f32,
    pub taper_opacity_start: f32,
    pub taper_opacity_end: f32,
    /// Se true, taper usa pressure curve em vez dos valores literais.
    pub taper_pressure_link: bool,
    /// Se true, taper_size_end espelha taper_size_start (idem opacity).
    pub taper_link_sizes: bool,
    // 8 fields v1, 4 slots de headroom (cap 12).
}

impl Default for TaperParams {
    fn default() -> Self {
        Self {
            taper_size_start: 0.0,
            taper_size_end: 0.0,
            taper_length_start: 0.0,
            taper_length_end: 0.0,
            taper_opacity_start: 0.0,
            taper_opacity_end: 0.0,
            taper_pressure_link: true,
            taper_link_sizes: true,
        }
    }
}
