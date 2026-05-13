//! Trim Transparency — one-shot image action.
//!
//! Removes fully-transparent borders from an RGBA8 image, returning
//! a tightly-cropped buffer plus the original-coordinate bounding box
//! so the caller can reproject sprite metadata (pivot, child offsets,
//! etc.) and preserve the world-space appearance.
//!
//! ## Shape of this island
//!
//! This module is deliberately NOT a [`crate::tool::Tool`] implementor.
//! Trim Transparency is a stateless action dispatched from the
//! top-bar Image Tools row, not a stateful tool with a panel. See
//! [`INTEGRATION.md`](INTEGRATION.md) for the full integration plan
//! (where the button lives, how clicks dispatch, how undo wires up).
//!
//! Public surface:
//! - [`trim_transparency`] — the algorithm (see [`algorithm`]).
//! - [`TrimResult`], [`Bounds`] — return types.
//! - [`crop_bezpath`] — Lucide-derived icon glyph (see [`icon`]).
//!
//! No reachable `pub use` from `crate::lib` yet — the Integrator
//! re-exports as part of wiring the top-bar.

pub mod algorithm;
pub mod icon;

pub use algorithm::{Bounds, TrimResult, trim_transparency};
pub use icon::crop_bezpath;
