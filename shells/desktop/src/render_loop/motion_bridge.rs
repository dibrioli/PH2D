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

    if !motion_active {
        return;
    }

    // ── 3. Advance transport + cook the sink into the reused buffer ────────
    motion.transport.advance(frame_ticks as u64);
    let playhead = motion.playhead(fixed_dt);
    match motion.sink {
        Some(sink) => {
            // Disjoint field borrows (cook / doc / registry / instances).
            let _ = ph2d_eval_motion::evaluate_motion_into(
                &mut motion.cook,
                &motion.doc.graph,
                &motion.registry,
                sink,
                playhead,
                // No framing node in M0 → instances have no `uv_rect` column;
                // sample one opaque atlas tile so the raw output reads as clean
                // solid quads (set from the atlas in init.rs).
                motion.default_uv_rect,
                &mut motion.instances,
            );
            // Advance the 1-tick `pre` feedback once per frame, same playhead
            // (harmless when the graph has no `pre` edge, as the M0 vertical).
            let _ = motion
                .cook
                .advance_tick(&motion.doc.graph, &motion.registry, playhead);
        }
        None => motion.instances.clear(),
    }
}
