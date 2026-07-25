//! The joint half of the bridge: [`PhysicsJoint`] entities → rapier joints.
//!
//! # Where the anchors come from — one sentence, and it is the whole policy
//!
//! **The joint entity's `Transform` is the anchor on body A. Body B's anchor is
//! the same point for a Pin — two bodies sharing a place is *what a pin is* —
//! and body B's own centre for a Spring or a Rope, whose two ends are meant to
//! be apart.**
//!
//! The engine takes both anchors and has no opinion about which points they
//! are; that is deliberate, so this policy lives in exactly one function and
//! the wrapper stays general. Collapsing the pair would make a 2 m rope hang
//! its ball 2.5 m down whenever the authored point was not the ball's centre.
//!
//! # Joints are reconciled AFTER bodies, and a PARAMETER edit re-describes live
//!
//! Reconciled after bodies for one fact: a joint's local anchors are derived
//! from where the bodies *are*, so they have to be derived from where the
//! artist *put* them, and a joint spawned before its bodies would have nothing
//! to bind to.
//!
//! **Re-describing (a parameter or anchor edit rebuilding the rapier joint) is
//! NOT gated on the clock being at rest** — the point of a spring is that you
//! tune it while it bounces (W-JointParams, 2026-07-25). An earlier version
//! gated the whole re-describe on `at_rest`, which meant every parameter edit —
//! stiffness, motor speed, a rope's length — did nothing until a Reset. That
//! gate was written in W3 to stop an ANCHOR being re-derived mid-swing, when a
//! re-derive read the bodies' LIVE poses and would have baked in the swing
//! offset. W-AnchorFollow moved the anchor to authored body-local state seeded
//! from the bodies' **rest** poses (see the seed below — it uses `rest_pose`,
//! never the live pose), so re-describing mid-play no longer touches the
//! anchor's frame and the protection the `at_rest` gate provided moved with it.
//! What remains true: a body MOVE never changes `desc` (the stored anchors are
//! unchanged), so it never re-describes and never churns the checkpoint ring —
//! only a genuine edit does (gate `an_unedited_joint_does_not_churn_the_scrub_cache`).

use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::{ImpulseJointHandle, JointDesc, MotorDesc, PhysicsWorld, RigidBodyHandle};

use super::BodyRef;

/// The pose a body was authored at — the triple `local_anchor_at_pose` wants.
fn rest_pose(b: &BodyRef) -> [f32; 3] {
    [b.rest.x, b.rest.y, b.rest.rotation]
}

use crate::joint::{JointKind, PhysicsJoint};

use super::PhysicsBridge;

/// A live rapier joint owned by the bridge, keyed by its ECS entity.
#[derive(Copy, Clone)]
pub(super) struct JointRef {
    pub(super) handle: ImpulseJointHandle,
    /// What it was spawned with — the joint's counterpart of `BodyRef::rest`,
    /// and for the same two reasons: a rewind rebuilds from it, and comparing
    /// against it is how an edit at tick 0 is noticed.
    pub(super) rest: JointDesc,
    /// The bodies it binds. Kept so that a body respawned underneath the joint
    /// is noticed: a joint holding a handle into an arena that has moved on is
    /// attached to nothing, in silence.
    pub(super) bodies: (RigidBodyHandle, RigidBodyHandle),
    /// And *whose* bodies they are. A rewind hands out fresh handles, so the
    /// only durable way back to them is the entity — the same reason nothing
    /// else in this bridge remembers a handle across a rebuild.
    pub(super) entities: (Entity, Entity),
}

/// The two WORLD points this joint attaches at. **The single place the anchor
/// policy above is expressed** — and it is expressed on world points, because
/// that is the frame the artist authors in.
pub(super) fn anchor_points(
    j: &PhysicsJoint,
    anchor: [f32; 2],
    body_b_centre: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    (
        anchor,
        // A shared-point joint (Pin OR Weld) anchors both bodies at the same
        // world point — two bodies in one place is what a pin (and a weld) is.
        // A two-ended one (Spring/Rope) anchors body B at its own centre so the
        // ends are apart. `is_hinge` was wrong here the moment Weld arrived: a
        // weld shares a point without being a hinge.
        if j.kind.shares_a_point() {
            anchor
        } else {
            body_b_centre
        },
    )
}

/// Translate the component + the already-LOCAL anchors into the plain
/// [`JointDesc`] the wrapper takes.
pub(super) fn joint_desc(j: &PhysicsJoint, local_a: [f32; 2], local_b: [f32; 2]) -> JointDesc {
    JointDesc {
        kind: match j.kind {
            JointKind::Pin => ph2d_physics::JointKind::Pin,
            JointKind::Spring => ph2d_physics::JointKind::Spring,
            JointKind::Rope => ph2d_physics::JointKind::Rope,
            JointKind::Weld => ph2d_physics::JointKind::Weld,
        },
        anchor_a: local_a,
        anchor_b: local_b,
        // A parameter the kind ignores is not *passed* to the solver either:
        // `is_hinge` is asked here exactly as the Inspector asks it to decide
        // which rows to paint, so a limit left over from a previous kind
        // cannot quietly still be in force.
        limits: (j.kind.is_hinge() && j.limits_enabled).then_some([j.limit_min, j.limit_max]),
        motor: (j.kind.is_hinge() && j.motor_enabled).then_some(MotorDesc {
            speed: j.motor_speed,
            max_force: j.motor_max_force,
        }),
        rest_length: j.rest_length,
        stiffness: j.stiffness,
        damping: j.damping,
        max_length: j.max_length,
    }
}

impl PhysicsBridge {
    /// Spawn / remove / re-describe joints to match the entities carrying
    /// [`PhysicsJoint`]. Called from `reconcile_structure`, after the bodies.
    ///
    /// Takes `&mut SimWorld` (unlike its body sibling) because it SEEDS a joint's
    /// body-local anchors on first sight — writing `local_a`/`local_b`/`anchored`
    /// back into the component. After the seed the anchors are authored state and
    /// this only reads them, which is the whole slide fix (a body move no longer
    /// re-derives against the world `Transform`).
    pub(super) fn reconcile_joints(&mut self, sim: &mut SimWorld) {
        let world = sim.world();

        // Nothing to do, and — the part that matters — nothing to allocate or
        // walk in the overwhelmingly common case of a scene without joints.
        let mut q = self.joint_query.take().expect("query built in dispatch");
        if q.iter(world).next().is_none() && self.joints.is_empty() {
            self.joint_query = Some(q);
            return;
        }

        // Name → body entity, for the bodies THIS BRIDGE holds. Built from
        // `self.bodies` rather than from a second world query: a joint may only
        // name something that is actually a body, and this way that is true by
        // construction instead of by a check someone has to remember.
        self.names.clear();
        for &e in self.bodies.keys() {
            if let Some(n) = world.get::<Name>(e) {
                self.names.insert(stable_name_id(n.as_str()), e);
            }
        }

        self.joints_seen.clear();
        self.joints_to_spawn.clear();
        self.joints_to_remove.clear();
        self.joints_to_seed.clear();

        for (e, joint, _local) in q.iter(world) {
            self.joints_seen.push(e);
            // The anchor is a point in the WORLD, and a joint entity can be
            // parented like any other. Its rest-pose siblings (`bb.rest`) are
            // world-space too, so reading this one locally would put the two
            // halves of the same question in different spaces.
            let transform = match super::space::world_transform(world, e, &mut self.chain) {
                Some(t) => t,
                None => continue,
            };
            let transform = &transform;
            // ⚠️ **A joint that cannot be built is only REMOVED if it was ever
            // built.** Pushing it unconditionally makes `joints_to_remove`
            // non-empty on every single frame for as long as the scene holds
            // one dormant joint — and the ring is cleared on a non-empty
            // list, so W1.5's bit-exact scrub would die, silently, forever.
            //
            // Reachable by the most ordinary gestures: delete one of two
            // jointed bodies, or rename one, or start a joint and not pick the
            // second body yet. Measured before the guard: a scrub back to tick
            // 150 replayed **150** steps instead of 0, with a ring of 1.
            //
            // The body half never had this bug (`reconcile_structure` only
            // pushes entities that are in `self.bodies`); this half had
            // diverged from its own sibling.
            let drop_if_live = |me: &mut Self| {
                if me.joints.contains_key(&e) {
                    me.joints_to_remove.push(e);
                }
            };
            if !joint.names_two_bodies() {
                // Half-authored: the artist has a joint object but has not
                // picked both bodies yet. Not an error, and not a joint.
                drop_if_live(self);
                continue;
            }
            let (Some(&ea), Some(&eb)) =
                (self.names.get(&joint.body_a), self.names.get(&joint.body_b))
            else {
                // A named body that is not here — deleted, renamed, or not yet
                // spawned. The joint goes dormant and comes back by itself if
                // the body does (the same healing the timeline's bindings get).
                drop_if_live(self);
                continue;
            };
            let (Some(ba), Some(bb)) = (self.bodies.get(&ea), self.bodies.get(&eb)) else {
                drop_if_live(self);
                continue;
            };
            let handles = (ba.handle, bb.handle);
            // The body-local anchors. Authored state once seeded; the whole
            // slide fix is that a body move reads them UNCHANGED (the anchor
            // follows the body) instead of re-deriving against the world point.
            //
            // ⚠️ **Seed once, from the world `Transform`, against the bodies'
            // REST poses.** A joint arriving with only a world anchor (fresh
            // create, a bare fixture, a re-pick) has `anchored: false`; this is
            // the SAME `anchor_points` + `local_anchor_at_pose` conversion the
            // pre-fix model ran EVERY reconcile — the only change is that now it
            // runs once and the result is persisted. Converting against the rest
            // pose (not the live one) is why the pin does not walk across a
            // Reset — the lesson that cost **1.771 m** of drift before it.
            let (la, lb) = if joint.anchored {
                (joint.local_a, joint.local_b)
            } else {
                // Body B's centre for the spring/rope policy — its **rest**
                // centre, for the same reason the anchor uses the rest pose.
                let centre_b = [bb.rest.x, bb.rest.y];
                let (wa, wb) = anchor_points(
                    joint,
                    [transform.translation.x, transform.translation.y],
                    centre_b,
                );
                let la = PhysicsWorld::local_anchor_at_pose(rest_pose(ba), wa);
                let lb = PhysicsWorld::local_anchor_at_pose(rest_pose(bb), wb);
                // Persist after the query borrow releases (below), and flip
                // `anchored` so the next reconcile — and every body move — reads
                // these instead of re-deriving.
                self.joints_to_seed.push((e, la, lb));
                (la, lb)
            };
            // `clamped()` here and not only in the Inspector: a component is
            // serde and arrives from the project file too, and this is the last
            // door before rapier.
            let desc = joint_desc(&joint.clamped(), la, lb);
            match self.joints.get(&e) {
                None => self.joints_to_spawn.push((e, desc, handles, (ea, eb))),
                // Re-described whenever `desc` changed — a parameter or anchor
                // edit — with NO `at_rest` gate, so a spring tunes while it
                // bounces (module docs; the anchor is safe because it is seeded
                // from the rest pose). `desc` is stable frame-to-frame absent an
                // edit, so a body move never lands here and never churns the
                // ring. Also fires when the bodies it binds have been re-spawned
                // underneath it, or the joint would hold handles into an arena
                // that has moved on.
                Some(j) if j.rest != desc || j.bodies != handles => {
                    self.joints_to_spawn.push((e, desc, handles, (ea, eb)));
                    self.joints_to_remove.push(e);
                }
                Some(_) => {}
            }
        }
        self.joint_query = Some(q);

        // Joints whose entity is gone.
        for &e in self.joints.keys() {
            if !self.joints_seen.contains(&e) {
                self.joints_to_remove.push(e);
            }
        }

        // Any structural change invalidates every cached state, for the reason
        // the body half already documents: a checkpoint indexes rapier's arenas,
        // and restoring one after the arenas moved publishes a stale pose in
        // silence.
        if !self.joints_to_spawn.is_empty() || !self.joints_to_remove.is_empty() {
            self.ring.clear();
        }

        self.joints_to_remove.sort_unstable_by_key(|e| e.to_bits());
        for i in 0..self.joints_to_remove.len() {
            let e = self.joints_to_remove[i];
            if let Some(j) = self.joints.remove(&e) {
                self.world.remove_joint(j.handle);
            }
        }
        self.joints_to_remove.clear();

        // Deterministic spawn order (HR-5), the same sort the bodies get: the
        // solver's joint order is a pure function of the entity set, never of
        // ECS archetype order.
        self.joints_to_spawn
            .sort_unstable_by_key(|(e, _, _, _)| e.to_bits());
        for i in 0..self.joints_to_spawn.len() {
            let (e, desc, bodies, entities) = self.joints_to_spawn[i];
            if let Some(handle) = self.world.spawn_joint(bodies.0, bodies.1, desc) {
                self.joints.insert(
                    e,
                    JointRef {
                        handle,
                        rest: desc,
                        bodies,
                        entities,
                    },
                );
            }
        }
        self.joints_to_spawn.clear();

        // Persist the anchors just seeded. Done after the query borrow so the
        // component write does not alias the read; `anchored = true` so a body
        // move never re-derives them — the seed is once, the follow is forever.
        for i in 0..self.joints_to_seed.len() {
            let (e, la, lb) = self.joints_to_seed[i];
            if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
                j.local_a = la;
                j.local_b = lb;
                j.anchored = true;
            }
        }
        self.joints_to_seed.clear();
    }

    /// Sync each joint's DISPLAY pivot — its `Transform.translation` — to where
    /// its authored body-A anchor now sits (`bodyA_rest · local_a`). This is what
    /// makes the anchor dot and the Inspector Position FOLLOW the body: move a
    /// body and its stored `local_a` is unchanged, so the derived pivot — and the
    /// dot drawn there — moves with it.
    ///
    /// Rest-only (the caller gates on `!playing`): during play the dot is not
    /// shown and the overlay draws the live solver anchors, so writing a stale
    /// display value every frame would be work for no reader.
    ///
    /// The write is idempotent after a seed/re-seat (the pivot equals what the
    /// gesture set), so it never manufactures a diff — a body move and the pivot
    /// that follows it land in the SAME undo step (one gesture).
    ///
    /// ⚠️ Joints are root entities, so `translation` IS the world pivot with no
    /// parent compose (the same assumption `point_gizmo::build_point_view` makes).
    pub(super) fn sync_joint_pivots(&mut self, sim: &mut SimWorld) {
        if self.joints.is_empty() {
            return;
        }
        // Take the scratch out so the collect (which borrows `self.joints` and
        // `self.bodies`) does not clash with pushing into it.
        let mut scratch = std::mem::take(&mut self.joints_to_sync);
        scratch.clear();
        {
            let world = sim.world();
            for (&e, j) in self.joints.iter() {
                let (Some(ba), Some(joint)) =
                    (self.bodies.get(&j.entities.0), world.get::<PhysicsJoint>(e))
                else {
                    continue;
                };
                // Only a seeded joint has a meaningful local anchor; an un-seeded
                // one still carries the world `Transform` the seed will read, so
                // leave it be until reconcile seeds it next dispatch.
                if !joint.anchored {
                    continue;
                }
                let pivot = PhysicsWorld::world_from_local_at_pose(rest_pose(ba), joint.local_a);
                scratch.push((e, pivot));
            }
        }
        for &(e, pivot) in scratch.iter() {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
                // Write only on a real change so an untouched joint never
                // manufactures a diff for the frame-level undo.
                if t.translation.x != pivot[0] || t.translation.y != pivot[1] {
                    t.translation.x = pivot[0];
                    t.translation.y = pivot[1];
                }
            }
        }
        scratch.clear();
        self.joints_to_sync = scratch;
    }

    /// Re-attach every joint after the bodies have been rebuilt from their rest
    /// descriptions (`rebuild_from_rest`, which has just handed out fresh body
    /// handles). The joints have to come back **in the same call**, not on the
    /// next frame: the rewind replays the owed steps immediately, and a replay
    /// without the joints is a different simulation.
    pub(super) fn respawn_joints_from_rest(&mut self) {
        let existing: Vec<(Entity, JointRef)> = self.joints.iter().map(|(&e, &j)| (e, j)).collect();
        self.joints.clear();
        for (e, mut j) in existing {
            // Copy the handles out before touching `self.world` — and read them
            // from `self.bodies`, which the rebuild has already refreshed.
            let a = self.bodies.get(&j.entities.0).map(|r| r.handle);
            let b = self.bodies.get(&j.entities.1).map(|r| r.handle);
            let (Some(a), Some(b)) = (a, b) else {
                continue;
            };
            j.bodies = (a, b);
            if let Some(handle) = self.world.spawn_joint(a, b, j.rest) {
                j.handle = handle;
                self.joints.insert(e, j);
            }
        }
    }
}
