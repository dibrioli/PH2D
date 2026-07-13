//! Subgraph gestures (Motion Nodes doc 57) — group, ungroup, enter, walk out.
//! Sibling of `interact` (panel LOC cap), same shape as `interact_backdrop`.
//!
//! The panel does not own the nesting and does not own the level: it says what the
//! artist DID, and the shell decides what that means to the document. So everything
//! here is one line of intent — which is also why a card and a node can share the
//! same gestures (drag, select, delete): they are both cards on a canvas, and the
//! shell decodes the id.

use crate::geom::{self, View};
use crate::snapshot::{
    GraphIntent, GraphViewSnapshot, NodeViewKind, PortView, card_hidden_ports, is_subgraph_view,
    push_intent, same_type,
};
use crate::state::{Interaction, Menu, MenuBody, MotionGraphPanelState};

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

/// **A wire dropped on a collapsed card's BODY** (doc 57 §5) — the popup that asks which
/// port inside the group it lands on. `None` when the drop was not on a card, or the group
/// holds nothing this wire could legally reach (the caller then falls back to whatever a
/// drop on empty canvas does — smart-connect, or nothing).
///
/// **Why a menu, and not a socket.** The card's sockets ARE the wires that already cross
/// it. A *new* wire therefore has nowhere to land: the socket it wants cannot exist until
/// the wire does. Blender and Nuke dodge this by making the group's interface DECLARED (a
/// Group Input node, an Input node) — a second thing to build and keep in sync. We derive
/// it instead, and pay for that here, once: the drop asks the one question the editor
/// cannot answer for itself, and the socket appears on the pick, derived from the edge.
///
/// The list is filtered to what the wire can actually feed (domain + dim + clock — the same
/// local rule the drag's compatibility ghost uses). Offering ports it would only be refused
/// on would make the menu a list of disappointments.
pub(super) fn card_port_menu(
    snap: &GraphViewSnapshot,
    view: &View,
    other: (u32, u16),
    forward: bool,
    detach: Option<(u32, u16)>,
    x: f32,
    y: f32,
) -> Option<Menu> {
    // A CARD hides ports (doc 57 §5); an ordinary NODE hides its parameters (doc 58) — a
    // param has no port and never will, so the only way a wire reaches one is to ask. Same
    // gesture, same menu, same question: *where does this wire go?* A GHOST hides nothing —
    // it lives on another level, and its body is not a door.
    let card = snap
        .nodes
        .iter()
        .find(|n| n.kind != NodeViewKind::Ghost && geom::card_rect(n, view).contains(x, y))?;
    let held = port_view(snap, other, !forward)?;
    let hidden = card_hidden_ports(card.id);
    // Forward: the wire comes OUT of `other` and needs an input inside. Backwards: it needs
    // an output inside to feed `other`.
    let side = if forward {
        hidden.inputs
    } else {
        hidden.outputs
    };
    let rows: Vec<_> = side
        .into_iter()
        .filter(|p| same_type(&p.port_type, &held))
        .collect();
    if rows.is_empty() {
        return None;
    }
    Some(Menu {
        scroll: 0.0,
        screen: (x, y),
        spawn: view.graph(x, y),
        query: String::new(),
        opened: false,
        body: MenuBody::CardPorts {
            rows,
            other,
            forward,
            detach,
        },
    })
}

/// The type of the port already in hand — an output when the wire was drawn forwards, an
/// input when it was drawn backwards.
fn port_view(snap: &GraphViewSnapshot, (node, port): (u32, u16), input: bool) -> Option<PortView> {
    let n = snap.nodes.iter().find(|n| n.id == node)?;
    let ports = if input { &n.inputs } else { &n.outputs };
    ports.get(port as usize).cloned()
}

/// **A breadcrumb crumb was clicked** — walk to that level (`None` = the root).
pub(super) fn go_to_crumb(snap: &GraphViewSnapshot, i: usize) {
    if let Some(c) = snap.breadcrumb.get(i) {
        push_intent(GraphIntent::GoToLevel { level: c.level });
    }
}
