//! Subgraph gestures (Motion Nodes doc 57) — group, ungroup, enter, walk out.
//! Sibling of `interact` (panel LOC cap), same shape as `interact_backdrop`.
//!
//! The panel does not own the nesting and does not own the level: it says what the
//! artist DID, and the shell decides what that means to the document. So everything
//! here is one line of intent — which is also why a card and a node can share the
//! same gestures (drag, select, delete): they are both cards on a canvas, and the
//! shell decodes the id.

use crate::snapshot::{
    GraphIntent, GraphViewSnapshot, NodeViewKind, is_subgraph_view, push_intent,
};
use crate::state::{Interaction, MotionGraphPanelState};

/// **Ctrl+G** — collapse the selection into a subgraph. Nothing selected, nothing to
/// collapse: the key is inert rather than creating an empty group nobody asked for.
pub(super) fn group(state: &mut MotionGraphPanelState) {
    if state.selected.is_empty() {
        return;
    }
    push_intent(GraphIntent::GroupSelection {
        nodes: state.selected.iter().copied().collect(),
    });
    // The shell mints the card's id and hands it back as the selection
    // (`request_graph_selection`), the way Ctrl+D hands back its copies.
    state.selected.clear();
}

/// **What the toolbar's Group chip will do if you press it right now** — because a
/// button that always says the same thing while doing nothing half the time is a
/// button the artist stops pressing (Enio, smoke 2026-07-13: *"o botão de agrupar
/// deveria ser usado para desagrupar tb"*).
///
/// A card selected → it will DISSOLVE it. Anything else selected → it will collapse
/// it. Nothing selected → it is inert, and reads inert.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum GroupVerb {
    Group,
    Ungroup,
    Inert,
}

pub(crate) fn verb(state: &MotionGraphPanelState) -> GroupVerb {
    if state.selected.iter().any(|id| is_subgraph_view(*id)) {
        GroupVerb::Ungroup
    } else if state.selected.is_empty() {
        GroupVerb::Inert
    } else {
        GroupVerb::Group
    }
}

/// The chip. One button, both verbs — it does whatever [`verb`] said it would.
pub(super) fn chip(state: &mut MotionGraphPanelState, snap: &GraphViewSnapshot) {
    match verb(state) {
        GroupVerb::Group => group(state),
        GroupVerb::Ungroup => ungroup(state, snap),
        GroupVerb::Inert => {}
    }
}

/// **Ctrl+Alt+G** — dissolve. With cards selected, dissolve those. With nothing
/// selected while standing INSIDE a group, dissolve the group you are in and step
/// out to where it was (Nuke's *Expand Group* from inside).
pub(super) fn ungroup(state: &mut MotionGraphPanelState, snap: &GraphViewSnapshot) {
    let cards: Vec<u32> = state
        .selected
        .iter()
        .copied()
        .filter(|id| is_subgraph_view(*id))
        .collect();
    if cards.is_empty() {
        if let Some(level) = snap.level {
            push_intent(GraphIntent::Ungroup { id: level });
        }
        return;
    }
    for id in cards {
        push_intent(GraphIntent::Ungroup {
            id: id & !crate::snapshot::SUBGRAPH_VIEW_TAG,
        });
        state.selected.remove(&id);
    }
}

/// **Double-click a collapsed card → go inside it** (Houdini's gesture: *"Go inside a
/// network node: double-click the node"*). On any other card it does nothing — the
/// double-click that splices a reroute belongs to WIRES, and a node has no inside.
///
/// The gesture arrives after a `Begin` (which already selected the card and armed a
/// drag of zero delta, harmlessly), so it also has to put the interaction back to
/// idle — a drag left armed across a level change would move a card that is no
/// longer on screen.
pub(super) fn enter(
    state: &mut MotionGraphPanelState,
    snap: &GraphViewSnapshot,
    node: u32,
) -> bool {
    state.interaction = Interaction::Idle;
    let is_card = snap
        .nodes
        .iter()
        .any(|n| n.id == node && n.kind == NodeViewKind::Subgraph);
    if !is_card {
        return false;
    }
    push_intent(GraphIntent::EnterSubgraph {
        id: node & !crate::snapshot::SUBGRAPH_VIEW_TAG,
    });
    // The selection is dropped by the SHELL on the level change (it is the side that
    // knows whether the level actually changed), so nothing is cleared here.
    true
}

/// **A breadcrumb crumb was clicked** — walk to that level (`None` = the root).
pub(super) fn go_to_crumb(snap: &GraphViewSnapshot, i: usize) {
    if let Some(c) = snap.breadcrumb.get(i) {
        push_intent(GraphIntent::GoToLevel { level: c.level });
    }
}
