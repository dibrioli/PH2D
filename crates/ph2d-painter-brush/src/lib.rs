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
//! T1.3 status: **skeleton stub.** Tipos + defaults + library com brushes
//! built-in. GPU compute pipeline (`StampPipeline`) é T1.4+; atlas real é
//! T1.5+; Mixbox port WGSL é T-color+T1.X+; Procedural Grain compute é W5+.
//!
//! Crates consumidores (W1+):
//! - `ph2d-tool-painter` (ADR-0043) — consome `BrushHandle` em `PainterParams.active_brush`.
//! - `ph2d-painter-stroke` (ADR-0046) — consome `BrushHandle` + `BrushParamsHash` em `StrokeRecord`.

pub mod about;
pub mod atlas;
pub mod brush;
pub mod brush_handle;
pub mod color_dynamics;
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
pub mod stroke_path;
pub mod taper;
pub mod wet_mix;

pub use about::AboutParams;
pub use brush::Brush;
pub use brush_handle::{BrushHandle, BrushParamsHash};
pub use color_dynamics::ColorDynamicsParams;
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
pub use stroke_path::StrokePathParams;
pub use taper::TaperParams;
pub use wet_mix::WetMixParams;
