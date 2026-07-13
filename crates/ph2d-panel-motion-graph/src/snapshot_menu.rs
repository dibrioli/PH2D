//! **What the popup lists** (Motion Nodes doc 54 + 59) — the ONE row source, read by the
//! paint, by the hit-test and by the panel's geometry. Sibling of `snapshot` (panel LOC cap).
//!
//! They used to disagree: the paint drew `current_catalog()` (all 86 types) while the click
//! resolved against the *filtered* smart-connect list, so on a wire-drop the artist read one
//! row and pressed another. Two derivations of the same list is the bug; one is the fix — and
//! the search (doc 59) makes it matter more, because now the list is re-ordered too.

use super::{GraphViewSnapshot, NodeChoice, PortView, current_catalog};
use crate::state::{Menu, MenuBody};
use ph2d_node_registry::NodeUiCategory;

/// **One row of the popup** — a label, a tinted dot, and (for the library) the node type
/// a pick would create.
///
/// The popup has ONE row source, and `paint`, the hit-test and the panel's GEOMETRY all
/// read it. They used to disagree: the paint drew `current_catalog()` (all 86 types) while
/// the click resolved against the *filtered* smart-connect list, so on a wire-drop the
/// artist clicked "Attractor" and got whichever type sat at that index in a list they were
/// never shown. Two derivations of the same list is the bug; one is the fix.
pub(crate) struct MenuRow<'a> {
    pub label: &'a str,
    pub category: NodeUiCategory,
}

/// The rows the popup shows.
///
/// **Library:** plain `A` / R-click → the whole catalog. Opened by a wire dropped in empty
/// space (**smart-connect**) → only the node types with an input that wire can feed (the
/// domain/dim/clock rule the drag's compatibility ghost already uses; the shell still has
/// the last word on cycles and the membrane). Filtering — rather than listing all 86 and
/// refusing 60 of them after the click — is the whole point: the menu answers *"what can I
/// plug in here?"*.
///
/// **CardPorts:** the rows were already chosen and filtered at drop time (they name ports
/// the panel cannot see from here); it just reads them back.
pub(crate) fn menu_rows<'a>(snap: &GraphViewSnapshot, menu: &'a Menu) -> Vec<MenuRow<'a>> {
    match &menu.body {
        MenuBody::CardPorts { rows, .. } => rows
            .iter()
            .map(|p| MenuRow {
                label: &p.label,
                category: p.category,
            })
            .collect(),
        MenuBody::Library { connect_from } => menu_matches(snap, menu, *connect_from)
            .iter()
            .map(|c| MenuRow {
                label: c.display,
                category: c.category,
            })
            .collect(),
    }
}

/// The library entries that survive the SEARCH, best match first (`menu_search`).
///
/// 86 node types in a flat list is a list you have to scan, and Enio knew exactly which node
/// he wanted and still could not find it. The filter runs on both names — the label he reads
/// and the canonical name the docs say out loud — and RANKS, because a search that merely
/// contains the answer somewhere is the list he already had.
pub(crate) fn menu_matches(
    snap: &GraphViewSnapshot,
    menu: &Menu,
    connect_from: Option<(u32, u16)>,
) -> Vec<NodeChoice> {
    let mut scored: Vec<(i32, usize, NodeChoice)> = menu_catalog(snap, connect_from)
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| {
            crate::menu_search::rank(&menu.query, c.display, c.type_name).map(|s| (s, i, c))
        })
        .collect();
    // Best first; ties keep the catalog's own order (stable, and it is the order the artist
    // learned).
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, c)| c).collect()
}

/// The library entries a pick could create — the same filter [`menu_rows`] shows, so row
/// `i` here IS row `i` there.
pub(crate) fn menu_catalog(snap: &GraphViewSnapshot, from: Option<(u32, u16)>) -> Vec<NodeChoice> {
    let all = current_catalog();
    let Some((from_node, from_port)) = from else {
        return all;
    };
    let Some(out) = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize))
    else {
        return all;
    };
    all.into_iter()
        .filter(|c| c.inputs.iter().any(|i| accepts(&i.ty, out)))
        .collect()
}

/// Whether an input port could take the dragged output (the panel-side rule:
/// domain + dim + clock — `connects_directly` minus the membrane).
fn accepts(input: &ph2d_nodegraph::port::PortType, out: &PortView) -> bool {
    input.domain == out.domain && input.dim == out.dim && input.clock == out.clock
}
