//! Hierarchy panel `populate` + `repopulate` — pre-allocates the
//! Hierarchy-owned widget state slots in the `WidgetStore`. Called
//! once at host boot via `Panel::populate`; `repopulate` is called
//! per frame by [`sync_from_hierarchy`](super::sync_from_hierarchy)
//! to merge the host's live entity ids in.
//!
//! ⚠️ The Hierarchy drag/resize handles are GONE — the side columns are anchored (gate
//! `the_side_columns_are_anchored`), and their two orphan ids (`HIER_DRAG_HANDLE`,
//! `HIER_RESIZE_HANDLE`) left `ph2d-editor-core` on 2026-09-12.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, DropdownState, TextInputState};

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
        crate::ids::HIER_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    register_filter(store);
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
        crate::ids::HIER_SEARCH,
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
    register_filter(store);
    store.set_hierarchy_order(ids_.to_vec());
}

/// O chip ▾ do filtro por componente e as linhas da lista dele (F6).
fn register_filter(store: &mut WidgetStore) {
    store.register_if_absent(
        crate::ids::HIER_FILTER_DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );
    for i in 0..crate::ids::HIER_FILTER_ROWS {
        store.register_if_absent(
            crate::ids::hier_filter_opt(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **Os ícones de componente são WIDGETS** ([`crate::row_components`]): registados aqui, o clique
/// neles chega como `Click(id)`. ⚠️ Os companheiros da linha (olho, galo…) dispensam o registo
/// porque o `pointer_down` os reconhece pelos BITS; estes não têm bits — são um id por família.
pub(crate) fn register_component_icons(
    store: &mut WidgetStore,
    ids_: impl IntoIterator<Item = ph2d_a11y::NodeId>,
) {
    for id in ids_ {
        store.register_if_absent(id, InteractiveState::Plain);
    }
}
