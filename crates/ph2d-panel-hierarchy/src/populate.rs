//! Hierarchy panel `populate` + `repopulate` — pre-allocates the
//! Hierarchy-owned widget state slots in the `WidgetStore`. Called
//! once at host boot via `Panel::populate`; `repopulate` is called
//! per frame by [`sync_from_hierarchy`](super::sync_from_hierarchy)
//! to merge the host's live entity ids in.
//!
//! Hierarchy drag/resize handles (`HIER_DRAG_HANDLE`,
//! `HIER_RESIZE_HANDLE`) remain in `ph2d_editor_core::screens::hero::
//! pre_populate` because they consume the panel rect (which the
//! editor-core still owns) and the dispatcher's BlenderHit table
//! reaches them at chrome-dispatch time.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

/// Panel boot-time populate — registers the Hierarchy header `+`
/// button + search input + the fixture placeholder row.
pub(crate) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::HIERARCHY_ADD,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.register(
        ids::HIER_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let entities = [ids::HIER_PLAYER];
    for id in entities {
        store.register(id, InteractiveState::Plain);
    }
    store.init_hierarchy_order(entities.to_vec());
}

/// Live-data variant of [`populate`]: register every id in `ids_` as
/// a `Plain` interactive row + seed the hierarchy display order.
/// Called per frame from [`super::sync_from_hierarchy`] with the
/// entity-id list produced by the host's `EntityNodeMap` bridge.
///
/// `register_if_absent` semantics for the `+` button and search field
/// preserve their per-frame interactive state (Pressed/Hovered for
/// the button; typed text/caret for the search input). The row ids
/// use plain `register_if_absent` so stale rows from a previous
/// frame keep their slot in `states` but drop out of
/// `hierarchy_order` (acceptable bounded leak for a session-long
/// editor; M15+ may add a `WidgetStore::retain` if it matters).
pub fn repopulate(store: &mut WidgetStore, ids_: &[ph2d_a11y::NodeId]) {
    store.register_if_absent(
        ids::HIERARCHY_ADD,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.register_if_absent(
        ids::HIER_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for &id in ids_ {
        store.register_if_absent(id, InteractiveState::Plain);
    }
    // `set_hierarchy_order` (force-overwrite) — `init_hierarchy_order`
    // is idempotent and would no-op after `populate()` seeded
    // `[HIER_PLAYER]`, so live mode would paint zero rows.
    store.set_hierarchy_order(ids_.to_vec());
}
