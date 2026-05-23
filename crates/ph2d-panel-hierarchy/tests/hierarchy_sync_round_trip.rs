//! ADR-0029 Phase C.2 — round-trip test for the host-supplied live
//! hierarchy data path. Migrated from
//! `crates/ph2d-editor-core/tests/hero_sync_round_trip.rs` after the
//! Hierarchy panel moved to its own crate (panel-state-living-inside-
//! ErasedPanel pattern); the legacy editor-core test crate could not
//! dev-dep the panel crate without creating a duplicate-crate cycle
//! through `ph2d-editor-core`. Integration tests in this crate sit
//! downstream of both and have no such restriction.
//!
//! Covers:
//!
//! 1. [`ph2d_panel_hierarchy::sync_from_hierarchy`] accepts external
//!    entries + overrides the fixture data the next time the panel
//!    renders.
//! 2. Selection echo: a click on a live row (via `HierarchyPanel::
//!    apply_event`) updates `HeroScreen::selection.label` to that
//!    entry's name, not the fixture default.
//! 3. [`ph2d_panel_hierarchy::clear_live_hierarchy`] reverts to
//!    fixture behavior.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{ErasedPanel, EventOutcome, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_panel_hierarchy::{HierarchyPanel, HierarchyState};
use std::collections::BTreeMap;
use std::sync::Once;

/// Install the typed `Hierarchy` panel into the process-wide registry
/// exactly once per process. Mirrors `ph2d_editor_core::test_support::
/// ensure_panel_registry` (which only seeds the legacy registry); we
/// need the typed one too so `HeroScreen::new` registers our
/// `HIERARCHY_ADD` / `HIER_SEARCH` / `HIER_PLAYER` slots via
/// `HierarchyPanel::populate`.
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
    // Re-clear the panel-owned live entries thread-local each test —
    // `current_live_entries` survives across tests in the same binary.
    ph2d_panel_hierarchy::clear_live_hierarchy();
    HeroScreen::new(NodeId(1))
}

/// Fire one WidgetEvent through the typed Hierarchy panel directly.
/// Bypasses `HeroScreen::apply_event` so the assertions are scoped
/// to the panel under test (mirrors the panel-as-crate boundary).
fn dispatch(hero: &mut HeroScreen, state: &mut HierarchyState, ev: WidgetEvent) -> bool {
    match HierarchyPanel::apply_event(state, hero, ev) {
        EventOutcome::Consumed | EventOutcome::Observed => true,
        EventOutcome::Ignored => false,
    }
}

fn make_entry(name: &str) -> HierarchyEntity {
    HierarchyEntity {
        name: name.into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        muted: false,
    }
}

#[test]
fn sync_from_hierarchy_caches_entries() {
    let mut hero = setup_hero();
    assert!(ph2d_panel_hierarchy::current_live_entries().is_none());

    let id_a = NodeId(100_000);
    let id_b = NodeId(100_001);
    let mut entries = BTreeMap::new();
    entries.insert(id_a, make_entry("Alpha"));
    entries.insert(id_b, make_entry("Beta"));

    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id_a, id_b], entries.clone());
    let cached = ph2d_panel_hierarchy::current_live_entries().expect("cache must be Some");
    assert_eq!(cached.len(), 2);
    assert_eq!(cached.get(&id_a).unwrap().name, "Alpha");
    assert_eq!(cached.get(&id_b).unwrap().name, "Beta");
}

#[test]
fn click_on_live_row_updates_selection_label() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let id_player = NodeId(100_005);
    let mut entries = BTreeMap::new();
    entries.insert(id_player, make_entry("Player"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id_player], entries);

    let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(id_player));
    assert!(consumed, "live click should be consumed");
    let sel = hero.selection.as_ref().expect("selection set by click");
    assert_eq!(sel.label, "Player");
    assert_eq!(sel.kind, "ENT");
}

#[test]
fn click_uses_badge_as_kind_when_present() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let id = NodeId(100_010);
    let mut entries = BTreeMap::new();
    entries.insert(
        id,
        HierarchyEntity {
            badge: Some("SPR".into()),
            ..make_entry("WithBadge")
        },
    );
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id], entries);

    dispatch(&mut hero, &mut state, WidgetEvent::Click(id));
    let sel = hero.selection.as_ref().unwrap();
    assert_eq!(sel.kind, "SPR");
}

#[test]
fn clear_live_reverts_to_fixture_behavior() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let id = NodeId(100_020);
    let mut entries = BTreeMap::new();
    entries.insert(id, make_entry("Temp"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id], entries);
    assert!(ph2d_panel_hierarchy::current_live_entries().is_some());

    ph2d_panel_hierarchy::clear_live_hierarchy();
    assert!(ph2d_panel_hierarchy::current_live_entries().is_none());

    // After clear, a click on the previously-live id is NOT consumed
    // because it's neither in fixture's hierarchy_label_for_id nor
    // in live entries.
    let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(id));
    assert!(!consumed, "stale live id click ignored after clear");
}

#[test]
fn fixture_click_still_works_in_live_mode_for_unknown_id() {
    use ph2d_editor_core::screens::hero::ids::HIER_PLAYER;

    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let live_id = NodeId(100_030);
    let mut entries = BTreeMap::new();
    entries.insert(live_id, make_entry("Live"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[live_id], entries);

    // Click HIER_PLAYER (NOT in live entries) → fixture fallback wins.
    let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(HIER_PLAYER));
    assert!(consumed);
    let sel = hero.selection.as_ref().unwrap();
    assert_eq!(sel.label, "Scene Root");
}

/// Regression: ensure `sync_from_hierarchy` ACTUALLY publishes the
/// live ids into `WidgetStore::hierarchy_order` (not just into the
/// cached `live_hierarchy_entries`). The original M14.4a wiring used
/// `init_hierarchy_order` which is idempotent — after the fixture
/// `populate` seeded `[HIER_PLAYER]`, the live-mode call was a no-op
/// and the hierarchy panel painted zero rows. The fix uses
/// `set_hierarchy_order` (force-overwrite). This test would have
/// caught that bug.
#[test]
fn sync_overwrites_widget_store_hierarchy_order() {
    use ph2d_editor_core::screens::hero::ids::HIER_PLAYER;

    let mut hero = setup_hero();
    // After `HeroScreen::new`, the panel's `populate` seeded the
    // order with the placeholder.
    assert_eq!(hero.store.hierarchy_order(), &[HIER_PLAYER]);

    let id_a = NodeId(100_100);
    let id_b = NodeId(100_101);
    let id_c = NodeId(100_102);
    let mut entries = BTreeMap::new();
    entries.insert(id_a, make_entry("A"));
    entries.insert(id_b, make_entry("B"));
    entries.insert(id_c, make_entry("C"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id_a, id_b, id_c], entries);

    assert_eq!(hero.store.hierarchy_order(), &[id_a, id_b, id_c]);
    assert!(
        !hero.store.hierarchy_order().contains(&HIER_PLAYER),
        "fixture placeholder must be evicted by live sync"
    );
}

#[test]
fn sync_overwrites_previous_live_entries() {
    let mut hero = setup_hero();
    let id_v1 = NodeId(100_040);
    let id_v2 = NodeId(100_041);
    let mut entries_v1 = BTreeMap::new();
    entries_v1.insert(id_v1, make_entry("V1"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id_v1], entries_v1);

    let mut entries_v2 = BTreeMap::new();
    entries_v2.insert(id_v2, make_entry("V2"));
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[id_v2], entries_v2);

    let cached = ph2d_panel_hierarchy::current_live_entries().unwrap();
    assert!(!cached.contains_key(&id_v1));
    assert!(cached.contains_key(&id_v2));
    assert_eq!(cached.get(&id_v2).unwrap().name, "V2");
}

#[test]
fn hierarchy_row_click_raises_pending_for_live_entries() {
    // Ported from `screens::hero::tests::hierarchy_row_click_raises_pending_for_live_entries`.
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row_id = NodeId(100_500);
    let mut entries = BTreeMap::new();
    entries.insert(
        row_id,
        HierarchyEntity {
            name: "hero_001".into(),
            icon: IconId::Sprite,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
    );
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &[row_id], entries);
    let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(row_id));
    assert!(consumed, "live-mode row click should consume");
    let drained: Vec<_> = hero.bus.drain().collect();
    // Onda 2: legacy `HierRowClick` was replaced by `HierSelectRow`
    // (modifier-aware multi-select). Bare-click on a live row with no
    // modifier emits Replace — selection swaps to just this row.
    assert_eq!(
        drained,
        vec![EditorAction::HierSelectRow {
            row: row_id,
            modifier: ph2d_editor_core::action_bus::SelectModifier::Replace,
        }]
    );
}
