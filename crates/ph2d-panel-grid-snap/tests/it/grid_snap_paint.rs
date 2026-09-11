//! ADR-0029 Phase C.4 — Grid Snap panel integration tests.
//!
//! Migrated from `crates/ph2d-editor-core/src/screens/hero/tests.rs`
//! after the panel chrome moved out of editor-core (typed `Panel`
//! migration closing Phase C). Visibility flag now lives in
//! `HeroScreen::panel_visibility` (keyed `"grid_snap"`); the persisted
//! rect remains on `GridSnapState::panel_rect` (canvas renderer still
//! reads other GridSnapState fields).
//!
//! Covers:
//!
//! 1. After `paint_hero_screen` the panel publishes `content_h` and
//!    `visible_h` to the store, the cursor over the panel center
//!    resolves to `GS_PANEL` via `panel_at`, and a wheel dispatch at
//!    that center advances `panel_scroll(GS_PANEL)`.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_grid_snap::GridSnapPanel;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
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
    let mut hero = HeroScreen::new(NodeId(1));
    // Phase C.4 — visibility flows through the canonical map.
    hero.panel_visibility.insert(GridSnapPanel::ID, true);
    hero
}

fn ipad12_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

#[test]
fn grid_settings_publishes_scroll_bounds_and_wheel_advances_scroll() {
    let mut hero = setup_hero();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let gs_id = ids::GS_PANEL;
    let content_h = hero
        .store
        .panel_content_h(gs_id)
        .expect("GS_PANEL content_h must be published after paint");
    let visible_h = hero
        .store
        .panel_visible_h(gs_id)
        .expect("GS_PANEL visible_h must be published after paint");
    assert!(
        content_h > 0.0,
        "grid panel content_h should be positive, got {content_h}"
    );
    assert!(
        visible_h > 0.0,
        "grid panel visible_h should be positive, got {visible_h}"
    );
    assert!(
        content_h > visible_h,
        "grid panel should overflow (content_h={content_h} > visible_h={visible_h}) \
         so the wheel has somewhere to scroll to"
    );
    let panel_rect = hero
        .store
        .panel_rect(gs_id)
        .expect("GS_PANEL rect must be registered for panel_at");
    let cx = panel_rect.x + panel_rect.w * 0.5;
    let cy = panel_rect.y + panel_rect.h * 0.5;
    assert_eq!(
        hero.store.panel_at(cx, cy),
        Some(gs_id),
        "cursor over grid-panel center should resolve to GS_PANEL \
         (got {:?})",
        hero.store.panel_at(cx, cy)
    );
    let arena = bumpalo::Bump::new();
    let before = hero.store.panel_scroll(gs_id);
    let _ = ph2d_editor_core::interaction::dispatch_wheel(
        &mut hero.store,
        ph2d_host::WheelEvent {
            x: cx,
            y: cy,
            delta_x: 0.0,
            delta_y: -40.0,
            modifiers: ph2d_host::Modifiers::default(),
            timestamp_ns: 0,
        },
        &arena,
    );
    let after = hero.store.panel_scroll(gs_id);
    assert!(
        after > before,
        "wheel down on Grid Settings should increase panel_scroll \
         (before={before}, after={after})"
    );
}
