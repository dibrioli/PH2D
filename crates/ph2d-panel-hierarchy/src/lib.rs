//! `ph2d-panel-hierarchy` — ADR-0029 Phase C.2 typed panel crate.
//!
//! Owns the Hierarchy panel's:
//! - retained state ([`HierarchyState`]) — held by `ErasedPanel` in
//!   the runtime `panel::PANEL_REGISTRY`.
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   widget registration ([`populate`]).
//! - typed contract via [`HierarchyPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//!
//! Per-frame snapshots the host publishes to drive the live entity
//! list flow through thread-local setters re-exported below
//! ([`sync_from_hierarchy`], [`clear_live_hierarchy`],
//! [`set_live_component_count`]); these supersede the pre-C.2
//! `hero.hierarchy.live_entries` field writes and the
//! `HeroScreen::sync_from_hierarchy` method.

#![forbid(unsafe_code)]

mod event;
mod paint;
mod populate;
mod row;
mod search;
pub mod state;

pub use state::{
    HierarchyState, current_live_entries, last_hierarchy_content_h, set_live_component_count,
    set_live_entries,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::screens::hero::fixture;

/// Zero-size marker implementing the typed Hierarchy panel contract.
pub struct HierarchyPanel;

impl Panel for HierarchyPanel {
    type State = HierarchyState;

    const ID: &'static str = "hierarchy";
    const NODE_ID: NodeId = ids::HIER_PANEL;
    const DEFAULT_VISIBLE: bool = true;

    fn paint(state: &mut HierarchyState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut HierarchyState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}

/// Inject host-supplied live entity rows into the hierarchy panel
/// (ADR-0025 M14.4a). Each call:
///
/// 1. Re-registers the `ordered` `NodeId`s on the [`WidgetStore`] as
///    plain interactive rows (idempotent — repeat calls cost nothing
///    for ids already seen this session).
/// 2. Replaces the `WidgetStore::hierarchy_order` list so the painter
///    iterates in the order the host supplies (DFS root-first).
/// 3. Stores `entries` in the panel-owned thread-local so
///    `paint_hierarchy` can override `fixture::hierarchy()` and so
///    `apply_event` can resolve click ids back to entity names
///    without crossing the `bevy_ecs::World` boundary (HR-8).
///
/// Call once per frame from the host's `render_frame` loop before
/// `paint_hero_screen`. Passing an empty `ordered` slice is valid
/// (renders an empty hierarchy).
pub fn sync_from_hierarchy(
    store: &mut WidgetStore,
    ordered: &[NodeId],
    entries: std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>,
) {
    populate::repopulate(store, ordered);
    state::set_live_entries(Some(entries));
}

/// Drop any host-supplied hierarchy state, reverting to the fixture
/// data set in [`populate`]. The host calls this when leaving live-
/// edit mode (e.g. user pressed `PH2D_HERO_LIVE` toggle off).
pub fn clear_live_hierarchy() {
    state::set_live_entries(None);
}
