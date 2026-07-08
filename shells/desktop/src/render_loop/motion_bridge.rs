//! Motion Nodes tool ⟷ shell bridge (Motion Nodes M0.T10). Replaces the retired
//! `motion_smoke` debug path with the production per-frame cook.
//!
//! Per-frame jobs (mirror of `vector_bridge`), all no-ops unless the `motion`
//! tool is active:
//!
//! 1. **Panel visibility** — show the docked graph + params panels; hide the
//!    real Inspector (edge-triggered) so they don't both claim the slot.
//! 2. **Center split** — split the center into scene ⟂ graph on activate
//!    (remembered orientation, default `Horizontal { t: 0.55 }`), restore to
//!    `None` on deactivate.
//! 3. **Per-frame cook** — advance the transport by this frame's fixed steps,
//!    then cook the graph's sink into the reused `MotionState.instances` buffer.
//!    The render loop injects that slice via `SpriteRenderer::render_with_extra`
//!    (`present.rs`) — the cooked stream draws without being spawned into the
//!    ECS `PresentWorld` (stream ≠ ECS, ADR-0035).
//!
//! **Zero concrete-tool downcast:** unlike `vector_bridge`, the document lives in
//! `MotionState` (shell), not the tool, so the central render loop stays
//! downcast-free without this bridge reaching into a concrete tool at all.

use crate::motion_state::MotionState;
use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{HeroScreen, ToastQueue, ToolId, ToolRegistry};

/// Per-frame Motion-tool plumbing. Safe to call every frame; a no-op when the
/// Motion tool is inactive (beyond flipping panel visibility / the split off).
///
/// - `frame_ticks`: fixed steps this frame (`FixedStepReport::ticks`) — advances
///   the transport when playing.
/// - `fixed_dt`: the fixed timestep in seconds (playhead = `tick × fixed_dt`).
/// - `cursor`: the latest pointer position (screen px) — drives the cursor-gated
///   graph keyboard focus (Blender-style F acts on the hovered area).
/// - `toasts`: the shell toast queue — the connect authority raises a refusal
///   toast here when a dragged edge is rejected (cycle / occupied / typing /
///   membrane).
#[cfg_attr(not(feature = "panel-motion-graph"), allow(unused_variables))]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &ToolRegistry,
    motion: &mut MotionState,
    frame_ticks: u32,
    fixed_dt: f64,
    cursor: (f32, f32),
    toasts: &mut ToastQueue,
) {
    let motion_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("motion"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover) ──────────
    hero.panel_visibility.insert("motion_graph", motion_active);
    hero.panel_visibility.insert("motion_params", motion_active);

    // Graph keyboard focus follows the cursor, re-evaluated EVERY frame (not just
    // on move) so a cursor that stopped over the graph before the panel published
    // its rect still gets focus by the time a key is pressed. `panel_rect` is from
    // last frame's paint (stable); `None` off the graph → the scene owns keys.
    let over_graph = motion_active
        && hero
            .store
            .panel_rect(ph2d_editor::ids::MOTION_GRAPH_PANEL)
            .is_some_and(|r| r.contains(cursor.0, cursor.1));
    hero.store
        .set_graph_focused(over_graph.then_some(ph2d_editor::ids::MOTION_GRAPH_PANEL));

    // ── 2. Center split + Inspector takeover — edge-triggered on activation ──
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(motion_active, Ordering::Relaxed);
        if was != motion_active {
            hero.panel_visibility.insert("inspector", !motion_active);
            if motion_active {
                // Split into scene ⟂ graph. Keep any orientation the user already
                // chose (SplitH/SplitV chips); default to Cavalry-style horizontal.
                if !hero.view.center_split.is_split() {
                    hero.view.center_split = CenterSplit::Horizontal {
                        t: CenterSplit::T_DEFAULT,
                    };
                }
            } else {
                hero.view.center_split = CenterSplit::None;
            }
        }
    }

    // ── Apply the panel's edits, then publish the fresh view (M1.E10) ──────
    // The panel pushed `GraphIntent`s during last frame's paint; apply them to
    // the doc (each a single undo step) BEFORE rebuilding the snapshot so the
    // change shows this frame. Rebuilt each active frame (Phase 1a); a dirty
    // gate lands later. `None` while inactive → no allocation off the editor.
    #[cfg(feature = "panel-motion-graph")]
    {
        if motion_active {
            apply_graph_intents(motion, toasts);
            // Publish the addable-node catalog for the add-menu. Rebuilt each
            // active frame (cheap: ~dozens of `Copy` entries) alongside the
            // snapshot; memoizing it is a follow-up like the snapshot's own
            // dirty gate.
            ph2d_panel_motion_graph::set_current_node_catalog(build_catalog(&motion.registry));
        } else {
            ph2d_panel_motion_graph::set_current_node_catalog(Vec::new());
        }
        ph2d_panel_motion_graph::set_current_motion_graph(
            motion_active.then(|| {
                ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry)
            }),
        );
    }

    if !motion_active {
        return;
    }

    // ── 3. Advance transport + cook the sink (skipped when paused/unchanged) ──
    // `advance` is a no-op while paused, so the tick only moves when playing or
    // scrubbing → the pump skips a static paused frame (zero-alloc, M0.T12). No
    // framing node in M0, so the pump samples one opaque atlas tile (set in
    // init.rs) at a sub-spacing size → clean, distinct dots.
    motion.transport.advance(frame_ticks as u64);
    let playhead = motion.playhead(fixed_dt);
    let tick = motion.transport.tick;
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        motion.sink,
        tick,
        playhead,
        motion.default_uv_rect,
        motion.default_size,
    );
}

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
#[cfg(feature = "panel-motion-graph")]
fn apply_graph_intents(motion: &mut MotionState, toasts: &mut ToastQueue) {
    use ph2d_editor::Toast;
    use ph2d_nodegraph::graph::{Edge, NodeId, Pos};
    use ph2d_panel_motion_graph::GraphIntent;
    for intent in ph2d_panel_motion_graph::drain_intents() {
        match intent {
            GraphIntent::BeginDrag => motion.history.begin(&motion.doc),
            GraphIntent::MoveNodes { nodes, dx, dy } => {
                for id in nodes {
                    let nid = NodeId(id);
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
            GraphIntent::EndDrag => motion.history.commit_if_changed(&motion.doc),
            GraphIntent::Connect {
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                let edge = Edge {
                    from: (NodeId(from_node), from_port),
                    to: (NodeId(to_node), to_port),
                    delayed: false,
                };
                let mut trial = motion.doc.graph.clone();
                match trial.connect(edge) {
                    Err(e) => {
                        toasts.push(Toast::warning(connect_err_msg(e)));
                    }
                    Ok(()) => {
                        let rejected = match trial.validate(&motion.registry) {
                            Ok(()) => false,
                            Err(viols) => viols.iter().any(|v| violation_blocks_edge(v, edge)),
                        };
                        if rejected {
                            toasts.push(Toast::warning("Can't connect: incompatible ports"));
                        } else {
                            let pre = motion.doc.clone();
                            motion.doc.graph = trial;
                            motion.history.push_undo(pre);
                            motion.pump.mark_dirty();
                        }
                    }
                }
            }
            GraphIntent::Disconnect { to_node, to_port } => {
                let pre = motion.doc.clone();
                if motion
                    .doc
                    .graph
                    .disconnect(NodeId(to_node), to_port)
                    .is_some()
                {
                    motion.history.push_undo(pre);
                    motion.pump.mark_dirty();
                }
            }
            GraphIntent::AddNode { type_name, x, y } => {
                let pre = motion.doc.clone();
                let id = motion.doc.graph.add_node(type_name);
                motion.doc.graph.set_pos(id, Pos { x, y });
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
            }
            GraphIntent::DeleteSelection { nodes } => {
                let pre = motion.doc.clone();
                let mut changed = false;
                for id in nodes {
                    changed |= motion.doc.graph.remove_node(NodeId(id));
                }
                if changed {
                    // Deleting the sink stops the cook cleanly rather than
                    // pointing it at a phantom node.
                    if let Some(s) = motion.sink
                        && motion.doc.graph.node(s).is_none()
                    {
                        motion.sink = None;
                    }
                    motion.history.push_undo(pre);
                    motion.pump.mark_dirty();
                }
            }
        }
    }
}

/// Human-readable reason a structural `connect` was rejected (add-menu toast).
#[cfg(feature = "panel-motion-graph")]
fn connect_err_msg(e: ph2d_nodegraph::graph::EdgeError) -> &'static str {
    use ph2d_nodegraph::graph::EdgeError;
    match e {
        EdgeError::WouldCycle => "Can't connect: would create a cycle",
        EdgeError::InputAlreadyConnected => "Can't connect: input already wired",
        EdgeError::UnknownNode => "Can't connect: unknown node",
    }
}

/// Does a validation violation reject *this* just-added edge (as opposed to a
/// pre-existing problem elsewhere)? Only a type mismatch or a membrane crossing
/// on the same endpoints blocks the connect.
#[cfg(feature = "panel-motion-graph")]
fn violation_blocks_edge(
    v: &ph2d_nodegraph::graph::Violation,
    edge: ph2d_nodegraph::graph::Edge,
) -> bool {
    use ph2d_nodegraph::graph::Violation;
    match v {
        Violation::TypeMismatch { from, to } | Violation::Membrane { from, to } => {
            *from == edge.from && *to == edge.to
        }
        _ => false,
    }
}

/// Build the addable-node catalog from the registry (canonical name + English
/// display label + category), sorted by category then label so the menu groups
/// by color (the palette teaches the library map, plan §2.4).
#[cfg(feature = "panel-motion-graph")]
fn build_catalog(
    registry: &ph2d_node_registry::NodeRegistry,
) -> Vec<ph2d_panel_motion_graph::NodeChoice> {
    use ph2d_node_registry::NodeUiCategory;
    use ph2d_panel_motion_graph::NodeChoice;
    let mut v: Vec<NodeChoice> = registry
        .manifests()
        .map(|m| {
            let ui = registry.ui_manifest(m.id);
            NodeChoice {
                type_name: m.name,
                display: ui.map(|u| u.display_name).unwrap_or(m.name),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
            }
        })
        .collect();
    v.sort_by(|a, b| (a.category as u8, a.display).cmp(&(b.category as u8, b.display)));
    v
}
