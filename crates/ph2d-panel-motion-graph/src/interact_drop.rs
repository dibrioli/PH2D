//! **Resolving a loose wire-END dropped away from a socket** (Motion Nodes F2) — the tail of
//! `interact::apply_socket_out`, split out for the panel LOC cap; `super` is `interact`.
//!
//! When a forward wire is let go and the magnet found no socket, three things can still be meant,
//! in order of specificity:
//!
//! - **A collapsed CARD** — its hidden ports have no socket to land on yet, so ask which one
//!   (doc 57 §5).
//! - **A regular node's BODY** — connect to its first FREE compatible input. Blender and Nuke let
//!   a wire dropped anywhere on a node land on its first open input, so the artist need not hit
//!   the exact socket; the magnet only covers a halo around each one.
//! - **Empty canvas** — smart-connect: open the add-menu filtered to what THIS wire can feed, and
//!   wire it on the pick (dropping in space used to be a silent no-op).

use super::{View, geom, push_intent};
use crate::snapshot::{GraphIntent, GraphViewSnapshot};
use crate::state::{Menu, MenuBody, MotionGraphPanelState};

/// Resolve a forward wire dropped at `(x, y)` that landed on no input socket. Always terminal — it
/// sets `state.menu` or pushes an intent, and the caller returns.
pub(super) fn resolve_loose_output_drop(
    state: &mut MotionGraphPanelState,
    snap: &GraphViewSnapshot,
    view: &View,
    from_node: u32,
    from_port: u16,
    x: f32,
    y: f32,
) {
    // A collapsed CARD's body: the group has ports this wire could feed, but no socket for them
    // yet (doc 57 §5). Ask which one.
    if let Some(menu) =
        super::subgraph_gesture::card_port_menu(snap, view, (from_node, from_port), true, None, x, y)
    {
        state.menu = Some(menu);
        return;
    }
    // A regular node's BODY: connect to its first FREE compatible input — the forgiving "drop on
    // the node" of Blender / Nuke. No free compatible input (or it is the source) falls through.
    if let Some((to_node, to_port)) = node_body_target(snap, view, from_node, from_port, x, y) {
        push_intent(GraphIntent::Connect {
            from_node,
            from_port,
            to_node,
            to_port,
        });
        return;
    }
    // Empty canvas — smart-connect: the gesture already said what it wanted (a consumer for THIS
    // wire), so open the add-menu filtered to the node types that can take it, and wire it on the
    // pick.
    state.menu = Some(Menu {
        scroll: 0.0,
        screen: (x, y),
        spawn: view.graph(x, y),
        query: String::new(),
        opened: false,
        body: MenuBody::Library {
            connect_from: Some((from_node, from_port)),
            splice: None,
        },
    });
}

/// The first FREE input of the regular node whose card body contains `(x, y)`, type-compatible
/// with the source output — the drop target of a wire let go over a node's BODY. `None` when the
/// point is over no regular node, the node is the SOURCE (dropping a wire on its own body would
/// self-connect), or the node has no free compatible input. A COLLAPSED card is skipped: its
/// hidden ports go through the card menu above, not this.
pub(super) fn node_body_target(
    snap: &GraphViewSnapshot,
    view: &View,
    from_node: u32,
    from_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16)> {
    let out = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize))?;
    let node = snap.nodes.iter().find(|n| {
        n.id != from_node
            && n.kind != crate::snapshot::NodeViewKind::Subgraph
            && geom::card_rect(n, view).contains(x, y)
    })?;
    node.inputs.iter().enumerate().find_map(|(i, inp)| {
        let occupied = snap
            .edges
            .iter()
            .any(|e| e.to_node == node.id && e.to_port == i as u16 && !e.delayed);
        let compat = inp.domain == out.domain && inp.dim == out.dim && inp.clock == out.clock;
        (!occupied && compat).then_some((node.id, i as u16))
    })
}

/// The first OUTPUT of the regular node whose card body contains `(x, y)`, type-compatible with the
/// target INPUT — the mirror of [`node_body_target`], for a wire dragged BACKWARDS out of an empty
/// input and let go over a node's body (so a backward drag is as forgiving as a forward one).
/// Outputs fan out, never "occupied", so it is simply the first compatible one. `None` over no
/// regular node, the TARGET node itself (that would self-loop), or no compatible output. A
/// collapsed card is skipped: its hidden ports go through the card menu.
pub(super) fn node_body_source(
    snap: &GraphViewSnapshot,
    view: &View,
    to_node: u32,
    to_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16)> {
    let inp = snap
        .nodes
        .iter()
        .find(|n| n.id == to_node)
        .and_then(|n| n.inputs.get(to_port as usize))?;
    let node = snap.nodes.iter().find(|n| {
        n.id != to_node
            && n.kind != crate::snapshot::NodeViewKind::Subgraph
            && geom::card_rect(n, view).contains(x, y)
    })?;
    node.outputs.iter().enumerate().find_map(|(i, out)| {
        let compat = out.domain == inp.domain && out.dim == inp.dim && out.clock == inp.clock;
        compat.then_some((node.id, i as u16))
    })
}
