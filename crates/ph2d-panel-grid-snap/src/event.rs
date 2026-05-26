//! Grid Snap panel `apply_event` — ADR-0029 Phase C.4 port of
//! `apply_event_thunk` + `panel::apply_event` from editor-core.
//!
//! Migrated from the legacy fn-pointer manifest's
//! `apply_event_thunk(hero: &mut HeroScreen, ev: WidgetEvent) -> EventOutcome`
//! to the typed contract
//! `(state: &mut GridSnapPanelState, host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> EventOutcome`.
//!
//! Visibility flips for the panel chrome go through
//! `host.set_panel_visible(GridSnapPanel::ID, ...)`. Per-kind / per-row
//! config writes go through the canvas-renderer `GridSnapState` exposed
//! via `host.grid_snap_state_mut()`.

use crate::GridSnapPanel;
use crate::ids;
use crate::paint_helpers::{kind_option_ids_in_order, set_neighborhood_for_active_kind};
use crate::state::{GridSnapPanelState, display_to_meters, set_current_display_unit};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState, HexCfg};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggerParity;
use ph2d_grid::tri::TriNeighborhood;

/// World-space lower-bound for cell-size / tile / bounds floats —
/// in METERS (not pixels). Used to clamp configuration inputs so the
/// grid renderer doesn't divide by ~zero. Not a UI design metric.
const MIN_CELL_SIZE_M: f32 = 0.01; // LITERAL-PX-OK: world-space minimum (meters), not a UI value

pub(crate) fn apply_event(
    _state: &mut GridSnapPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // TOPBAR_GRID_SETTINGS pill toggle + GS_CLOSE X — owned here so
    // both abrir e fechar passem pelo mesmo set_panel_visible (Enio
    // 2026-05-25: "O botão de fechar não fecha. Corrija isso.").
    if let WidgetEvent::Click(id) = ev
        && (id == ph2d_editor_core::ids::TOPBAR_GRID_SETTINGS || id == ids::GS_CLOSE)
    {
        let next = !host.panel_visible(GridSnapPanel::ID);
        host.set_panel_visible(GridSnapPanel::ID, next);
        return EventOutcome::Consumed;
    }
    // GS_TITLE_COLOR — panel-level color tag (UI canon post-2026-05-24).
    if let WidgetEvent::Click(id) = ev
        && id == ph2d_editor_core::ids::GS_TITLE_COLOR
    {
        let seed = host
            .store()
            .widget_color(id)
            .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral seed
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ph2d_editor_core::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return EventOutcome::Consumed;
    }
    // GS_COLOR_PICKER click — open the BlenderColorPicker bound to
    // the grid-snap color slot, seeding widget_color from the live
    // `state.color_rgba` so the picker reads the right initial hue.
    if let WidgetEvent::Click(id) = ev
        && id == ids::GS_COLOR_PICKER
    {
        let cur_rgba = host.grid_snap_state_clone().color_rgba;
        host.store_mut()
            .set_widget_color(ids::GS_COLOR_PICKER, cur_rgba);
        host.store_mut()
            .set_picker_target(Some(ids::GS_COLOR_PICKER));
        return EventOutcome::Consumed;
    }
    // Set display-unit pair so display→meters conversion sees the
    // active project unit when writing NumberInput edits back.
    set_current_display_unit(host.project().display_unit, host.project().pixels_per_meter);
    // Disjoint split borrow: `(&WidgetStore, &mut GridSnapState)`.
    let consumed = {
        let (store, snap_state_mut) = host.store_and_grid_snap_state_mut();
        apply_inner(snap_state_mut, ev, store)
    };
    if consumed {
        EventOutcome::Consumed
    } else {
        EventOutcome::Ignored
    }
}

/// Inner dispatch — mirror of the legacy `grid_snap::apply_event` body.
/// `store` is a read-only snapshot of the live store passed in by the
/// caller (cloned to avoid aliasing the `&mut GridSnapState` we get
/// from `grid_snap_state_mut`).
fn apply_inner(state: &mut GridSnapState, event: WidgetEvent, store: &WidgetStore) -> bool {
    match event {
        WidgetEvent::Toggled(id) => apply_toggle(state, id, store),
        WidgetEvent::ValueChanged(id) => apply_value_changed(state, id, store),
        WidgetEvent::Click(id) => apply_click(state, id),
        _ => false,
    }
}

fn apply_toggle(state: &mut GridSnapState, id: NodeId, store: &WidgetStore) -> bool {
    if id == ids::GS_SNAP_ENABLED {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get(id) {
            state.snap_enabled = *on;
        }
        return true;
    }
    if id == ids::GS_SHOW_OVERLAY {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get(id) {
            state.show_overlay = *on;
        }
        return true;
    }
    false
}

fn apply_value_changed(state: &mut GridSnapState, id: NodeId, store: &WidgetStore) -> bool {
    if id == ids::GS_OPACITY_SLIDER {
        if let Some((_, v)) = store.slider(id) {
            state.opacity = v.clamp(0.0, 1.0);
        }
        return true;
    }
    let Some(v) = store.number_value(id) else {
        return false;
    };
    let v_m = display_to_meters(v);
    if id == ids::GS_CFG_CELL_SIZE {
        match state.kind {
            GridKind::Square => state.square_cfg.cell_size = v_m.max(MIN_CELL_SIZE_M),
            GridKind::Hex => state.hex_cfg.cell_size = v_m.max(MIN_CELL_SIZE_M),
            GridKind::StaggeredSquare => {
                state.staggered_square_cfg.cell_w = v_m.max(MIN_CELL_SIZE_M);
                state.staggered_square_cfg.cell_h = v_m.max(MIN_CELL_SIZE_M);
            }
            GridKind::StaggeredHex => {
                state.staggered_hex_cfg.hex.cell_size = v_m.max(MIN_CELL_SIZE_M)
            }
            GridKind::Tri => state.tri_cfg.edge_length = v_m.max(MIN_CELL_SIZE_M),
            GridKind::Chunks => state.chunks_cfg.cell_size = v_m.max(MIN_CELL_SIZE_M),
            _ => {}
        }
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_W {
        state.iso_cfg.tile_w = v_m.max(MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_H {
        state.iso_cfg.tile_h = v_m.max(MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_QT_MAX_PER_LEAF {
        state.quadtree_cfg.max_points_per_leaf = (v as usize).max(1);
        return true;
    }
    if id == ids::GS_CFG_QT_MAX_DEPTH {
        state.quadtree_cfg.max_depth = (v as u32).max(1);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_SEED_COUNT {
        state.voronoi_cfg.seed_count = (v as usize).max(3);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_RNG_SEED {
        state.voronoi_cfg.rng_seed = v as u64;
        return true;
    }
    if id == ids::GS_CFG_VORONOI_LLOYD_ITERS {
        state.voronoi_cfg.lloyd_iterations = (v as u32).min(8);
        return true;
    }
    if id == ids::GS_CFG_CHUNKS_SIZE {
        state.chunks_cfg.chunk_size_cells = (v as u32).max(1);
        return true;
    }
    if id == ids::GS_CFG_ORIGIN_X {
        write_active_origin_x(state, v_m);
        return true;
    }
    if id == ids::GS_CFG_ORIGIN_Y {
        write_active_origin_y(state, v_m);
        return true;
    }
    if id == ids::GS_CFG_SPACING_MAJOR && state.kind == GridKind::Square {
        state.square_cfg.spacing_major = v_m.max(state.square_cfg.cell_size);
        return true;
    }
    let to_u8 = |v: f64| v.clamp(0.0, 255.0) as u8; // LITERAL-PX-OK: sRGB byte upper-bound (color component, not a UI metric)
    if id == ids::GS_CFG_COLOR_R {
        state.color_rgba[0] = to_u8(v);
        return true;
    }
    if id == ids::GS_CFG_COLOR_G {
        state.color_rgba[1] = to_u8(v);
        return true;
    }
    if id == ids::GS_CFG_COLOR_B {
        state.color_rgba[2] = to_u8(v);
        return true;
    }
    if id == ids::GS_CFG_SNAP_SUBDIVISIONS {
        state.snap_subdivisions = (v as u32).clamp(1, 64);
        return true;
    }
    if id == ids::GS_CFG_SNAP_MAGNETISM_RADIUS {
        state.snap_magnetism_radius = v_m.clamp(0.0, 10.0); // LITERAL-PX-OK: magnetism radius upper-bound in meters (world-space)
        return true;
    }
    if id == ids::GS_PROBE_A_X {
        state.probe_a[0] = v_m;
        return true;
    }
    if id == ids::GS_PROBE_A_Y {
        state.probe_a[1] = v_m;
        return true;
    }
    if id == ids::GS_PROBE_B_X {
        state.probe_b[0] = v_m;
        return true;
    }
    if id == ids::GS_PROBE_B_Y {
        state.probe_b[1] = v_m;
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MIN_X {
        state.quadtree_cfg.bounds.min[0] = v_m;
        state.quadtree_cfg.bounds.max[0] =
            state.quadtree_cfg.bounds.max[0].max(v_m + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MIN_Y {
        state.quadtree_cfg.bounds.min[1] = v_m;
        state.quadtree_cfg.bounds.max[1] =
            state.quadtree_cfg.bounds.max[1].max(v_m + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_X {
        state.quadtree_cfg.bounds.max[0] =
            v_m.max(state.quadtree_cfg.bounds.min[0] + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_Y {
        state.quadtree_cfg.bounds.max[1] =
            v_m.max(state.quadtree_cfg.bounds.min[1] + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_QT_DEMO_POINTS {
        state.quadtree_cfg.demo_point_count = (v as usize).min(4096);
        return true;
    }
    if id == ids::GS_CFG_QT_DEMO_SEED {
        state.quadtree_cfg.demo_rng_seed = v as u64;
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_X {
        state.voronoi_cfg.bounds.min[0] = v_m;
        state.voronoi_cfg.bounds.max[0] =
            state.voronoi_cfg.bounds.max[0].max(v_m + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_Y {
        state.voronoi_cfg.bounds.min[1] = v_m;
        state.voronoi_cfg.bounds.max[1] =
            state.voronoi_cfg.bounds.max[1].max(v_m + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_X {
        state.voronoi_cfg.bounds.max[0] =
            v_m.max(state.voronoi_cfg.bounds.min[0] + MIN_CELL_SIZE_M);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_Y {
        state.voronoi_cfg.bounds.max[1] =
            v_m.max(state.voronoi_cfg.bounds.min[1] + MIN_CELL_SIZE_M);
        return true;
    }
    false
}

fn write_active_origin_x(state: &mut GridSnapState, v: f32) {
    match state.kind {
        GridKind::Square => state.square_cfg.origin[0] = v,
        GridKind::Hex => state.hex_cfg.origin[0] = v,
        GridKind::Iso => state.iso_cfg.origin[0] = v,
        GridKind::StaggeredSquare => state.staggered_square_cfg.origin[0] = v,
        GridKind::StaggeredHex => state.staggered_hex_cfg.hex.origin[0] = v,
        GridKind::Tri => state.tri_cfg.origin[0] = v,
        GridKind::Chunks => state.chunks_cfg.origin[0] = v,
        GridKind::Quadtree | GridKind::Voronoi => {}
    }
}

fn write_active_origin_y(state: &mut GridSnapState, v: f32) {
    match state.kind {
        GridKind::Square => state.square_cfg.origin[1] = v,
        GridKind::Hex => state.hex_cfg.origin[1] = v,
        GridKind::Iso => state.iso_cfg.origin[1] = v,
        GridKind::StaggeredSquare => state.staggered_square_cfg.origin[1] = v,
        GridKind::StaggeredHex => state.staggered_hex_cfg.hex.origin[1] = v,
        GridKind::Tri => state.tri_cfg.origin[1] = v,
        GridKind::Chunks => state.chunks_cfg.origin[1] = v,
        GridKind::Quadtree | GridKind::Voronoi => {}
    }
}

fn active_hex_cfg_mut(state: &mut GridSnapState) -> &mut HexCfg {
    if state.kind == GridKind::StaggeredHex {
        &mut state.staggered_hex_cfg.hex
    } else {
        &mut state.hex_cfg
    }
}

fn apply_click(state: &mut GridSnapState, id: NodeId) -> bool {
    // GS_CLOSE handled in `apply_event` above (early dispatch
    // alongside TOPBAR_GRID_SETTINGS).
    for (kind, opt_id) in kind_option_ids_in_order() {
        if id == opt_id {
            state.kind = kind;
            return true;
        }
    }
    if id == ids::GS_SNAP_CENTER {
        state.snap_target = SnapTarget::Center;
        return true;
    }
    if id == ids::GS_SNAP_INTERSECTION {
        state.snap_target = SnapTarget::Intersection;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CORNER {
        state.snap_target = SnapTarget::Corner;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION {
        state.snap_target = SnapTarget::CenterAndIntersection;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS {
        state.snap_target = SnapTarget::CenterIntersectionAndCorners;
        return true;
    }
    if id == ids::GS_CFG_NEIGHBORHOOD_4 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Von4);
        return true;
    }
    if id == ids::GS_CFG_NEIGHBORHOOD_8 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Moore8);
        return true;
    }
    if id == ids::GS_CFG_HEX_POINTY {
        active_hex_cfg_mut(state).orientation = HexOrientation::Pointy;
        return true;
    }
    if id == ids::GS_CFG_HEX_FLAT {
        active_hex_cfg_mut(state).orientation = HexOrientation::Flat;
        return true;
    }
    if id == ids::GS_CFG_HEX_OFFSET_ODDR {
        active_hex_cfg_mut(state).offset_variant = HexOffset::OddR;
        return true;
    }
    if id == ids::GS_CFG_HEX_OFFSET_EVENR {
        active_hex_cfg_mut(state).offset_variant = HexOffset::EvenR;
        return true;
    }
    if id == ids::GS_CFG_HEX_OFFSET_ODDQ {
        active_hex_cfg_mut(state).offset_variant = HexOffset::OddQ;
        return true;
    }
    if id == ids::GS_CFG_HEX_OFFSET_EVENQ {
        active_hex_cfg_mut(state).offset_variant = HexOffset::EvenQ;
        return true;
    }
    if id == ids::GS_CFG_STAGGER_PARITY_ODD {
        state.staggered_square_cfg.parity = StaggerParity::OddRows;
        return true;
    }
    if id == ids::GS_CFG_STAGGER_PARITY_EVEN {
        state.staggered_square_cfg.parity = StaggerParity::EvenRows;
        return true;
    }
    if id == ids::GS_CFG_TRI_EDGE3 {
        state.tri_cfg.neighborhood = TriNeighborhood::Edge3;
        return true;
    }
    if id == ids::GS_CFG_TRI_VERTEX12 {
        state.tri_cfg.neighborhood = TriNeighborhood::Vertex12;
        return true;
    }
    if id == ids::GS_LAYER_IN_FRONT {
        state.grid_in_front = true;
        return true;
    }
    if id == ids::GS_LAYER_BEHIND {
        state.grid_in_front = false;
        return true;
    }
    if id == ids::GS_COLOR_PICKER {
        return true;
    }
    if id == ids::GS_CFG_VORONOI_RESEED {
        state.voronoi_cfg.rng_seed = state.voronoi_cfg.rng_seed.wrapping_add(2_654_435_761);
        return true;
    }
    false
}
