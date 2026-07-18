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
//! # Joints are reconciled AFTER bodies, and re-described only at rest
//!
//! Both follow from the same fact: a joint's local anchors are derived from
//! where the bodies *are*, so they have to be derived from where the artist
//! *put* them. A joint spawned before its bodies would have nothing to bind
//! to; one re-derived mid-simulation would bake in whatever offset the swing
//! happened to have at that instant. The body half of the bridge already has
//! this rule (`at_rest && b.rest != desc`), and joints ride it rather than
//! inventing a second one.

use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::{ImpulseJointHandle, JointDesc, MotorDesc, RigidBodyHandle};

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

/// Translate the component + the poses into the plain [`JointDesc`] the
/// wrapper takes. **The single place the anchor policy above is expressed.**
pub(super) fn joint_desc(j: &PhysicsJoint, anchor: [f32; 2], body_b_centre: [f32; 2]) -> JointDesc {
    JointDesc {
        kind: match j.kind {
            JointKind::Pin => ph2d_physics::JointKind::Pin,
            JointKind::Spring => ph2d_physics::JointKind::Spring,
            JointKind::Rope => ph2d_physics::JointKind::Rope,
        },
        anchor_a: anchor,
        anchor_b: if j.kind.is_hinge() {
            anchor
        } else {
            body_b_centre
        },
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
    pub(super) fn reconcile_joints(&mut self, sim: &SimWorld) {
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
        let at_rest = self.last_stepped == 0;

        for (e, joint, transform) in q.iter(world) {
            self.joints_seen.push(e);
            if !joint.names_two_bodies() {
                // Half-authored: the artist has a joint object but has not
                // picked both bodies yet. Not an error, and not a joint.
                self.joints_to_remove.push(e);
                continue;
            }
            let (Some(&ea), Some(&eb)) =
                (self.names.get(&joint.body_a), self.names.get(&joint.body_b))
            else {
                // A named body that is not here — deleted, renamed, or not yet
                // spawned. The joint goes dormant and comes back by itself if
                // the body does (the same healing the timeline's bindings get).
                self.joints_to_remove.push(e);
                continue;
            };
            let (Some(ba), Some(bb)) = (self.bodies.get(&ea), self.bodies.get(&eb)) else {
                self.joints_to_remove.push(e);
                continue;
            };
            let centre_b = world
                .get::<Transform>(eb)
                .map(|t| [t.translation.x, t.translation.y])
                .unwrap_or_default();
            let desc = joint_desc(
                joint,
                [transform.translation.x, transform.translation.y],
                centre_b,
            );
            let handles = (ba.handle, bb.handle);
            match self.joints.get(&e) {
                None => self.joints_to_spawn.push((e, desc, handles, (ea, eb))),
                // Re-described at rest, exactly as bodies are — and also when
                // the bodies it binds have been re-spawned underneath it, or
                // the joint would hold handles into an arena that has moved on.
                Some(j) if (at_rest && j.rest != desc) || j.bodies != handles => {
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
