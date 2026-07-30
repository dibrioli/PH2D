//! Graph keyboard verbs (Motion Nodes M1.E4) — sibling of `interact` (panel LOC cap).
//!
//! Editor-core maps the keycode to a [`GraphKey`]; this decides what the verb DOES.
//! Every one of them is either ephemeral view state (fit / arm the knife / arm the
//! probe) or a single [`GraphIntent`] — the panel never edits the document itself.

use super::{
    GraphIntent, GraphKey, GraphViewSnapshot, Interaction, Menu, MotionGraphPanelState, Rect, View,
    push_intent, subgraph_gesture,
};
use crate::state::MenuBody;

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
        // dispatch: M0 focus gate + the shell's cursor push). Node and backdrop
        // selection are mutually exclusive, so Delete is never ambiguous — and a
        // deleted backdrop takes nothing with it (it owns no nodes; it draws
        // around them).
        GraphKey::Delete => {
            if !state.selected.is_empty() {
                push_intent(GraphIntent::DeleteSelection {
                    nodes: state.selected.iter().copied().collect(),
                });
                state.selected.clear();
            } else if let Some(id) = state.selected_backdrop.take() {
                push_intent(GraphIntent::DeleteBackdrop { id });
            }
            state.interaction = Interaction::Idle;
        }
        // `A` opens the add-node menu at the canvas center (the keyboard verb
        // carries no cursor position). Idempotent: a second `A` (menu already
        // open) falls through to the no-op arm below.
        GraphKey::Add if state.menu.is_none() => {
            let center = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
            let spawn = View::new(rect, state.view).graph(center.0, center.1);
            state.menu = Some(Menu {
                scroll: 0.0,
                screen: center,
                spawn,
                query: String::new(),
                opened: false,
                body: MenuBody::Library {
                    connect_from: None,
                    splice: None,
                },
            });
        }
        // Ctrl+A — select every node at THIS level (the snapshot is level-scoped). A backdrop is
        // a separate subject (see `select_on_press`), so it clears; an empty graph selects
        // nothing, idempotent against the double key dispatch.
        GraphKey::SelectAll => {
            state.selected = snap.nodes.iter().map(|n| n.id).collect();
            state.selected_backdrop = None;
        }
        // Space — toggle transport play/pause (the shell owns the transport).
        GraphKey::TogglePlay => push_intent(GraphIntent::TogglePlay),
        _ => {}
    }
}
