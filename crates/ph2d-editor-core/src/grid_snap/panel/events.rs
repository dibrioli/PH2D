//! Event dispatch — apply_toggle, apply_value_changed, apply_click.
//!
//! Extracted from `panel/mod.rs` in Wave 2.5 PR 11.7a to bring the
//! parent module under the HR-18 hygiene cap. Same private surface
//! as before; re-exported by `mod.rs` so call sites are unchanged.

use super::*;

pub(crate) fn apply_toggle(
    state: &mut GridSnapState,
    id: crate::NodeId,
    store: &WidgetStore,
) -> bool {
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

pub(crate) fn apply_value_changed(
    state: &mut GridSnapState,
    id: crate::NodeId,
    store: &WidgetStore,
) -> bool {
    if id == ids::GS_OPACITY_SLIDER {
        if let Some((_, v)) = store.slider(id) {
            state.opacity = v.clamp(0.0, 1.0);
        }
        return true;
    }
    // NumberInput updates — pull value and write to the matching cfg
    // field of the ACTIVE kind. Cross-kind shared ids (CELL_SIZE,
    // NEIGHBORHOOD_4) write to the active kind's field; switching
    // kind re-reads from that kind's cfg on the next paint.
    let Some(v) = store.number_value(id) else {
        return false;
    };
    // For meter-domain fields, convert the user-typed display value
    // back to meters before writing into state. Integer / unitless
    // fields (subdivisions, max_per_leaf, RGB) continue using `v`
    // directly.
    let v_m = display_to_meters(v);
    if id == ids::GS_CFG_CELL_SIZE {
        match state.kind {
            GridKind::Square => state.square_cfg.cell_size = v_m.max(0.01),
            GridKind::Hex => state.hex_cfg.cell_size = v_m.max(0.01),
            GridKind::StaggeredSquare => {
                state.staggered_square_cfg.cell_w = v_m.max(0.01);
                state.staggered_square_cfg.cell_h = v_m.max(0.01);
            }
            GridKind::StaggeredHex => state.staggered_hex_cfg.hex.cell_size = v_m.max(0.01),
            GridKind::Tri => state.tri_cfg.edge_length = v_m.max(0.01),
            GridKind::Chunks => state.chunks_cfg.cell_size = v_m.max(0.01),
            _ => {}
        }
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_W {
        state.iso_cfg.tile_w = v_m.max(0.01);
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_H {
        state.iso_cfg.tile_h = v_m.max(0.01);
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
    // Universal origin offset — applies to the active kind's cfg.
    if id == ids::GS_CFG_ORIGIN_X {
        write_active_origin_x(state, v_m);
        return true;
    }
    if id == ids::GS_CFG_ORIGIN_Y {
        write_active_origin_y(state, v_m);
        return true;
    }
    // Major-line spacing — Square only (other kinds ignore the id).
    if id == ids::GS_CFG_SPACING_MAJOR && state.kind == GridKind::Square {
        state.square_cfg.spacing_major = v_m.max(state.square_cfg.cell_size);
        return true;
    }
    // Grid color RGB — clamp each channel to 0..=255 and write
    // into state.color_rgba. Alpha (index 3) is owned by the
    // opacity slider; leave it untouched.
    let to_u8 = |v: f64| v.clamp(0.0, 255.0) as u8;
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
    // Snap subdivisions — clamp to [1, 64] so sub-grid stays useful
    // (1 = no subdivision; 64 covers reasonable half/quarter use).
    if id == ids::GS_CFG_SNAP_SUBDIVISIONS {
        state.snap_subdivisions = (v as u32).clamp(1, 64);
        return true;
    }
    // Magnetism radius (meter-domain) — non-negative, soft upper
    // clamp at 10 m so typing a runaway value doesn't lock every
    // drag to a snap target. `0.0` disables the gate (back-compat
    // always-snap), matching the docs on `snap_magnetism_radius`.
    if id == ids::GS_CFG_SNAP_MAGNETISM_RADIUS {
        state.snap_magnetism_radius = v_m.clamp(0.0, 10.0);
        return true;
    }
    // Probe coords (world meters) — direct write after display→meters.
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
    // Quadtree bounds (4 floats) + demo controls. AABB::new debug-
    // asserts only ordering; we soft-clamp max ≥ min to avoid
    // degenerate boxes after edits.
    if id == ids::GS_CFG_QT_BOUNDS_MIN_X {
        state.quadtree_cfg.bounds.min[0] = v_m;
        state.quadtree_cfg.bounds.max[0] = state.quadtree_cfg.bounds.max[0].max(v_m + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MIN_Y {
        state.quadtree_cfg.bounds.min[1] = v_m;
        state.quadtree_cfg.bounds.max[1] = state.quadtree_cfg.bounds.max[1].max(v_m + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_X {
        state.quadtree_cfg.bounds.max[0] = v_m.max(state.quadtree_cfg.bounds.min[0] + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_Y {
        state.quadtree_cfg.bounds.max[1] = v_m.max(state.quadtree_cfg.bounds.min[1] + 0.01);
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
    // Voronoi bounds (same soft-clamp pattern as Quadtree).
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_X {
        state.voronoi_cfg.bounds.min[0] = v_m;
        state.voronoi_cfg.bounds.max[0] = state.voronoi_cfg.bounds.max[0].max(v_m + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_Y {
        state.voronoi_cfg.bounds.min[1] = v_m;
        state.voronoi_cfg.bounds.max[1] = state.voronoi_cfg.bounds.max[1].max(v_m + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_X {
        state.voronoi_cfg.bounds.max[0] = v_m.max(state.voronoi_cfg.bounds.min[0] + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_Y {
        state.voronoi_cfg.bounds.max[1] = v_m.max(state.voronoi_cfg.bounds.min[1] + 0.01);
        return true;
    }
    false
}

pub(crate) fn write_active_origin_x(state: &mut GridSnapState, v: f32) {
    match state.kind {
        GridKind::Square => state.square_cfg.origin[0] = v,
        GridKind::Hex => state.hex_cfg.origin[0] = v,
        GridKind::Iso => state.iso_cfg.origin[0] = v,
        GridKind::StaggeredSquare => state.staggered_square_cfg.origin[0] = v,
        GridKind::StaggeredHex => state.staggered_hex_cfg.hex.origin[0] = v,
        GridKind::Tri => state.tri_cfg.origin[0] = v,
        GridKind::Chunks => state.chunks_cfg.origin[0] = v,
        // Quadtree/Voronoi use `bounds: AABB` instead; origin is a
        // no-op for them.
        GridKind::Quadtree | GridKind::Voronoi => {}
    }
}

pub(crate) fn write_active_origin_y(state: &mut GridSnapState, v: f32) {
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

pub(crate) fn apply_click(state: &mut GridSnapState, id: crate::NodeId) -> bool {
    if id == ids::GS_CLOSE {
        state.panel_visible = false;
        return true;
    }
    // Kind dropdown option clicks — map option id → GridKind.
    // (Click on the chip itself opens the dropdown; the dispatcher
    // handles open/closed state on Dropdown widgets, so we don't
    // handle GS_KIND_DROPDOWN here.)
    for (kind, opt_id) in kind_option_ids_in_order() {
        if id == opt_id {
            state.kind = kind;
            return true;
        }
    }
    // Target group option clicks (5-button vertical stack).
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
    // Neighborhood group option clicks (Square family: Von4 / Moore8).
    if id == ids::GS_CFG_NEIGHBORHOOD_4 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Von4);
        return true;
    }
    if id == ids::GS_CFG_NEIGHBORHOOD_8 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Moore8);
        return true;
    }
    // Hex Orientation segmented group (sets value directly).
    if id == ids::GS_CFG_HEX_POINTY {
        active_hex_cfg_mut(state).orientation = HexOrientation::Pointy;
        return true;
    }
    if id == ids::GS_CFG_HEX_FLAT {
        active_hex_cfg_mut(state).orientation = HexOrientation::Flat;
        return true;
    }
    // Hex Offset segmented group (4 options, sets value directly).
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
    // Stagger parity segmented group.
    if id == ids::GS_CFG_STAGGER_PARITY_ODD {
        state.staggered_square_cfg.parity = StaggerParity::OddRows;
        return true;
    }
    if id == ids::GS_CFG_STAGGER_PARITY_EVEN {
        state.staggered_square_cfg.parity = StaggerParity::EvenRows;
        return true;
    }
    // Tri neighborhood group option clicks.
    if id == ids::GS_CFG_TRI_EDGE3 {
        state.tri_cfg.neighborhood = TriNeighborhood::Edge3;
        return true;
    }
    if id == ids::GS_CFG_TRI_VERTEX12 {
        state.tri_cfg.neighborhood = TriNeighborhood::Vertex12;
        return true;
    }
    // Layer-order segmented group (In front / Behind sprites).
    if id == ids::GS_LAYER_IN_FRONT {
        state.grid_in_front = true;
        return true;
    }
    if id == ids::GS_LAYER_BEHIND {
        state.grid_in_front = false;
        return true;
    }
    // Color swatch click — open the BlenderColorPicker bound to
    // GS_COLOR_PICKER. Same pattern as Inspector samples.
    if id == ids::GS_COLOR_PICKER {
        // No state mutation here; the dispatcher sets picker_target.
        return true;
    }
    if id == ids::GS_CFG_VORONOI_RESEED {
        // Bump rng_seed by an odd prime to land on a new pattern
        // without collapsing back to the SplitMix64 starting state
        // for any seen seed.
        state.voronoi_cfg.rng_seed = state.voronoi_cfg.rng_seed.wrapping_add(2_654_435_761);
        return true;
    }
    false
}
