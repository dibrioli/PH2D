//! **Break force — the joint that gives way** (ADR-0131 W-J7).
//!
//! A joint holds two bodies together *no matter what*, and that is exactly the
//! thing an artist eventually does not want: the rope that snaps, the hinge that
//! is torn off, the chain that parts under a load too big for it. rapier does not
//! model breaking — but it publishes the number that decides it, and it has been
//! public all along: [`rapier2d::dynamics::ImpulseJoint::impulses`], *the reaction
//! the constraint applied this step*. (The Godot proposal 7672 asks for exactly
//! this and does not have it; §3 of the plan names it.)
//!
//! ## The threshold is a FORCE, not an impulse — and that is not cosmetic
//!
//! `impulses` is an **impulse** (N·s) over the solver's own small step. A
//! threshold written in those units would silently change meaning every time the
//! world's sub-step count changed: doubling [`super::PhysicsWorld::substeps`]
//! halves every impulse, so a chain tuned to break would quietly stop breaking.
//! That is the timestep-dependence complaint the research collected against Godot
//! (godot-proposals 15126, *"softness/bias without units"*), and the answer this
//! project already gives everywhere else is P5: **every knob carries a unit**.
//!
//! So the ledger converts ([`super::PhysicsWorld::joint_impulse_to_force`]) and
//! reports **newtons** and **newton-metres**. MEASURED — a weight hung from a
//! pin, settled, reads its own weight exactly:
//!
//! | mass | `m·g` | read | ratio |
//! |---|---|---|---|
//! | 0.5 kg | 4.9050 N | 4.9050 | **1.0000** |
//! | 1 kg | 9.8100 | 9.8100 | **1.0000** |
//! | 2 kg | 19.6200 | 19.6200 | **1.0000** |
//! | 5 kg | 49.0500 | 49.0500 | **1.0000** |
//! | 10 kg | 98.1000 | 98.1000 | **1.0000** |
//!
//! and it does not move when either divisor does — 9.8100 N at 1, 2, 4, 8 and 16
//! sub-steps, and 9.8100 N at 1, 2, 4, 8 and 16 solver iterations. There is a
//! gate that turns both knobs and asserts exactly that.
//!
//! ## ⚠️ It reads the load a joint is HOLDING, not the spike of an impact
//!
//! MEASURED, and this is the honest boundary of the feature. A 1 kg load on a 3 m
//! rope, free-falling before the rope catches it:
//!
//! | fall | speed at catch | peak reported | overshoot |
//! |---|---|---|---|
//! | 0.10 m | 1.40 m/s | 9.8 N | none (3.0000) |
//! | 0.50 m | 3.13 m/s | 23.4 N | none |
//! | 2.00 m | 6.26 m/s | **9.8 N** | none |
//!
//! The rope stops the load dead (the separation never exceeds 3.0000), so the
//! impulse is genuinely enormous — and it is **invisible from out here**, because
//! rapier's writeback reports the impulse of the *last* internal solver iteration
//! of each pipeline step, and a catch is resolved inside one. Raising the
//! sub-step count splits the catch across steps we can see, and the spike appears:
//! the same 2 m fall reads **9.8 N at 4 and 8 sub-steps, 11314 N at 16, 24584 at
//! 32, 37485 at 64**.
//!
//! This is the same distinction [`super::contacts`] draws between `impulse` (the
//! load a pair carries) and `impact` (the peak of the hit) — but there the cure
//! worked, because a contact manifold survives the sub-step and can be sampled.
//! Here it cannot be, short of patching rapier.
//!
//! **So a break threshold is a LOAD threshold**: the chain that cannot hold the
//! weight, the hinge torn off by a heavy door, the rope that parts because you
//! hung one crate too many on it. That is also what break force means in practice
//! in Unity, and it is what the smoke scene demonstrates. A "snaps on impact"
//! threshold would need a solver-internal peak, and it is named as not built
//! rather than shipped as a knob that fires at random.
//!
//! ## Where the threshold lives: the joint's own `user_data`
//!
//! Not a side table. `GenericJoint::user_data` is a `u128` rapier provides for
//! precisely this, and the deciding property is that it rides **inside**
//! `ImpulseJointSet` — which the checkpoint ring already deep-clones ([`super::checkpoint`]).
//! A side map would have to be checkpointed in parallel, and the failure mode of
//! forgetting is a joint that scrubs back to being unbreakable. The one-way
//! platform bit ([`super::oneway`]) takes the same route through a collider's
//! `user_data`, for the same reason.
//!
//! **A zero slot means ∞ (off)** — which is also what a joint that was never told
//! a threshold carries, since `GenericJoint::default().user_data == 0`. *Not
//! authored* and *off* are therefore the same state, exactly as "∞ = off" (P7)
//! says they should be: the representation deletes the special case rather than
//! carrying a second "is it set?" bit that could disagree with the value.
//!
//! ## Breaking is a SIMULATION event, not a readout
//!
//! Unlike [`super::contacts`]'s peaks, this one writes back: a broken joint is
//! **disabled** (`JointEnabled::Disabled`) inside the sub-step loop, so the rest
//! of the tick already runs without it. That is the only correct place — a break
//! noticed a frame later is a different simulation, and it would make the
//! deterministic replay disagree with the live run.
//!
//! It follows that a break *does* move the C9 hash, and it should: the world
//! genuinely diverges. A world where no joint carries a finite threshold is
//! byte-identical (nothing is compared, nothing is disabled), which is what keeps
//! every scene that predates this wave exactly where it was.
//!
//! ## The action is DISABLE, never delete
//!
//! The joint entity survives, its parameters survive, the artist's authoring
//! survives; what stops is the constraint. Deleting would make a break destroy
//! work (and would need an undo step for something the solver did, not the
//! artist). A Reset — which rebuilds the world from the descriptors — brings it
//! back, because the break was never authored state.

use std::collections::BTreeMap;

use rapier2d::dynamics::{ImpulseJoint, ImpulseJointHandle, ImpulseJointSet, RigidBodySet};

/// The ledger's key: the joint handle's raw parts, so the map is `Ord`-keyed and
/// the readout is reproducible cross-OS (the rule the whole module follows).
pub type JointKey = (u32, u32);

/// What a joint is carrying, in the two grandezas a break threshold is written
/// in — **newtons** and **newton-metres**, never impulses.
///
/// The linear number is the magnitude of the reaction force: a rope holding a
/// 1 kg load reads its weight, exactly.
///
/// ## ⚠️ The TORQUE is only reported where the angular axis is limited or
/// motorised — a LOCKED one reads zero while plainly resisting
///
/// MEASURED, with a 1 kg, 1 m plank held out horizontally (`m·g·r = 4.905 N·m`
/// in every row):
///
/// | rig | force | torque |
/// |---|---|---|
/// | Weld to a static wall | 9.810 N | **0.0000** |
/// | Weld cantilever, 5 kg | 49.050 N | **0.0000** |
/// | **Pin at its LIMIT** | 2.471 N | **4.9050** ✓ |
/// | **Servo holding the arm out** | 9.812 N | **4.9049** ✓ |
///
/// A Weld locks `AngX`, and rapier never populates `impulses[2]` for a locked
/// angular axis in 2D (the plank is held — `rot = 0.0000` — and the slot stays
/// exactly zero). A limit writes `limits[2].impulse` and a motor writes
/// `motors[2].impulse`, and both are exact.
///
/// **So in this kind set the torque is observable on a Pin and nowhere else** —
/// a Slider locks its angular axis (invisible), a Rope and a Spring leave it free
/// (genuinely zero). The §12 row is offered on the Pin for exactly that reason:
/// a Break Torque on a Weld would be a control that can never fire.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointLoad {
    /// Linear reaction, newtons.
    pub force: f32,
    /// Angular reaction, newton-metres. Always non-negative — a break threshold
    /// asks *how hard*, not which way.
    pub torque: f32,
}

impl JointLoad {
    /// A joint carrying nothing.
    pub const ZERO: Self = Self {
        force: 0.0,
        torque: 0.0,
    };

    /// The larger of two loads, per component — the `max` that makes the peak a
    /// peak. Component-wise and not "whichever is bigger overall", because the
    /// two thresholds are separate (Unity ships them separate for the same
    /// reason): a joint can be torn apart without ever being twisted.
    #[must_use]
    fn joined(self, other: Self) -> Self {
        Self {
            force: self.force.max(other.force),
            torque: self.torque.max(other.torque),
        }
    }
}

/// One joint that gave way during the last [`super::PhysicsWorld::step`].
///
/// An **event**, with its own channel, for the reason W-TickContacts established:
/// a transition is not derivable from the state that follows it. The load at the
/// instant of breaking is here and nowhere else — a moment later the joint is
/// carrying nothing, because it is no longer holding.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointBreak {
    /// Which joint. The bridge maps this back to its entity.
    pub joint: ImpulseJointHandle,
    /// Where it broke, in world metres: the midpoint of the two anchors.
    ///
    /// The midpoint and not one anchor, because the two are the same point for a
    /// Pin and a Weld (that is what those are) and the two ends of the span for a
    /// Spring, a Rope and a Slider — so the midpoint is on the segment the
    /// overlay draws in every case, which is where the artist is looking.
    pub point: [f32; 2],
    /// The reaction force at the instant it gave, newtons.
    pub force: f32,
    /// The reaction torque at that same instant, newton-metres.
    pub torque: f32,
}

/// A threshold as it rides in `user_data`: the `f32`'s bits, with **0 meaning
/// ∞** (see the module doc). A non-finite or non-positive threshold is *off*,
/// which is the same statement.
fn to_slot(v: f32) -> u32 {
    if v.is_finite() && v > 0.0 {
        v.to_bits()
    } else {
        0
    }
}

/// The inverse of [`to_slot`].
fn from_slot(bits: u32) -> f32 {
    if bits == 0 {
        f32::INFINITY
    } else {
        f32::from_bits(bits)
    }
}

/// Pack the two thresholds into the `user_data` word a joint carries.
pub(super) fn pack_thresholds(force: f32, torque: f32) -> u128 {
    (u128::from(to_slot(force)) << 32) | u128::from(to_slot(torque))
}

/// Read them back. `(∞, ∞)` for a joint that never carried any — including
/// every joint spawned before this wave existed.
#[must_use]
pub(super) fn unpack_thresholds(data: u128) -> (f32, f32) {
    (
        from_slot(((data >> 32) & 0xFFFF_FFFF) as u32),
        from_slot((data & 0xFFFF_FFFF) as u32),
    )
}

/// The reaction a joint is applying, converted from rapier's impulses.
///
/// ## `impulses` alone is NOT the reaction, and measuring said so
///
/// The obvious reading — `ImpulseJoint::impulses` — carries only the **locked**
/// degrees of freedom, because that is the only `WritebackId::Dof` the solver
/// writes there. Measured on a 1 kg weight hung 1 m down:
///
/// | kind | `impulses` alone | what it should read |
/// |---|---|---|
/// | Pin | 9.81 N | 9.81 N ✓ |
/// | Weld | 9.81 N | 9.81 N ✓ |
/// | **Rope** | **0.00 N** | 9.81 N |
/// | **Spring** | **0.00 N** | ≈9.81 N |
///
/// A rope's tension lives in `data.limits[i].impulse` (`WritebackId::Limit`) and a
/// spring's force in `data.motors[i].impulse` (`WritebackId::Motor`) — rapier
/// models a rope as a *limit* and a spring as a *motor*, so neither ever touches
/// `impulses`. Reading only `impulses` would have shipped a break force that
/// **can never fire on the two kinds that most obviously want it**, with every
/// Pin gate green.
///
/// So all three sources are summed per axis. In 2D the array index IS the axis
/// (`LinX = 0`, `LinY = 1`, `AngX = 2`), and for every kind this engine ships at
/// most one source is live on a given axis — so the sum is the magnitude, not an
/// over-estimate.
///
/// ## The motor's own impulse counts, and that is deliberate
///
/// A servo straining against a load is stressing the joint exactly as an external
/// pull would; Unity's Break Torque breaks a motored hinge for the same reason.
/// And it is not optional: rapier models a Spring **as** a motor, so excluding it
/// would make a spring unbreakable.
///
/// ⚠️ **`(x*x + y*y).sqrt()`, never `hypot`.** `sqrt` is specified exactly by
/// IEEE-754 and reproduces bit-for-bit on every platform; `hypot` is the
/// platform's libm and does not (the rule W-AreaFalloff wrote down). This number
/// decides whether a joint breaks, and a break moves poses — so it reaches the
/// C9 determinism hash.
#[must_use]
pub(super) fn load_of(j: &ImpulseJoint, inv_dt: f32) -> JointLoad {
    let i = j.impulses;
    let d = &j.data;
    let lin_x = i.x.abs() + d.limits[0].impulse.abs() + d.motors[0].impulse.abs();
    let lin_y = i.y.abs() + d.limits[1].impulse.abs() + d.motors[1].impulse.abs();
    let ang = i.z.abs() + d.limits[2].impulse.abs() + d.motors[2].impulse.abs();
    JointLoad {
        force: (lin_x * lin_x + lin_y * lin_y).sqrt() * inv_dt,
        torque: ang * inv_dt,
    }
}

/// The world point a break is reported at — the midpoint of the two anchors.
fn midpoint(j: &ImpulseJoint, bodies: &RigidBodySet) -> Option<[f32; 2]> {
    let a = bodies.get(j.body1)?;
    let b = bodies.get(j.body2)?;
    let pa = a.position().transform_point(&j.data.local_anchor1());
    let pb = b.position().transform_point(&j.data.local_anchor2());
    Some([(pa.x + pb.x) * 0.5, (pa.y + pb.y) * 0.5])
}

/// **One sub-step's worth of joint bookkeeping**: fold every live joint's load
/// into the peak ledger, and disable the ones that just exceeded what they were
/// told they could take.
///
/// Called from inside [`super::PhysicsWorld::step`]'s sub-step loop, right after
/// the pipeline — the same slot [`super::contacts::accumulate_peaks`] occupies,
/// and for the same reason: the number lives *between* the sub-steps.
///
/// Two passes, and the second is empty in every tick where nothing breaks. The
/// read pass needs the bodies (for the anchor point) while the write pass needs
/// the joint set mutably; splitting them also lets the disable go through
/// `get_mut(h, true)`, which is what queues the two bodies to be woken — a body
/// that was being held up by the joint has to fall the instant it is not.
pub(super) fn accumulate_and_break(
    joints: &mut ImpulseJointSet,
    bodies: &RigidBodySet,
    inv_dt: f32,
    peaks: &mut BTreeMap<JointKey, JointLoad>,
    breaks: &mut Vec<JointBreak>,
) {
    let mut gave_way: Option<Vec<ImpulseJointHandle>> = None;
    for (handle, j) in joints.iter() {
        if !j.data.is_enabled() {
            continue;
        }
        let load = load_of(j, inv_dt);
        let key = handle.0.into_raw_parts();
        let slot = peaks.entry(key).or_insert(JointLoad::ZERO);
        *slot = slot.joined(load);

        let (max_force, max_torque) = unpack_thresholds(j.data.user_data);
        if (load.force > max_force || load.torque > max_torque)
            && let Some(point) = midpoint(j, bodies)
        {
            breaks.push(JointBreak {
                joint: handle,
                point,
                force: load.force,
                torque: load.torque,
            });
            gave_way.get_or_insert_with(Vec::new).push(handle);
        }
    }
    for handle in gave_way.into_iter().flatten() {
        if let Some(j) = joints.get_mut(handle, true) {
            j.data.set_enabled(false);
        }
    }
}

impl super::PhysicsWorld {
    /// **What one of rapier's joint impulses is worth in newtons** — the
    /// reciprocal of the step the solver actually charged it over.
    ///
    /// ⚠️ **That step is not `substep_dt`, and the first version was wrong by
    /// exactly 4×.** rapier's island solver splits each sub-step again, into
    /// `num_solver_iterations` small steps (`params.dt /= num_solver_iterations`),
    /// and the writeback reports the impulse of one of those. Dividing by
    /// `substep_dt` made a hanging 1 kg weight read **2.4525 N** against the
    /// 9.8100 it carries — a ratio of exactly 0.2500 at every mass, which is what
    /// named the missing factor.
    ///
    /// Both divisors are knobs the artist can turn (`set_substeps`,
    /// `set_solver_iterations`), which is the whole reason this is a function and
    /// not a constant: a threshold written in newtons has to stay put when they
    /// move, and there is a gate that turns both and asserts it does.
    ///
    /// Limitation, named: rapier lets an *island* ask for extra iterations
    /// (`RigidBody::additional_solver_iterations`). Nothing in this wrapper sets
    /// one, so the count is global; a body that did would read low here.
    pub(super) fn joint_impulse_to_force(&self) -> f32 {
        self.integration_parameters.num_solver_iterations.get() as f32
            / self.integration_parameters.dt
    }

    /// **The peak reaction this joint carried over the last tick's sub-steps** —
    /// newtons and newton-metres — or `None` for a handle the world does not know.
    ///
    /// The peak and not the endpoint, for the reason
    /// [`super::contacts::ContactReport::impact`] spells out at length: the load
    /// that decides a break happens *between* the sub-steps and is already
    /// resolved by the time `step` returns.
    ///
    /// A joint that has not been stepped yet reads [`JointLoad::ZERO`] rather than
    /// `None`: it exists and is carrying nothing, which is a different answer from
    /// "there is no such joint".
    #[must_use]
    pub fn joint_load(&self, handle: ImpulseJointHandle) -> Option<JointLoad> {
        self.impulse_joints.get(handle)?;
        Some(
            self.joint_peaks
                .get(&handle.0.into_raw_parts())
                .copied()
                .unwrap_or(JointLoad::ZERO),
        )
    }

    /// Every joint that gave way during the last [`Self::step`]. Empty in almost
    /// every tick, and that is the point — this is a transition channel.
    #[must_use]
    pub fn joint_breaks(&self) -> &[JointBreak] {
        &self.joint_breaks
    }

    /// Is this joint still holding? `false` once it has broken.
    ///
    /// A separate question from *"does this joint exist"* — a broken joint is still
    /// in the set, still owns its anchors and parameters, and comes back the moment
    /// the world is rebuilt from the descriptors. Breaking never destroys anything.
    #[must_use]
    pub fn joint_is_enabled(&self, handle: ImpulseJointHandle) -> Option<bool> {
        Some(self.impulse_joints.get(handle)?.data.is_enabled())
    }

    /// The thresholds a joint is carrying, `(force, torque)` — `(∞, ∞)` for an
    /// unbreakable one. Reads them back out of the joint itself, so a test cannot
    /// be fooled by a `JointDesc` that never reached the solver.
    #[must_use]
    pub fn joint_break_thresholds(&self, handle: ImpulseJointHandle) -> Option<(f32, f32)> {
        Some(unpack_thresholds(
            self.impulse_joints.get(handle)?.data.user_data,
        ))
    }
}
