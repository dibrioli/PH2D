#![forbid(unsafe_code)]
//! `ph2d-painter-brush` — pure raster brush engine.
//!
//! **Clean-room reimplementation** of the Blender Texture Paint brush, written from the
//! *behaviour* of the Blender source (vendored at `reference/blender-texture-paint/`), not
//! from its code. Blender is GPL-2.0-or-later and PH2D is proprietary, so only the published
//! algorithms are ported — never the expression. See `docs/Painter/` for the full plan and the
//! behavioural references.
//!
//! This crate is **not** a continuation of the brush engine deleted by
//! [ADR-0099](../../docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md);
//! it is built fresh with Blender as the single reference. It is a *pure* engine: no UI, no
//! editor-core/contract dependency, no GPU. It produces pixel writes into an RGBA8 layer buffer
//! that the existing layer/effects host (`ph2d-tool-painter`) owns.
//!
//! Modules:
//! - [`spec`]    — `BrushSpec`, the brush parameters (clean-room model of Blender's `Brush`).
//! - [`falloff`] — radial distance falloff presets (Blender `eBrushCurvePreset` shapes).
//! - [`falloff_curve`] — the editable `Custom` falloff profile (Blender `CurveMapping`).
//! - [`blend`]   — the 24 Blender brush blend modes, applied in the layer's native space.
//! - [`dab`]     — stamp one dab into an RGBA8 buffer using falloff + blend.
//! - [`dynamics`]— how pen pressure drives dab size and coverage.
//! - [`stroke_method`] — the "Stroke" panel's discrete options (method + jitter unit).
//! - [`stroke`]  — the stroke engine: a pointer path → dabs (spacing, dash, jitter, stabilize).
//! - [`texture`] — the brush texture mask (procedural patterns + 2D mapping modes).

pub mod blend;
pub mod dab;
pub mod dynamics;
pub mod falloff;
pub mod falloff_curve;
pub mod sampler;
pub mod spec;
pub mod stamp;
pub mod stroke;
pub mod stroke_method;
pub mod texture;

pub use blend::{BrushBlend, MAX_BRUSH_BLEND_MODES, blend_over};
pub use dab::{DirtyRect, stamp_dab, stamp_dab_textured};
pub use dynamics::Dynamics;
pub use falloff::{Falloff, MAX_FALLOFF};
pub use falloff_curve::{
    FalloffCurve, FalloffPoint, HandleType, MAX_FALLOFF_POINTS, MAX_HANDLE_TYPES,
    eval_falloff_curve,
};
pub use sampler::MAX_INPUT_SAMPLES;
pub use spec::{AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec};
pub use stamp::{StampMask, blit_stamp, render_stamp_mask};
pub use stroke::{
    Dab, POLY_MAX_SIDES, POLY_MIN_SIDES, Stroke, StrokePoint, ellipse_perimeter,
    flatten_catmull_rom, polygon_perimeter,
};
pub use stroke_method::{JitterUnit, StrokeMethod};
pub use texture::{
    DEG_STEP, ImageMask, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX,
    TEX_SIZE_MIN, TEX_TILE_BASE_PX, TexDabBasis, TextureKind, TextureMapping, TextureSettings,
    stencil_frame,
};
