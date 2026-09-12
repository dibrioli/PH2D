//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/menus.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids::{GROUP_TOGGLE_BIT, HIER_PLAYER, ICON_COMPANION_BIT, LOCK_TOGGLE_BIT};
use ph2d_tool_registry::hash_node_id;

/// M14.6 E: search/filter TextInput in the Hierarchy header. Empty
/// query shows every row; non-empty case-insensitively filters by
/// `name.contains(query)` with ancestor-path preservation (a parent
/// stays visible if any descendant matches, so the user sees where
/// the hit lives in the tree).
pub const HIER_SEARCH: NodeId = hash_node_id("hier_search");

/// Map fixture entity name to canonical hierarchy `NodeId`. The
/// placeholder fixture currently exposes only "Scene Root"; the
/// other `HIER_*` ids are kept reserved for the pilot project's
/// real entities.
pub fn hierarchy_id(name: &str) -> Option<NodeId> {
    Some(match name {
        "Scene Root" => HIER_PLAYER,
        _ => return None,
    })
}

/// Map a hierarchy `NodeId` back to its fixture entity name. Inverse
/// of [`hierarchy_id`].
pub fn hierarchy_label_for_id(id: NodeId) -> Option<&'static str> {
    Some(match id {
        x if x == HIER_PLAYER => "Scene Root",
        _ => return None,
    })
}

/// Best-effort 3-letter "kind" badge for the selection tag.
/// Placeholder fixture has a single Scene Root; pilot replaces.
pub fn hierarchy_kind_for_label(_label: &str) -> &'static str {
    "ENT"
}

/// 2026-05-26: lock-toggle companion (per hierarchy row). Mirrors
/// `hier_eye_companion` — XORs `LOCK_TOGGLE_BIT` so dispatch can
/// recognize a click on the row's lock icon.
#[inline]
pub fn hier_lock_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | LOCK_TOGGLE_BIT)
}

/// 2026-05-26: group-lock-toggle companion (per hierarchy row).
#[inline]
pub fn hier_group_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | GROUP_TOGGLE_BIT)
}

/// 2026-05-26: entity-icon companion (left glyph in a hierarchy row).
/// Double-click here triggers focus (View → Selected); double-click
/// on the row's name body triggers rename.
#[inline]
pub fn hier_icon_companion(row_id: NodeId) -> NodeId {
    NodeId(row_id.0 | ICON_COMPANION_BIT)
}
