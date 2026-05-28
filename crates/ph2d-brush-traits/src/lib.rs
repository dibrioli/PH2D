#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! `ph2d-brush-traits` — foundational decoupling crate for the
//! Painter ↔ Vector circular-dep risk caught by L6F2 Antigravity 3rd
//! iteration.
//!
//! Per [ADR-0067](../../../docs/architecture/decisions/0067-brush-traits-decoupling.md).
//!
//! ## Mandate
//!
//! Zero dependency on `ph2d-painter-*` OR `ph2d-vector-*`. Minimal surface:
//! 3 trait methods + 2 data types. Sits below both Painter and Vector in
//! the dep graph so:
//!
//! ```text
//! ph2d-brush-traits  ◄─── ph2d-painter-brush       (impl BrushEngine)
//!         ▲
//!         └────  ph2d-node-vector-pattern-along-path  (consume dyn BrushEngine)
//!         └────  ph2d-node-vector-auto-trace          (output VectorNetwork; no Painter dep)
//! ```
//!
//! ## Why `BrushEngine` uses an associated `Target` type
//!
//! The ADR-0067 §2.3 sketch wrote `fn stamp(&self, target: &mut RenderTexture, ...)`
//! with `RenderTexture` undefined — pulling in a GPU texture type would
//! make this crate depend on `ph2d-gpu` and reintroduce a wider coupling.
//! W1 implementation chooses an **associated type** `BrushEngine::Target`
//! so each impl provides its own concrete texture type (Painter →
//! `wgpu::Texture`, Vector preview node → an in-memory raster buffer).
//! Trait object users pin the target via `dyn BrushEngine<Target = T>`.

pub mod brush_engine;
pub mod brush_ref;
pub mod stamp_spec;

pub use brush_engine::BrushEngine;
pub use brush_ref::{BrushHandle, BrushRef};
pub use stamp_spec::StampSpec;
