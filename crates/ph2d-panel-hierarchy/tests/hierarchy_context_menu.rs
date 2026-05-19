//! ADR-0029 Phase C.2 — context-menu regression tests for the
//! Hierarchy panel. Ported from `ph2d-editor-core`'s
//! `screens::hero::tests::hier_menu_*` after the dev-dep-cycle gate.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, EventOutcome, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::ids;
use ph2d_panel_hierarchy::{HierarchyPanel, HierarchyState};
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<HierarchyPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn setup_hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    ph2d_panel_hierarchy::clear_live_hierarchy();
    HeroScreen::new(NodeId(1))
}

fn dispatch(hero: &mut HeroScreen, state: &mut HierarchyState, ev: WidgetEvent) -> bool {
    matches!(
        HierarchyPanel::apply_event(state, hero, ev),
        EventOutcome::Consumed | EventOutcome::Observed,
    )
}

/// Stage a closed HierarchyRow snapshot so `apply_event` can read it
/// via `consume_last_context_menu`. Mirrors what dispatch does on
/// the menu-closing Down → next-frame-Click sequence.
fn stage_hierarchy_row_snapshot(hero: &mut HeroScreen, row: NodeId) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::HierarchyRow { row },
    });
    hero.store.close_context_menu();
}

#[test]
fn hier_menu_duplicate_sets_pending_duplicate() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_500);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
    assert!(hero.store.last_context_menu().is_none());
}

#[test]
fn hier_menu_add_child_sets_pending_add_child() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_501);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_ADD_CHILD),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierAddChild { row }]);
}

#[test]
fn hier_menu_reset_transform_sets_pending() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_502);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_RESET_TRANSFORM),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierResetTransform { row }]);
}

#[test]
fn hier_menu_delete_sets_pending_delete() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_503);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDelete { row }]);
}

#[test]
fn hier_menu_click_without_snapshot_consumes_but_no_pending() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    // Defensive case: stray Click without any prior right-click
    // snapshot still consumes the event so the click doesn't
    // bubble to row selection, but no pending action is raised.
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    assert!(consumed);
    assert!(hero.bus.is_empty());
}

#[test]
fn hier_menu_one_action_per_drain() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_504);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let _ = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    let _ = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE),
    );
    let drained: Vec<_> = hero.bus.drain().collect();
    // Only Duplicate — the second Click consumed but found no
    // snapshot to attach a row to, so no Delete variant pushed.
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
}
