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
//! - [`height`]  — Impasto: the dab's *second* output, the paint's own thickness.
//! - `jitter` — the shared deterministic RNG + the per-dab Scale / Rotate / Randomize-Color scatter.

pub mod blend;
pub mod blur;
pub mod blur_grain;
pub mod clone;
pub mod clone_grain;
pub mod curve_fit;
pub mod dab;
pub mod dynamics;
pub mod falloff;
pub mod falloff_curve;
pub mod footprint;
pub mod heading;
pub mod height;
/// **Impasto — volume conservation**: the paint the brush shoves aside, and the ridge it banks it into.
pub mod height_film;
#[cfg(test)]
#[path = "height_film_aa_tests.rs"]
mod height_film_aa_tests; // paridade 5-amostras x 9-amostras do AA (plano 26 §9.5)
pub mod height_modes;
pub mod height_push;
pub(crate) mod jitter;
pub mod mask_ops;
pub mod material;
/// The **local plane** fitted to a dab's footprint — the engine behind Flatten / Scrape / Fill. Tilted,
/// which is the whole point: a horizontal fit cuts a crater into a hillside.
pub mod plane;
pub mod ramp_alpha;
pub mod sampler;
/// **Sculpt** — the brush as a local operator on the relief (Smooth / Sharpen / Flatten / Scrape / Fill).
pub mod sculpt;
pub mod smear;
pub mod smear_field;
pub mod spec;
mod spec_default;
mod spec_frame;
#[cfg(test)]
mod spec_tests;
pub mod stamp;
pub mod stamp_color;
pub mod stamp_ramped;
pub mod stroke;
// A aritmética do cap por-traço (Accumulate OFF), numa cópia só. Arquivo próprio: é uma LEI, não um
// helper — e o módulo registra a lei alternativa que foi construída e REPROVADA (doc 25 §13.10).
pub(crate) mod stroke_cover;
pub mod stroke_method;
pub mod symmetry;
pub mod texture;

pub use blend::{BrushBlend, MAX_BRUSH_BLEND_MODES, blend_over};
pub use blur::{blur_blit_stamp, blur_dab};
pub use blur_grain::blur_blit_grain;
pub use clone::{clone_blit_stamp, clone_dab};
pub use clone_grain::clone_blit_grain;
pub use curve_fit::{CurveFit, auto_handles, fit_curve, flatten_bezier};
pub use dab::{
    DirtyRect, ShapeInput, stamp_dab, stamp_dab_ramped, stamp_dab_textured,
    stamp_dab_textured_masked,
};
pub use dynamics::Dynamics;
pub use falloff::{Falloff, MAX_FALLOFF};
pub use falloff_curve::{
    FalloffCurve, FalloffPoint, HandleType, MAX_FALLOFF_POINTS, MAX_HANDLE_TYPES,
    eval_falloff_curve,
};
pub use footprint::{DAB_FLATTEN_MAX, FootprintDeform};
pub use height::{DepthSource, DrawTo};
pub use jitter::shift_colors_like;
pub use mask_ops::{MaskCanvasOp, apply_mask_op};
pub use ramp_alpha::RampAlphaMode;
pub use sampler::MAX_INPUT_SAMPLES;
pub use smear::smear_dab;
pub use smear_field::{SmearOut, accumulate_dab_smear};
pub use spec::{AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec};
pub use stamp::{StampMask, blit_canvas_cached, blit_stamp, render_stamp_mask};
pub use stamp_color::{
    ColorStampMask, DynDab, FusedDab, accumulate_color_stamp_coverage,
    accumulate_color_stamps_fused, accumulate_color_stamps_fused_batch,
    accumulate_color_stamps_rgba_batch, accumulate_shape_layer_rgba,
    accumulate_shape_layers_rgba_batch, blit_color_stamp, render_color_stamp_mask,
    render_ramp_color_stamp,
};
pub use stamp_ramped::blit_stamp_ramped;
pub use stroke::stabilize::lazy_mouse_step;
pub use stroke::{
    Dab, POLY_MAX_SIDES, POLY_MIN_SIDES, Stroke, StrokePoint, ellipse_perimeter,
    flatten_catmull_rom, polygon_perimeter,
};
pub use stroke_method::{JitterUnit, StrokeMethod};
pub use symmetry::{MirrorAxis, SYMMETRY_MAX_SEGMENTS, SYMMETRY_MIN_SEGMENTS, SymmetrySettings};
pub use texture::patterns::{render_texture_layer, render_texture_preview};
pub use texture::{
    DEG_STEP, ImageMask, ImageRgb, MAX_TEX_PARAMS, ParamSpec, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX,
    TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TEX_TILE_BASE_PX, TexDabBasis, TextureKind,
    TextureMapping, TextureSettings, compose_shape_silhouette_kind, param_specs,
    render_shape_preview, render_stencil_preview, stencil_frame,
};
