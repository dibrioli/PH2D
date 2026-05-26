//! `ShapeParams` sub-struct + enums relacionados. Brush Studio §1.3.4.
//! Cap: ShapeParams ≤ 20 fields (v1 = 15).

use crate::brush_handle::BrushParamsHash;
use serde::{Deserialize, Serialize};

/// Source da textura Shape (silhueta do stamp). 4 variants v1.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShapeSource {
    /// Builtin atlas slot (round_hard, round_soft, ...). Vide library §1.6.7.
    /// `name` é String para Serde Deserialize-friendly (Serde não desserializa
    /// `&'static str` sem `#[serde(borrow)]` + lifetime).
    Builtin { atlas_layer: u32, name: String },
    /// Imported user PNG → atlas layer dynamic + blake3-keyed.
    Imported {
        atlas_layer: u32,
        blake3: BrushParamsHash,
    },
}

impl ShapeSource {
    /// Default = `round_hard` builtin (atlas slot 0, spec §1.6.7).
    pub fn default_round_hard() -> Self {
        Self::Builtin {
            atlas_layer: 0,
            name: "round_hard".to_string(),
        }
    }
}

impl Default for ShapeSource {
    fn default() -> Self {
        Self::default_round_hard()
    }
}

/// Como Pencil tilt/azimuth/barrel-roll afetam a Shape.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ShapeInputStyle {
    /// Tilt + azimuth ignorados.
    #[default]
    TouchOnly,
    /// Azimuth rotaciona a Shape.
    Azimuth,
    /// Azimuth + barrel-roll rotacionam + comprimem.
    AzimuthBarrelRoll,
}

/// Antialiasing do stamp.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ShapeFiltering {
    None,
    Classic,
    /// Bilinear high-quality em compute (default).
    #[default]
    Improved,
}

/// Brush Studio §1.3.4. **Sub-cap ≤ 20 fields** (v1 = 15).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShapeParams {
    pub shape_source: ShapeSource,
    pub shape_input_style: ShapeInputStyle,
    pub shape_rotation_follow: bool,
    pub shape_scatter: f32,
    pub shape_count: u32,
    pub shape_count_jitter: f32,
    pub shape_randomized: bool,
    pub shape_flip_x: bool,
    pub shape_flip_y: bool,
    pub shape_roundness: f32,
    pub shape_pressure_roundness: f32,
    pub shape_tilt_roundness: f32,
    pub shape_vertical_jitter: f32,
    pub shape_horizontal_jitter: f32,
    pub shape_filtering: ShapeFiltering,
    // 15 fields v1, 5 slots de headroom (cap 20).
}

impl Default for ShapeParams {
    fn default() -> Self {
        // Defaults espelham Round Hard (spec §1.3.4 column "Default").
        Self {
            shape_source: ShapeSource::default(),
            shape_input_style: ShapeInputStyle::TouchOnly,
            shape_rotation_follow: false,
            shape_scatter: 0.0,
            shape_count: 1,
            shape_count_jitter: 0.0,
            shape_randomized: false,
            shape_flip_x: false,
            shape_flip_y: false,
            shape_roundness: 1.0,
            shape_pressure_roundness: 0.0,
            shape_tilt_roundness: 0.0,
            shape_vertical_jitter: 0.0,
            shape_horizontal_jitter: 0.0,
            shape_filtering: ShapeFiltering::Improved,
        }
    }
}
