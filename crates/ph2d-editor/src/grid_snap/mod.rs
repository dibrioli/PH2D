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

/// Full per-frame thunk for the Grid Settings floating panel. Mirrors
/// the Widget Gallery pattern (lazy default rect on first show,
/// drag/resize deltas via store, panel rect published for chrome
/// routing) plus the meter↔display-unit reseed that lets
/// `paint`/`apply_event` see this frame's active display unit.
fn paint_thunk(ctx: &mut PaintCtx) {
    use crate::screens::hero::style::clamp_panel_rect;
    if !ctx.hero.grid.snap_state.panel_visible {
        // Symmetric stale-rect cleanup (mirrors GAL_PANEL).
        ctx.hero.store.clear_panel_rect(ids::GS_PANEL);
        return;
    }
    let base_rect = match ctx.hero.grid.snap_state.panel_rect {
        Some(r) => r,
        None => {
            let r = default_rect(ctx.layout.viewport.w, ctx.layout.viewport.h);
            ctx.hero.grid.snap_state.panel_rect = Some(r);
            r
        }
    };
    let gs_off = ctx.hero.store.blender_picker_offset(ids::GS_PANEL);
    let gs_resize = ctx.hero.store.panel_resize_delta(ids::GS_PANEL);
    let (gs_rect, gs_clamped_off, gs_clamped_resize) =
        clamp_panel_rect(base_rect, gs_off, gs_resize, ctx.viewport);
    if (gs_clamped_off.0 - gs_off.0).abs() > f32::EPSILON
        || (gs_clamped_off.1 - gs_off.1).abs() > f32::EPSILON
    {
        ctx.hero
            .store
            .set_blender_picker_offset(ids::GS_PANEL, gs_clamped_off.0, gs_clamped_off.1);
    }
    if (gs_clamped_resize.0 - gs_resize.0).abs() > f32::EPSILON
        || (gs_clamped_resize.1 - gs_resize.1).abs() > f32::EPSILON
    {
        ctx.hero.store.set_panel_resize_delta(
            ids::GS_PANEL,
            gs_clamped_resize.0,
            gs_clamped_resize.1,
        );
    }
    ctx.hero.store.set_panel_rect(ids::GS_PANEL, gs_rect);
    // Publish the active DisplayUnit + pixels_per_meter into the
    // panel's thread-local pair (read by meter↔display conversion
    // helpers inside `paint` / `apply_event`), then reseed every
    // meter-domain NumberInput's store value from the live state so
    // the rows that read from the store display the right magnitude
    // for the active unit. Focused fields are skipped by
    // `set_number_value` so in-progress edits don't get clobbered.
    set_current_display_unit(
        ctx.hero.project.display_unit,
        ctx.hero.project.pixels_per_meter,
    );
    sync_meter_inputs_to_display_unit(&ctx.hero.grid.snap_state, &mut ctx.hero.store);
    paint(
        gs_rect,
        ctx.scene,
        ctx.text_system,
        ctx.hero.theme,
        &mut ctx.hero.hit_index,
        &ctx.hero.store,
        &ctx.hero.grid.snap_state,
    );
    let content_h = last_content_h();
    let visible_h = last_visible_h();
    ctx.hero.store.set_panel_content_h(ids::GS_PANEL, content_h);
    ctx.hero.store.set_panel_visible_h(ids::GS_PANEL, visible_h);
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = ctx.hero.store.panel_scroll(ids::GS_PANEL);
    if cur > max_scroll {
        ctx.hero.store.set_panel_scroll(ids::GS_PANEL, max_scroll);
    }
}

fn apply_event_thunk(hero: &mut HeroScreen, ev: WidgetEvent) -> bool {
    // GS_COLOR_PICKER click — open the BlenderColorPicker bound to
    // the grid-snap color slot, seeding widget_color from the live
    // `state.color_rgba` so the picker reads the right initial hue.
    if let WidgetEvent::Click(id) = ev
        && id == crate::grid_snap::ids::GS_COLOR_PICKER
    {
        hero.store.set_widget_color(
            crate::grid_snap::ids::GS_COLOR_PICKER,
            hero.grid.snap_state.color_rgba,
        );
        hero.store
            .set_picker_target(Some(crate::grid_snap::ids::GS_COLOR_PICKER));
        return true;
    }
    false
}
