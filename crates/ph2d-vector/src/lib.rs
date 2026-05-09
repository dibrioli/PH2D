#![forbid(unsafe_code)]
//! ph2d-vector — Vello GPU vector renderer wrapper (M11).
//!
//! Vello is Linebender's experimental compute-based vector renderer.
//! It is currently 0.x and the API churns roughly per-quarter; we
//! pin to a specific minor version and treat regressions as
//! ADR-worthy events (plan note: "Vello em alpha — risco aceito,
//! ADR-0004 a escrever quando aparecer regressão").
//!
//! M11 ships the wrapper surface only:
//! - [`VectorScene`] — a `vello::Scene` with our naming convention
//!   and a couple of convenience helpers (filled rect, clipped
//!   path push/pop).
//! - The `Renderer` (which owns wgpu pipeline state) is NOT wrapped
//!   yet — callers that need to actually render to a surface go
//!   through `vello::Renderer` directly. M12 (editor) wires it
//!   into the desktop shell with a real surface.
//!
//! Re-exports the kurbo + peniko types that show up in the public
//! API so downstream crates don't need direct deps on every
//! Linebender package.

pub mod scene;

pub use scene::VectorScene;

// Re-export the few kurbo/peniko primitives that callers need to
// build paths and brushes. We reach for them via `vello::*` rather
// than declaring kurbo/peniko as direct deps — guarantees the
// Brush/Color/Fill types our wrapper hands to vello are the same
// monomorphic types vello expects (no version-skew accidents).
pub use vello::kurbo::{Affine, BezPath, Point, Rect, Vec2};
pub use vello::peniko::{Brush, Color, Fill};
