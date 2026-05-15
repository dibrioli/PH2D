//! Grid-snap subsystem — floating panel + state for configuring the
//! editor's grid + snap behavior across 9 grid kinds.
//!
//! # Architectural analog
//!
//! Mirror of [`super::screens::hero::widget_gallery`] — a panel-only
//! always-on subsystem (NOT a Tool, NOT in the LeftRail) opened from
//! the TopBar Settings submenu. State lives on `HeroScreen` like
//! `widget_gallery_visible`; the Coordenador wires the trigger button
//! + `populate` / `paint` calls during integration.
//!
//! # Layout
//!
//! - [`state`] — `GridSnapState` + per-kind `*Cfg` structs + snap
//!   dispatch over `ph2d_grid`.
//! - [`ids`] — `NodeId` consts in the `1000..1099` range for the
//!   panel's interactive widgets.
//! - [`panel`] — paint helpers, populate / paint entry points
//!   matching the Widget Gallery signature.
//! - [`inspect`] — collapsible inspector subsection (probe inputs +
//!   coord-system display + computed distance/line/neighbors).
//! - [`render`] — adapters that translate `ph2d_grid` math into
//!   `ph2d_vector::VectorScene` strokes for the canvas overlay.
//!
//! Stages 8–12 fill in the empty modules as the agent progresses.

pub mod ids;
pub mod state;

pub use state::{
    ChunksCfg, GridKind, GridSnapState, HexCfg, IsoCfg, QuadtreeCfg, SquareCfg, StaggeredHexCfg,
    StaggeredSquareCfg, TriCfg, VoronoiCfg,
};
