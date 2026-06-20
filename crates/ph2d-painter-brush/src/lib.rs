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
//! - [`spec`]   — `BrushSpec`, the brush parameters (clean-room model of Blender's `Brush`).
//! - [`falloff`]— radial distance falloff presets (Blender `eBrushCurvePreset` shapes).
//! - [`blend`]  — the 24 Blender brush blend modes, applied in the layer's native space.
//! - [`dab`]    — stamp one dab into an RGBA8 buffer using falloff + blend.

pub mod blend;
pub mod dab;
pub mod falloff;
pub mod spec;

pub use blend::{blend_over, BrushBlend};
pub use dab::{stamp_dab, DirtyRect};
pub use falloff::Falloff;
pub use spec::BrushSpec;
