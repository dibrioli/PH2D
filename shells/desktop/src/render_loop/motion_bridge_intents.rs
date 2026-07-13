//! **The intent funnel** (Motion Nodes M1.E10) — every edit the panel asks for, applied
//! to the shell-owned document. Sibling of `motion_bridge` (shell LOC cap); `super` is
//! `render_loop::motion_bridge`, so the apply helpers, the sub-modules and `reconcile`
//! all resolve through it.
//!
//! **The id a panel intent carries is a VIEW id** (doc 57): it names a node, or it names
//! a collapsed subgraph CARD. This is where that is decoded (`subgraph::target` /
//! `subgraph::resolve_port`) — so that everything downstream, from the connect authority
//! to the plumbing to the toasts, goes on speaking about real nodes and never learns that
//! a fold exists.

use super::{
    MotionState, apply_delete_selection, apply_disconnect, backdrops, connect, edit, reconcile,
    rewire, subgraph,
};
use ph2d_editor::ToastQueue;
use ph2d_editor::screens::layout::CenterSplit;

/// Apply the panel's queued [`GraphIntent`]s to the shell-owned document (M1.E10).
///
/// - **Drag** (`BeginDrag`/`MoveNodes`/`EndDrag`) is a live sequence: the bracket
///   opens the undo step, each incremental delta applies immediately (so the node
///   tracks the cursor with no end-jump), and the release commits one step.
///   Positions are UI-only (they never touch the cook) → no `mark_dirty`.
/// - **Structural** edits (`Connect`/`Disconnect`/`AddNode`/`DeleteSelection`)
///   each are one atomic undo step and change the cook → `mark_dirty`. Connect is
///   validated here (the shell is the authority): a trial clone runs
///   `Graph::connect` (cycle / occupied-input) then `Graph::validate` (typing /
///   membrane), and the edit is kept only when the new edge is legal — else a
///   refusal toast is raised and the document is untouched.
pub(super) fn apply_graph_intents(
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ToastQueue,
    split: &mut CenterSplit,
) {
    use ph2d_nodegraph::graph::Pos;
    use ph2d_panel_motion_graph::GraphIntent;
    for intent in ph2d_panel_motion_graph::drain_intents() {
        match intent {
            GraphIntent::BeginDrag => motion.history.begin(&motion.doc),
            // A drag can carry cards and nodes together (they are both just cards on
            // the canvas). The id says which — and a CARD carries everything inside
            // it, at every depth, or entering it would land the artist on empty
            // canvas a screen away from the card they just dragged.
            GraphIntent::MoveNodes { nodes, dx, dy } => {
                for id in nodes {
                    match subgraph::target(id) {
                        subgraph::Target::Card(sid) => subgraph::translate(motion, sid, dx, dy),
                        subgraph::Target::Node(nid) => {
                            if let Some(p) = motion.doc.graph.pos(nid) {
                                motion.doc.graph.set_pos(
                                    nid,
                                    Pos {
                                        x: p.x + dx,
                                        y: p.y + dy,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            // ── Subgraphs (doc 57) — a fold in the VIEW. Grouping and ungrouping are
            // document edits (undoable) but the GRAPH is untouched, so none of them
            // re-cooks: `mark_dirty` is deliberately absent, exactly as it is for the
            // backdrops. Navigation is not even a document edit.
            GraphIntent::GroupSelection { nodes } => subgraph::group(motion, nodes),
            GraphIntent::Ungroup { id } => subgraph::ungroup(motion, id),
            GraphIntent::EnterSubgraph { id } => subgraph::set_level(motion, Some(id)),
            GraphIntent::GoToLevel { level } => subgraph::set_level(motion, level),
            GraphIntent::EndDrag => motion.history.commit_if_changed(&motion.doc),
            // A wire endpoint may land on a CARD's socket — which stands for a real
            // port inside the group (`subgraph::resolve_port`, the same derivation
            // that drew it). Resolved HERE, so everything downstream — the connect
            // authority, the plumbing, the toasts — goes on speaking about real
            // nodes and never learns that a fold exists.
            GraphIntent::Connect {
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if let Some((f, fp)) = subgraph::resolve_port(motion, from_node, from_port, false)
                    && let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true)
                {
                    connect::apply_connect(motion, toasts, f.0, fp, t.0, tp);
                }
            }
            GraphIntent::Disconnect { to_node, to_port } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    apply_disconnect(motion, toasts, t.0, tp);
                }
            }
            GraphIntent::AddNode { type_name, x, y } => {
                let pre = motion.doc.clone();
                let id = motion.doc.graph.add_node(type_name);
                motion.doc.graph.set_pos(id, Pos { x, y });
                // Sequential-node template (docs/Motion Nodes/03): a feedback
                // host (`state`/`forces` input) lands with its `pre` self-loop
                // already plumbed, so integrate/spring arrive alive instead of
                // frozen at their seed.
                reconcile(motion, &pre.graph);
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
            }
            GraphIntent::DeleteSelection { nodes } => {
                apply_delete_selection(motion, nodes, toasts);
            }
            // F2 — Ctrl+D and the knife (both in `motion_bridge_edit`). A duplicated
            // CARD duplicates its contents (Unreal, on collapsed graphs: "if you copy
            // the collapsed node, it duplicates the internal graph") — the nodes are
            // copied by the ordinary duplicate, the NESTING by `duplicate_nesting`.
            GraphIntent::DuplicateSelection { nodes } => {
                let cards: Vec<u32> = nodes
                    .iter()
                    .filter_map(|v| subgraph::subgraph_of(*v))
                    .collect();
                let mut sources: Vec<u32> = nodes
                    .iter()
                    .filter(|v| subgraph::subgraph_of(**v).is_none())
                    .copied()
                    .collect();
                for sid in &cards {
                    sources.extend(
                        ph2d_motion_doc::subgraph::member_nodes_deep(
                            &motion.doc.subgraphs,
                            &motion.doc.members,
                            *sid,
                        )
                        .iter()
                        .map(|n| n.0),
                    );
                }
                let copy_of = edit::duplicate(motion, sources);
                if !copy_of.is_empty() {
                    // Same undo step: `duplicate` snapshotted the doc BEFORE any of it.
                    let selection = subgraph::duplicate_nesting(motion, &cards, &copy_of);
                    ph2d_panel_motion_graph::request_graph_selection(selection);
                }
            }
            GraphIntent::CutWires { targets } => {
                let targets: Vec<(u32, u16)> = targets
                    .into_iter()
                    .filter_map(|(n, p)| {
                        subgraph::resolve_port(motion, n, p, true).map(|(n, p)| (n.0, p))
                    })
                    .collect();
                edit::cut_wires(motion, toasts, targets);
            }
            // F2 — rewiring (doc 45). Both are one undo step and both re-cook: a reroute is
            // a NODE, and a moved wire is a changed graph.
            GraphIntent::SpliceReroute {
                to_node,
                to_port,
                x,
                y,
            } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    rewire::splice_reroute(motion, toasts, t.0, tp, x, y);
                }
            }
            GraphIntent::MoveWireEnd {
                from_node,
                from_port,
                old_to_node,
                old_to_port,
                new_to,
            } => {
                let src = subgraph::resolve_port(motion, from_node, from_port, false);
                let old = subgraph::resolve_port(motion, old_to_node, old_to_port, true);
                // `None` landing = unplugged (the wire was dropped on empty canvas);
                // a landing that will not resolve is a slot an earlier intent in this
                // same batch took away, and there is nothing to plug into.
                let land = match new_to {
                    Some((n, p)) => {
                        subgraph::resolve_port(motion, n, p, true).map(|(n, p)| Some((n.0, p)))
                    }
                    None => Some(None),
                };
                if let (Some((f, fp)), Some((o, op)), Some(land)) = (src, old, land) {
                    rewire::move_wire_end(motion, toasts, f.0, fp, o.0, op, land);
                }
            }
            GraphIntent::SmartConnect {
                from_node,
                from_port,
                to_type,
                x,
                y,
            } => {
                if let Some((f, fp)) = subgraph::resolve_port(motion, from_node, from_port, false) {
                    edit::smart_connect(motion, toasts, f.0, fp, to_type, x, y);
                }
            }
            // The probe is a READOUT: it points at a node and reads what that node
            // already cooks. No document edit, no undo step, no `mark_dirty`. Probing
            // a CARD reads what the group EMITS (its first output's source) — the
            // question a closed door can still answer.
            GraphIntent::SetProbe { node } => {
                motion.probe = node.and_then(|v| subgraph::probe_target(motion, v));
                motion.probe_ring.clear();
            }
            // Split chrome (E9) — UI-only (no cook / undo). `with_t` clamps the
            // fraction; orientation flips preserve it.
            GraphIntent::SetSplit { t } => {
                if split.is_split() {
                    *split = split.with_t(t);
                }
            }
            GraphIntent::SetSplitVertical { vertical } => {
                *split = if vertical {
                    split.to_vertical()
                } else {
                    split.to_horizontal()
                };
            }
            // Transport play/pause (Space) — no doc edit / undo. Toggles the
            // EDITOR's clock (W4.T7), so pausing the graph pauses the timeline
            // and vice versa; the cook simply follows wherever the playhead is.
            GraphIntent::TogglePlay => {
                playhead.toggle_play();
            }
            // Backdrops (F2) — document state, so undoable, but UI-only, so NONE
            // of these re-cooks (`mark_dirty` is deliberately absent: a cook
            // cannot depend on decoration). Details in `motion_bridge_backdrops`.
            GraphIntent::AddBackdrop { x, y, w, h } => backdrops::add(motion, x, y, w, h),
            GraphIntent::MoveBackdrop { id, dx, dy } => backdrops::translate(motion, id, dx, dy),
            GraphIntent::ResizeBackdrop { id, left, dx, dy } => {
                backdrops::resize(motion, id, left, dx, dy)
            }
            GraphIntent::DeleteBackdrop { id } => backdrops::delete(motion, id),
            GraphIntent::SetBackdropTitle { id, title } => backdrops::set_title(motion, id, title),
            GraphIntent::SetBackdropColor { id, color } => backdrops::set_color(motion, id, color),
        }
    }
}
