//! ADR-0029 Phase C.4 — apply_event behavior tests for the typed
//! Grid Snap panel.
//!
//! Migrated from `crates/ph2d-editor-core/src/grid_snap/panel/mod_tests.rs`
//! after the panel chrome moved out of editor-core. The unit-level
//! behavior tests covered the canvas-renderer `GridSnapState` mutations
//! that flow through the panel — they now drive the typed
//! `HeroScreen::apply_event` path (with `GridSnapPanel` installed) so
//! they end-to-end-verify the new dispatch.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState, ids as gs_ids};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggerParity;
use ph2d_panel_grid_snap::GridSnapPanel;
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<GridSnapPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn setup_hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    HeroScreen::new(NodeId(1))
}

fn snap(hero: &HeroScreen) -> &GridSnapState {
    &hero.grid.snap_state
}

#[test]
fn close_button_consumed() {
    let mut hero = setup_hero();
    hero.panel_visibility.insert(GridSnapPanel::ID, true);
    let consumed = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CLOSE));
    assert!(consumed, "GS_CLOSE click should be consumed");
}

#[test]
fn topbar_grid_settings_toggles_visibility() {
    let mut hero = setup_hero();
    assert!(!hero.is_panel_visible(GridSnapPanel::ID));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::TOPBAR_GRID_SETTINGS));
    assert!(consumed);
    assert!(hero.is_panel_visible(GridSnapPanel::ID));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::TOPBAR_GRID_SETTINGS));
    assert!(consumed);
    assert!(!hero.is_panel_visible(GridSnapPanel::ID));
}

#[test]
fn kind_option_click_sets_active_kind() {
    let mut hero = setup_hero();
    assert_eq!(snap(&hero).kind, GridKind::Square);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_HEX));
    assert_eq!(snap(&hero).kind, GridKind::Hex);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_VORONOI));
    assert_eq!(snap(&hero).kind, GridKind::Voronoi);
}

#[test]
fn snap_toggle_writes_state() {
    let mut hero = setup_hero();
    assert!(!snap(&hero).snap_enabled);
    if let Some(InteractiveState::Toggle { on, .. }) = hero.store.get_mut(gs_ids::GS_SNAP_ENABLED) {
        *on = true;
    }
    let _ = hero.apply_event(WidgetEvent::Toggled(gs_ids::GS_SNAP_ENABLED));
    assert!(snap(&hero).snap_enabled);
}

#[test]
fn overlay_toggle_writes_state() {
    let mut hero = setup_hero();
    let initial = snap(&hero).show_overlay;
    if let Some(InteractiveState::Toggle { on, .. }) = hero.store.get_mut(gs_ids::GS_SHOW_OVERLAY) {
        *on = !*on;
    }
    let _ = hero.apply_event(WidgetEvent::Toggled(gs_ids::GS_SHOW_OVERLAY));
    assert_eq!(snap(&hero).show_overlay, !initial);
}

#[test]
fn number_changed_writes_to_square_cell_size() {
    let mut hero = setup_hero();
    if let Some(InteractiveState::NumberInput {
        value,
        last_committed,
        buffer,
        ..
    }) = hero.store.get_mut(gs_ids::GS_CFG_CELL_SIZE)
    {
        *value = 4.5;
        *last_committed = 4.5;
        *buffer = "4.5".to_string();
    }
    let _ = hero.apply_event(WidgetEvent::ValueChanged(gs_ids::GS_CFG_CELL_SIZE));
    assert!((snap(&hero).square_cfg.cell_size - 4.5).abs() < 1e-5);
}

#[test]
fn neighborhood_options_set_value() {
    let mut hero = setup_hero();
    assert_eq!(
        snap(&hero).square_cfg.neighborhood,
        SquareNeighborhood::Von4
    );
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_NEIGHBORHOOD_8));
    assert_eq!(
        snap(&hero).square_cfg.neighborhood,
        SquareNeighborhood::Moore8
    );
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_NEIGHBORHOOD_4));
    assert_eq!(
        snap(&hero).square_cfg.neighborhood,
        SquareNeighborhood::Von4
    );
}

#[test]
fn hex_orientation_options_set_value() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_HEX));
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_HEX_FLAT));
    assert_eq!(snap(&hero).hex_cfg.orientation, HexOrientation::Flat);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_HEX_POINTY));
    assert_eq!(snap(&hero).hex_cfg.orientation, HexOrientation::Pointy);
}

#[test]
fn hex_offset_options_set_value() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_HEX));
    for (id, expected) in [
        (gs_ids::GS_CFG_HEX_OFFSET_EVENR, HexOffset::EvenR),
        (gs_ids::GS_CFG_HEX_OFFSET_ODDQ, HexOffset::OddQ),
        (gs_ids::GS_CFG_HEX_OFFSET_EVENQ, HexOffset::EvenQ),
        (gs_ids::GS_CFG_HEX_OFFSET_ODDR, HexOffset::OddR),
    ] {
        let _ = hero.apply_event(WidgetEvent::Click(id));
        assert_eq!(snap(&hero).hex_cfg.offset_variant, expected);
    }
}

#[test]
fn hex_orientation_routes_to_staggered_hex_when_active() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_STAGGERED_HEX));
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_HEX_FLAT));
    assert_eq!(
        snap(&hero).staggered_hex_cfg.hex.orientation,
        HexOrientation::Flat
    );
    assert_eq!(snap(&hero).hex_cfg.orientation, HexOrientation::Pointy);
}

#[test]
fn stagger_parity_options_set_value() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_STAGGERED_SQ));
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_STAGGER_PARITY_EVEN));
    assert_eq!(
        snap(&hero).staggered_square_cfg.parity,
        StaggerParity::EvenRows
    );
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_STAGGER_PARITY_ODD));
    assert_eq!(
        snap(&hero).staggered_square_cfg.parity,
        StaggerParity::OddRows
    );
}

#[test]
fn layer_options_flip_grid_in_front() {
    let mut hero = setup_hero();
    assert!(snap(&hero).grid_in_front);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_LAYER_BEHIND));
    assert!(!snap(&hero).grid_in_front);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_LAYER_IN_FRONT));
    assert!(snap(&hero).grid_in_front);
}

#[test]
fn snap_target_options_set_value() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_SNAP_INTERSECTION));
    assert_eq!(snap(&hero).snap_target, SnapTarget::Intersection);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_SNAP_TARGET_OPT_CORNER));
    assert_eq!(snap(&hero).snap_target, SnapTarget::Corner);
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_SNAP_CENTER));
    assert_eq!(snap(&hero).snap_target, SnapTarget::Center);
}

#[test]
fn color_b_writes_state() {
    let mut hero = setup_hero();
    if let Some(InteractiveState::NumberInput { value, .. }) =
        hero.store.get_mut(gs_ids::GS_CFG_COLOR_B)
    {
        *value = 200.0;
    }
    let _ = hero.apply_event(WidgetEvent::ValueChanged(gs_ids::GS_CFG_COLOR_B));
    assert_eq!(snap(&hero).color_rgba[2], 200);
}

#[test]
fn subdivisions_clamps_to_unit_floor() {
    let mut hero = setup_hero();
    if let Some(InteractiveState::NumberInput { value, .. }) =
        hero.store.get_mut(gs_ids::GS_CFG_SNAP_SUBDIVISIONS)
    {
        *value = 0.0;
    }
    let _ = hero.apply_event(WidgetEvent::ValueChanged(gs_ids::GS_CFG_SNAP_SUBDIVISIONS));
    assert_eq!(snap(&hero).snap_subdivisions, 1);
}

#[test]
fn voronoi_reseed_changes_rng_seed() {
    let mut hero = setup_hero();
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_KIND_OPT_VORONOI));
    let before = snap(&hero).voronoi_cfg.rng_seed;
    let _ = hero.apply_event(WidgetEvent::Click(gs_ids::GS_CFG_VORONOI_RESEED));
    assert_ne!(snap(&hero).voronoi_cfg.rng_seed, before);
}

#[test]
fn opacity_slider_value_changed_clamps_to_unit() {
    let mut hero = setup_hero();
    if let Some(InteractiveState::Slider { value, .. }) =
        hero.store.get_mut(gs_ids::GS_OPACITY_SLIDER)
    {
        *value = 1.5;
    }
    let _ = hero.apply_event(WidgetEvent::ValueChanged(gs_ids::GS_OPACITY_SLIDER));
    assert!((snap(&hero).opacity - 1.0).abs() < 1e-5);
}
