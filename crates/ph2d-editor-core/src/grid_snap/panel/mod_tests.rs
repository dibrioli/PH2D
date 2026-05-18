use super::*;

fn populated_store() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(32);
    populate(&mut store);
    store
}

fn flip_toggle(store: &mut WidgetStore, id: crate::NodeId) {
    if let Some(InteractiveState::Toggle { on, .. }) = store.get_mut(id) {
        *on = !*on;
    }
}

#[test]
fn paint_hex_segmented_buttons_register_distinct_rects() {
    // Regression for "click Flat does nothing / Offset stuck on
    // OddR" reported 2026-05-15: each of the 6 hex segmented
    // buttons (Pointy, Flat, OddR, EvenR, OddQ, EvenQ) must have
    // its own non-overlapping rect in hit_index after a paint.
    use crate::interaction::HitIndex;
    let mut text_system = TextSystem::default();
    let mut scene = VectorScene::default();
    let mut hit_index = HitIndex::default();
    let store = populated_store();
    let state = GridSnapState {
        kind: GridKind::Hex,
        panel_visible: true,
        ..Default::default()
    };
    let panel_rect = Rect::new(100.0, 50.0, 304.0, 800.0);
    paint(
        panel_rect,
        &mut scene,
        &mut text_system,
        Theme::default(),
        &mut hit_index,
        &store,
        &state,
    );

    let ids_under_test = [
        ("Pointy", ids::GS_CFG_HEX_POINTY),
        ("Flat", ids::GS_CFG_HEX_FLAT),
        ("OddR", ids::GS_CFG_HEX_OFFSET_ODDR),
        ("EvenR", ids::GS_CFG_HEX_OFFSET_EVENR),
        ("OddQ", ids::GS_CFG_HEX_OFFSET_ODDQ),
        ("EvenQ", ids::GS_CFG_HEX_OFFSET_EVENQ),
    ];
    let mut rects: Vec<(&str, crate::NodeId, Rect)> = Vec::new();
    for (label, id) in ids_under_test {
        let r = hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("{label} ({id:?}) not registered in hit_index"));
        rects.push((label, id, r));
    }
    // No two buttons may share the same rect, and clicking the
    // center of each rect must hit the matching id.
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            assert_ne!(
                rects[i].2, rects[j].2,
                "{} and {} share rect {:?}",
                rects[i].0, rects[j].0, rects[i].2
            );
        }
        let (label, id, r) = rects[i];
        let cx = r.x + r.w * 0.5;
        let cy = r.y + r.h * 0.5;
        let hit = hit_index.hit(cx, cy);
        assert_eq!(
            hit,
            Some(id),
            "click on {label} center ({cx}, {cy}) hit {hit:?}, expected {id:?}"
        );
    }
}

#[test]
fn apply_event_close_clears_visible() {
    let mut s = GridSnapState {
        panel_visible: true,
        ..Default::default()
    };
    let store = populated_store();
    assert!(apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CLOSE),
        &store
    ));
    assert!(!s.panel_visible);
}

#[test]
fn apply_event_kind_option_click_sets_active_kind() {
    let mut s = GridSnapState::default();
    let store = populated_store();
    assert_eq!(s.kind, GridKind::Square);
    apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_HEX), &store);
    assert_eq!(s.kind, GridKind::Hex);
    apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_VORONOI), &store);
    assert_eq!(s.kind, GridKind::Voronoi);
}

#[test]
fn apply_event_chip_click_is_passthrough_for_dispatcher() {
    // The dispatcher handles Dropdown.open toggling on chip
    // clicks — apply_event must NOT mutate kind on a chip click.
    let mut s = GridSnapState::default();
    let store = populated_store();
    let before = s.kind;
    apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_DROPDOWN), &store);
    assert_eq!(s.kind, before);
}

#[test]
fn apply_event_snap_toggle_via_toggled() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    flip_toggle(&mut store, ids::GS_SNAP_ENABLED);
    apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SNAP_ENABLED), &store);
    assert!(s.snap_enabled);
}

#[test]
fn apply_event_overlay_toggle_via_toggled() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    let initial = s.show_overlay;
    flip_toggle(&mut store, ids::GS_SHOW_OVERLAY);
    apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SHOW_OVERLAY), &store);
    assert_eq!(s.show_overlay, !initial);
}

#[test]
fn apply_event_unrelated_id_returns_false() {
    let mut s = GridSnapState::default();
    let store = populated_store();
    let unrelated = crate::NodeId(42);
    assert!(!apply_event(&mut s, WidgetEvent::Click(unrelated), &store));
}

#[test]
fn apply_event_number_changed_writes_to_square_cell_size() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    // Set the store's value as the dispatcher would, then fire
    // ValueChanged.
    if let Some(InteractiveState::NumberInput {
        value,
        last_committed,
        buffer,
        ..
    }) = store.get_mut(ids::GS_CFG_CELL_SIZE)
    {
        *value = 4.5;
        *last_committed = 4.5;
        *buffer = "4.5".to_string();
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_CFG_CELL_SIZE),
        &store,
    );
    assert!((s.square_cfg.cell_size - 4.5).abs() < 1e-5);
}

#[test]
fn apply_event_neighborhood_options_set_value() {
    // Post-segmented-button semantics: each option id explicitly
    // SETS its value (Von4 / Moore8), no cycling. Idempotent.
    let mut s = GridSnapState::default();
    let store = populated_store();
    assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
    apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_8),
        &store,
    );
    assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Moore8);
    apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_4),
        &store,
    );
    assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
}

#[test]
fn apply_event_hex_orientation_options_set_value() {
    let mut s = GridSnapState {
        kind: GridKind::Hex,
        ..Default::default()
    };
    let store = populated_store();
    // Default is Pointy. Clicking Flat sets it; clicking Pointy
    // again restores it. No cycle behavior — each option lands
    // on its own value.
    apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_FLAT), &store);
    assert_eq!(s.hex_cfg.orientation, HexOrientation::Flat);
    apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_POINTY), &store);
    assert_eq!(s.hex_cfg.orientation, HexOrientation::Pointy);
}

#[test]
fn apply_event_hex_offset_options_set_value() {
    let mut s = GridSnapState {
        kind: GridKind::Hex,
        ..Default::default()
    };
    let store = populated_store();
    for (id, expected) in [
        (ids::GS_CFG_HEX_OFFSET_EVENR, HexOffset::EvenR),
        (ids::GS_CFG_HEX_OFFSET_ODDQ, HexOffset::OddQ),
        (ids::GS_CFG_HEX_OFFSET_EVENQ, HexOffset::EvenQ),
        (ids::GS_CFG_HEX_OFFSET_ODDR, HexOffset::OddR),
    ] {
        apply_event(&mut s, WidgetEvent::Click(id), &store);
        assert_eq!(s.hex_cfg.offset_variant, expected);
    }
}

#[test]
fn apply_event_hex_orientation_routes_to_staggered_hex_when_active() {
    let mut s = GridSnapState {
        kind: GridKind::StaggeredHex,
        ..Default::default()
    };
    let store = populated_store();
    apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_FLAT), &store);
    assert_eq!(
        s.staggered_hex_cfg.hex.orientation,
        HexOrientation::Flat,
        "StaggeredHex should receive the orientation update"
    );
    // Plain hex_cfg untouched.
    assert_eq!(s.hex_cfg.orientation, HexOrientation::Pointy);
}

#[test]
fn apply_event_stagger_parity_options_set_value() {
    let mut s = GridSnapState {
        kind: GridKind::StaggeredSquare,
        ..Default::default()
    };
    let store = populated_store();
    apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CFG_STAGGER_PARITY_EVEN),
        &store,
    );
    assert_eq!(s.staggered_square_cfg.parity, StaggerParity::EvenRows);
    apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CFG_STAGGER_PARITY_ODD),
        &store,
    );
    assert_eq!(s.staggered_square_cfg.parity, StaggerParity::OddRows);
}

#[test]
fn apply_event_layer_options_flip_grid_in_front() {
    let mut s = GridSnapState::default();
    let store = populated_store();
    assert!(s.grid_in_front, "default is In front");
    apply_event(&mut s, WidgetEvent::Click(ids::GS_LAYER_BEHIND), &store);
    assert!(!s.grid_in_front);
    apply_event(&mut s, WidgetEvent::Click(ids::GS_LAYER_IN_FRONT), &store);
    assert!(s.grid_in_front);
}

#[test]
fn apply_event_color_r_writes_state() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(ids::GS_CFG_COLOR_R) {
        *value = 200.0;
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_R),
        &store,
    );
    assert_eq!(s.color_rgba[0], 200);
    // Other channels untouched.
    let defaults = GridSnapState::default();
    assert_eq!(s.color_rgba[1], defaults.color_rgba[1]);
    assert_eq!(s.color_rgba[2], defaults.color_rgba[2]);
    assert_eq!(s.color_rgba[3], defaults.color_rgba[3]);
}

#[test]
fn apply_event_subdivisions_clamps_to_unit_floor() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    if let Some(InteractiveState::NumberInput { value, .. }) =
        store.get_mut(ids::GS_CFG_SNAP_SUBDIVISIONS)
    {
        *value = 0.0; // floor enforced to 1
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_CFG_SNAP_SUBDIVISIONS),
        &store,
    );
    assert_eq!(s.snap_subdivisions, 1);
}

#[test]
fn apply_event_subdivisions_actually_subdivides_snap() {
    // With cell_size=1 and subdivisions=2, snap_to_center should
    // pull to half-cell centers. World [0.3, 0.3] is in cell
    // (0, 0) of the sub-grid (cells span 0..0.5) → center
    // (0.25, 0.25), not (0.5, 0.5).
    let mut s = GridSnapState {
        snap_enabled: true,
        snap_target: SnapTarget::Center,
        snap_subdivisions: 2,
        ..Default::default()
    };
    let p = s.snap_world([0.3, 0.3], [0.0, 0.0]);
    assert!(
        (p[0] - 0.25).abs() < 1e-5 && (p[1] - 0.25).abs() < 1e-5,
        "expected [0.25, 0.25] with subdivisions=2; got {p:?}"
    );
}

#[test]
fn apply_event_probe_a_x_writes_state() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(ids::GS_PROBE_A_X) {
        *value = 7.5;
    }
    apply_event(&mut s, WidgetEvent::ValueChanged(ids::GS_PROBE_A_X), &store);
    assert!((s.probe_a[0] - 7.5).abs() < 1e-5);
}

#[test]
fn apply_event_qt_bounds_max_soft_clamps_to_min_plus_eps() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    // Try to set MAX below MIN — should snap up to min+eps.
    let min_x = s.quadtree_cfg.bounds.min[0];
    if let Some(InteractiveState::NumberInput { value, .. }) =
        store.get_mut(ids::GS_CFG_QT_BOUNDS_MAX_X)
    {
        *value = min_x as f64 - 5.0;
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_CFG_QT_BOUNDS_MAX_X),
        &store,
    );
    assert!(
        s.quadtree_cfg.bounds.max[0] > s.quadtree_cfg.bounds.min[0],
        "MAX must stay > MIN after clamp; got min={} max={}",
        s.quadtree_cfg.bounds.min[0],
        s.quadtree_cfg.bounds.max[0]
    );
}

#[test]
fn apply_event_color_clamps_above_255() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(ids::GS_CFG_COLOR_B) {
        *value = 999.0;
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_B),
        &store,
    );
    assert_eq!(s.color_rgba[2], 255);
}

#[test]
fn apply_event_voronoi_reseed_changes_rng_seed() {
    let mut s = GridSnapState {
        kind: GridKind::Voronoi,
        ..Default::default()
    };
    let store = populated_store();
    let before = s.voronoi_cfg.rng_seed;
    apply_event(
        &mut s,
        WidgetEvent::Click(ids::GS_CFG_VORONOI_RESEED),
        &store,
    );
    assert_ne!(s.voronoi_cfg.rng_seed, before);
}

#[test]
fn opacity_slider_value_changed_clamps_to_unit() {
    let mut s = GridSnapState::default();
    let mut store = populated_store();
    if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(ids::GS_OPACITY_SLIDER) {
        *value = 1.5;
    }
    apply_event(
        &mut s,
        WidgetEvent::ValueChanged(ids::GS_OPACITY_SLIDER),
        &store,
    );
    assert!((s.opacity - 1.0).abs() < 1e-5);
}
