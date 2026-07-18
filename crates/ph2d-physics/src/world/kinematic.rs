//! **Kinematic bodies** — the ones whose pose is an INPUT to the solver.
//!
//! A dynamic body's pose is decided by the solver; a static body's is decided
//! by nobody. A kinematic body's is decided by the SCENE — a timeline curve, a
//! gizmo drag, a baked simulation — and the solver is told, which is what makes
//! it push what it touches instead of ghosting through.
//!
//! Its own module because the aim is not a one-line setter: it is a target for
//! the end of the **tick**, and [`PhysicsWorld::step`] has to spread it across
//! the sub-steps. That rule, and the shortest-arc interpolation it needs, are
//! one responsibility and they live together here.

use super::PhysicsWorld;
use rapier2d::dynamics::{RigidBodyHandle, RigidBodyType};
use rapier2d::na::{Isometry2, Vector2};

impl PhysicsWorld {
    /// Aim a **kinematic** body at `(x, y, rotation)`: the next [`step`] moves
    /// it there and rapier derives the velocity it needed to arrive.
    ///
    /// [`step`]: PhysicsWorld::step
    ///
    /// This is what separates a kinematic body from a teleported one, and the
    /// difference is the whole reason the kind exists. [`set_body_pose`] plants
    /// the body at a place with **zero velocity**, so a dynamic body resting
    /// against it is discovered already overlapping and gets squirted out by
    /// the penetration solver. Aiming instead gives the contact a velocity to
    /// read, so the kinematic body *pushes* — an animated platform carries what
    /// stands on it, and a baked ball knocks over the boxes it passes through.
    ///
    /// [`set_body_pose`]: PhysicsWorld::set_body_pose
    ///
    /// No-op for a body of any other type — and the refusal is about **cost,
    /// not correctness**, which was measured rather than assumed: rapier
    /// ignores a next-position on a dynamic body entirely (a ball aimed at
    /// `(9, 9)` every tick for a second lands on the floor at the same
    /// `y = 0.349967` as one never aimed at all). What the guard buys is that
    /// a body which can never use an aim never joins the per-sub-step loop
    /// that replays it. Saying it "would corrupt the body" would be a comment
    /// that lies; see `the_aim_of_a_body_that_cannot_use_it_is_refused`.
    ///
    /// ⚠️ **The target is for the end of the TICK, and [`step`] slices it
    /// across the sub-steps** — it is not handed to rapier here. Handing it
    /// over directly is wrong by exactly the sub-step count, and wrong in a way
    /// that reads as "friction is broken": the first of the four sub-steps
    /// consumes the whole tick's displacement, so the body arrives at 4× the
    /// real speed and then stands still for three sub-steps, giving a contact
    /// almost nothing to integrate. Measured on a platform moving 1.0 m under a
    /// box: **the box travelled 0.009 m instead of riding along.** It is the
    /// same law `drag::apply` states one line above the sub-step loop — *per
    /// sub-step, not per tick*.
    ///
    /// [`step`]: PhysicsWorld::step
    pub fn set_next_kinematic_pose(
        &mut self,
        handle: RigidBodyHandle,
        x: f32,
        y: f32,
        rotation: f32,
    ) {
        let Some(b) = self.bodies.get(handle) else {
            return;
        };
        if b.body_type() != RigidBodyType::KinematicPositionBased {
            return;
        }
        let start = *b.position();
        let target = Isometry2::new(Vector2::new(x, y), rotation);
        self.kinematic_targets.push((handle, start, target));
    }

    /// The pose a kinematic body should hold `f` of the way through its tick
    /// (`f` in `(0, 1]`), given where it started and where it must arrive.
    ///
    /// The angle takes the **shortest arc**, the same law
    /// `ph2d_anim::unwrap_angles` applies to a recorded rotation: without it a
    /// body whose authored rotation crosses ±π would be told to unwind almost a
    /// full turn inside one tick, and would fling whatever it touched. Constants
    /// only — no transcendental in the interpolation itself (HR-5); building the
    /// `Isometry2` costs the same `sincos` that spawning and posing a body
    /// already cost, so this adds no new convention.
    pub(super) fn kinematic_slice(
        start: &Isometry2<f32>,
        target: &Isometry2<f32>,
        f: f32,
    ) -> Isometry2<f32> {
        // The LAST slice is the target itself, not an arithmetic path to it.
        // `a0 + (a1 - a0)` is not always exactly `a1` in f32, and the body's
        // pose is read straight back out into the scene's `Transform` — so a
        // last-slice ulp would be a pose the artist did not author, arriving
        // every tick, in the same direction. Exact by construction costs one
        // comparison and removes the whole class.
        if f >= 1.0 {
            return *target;
        }
        let (a0, a1) = (start.rotation.angle(), target.rotation.angle());
        let mut d = a1 - a0;
        while d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        while d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        let t =
            start.translation.vector + (target.translation.vector - start.translation.vector) * f;
        Isometry2::new(t, a0 + d * f)
    }
}
