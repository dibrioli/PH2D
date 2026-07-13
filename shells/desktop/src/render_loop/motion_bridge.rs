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

#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_color.rs"]
mod color;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_plumbing.rs"]
mod plumbing;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_backdrops.rs"]
mod backdrops;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_edit.rs"]
mod edit;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_readout.rs"]
mod readout;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_rewire.rs"]
mod rewire;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_connect.rs"]
mod connect;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_fold.rs"]
mod fold;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_subgraph.rs"]
mod subgraph;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_intents.rs"]
mod intents;
#[cfg(feature = "panel-motion-graph")]
use intents::apply_graph_intents;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_subgraph_tests.rs"]
mod subgraph_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_tests.rs"]
mod tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_param_tests.rs"]
mod param_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_range_tests.rs"]
mod range_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_plumbing_tests.rs"]
mod plumbing_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_backdrop_tests.rs"]
mod backdrop_tests;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_edit_tests.rs"]
mod edit_tests;

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
            // The level the artist is standing in can stop existing under their feet
            // (an undo that unmakes the group). Checked AFTER the intents — an undo
            // arrives as one — and before anything is published, so the fold never
            // runs against a room that is not there.
            subgraph::clamp_level(motion);
            // Publish the addable-node catalog for the add-menu. Rebuilt each
            // active frame (cheap: ~dozens of `Copy` entries) alongside the
            // snapshot; memoizing it is a follow-up like the snapshot's own
            // dirty gate.
            ph2d_panel_motion_graph::set_current_node_catalog(build_catalog(&motion.registry));
        } else {
            ph2d_panel_motion_graph::set_current_node_catalog(Vec::new());
        }
        // The probe's reading for this frame (a memo lookup on the pump's own cook —
        // see `edit::sample_probe`). Taken before the snapshot borrow.
        //
        // At the COOK's time, not the playhead's raw seconds: the memo is keyed by the tick
        // the pump cooked, and mid-tick (a scrub lands anywhere) those two differ. Asking for
        // a time nobody cooked is how a readout starts lying.
        let cook_time = motion_time(playhead, fixed_dt);
        let probe = motion_active
            .then(|| edit::sample_probe(motion, cook_time))
            .flatten();
        ph2d_panel_motion_graph::set_current_motion_graph(motion_active.then(|| {
            let mut snap =
                ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
            snap.probe = probe;
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
            // What each card is DOING this frame, read out of the pump's memo (`Cook::peek`,
            // never a second cook): its readout, the mass of its stream (the wire's width),
            // and whether that changed since last frame (the wire's march). A node no sink
            // consumes has no entry and stays blank — which is the diagnosis, not a gap.
            readout::stamp(motion, &mut snap);
            // **The fold** (doc 57), LAST: everything above published the whole flat
            // graph, and this cuts it down to the level the artist is standing in —
            // folding the nested nodes into cards (which is why it runs after the
            // readouts: a card aggregates what its members are doing) and drawing the
            // outsiders that touch the boundary as ghosts. A document with no
            // subgraphs pays a breadcrumb and nothing else.
            fold::fold(motion, &mut snap);
            // The ONE clock the marching dashes read (`ph2d_core::Playhead`, W4.T7). The panel
            // has none of its own: a flow animation driven by a paint counter would keep
            // marching on a paused graph.
            snap.now = cook_time as f32;
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
                color::seed_color_swatches(&mut hero.store, s);
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
pub(crate) fn motion_tick(playhead: &ph2d_core::Playhead, fixed_dt: f64) -> u64 {
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

/// The COOK's time in seconds — the tick of [`motion_tick`] measured back out in seconds.
///
/// This is what a readout (probe, flow marching) must be sampled at, and it is NOT the same
/// as `Playhead::time()`: the playhead is continuous and a scrub lands mid-tick, while the
/// pump's memo only ever holds the tick it cooked. Reading the panel at the raw playhead time
/// would ask the memo for an instant that was never simulated — the derived-coordinate trap
/// (the seed must match the sample), and it has bitten this codebase before.
fn motion_time(playhead: &ph2d_core::Playhead, fixed_dt: f64) -> f64 {
    motion_tick(playhead, fixed_dt) as f64 * fixed_dt
}

/// **Re-derive everything the graph's SHAPE implies**, after any structural edit —
/// the ONE funnel every mint and every rewire goes through:
///
/// - the **`pre` plumbing** (a sequential node's self-loop follows its `forces` chain);
/// - the **membership** of a node that was just minted (doc 57): created while the
///   artist is inside a group, it belongs to that group — otherwise it would land at
///   the root and **vanish the instant it was created**.
///
/// Both are functions of `graph` vs `before`, and both are wrong the moment somebody
/// adds a fifth way to mint a node and forgets one of them. So there is one door.
#[cfg(feature = "panel-motion-graph")]
pub(super) fn reconcile(motion: &mut MotionState, before: &ph2d_nodegraph::graph::Graph) {
    plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, before);
    subgraph::adopt_new(motion, before);
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
/// Apply a `Disconnect` intent. An engine-managed `pre` (the plumbing badge) is not
/// hand-deletable — it would re-derive on the next reconcile anyway — so the gesture steers
/// the user to the edit that DOES change topology. Everything else disconnects, the plumbing
/// re-heals (a chain pulled off a `forces` port gets its host's self-loop back; a chain split
/// mid-way moves the state entry to the new dangling head),.
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
        reconcile(motion, &pre.graph);
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
    let pre = motion.doc.clone();
    let mut changed = false;
    for id in nodes {
        match subgraph::target(id) {
            // A collapsed card IS its contents (Nuke: "the original nodes are
            // replaced with the Group node"), so deleting it deletes them — every
            // member, at every depth, and the decoration that lived in there.
            subgraph::Target::Card(sid) => changed |= subgraph::delete_deep(motion, sid),
            subgraph::Target::Node(nid) => {
                if motion.doc.graph.remove_node(nid) {
                    motion.doc.forget_nodes(&[nid]);
                    changed = true;
                }
            }
        }
    }
    if changed {
        reconcile(motion, &pre.graph);
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
/// Build the addable-node catalog from the registry (canonical name + English
/// display label + category), sorted by category then label so the menu groups
/// by color (the palette teaches the library map, plan §2.4).
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
                // Straight off the manifest (`&'static`), so the panel can filter
                // the smart-connect menu by what each type can actually take.
                inputs: m.inputs,
            }
        })
        .collect();
    v.sort_by(|a, b| (a.category as u8, a.display).cmp(&(b.category as u8, b.display)));
    v
}
