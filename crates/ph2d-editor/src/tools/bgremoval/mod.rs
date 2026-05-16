//! Background Removal — Tool stateful for raster bg removal in the editor.
//!
//! Pipeline: a primary segmentation backend (Chroma+Flood or GrabCut) produces
//! a binary mask; an optional Guided Filter post-process refines the mask
//! boundary into an edge-aware soft alpha; a compose step writes the alpha
//! into a copy of the input RGBA. The tool runs the pipeline twice per
//! Apply:
//!
//! 1. On a 160×160 thumbnail every slider tick → preview in the panel.
//! 2. On full RGBA when the user clicks Apply → host drains
//!    [`tool::BgRemovalTool::take_pending_apply`] and re-runs the pipeline
//!    at full resolution, then swaps `Sprite.source` to a new Individual
//!    texture (the canonical Image Tools pattern from
//!    [`crate::tools::trim_transparency`]).
//!
//! See [`INTEGRATION.md`](INTEGRATION.md) for the host wiring contract.
//!
//! ## Module layout
//!
//! - [`params`] — `BgRemovalMode` + per-mode parameter structs.
//! - [`scratch`] — reusable `BgRemovalScratch` buffer (HR-3 zero-alloc).
//! - [`algorithm`] — pure algorithms: chroma, grabcut, guided_filter +
//!   the `run_pipeline` orchestrator that wires them.
//! - [`tool`] — `BgRemovalTool` implementing [`crate::tool::Tool`].
//! - [`icon`] — `eraser_bezpath()` glyph helper for the LeftRail entry.

pub mod algorithm;
pub mod icon;
pub mod params;
pub mod scratch;
pub mod tool;

pub use algorithm::run_pipeline;
pub use icon::eraser_bezpath;
pub use params::{BgRemovalMode, BgRemovalParams, ChromaParams, GrabCutParams, GuidedFilterParams};
pub use scratch::{BgRemovalScratch, FloodSpan};
pub use tool::BgRemovalTool;
