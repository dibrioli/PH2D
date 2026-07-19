//! Shell-side driver for the ECS physics bridge (ADR-0131 W1).
//!
//! Thin by design: it reads the ONE clock (`ph2d_core::Playhead`, the same
//! one Motion stands on since W4.T7) and hands the primitive `(playing,
//! target tick)` to the crate's [`PhysicsBridge`]. The play-vs-settle split
//! lives inside `PhysicsBridge::dispatch` (the physics analog of
//! `motion_bridge::ticks_owed`), so there is no second clock and no second
//! copy of the tick logic.
//!
//! Called each frame BETWEEN the timeline apply and `sim_extract`, so a
//! body's readback `Transform` renders the same frame it was computed.

use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// The fixed tick the playhead currently sits on — `round(time / dt)`, the
/// same mapping Motion uses (`motion_tick`), so physics and motion step on
/// the identical tick grid.
fn physics_tick(playhead: &Playhead, fixed_dt: f64) -> u64 {
    (playhead.time() / fixed_dt).round().max(0.0) as u64
}

/// Step (or settle) the rigid world for this frame. Play = advance the owed
/// ticks + read poses back into `Transform`; paused = settle bodies to the
/// authored `Transform` (read-only on `Transform`, so an idle paused frame
/// produces no snapshot diff — the fixed-point rule).
///
/// `simulate` is the transport's Physics toggle
/// (`TimelineFlags::simulate_physics`, off by default). Disarmed, the world is
/// HELD rather than skipped — see [`PhysicsBridge::hold`], which explains why
/// those are different things.
pub(crate) fn dispatch(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    playhead: &Playhead,
    fixed_dt: f64,
    doc: &mut ph2d_timeline::TimelineDoc,
    simulate: bool,
) {
    let target = physics_tick(playhead, fixed_dt);
    if !simulate {
        bridge.hold(sim, target);
        return;
    }
    // ⚠️ The bridge is told where the timeline puts scene-driven bodies at each
    // tick it runs. It matters on the REWIND path above all: a scrub replays
    // ticks, and a kinematic body frozen at its rest pose through that replay is
    // a platform that is not where it was, so a scrub and a play disagree about
    // the same instant. Passing `FrozenScene` here would compile and would be
    // that bug (see `bake_and_scrub_agree_with_play`).
    let mut scene = super::physics_bake::TimelineScene { doc, fixed_dt };
    bridge.dispatch_with_scene(sim, playhead.is_playing(), target, &mut scene);
}
