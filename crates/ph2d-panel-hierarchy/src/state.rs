//! Hierarchy panel state — owned by `ErasedPanel<HierarchyPanel>` after
//! ADR-0029 Phase C.2. Pre-Phase-C this struct lived on `HeroScreen`
//! as `pub hierarchy: HierarchyState`; now it's typed-down behind the
//! trait registry and reached via the typed `state: &mut HierarchyState`
//! parameter each `Panel` fn receives.
//!
//! Thread-locals here hold per-frame snapshots the host publishes
//! BEFORE the panel paints: the live entity map (set via
//! [`set_live_entries`] from the shell's `sync_from_hierarchy` flow)
//! plus the running component count. They live as thread-locals (not
//! on `HierarchyState`) because the publish/clear cadence matches the
//! paint loop, mirroring the Inspector pattern.

use ph2d_a11y::NodeId;
use ph2d_editor_core::screens::hero::fixture;

/// Hierarchy panel retained state. Held inside
/// `ErasedPanel<HierarchyPanel>` after Phase C.2; mutated by the
/// panel's `apply_event` (inline-rename mode transitions) and read by
/// `paint` (so the painter swaps the row's name label for a TextInput
/// overlay). `visible` moved to `HeroScreen::panel_visibility` per the
/// canonical Phase C visibility model; `live_entries` moved to the
/// [`CURRENT_LIVE_ENTRIES`] thread-local below.
#[derive(Clone, Debug, Default)]
pub struct HierarchyState {
    /// Row currently in inline-rename mode (M14.7 polish). The
    /// painter replaces that row's name label with a TextInput
    /// overlay; `apply_event` writes this on long-press / right-click
    /// "Rename" and clears it on Submit / Cancel / Blur.
    pub rename_target_row: Option<NodeId>,
}

thread_local! {
    /// Total height of the hierarchy entity list painted last frame.
    /// `paint::paint_thunk` clamps `panel_scroll(HIER_PANEL)` against
    /// this each frame so wheeling at the bottom doesn't overshoot.
    pub(crate) static LAST_HIER_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// Live-mode entity rows published by the shell via
    /// [`sync_from_hierarchy`] (ADR-0025 M14.4a). When `Some`,
    /// `paint_hierarchy` uses these instead of `fixture::hierarchy()`.
    pub(crate) static CURRENT_LIVE_ENTRIES: std::cell::RefCell<
        Option<std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>>,
    > = const { std::cell::RefCell::new(None) };

    /// Total component count across the live SimWorld, surfaced into
    /// the hierarchy header next to the entity count. Host writes this
    /// once per frame before `paint_hero_screen`; defaults to 0 when
    /// no host is publishing (e.g. fixture-only smoke tests).
    pub(crate) static CURRENT_COMPONENT_COUNT: std::cell::Cell<u32> =
        const { std::cell::Cell::new(0) };
}

/// Set the live entity map. Pass `None` to revert to fixture mode.
pub fn set_live_entries(
    entries: Option<std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>>,
) {
    CURRENT_LIVE_ENTRIES.with(|c| *c.borrow_mut() = entries);
}

/// Read a snapshot of the current live entity map (cloned). Returns
/// `None` in fixture mode.
pub fn current_live_entries() -> Option<std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>>
{
    CURRENT_LIVE_ENTRIES.with(|c| c.borrow().clone())
}

pub(crate) fn live_entries_contains(id: NodeId) -> bool {
    CURRENT_LIVE_ENTRIES.with(|c| c.borrow().as_ref().is_some_and(|m| m.contains_key(&id)))
}

pub(crate) fn live_entry_for(id: NodeId) -> Option<fixture::HierarchyEntity> {
    CURRENT_LIVE_ENTRIES.with(|c| c.borrow().as_ref().and_then(|m| m.get(&id).cloned()))
}

/// Publish the live component count for the Hierarchy header. Hosts
/// call this once per frame before `paint_hero_screen`.
pub fn set_live_component_count(count: u32) {
    CURRENT_COMPONENT_COUNT.with(|c| c.set(count));
}

pub(crate) fn current_component_count() -> u32 {
    CURRENT_COMPONENT_COUNT.with(|c| c.get())
}

pub fn last_hierarchy_content_h() -> f32 {
    LAST_HIER_CONTENT_H.with(|c| c.get())
}

pub(crate) fn set_last_hierarchy_content_h(h: f32) {
    LAST_HIER_CONTENT_H.with(|c| c.set(h));
}
