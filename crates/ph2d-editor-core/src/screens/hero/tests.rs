use super::*;
use crate::widget::ButtonState;

fn ipad12_viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

#[test]
fn layout_top_bar_inset_from_edge() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    assert!((layout.top_bar.x - style::EDGE_PAD).abs() < f32::EPSILON);
    assert!((layout.top_bar.h - style::TOPBAR_H).abs() < f32::EPSILON);
}

#[test]
fn layout_left_rail_below_top_bar() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    assert!(layout.left_rail.y > layout.top_bar.y + layout.top_bar.h);
    assert!((layout.left_rail.w - style::RAIL_W).abs() < f32::EPSILON);
}

#[test]
fn layout_hierarchy_after_rail_by_default() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    assert!(layout.hierarchy.x > layout.left_rail.x + layout.left_rail.w);
    assert!((layout.hierarchy.w - style::HIERARCHY_W).abs() < f32::EPSILON);
}

#[test]
fn layout_inspector_pinned_right_by_default() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let right_edge = layout.inspector.x + layout.inspector.w;
    assert!((right_edge - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
}

#[test]
fn layout_canvas_spans_full_viewport_default() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    // Canvas is the full-viewport backdrop; chrome floats over.
    assert!((layout.canvas.x - layout.viewport.x).abs() < f32::EPSILON);
    assert!((layout.canvas.w - layout.viewport.w).abs() < f32::EPSILON);
    // Side panels still sit at their canonical positions.
    assert!(layout.hierarchy.x > layout.left_rail.x + layout.left_rail.w);
    let insp_right = layout.inspector.x + layout.inspector.w;
    assert!((insp_right - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
}

#[test]
fn layout_mirror_swaps_sides() {
    let layout = HeroLayout::for_viewport_mirrored(ipad12_viewport(), true);
    // Mirrored: inspector after rail (left), hierarchy pinned right.
    assert!(layout.inspector.x > layout.left_rail.x + layout.left_rail.w);
    let hier_right = layout.hierarchy.x + layout.hierarchy.w;
    assert!((hier_right - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
    // Canvas is full-viewport in either orientation.
    assert!((layout.canvas.w - layout.viewport.w).abs() < f32::EPSILON);
}

#[test]
fn layout_bottom_hud_centered_horizontally() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mid = layout.bottom_hud.x + layout.bottom_hud.w * 0.5;
    assert!((mid - HERO_VIEWPORT_W * 0.5).abs() < 0.5);
}

#[test]
fn hero_default_carries_fixture_selection() {
    crate::test_support::ensure_panel_registry();
    let h = HeroScreen::new(NodeId(1));
    assert!(h.selection.is_some());
}

#[test]
fn hero_selection_clearable() {
    crate::test_support::ensure_panel_registry();
    let h = HeroScreen::new(NodeId(1)).selection(None);
    assert!(h.selection.is_none());
}

#[test]
fn a11y_root_is_window() {
    crate::test_support::ensure_panel_registry();
    let h = HeroScreen::new(NodeId(1));
    let node = h.build_a11y(ipad12_viewport());
    assert_eq!(node.role(), Role::Window);
}

#[test]
fn paint_hero_smoke_default() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
}

#[test]
fn paint_hero_smoke_alternate_theme() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1)).theme(Theme::Sunstone);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
}

#[test]
fn paint_hero_smoke_no_selection() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1)).selection(None);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
}

#[test]
fn paint_hero_smoke_all_themes() {
    crate::test_support::ensure_panel_registry();
    for theme in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        let mut hero = HeroScreen::new(NodeId(1)).theme(theme);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    }
}

use bumpalo::Bump;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};

fn down(x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    }
}

#[allow(dead_code)]
fn up(x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind: PointerKind::Up,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    }
}

#[test]
fn hero_pre_populates_store_with_topbar_and_tools() {
    crate::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    for id in [
        ids::TOPBAR_SAVE,
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::HIERARCHY_ADD,
        ids::TOOL_TRANSLATE,
        ids::TOOL_REDO,
    ] {
        assert!(
            hero.store.contains(id),
            "store missing pre-populated id {id:?}"
        );
    }
}

#[test]
fn hero_translate_tool_starts_pressed() {
    crate::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    assert_eq!(
        hero.store.button_state(ids::TOOL_TRANSLATE),
        Some(ButtonState::Pressed),
    );
}

#[test]
fn hero_topbar_save_click_opens_save_menu() {
    crate::test_support::ensure_panel_registry();
    // Save chip on the topbar now opens the SaveMenu context
    // menu (same pattern as the Theme chip → ThemeSelector). The
    // pointer Down → menu-open short-circuits the Up's
    // Click(TOPBAR_SAVE) emit, so we assert on the open menu's
    // kind instead.
    let mut hero = HeroScreen::new(NodeId(1));
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let arena = Bump::new();
    let mut save_x = 0.0;
    let mut save_y = 0.0;
    'outer: for y_int in (14..54).step_by(4) {
        for x_int in (14..1352).step_by(4) {
            if hero.hit_index.hit(x_int as f32, y_int as f32) == Some(ids::TOPBAR_SAVE) {
                save_x = x_int as f32;
                save_y = y_int as f32;
                break 'outer;
            }
        }
    }
    assert!(save_x > 0.0, "TOPBAR_SAVE rect not found in hit_index");
    let _ = hero.handle_pointer(down(save_x, save_y), &arena);
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(crate::interaction::ContextMenuKind::SaveMenu)
    ));
}

#[test]
fn hero_apply_event_hierarchy_click_changes_selection() {
    crate::test_support::ensure_panel_registry();
    // Placeholder fixture only registers Scene Root; the reserved
    // HIER_* ids return None from `hierarchy_label_for_id` until
    // the pilot project wires real entities.
    let mut hero = HeroScreen::new(NodeId(1));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::HIER_PLAYER));
    assert!(consumed);
    assert_eq!(
        hero.selection.as_ref().map(|s| s.label.as_str()),
        Some("Scene Root")
    );
}

#[test]
fn hero_apply_event_unrelated_click_returns_false() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::TOPBAR_SAVE));
    assert!(!consumed);
}

/// Regression: the Widget Gallery must publish content_h /
/// visible_h to the store after painting so the wheel dispatch
/// can clamp the scroll bound on `GAL_PANEL`. Without this the
/// user reports "scroll doesn't work" — wheel events would either
/// be ignored (no panel match) or fail to advance (max_scroll = 0).
#[test]
fn gallery_publishes_scroll_bounds_after_paint() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
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
    // The cursor at the center of the panel must select GAL_PANEL
    // when dispatch_wheel calls `panel_at`.
    let cx = panel_rect.x + panel_rect.w * 0.5;
    let cy = panel_rect.y + panel_rect.h * 0.5;
    assert_eq!(
        hero.store.panel_at(cx, cy),
        Some(ids::GAL_PANEL),
        "cursor over gallery center should resolve to GAL_PANEL"
    );
    // End-to-end wheel: dispatch a wheel event at the gallery
    // center with a negative delta (macOS "swipe up" / scroll
    // forward) and assert panel_scroll advanced.
    let arena = bumpalo::Bump::new();
    let before = hero.store.panel_scroll(ids::GAL_PANEL);
    let _ = crate::interaction::dispatch_wheel(
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

#[test]
fn inspector_position_value_displayed_in_pixels_round_trips_to_meters() {
    crate::test_support::ensure_panel_registry();
    // Sim position = 1.5 m; project in Pixels mode (default 100
    // px/m) → store NumberInput shows 150. Editing to 200 and
    // committing should publish 2.0 m (200 / 100) into
    // `pending_transform_edit.translation`.
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.visible = true;
    hero.project.display_unit = crate::project::DisplayUnit::Pixels;
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 1,
        translation: [1.5, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
    });
    // Paint once so sync_inspector_from_snapshots seeds the store
    // with the *converted* value (150 px, not 1.5 m).
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let stored_x = hero
        .store
        .number_value(ids::INSP_TRANSFORM_POS_X)
        .expect("Position X must be seeded");
    assert!(
        (stored_x - 150.0).abs() < 1e-3,
        "Position X should be displayed in pixels (150), got {stored_x}"
    );
    // User edits 150 → 200 (in pixels), commits.
    hero.store
        .set_number_value(ids::INSP_TRANSFORM_POS_X, 200.0);
    let _ = hero.apply_event(WidgetEvent::ValueChanged(ids::INSP_TRANSFORM_POS_X));
    let pending = hero
        .bus
        .drain()
        .find_map(|a| match a {
            crate::action_bus::EditorAction::InspectorTransformEdit(info) => Some(info),
            _ => None,
        })
        .expect("commit must publish InspectorTransformEdit");
    assert!(
        (pending.translation[0] - 2.0).abs() < 1e-3,
        "200 px should commit as 2.0 m (200 / 100 px/m), got {} m",
        pending.translation[0]
    );
}

#[test]
fn inspector_position_meters_mode_displays_raw_meters() {
    crate::test_support::ensure_panel_registry();
    // Sanity: default Meters mode is a no-op — store displays the
    // raw meter value and commit is identity.
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.visible = true;
    assert_eq!(
        hero.project.display_unit,
        crate::project::DisplayUnit::Meters
    );
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 1,
        translation: [1.5, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
    });
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let stored_x = hero.store.number_value(ids::INSP_TRANSFORM_POS_X).unwrap();
    assert!(
        (stored_x - 1.5).abs() < 1e-3,
        "Meters mode should display raw value 1.5, got {stored_x}"
    );
}

#[test]
fn settings_unit_submenu_options_flip_project_display_unit() {
    crate::test_support::ensure_panel_registry();
    // Clicking "Pixels" / "Meters" in the SettingsUnit submenu
    // writes `project.display_unit` and closes the context menu.
    let mut hero = HeroScreen::new(NodeId(1));
    assert_eq!(
        hero.project.display_unit,
        crate::project::DisplayUnit::Meters
    );
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsUnitSubmenu,
        });
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_UNIT_PIXELS));
    assert!(consumed, "Pixels click should be consumed");
    assert_eq!(
        hero.project.display_unit,
        crate::project::DisplayUnit::Pixels,
        "display_unit must flip to Pixels"
    );
    assert!(
        hero.store.context_menu().is_none(),
        "menu must close after pick"
    );
    // Re-open to flip back.
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsUnitSubmenu,
        });
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_UNIT_METERS));
    assert_eq!(
        hero.project.display_unit,
        crate::project::DisplayUnit::Meters
    );
}

#[test]
fn settings_unit_cascade_opens_unit_submenu() {
    crate::test_support::ensure_panel_registry();
    // Clicking the top-level "Display unit ▶" row swaps the open
    // context menu to `SettingsUnitSubmenu`.
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsMenu,
        });
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_UNIT));
    assert!(consumed);
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(crate::interaction::ContextMenuKind::SettingsUnitSubmenu)
    ));
}

/// Same shape as `gallery_publishes_scroll_bounds_after_paint`, but
/// for the Grid Settings floating panel. Pins the end-to-end wheel
/// pipeline so Enio's "scroll wheel doesn't work" report has a
/// regression net: GS_PANEL must (1) publish a content_h that
/// exceeds visible_h, (2) own the panel rect under its center, and
/// (3) advance `panel_scroll` when a wheel event hits.
#[test]
fn grid_settings_publishes_scroll_bounds_and_wheel_advances_scroll() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.grid.snap_state.panel_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let gs_id = crate::grid_snap::ids::GS_PANEL;
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
    let _ = crate::interaction::dispatch_wheel(
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

/// Regression: right-clicking inside the gallery body → choosing
/// "Create note" must push a `NoteData` keyed on `GAL_PANEL` (NOT
/// `INSP_PANEL`) so the gallery renders it on the next frame. The
/// gallery is the canonical UI ground-truth for peripheral agents
/// — features the showcase advertises (sticky notes, section
/// outline) need to work in the in-app gallery, not just in the
/// retired reference snapshot.
#[test]
fn gallery_create_note_targets_gal_panel() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    // Paint once so `panel_rect(GAL_PANEL)` is published and
    // `LAST_BODY_TOP_SCREEN_Y` is set for the upcoming dispatch.
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    let gallery_rect = hero.store.panel_rect(ids::GAL_PANEL).unwrap();
    let cx = gallery_rect.x + gallery_rect.w * 0.5;
    let cy = gallery_rect.y + gallery_rect.h * 0.5;
    // Open the CreateNote context menu via the same path the
    // pointer dispatch uses for a secondary-button down at the
    // gallery center.
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: cx,
            y: cy,
            kind: crate::interaction::ContextMenuKind::CreateNote {
                panel: ids::GAL_PANEL,
                before_section: None,
            },
        });
    // The real pointer dispatch closes the menu on the Down that
    // hit the menu item, snapshotting the request into
    // `last_context_menu` before the Click reaches `apply_event`.
    // Skipping this step would leave the request in the still-open
    // `context_menu` slot where `consume_last_context_menu` can't
    // see it.
    hero.store.close_context_menu();
    // Click "Create note" — inspector::apply_event handles it.
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

/// Regression: right-clicking on a gallery section header →
/// choosing a color must write `section_outline_color` so the
/// gallery's next paint draws the colored ring around that
/// section's body. Mirror of the live Inspector's right-click
/// outline path — same NodeIds (`INSP_SECTION_*`) because the
/// gallery re-uses the section painters.
#[test]
fn gallery_section_outline_color_writes_through() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    // Open the SectionOutline menu for the Inputs section header.
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SectionOutline {
                section: ids::INSP_SECTION_INPUTS,
            },
        });
    // Mirror the real Down-on-menu-item path that snapshots the
    // request into `last_context_menu` before the Click fires.
    hero.store.close_context_menu();
    // Pick "Yellow" (color_idx 0).
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_OUTLINE_0));
    assert!(consumed, "CTX_MENU_OUTLINE_0 click should be consumed");
    assert_eq!(
        hero.store.section_outline_color(ids::INSP_SECTION_INPUTS),
        Some(0),
        "Inputs section should have outline color 0 (Yellow) set"
    );
}

#[test]
fn paint_top_bar_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_top_bar(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &store,
        false,
    );
}

/// With `image_tools_mode = true`, the painter must register the
/// `IMAGE_ACTION_TRIM` hit and must NOT register the right-side
/// default clusters (Project/Play/Right/Settings).
#[test]
fn paint_top_bar_image_tools_mode_swaps_right_side() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_top_bar(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &store,
        true,
    );
    assert!(
        hits.rect_for(ids::IMAGE_ACTION_TRIM).is_some(),
        "trim action pill must be hit-registered when image_tools_mode is on",
    );
    for default_right in [
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::TOPBAR_SETTINGS,
    ] {
        assert!(
            hits.rect_for(default_right).is_none(),
            "right-side default cluster {default_right:?} must NOT be registered in image_tools mode",
        );
    }
    // Left half stays intact — Save/Open/ImageTools are still hit-able.
    assert!(hits.rect_for(ids::TOPBAR_SAVE).is_some());
    assert!(hits.rect_for(ids::TOPBAR_OPEN).is_some());
    assert!(hits.rect_for(ids::TOPBAR_IMAGE_TOOLS).is_some());
}

/// Clicking the Image Tools pill flips `image_tools_mode`; clicking
/// again flips it back. Verified through `HeroScreen::apply_event`
/// so the dispatcher hook is exercised end-to-end.
#[test]
fn click_on_image_tools_pill_toggles_mode() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(!hero.image_edit.mode_on);
    assert!(hero.apply_event(WidgetEvent::Click(ids::TOPBAR_IMAGE_TOOLS)));
    assert!(hero.image_edit.mode_on);
    assert!(hero.apply_event(WidgetEvent::Click(ids::TOPBAR_IMAGE_TOOLS)));
    assert!(!hero.image_edit.mode_on);
}

/// M14.A: a `ValueChanged` event on any Transform NumberInput
/// pushes a fresh `EditorAction::InspectorTransformEdit` onto the
/// bus (Wave 2.5 PR 11.8d — was `pending_transform_edit`), taking
/// the current store values for every axis (X/Y/Rot/Scale-X/Scale-Y)
/// plus the selected entity id from `inspector_transform`.
/// Rotation is converted from degrees (UI) back to radians
/// (canonical) at commit.
#[test]
fn transform_field_commit_raises_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    // No selection → no push even on commit (avoids silently
    // editing a non-existent entity).
    hero.inspector.transform = None;
    assert!(!hero.apply_event(WidgetEvent::ValueChanged(ids::INSP_TRANSFORM_POS_X)));
    assert!(hero.bus.is_empty());

    // With selection + custom store values → push mirrors the
    // store snapshot exactly. We seed the store with non-identity
    // numbers and verify the commit assembles them all.
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xCAFE_F00D,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
    });
    hero.store.set_number_value(ids::INSP_TRANSFORM_POS_X, 1.5);
    hero.store
        .set_number_value(ids::INSP_TRANSFORM_POS_Y, -2.25);
    hero.store.set_number_value(ids::INSP_TRANSFORM_ROT, 90.0); // degrees
    hero.store
        .set_number_value(ids::INSP_TRANSFORM_SCALE_X, 2.0);
    hero.store
        .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, 0.5);
    assert!(hero.apply_event(WidgetEvent::ValueChanged(ids::INSP_TRANSFORM_POS_X)));
    let pending = hero
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(info) => Some(info),
            _ => None,
        })
        .expect("pending populated");
    assert_eq!(pending.entity_bits, 0xCAFE_F00D);
    assert_eq!(pending.translation, [1.5, -2.25]);
    // 90° → π/2 rad. `to_radians` is bit-deterministic (HR-5).
    assert!((pending.rotation_rad - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    assert_eq!(pending.scale, [2.0, 0.5]);
}

/// M14.A: clicking the Reset-to-Identity button pushes an
/// Identity-shaped `EditorAction::InspectorTransformEdit` (Wave
/// 2.5 PR 11.8d — was `pending_transform_edit`). Same commit
/// path as a field ValueChanged so the shell's queue-push code
/// stays uniform.
#[test]
fn transform_reset_button_publishes_identity() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xBABE_0042,
        translation: [10.0, 20.0],
        rotation_rad: 1.0,
        scale: [3.0, 3.0],
    });
    // Even if the store has garbage in it, Reset always publishes
    // pure identity — independent of buffer state.
    hero.store.set_number_value(ids::INSP_TRANSFORM_POS_X, 99.0);
    assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_TRANSFORM_RESET)));
    let pending = hero
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(info) => Some(info),
            _ => None,
        })
        .expect("pending populated");
    assert_eq!(pending.entity_bits, 0xBABE_0042);
    assert_eq!(pending.translation, [0.0, 0.0]);
    assert_eq!(pending.rotation_rad, 0.0);
    assert_eq!(pending.scale, [1.0, 1.0]);

    // Without a selection, Reset is a no-op (consumes the click
    // returning false → dispatcher walks; matches non-sprite
    // Reimport behavior).
    hero.inspector.transform = None;
    assert!(!hero.apply_event(WidgetEvent::Click(ids::INSP_TRANSFORM_RESET)));
    assert!(hero.bus.is_empty());
}

/// M14.D: Toggled on the Visibility checkbox pushes
/// `EditorAction::InspectorVisibilityEdit` (Wave 2.5 PR 11.8d —
/// was `pending_visibility_edit`) with the POST-toggle store
/// value. Sequence: snapshot says visible=true → dispatch flipped
/// Checkbox to Unchecked → apply_event reads Unchecked → push
/// `visible: false`.
#[test]
fn visibility_toggle_publishes_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    // Selection that has a Transform component (we don't paint
    // here, just exercise apply_event semantics).
    hero.inspector.visibility = Some(InspectorVisibilityInfo {
        entity_bits: 0xBABE_BEEF,
        visible: true,
    });
    // Simulate the dispatch having toggled Checked → Unchecked.
    if let Some(InteractiveState::Checkbox { value, .. }) =
        hero.store.get_mut(ids::INSP_VISIBILITY_CHECK)
    {
        *value = crate::widget::CheckboxValue::Unchecked;
    }
    assert!(hero.apply_event(WidgetEvent::Toggled(ids::INSP_VISIBILITY_CHECK)));
    let pending = hero
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorVisibilityEdit(info) => Some(info),
            _ => None,
        })
        .expect("pending populated");
    assert_eq!(pending.entity_bits, 0xBABE_BEEF);
    assert!(!pending.visible, "toggle should commit visible=false");
}

/// M14.C: Click on a Strategy button different from the current
/// `source_kind` pushes `EditorAction::InspectorSpriteSourceChange`
/// (Wave 2.5 PR 11.8d — was `pending_sprite_source_change`) with
/// the requested kind. Same-kind click is consumed silently.
#[test]
fn strategy_click_raises_pending_when_kind_differs() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.sprite = Some(InspectorSpriteInfo {
        entity_bits: 0xC0FF_EE00,
        name: "Player".into(),
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 7 },
        source_pixels: Some((256, 256)),
        can_reimport: true,
    });
    // Current = Atlas → click on Individual button publishes.
    assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_INDIVIDUAL)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::InspectorSpriteSourceChange {
            entity_bits: 0xC0FF_EE00,
            strategy: RequestedSpriteStrategy::Individual
        }]
    );

    // Click on Atlas (already-current) is consumed but no push.
    assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_ATLAS)));
    assert!(hero.bus.is_empty());

    // HandPacked → publishes too (shell decides to skip with toast).
    assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_HANDPACKED)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::InspectorSpriteSourceChange {
            entity_bits: 0xC0FF_EE00,
            strategy: RequestedSpriteStrategy::HandPacked
        }]
    );
}

/// M14.C: Without `inspector_sprite` (nothing selected), Strategy
/// clicks are no-ops — apply_event returns false so the dispatcher
/// keeps walking and the bus stays empty.
#[test]
fn strategy_click_no_pending_without_sprite_selection() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.sprite = None;
    assert!(!hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_INDIVIDUAL)));
    assert!(hero.bus.is_empty());
}

/// M14.E: `TextChanged` on the editable entity-name field pushes
/// `EditorAction::InspectorNameEdit` (Wave 2.5 PR 11.8d — was
/// `pending_name_edit`) with the current store text. Multiple
/// keystrokes within one frame each push their own variant; the
/// shell drains them in push order.
#[test]
fn name_text_changed_publishes_pending_with_current_text() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.name = Some(InspectorNameInfo {
        entity_bits: 0xDEAD_BEEF,
        name: "Old".to_string(),
    });
    // Simulate the dispatch having mutated the TextInput buffer
    // to "Player" via a sequence of keystrokes.
    if let Some(InteractiveState::TextInput { text, caret, .. }) =
        hero.store.get_mut(ids::INSP_ENTITY_NAME)
    {
        text.clear();
        text.push_str("Player");
        *caret = text.len();
    }
    assert!(hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
    let pending = hero
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorNameEdit(info) => Some(info),
            _ => None,
        })
        .expect("pending populated after TextChanged");
    assert_eq!(pending.entity_bits, 0xDEAD_BEEF);
    assert_eq!(pending.name, "Player");
}

/// M14.E: without an `inspector_name` snapshot (no selection),
/// `TextChanged` is a no-op — apply_event returns false so the
/// dispatcher keeps walking.
#[test]
fn name_text_changed_no_pending_without_selection() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.name = None;
    assert!(!hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
    assert!(hero.bus.is_empty());
}

/// M14.E: TextChanged on the entity-name field with a selection
/// pushes `EditorAction::InspectorNameEdit` (Wave 2.5 PR 11.8d).
/// Without a selection, returns false and the bus stays empty.
#[test]
fn entity_name_text_changed_raises_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    // Seed the TextInput buffer with what the user just typed.
    if let Some(InteractiveState::TextInput { text, caret, .. }) =
        hero.store.get_mut(ids::INSP_ENTITY_NAME)
    {
        *text = "Player Two".to_string();
        *caret = text.len();
    }
    hero.inspector.name = Some(InspectorNameInfo {
        entity_bits: 0xDEAD_F00D,
        name: "Player".into(),
    });
    assert!(hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
    let p = hero
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorNameEdit(info) => Some(info),
            _ => None,
        })
        .expect("pending populated");
    assert_eq!(p.entity_bits, 0xDEAD_F00D);
    assert_eq!(p.name, "Player Two");

    // No selection → no push.
    hero.inspector.name = None;
    assert!(!hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
    assert!(hero.bus.is_empty());
}

/// Audit #2 fix (MEDIUM): `paint_hero_screen` selection-change
/// block resets the entity-name TextInput state to `Normal` (not
/// just `text`/`caret`/`selection_anchor`). Otherwise the
/// painter keeps drawing the focused chrome (caret + focus ring)
/// on a field the user hasn't authored yet — same canonical
/// cleanup dispatch.rs:1189 does on Blur.
#[test]
fn selection_switch_resets_entity_name_input_state_to_normal() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    // 1) Frame 1: select entity A, mark its TextInput Focused
    //    (simulating user click on the field).
    hero.inspector.name = Some(InspectorNameInfo {
        entity_bits: 0xAAAA_0001,
        name: "Player A".into(),
    });
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xAAAA_0001,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
    });
    paint_hero_screen(&mut hero, layout.viewport, &mut scene, &mut text);
    if let Some(InteractiveState::TextInput { state, .. }) =
        hero.store.get_mut(ids::INSP_ENTITY_NAME)
    {
        *state = crate::widget::TextInputState::Focused;
    }
    // 2) Frame 2: switch to entity B. The selection-change block
    //    must flip state back to Normal regardless of the focus
    //    snapshot the user left on entity A.
    hero.inspector.name = Some(InspectorNameInfo {
        entity_bits: 0xBBBB_0002,
        name: "Player B".into(),
    });
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xBBBB_0002,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
    });
    paint_hero_screen(&mut hero, layout.viewport, &mut scene, &mut text);
    match hero.store.get(ids::INSP_ENTITY_NAME) {
        Some(InteractiveState::TextInput { state, text, .. }) => {
            assert_eq!(
                *state,
                crate::widget::TextInputState::Normal,
                "state must reset to Normal on selection switch"
            );
            assert_eq!(text, "Player B", "buffer must reset to new entity's name");
        }
        _ => panic!("INSP_ENTITY_NAME state missing"),
    }
}

/// Audit fix #7 (HIGH): clicking a strategy button resets the
/// stored ButtonState to Normal so the painter's snapshot-driven
/// `Pressed` pin is the single visual source of truth.
#[test]
fn strategy_click_resets_button_state_to_normal() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.sprite = Some(InspectorSpriteInfo {
        entity_bits: 0x00C0_FFEE,
        name: "S".into(),
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Individual { texture_id: 1 },
        source_pixels: Some((64, 64)),
        can_reimport: true,
    });
    // Simulate dispatch having set Pressed on the click target.
    if let Some(InteractiveState::Button { state }) =
        hero.store.get_mut(ids::INSP_RENDER_STRATEGY_ATLAS)
    {
        *state = crate::widget::ButtonState::Pressed;
    }
    assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_ATLAS)));
    // After apply_event: pending raised AND button state forced
    // back to Normal so the painter's pin re-runs cleanly.
    assert!(matches!(
        hero.store.button_state(ids::INSP_RENDER_STRATEGY_ATLAS),
        Some(crate::widget::ButtonState::Normal),
    ));
}

/// M14.D: Toggled without an `inspector_visibility` snapshot
/// (e.g. nothing selected) is a no-op — apply_event returns
/// false so the dispatcher keeps walking and the bus stays
/// empty.
#[test]
fn visibility_toggle_no_pending_without_selection() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.visibility = None;
    assert!(!hero.apply_event(WidgetEvent::Toggled(ids::INSP_VISIBILITY_CHECK)));
    assert!(hero.bus.is_empty());
}

/// Clicking the Trim Transparency action pill pushes an
/// `EditorAction::Trim` onto the bus capturing the current
/// `gizmo_selection`. Wave 2.5 PR 11.8b1: bus migration. When
/// nothing is selected, the bus stays empty (click still
/// consumed so the dispatcher doesn't keep walking).
#[test]
fn click_on_trim_pill_raises_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    // No selection → nothing pushed after click.
    hero.gizmo.selection = None;
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
    assert!(hero.bus.is_empty());

    // With selection → bus contains exactly one Trim with the
    // entity_bits taken from gizmo_selection at click time.
    hero.gizmo.selection = Some(0xDEAD_BEEF);
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::Trim {
            entity_bits: 0xDEAD_BEEF
        }]
    );
}

/// Make Square pill mirrors the Trim bus-push semantics (Wave 2.5 PR 11.8b2).
#[test]
fn click_on_make_square_pill_raises_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.selection = None;
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_MAKE_SQUARE)));
    assert!(hero.bus.is_empty());

    hero.gizmo.selection = Some(0xCAFE_BABE);
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_MAKE_SQUARE)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::MakeSquare {
            entity_bits: 0xCAFE_BABE
        }]
    );
}

/// Bg Removal pill raises `EditorAction::ActivateBgRemoval`
/// (not `EditorAction::Bgremoval` — that one is the Apply
/// trigger, raised shell-side when the tool's panel Toggle
/// fires). Shell drains the Activate variant by calling
/// `tools.set_active(...)`. (Wave 2.5 PR 11.8b3.)
#[test]
fn click_on_bgremoval_pill_raises_activate_intent() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(hero.bus.is_empty());
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_BGREMOVAL)));
    let drained: Vec<_> = hero.bus.drain().collect();
    // Only the Activate variant — the Apply variant (Bgremoval)
    // is raised shell-side, not by the pill click.
    assert_eq!(drained, vec![EditorAction::ActivateBgRemoval]);
}

#[test]
fn paint_left_rail_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_left_rail(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &store,
    );
}

#[test]
fn paint_inspector_smoke_with_selection() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let sel = fixture::default_selection();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_inspector(
        &layout,
        Some(&sel),
        &mut scene,
        &mut text,
        Theme::Sunstone,
        &mut hits,
        &store,
    );
}

#[test]
fn paint_inspector_smoke_no_selection() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_inspector(
        &layout,
        None,
        &mut scene,
        &mut text,
        Theme::Blueprint,
        &mut hits,
        &store,
    );
}

#[test]
fn paint_hierarchy_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    let mut hits = HitIndex::new();
    let mut store = WidgetStore::with_capacity(32);
    paint_hierarchy(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &mut store,
    );
}

#[test]
fn paint_bottom_hud_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_bottom_hud(
        &layout,
        &mut scene,
        &mut text,
        Theme::Workshop,
        BottomHudStats::default(),
    );
}

#[test]
fn paint_selection_overlay_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let sel = fixture::default_selection();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::new();
    paint_selection_overlay(&layout, &sel, &mut scene, &mut text, Theme::Forge);
}

// ─────────────── M14.6 F: per-row context-menu apply_event ────────────────

/// Stage a closed HierarchyRow snapshot so `apply_event` can read
/// it via `consume_last_context_menu`. Mirrors what dispatch does
/// on the menu-closing Down → next-frame-Click sequence.
fn stage_hierarchy_row_snapshot(hero: &mut HeroScreen, row: NodeId) {
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::HierarchyRow { row },
        });
    // Closing copies the request into `last_context_menu`, which
    // is what `consume_last_context_menu` returns.
    hero.store.close_context_menu();
}

#[test]
fn hier_menu_duplicate_sets_pending_duplicate() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row = NodeId(100_500);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
    // Snapshot was consumed.
    assert!(hero.store.last_context_menu().is_none());
}

#[test]
fn hier_menu_add_child_sets_pending_add_child() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row = NodeId(100_501);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_ADD_CHILD));
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierAddChild { row }]);
}

#[test]
fn hier_menu_reset_transform_sets_pending() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row = NodeId(100_502);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_RESET_TRANSFORM));
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierResetTransform { row }]);
}

#[test]
fn hier_menu_delete_sets_pending_delete() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row = NodeId(100_503);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE));
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDelete { row }]);
}

#[test]
fn hier_menu_click_without_snapshot_consumes_but_no_pending() {
    crate::test_support::ensure_panel_registry();
    // Defensive case: stray Click without any prior right-click
    // snapshot still consumes the event so the click doesn't
    // bubble to row selection, but no pending action is raised.
    let mut hero = HeroScreen::new(NodeId(1));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
    assert!(consumed);
    assert!(hero.bus.is_empty());
}

#[test]
fn hierarchy_row_click_raises_pending_for_live_entries() {
    crate::test_support::ensure_panel_registry();
    // Build a live-mode hierarchy with one entry, then click the
    // matching NodeId. `HierRowClick` should fire on the bus so
    // the shell can sync `gizmo_selection`.
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row_id = NodeId(100_500);
    let mut entries = std::collections::BTreeMap::new();
    entries.insert(
        row_id,
        fixture::HierarchyEntity {
            name: "hero_001".into(),
            icon: crate::icons::IconId::Sprite,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
    );
    hero.sync_from_hierarchy(&[row_id], entries);
    let consumed = hero.apply_event(WidgetEvent::Click(row_id));
    assert!(consumed, "live-mode row click should consume");
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierRowClick { row: row_id }]);
}

#[test]
fn hierarchy_row_click_silent_for_fixture_only_rows() {
    crate::test_support::ensure_panel_registry();
    // Fixture-mode click (no `sync_from_hierarchy`) shouldn't
    // push `HierRowClick` — the M14.6 D path is live-only.
    let mut hero = HeroScreen::new(NodeId(1));
    let _ = hero.apply_event(WidgetEvent::Click(ids::HIER_PLAYER));
    assert!(hero.bus.is_empty());
}

#[test]
fn hier_menu_one_action_per_drain() {
    crate::test_support::ensure_panel_registry();
    // Two consecutive clicks (Duplicate then Delete) only fire
    // the first — the snapshot is consumed and the second click
    // sees an empty `last_context_menu`. This protects against
    // double-trigger if a synthetic event stream emits both.
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    let row = NodeId(100_504);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE));
    let drained: Vec<_> = hero.bus.drain().collect();
    // Only Duplicate — the second Click consumed but found no
    // snapshot to attach a row to, so no Delete variant pushed.
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
}
