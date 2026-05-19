//! ADR-0029 Phase C.2 — `HierarchyPanel::apply_event` regression tests
//! that were originally inline in `ph2d-editor-core`'s `screens::hero::
//! tests`. Migrated here once the panel moved out of editor-core (see
//! `docs/HANDOFF_WAVE_8_PHASE_C2.md` §4.1).

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
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

/// Ported from `hero_apply_event_hierarchy_click_changes_selection`
/// — the placeholder fixture registers `Scene Root` under
/// `HIER_PLAYER`; clicking that row updates `selection.label`.
#[test]
fn hierarchy_click_changes_selection() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(ids::HIER_PLAYER));
    assert!(consumed);
    assert_eq!(
        hero.selection.as_ref().map(|s| s.label.as_str()),
        Some("Scene Root"),
    );
}
