//! `GrainParams` sub-struct + enums relacionados. Brush Studio §1.3.5.
//! Cap: GrainParams ≤ 20 fields (v1 = 14).
//!
//! `GrainSource` cap ≤ 6 (v1 = 4, ADR-0044 §2.6).

use crate::brush_handle::BrushParamsHash;
use crate::procedural::ProceduralGrain;
use serde::{Deserialize, Serialize};

/// Source da textura Grain. 4 variants v1.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum GrainSource {
    /// Sem grain (brush 100% shape).
    #[default]
    None,
    /// Bitmap escaneado/desenhado, atlas-resident (charcoal, canvas_weave, etc.).
    Bitmap {
        atlas_layer: u32,
        blake3: BrushParamsHash,
    },
    /// Procedural — generated em compute shader (resolução infinita).
    Procedural(ProceduralGrain),
    /// User-imported PNG → atlas-resident.
    Imported {
        atlas_layer: u32,
        blake3: BrushParamsHash,
    },
    // === 2 slots de headroom (cap ≤ 6 ADR-0044 §2.6) ===
}

/// Comportamento da grain ao longo do stroke.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum GrainBehavior {
    /// Grain segue stroke (smeary mode).
    Moving,
    /// Grain estática "atrás" do stroke (papel; default).
    #[default]
    Texturized,
}

/// Como a grain escala relativo ao brush size.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum GrainZoom {
    /// Scale fixo independente do brush size.
    Cropped,
    /// Escala com brush size (default).
    #[default]
    FollowSize,
}

/// Blend mode entre Shape e Grain. 8 modes canônicos (spec §1.5.1).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum GrainBlendMode {
    /// `grain * shape` — escurece (default).
    #[default]
    Multiply,
    LinearBurn,
    Overlay,
    SoftLight,
    HardLight,
    Screen,
    Add,
    Subtract,
}

/// Filter da Grain texture.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum GrainFiltering {
    None,
    Classic,
    #[default]
    Improved,
}

/// Brush Studio §1.3.5. **Sub-cap ≤ 20 fields** (v1 = 14).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GrainParams {
    pub grain_source: GrainSource,
    pub grain_behavior: GrainBehavior,
    pub grain_movement: f32,
    pub grain_scale: f32,
    pub grain_zoom: GrainZoom,
    pub grain_rotation: f32,
    pub grain_depth: f32,
    pub grain_depth_min: f32,
    pub grain_depth_jitter: f32,
    pub grain_offset_jitter: f32,
    pub grain_blend_mode: GrainBlendMode,
    pub grain_brightness: f32,
    pub grain_contrast: f32,
    pub grain_filtering: GrainFiltering,
    // 14 fields v1, 6 slots de headroom (cap 20).
}

impl Default for GrainParams {
    fn default() -> Self {
        Self {
            grain_source: GrainSource::None,
            grain_behavior: GrainBehavior::Texturized,
            grain_movement: 0.0,
            grain_scale: 1.0,
            grain_zoom: GrainZoom::FollowSize,
            grain_rotation: 0.0,
            grain_depth: 1.0,
            grain_depth_min: 0.0,
            grain_depth_jitter: 0.0,
            grain_offset_jitter: 0.0,
            grain_blend_mode: GrainBlendMode::Multiply,
            grain_brightness: 0.0,
            grain_contrast: 1.0,
            grain_filtering: GrainFiltering::Improved,
        }
    }
}
