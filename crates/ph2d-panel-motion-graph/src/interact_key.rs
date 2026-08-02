//! Graph keyboard verbs (Motion Nodes M1.E4) — sibling of `interact` (panel LOC cap).
//!
//! Editor-core maps the keycode to a [`GraphKey`]; this decides what the verb DOES.
//! Every one of them is either ephemeral view state (fit / arm the knife / arm the
//! probe) or a single [`GraphIntent`] — the panel never edits the document itself.

use super::{
    GraphIntent, GraphKey, GraphViewSnapshot, Interaction, MotionGraphPanelState, Rect, View,
    push_intent, subgraph_gesture,
};

pub(super) fn apply_key(
    state: &mut MotionGraphPanelState,
    k: GraphKey,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    match k {
        // Ctrl+G / Ctrl+Alt+G — collapse the selection into a subgraph, and dissolve
        // one (doc 57). Blender's and Nuke's chords, in both cases.
        GraphKey::Group => subgraph_gesture::group(state),
        GraphKey::Ungroup => subgraph_gesture::ungroup(state, snap),
        // Re-fit on the next paint (the draw pass owns the fit math): the SELECTION
        // when there is one, the whole graph otherwise — the universal `F`.
        GraphKey::Fit => state.request_fit(),
        GraphKey::Escape => {
            state.selected.clear();
            state.selected_backdrop = None;
            state.selected_wires.clear();
            state.interaction = Interaction::Idle;
            state.menu = None;
            state.knife_armed = false;
            state.probe_armed = false;
            // Esc also puts the probe away — it is a readout, not a commitment.
            if state.probe.take().is_some() {
                push_intent(GraphIntent::SetProbe { node: None });
            }
        }
        // Ctrl+D — duplicate the selection (the shell mints the copies, wires the
        // links INTERNAL to the selection, and hands the copies back as the new
        // selection so the drag that follows moves them, not the originals).
        GraphKey::Duplicate if !state.selected.is_empty() => {
            push_intent(GraphIntent::DuplicateSelection {
                nodes: state.selected.iter().copied().collect(),
            });
        }
        // Ctrl+C — copy the selection into the graph clipboard (the shell owns it).
        // Nothing selected → inert (a copy of nothing is not an operation), so it
        // falls through to the no-op arm and leaves the clipboard as it was.
        GraphKey::Copy if !state.selected.is_empty() => {
            push_intent(GraphIntent::CopySelection {
                nodes: state.selected.iter().copied().collect(),
            });
        }
        // Ctrl+V — paste the clipboard. No selection guard: paste depends on what was
        // COPIED, not on what is selected now (the shell drops it when the clipboard
        // is empty), and the pastes become the new selection when it lands.
        GraphKey::Paste => push_intent(GraphIntent::Paste),
        // Ctrl+X — CUT: copy the selection into the clipboard, THEN delete it. It is
        // exactly copy-then-delete and says so by emitting both intents in that order:
        // the copy is a READ (no undo step) and the delete is the one undo step, so a
        // cut is a single Ctrl+Z — no third code path, no dedicated intent to keep in
        // sync with the two it composes. The copy runs first (intents drain FIFO), so it
        // captures the nodes while they still exist; then the delete removes them, healing
        // the chain and expanding group cards exactly as the Delete key does. Nothing
        // selected → inert (falls through to the no-op arm), and — unlike Delete — a lone
        // selected backdrop is NOT cut: a backdrop is decoration, not graph content the
        // clipboard carries, so cutting it would delete-without-copy, a lie about the verb.
        GraphKey::Cut if !state.selected.is_empty() => {
            let nodes: Vec<u32> = state.selected.iter().copied().collect();
            push_intent(GraphIntent::CopySelection {
                nodes: nodes.clone(),
            });
            push_intent(GraphIntent::DeleteSelection { nodes });
            state.selected.clear();
            state.interaction = Interaction::Idle;
        }
        // `K` arms the knife: the next left-drag slices wires instead of selecting.
        // A second K disarms it (so does Esc, and so does the stroke itself). The
        // toolbar chip mirrors the state (Accent ring) — a mode with no visible
        // sign is a mystery (Enio, smoke: "não entendi K o que faz").
        // **F2 names the thing** (doc 61). Nothing selected, or many things selected, and there
        // is no single name to ask about — so the key is inert rather than guessing.
        GraphKey::Rename => crate::rename::arm(state, snap),
        GraphKey::Knife => state.knife_armed = !state.knife_armed,
        // `P` arms the probe: the next click on a node points the readout at it.
        GraphKey::Probe => state.probe_armed = !state.probe_armed,
        // Delete the selection (orphan edges go with the nodes, shell-side).
        // Empty selection → no intent (idempotent against the double key
        // dispatch: M0 focus gate + the shell's cursor push). Node, backdrop and
        // wire selection are mutually exclusive, so Delete is never ambiguous — a
        // deleted backdrop takes nothing with it (it owns no nodes; it draws around
        // them), and each deleted wire is exactly the alt-click `Disconnect` (its
        // unique target input). The set drains in one gesture — a bundle of edges
        // dropped together, one Disconnect apiece.
        GraphKey::Delete => {
            if !state.selected.is_empty() {
                push_intent(GraphIntent::DeleteSelection {
                    nodes: state.selected.iter().copied().collect(),
                });
                state.selected.clear();
            } else if let Some(id) = state.selected_backdrop.take() {
                push_intent(GraphIntent::DeleteBackdrop { id });
            } else if !state.selected_wires.is_empty() {
                for (to_node, to_port) in std::mem::take(&mut state.selected_wires) {
                    push_intent(GraphIntent::Disconnect { to_node, to_port });
                }
            }
            state.interaction = Interaction::Idle;
        }
        // `A` opens the full-screen "Add Node" palette at the canvas center (the keyboard verb carries no
        // cursor position). The shell owns the palette (it lives over the whole app, not in the graph
        // panel), so the panel only ASKS via `OpenLibrary`; the picked node lands back as an `AddNode`
        // at this graph-space spawn. Idempotent against the double key dispatch (the shell no-ops a
        // second open while the palette is already up).
        GraphKey::Add if state.menu.is_none() => {
            let center = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
            let spawn = View::new(rect, state.view).graph(center.0, center.1);
            push_intent(GraphIntent::OpenLibrary {
                x: spawn.0,
                y: spawn.1,
                connect_from: None,
                splice: None,
                compatible: Vec::new(),
            });
        }
        // Ctrl+A — select every node at THIS level (the snapshot is level-scoped). A backdrop is
        // a separate subject (see `select_on_press`), so it clears; an empty graph selects
        // nothing, idempotent against the double key dispatch.
        GraphKey::SelectAll => {
            state.selected = snap.nodes.iter().map(|n| n.id).collect();
            state.selected_backdrop = None;
            state.selected_wires.clear();
        }
        // Ctrl+L — grow the selection to the connected island(s) it touches (Select Linked).
        GraphKey::SelectLinked => grow_selection_to_linked(state, snap),
        // Ctrl+I — invert the selection at THIS level (the snapshot is level-scoped), the
        // sibling of Ctrl+A. Every node the level has that is NOT currently selected becomes
        // selected, and the currently-selected ones drop out. `take` reads the old set and
        // leaves the field empty, so the rebuild has no borrow to fight; a backdrop is a
        // separate subject (like SelectAll), so it clears. Empty graph → empty selection.
        GraphKey::SelectInvert => {
            let was = std::mem::take(&mut state.selected);
            state.selected = snap
                .nodes
                .iter()
                .map(|n| n.id)
                .filter(|id| !was.contains(id))
                .collect();
            state.selected_backdrop = None;
            state.selected_wires.clear();
        }
        // H — switch the selected node(s) off/on (bypass/mute). The scope rule is the rove
        // idiom: press once and everything selected goes OFF; if it is ALL already off, press
        // again and it comes back on. `bypassed` is read off the view (which mirrors the graph),
        // so a mixed selection resolves to "off" and one more press to "on". Nothing selected →
        // inert (falls through to the no-op arm).
        GraphKey::Bypass if !state.selected.is_empty() => {
            let nodes: Vec<u32> = state.selected.iter().copied().collect();
            let all_muted = nodes.iter().all(|id| {
                snap.nodes
                    .iter()
                    .find(|n| n.id == *id)
                    .is_some_and(|n| n.bypassed)
            });
            push_intent(GraphIntent::SetBypass {
                nodes,
                on: !all_muted,
            });
        }
        // Space — toggle transport play/pause (the shell owns the transport).
        GraphKey::TogglePlay => push_intent(GraphIntent::TogglePlay),
        _ => {}
    }
}

/// Flood-fill the selection out along edges (UNDIRECTED): every node reachable from a currently
/// selected one joins it. The graph's islands, grabbed whole — to move or delete a whole subtree
/// from any one of its nodes. Edge direction is irrelevant here (the point is topological
/// reachability, not evaluation order), so a `pre` (feedback) edge links its endpoints like any
/// other. `state.selected` doubles as the visited set: a node re-explores only when it was newly
/// inserted, so the walk terminates. Nothing selected → nothing to grow from.
fn grow_selection_to_linked(state: &mut MotionGraphPanelState, snap: &GraphViewSnapshot) {
    let mut stack: Vec<u32> = state.selected.iter().copied().collect();
    while let Some(n) = stack.pop() {
        for e in &snap.edges {
            let neighbour = if e.from_node == n {
                Some(e.to_node)
            } else if e.to_node == n {
                Some(e.from_node)
            } else {
                None
            };
            if let Some(m) = neighbour
                && state.selected.insert(m)
            {
                stack.push(m);
            }
        }
    }
}

/// Selection on node press: plain click selects only this node (unless it is
/// already part of the selection — then keep it, so a multi-drag works); Shift
/// toggles it into/out of the selection. (The caller — `apply_node` — has already
/// cleared the backdrop and wire selections: one subject at a time.)
pub(super) fn select_on_press(state: &mut MotionGraphPanelState, node: u32, shift: bool) {
    if shift {
        if !state.selected.insert(node) {
            state.selected.remove(&node);
        }
    } else if !state.selected.contains(&node) {
        state.selected.clear();
        state.selected.insert(node);
    }
}

/// Selection on wire press — the mirror of [`select_on_press`], on the wire's target `(to_node,
/// to_port)`. Shift toggles it into/out of the set; a plain press makes it the sole wire. Unlike a
/// node there is no keep-for-multi-drag branch: a wire is not draggable, so a plain press always
/// replaces. (The caller — the `Wire` arm in `interact` — has already cleared the node and backdrop
/// selections: one subject at a time.)
pub(super) fn select_wire_on_press(
    state: &mut MotionGraphPanelState,
    wire: (u32, u16),
    shift: bool,
) {
    if shift {
        if !state.selected_wires.insert(wire) {
            state.selected_wires.remove(&wire);
        }
    } else {
        state.selected_wires.clear();
        state.selected_wires.insert(wire);
    }
}

/// Collapse a CONSECUTIVE repeat of the same graph verb — the double-dispatch
/// artifact (the store gets each verb from both the focus gate and the shell's cursor
/// router). A human cannot press a key twice in one frame, so an adjacent repeat is
/// never intentional. Distinct verbs, and a repeat from two SEPARATE presses (not
/// adjacent), are preserved — only the back-to-back double is dropped.
pub(super) fn dedup_double_dispatch(keys: Vec<GraphKey>) -> Vec<GraphKey> {
    let mut out: Vec<GraphKey> = Vec::with_capacity(keys.len());
    for k in keys {
        if out.last() == Some(&k) {
            continue;
        }
        out.push(k);
    }
    out
}
