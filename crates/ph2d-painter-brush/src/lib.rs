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
//! Status (T1.4 ship — 2026-05-26):
//! - **Types + ABI:** `Brush` / 12 sub-structs / `Stamp` 96B align(16) ✓
//! - **Library:** `ROUND_HARD` builtin + `round_hard_shape()` procedural CPU ✓
//! - **GPU pipeline:** [`StampPipeline`] W1 unified compute shader (write-only
//!   `rgba8unorm` storage texture, 6 RenderingMode, premul-consistent) ✓
//! - **Shape atlas:** procedural inline in shader; texture-array atlas binding
//!   T1.5+.
//! - **Mixbox WGSL:** `pigment_mode` is a Stamp ABI slot but compute is T1.X+
//!   (ADR-0044 §2.5; det-painter excluded per §2.5.1).
//! - **Procedural Grain compute:** types in `procedural::*`; compute shader
//!   wiring is W5+.
//! - **Multi-stamp alpha-over:** T1.5+ ping-pong via `StampScheduler` (T1.4
//!   write-only stage races overlapping stamps; single-stamp Day-5 smoke
//!   unaffected).
//!
//! Crates consumidores (W1+):
//! - `ph2d-tool-painter` (ADR-0043) — consome `BrushHandle` em `PainterParams.active_brush`.
//! - `ph2d-painter-stroke` (ADR-0046) — consome `BrushHandle` + `BrushParamsHash` em `StrokeRecord`.

pub mod about;
pub mod atlas;
pub mod brush;
pub mod brush_handle;
pub mod color_dynamics;
pub mod cpu_render;
pub mod dynamics;
pub mod grain;
pub mod library;
pub mod mixbox;
pub mod pencil;
pub mod pigment;
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
pub mod wet_mix;

pub use about::AboutParams;
pub use brush::Brush;
pub use brush_handle::{BrushHandle, BrushParamsHash};
pub use color_dynamics::ColorDynamicsParams;
pub use cpu_render::apply_stamps;
pub use dynamics::DynamicsParams;
pub use grain::{
    GrainBehavior, GrainBlendMode, GrainFiltering, GrainParams, GrainSource, GrainZoom,
};
pub use library::ROUND_HARD;
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
pub use wet_mix::WetMixParams;
