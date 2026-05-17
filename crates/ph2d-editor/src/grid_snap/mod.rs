//! Grid-snap subsystem — floating panel + state for configuring the
//! editor's grid + snap behavior across 9 grid kinds.
//!
//! # Architectural analog
//!
//! Mirror of [`super::screens::hero::widget_gallery`] — a panel-only
//! always-on subsystem (NOT a Tool, NOT in the LeftRail) opened from
//! the TopBar Settings submenu. State lives on `HeroScreen` like
//! `widget_gallery_visible`; the Coordenador wires the trigger button
//! + `populate` / `paint` / `apply_event` calls during integration.
//!
//! # Public facade
//!
//! Six entry points the Coordenador's integration calls:
//! - [`populate`] — register interactive nodes in `WidgetStore`.
//! - [`default_rect`] — initial panel rect when first shown.
//! - [`paint`] — render the panel + inspect subsection.
//! - [`apply_event`] — consume a `WidgetEvent::Click`, mutate state,
//!   return `true` when handled (stops dispatcher fall-through).
//! - [`render::paint`] — render the grid overlay onto the canvas
//!   (called from the host's render loop in place of the existing
//!   hardcoded-square grid).
//! - [`state::GridSnapState::snap_world`] — snap a world point to
//!   the active grid (called from gizmo Translate, drag-drop, paste).
//!
//! # Module layout
//!
//! - [`state`] — `GridSnapState` + per-kind `*Cfg` structs + snap
//!   dispatch over `ph2d_grid`.
//! - [`ids`] — `NodeId` consts in the `1000..1099` range.
//! - [`panel`] — panel paint + event handler + `default_rect`.
//! - [`inspect`] — read-only diagnostics (coord systems +
//!   distance/line/neighbors) for the probe pair.
//! - [`render`] — per-kind canvas-overlay adapters consuming
//!   `ph2d_grid` math.

pub mod ids;
pub mod inspect;
pub mod panel;
pub mod render;
pub mod state;

pub use panel::{
    apply_event, default_rect, last_content_h, last_visible_h, paint, populate,
    set_current_display_unit, sync_meter_inputs_to_display_unit,
};
pub use state::{
    ChunksCfg, GridKind, GridSnapState, HexCfg, IsoCfg, QuadtreeCfg, SquareCfg, StaggeredHexCfg,
    StaggeredSquareCfg, TriCfg, VoronoiCfg,
};

use crate::interaction::WidgetEvent;
use crate::panel_registry::{PaintCtx, PanelManifest};
use crate::screens::hero::HeroScreen;

/// Wave 5 stage C+D — declarative panel manifest. Stage C is a no-op
/// paint thunk; stage D moves the per-frame logic here.
pub static PANEL_MANIFEST: PanelManifest = PanelManifest {
    id: "grid_snap",
    panel_node_id: ids::GS_PANEL,
    default_visible: false,
    paint_fn: paint_thunk,
    apply_event_fn: apply_event_thunk,
    populate_fn: populate,
};

#[allow(clippy::needless_pass_by_ref_mut)]
fn paint_thunk(_ctx: &mut PaintCtx) {}

fn apply_event_thunk(_hero: &mut HeroScreen, _ev: WidgetEvent) -> bool {
    false
}
