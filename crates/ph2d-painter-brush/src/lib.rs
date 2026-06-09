#![forbid(unsafe_code)]

//! ph2d-painter-brush — Brush engine core do Painter (sucessor do Procreate).
//!
//! Contrato congelado por [ADR-0044](../../../docs/architecture/decisions/0044-brush-engine-gpu.md)
//! (cascata W0 ratificada 2026-05-26). Esta crate é o **homestead** das
//! estruturas de dados foundational do brush engine:
//!
//! - [`Brush`] — 12 sub-structs (StrokePath/Stabilization/.../About) + version HR-14.
//!   Cap top-level ≤ 14; sub-caps tabela em ADR-0044 §2.2.1.
//! - [`Stamp`] — 96 bytes `repr(C, align(16))` ABI FROZEN. Hot path GPU compute.
//! - [`BrushHandle`] — opaco `u32` com bit-31 flag (built-in vs imported).
//! - [`RenderingMode`] — 6 FROZEN (Light/Uniform/Intense/Heavy Glaze + Uniform/Intense Blending).
//! - [`PigmentMode`] — Linear / Mixbox (axis ortogonal — Proposta 2 SIGGRAPH 2021).
//! - [`GrainSource`] — None / Bitmap / Procedural / Imported.
//! - [`ProceduralGrain`] — SimplexNoise / GaborNoise / PaperWeave / SprayDot (Proposta 3).
//!
//! Status (T1.6 ship — 2026-05-26):
//! - **Types + ABI:** `Brush` / 12 sub-structs / `Stamp` 96B align(16) ✓
//! - **Library (T1.6 cumulative through R5):** **4 procedural shapes
//!   shipped** — `round_hard` (slot 0), `round_soft` (slot 1),
//!   `square_hard` (slot 2), `oval_hard` (slot 3 — added R4 V-2 as the
//!   demo brush for `shape_rotation_follow`; calligraphic 2:1 oblong).
//!   Slot dispatch via `library::shape_alpha_for_slot` (CPU) +
//!   `stamp.wgsl::shape_alpha_for_slot` (GPU); gate-protected by
//!   `cpu_shader_shape_kernels_textual_parity`.
//! - **Shape atlas binding:** **deferred** to W6+ (when first custom-art
//!   shape PNG ships — `flat_chisel`, `bristle_spread`, `splatter_spread`).
//!   T1.6 chose procedural inline switch over `texture_2d_array<f32>`
//!   binding because R8 atlas quantization would drift ~1/255 per pixel
//!   CPU↔GPU and break HR-5. `library::build_shape_atlas()` rasterizer
//!   ships; consumer ships in W6+. See `library.rs` header for rationale.
//! - **Multi-stamp + rotation (T1.6):** `StampScheduler` honors
//!   `shape_count` (1..=16 stamps per spacing step), `shape_count_jitter`,
//!   `shape_rotation_follow` (atan2 stroke direction), `shape_scatter`
//!   (per-stamp rotation jitter), `shape_randomized` (rotation base
//!   lazy-init on first `advance` of the stroke when the brush has the
//!   flag set — see `StampScheduler::stroke_rotation_base` field doc
//!   for the mid-stroke brush-swap policy), and `shape_flip_x` /
//!   `shape_flip_y` (flag bits). Non-radial shapes get a tight analytic
//!   bounding-box scale `|cos θ| + |sin θ|` (peaks at √2 at 45°) when
//!   rotated — see `library::rotated_footprint_scale`.
//! - **Color Dynamics stamp-level jitter (T1.6):** per-stamp OKLab
//!   perturbation — `stamp_lightness_jitter` (bidir L offset),
//!   `stamp_darkness_jitter` (monotonic down), `stamp_hue_jitter`
//!   (rotates a/b), `stamp_saturation_jitter` (scales a/b). Distinct PRNG
//!   axis tags `0xC1..0xC4` so toggling one channel doesn't disturb the
//!   others' stream. Stroke-level + pressure/tilt/barrel modulation is
//!   deferred to T-color-full (ADR-0051).
//! - **GPU pipeline:** [`StampPipeline`] W1 unified compute shader (write-only
//!   `rgba8unorm` storage texture, 6 RenderingMode, premul-consistent,
//!   T1.5 ping-pong) ✓; T1.6 adds shape-layer dispatch + rotation/flip uv
//!   transform inside `cs_stamp`.
//! - **Mixbox WGSL:** `pigment_mode` is a Stamp ABI slot but compute is W5+
//!   (ADR-0044 §2.5; det-painter excluded per §2.5.1).
//! - **Procedural Grain compute:** types in `procedural::*`; compute shader
//!   wiring is W5+.
//!
//! Crates consumidores (W1+):
//! - `ph2d-tool-painter` (ADR-0043) — consome `BrushHandle` em `PainterParams.active_brush`.
//! - `ph2d-painter-stroke` (ADR-0046) — consome `BrushHandle` + `BrushParamsHash` em `StrokeRecord`.

pub mod about;
pub mod adjustments;
pub mod atlas;
pub mod blend;
pub mod brush;
pub mod brush_handle;
pub mod color_dynamics;
pub mod cpu_render;
pub mod diffusion;
pub mod dynamics;
pub mod grain;
pub mod grain_noise;
pub mod library;
pub mod pencil;
pub mod pigment;
pub mod pigment_mix;
pub mod procedural;
pub mod properties;
pub mod rendering;
pub mod rendering_mode;
pub mod shape;
pub mod stabilization;
pub mod stamp;
pub mod stamp_pipeline;
pub mod stamp_scheduler;
pub mod stroke_path;
pub mod taper;
pub mod watercolor;
pub mod wet_composite;
pub mod wet_mix;

pub use about::AboutParams;
pub use blend::{BlendMode, MAX_BLEND_MODES, apply as apply_blend};
pub use brush::{Brush, BrushSerializeError};
pub use brush_handle::{BrushHandle, BrushParamsHash};
pub use color_dynamics::ColorDynamicsParams;
pub use cpu_render::{
    EdgeStyle, apply_stamps, apply_stamps_buildup, apply_stamps_wash, apply_stamps_with_options,
    apply_wash_settle, coverage_bbox,
};
pub use dynamics::DynamicsParams;
pub use grain::{
    GrainBehavior, GrainBlendMode, GrainFiltering, GrainParams, GrainSource, GrainZoom,
};
// **Audit T1.6 R7 I1-3:** crate-root re-exports cover ALL canonical
// library entry-points so downstream callers can `use ph2d_painter_
// brush::{oval_hard, OVAL_HARD_SLOT, ...}` consistently. Mixing
// `use ph2d_painter_brush::ROUND_HARD` (re-exported) with
// `use ph2d_painter_brush::library::oval_hard` (deep import) was
// silently the only pattern before — the asymmetry suggested
// `oval_hard` was somehow "second-class" and would have rotted into
// a future `pub(crate)` regression. Build/atlas internals
// (`build_shape_atlas`, `round_soft_shape`, etc.) stay deep-imports
// because they're construction-helpers, not the public brush
// surface.
pub use library::{
    BUILTIN_SHAPE_SLOT_COUNT, OVAL_HARD, OVAL_HARD_SLOT, ROUND_HARD, ROUND_HARD_SLOT, ROUND_SOFT,
    ROUND_SOFT_SLOT, SQUARE_HARD, SQUARE_HARD_SLOT, brush_from_handle, oval_hard,
    rotated_footprint_scale, round_hard, round_soft, shape_alpha_for_slot,
    shape_is_radial_symmetric, shape_oval_hard, shape_round_hard, shape_round_soft,
    shape_square_hard, square_hard,
};
pub use pencil::{CursorOutline, PencilParams};
pub use pigment::PigmentMode;
pub use procedural::ProceduralGrain;
pub use properties::PropertiesParams;
pub use rendering::{BurntEdgesMode, RenderingParams};
pub use rendering_mode::{MAX_RENDERING_MODES, RenderingMode};
pub use shape::{ShapeFiltering, ShapeInputStyle, ShapeParams, ShapeSource};
pub use stabilization::StabilizationParams;
pub use stamp::{MAX_STAMP_SIZE_PX, MAX_STAMPS_PER_DISPATCH, Stamp};
pub use stamp_pipeline::{StampGlobals, StampPipeline};
pub use stamp_scheduler::{PointerSample, StampScheduler};
pub use stroke_path::StrokePathParams;
pub use taper::TaperParams;
pub use watercolor::{WatercolorControl, WatercolorParams};
pub use wet_mix::WetMixParams;
