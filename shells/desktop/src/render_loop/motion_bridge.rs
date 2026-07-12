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
//! 3. **Per-frame cook** — cook the graph's sink, at the tick the editor's ONE
//!    clock (`ph2d_core::Playhead`, W4.T7) is standing on, into the reused
//!    `MotionState.instances` buffer. Motion keeps NO transport of its own.
//!    The render loop injects that slice via `SpriteRenderer::render_with_extra`
//!    (`present.rs`) — the cooked stream draws without being spawned into the
//!    ECS `PresentWorld` (stream ≠ ECS, ADR-0035).
//!
//! **Zero concrete-tool downcast:** unlike `vector_bridge`, the document lives in
//! `MotionState` (shell), not the tool, so the central render loop stays
//! downcast-free without this bridge reaching into a concrete tool at all.
//!
//! The params-panel side (scalar/enum/checkbox rows + OKLCH colour authoring)
//! lives in the sibling [`params`] module, and the headless tests in
//! `motion_bridge_tests.rs` (+ `motion_bridge_param_tests.rs` and
//! `motion_bridge_plumbing_tests.rs`), all split out for the HR-18 LOC cap.

use crate::motion_state::MotionState;
use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{HeroScreen, ToastQueue, ToolId, ToolRegistry};

#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_params.rs"]
mod params;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_plumbing.rs"]
mod plumbing;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_backdrops.rs"]
mod backdrops;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_tests.rs"]
mod tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_param_tests.rs"]
mod param_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_plumbing_tests.rs"]
mod plumbing_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_backdrop_tests.rs"]
mod backdrop_tests;

/// Per-frame Motion-tool plumbing. Safe to call every frame; a no-op when the
/// Motion tool is inactive (beyond flipping panel visibility / the split off).
///
/// - `playhead`: the editor's ONE clock (`ph2d_core::Playhead`, W4.T7). Motion
///   READS it for the tick to cook and WRITES it for transport intents (Space,
///   auto-play on entry) — it no longer keeps a transport of its own.
/// - `fixed_dt`: the fixed timestep in seconds (`tick × fixed_dt` = seconds).
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
    playhead: &mut ph2d_core::Playhead,
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
                // Auto-play on entry so time-driven behaviours animate live the
                // moment the tool opens (Cavalry/AE preview semantics). Space
                // toggles pause; nothing moves until a `Temporal` node is wired.
                // This plays the EDITOR's clock (W4.T7) — the timeline runs with
                // the graph, which is the point of there being only one.
                playhead.play();
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
            apply_graph_intents(motion, playhead, toasts, &mut hero.view.center_split);
            // Publish the addable-node catalog for the add-menu. Rebuilt each
            // active frame (cheap: ~dozens of `Copy` entries) alongside the
            // snapshot; memoizing it is a follow-up like the snapshot's own
            // dirty gate.
            ph2d_panel_motion_graph::set_current_node_catalog(build_catalog(&motion.registry));
        } else {
            ph2d_panel_motion_graph::set_current_node_catalog(Vec::new());
        }
        ph2d_panel_motion_graph::set_current_motion_graph(motion_active.then(|| {
            let mut snap =
                ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
            // The backdrops ride the DOCUMENT, not the graph (they are decoration
            // and never cook), so `snapshot_from` — which only sees the graph —
            // cannot know them. Resolve them here, into the panel's own view type.
            snap.backdrops = motion
                .doc
                .backdrops
                .iter()
                .map(|b| ph2d_panel_motion_graph::GraphBackdropView {
                    id: b.id,
                    x: b.x,
                    y: b.y,
                    w: b.w,
                    h: b.h,
                    color: b.color,
                    title: b.title.clone(),
                })
                .collect();
            snap
        }));
    }

    // ── Params panel (M1.P1): apply the selected node's param edits, then
    // publish its params snapshot ────────────────────────────────────────────
    // Needs BOTH panels: the selection comes from the graph panel, the rows go
    // to the params panel. Apply BEFORE the cook so a param change shows this
    // frame; publish the freshly-mutated doc so the panel reflects it.
    #[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
    {
        if motion_active {
            // Apply this frame's edits (colour picks + scalar sliders) BEFORE
            // rebuilding, so the panel reflects them; then seed each colour
            // swatch's picker colour from the freshly-mutated snapshot.
            params::apply_param_edits(motion, &hero.store);
            let snap = params::build_params_snapshot(motion);
            if let Some(s) = &snap {
                params::seed_color_swatches(&mut hero.store, s);
            }
            ph2d_panel_motion_params::set_current_params(snap);
        } else {
            ph2d_panel_motion_params::set_current_params(None);
        }
    }

    if !motion_active {
        return;
    }

    // ── 3. Cook the sink at the tick the PLAYHEAD is on (W4.T7: one clock) ────
    // The render output IS the Output node — the sink follows the graph (wire a
    // chain into an Output node and it draws). `None` (no Output node) → the pump
    // renders nothing. Recomputed each frame so add/delete/rewire just works.
    motion.sinks = output_nodes(&motion.doc.graph);
    // Time scopes (M2.N1): each `motion.time_remap` node rewrites the clock of
    // its upstream subtree. Rebuilt per frame — one pass over the node list, and
    // empty for a graph with no remapper (the common case), so the cook takes
    // its unscoped path unchanged.
    let scopes = ph2d_node_motion_time_remap::time_scopes(&motion.doc.graph, &motion.registry);
    let target = motion_tick(playhead, fixed_dt);
    for tick in ticks_owed(motion.pump.last_cooked_tick(), target) {
        motion.pump.advance_or_scrub_scoped(
            &motion.doc.graph,
            &motion.registry,
            &motion.sinks,
            tick,
            |t| t as f64 * fixed_dt,
            motion.default_uv_rect,
            motion.default_size,
            &scopes,
        );
    }
}

/// The ticks the cook still owes to reach `target`, given the last one it rendered.
///
/// **Forward: every tick, never a skip.** One cook + `pre`-advance per FIXED TICK,
/// never per frame (M2-dynamics). A sequential node's trajectory (`integrate`,
/// `spring`, `verlet_rope`) is the SUM of its steps, so a slow frame that produced
/// two fixed steps — or a playhead running at `rate 2` — must sim BOTH, or the
/// motion would depend on the frame rate (plan §1.4: dt fixo). The common frame
/// owes exactly one tick, which takes the pump's cheap forward path.
///
/// **Backwards or a jump: one call.** The playhead was scrubbed, sought, or wrapped
/// its loop. [`MotionCookPump::advance_or_scrub_scoped`] restores the newest
/// checkpoint at or before the target and re-sims forward, bit-exact (M2.N2) —
/// walking there tick by tick would instead re-cook from the ring on every step.
/// Reading the marching future would show a spring mid-flight at a time it never
/// was in.
///
/// **Standing still: the same tick.** A paused playhead re-issues its current tick,
/// which the pump early-returns on unless a param drag / rewire dirtied the cook
/// (zero-alloc paused frame, M0.T12).
fn ticks_owed(last_cooked: Option<u64>, target: u64) -> std::ops::RangeInclusive<u64> {
    match last_cooked {
        Some(last) if target > last => last + 1..=target,
        _ => target..=target,
    }
}

/// The fixed tick the playhead is standing on — the Motion cook's clock, DERIVED
/// (W4.T7). Motion keeps no transport of its own: it used to, and two clocks that
/// each advance themselves are two clocks that drift.
///
/// The cook is tick-based on purpose (a fixed `dt` is what makes a spring
/// deterministic, plan §1.4) while the playhead is a continuous position in
/// seconds — so the seam rounds. At `rate == 1` the playhead's time is an exact
/// multiple of `fixed_dt` and the rounding is a no-op; a seek to mid-tick lands on
/// the nearest one.
fn motion_tick(playhead: &ph2d_core::Playhead, fixed_dt: f64) -> u64 {
    if fixed_dt <= 0.0 {
        return 0;
    }
    // `Playhead::time` is `>= 0` by construction; the clamp is belt-and-braces
    // against a NaN reaching the cast, which would saturate to 0 silently.
    let t = playhead.time() / fixed_dt;
    if t.is_finite() && t > 0.0 {
        t.round() as u64
    } else {
        0
    }
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
fn apply_graph_intents(
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ToastQueue,
    split: &mut CenterSplit,
) {
    use ph2d_nodegraph::graph::{NodeId, Pos};
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
            } => apply_connect(motion, toasts, from_node, from_port, to_node, to_port),
            GraphIntent::Disconnect { to_node, to_port } => {
                apply_disconnect(motion, toasts, to_node, to_port);
            }
            GraphIntent::AddNode { type_name, x, y } => {
                let pre = motion.doc.clone();
                let id = motion.doc.graph.add_node(type_name);
                motion.doc.graph.set_pos(id, Pos { x, y });
                // Sequential-node template (docs/Motion Nodes/03): a feedback
                // host (`state`/`forces` input) lands with its `pre` self-loop
                // already plumbed, so integrate/spring arrive alive instead of
                // frozen at their seed.
                plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
            }
            GraphIntent::DeleteSelection { nodes } => {
                apply_delete_selection(motion, nodes);
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
            GraphIntent::ResizeBackdrop { id, dw, dh } => backdrops::resize(motion, id, dw, dh),
            GraphIntent::DeleteBackdrop { id } => backdrops::delete(motion, id),
            GraphIntent::SetBackdropTitle { id, title } => backdrops::set_title(motion, id, title),
            GraphIntent::SetBackdropColor { id, color } => backdrops::set_color(motion, id, color),
        }
    }
}

/// Apply a `Connect` intent: validate against the shell-owned document (the
/// shell is the authority) and commit iff the new edge is legal.
///
/// A wire that would close a **cycle** has exactly ONE legal meaning in this
/// substrate: 1-tick feedback (the Lustre `pre`, ADR-0032) — so it is retried
/// as a delayed edge instead of refused, with an informative toast. This is
/// the EXPERT path (arbitrary feedback); the bread-and-butter force loop is
/// never hand-wired — `plumbing` derives it from the `forces` port
/// (docs/Motion Nodes/03).
#[cfg(feature = "panel-motion-graph")]
fn apply_connect(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    to_node: u32,
    to_port: u16,
) {
    use ph2d_editor::Toast;
    use ph2d_nodegraph::graph::{Edge, EdgeError, NodeId};
    let fwd = Edge {
        from: (NodeId(from_node), from_port),
        to: (NodeId(to_node), to_port),
        delayed: false,
    };
    let mut trial = motion.doc.graph.clone();
    // A feedback port (or a plumbed branch head) holds an engine-managed `pre`;
    // the user's forward wire takes precedence — clear it here and let the
    // post-commit reconcile re-plumb the state entry at the branch's new head.
    // An expert (non-managed) `pre` is NOT cleared: the connect then fails with
    // the ordinary occupied-port refusal.
    plumbing::clear_managed_pre_at(&mut trial, &motion.registry, NodeId(to_node), to_port);
    let (attempt, edge) = match trial.connect(fwd) {
        Err(EdgeError::WouldCycle) => {
            let pre_edge = Edge {
                delayed: true,
                ..fwd
            };
            (trial.connect(pre_edge), pre_edge)
        }
        other => (other, fwd),
    };
    match attempt {
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
            } else if scopes_a_sequential_node(&trial) {
                // A spring / integrate upstream of a Time Remap would be asked
                // to integrate a recurrence on a rewritten clock — the cook
                // refuses it (`SequentialInTimeScope`) and the scene goes dark.
                // Refuse the WIRE instead, with the reason (M2.N1).
                toasts.push(Toast::warning(
                    "Can't connect: Time Remap can't rewrite time for a spring or integrate upstream",
                ));
            } else {
                let pre = motion.doc.clone();
                motion.doc.graph = trial;
                plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
                if edge.delayed {
                    toasts.push(Toast::info("Connected as 1-tick feedback (pre)"));
                }
            }
        }
    }
}

/// Does any `motion.time_remap` in `graph` now have a **sequential** node (a
/// spring / integrate, i.e. one fed by a `pre` edge) in its upstream subtree?
/// Such a node integrates a recurrence over the outer tick and has no defined
/// behaviour on a rewritten clock (`CookError::SequentialInTimeScope`), so the
/// editor refuses the wire that would create the situation rather than let the
/// cook drop the sink and the scene go dark (M2.N1, plan §1.5).
#[cfg(feature = "panel-motion-graph")]
fn scopes_a_sequential_node(graph: &ph2d_nodegraph::graph::Graph) -> bool {
    graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ph2d_node_motion_time_remap::MANIFEST.name)
        .any(|n| ph2d_node_motion_time_remap::scopes_a_sequential_node(graph, n.id))
}

/// Apply a `Disconnect` intent. An engine-managed `pre` (the plumbing badge)
/// is not hand-deletable — it would re-derive on the next reconcile anyway —
/// so the gesture steers the user to the edit that DOES change topology.
/// Everything else disconnects, then the plumbing re-heals (a chain pulled off
/// a `forces` port gets the host its self-loop back; a chain split mid-way
/// moves the state entry to the new dangling head).
#[cfg(feature = "panel-motion-graph")]
fn apply_disconnect(motion: &mut MotionState, toasts: &mut ToastQueue, to_node: u32, to_port: u16) {
    use ph2d_nodegraph::graph::NodeId;
    if plumbing::is_managed_pre(
        &motion.doc.graph,
        &motion.registry,
        NodeId(to_node),
        to_port,
    ) {
        toasts.push(ph2d_editor::Toast::info(
            "State wiring is automatic - disconnect the chain from the forces port instead",
        ));
        return;
    }
    let pre = motion.doc.clone();
    if motion
        .doc
        .graph
        .disconnect(NodeId(to_node), to_port)
        .is_some()
    {
        plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}

/// Apply a `DeleteSelection` intent. Deleting a mid-chain force re-heals the
/// state entry onto the branch's new dangling head; orphaned plumbing dies
/// with the branch (before/after diff in `plumbing`). The sinks are
/// re-resolved from the Output nodes each frame (before the cook), so deleting
/// one cleanly stops its scene — no manual sink bookkeeping here.
#[cfg(feature = "panel-motion-graph")]
fn apply_delete_selection(motion: &mut MotionState, nodes: Vec<u32>) {
    use ph2d_nodegraph::graph::NodeId;
    let pre = motion.doc.clone();
    let mut changed = false;
    for id in nodes {
        changed |= motion.doc.graph.remove_node(NodeId(id));
    }
    if changed {
        plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, &pre.graph);
        motion.history.push_undo(pre);
        motion.pump.mark_dirty();
    }
}

/// The graph's render sinks: **every** `motion.output` node, in node-id order
/// (deterministic). Each lowers onto the same instance buffer, so a document can
/// hold several independent scenes — the default one holds a grid rig and a
/// particle fountain. Empty (nothing renders) until a chain is wired into an
/// Output node.
fn output_nodes(graph: &ph2d_nodegraph::graph::Graph) -> Vec<ph2d_nodegraph::graph::NodeId> {
    // `"motion.output"` is the node's canonical type name (as authored in the
    // default graph); the shell addresses node types by name, like the tool id.
    let mut ids: Vec<_> = graph
        .nodes()
        .iter()
        .filter(|inst| inst.type_name == "motion.output")
        .map(|inst| inst.id)
        .collect();
    ids.sort();
    ids
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
