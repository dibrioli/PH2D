//! [`PhysicsCheckpoint`] — a save/restore of everything `step()` mutates.
//!
//! The GGPO save/load/advance shape, same as `ph2d-nodegraph`'s
//! `Cook::checkpoint`/`restore` (M2.N2): the sim state is a pure function of
//! the tick, so scrubbing backwards means *restore an earlier state and
//! replay forward*, never "step in reverse" (rapier cannot, and no physics
//! engine can — contact resolution is not invertible).
//!
//! ## What is state and what is workspace
//!
//! The eight sets below are exactly the fields of [`PhysicsWorld`] that
//! rapier makes `Clone`, and that is not a coincidence: `PhysicsPipeline` —
//! the one field `step()` takes `&mut` that is **not** `Clone` — is scratch
//! (per-step manifold/constraint index buffers + profiling counters), which
//! is why rapier's own snapshot examples serialize the sets and rebuild the
//! pipeline. Config (`gravity`, `integration_parameters`) is deliberately
//! **not** captured: it is authored, not simulated. Changing it invalidates
//! the cache (the ring's `clear`), exactly like a graph edit in Motion.
//!
//! That reasoning is not trusted on its own — `physics_checkpoint.rs`'s
//! bit-exactness gate compares *restore + replay* against *replay from zero*
//! and would go red if a byte of real state lived in the pipeline.

use std::collections::VecDeque;

use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IslandManager, MultibodyJointSet, RigidBodySet,
};
use rapier2d::geometry::{BroadPhaseBvh, ColliderSet, NarrowPhase};

use super::PhysicsWorld;

/// How much bigger a checkpoint is than the bare body+collider structs it
/// contains: rapier's arenas carry a generation and a free list per slot,
/// their `Vec`s hold growth slack, and the broad-phase BVH and narrow-phase
/// contact graph are built over the same population.
///
/// **Calibrated against `measure_checkpoint`** (2026-07-18: 1175 B/body
/// measured at 200 bodies against 432 B of structs = 2.7×), rounded up to 3
/// because a budget must never believe it is using less than it is.
const ACCEL_FACTOR: usize = 3;

/// A complete, restorable snapshot of a [`PhysicsWorld`]'s simulated state.
///
/// Deep-cloned on capture (rapier's sets own their storage), so a checkpoint
/// is independent of the world it came from and of every other checkpoint —
/// there is no aliasing to reason about when one is evicted.
#[derive(Clone)]
pub struct PhysicsCheckpoint {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    step_count: u64,
}

impl PhysicsCheckpoint {
    /// The step counter this checkpoint was taken at (the tick it reproduces).
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Heap footprint estimate, in bytes — the number the ring caps itself
    /// by. Counts the storage that scales with the scene (bodies, colliders,
    /// and the acceleration structures built over them).
    ///
    /// **Why an estimate exists at all:** the ring caps in BYTES, not in a
    /// count of checkpoints — ADR-0117's lesson, paid for by the audio
    /// editor, is that a count is a *multiplier*, not a ceiling (a 5000-body
    /// scene would blow a 30-checkpoint ring past the whole budget while the
    /// count looked reassuring). Capping by bytes needs a cheap per-frame
    /// answer, and dhat is not that.
    ///
    /// **Why it rounds up:** for a budget, over-reporting is safe and
    /// under-reporting is a silent overrun, so [`ACCEL_FACTOR`] is rounded
    /// generously. `measure_checkpoint` pins the estimate against real dhat
    /// bytes, so it cannot quietly drift into fiction (HR-13's amendment —
    /// whoever declares a budget owns a gate that MEASURES).
    pub fn approx_bytes(&self) -> usize {
        use std::mem::size_of;
        let structs = self.bodies.len() * size_of::<rapier2d::dynamics::RigidBody>()
            + self.colliders.len() * size_of::<rapier2d::geometry::Collider>();
        structs * ACCEL_FACTOR + size_of::<Self>()
    }
}

impl PhysicsWorld {
    /// Capture the current simulated state (GGPO *save*).
    pub fn checkpoint(&self) -> PhysicsCheckpoint {
        PhysicsCheckpoint {
            bodies: self.bodies.clone(),
            colliders: self.colliders.clone(),
            impulse_joints: self.impulse_joints.clone(),
            multibody_joints: self.multibody_joints.clone(),
            ccd_solver: self.ccd_solver.clone(),
            island_manager: self.island_manager.clone(),
            broad_phase: self.broad_phase.clone(),
            narrow_phase: self.narrow_phase.clone(),
            step_count: self.step_count,
        }
    }

    /// Put the world back at a captured state (GGPO *load*). Handles stay
    /// valid: the sets are restored wholesale, so every `RigidBodyHandle`
    /// that was live at capture time addresses the same body again.
    ///
    /// Config (gravity, dt) is untouched — see the module docs.
    pub fn restore(&mut self, cp: &PhysicsCheckpoint) {
        self.bodies = cp.bodies.clone();
        self.colliders = cp.colliders.clone();
        self.impulse_joints = cp.impulse_joints.clone();
        self.multibody_joints = cp.multibody_joints.clone();
        self.ccd_solver = cp.ccd_solver.clone();
        self.island_manager = cp.island_manager.clone();
        self.broad_phase = cp.broad_phase.clone();
        self.narrow_phase = cp.narrow_phase.clone();
        self.step_count = cp.step_count;
    }
}

/// A bounded cache of `(tick, checkpoint)` that turns a backwards scrub from
/// `O(target)` into `O(STRIDE)`.
///
/// ## Sparse, and the stride is a MEASURED decision
///
/// `ph2d-eval-motion`'s ring is **dense** — one checkpoint per tick — on the
/// GGRS rule: *go dense unless the state-copy cost dominates `K × re-sim`*.
/// Motion's state is a few small columns and a cook is cheap, so it does.
///
/// The same rule, applied to the same numbers for physics, points the other
/// way. `measure_checkpoint` and its timing sibling say a checkpoint of a
/// 50-body scene is **59 KB and costs about as much as one `step()`**
/// (11.2 µs vs 7.3 µs; at 200 bodies, 40 µs vs 46 µs). So a dense ring would
/// *double the cost of play* — against HR-4's 1.5 ms physics frame — and
/// spend 17.4 MB of HR-13's 20 MB budget on 5 seconds of window.
///
/// At [`STRIDE`] = 10 the same window costs **1.74 MB and ~10% of one step
/// per tick**, and the worst-case scrub replays 10 steps (~0.07 ms — below
/// perception, which is the only thing a scrub owes anyone).
///
/// ## The fallback is the product, not a second implementation
///
/// A miss returns `None`, and the caller's answer to `None` is the path that
/// already existed and was already gated before this ring was written:
/// rebuild from the bodies' rest descriptions and replay. **Delete this ring
/// and the product still scrubs correctly, only slower** — so there is no
/// second copy of the physics to drift out of agreement with the first (the
/// same shape as the audio editor's splice fallback, ADR-0124).
pub struct PhysicsCheckpointRing {
    /// Strictly ascending by tick, newest at the back.
    recent: VecDeque<(u64, PhysicsCheckpoint)>,
    /// Running sum of `approx_bytes` over `recent` — kept incrementally so
    /// the per-frame budget question costs nothing.
    bytes: usize,
    budget_bytes: usize,
}

/// Record one checkpoint every this many ticks. See the type docs — this is
/// a measured trade, not a guess.
pub const STRIDE: u64 = 10;

/// The ring's slice of HR-13's 20 MB physics budget. The live world, its
/// broad phase, and the bridge's own maps all have to fit in the rest, so
/// the cache does not get to claim the whole thing.
///
/// This is a **ceiling, not a target**: at 50 bodies it buys ~135
/// checkpoints (≈22 s of scrub window) and at 200 bodies ~35 (≈6 s). A
/// heavier scene automatically gets a shorter window instead of a bigger
/// bill, which is the entire point of capping in bytes.
pub const DEFAULT_BUDGET_BYTES: usize = 8 * 1024 * 1024;

impl Default for PhysicsCheckpointRing {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsCheckpointRing {
    pub fn new() -> Self {
        Self {
            recent: VecDeque::new(),
            bytes: 0,
            budget_bytes: DEFAULT_BUDGET_BYTES,
        }
    }

    /// Whether a checkpoint at `tick` is worth taking: on the stride, and
    /// strictly newer than what the window already holds. Lets the caller
    /// skip the deep clone entirely — asking is free, capturing is not.
    #[must_use]
    pub fn should_record(&self, tick: u64) -> bool {
        tick > 0
            && tick.is_multiple_of(STRIDE)
            && self.recent.back().is_none_or(|(bt, _)| tick > *bt)
    }

    /// Store the checkpoint reproducing `tick`, evicting the oldest entries
    /// until the window is back inside its byte budget.
    pub fn record(&mut self, tick: u64, cp: PhysicsCheckpoint) {
        if self.recent.back().is_some_and(|(bt, _)| tick <= *bt) {
            return; // already covered — identical by determinism
        }
        self.bytes += cp.approx_bytes();
        self.recent.push_back((tick, cp));
        while self.bytes > self.budget_bytes && self.recent.len() > 1 {
            if let Some((_, old)) = self.recent.pop_front() {
                self.bytes = self.bytes.saturating_sub(old.approx_bytes());
            }
        }
    }

    /// The newest checkpoint at or before `target`, or `None` when the
    /// target predates the window — in which case the caller replays from
    /// the bodies' rest state, which is always available and always correct.
    #[must_use]
    pub fn anchor_at_or_before(&self, target: u64) -> Option<(u64, &PhysicsCheckpoint)> {
        self.recent
            .iter()
            .rev()
            .find(|(t, _)| *t <= target)
            .map(|(t, cp)| (*t, cp))
    }

    /// Seed `world` with the newest checkpoint at or before `target`,
    /// returning the anchor tick it landed on. `None` means the target
    /// predates the window and the caller must rebuild from rest.
    ///
    /// The single door for scrubbing: finding the anchor and installing it
    /// are one operation, so no caller can look one up and restore another.
    pub fn seed(&self, world: &mut PhysicsWorld, target: u64) -> Option<u64> {
        let (tick, cp) = self.anchor_at_or_before(target)?;
        world.restore(cp);
        Some(tick)
    }

    /// Drop everything. **Mandatory on any structural change** — spawning or
    /// removing a body, or editing gravity, makes every stored checkpoint a
    /// snapshot of a *different world*. Restoring one would hand back body
    /// handles that no longer address the entities the bridge is holding,
    /// and the wrong pose would be published in silence.
    pub fn clear(&mut self) {
        self.recent.clear();
        self.bytes = 0;
    }

    /// Live heap estimate of the whole window (the number the budget gate
    /// measures).
    #[must_use]
    pub fn approx_bytes(&self) -> usize {
        self.bytes
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.recent.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.recent.is_empty()
    }
}
