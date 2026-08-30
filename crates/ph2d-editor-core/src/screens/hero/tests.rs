use super::*;
use crate::action_bus::EditorAction;
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
    assert!((layout.left_rail.w - style::rail_w()).abs() < f32::EPSILON);
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
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
}

#[test]
fn paint_hero_smoke_alternate_theme() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1)).theme(Theme::Sunstone);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
}

#[test]
fn new_image_modal_size_bg_create_raises_request() {
    // The New-image modal's Size/Background radio clicks update the store selection; Create raises the
    // `(size, bg)` request the shell services, and closes the modal (Enio 2026-06-24).
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_new_image_dialog();
    assert_eq!(hero.store.new_image_size(), 512); // defaults
    assert_eq!(hero.store.new_image_bg(), 0); // transparent
    let _ = hero.apply_event(WidgetEvent::Click(crate::ids::CTX_MENU_NEW_IMAGE_SIZE_1024));
    let _ = hero.apply_event(WidgetEvent::Click(crate::ids::CTX_MENU_NEW_IMAGE_BG_WHITE));
    assert_eq!(hero.store.new_image_size(), 1024);
    assert_eq!(hero.store.new_image_bg(), 2); // white
    let _ = hero.apply_event(WidgetEvent::Click(crate::ids::CTX_MENU_NEW_IMAGE_CREATE));
    assert_eq!(hero.store.take_new_image_request(), Some((1024, 2)));
    assert!(
        hero.store.context_menu().is_none(),
        "Create closes the modal"
    );
    assert_eq!(hero.store.take_new_image_request(), None, "drained once");
}

#[test]
fn paint_hero_smoke_no_selection() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1)).selection(None);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
        let mut text = TextSystem::without_system_fonts();
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

// ⛔ `fn up(...)` foi REMOVIDA em 2026-08-21: zero call sites, e o `#[allow(dead_code)]` em
// cima dela estava a silenciar um cadáver real (auditoria `docs/Sprite_projeto/20` §8).

#[test]
fn hero_pre_populates_store_with_topbar_and_tools() {
    crate::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    // ADR-0029 Phase C.2: `HIERARCHY_ADD` is now registered by the
    // typed `HierarchyPanel::populate`; this editor-core test
    // doesn't install the typed registry, so we no longer check
    // hierarchy ids here. The Hierarchy panel's own regression
    // tests in `crates/ph2d-panel-hierarchy/tests/` cover that.
    for id in [
        ids::TOPBAR_SAVE,
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_RIGHT_LAYERS,
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
    let mut text = TextSystem::without_system_fonts();
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
fn hero_apply_event_unrelated_click_returns_false() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let consumed = hero.apply_event(WidgetEvent::Click(ids::TOPBAR_SAVE));
    assert!(!consumed);
}

// ADR-0029 Phase C.3: disabled in editor-core — touches
// `view.widget_gallery_visible` (migrated to `panel_visibility`) and
// asserts on `paint_hero_screen` output that needs the typed registry
// installed with `WidgetGalleryPanel`. Recreated at
// `crates/ph2d-panel-widget-gallery/tests/widget_gallery_paint.rs`.
/// Regression: the Widget Gallery must publish content_h /
/// visible_h to the store after painting so the wheel dispatch
/// can clamp the scroll bound on `GAL_PANEL`. Without this the
/// user reports "scroll doesn't work" — wheel events would either
/// be ignored (no panel match) or fail to advance (max_scroll = 0).
#[cfg(any())]
#[test]
fn gallery_publishes_scroll_bounds_after_paint() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
fn settings_unit_submenu_options_flip_project_display_unit() {
    crate::test_support::ensure_panel_registry();
    // Clicking "Pixels" / "Meters" in the SettingsUnit submenu
    // writes `project.display_unit` and closes the context menu.
    let mut hero = HeroScreen::new(NodeId(1));
    // Default is Pixels (Enio 2026-05-21).
    assert_eq!(
        hero.project.display_unit,
        crate::project::DisplayUnit::Pixels
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
fn curve_point_handle_menu_routes_each_kind_to_pending_wire() {
    crate::test_support::ensure_panel_registry();
    // Each entry in the CurvePointHandle menu parks its wire u8 in `pending_curve_point_handle`
    // (Free/Aligned/Vector/Auto = 0/1/2/3) and closes the menu; the shell drains it and calls
    // `PainterTool::set_curve_handle_kind`.
    let cases = [
        (ids::CTX_MENU_CURVE_HANDLE_FREE, 0u8),
        (ids::CTX_MENU_CURVE_HANDLE_ALIGNED, 1),
        (ids::CTX_MENU_CURVE_HANDLE_VECTOR, 2),
        (ids::CTX_MENU_CURVE_HANDLE_AUTO, 3),
        (ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC, 4),
    ];
    for (entry_id, expected) in cases {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::CurvePointHandle,
            });
        let consumed = hero.apply_event(WidgetEvent::Click(entry_id));
        assert!(consumed, "curve-handle entry click should be consumed");
        assert_eq!(
            hero.pending_curve_point_handle.take(),
            Some(expected),
            "entry must park its handle-kind wire u8"
        );
        assert!(
            hero.store.context_menu().is_none(),
            "menu must close after pick"
        );
    }
}

#[test]
fn motion_path_anchor_menu_routes_each_kind_carrying_the_anchor() {
    crate::test_support::ensure_panel_registry();
    // Each entry parks `(target bits, anchor index, wire u8)` in `pending_motion_path_handle`
    // (Corner/Smooth/Symmetric = 0/1/2) and closes the menu; the shell drains it and calls
    // `TimelineDoc::set_path_tangent_kind`. Unlike the curve/falloff menus, the anchor identity
    // travels IN the menu kind (a path anchor has no persistent selection to recall) — so the
    // gate pins that `(77, 3)` survives the round-trip, not just the kind.
    let cases = [
        (ids::CTX_MENU_PATH_HANDLE_CORNER, 0u8),
        (ids::CTX_MENU_PATH_HANDLE_SMOOTH, 1),
        (ids::CTX_MENU_PATH_HANDLE_SYMMETRIC, 2),
    ];
    for (entry_id, expected) in cases {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::MotionPathAnchor { target: 77, i: 3 },
            });
        let consumed = hero.apply_event(WidgetEvent::Click(entry_id));
        assert!(consumed, "path-handle entry click should be consumed");
        assert_eq!(
            hero.pending_motion_path_handle.take(),
            Some((77, 3, expected)),
            "entry must park (target, i, kind) — the anchor identity travels with the menu"
        );
        assert!(
            hero.store.context_menu().is_none(),
            "menu must close after pick"
        );
    }
}

#[test]
fn simple_row_context_menu_items_are_populate_registered() {
    // Regression gate for the Painter Falloff "Vector handle does nothing" bug.
    //
    // A simple-row context-menu item drives its chrome handler via the generic
    // dispatch Click path: the Down arms `active` only when the row id is
    // `is_focusable`, which requires a WidgetStore entry (registered `Plain` by
    // `pre_populate::populate_global_context_menu`). The Falloff handle rows were
    // added to the overlay paint + the `chrome::falloff_handle` Click handler but
    // OMITTED from that populate list, so the rows hit-registered yet never armed
    // `active` → the Up emitted no `Click` → the handler never ran (a silent
    // no-op). The dispatch-layer test
    // (`dispatch::tests::context_menu_item_click_emits_click_even_though_menu_closes_on_down`)
    // ASSUMES this registration; THIS test pins the registration itself for every
    // chrome-driven simple-row menu so the populate-register gotcha can't recur.
    crate::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    for id in [
        // Painter Falloff handle (the regression).
        ids::CTX_MENU_FALLOFF_HANDLE_VECTOR,
        ids::CTX_MENU_FALLOFF_HANDLE_AUTO,
        // On-canvas Curve / Free Hand point-handle kinds.
        ids::CTX_MENU_CURVE_HANDLE_FREE,
        ids::CTX_MENU_CURVE_HANDLE_ALIGNED,
        ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC,
        ids::CTX_MENU_CURVE_HANDLE_VECTOR,
        ids::CTX_MENU_CURVE_HANDLE_AUTO,
        // On-canvas motion-path anchor handle types (the vector Node trio, ADR-0141).
        ids::CTX_MENU_PATH_HANDLE_CORNER,
        ids::CTX_MENU_PATH_HANDLE_SMOOTH,
        ids::CTX_MENU_PATH_HANDLE_SYMMETRIC,
    ] {
        assert!(
            hero.store.contains(id),
            "context-menu row {id:?} must be registered by populate_global_context_menu \
             (else its dispatch Click never fires — the populate-register gotcha)"
        );
    }
    // ⛔⛔⛔ **A LISTA DO HierarchyRow ERA ESCRITA À MÃO, E JÁ TINHA DOIS BURACOS** (2026-08-30).
    //
    // O comentário que aqui estava contava a história certa — *"Use as Brush Shape shipped dead
    // because it was hit-painted but OMITTED here"*, Enio 2026-06-25 — e depois disso o menu ganhou
    // `Merge to Layers` e `Export Image…`, e **nenhum dos dois entrou nesta lista**. Os dois estão
    // vivos hoje por sorte, não por gate: um terceiro que nascesse morto passaria igual.
    //
    // ⇒ *Uma lista escrita à mão ao lado de uma tabela é duas respostas à mesma pergunta, e a que
    // envelhece é sempre a escrita à mão.* A lista passa a ser **derivada do próprio menu**: o que
    // o artista vê é exactamente o que este gate exige, e uma linha nova entra sozinha.
    for (id, label, _) in
        super::menu_rows::menu_rows(crate::interaction::ContextMenuKind::HierarchyRow {
            row: NodeId(1),
        })
    {
        assert!(
            hero.store.contains(*id),
            "a linha `{label}` do menu da Hierarquia nao esta' registada pelo \
             populate_global_context_menu - ela pinta, o ponteiro acende-a, e o Click NUNCA sai \
             (o gotcha do populate-register). Ela shipa MORTA."
        );
    }
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

#[test]
fn settings_filter_cascade_opens_filter_submenu() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsMenu,
        });
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_FILTER));
    assert!(consumed);
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(crate::interaction::ContextMenuKind::SettingsFilterSubmenu)
    ));
}

#[test]
fn picking_filter_option_sets_project_and_raises_action() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    // Default is Smooth (Enio 2026-05-21).
    assert_eq!(
        hero.project.image_filter,
        crate::project::ImageFilterMode::Smooth
    );
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsFilterSubmenu,
        });
    // Pick the NON-default option so we verify a real flip.
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_FILTER_PIXELART));
    assert!(consumed);
    // Project setting flips (so the menu state is correct next paint)…
    assert_eq!(
        hero.project.image_filter,
        crate::project::ImageFilterMode::PixelArt
    );
    // …and the shell-bound action is queued so the GPU samplers rebuild.
    let queued: Vec<_> = hero.bus.iter().cloned().collect();
    assert!(
        queued.contains(&crate::action_bus::EditorAction::SetImageFilter {
            mode: crate::project::ImageFilterMode::PixelArt
        })
    );
    // Menu closes after a pick (mirrors the Display-unit flow).
    assert!(hero.store.context_menu().is_none());
}

// ADR-0029 Phase C.4: disabled in editor-core — touches the legacy
// `grid.snap_state.panel_visible` field (now removed) and routes a
// `hero.apply_event` that needs the typed `GridSnapPanel` installed.
// Recreated at
// `crates/ph2d-panel-grid-snap/tests/grid_snap_paint.rs`.
/// Same shape as `gallery_publishes_scroll_bounds_after_paint`, but
/// for the Grid Settings floating panel. Pins the end-to-end wheel
/// pipeline so Enio's "scroll wheel doesn't work" report has a
/// regression net: GS_PANEL must (1) publish a content_h that
/// exceeds visible_h, (2) own the panel rect under its center, and
/// (3) advance `panel_scroll` when a wheel event hits.
#[cfg(any())]
#[test]
fn grid_settings_publishes_scroll_bounds_and_wheel_advances_scroll() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.grid.snap_state.panel_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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

// ADR-0029 Phase C.3: disabled in editor-core — touches
// `view.widget_gallery_visible` and routes a `hero.apply_event` that
// needs the typed registry with `WidgetGalleryPanel`. Recreated at
// `crates/ph2d-panel-widget-gallery/tests/widget_gallery_paint.rs`.
/// Regression: right-clicking inside the gallery body → choosing
/// "Create note" must push a `NoteData` keyed on `GAL_PANEL` (NOT
/// `INSP_PANEL`) so the gallery renders it on the next frame. The
/// gallery is the canonical UI ground-truth for peripheral agents
/// — features the showcase advertises (sticky notes, section
/// outline) need to work in the in-app gallery, not just in the
/// retired reference snapshot.
#[cfg(any())]
#[test]
fn gallery_create_note_targets_gal_panel() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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

// ADR-0029 Phase C.3: disabled in editor-core — touches
// `view.widget_gallery_visible` and routes a `hero.apply_event` that
// needs the typed registry with `WidgetGalleryPanel`. Recreated at
// `crates/ph2d-panel-widget-gallery/tests/widget_gallery_paint.rs`.
/// Regression: right-clicking on a gallery section header →
/// choosing a color must write `section_outline_color` so the
/// gallery's next paint draws the colored ring around that
/// section's body. Mirror of the live Inspector's right-click
/// outline path — same NodeIds (`INSP_SECTION_*`) because the
/// gallery re-uses the section painters.
#[cfg(any())]
#[test]
fn gallery_section_outline_color_writes_through() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.widget_gallery_visible = true;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
    let mut text = TextSystem::without_system_fonts();
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
        &crate::motion::UiMotion::default(),
    );
}

/// With `image_tools_mode = true`, the painter must register the
/// `IMAGE_ACTION_TRIM` hit and must NOT register the right-side
/// default clusters (Project/Play/Right/Settings).
#[test]
fn paint_top_bar_image_tools_mode_swaps_right_side() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
        &crate::motion::UiMotion::default(),
    );
    assert!(
        hits.rect_for(ids::IMAGE_ACTION_TRIM).is_some(),
        "trim action pill must be hit-registered when image_tools_mode is on",
    );
    // PROJECT moved to LEFT half 2026-05-24 (user request: "o seletor
    // de level deve ser deslocado para esquerda ao lado do seletor de
    // themes") — it's now in the always-painted left section, so it
    // stays hit-registered in image_tools mode along with the rest.
    for default_right in [
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::TOPBAR_SETTINGS,
    ] {
        assert!(
            hits.rect_for(default_right).is_none(),
            "right-side default cluster {default_right:?} must NOT be registered in image_tools mode",
        );
    }
    // Left half stays intact — Project/Save/Open/ImageTools still hit-able.
    assert!(hits.rect_for(ids::TOPBAR_PROJECT).is_some());
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

/// Audit #2 fix (MEDIUM): `paint_hero_screen` selection-change
/// block resets the entity-name TextInput state to `Normal` (not
/// just `text`/`caret`/`selection_anchor`). Otherwise the
/// painter keeps drawing the focused chrome (caret + focus ring)
/// on a field the user hasn't authored yet — same canonical
/// cleanup dispatch.rs:1189 does on Blur.
// ⏸️ POR MIGRAR. O destino EXISTE desde 2026-08-21:
// `crates/ph2d-panel-inspector/tests/inspector_regression{,_sections}.rs`.
// O que falta aqui é o que nenhum dos dois cobre: a troca de SELEÇÃO entre dois
// quadros — publicar a entidade A, pintar, publicar a B, pintar, e afirmar que o
// estado do campo de texto voltou a `Normal`. As tabelas novas exercitam UM
// snapshot de cada vez, de propósito; esta é a costura ENTRE quadros.
#[cfg(any())]
#[test]
fn selection_switch_resets_entity_name_input_state_to_normal() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    // 1) Frame 1: select entity A, mark its TextInput Focused
    //    (simulating user click on the field).
    hero.inspector.name = Some(InspectorNameInfo {
        entity_bits: 0xAAAA_0001,
    });
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xAAAA_0001,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
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
    });
    hero.inspector.transform = Some(InspectorTransformInfo {
        entity_bits: 0xBBBB_0002,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
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
// ⏸️ POR MIGRAR. O destino EXISTE (ver acima).
// `seam_render_source.rs` já prova que cada botão de Strategy levanta a sua ação,
// que clicar no ativo não age e que um Reimport desativado não age; e a onda 2
// prova que nenhum deles age sem sprite publicada. O que continua sem prova é o
// RESÍDUO VISUAL: um botão momentâneo que fica `Pressed` depois do clique. Só
// `EqualizeCorners` repõe `ButtonState::Normal` hoje — se essa é a lei, ela vale
// para os três de Strategy também, e a resposta é de produto antes de ser de teste.
#[cfg(any())]
#[test]
fn strategy_click_resets_button_state_to_normal() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.inspector.sprite = Some(InspectorSpriteInfo {
        emissive: 0.0,
        entity_bits: 0x00C0_FFEE,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Individual { texture_id: 1 },
        source_pixels: Some((64, 64)),
        can_reimport: true,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        frame: 0,
        tint: [1.0, 1.0, 1.0, 1.0],
        self_tint: [1.0, 1.0, 1.0, 1.0],
        per_corner_tint: [[1.0, 1.0, 1.0, 1.0]; 4],
        region_enabled: false,
        region_rect: [0.0, 0.0, 0.0, 0.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
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

/// Clicking the Trim Transparency action pill pushes a generic
/// `EditorAction::OneShotImageOp { tool_id: "trim_transparency" }` onto
/// the bus capturing the current `gizmo_selection`. Wave 2.5 PR 11.8b1
/// (bus migration) + ADR-0040 TG-A (generic variant). When nothing is
/// selected, the bus stays empty (click still consumed).
#[test]
fn click_on_trim_pill_raises_pending_with_selection() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.selection = None;
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
    assert!(hero.bus.is_empty());

    hero.gizmo.selection = Some(0xDEAD_BEEF);
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 0xDEAD_BEEF,
        }]
    );
}

/// Make Square pill mirrors the Trim bus-push semantics.
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
        vec![EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 0xCAFE_BABE,
        }]
    );
}

/// Bg Removal pill raises `EditorAction::ActivateTool { tool_id:
/// "bgremoval" }` (ADR-0040 TG-A generic activation). The Apply /
/// commit is raised shell-side as `OneShotImageOp { tool_id:
/// "bgremoval", entity_bits }` when the tool's panel Toggle fires.
#[test]
fn click_on_bgremoval_pill_raises_activate_intent() {
    crate::test_support::ensure_panel_registry();
    use crate::action_bus::EditorAction;
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(hero.bus.is_empty());
    assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_BGREMOVAL)));
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::ActivateTool {
            tool_id: "bgremoval"
        }]
    );
}

#[test]
fn paint_left_rail_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hits = HitIndex::new();
    let store = WidgetStore::with_capacity(32);
    paint_left_rail(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &store,
        false,
        &crate::motion::UiMotion::default(),
    );
}

#[test]
fn paint_left_rail_painter_mode_smoke() {
    // Painter mode + the Shapes flyout open: exercises the paint-tool entries
    // and the flyout column (different geometry path than object mode).
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hits = HitIndex::new();
    let mut store = WidgetStore::with_capacity(32);
    super::left_rail::populate(&mut store);
    store.set_painter_shapes_flyout_open(true);
    paint_left_rail(
        &layout,
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hits,
        &store,
        true,
        &crate::motion::UiMotion::default(),
    );
    // The flyout's shape chips are hit-registered while open.
    assert!(
        hits.rect_for(crate::ids::PAINTER_RAIL_SHAPE_ELLIPSE)
            .is_some()
    );
    // The painter Brush tool chip is hit-registered in painter mode.
    assert!(hits.rect_for(crate::ids::PAINTER_RAIL_BRUSH).is_some());
    // The object-mode transform tools are NOT painted in painter mode.
    assert!(hits.rect_for(crate::ids::TOOL_ROTATE).is_none());
}

// ⏸️ POR MIGRAR, e NÃO subsumido — a distinção importa. Todos os testes das duas
// tabelas novas pintam o painel do Inspector com snapshot publicado, portanto o
// smoke ao NÍVEL DO PAINEL está coberto muitas vezes. O que este cobria é outra
// coisa: `paint_hero_screen` — o ecrã inteiro, com chrome, layout e o painel lá
// dentro. Migrá-lo para a crate do painel perderia exatamente a parte que ele mede.
#[cfg(any())]
#[test]
fn paint_inspector_smoke_with_selection() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let sel = fixture::default_selection();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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

// ⏸️ POR MIGRAR — o par do de cima, na metade da ausência (sem seleção).
#[cfg(any())]
#[test]
fn paint_inspector_smoke_no_selection() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
fn paint_bottom_hud_smoke() {
    let layout = HeroLayout::for_viewport(ipad12_viewport());
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
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
    let mut text = TextSystem::without_system_fonts();
    paint_selection_overlay(&layout, &sel, &mut scene, &mut text, Theme::Forge);
}

// ─────────────── M14.6 F: per-row context-menu apply_event ────────────────

/// Stage a closed HierarchyRow snapshot so `apply_event` can read
/// it via `consume_last_context_menu`. Mirrors what dispatch does
/// on the menu-closing Down → next-frame-Click sequence.
///
/// ADR-0029 Phase C.2: all callers of this helper moved to
/// `crates/ph2d-panel-hierarchy/tests/`; the function stays here
/// under `#[cfg(any())]` as a documented reference for the panel-
/// crate tests but is excluded from compilation.
#[cfg(any())]
fn stage_hierarchy_row_snapshot(hero: &mut HeroScreen, row: NodeId) {
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::HierarchyRow { row },
        });
    hero.store.close_context_menu();
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

// ───────────── W3.E4: timeline segment preset menu (chrome side) ─────────────

/// The segment these tests right-click.
const SCOPE: crate::interaction::TimelineInterpScope =
    crate::interaction::TimelineInterpScope::Key { target: 4, key: 9 };

/// Stage the menu the way dispatch really leaves it: the Down that precedes the
/// item Click already CLOSED it, parking the request in `last_context_menu`. A
/// handler that reads only `context_menu()` passes a naive test and ships dead.
fn stage_closed_timeline_menu(hero: &mut HeroScreen, kind: crate::interaction::ContextMenuKind) {
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind,
        });
    hero.store.close_context_menu();
}

#[test]
fn timeline_segment_menu_parks_each_leaf_pick_for_the_shell() {
    crate::test_support::ensure_panel_registry();
    use crate::interaction::{ContextMenuKind, TL_NO_EASE_MODE};
    // EVERY non-cascade row of the table the menu paints — walked, not hand-listed,
    // so a row added to `TIMELINE_SEGMENT_MENU` (this once dropped `Nearest`) is
    // proven to park a pick instead of shipping a dead item. The three cascade rows
    // open the submenu instead (covered by the cascade test below).
    let cascades = [
        ids::CTX_MENU_TL_EASE_IN,
        ids::CTX_MENU_TL_EASE_OUT,
        ids::CTX_MENU_TL_EASE_INOUT,
    ];
    let mut leaves = 0;
    for &(item, _, _) in ids::TIMELINE_SEGMENT_MENU.iter() {
        if cascades.contains(&item) {
            continue;
        }
        leaves += 1;
        let mut hero = HeroScreen::new(NodeId(1));
        stage_closed_timeline_menu(&mut hero, ContextMenuKind::TimelineSegment { scope: SCOPE });
        assert!(hero.apply_event(WidgetEvent::Click(item)), "not consumed");
        let pick = hero.pending_timeline_interp.take().expect("pick parked");
        assert_eq!((pick.scope, pick.item), (SCOPE, item));
        assert_eq!(pick.mode, TL_NO_EASE_MODE, "leaf rows carry no mode");
        assert!(hero.store.last_context_menu().is_none(), "request consumed");
    }
    // Hold, Nearest, Linear, Custom, Rove — the five today. A guard so the walk
    // above can never silently cover nothing.
    assert_eq!(
        leaves, 5,
        "the segment menu's non-cascade leaf count changed"
    );
}

#[test]
fn a_cascade_row_opens_the_family_submenu_for_its_mode_and_sets_nothing() {
    crate::test_support::ensure_panel_registry();
    use crate::interaction::ContextMenuKind;
    for (row, mode) in [
        (ids::CTX_MENU_TL_EASE_IN, ids::TL_EASE_MODE_IN),
        (ids::CTX_MENU_TL_EASE_OUT, ids::TL_EASE_MODE_OUT),
        (ids::CTX_MENU_TL_EASE_INOUT, ids::TL_EASE_MODE_INOUT),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        stage_closed_timeline_menu(&mut hero, ContextMenuKind::TimelineSegment { scope: SCOPE });
        assert!(hero.apply_event(WidgetEvent::Click(row)), "not consumed");
        assert_eq!(
            hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::TimelineSegmentEase { scope: SCOPE, mode }),
            "the cascade must carry the segment AND its mode"
        );
        assert!(
            hero.pending_timeline_interp.is_none(),
            "a cascade row sets no interpolation"
        );
    }
}

#[test]
fn a_family_click_carries_the_mode_it_inherited_from_the_cascade() {
    crate::test_support::ensure_panel_registry();
    use crate::interaction::ContextMenuKind;
    let mut hero = HeroScreen::new(NodeId(1));
    stage_closed_timeline_menu(
        &mut hero,
        ContextMenuKind::TimelineSegmentEase {
            scope: SCOPE,
            mode: ids::TL_EASE_MODE_OUT,
        },
    );
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_TL_FAM_BOUNCE)));
    let pick = hero.pending_timeline_interp.take().expect("pick parked");
    assert_eq!(pick.item, ids::CTX_MENU_TL_FAM_BOUNCE);
    assert_eq!(pick.mode, ids::TL_EASE_MODE_OUT);
}

#[test]
fn a_family_click_with_no_cascade_behind_it_parks_nothing() {
    crate::test_support::ensure_panel_registry();
    use crate::interaction::ContextMenuKind;
    // The tables are public: a family id reaching the top-level menu would have
    // no mode, and must not park a pick the shell cannot resolve.
    let mut hero = HeroScreen::new(NodeId(1));
    stage_closed_timeline_menu(&mut hero, ContextMenuKind::TimelineSegment { scope: SCOPE });
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_TL_FAM_SINE));
    assert!(hero.pending_timeline_interp.is_none());
}

#[test]
fn a_timeline_preset_click_with_no_menu_behind_it_is_ignored() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_TL_HOLD));
    assert!(hero.pending_timeline_interp.is_none());
}

#[test]
fn every_timeline_menu_row_is_registered_and_hittable() {
    // A row painted into the menu but never `register`ed is a menu item that
    // silently does nothing — the failure mode this table-driven wiring exists
    // to make impossible.
    //
    // ⚠️ **Este gate JÁ EXISTIA e estava VERDE sobre o defeito** (Enio, 2026-07-31: *"o
    // menu de Easing não está aceitando escolher Smooth"*): ele varria DUAS tabelas
    // escritas à mão, e a `TIMELINE_FADE_MENU` era a décima-primeira. Um gate que enumera
    // seus leitores apodrece exatamente como o código que ele vigia
    // ([[feedback_a_condition_that_enumerates_its_readers_rots]]) — agora ele varre a
    // lista ÚNICA, e a segunda metade fecha o vão que sobra: **toda tabela que um escopo
    // de menu PINTA** tem de estar registrada, mesmo que alguém esqueça de acrescentá-la
    // à lista.
    crate::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    let mut rows = 0_usize;
    for (id, label, _) in ids::ALL_TIMELINE_MENUS.iter().copied().flatten() {
        rows += 1;
        assert!(
            hero.store.get(*id).is_some(),
            "menu row {label:?} is painted but never registered in populate"
        );
    }
    assert!(rows > 40, "a lista não pode encolher em silêncio: {rows}");

    use crate::interaction::TimelineInterpScope as S;
    for scope in [
        S::Key { target: 1, key: 0 },
        S::Column { t_bits: 0 },
        S::StripFade {
            lane: 0,
            strip: 0,
            edge: 0,
        },
    ] {
        for (id, label, _) in scope.menu_table() {
            assert!(
                hero.store.get(*id).is_some(),
                "a row {label:?} que o escopo {scope:?} pinta não está registrada"
            );
        }
    }
}

/// **Os botões Undo/Redo da barra chegam ao desfazer do EDITOR.**
///
/// O Enio: *"undo/redo no sistema"*. O Ctrl+Z funcionava; os **botões**, não — o Undo
/// levantava `UndoImageEdit` (o desfazer de IMAGEM, single-level: Trim / Make Square / Bg
/// Removal), então mover uma forma e clicar em Undo não fazia nada. E o **Redo não
/// despachava coisa alguma**: pintado, clicável, órfão.
///
/// O gate anterior deste arquivo pedia que `TOOL_REDO` estivesse **no store** — e ele estava.
/// Registrado não é despachado, do mesmo jeito que pintado não é populado.
#[test]
fn the_rail_undo_and_redo_buttons_reach_the_editor_undo() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOOL_UNDO)),
        "o clique no Undo tem de ser consumido por ALGUÉM"
    );
    assert!(chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(ids::TOOL_REDO)
    ));

    let raised: Vec<EditorAction> = hero.bus.drain().collect();
    assert_eq!(
        raised,
        vec![
            EditorAction::UndoStep { redo: false },
            EditorAction::UndoStep { redo: true },
        ],
        "os dois vão para o MESMO caminho do Ctrl+Z, não para o undo de imagem"
    );
}

/// **Nenhum chip PINTADO na barra pode ser órfão.** Um botão pintado que ninguém despacha é
/// indistinguível de um botão quebrado — e foi exatamente assim que o Redo passou meses na
/// tela sem fazer nada, com um gate ao lado afirmando que ele estava "no store".
///
/// A lista vem de `left_rail::rail_entries` — **a mesma que o rail pinta**. Escrevê-la à mão
/// aqui seria repetir o erro num nível acima: uma lista escrita à mão drifta da tela.
/// Acrescente um chip ao rail sem lhe dar um handler e este gate nasce vermelho.
#[test]
fn every_painted_rail_button_is_dispatched_by_somebody() {
    crate::test_support::ensure_panel_registry();
    for painter_active in [false, true] {
        let probe = HeroScreen::new(NodeId(1));
        let ids: Vec<NodeId> = super::left_rail::rail_entries(&probe.store, painter_active)
            .iter()
            .filter_map(super::super::super::widget::ToolRailEntry::node_id)
            .collect();
        assert!(
            ids.len() >= 8,
            "o rail tem chips (modo painter={painter_active})"
        );
        let mut dead = Vec::new();
        for id in ids {
            let mut hero = HeroScreen::new(NodeId(1));
            if !chrome::dispatch_all(&mut hero, WidgetEvent::Click(id)) {
                dead.push(id);
            }
        }
        assert!(
            dead.is_empty(),
            "chips PINTADOS no rail (painter={painter_active}) que ninguém despacha \
             — botões mortos: {dead:?}"
        );
    }
}

// ── A PORTA das réguas (plano 25 §9, a W6.2) ────────────────────────────────

/// **Visível ⇔ vivo**, e as duas condições, nenhuma bastando sozinha.
///
/// ⚠️ A segunda metade — a ferramenta vetorial em mãos — é uma CORREÇÃO, não escopo escolhido:
/// a faixa da régua **ocupa** a borda do canvas e o gesto dela corre antes de toda ferramenta,
/// então uma régua permanente comeria o pen-down do **Painter** nos 20 px de cima (o artista
/// pincela ali e nasce uma guia). Este gate existe porque a mutação que apaga essa metade
/// sobrevivia a todos os outros — a correção estava shipada e desguardada.
#[test]
fn the_rulers_are_live_only_with_the_vector_tool_and_the_toggle_on() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.rulers_visible = true;

    hero.panel_visibility.insert("vector", false);
    assert!(
        !hero.rulers_live(),
        "sem a ferramenta vetorial a faixa não existe — senão ela come o pen-down do Painter"
    );

    hero.panel_visibility.insert("vector", true);
    assert!(hero.rulers_live(), "com a ferramenta e o interruptor, viva");

    hero.view.rulers_visible = false;
    assert!(
        !hero.rulers_live(),
        "o interruptor do artista continua mandando — é ele o *lock* das guias"
    );
}

// ---------------------------------------------------------------------------
// Settings → Motion: o carácter da UI viva alcançável pelo artista.
//
// ⚠️ Os gates do `motion.rs` provam a LEI (a mola, a interrupção, os dois eixos); estes provam a
// PORTA — que existe um gesto que chega àquela lei. Sem eles o `set_character` era uma função sem
// chamador de produto: viva na suíte, inalcançável na tela.
// ---------------------------------------------------------------------------

#[test]
fn settings_motion_cascade_opens_the_motion_submenu() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsMenu,
        });
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_MOTION));
    assert!(consumed);
    assert!(matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(crate::interaction::ContextMenuKind::SettingsMotionSubmenu)
    ));
}

/// Escolher o carácter escreve no DONO do facto (`hero.motion`) e fecha o menu.
///
/// ⚠️ A fixture pica a opção NÃO-default de propósito: com Discreto dos dois lados, *escreveu* e
/// *não fez nada* são indistinguíveis.
#[test]
fn picking_a_character_writes_it_and_closes_the_menu() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert_eq!(
        hero.motion.character(),
        crate::motion::UiCharacter::Discrete,
        "o default do produto"
    );
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsMotionSubmenu,
        });
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_MOTION_EXPRESSIVE)));
    assert_eq!(
        hero.motion.character(),
        crate::motion::UiCharacter::Expressive
    );
    assert!(
        hero.store.context_menu().is_none(),
        "a escolha fecha o menu"
    );

    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsMotionSubmenu,
        });
    assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_MOTION_DISCRETE)));
    assert_eq!(
        hero.motion.character(),
        crate::motion::UiCharacter::Discrete,
        "e volta — um rádio anda nos dois sentidos"
    );
}

/// **A row do reduced motion é um TOGGLE, não uma escolha.**
///
/// ⚠️ Um `set(true)` passaria no gate ingénuo (*"clicar liga"*) e seria uma porta de sentido único:
/// o artista liga e nunca desliga, com o bullet a dizer a verdade sobre um interruptor que ele não
/// consegue mexer. É por isso que o gate clica DUAS vezes.
#[test]
fn the_reduced_motion_row_toggles_it_does_not_only_turn_it_on() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(!hero.motion.reduced_motion());
    for expected in [true, false] {
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::SettingsMotionSubmenu,
            });
        assert!(hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_MOTION_REDUCED)));
        assert_eq!(hero.motion.reduced_motion(), expected);
    }
}

/// **Os dois eixos continuam independentes DEPOIS de passarem pelo menu** — *Expressivo + reduced*
/// tem de ser alcançável por gestos reais, não só construível na memória.
///
/// ⚠️ É este gate que morre se alguém colapsar a submenu num selector de três posições.
#[test]
fn the_menu_can_reach_expressive_with_reduced_motion_on() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    for id in [
        ids::CTX_MENU_MOTION_EXPRESSIVE,
        ids::CTX_MENU_MOTION_REDUCED,
    ] {
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::SettingsMotionSubmenu,
            });
        assert!(hero.apply_event(WidgetEvent::Click(id)));
    }
    assert_eq!(
        hero.motion.character(),
        crate::motion::UiCharacter::Expressive
    );
    assert!(hero.motion.reduced_motion());
    assert!(
        hero.motion.law(crate::motion::Role::Travel).is_none(),
        "e a combinação FAZ alguma coisa: reduced mata o percurso mesmo no Expressivo"
    );
}

/// ⭐ **O CORPO DE UM PAINEL DESLIZA SEM PASSAR DO ALVO — e este gate lê o TIQUE, não a lei.**
///
/// ⚠️ **Os gates do `motion.rs` provam que o `Role::Surface` não ultrapassa; este prova que a
/// rolagem PEDE esse papel.** A distinção não é cerimónia — a versão que o Enio reprovou tinha a
/// lei toda certa e o tique a pedir `Role::Travel`, e nenhum gate de lei podia vê-lo.
///
/// Corre em **Expressivo**, que é o carácter em que a diferença existe (e o que o
/// `~/.ph2d/prefs.txt` dele diz). *Mutação: `Role::Surface` → `Role::Travel` no `live.rs` ⇒ a
/// superfície passa ~15% e sangra.*
#[test]
fn the_panel_body_glides_without_passing_the_target() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    const TARGET: f32 = 200.0;
    let panel = NodeId(4242);
    hero.store.set_panel_scroll(panel, 0.0);
    hero.tick_motion(1.0 / 60.0);
    hero.store.set_panel_scroll(panel, TARGET);
    let mut peak = 0.0_f32;
    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
        peak = peak.max(hero.store.panel_scroll(panel));
    }
    assert!(
        peak <= TARGET + 0.01,
        "a superfície passou {:.2} px do sítio para onde a roda a mandou",
        peak - TARGET
    );
    assert!(
        (hero.store.panel_scroll(panel) - TARGET).abs() < 1.0,
        "e chegou lá: {}",
        hero.store.panel_scroll(panel)
    );
}

/// ⭐ **A PRIMEIRA dobra de uma secção ANIMA** — e sem a partida gravada no `toggle_collapsed`
/// ela saltava.
///
/// A lei do substrato é que *a primeira vista de um id CHEGA ao alvo*. Uma secção nunca tocada não
/// está no mapa, então no instante em que o utilizador a fecha o relógio vê-a pela primeira vez
/// **já fechada** e assenta ali. O defeito seria **uma vez por secção por sessão** — a forma mais
/// fácil de nunca ser reproduzido por quem o reporta.
///
/// *Mutação: tirar o `or_insert` da partida no `toggle_collapsed` ⇒ o `t` cai a 0,0 no primeiro
/// quadro e este gate sangra.*
#[test]
fn the_first_fold_of_a_section_animates_instead_of_snapping() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    let sec = NodeId(9911);
    // CONTROLO: nunca tocada ⇒ aberta, e o neutro di-lo sem o relógio ter corrido.
    assert!((hero.store.section_open_live(sec) - 1.0).abs() < f32::EPSILON);

    hero.store.toggle_collapsed(sec);
    // ⚠️ DOIS quadros, e a razão é a ordem do tique: o `advance` corre no TOPO, então o quadro do
    //    clique semeia a partida e re-alveja, e o seguinte é o primeiro que integra. É a mesma
    //    lei que faz a cascata da paleta existir (*todo cartão nasce em 0 no quadro da abertura*)
    //    — um gate de um quadro só mediria a partida e chamar-lhe-ia salto.
    hero.tick_motion(1.0 / 60.0);
    hero.tick_motion(1.0 / 60.0);
    let t = hero.store.section_open_live(sec);
    assert!(
        t < 1.0 && t > 0.0,
        "a estreia da dobra SALTOU: t = {t} no primeiro quadro (esperado entre 0 e 1)"
    );

    // e chega ao fim
    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
    }
    assert!(
        hero.store.section_open_live(sec).abs() < 1e-3,
        "a dobra não fechou: {}",
        hero.store.section_open_live(sec)
    );
}

/// **E NASCER fechada não é dobrar-se.** O `populate` de vários painéis chama
/// `set_collapsed(id, true)` no arranque; essa secção tem de aparecer fechada, não a fechar-se
/// sozinha no primeiro quadro que o artista vê.
///
/// ⚠️ É a metade OPOSTA do gate acima, e é ela que impede a cura de ser *animar tudo o que entra
/// no mapa*. *Mutação: semear a partida em `set_collapsed` (e não só no `toggle`) ⇒ a secção
/// nasce aberta e fecha-se à vista, e este gate sangra.*
#[test]
fn a_section_born_collapsed_does_not_fold_itself_on_the_first_frame() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    let sec = NodeId(9912);
    hero.store.set_collapsed(sec, true); // a rota do `populate`
    hero.tick_motion(1.0 / 60.0);
    assert!(
        hero.store.section_open_live(sec).abs() < f32::EPSILON,
        "uma secção que NASCE fechada animou: t = {}",
        hero.store.section_open_live(sec)
    );
}

// ---------------------------------------------------------------------------
// A CORDA do card de Fill (`crate::tether`).
//
// ⚠️ Os gates do `tether.rs` provam a FÍSICA (o relógio, as pontas, o carácter); estes provam a
// LIGAÇÃO — que ela está atada a duas coisas que de facto existem e que se movem.
// ---------------------------------------------------------------------------

/// A âncora é onde o card NASCEU, e arrastá-lo não a leva junto — senão a corda descreve uma
/// relação que já não existe (as duas pontas no mesmo sítio, comprimento zero, nada desenhado).
#[test]
fn dragging_the_fill_card_moves_it_and_leaves_the_anchor_where_it_was_born() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_fill_modal(300.0, 200.0, 0.5);
    assert_eq!(hero.store.fill_modal_anchor(), Some((300.0, 200.0)));
    hero.store.move_fill_modal(120.0, -40.0);
    assert_eq!(hero.store.fill_modal_pos(), Some((420.0, 160.0)));
    assert_eq!(
        hero.store.fill_modal_anchor(),
        Some((300.0, 200.0)),
        "a âncora não se move: ela é o ponto de largada do ColorDrop"
    );
}

/// Fechar leva as DUAS metades. ⚠️ Elas vivem no mesmo campo, então isto é estrutural — o gate
/// existe para que uma refactorização que as separe tenha de o partir primeiro.
#[test]
fn closing_the_fill_card_takes_the_anchor_with_it() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_fill_modal(300.0, 200.0, 0.5);
    hero.store.close_fill_modal();
    assert_eq!(hero.store.fill_modal_pos(), None);
    assert_eq!(hero.store.fill_modal_anchor(), None);
}

/// A corda é avançada pelo `tick_motion` — a MESMA porta do resto da UI viva — e **esquece a pose
/// quando o card fecha**.
///
/// ⚠️ Sem o esquecimento, a largada seguinte noutro canto do ecrã faria a corda **voar** do sítio
/// onde a anterior morreu: um rasto que descreve um gesto que já acabou.
#[test]
fn the_tether_follows_the_card_and_forgets_when_it_closes() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    hero.store.open_fill_modal(300.0, 200.0, 0.5);
    hero.store.move_fill_modal(200.0, 0.0);
    for _ in 0..40 {
        hero.tick_motion(1.0 / 60.0);
    }
    let hanging = hero.tether.points()[crate::tether::NODES / 2];
    assert!(
        hanging[1] > 200.0,
        "a meio, a corda tem de estar ABAIXO das pontas (y={}) — senão não pendura",
        hanging[1]
    );

    // Fecha e reabre longe: o primeiro quadro tem de ser a recta NOVA, não um voo do sítio antigo.
    hero.store.close_fill_modal();
    hero.tick_motion(1.0 / 60.0);
    hero.store.open_fill_modal(900.0, 700.0, 0.5);
    hero.store.move_fill_modal(200.0, 0.0);
    hero.tick_motion(1.0 / 60.0);
    for p in hero.tether.points() {
        assert!(
            (p[1] - 700.0).abs() < 1e-3,
            "a corda re-nasceu na recta da largada nova; encontrei y={}",
            p[1]
        );
    }
}

/// **A COSTURA pergunta ao carácter** — e este gate nasceu de uma mutação que sobreviveu a todos os
/// outros: cravar `true` no lugar de `motion.decorates()` dentro do `tick_motion` deixava a corda a
/// simular em Discreto com a suíte inteira verde.
///
/// ⚠️ É a lição que este repo já nomeou: *um gate de unidade é CEGO à fiação*. O `tether.rs` prova
/// que o MÓDULO honra o `simulate` que lhe dão; só um gate no hero prova que quem lho dá é o
/// carácter, e não uma constante.
#[test]
fn the_seam_asks_the_character_it_does_not_hardcode_the_rope() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.open_fill_modal(300.0, 200.0, 0.5);
    hero.store.move_fill_modal(300.0, 0.0);

    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    for _ in 0..60 {
        hero.tick_motion(1.0 / 60.0);
    }
    assert!(
        hero.tether.points()[crate::tether::NODES / 2][1] > 210.0,
        "premissa: em Expressivo ela pendura — sem isto o resto do gate não distingue nada"
    );

    hero.motion
        .set_character(crate::motion::UiCharacter::Discrete);
    for _ in 0..60 {
        hero.tick_motion(1.0 / 60.0);
    }
    for p in hero.tether.points() {
        assert!(
            (p[1] - 200.0).abs() < 1e-3,
            "em Discreto a costura tem de passar `simulate = false`; a corda caiu para y={}",
            p[1]
        );
    }

    // E o reduced motion mata-a pelo MESMO caminho, sem uma segunda pergunta.
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    hero.motion.set_reduced_motion(true);
    for _ in 0..60 {
        hero.tick_motion(1.0 / 60.0);
    }
    for p in hero.tether.points() {
        assert!(
            (p[1] - 200.0).abs() < 1e-3,
            "reduced motion mata a decoração mesmo em Expressivo; y={}",
            p[1]
        );
    }
}

// ---------------------------------------------------------------------------
// O POLEGAR de uma scrollbar — o eixo que estava inteiro e nunca se movia.
// ---------------------------------------------------------------------------

/// ⭐ **O POLEGAR DESVANECE nos DOIS sentidos, em vez de saltar.**
///
/// Medido antes da cura, com o produto: `t = 1` nos **quatro** instantes — nunca tocado, sob o
/// ponteiro, assente, e cem quadros depois de sair. As 22 barras do app reagiam e SALTAVAM, com a
/// suíte inteira verde, porque o `hover_targets` deriva o *aceso* do estado GUARDADO e um polegar
/// guarda `Plain` — *nenhuma opinião*.
///
/// ⚠️ **O CONTROLO é a primeira asserção**, e é ela que mantém todo gate de painel a medir o que
/// media: um id que o relógio nunca viu publica [`crate::motion::SETTLED`], o token duro, o mundo
/// pré-substrato byte a byte.
///
/// *Mutação: tirar o `scrollbar_hover_targets` do `tick_hover` ⇒ `t = 1` outra vez e este gate
/// sangra na primeira metade.*
#[test]
fn a_scrollbar_thumb_fades_in_and_out_instead_of_snapping() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    let thumb = crate::widget::INSPECTOR_SCROLLBAR_ID;

    // CONTROLO: nunca tocado ⇒ o neutro, sem o relógio ter corrido.
    let (_, cold) = hero.store.scrollbar_visual(thumb);
    assert!(
        (cold - crate::motion::SETTLED).abs() < f32::EPSILON,
        "um polegar que o relogio nunca viu tem de publicar o NEUTRO: {cold}"
    );

    hero.store.set_hot(Some(thumb));
    hero.tick_motion(1.0 / 60.0);
    hero.tick_motion(1.0 / 60.0);
    let (state, t) = hero.store.scrollbar_visual(thumb);
    assert_eq!(state, crate::widget::ScrollbarState::Hovered);
    assert!(
        t > 0.0 && t < 1.0,
        "o polegar SALTOU ao acender: t = {t} (esperado entre 0 e 1)"
    );

    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
    }
    assert!(
        (hero.store.scrollbar_visual(thumb).1 - 1.0).abs() < 1e-3,
        "o polegar nao chegou ao extremo quente: {}",
        hero.store.scrollbar_visual(thumb).1
    );

    // E a metade que um id só-quente NAO teria: ele APAGA.
    hero.store.set_hot(None);
    hero.tick_motion(1.0 / 60.0);
    hero.tick_motion(1.0 / 60.0);
    let leaving = hero.store.scrollbar_visual(thumb).1;
    assert!(
        leaving < 1.0,
        "o polegar acendeu e ficou preso no quente: t = {leaving}"
    );
    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
    }
    let rest = hero.store.scrollbar_visual(thumb).1;
    assert!(
        rest.abs() < f32::EPSILON,
        "o voo de saida nao chegou ao ZERO exacto: t = {rest} \n\
         (a mola Expressiva ULTRAPASSA — se a condicao de paragem for `> 0.0` ela e largada em \n\
         −0,0109 e a track e podada a meio do caminho)"
    );
}

/// **RE-ENTRAR anima outra vez** — a lei da PARTIDA fria.
///
/// A lei do substrato é *a primeira vista de um id CHEGA ao alvo*, então depois de um desvanecer
/// completo a track foi podada e o quadro em que o rato volta veria o polegar pela primeira vez
/// **já aceso**. É um defeito de uma-vez-por-re-entrada: o primeiro hover anima e o segundo salta.
///
/// *Mutação: tirar a semente `animate(id, 0.0)` do `tick_hover` ⇒ o `t` do primeiro quadro é `1.0`
/// e este gate sangra.*
#[test]
fn re_entering_a_thumb_animates_again_instead_of_snapping() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    let thumb = crate::widget::HIERARCHY_SCROLLBAR_ID;

    hero.store.set_hot(Some(thumb));
    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
    }
    hero.store.set_hot(None);
    for _ in 0..240 {
        hero.tick_motion(1.0 / 60.0);
    }

    // Re-entra.
    hero.store.set_hot(Some(thumb));
    hero.tick_motion(1.0 / 60.0);
    let t = hero.store.scrollbar_visual(thumb).1;
    assert!(
        t > 0.0 && t < 1.0,
        "a RE-ENTRADA saltou: t = {t} no primeiro quadro"
    );
}

/// **E um polegar que arrefeceu é ESQUECIDO pelo relógio, sem ficar quente.**
///
/// Duas metades que se contradizem se qualquer uma faltar: a track tem de sair do mapa (senão a
/// alegação de custo do substrato — *lembrar é O(widgets tocados recentemente)* — passa a ser
/// falsa, porque conduzir um id todo quadro reinicia o `idle_s` e ele nunca é podado) **e** o
/// `hover_live` tem de guardar o `0.0` (apagar a entrada devolveria [`crate::motion::SETTLED`],
/// e o polegar ficaria a pintar o token QUENTE para sempre).
///
/// *Mutação: trocar a paragem por «conduzir sempre que estiver no mapa» ⇒ a track nunca é podada e
/// a primeira metade sangra.*
#[test]
fn a_thumb_that_cooled_is_forgotten_by_the_clock_but_not_by_the_painter() {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.motion
        .set_character(crate::motion::UiCharacter::Expressive);
    let thumb = crate::widget::INSPECTOR_SCROLLBAR_ID;

    hero.store.set_hot(Some(thumb));
    for _ in 0..120 {
        hero.tick_motion(1.0 / 60.0);
    }
    hero.store.set_hot(None);
    for _ in 0..240 {
        hero.tick_motion(1.0 / 60.0);
    }

    assert!(
        hero.motion.get(thumb).is_none(),
        "a track do polegar ficou no mapa para sempre — o `remembered` do substrato mente"
    );
    let (state, t) = hero.store.scrollbar_visual(thumb);
    assert_eq!(state, crate::widget::ScrollbarState::Normal);
    assert!(
        t.abs() < f32::EPSILON,
        "o polegar arrefecido publica {t}, e nao o `0.0` que faz o pintor ficar no repouso"
    );
}

/// ⭐ **A costura da UNIDADE DE ÂNGULO, ponta a ponta** — o irmão de
/// `settings_unit_submenu_options_flip_project_display_unit`, e a prova de que o clique
/// chega ao `project`.
///
/// ⚠️ **Verde-de-compilação não prova nada aqui.** As sete pontas de um item de menu
/// (id · registo · linha · variante de `ContextMenuKind` · handler · dispatch gerado ·
/// campo) compilam todas isoladas: faltar **uma** dá um item que é pintado, aceita o
/// clique e **não faz nada**. É este teste que dirige o evento real.
#[test]
fn settings_angle_submenu_options_flip_project_display_angle() {
    let mut hero = HeroScreen::new(NodeId(1));
    assert_eq!(
        hero.project.display_angle,
        crate::project::DisplayAngle::Degrees,
        "o default tem de ser Degrees — é o que preserva o comportamento anterior ao bit"
    );
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsAngleSubmenu,
        });
    let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_ANGLE_RADIANS));
    assert!(consumed, "o clique em Radians tem de ser consumido");
    assert_eq!(
        hero.project.display_angle,
        crate::project::DisplayAngle::Radians,
        "display_angle tem de virar Radians"
    );
    assert!(
        hero.store.context_menu().is_none(),
        "o menu tem de fechar depois da escolha"
    );
    // E de volta — um selector que só anda para um lado não é um selector.
    hero.store
        .open_context_menu(crate::interaction::ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::SettingsAngleSubmenu,
        });
    let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_ANGLE_DEGREES));
    assert_eq!(
        hero.project.display_angle,
        crate::project::DisplayAngle::Degrees
    );
}

/// ⛔ **A entrada do menu tem de ser ALCANÇÁVEL a partir do Settings** — o teste acima
/// prova que o clique funciona *se* alguém o der, e este prova que há por onde dar.
///
/// ⚠️ É a distinção que o `CLAUDE.md` §5.0 cobra: *um controlo nunca pintado e um morto
/// sob o dedo dão o MESMO report*. Sem esta metade, apagar a linha do `SettingsMenu`
/// deixaria o teste de cima **verde** sobre uma entrada que ninguém consegue abrir.
#[test]
fn the_angle_unit_entry_is_reachable_from_the_settings_menu() {
    let rows = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::SettingsMenu,
    );
    assert!(
        rows.iter()
            .any(|(id, _, _)| *id == ids::CTX_MENU_SETTINGS_ANGLE),
        "a entrada 'Angle unit' tem de estar no menu Settings"
    );
    let sub = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::SettingsAngleSubmenu,
    );
    let ids_in_sub: Vec<_> = sub.iter().map(|(id, _, _)| *id).collect();
    assert!(
        ids_in_sub.contains(&ids::CTX_MENU_ANGLE_DEGREES)
            && ids_in_sub.contains(&ids::CTX_MENU_ANGLE_RADIANS),
        "o submenu tem de oferecer as DUAS opções"
    );
}

/// ⭐⭐ **A ida-e-volta cabe de volta no `f32` de origem, AO BIT.**
///
/// É o que impede uma caixa que o artista abre e fecha sem tocar de deixar o documento
/// diferente do que estava. O gate mede o caminho do produto: o `sync` mostra (largo,
/// porque o `WidgetStore` guarda `f64`) e o `event` escreve de volta (estreito).
///
/// ⚠️⚠️ **Este gate reprovou a 1ª implementação, e o defeito era meu.** Eu tinha feito
/// o caminho estreito delegar no largo — copiando a forma do [`crate::project::DisplayUnit`]
/// sem medir se ela servia. Serve lá (a regra tem um parâmetro externo e a magnitude é
/// o perigo) e **não serve aqui**: a `std` já dá `to_radians` nas duas larguras, então
/// passar pelo `f64` acrescenta uma **segunda arredondagem** — `1 ULP` em
/// `-2.7182817`. *Arredondar duas vezes não é arredondar melhor.*
#[test]
fn the_angle_round_trip_lands_back_on_the_same_bits() {
    use crate::project::DisplayAngle;
    // Ângulos que um artista de facto autora, mais dois que não são múltiplos bonitos.
    for rad in [
        0.0_f32,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        -std::f32::consts::FRAC_PI_4,
        0.123_456_79,
        -2.718_281_7,
    ] {
        for unit in [DisplayAngle::Degrees, DisplayAngle::Radians] {
            let shown = unit.from_radians_f64(f64::from(rad));
            let back = unit.to_radians(shown as f32);
            assert_eq!(
                back.to_bits(),
                rad.to_bits(),
                "{unit:?}: {rad} → {shown} → {back} não voltou aos mesmos bits"
            );
        }
    }
}

/// ⛔ **Radianos NÃO é graus** — o controlo que impede a porta de ser um no-op.
///
/// Sem ele, uma implementação em que os dois braços devolvessem o mesmo valor passaria
/// os outros dois testes: o menu comuta um enum que não muda nada.
#[test]
fn the_two_angle_units_actually_disagree() {
    use crate::project::DisplayAngle;
    let rad = std::f32::consts::FRAC_PI_2;
    let as_deg = DisplayAngle::Degrees.from_radians(rad);
    let as_rad = DisplayAngle::Radians.from_radians(rad);
    assert!(
        (as_deg - 90.0).abs() < 1e-4,
        "meio π em graus são 90, e vieram {as_deg}"
    );
    assert!(
        (as_rad - rad).abs() < 1e-6,
        "em radianos o valor passa intacto, e veio {as_rad}"
    );
    assert!(
        (as_deg - as_rad).abs() > 1.0,
        "as duas unidades têm de DISCORDAR, senão o selector é decorativo"
    );
}
