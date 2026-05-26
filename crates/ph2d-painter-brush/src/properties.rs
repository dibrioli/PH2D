//! `PropertiesParams` sub-struct. Brush Studio §1.3.11 (meta-config do brush).
//! Cap ≤ 10 fields (v1 = 6).

use serde::{Deserialize, Serialize};

/// Meta-config do brush (sliders caps, smudge). §1.3.11.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertiesParams {
    /// Cap superior do brush size slider (px).
    pub max_size_px: f32,
    pub min_size_px: f32,
    pub max_opacity: f32,
    pub min_opacity: f32,
    /// Quanto puxa quando usado em modo Smudge (0..=1).
    pub smudge_pull: f32,
    /// Shape rotation independe do device rotation.
    pub orient_to_screen: bool,
    // 6 fields v1, 4 slots de headroom (cap 10).
}

impl Default for PropertiesParams {
    fn default() -> Self {
        Self {
            max_size_px: 256.0,
            min_size_px: 1.0,
            max_opacity: 1.0,
            min_opacity: 0.0,
            smudge_pull: 0.5,
            orient_to_screen: false,
        }
    }
}
