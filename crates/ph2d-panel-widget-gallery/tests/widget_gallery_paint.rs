//! ADR-0029 Phase C.3 — Widget Gallery panel integration tests.
//!
//! Migrated from `crates/ph2d-editor-core/src/screens/hero/tests.rs`
//! after the panel moved out of editor-core (typed `Panel` migration).
//! Visibility flag now lives in `HeroScreen::panel_visibility`
//! (keyed `"widget_gallery"`); the persistent rect lives on
//! `WidgetGalleryState::rect`.
//!
//! Covers:
//!
//! 1. After `paint_hero_screen` the panel publishes `content_h` and
//!    `visible_h` to the store + a wheel dispatch at the gallery
//!    center advances `panel_scroll(GAL_PANEL)`.
//! 2. Right-click → "Create note" inside the gallery body keys a
//!    `NoteData` against `GAL_PANEL` (not `INSP_PANEL`).
//! 3. Right-click → "Yellow" on a gallery section header writes the
//!    `section_outline_color` for that section.
//!
//! The host-level `apply_showcase_event` handler in
//! `ph2d_editor_core::widget::showcase` actually owns the
//! `CTX_MENU_CREATE_NOTE` / `CTX_MENU_OUTLINE_*` paths; this test only
//! validates the end-to-end flow when the typed `WidgetGalleryPanel`
//! has painted the `panel_rect(GAL_PANEL)` and the visibility flag is
//! set via `host.set_panel_visible`.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::ids;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_widget_gallery::WidgetGalleryPanel;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<WidgetGalleryPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn setup_hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    // Phase C.3 — visibility now flows through the canonical map.
    hero.panel_visibility.insert(WidgetGalleryPanel::ID, true);
    hero
}

fn ipad12_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

/// Regression: the Widget Gallery must publish `content_h` /
/// `visible_h` to the store after painting so the wheel dispatch can
/// clamp the scroll bound on `GAL_PANEL`. Without this the user
/// reports "scroll doesn't work" — wheel events would either be
/// ignored (no panel match) or fail to advance (max_scroll = 0).
#[test]
fn gallery_publishes_scroll_bounds_after_paint() {
    let mut hero = setup_hero();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let content_h = hero
        .store
        .panel_content_h(ids::GAL_PANEL)
        .expect("GAL_PANEL content_h must be published after paint");
    let visible_h = hero
        .store
        .panel_visible_h(ids::GAL_PANEL)
        .expect("GAL_PANEL visible_h must be published after paint");
    assert!(
        content_h > 0.0,
        "gallery content_h should be positive (sections painted), got {content_h}"
    );
    assert!(
        visible_h > 0.0,
        "gallery visible_h should be positive (body region), got {visible_h}"
    );
    assert!(
        content_h > visible_h,
        "gallery should overflow (content_h={content_h} > visible_h={visible_h}) \
         so scroll has effect — otherwise wheel is a no-op"
    );
    let panel_rect = hero
        .store
        .panel_rect(ids::GAL_PANEL)
        .expect("GAL_PANEL rect must be registered for panel_at");
    let cx = panel_rect.x + panel_rect.w * 0.5;
    let cy = panel_rect.y + panel_rect.h * 0.5;
    assert_eq!(
        hero.store.panel_at(cx, cy),
        Some(ids::GAL_PANEL),
        "cursor over gallery center should resolve to GAL_PANEL"
    );
    let arena = bumpalo::Bump::new();
    let before = hero.store.panel_scroll(ids::GAL_PANEL);
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
    let after = hero.store.panel_scroll(ids::GAL_PANEL);
    assert!(
        after > before,
        "wheel down on gallery should increase panel_scroll \
         (before={before}, after={after})"
    );
}

/// Regression: right-clicking inside the gallery body → choosing
/// "Create note" must push a `NoteData` keyed on `GAL_PANEL` (NOT
/// `INSP_PANEL`) so the gallery renders it on the next frame. The
/// gallery is the canonical UI ground-truth for peripheral agents.
#[test]
fn gallery_create_note_targets_gal_panel() {
    let mut hero = setup_hero();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let gallery_rect = hero.store.panel_rect(ids::GAL_PANEL).unwrap();
    let cx = gallery_rect.x + gallery_rect.w * 0.5;
    let cy = gallery_rect.y + gallery_rect.h * 0.5;
    hero.store
        .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
            x: cx,
            y: cy,
            kind: ph2d_editor_core::interaction::ContextMenuKind::CreateNote {
                panel: ids::GAL_PANEL,
                before_section: None,
            },
        });
    // Mirror the real Down-on-menu-item path that snapshots the
    // request into `last_context_menu` before the Click fires.
    hero.store.close_context_menu();
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_CREATE_NOTE));
    assert!(consumed, "CTX_MENU_CREATE_NOTE click should be consumed");
    assert_eq!(
        hero.store.notes_for_panel(ids::GAL_PANEL).len(),
        1,
        "exactly one note should be pushed against GAL_PANEL"
    );
    assert_eq!(
        hero.store.notes_for_panel(ids::INSP_PANEL).len(),
        0,
        "INSP_PANEL should be untouched — the gallery's note must \
         not leak into the live Inspector"
    );
}

/// Regression: right-clicking on a gallery section header → choosing
/// a color must write `section_outline_color` so the gallery's next
/// paint draws the colored ring around that section's body.
#[test]
fn gallery_section_outline_color_writes_through() {
    let mut hero = setup_hero();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    hero.store
        .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ph2d_editor_core::interaction::ContextMenuKind::SectionOutline {
                section: ids::INSP_SECTION_INPUTS,
            },
        });
    hero.store.close_context_menu();
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_OUTLINE_0));
    assert!(consumed, "CTX_MENU_OUTLINE_0 click should be consumed");
    assert_eq!(
        hero.store.section_outline_color(ids::INSP_SECTION_INPUTS),
        Some(0),
        "Inputs section should have outline color 0 (Yellow) set"
    );
}
