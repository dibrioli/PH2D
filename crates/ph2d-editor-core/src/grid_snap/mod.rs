//! Grid-snap subsystem — canvas overlay renderer + snap algorithm
//! state. ADR-0029 Phase C.4 split the floating-panel chrome out into
//! `ph2d-panel-grid-snap`; what remains here is the canvas-side half:
//!
//! - [`render::paint`] — paint the active grid overlay onto the canvas
//!   (called from `paint_hero_screen` in place of the legacy hardcoded
//!   square grid).
//! - [`state::GridSnapState`] — per-kind config + snap policy + canvas
//!   color/opacity + persisted `panel_rect` (which the panel chrome
//!   reads through the [`crate::panel::PanelHostInternal`] trait).
//! - [`state::GridSnapState::snap_world`] — snap a world point to the
//!   active grid (called from gizmo Translate, drag-drop, paste).
//! - [`inspect`] — read-only diagnostics (coord systems + distance /
//!   line / neighbors) — drives the Inspect subsection painted by the
//!   panel crate.
//!
//! # Module layout post-C.4
//!
//! - [`state`] — `GridSnapState` + per-kind `*Cfg` structs + snap
//!   dispatch over `ph2d_grid`.
//! - [`ids`] — `NodeId` consts in the `1000..1099` range; re-exported
//!   by the panel crate as `ph2d_panel_grid_snap::ids`.
//! - [`inspect`] — read-only diagnostics (coord systems +
//!   distance/line/neighbors) for the probe pair.
//! - [`render`] — per-kind canvas-overlay adapters consuming
//!   `ph2d_grid` math.

pub mod ids;
pub mod inspect;
pub mod render;
pub mod state;

pub use state::{
    ChunksCfg, GridKind, GridSnapState, HexCfg, IsoCfg, QuadtreeCfg, SquareCfg, StaggeredHexCfg,
    StaggeredSquareCfg, TriCfg, VoronoiCfg,
};

use crate::interaction::WidgetStore;
use crate::zones::Rect;

/// Default rect of the grid-snap floating panel when first opened.
/// Kept here (rather than the panel crate) so editor-core sites that
/// pre-publish `GridSnapState::panel_rect` (e.g. tests, headless
/// fixtures) can seed the same value the panel chrome uses.
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    // Width matches Inspector (`style::INSPECTOR_W = 304`) per Enio's
    // 2026-05-15 redesign.
    let w = 304.0_f32.min(viewport_w - 16.0);
    let h = 640.0_f32.min(viewport_h - 16.0).max(440.0);
    let x = ((viewport_w - w) * 0.5).max(8.0);
    let y = ((viewport_h - h) * 0.5).max(8.0);
    Rect::new(x, y, w, h)
}

/// Post-C.4 stub for the legacy `populate` entry point. The panel
/// crate now registers its own widgets via `Panel::populate`; this
/// function is kept for back-compat callers (tests, fixtures) that
/// pre-populate a `WidgetStore` without a typed registry installed.
/// Empty body — the typed registry's `populate` walk does the work.
pub fn populate(_store: &mut WidgetStore) {}
