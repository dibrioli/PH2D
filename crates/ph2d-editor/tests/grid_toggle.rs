//! ADR-0025 M14.4b — `HeroScreen` grid toggle integration test.
//!
//! Verifies the `Show Grid` context-menu entry flips
//! `grid_visible` and that `set_grid_view` round-trips a
//! [`GridView`].

use ph2d_editor::HeroScreen;
use ph2d_editor::NodeId;
use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::screens::hero::ids::CTX_MENU_SHOW_GRID;
use ph2d_editor::zones::Rect;
use ph2d_editor::{GridConfig, GridView};

fn sample_view() -> GridView {
    GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(200.0, 150.0, 400.0, 300.0),
    }
}

#[test]
fn grid_visible_starts_true_view_starts_none() {
    let hero = HeroScreen::new(NodeId(1));
    assert!(hero.grid_visible);
    assert!(hero.grid_view.is_none());
}

#[test]
fn set_grid_view_round_trips() {
    let mut hero = HeroScreen::new(NodeId(1));
    let view = sample_view();
    hero.set_grid_view(Some(view));
    assert!(hero.grid_view.is_some());
    let got = hero.grid_view.unwrap();
    assert_eq!(got.camera_height_world, view.camera_height_world);
    assert_eq!(got.window_w, view.window_w);
    hero.set_grid_view(None);
    assert!(hero.grid_view.is_none());
}

#[test]
fn show_grid_menu_entry_toggles_visibility() {
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(hero.grid_visible);
    let consumed = hero.apply_event(WidgetEvent::Click(CTX_MENU_SHOW_GRID));
    assert!(consumed);
    assert!(!hero.grid_visible);
    // Second click toggles back ON.
    let consumed = hero.apply_event(WidgetEvent::Click(CTX_MENU_SHOW_GRID));
    assert!(consumed);
    assert!(hero.grid_visible);
}

/// M14.4b.bis: View button (TOOL_HOME) advances through 4 modes;
/// landing on mode 3 ("Zero") raises `camera_reset_pending` so the
/// host knows to reset its Camera2d. Modes 0/1/2 are placeholders
/// (frame selected / camera / all) and do NOT raise the flag.
#[test]
fn view_button_zero_mode_raises_camera_reset() {
    use ph2d_editor::screens::hero::ids::TOOL_HOME;

    let mut hero = HeroScreen::new(NodeId(1));
    assert!(!hero.camera_reset_pending);

    // Click 1: 0 → 1 (Camera). No reset.
    hero.apply_event(WidgetEvent::Click(TOOL_HOME));
    assert!(!hero.camera_reset_pending);
    // Click 2: 1 → 2 (All). No reset.
    hero.apply_event(WidgetEvent::Click(TOOL_HOME));
    assert!(!hero.camera_reset_pending);
    // Click 3: 2 → 3 (Zero). Reset flag raised.
    hero.apply_event(WidgetEvent::Click(TOOL_HOME));
    assert!(hero.camera_reset_pending);
    // Caller is responsible for clearing after acting.
    hero.camera_reset_pending = false;
    // Click 4: 3 → 0 (Selected). Cycle wraps; no reset.
    hero.apply_event(WidgetEvent::Click(TOOL_HOME));
    assert!(!hero.camera_reset_pending);
}

#[test]
fn grid_config_mut_exposes_spacing() {
    let mut hero = HeroScreen::new(NodeId(1));
    let cfg = hero.grid_config_mut();
    cfg.spacing_minor = 0.5;
    cfg.spacing_major = 2.5;
    // Defaults of the original GridConfig should not be 0.5/2.5 — sanity.
    let default = GridConfig::default();
    assert!(default.spacing_minor != 0.5);
    // Re-read after mutation: state was committed.
    assert_eq!(hero.grid_config.spacing_minor, 0.5);
    assert_eq!(hero.grid_config.spacing_major, 2.5);
}
