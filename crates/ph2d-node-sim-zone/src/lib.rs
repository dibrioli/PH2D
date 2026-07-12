#![forbid(unsafe_code)]
//! **The Simulation Zone** (Motion Nodes O4, doc 48) — the highest ceiling the module's
//! re-entrancy study set out, and the last one left standing.
//!
//! ## What it is
//!
//! A zone holds a stream of live state across ticks. Whatever chain the artist wires into its
//! `state` port is the zone's **interior**: it receives last tick's state, does anything at all
//! to it, and hands it back. Then the zone emits it — and holds it for the next tick.
//!
//! ```text
//!   grid ─→ init ┌──────────┐ out ─→ output
//!                │ sim.zone │
//!        state ←─┴──────────┘
//!          ↑                      ⊙ ← the interior gets last tick's state here
//!   force.wind → sim.step → motion.cull
//! ```
//!
//! ## Why it beats the forces branch it joins (and does not replace)
//!
//! `motion.integrate`'s `forces` port (O1) is a zone with an implicit boundary: its interior
//! may only ACCUMULATE acceleration, because the integrator owns the state and everything on
//! the branch is `Pure`. A zone's interior owns nothing and may do everything — and that turns
//! nodes the library already has into things they could not be before:
//!
//! - **`motion.cull` becomes a KILL.** Outside a zone, culling drops elements from *this
//!   frame's* stream and the next frame rebuilds them. Inside, the state carries the survivors
//!   forward — so what you cull STAYS dead. Blender's zone and Houdini's POP kill, out of a
//!   filter that was already there.
//! - **`motion.combine` becomes a BIRTH.** Merge newborns into the live state and they persist,
//!   with their own history, instead of being re-emitted every frame by a stateless emitter.
//!
//! No new machinery bought either of those. The zone did.
//!
//! ## The engine underneath is the one that was already there
//!
//! The `state` port is a **feedback host** (the O1 convention: an input named `state` or
//! `forces`, of output 0's type). The editor's plumbing owns the `pre` edge — self-loop when
//! the port is bare, into the interior's head when a chain is wired in — and draws it as portal
//! badges rather than a spline. The zone adds no engine, no `Domain`, no contract: it is a node.
//!
//! ## The two things Blender's users found out the hard way
//!
//! 1. **A bare zone FREEZES its input.** Wire nothing into `state` and the plumbing self-loops
//!    it: tick 1 emits `init`, and every tick after re-emits what it emitted before. The
//!    Blender manual's users hit this as a surprise (*"even just a simulation zone that does
//!    nothing freezes the mesh"*) — so the port is called **`init`**, not `in`. It is the
//!    INITIAL state, read once. The name is the documentation.
//! 2. **An empty sim is not an unstarted one.** See `eval` below: this is what
//!    `EvalCtx::started` exists for, and getting it wrong resurrects the dead.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The initial state — read on the first tick and never again (Blender's frozen zone input,
/// named so that the surprise is impossible).
const IN_INIT: usize = 0;
/// The live state coming back from the interior. A **feedback host** port name (`state`), so
/// the editor's plumbing wires the `pre` edge and the artist never draws the loop.
const IN_STATE: usize = 1;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.zone"),
    name: "sim.zone",
    inputs: &[
        PortSpec {
            name: "init",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // It consumes a `pre` edge, so the cook already treats it as sequential (it may not run
    // inside a rewritten time scope, and it never reuses a memo across ticks). `Temporal` is
    // what integrate/spring declare for the same reason.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// **The transients: scratch a tick writes for itself, and the zone must NOT hold.**
///
/// - `accel` is the force nodes' accumulator, spent by the step that applies it.
/// - `falloff` is a MASK — a field a `motion.falloff` computes so the modifiers after it can
///   scale their effect by it (§1.2). It describes *this tick's* authoring intent, not the
///   element.
///
/// A zone that stored them would be a loop that feeds a tick its own leftovers, and the demo
/// found out what that costs: the kill's mask (`falloff` from `motion.falloff`, consumed by
/// `motion.cull`) rode the state back around and **masked the very gravity that made it** —
/// the fringe of the seed stopped falling — and then leaked out of the zone and scaled the
/// `motion.move` that positions the scene, stretching it. Every symptom of a state that was
/// remembering scratch.
///
/// **The zone stores state, not scratch.** What the elements ARE (`P`, `vel`, `id`, `size`,
/// `tint`…) survives; what a tick wrote for its own use does not. Anything downstream that wants
/// a mask computes one — masks are cheap and a stale one is not a mask, it is a ghost.
const TRANSIENTS: [&str; 2] = ["accel", "falloff"];

/// The state as the zone holds it: every column but the transients.
fn store(s: &Stream) -> Stream {
    let mut out = Stream::new(s.count());
    for (name, col) in s.columns() {
        if !TRANSIENTS.contains(&name.as_str()) {
            out.set(name.clone(), col.clone());
        }
    }
    out
}

struct SimZone;

impl NodeOp for SimZone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // **The whole zone is this branch, and it took two wrong answers to find the right one.**
        //
        // The question is *"has the sim started?"*, and it is NOT
        //
        // - *"is my `state` empty?"* — a sim that killed its last element hands back an EMPTY
        //   STREAM, a real answer that carries nothing. Read that as "not started" and the zone
        //   re-seeds from `init`: kill every particle and the scene RESURRECTS, one frame later,
        //   forever.
        // - *"did an edge deliver a value on `state`?"* — it always did. The interior is wired
        //   into `state` by a FORWARD edge, so the cook runs it BEFORE the zone, and on tick 1
        //   it dutifully hands back an empty stream (the absent value was its own input: the
        //   zone's previous output, through the `pre` edge).
        //
        // The state of a zone lives on the zone's OWN previous output. So that is what it asks:
        // did *I* emit anything last tick? (`EvalCtx::started` — doc 48.)
        let held = if ctx.started() {
            ctx.input(IN_STATE)
        } else {
            ctx.input(IN_INIT)
        };
        ctx.emit(store(held));
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimZone))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Simulation Zone",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
