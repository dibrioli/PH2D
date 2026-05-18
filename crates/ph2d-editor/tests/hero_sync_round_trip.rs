//! ADR-0025 M14.4a — round-trip test for the host-supplied live
//! hierarchy data path.
//!
//! Covers:
//!
//! 1. `HeroScreen::sync_from_hierarchy` accepts external entries +
//!    overrides the fixture data the next time the hierarchy panel
//!    renders.
//! 2. Selection echo: a click on a live row (via `apply_event`)
//!    updates `HeroScreen::selection.label` to that entry's name,
//!    not the fixture default.
//! 3. `clear_live_hierarchy` reverts to fixture behavior.

use ph2d_editor::HeroScreen;
use ph2d_editor::NodeId;
use ph2d_editor::icons::IconId;
use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::screens::hero::fixture::HierarchyEntity;
use std::collections::BTreeMap;

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
    ph2d_editor::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    assert!(hero.hierarchy.live_entries.is_none());

    let id_a = NodeId(100_000);
    let id_b = NodeId(100_001);
    let mut entries = BTreeMap::new();
    entries.insert(id_a, make_entry("Alpha"));
    entries.insert(id_b, make_entry("Beta"));

    hero.sync_from_hierarchy(&[id_a, id_b], entries.clone());
    assert!(hero.hierarchy.live_entries.is_some());
    let cached = hero.hierarchy.live_entries.as_ref().unwrap();
    assert_eq!(cached.len(), 2);
    assert_eq!(cached.get(&id_a).unwrap().name, "Alpha");
    assert_eq!(cached.get(&id_b).unwrap().name, "Beta");
}

#[test]
fn click_on_live_row_updates_selection_label() {
    ph2d_editor::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let id_player = NodeId(100_005);
    let mut entries = BTreeMap::new();
    entries.insert(id_player, make_entry("Player"));
    hero.sync_from_hierarchy(&[id_player], entries);

    // Fire a click on the live id via apply_event. The hierarchy
    // region's apply_event consumes it and writes selection.label.
    let consumed = hero.apply_event(WidgetEvent::Click(id_player));
    assert!(consumed, "live click should be consumed");
    let sel = hero.selection.as_ref().expect("selection set by click");
    assert_eq!(sel.label, "Player");
    // Default kind for live entries without an explicit badge is "ENT".
    assert_eq!(sel.kind, "ENT");
}

#[test]
fn click_uses_badge_as_kind_when_present() {
    ph2d_editor::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let id = NodeId(100_010);
    let mut entries = BTreeMap::new();
    entries.insert(
        id,
        HierarchyEntity {
            badge: Some("SPR".into()),
            ..make_entry("WithBadge")
        },
    );
    hero.sync_from_hierarchy(&[id], entries);

    hero.apply_event(WidgetEvent::Click(id));
    let sel = hero.selection.as_ref().unwrap();
    assert_eq!(sel.kind, "SPR");
}

#[test]
fn clear_live_reverts_to_fixture_behavior() {
    ph2d_editor::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let id = NodeId(100_020);
    let mut entries = BTreeMap::new();
    entries.insert(id, make_entry("Temp"));
    hero.sync_from_hierarchy(&[id], entries);
    assert!(hero.hierarchy.live_entries.is_some());

    hero.clear_live_hierarchy();
    assert!(hero.hierarchy.live_entries.is_none());

    // After clear, a click on the previously-live id is NOT consumed
    // because it's neither in fixture's hierarchy_label_for_id nor
    // in live entries.
    let consumed = hero.apply_event(WidgetEvent::Click(id));
    assert!(!consumed, "stale live id click ignored after clear");
}

#[test]
fn fixture_click_still_works_in_live_mode_for_unknown_id() {
    ph2d_editor::test_support::ensure_panel_registry();
    // Live mode only overrides ids the host supplied. Clicks on
    // ids NOT in live_entries fall through to the fixture
    // resolver — this lets a partially-wired editor still surface
    // selections for static toolbars, etc. The hierarchy panel
    // specifically: a click on HIER_PLAYER (fixture id) hits the
    // fallback when live entries don't contain that id.
    use ph2d_editor::screens::hero::ids::HIER_PLAYER;

    let mut hero = HeroScreen::new(NodeId(1));
    let live_id = NodeId(100_030);
    let mut entries = BTreeMap::new();
    entries.insert(live_id, make_entry("Live"));
    hero.sync_from_hierarchy(&[live_id], entries);

    // Click HIER_PLAYER (NOT in live entries) → fixture fallback wins.
    let consumed = hero.apply_event(WidgetEvent::Click(HIER_PLAYER));
    assert!(consumed);
    let sel = hero.selection.as_ref().unwrap();
    assert_eq!(sel.label, "Scene Root");
}

/// Regression: ensure `sync_from_hierarchy` ACTUALLY publishes the
/// live ids into `WidgetStore::hierarchy_order` (not just into the
/// cached `live_hierarchy_entries`). The original M14.4a wiring
/// used `init_hierarchy_order` which is idempotent — after the
/// fixture `populate` seeded `[HIER_PLAYER]`, the live-mode call
/// was a no-op and the hierarchy panel painted zero rows. The fix
/// uses `set_hierarchy_order` (force-overwrite). This test would
/// have caught that bug.
#[test]
fn sync_overwrites_widget_store_hierarchy_order() {
    ph2d_editor::test_support::ensure_panel_registry();
    use ph2d_editor::screens::hero::ids::HIER_PLAYER;

    let mut hero = HeroScreen::new(NodeId(1));
    // After `HeroScreen::new`, the fixture populate has seeded the
    // order with the placeholder.
    assert_eq!(hero.store.hierarchy_order(), &[HIER_PLAYER]);

    let id_a = NodeId(100_100);
    let id_b = NodeId(100_101);
    let id_c = NodeId(100_102);
    let mut entries = BTreeMap::new();
    entries.insert(id_a, make_entry("A"));
    entries.insert(id_b, make_entry("B"));
    entries.insert(id_c, make_entry("C"));
    hero.sync_from_hierarchy(&[id_a, id_b, id_c], entries);

    // The hierarchy order is now exactly the live ids, preserving
    // input order — NOT the fixture default.
    assert_eq!(hero.store.hierarchy_order(), &[id_a, id_b, id_c]);
    assert!(
        !hero.store.hierarchy_order().contains(&HIER_PLAYER),
        "fixture placeholder must be evicted by live sync"
    );
}

#[test]
fn sync_overwrites_previous_live_entries() {
    ph2d_editor::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let id_v1 = NodeId(100_040);
    let id_v2 = NodeId(100_041);
    let mut entries_v1 = BTreeMap::new();
    entries_v1.insert(id_v1, make_entry("V1"));
    hero.sync_from_hierarchy(&[id_v1], entries_v1);

    let mut entries_v2 = BTreeMap::new();
    entries_v2.insert(id_v2, make_entry("V2"));
    hero.sync_from_hierarchy(&[id_v2], entries_v2);

    let cached = hero.hierarchy.live_entries.as_ref().unwrap();
    assert!(!cached.contains_key(&id_v1));
    assert!(cached.contains_key(&id_v2));
    assert_eq!(cached.get(&id_v2).unwrap().name, "V2");
}
