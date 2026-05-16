//! Make Square — one-shot image action.
//!
//! Pads an RGBA8 image with fully-transparent pixels on its shorter
//! axis so the result is a perfect square (`width == height`) without
//! distorting or cropping any source pixel. The original is centered
//! inside the new canvas; the new transparent band is split as evenly
//! as possible between the two opposite edges (floor on the leading
//! edge, ceil on the trailing edge when the diff is odd).
//!
//! ## Shape of this island
//!
//! Make Square is a stateless action dispatched from the top-bar
//! Image Tools row — same shape as [`super::trim_transparency`], not a
//! [`crate::tool::Tool`] implementor with panel state. See
//! [`INTEGRATION.md`](INTEGRATION.md) for the integration plan (button
//! placement, click dispatch, pivot reprojection, undo).
//!
//! Public surface:
//! - [`make_square`] — the algorithm (see [`algorithm`]).
//! - [`MakeSquareResult`] — return type.
//! - [`square_bezpath`] — Lucide-derived icon glyph (see [`icon`]).
//!
//! No reachable `pub use` from `crate::lib` yet — the Coordenador
//! re-exports as part of wiring the Image Tools cluster.

pub mod algorithm;
pub mod icon;

pub use algorithm::{MakeSquareResult, make_square};
pub use icon::square_bezpath;
