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
use ph2d_editor::{HeroScreen, ToolId, ToolRegistry};

/// Per-frame Motion-tool plumbing. Safe to call every frame; a no-op when the
/// Motion tool is inactive (beyond flipping panel visibility / the split off).
///
/// - `frame_ticks`: fixed steps this frame (`FixedStepReport::ticks`) — advances
///   the transport when playing.
/// - `fixed_dt`: the fixed timestep in seconds (playhead = `tick × fixed_dt`).
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &ToolRegistry,
    motion: &mut MotionState,
    frame_ticks: u32,
    fixed_dt: f64,
) {
    let motion_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("motion"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover) ──────────
    hero.panel_visibility.insert("motion_graph", motion_active);
    hero.panel_visibility.insert("motion_params", motion_active);

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
            apply_graph_intents(motion);
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
/// Each intent is one undo step (snapshot begin → mutate → commit_if_changed).
/// Positions are UI-only (they never touch the cook), so no `mark_dirty`.
#[cfg(feature = "panel-motion-graph")]
fn apply_graph_intents(motion: &mut MotionState) {
    use ph2d_nodegraph::graph::{NodeId, Pos};
    for intent in ph2d_panel_motion_graph::drain_intents() {
        match intent {
            ph2d_panel_motion_graph::GraphIntent::MoveNodes { nodes, dx, dy } => {
                motion.history.begin(&motion.doc);
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
                motion.history.commit_if_changed(&motion.doc);
            }
        }
    }
}
