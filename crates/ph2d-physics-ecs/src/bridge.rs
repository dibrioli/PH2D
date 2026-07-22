//! [`PhysicsBridge`] — the per-frame seam between the ECS and rapier.
//!
//! Owns the transient [`PhysicsWorld`] + the `Entity -> body handle` map.
//! **Not serialized** (ADR-0131 D2): the live world is *derived* from the
//! `RigidBody`/`Collider` components each frame, exactly like Motion
//! re-cooks from its graph. The components are the truth-at-rest; the
//! world is the truth-in-motion; the pose flows into `Transform`.
//!
//! The shell owns one of these on `AppGfx.physics` and calls [`dispatch`]
//! once per frame (mirroring `motion_bridge`). It is a plain struct with
//! free-standing methods so the whole thing is drivable **headless** — the
//! `SimWorld` needs no window, so the W1 e2e gate is a unit test.
//!
//! [`dispatch`]: PhysicsBridge::dispatch

pub mod contacts;
mod damping;
mod diagnostics;
mod hold;
mod joints;
mod kinematic;
mod space;
mod triggers;

pub use kinematic::{FrozenScene, SceneAtTick};

use std::collections::BTreeMap;

use bevy_ecs::query::QueryState;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics::{BodyDesc, PhysicsCheckpointRing, PhysicsWorld, RigidBodyHandle};

use crate::joint::PhysicsJoint;
use joints::JointRef;

use crate::settings::PhysicsSettings;

use crate::components::{BodyKind, Collider, RigidBody};

/// The query the bridge iterates each frame. Cached (built once) because
/// `World::query` allocates a fresh `QueryState` per call — the cached
/// state is the zero-alloc idiom (HR-3), mirroring `TransformPropagationState`.
type BodyQuery = QueryState<(
    Entity,
    &'static RigidBody,
    &'static Collider,
    &'static Transform,
)>;

/// A joint waiting to be handed to the solver: which entity authored it, what
/// it is, the two body handles, and the two body ENTITIES. The entities travel
/// alongside the handles because a rewind hands out fresh handles and the only
/// durable way back to a body is its entity.
type PendingJoint = (
    Entity,
    ph2d_physics::JointDesc,
    (RigidBodyHandle, RigidBodyHandle),
    (Entity, Entity),
);

/// The joint query, cached for the same zero-alloc reason as [`BodyQuery`].
/// The `Transform` is the anchor — see `bridge::joints` for the policy.
type JointQuery = QueryState<(Entity, &'static PhysicsJoint, &'static Transform)>;

/// A live rapier body owned by the bridge, keyed by its ECS entity.
#[derive(Copy, Clone)]
pub(super) struct BodyRef {
    pub(super) handle: RigidBodyHandle,
    /// Cached so `readback` knows which bodies to read (only dynamic ones
    /// move; static bodies keep their authored pose).
    kind: BodyKind,
    /// The spawn description captured when this body was FIRST created —
    /// i.e. the **rest state at tick 0**. rapier cannot rewind, so a
    /// backwards clock replays from here (see
    /// [`PhysicsBridge::rewind_to`]). Without it, Reset could not put the
    /// ball back where it started: the live `Transform` has already been
    /// overwritten by the readback.
    pub(super) rest: BodyDesc,
}

/// The ECS ↔ rapier bridge. One per document, held on `AppGfx.physics`.
pub struct PhysicsBridge {
    world: PhysicsWorld,
    /// `BTreeMap`, not `HashMap`, on purpose: iteration is by `Entity`
    /// order — deterministic per run and cross-OS (entity allocation is
    /// sequential) — so the sim's body order and the hash never depend on a
    /// randomised hasher seed (HR-5). The repo's disallowed-`HashMap` lint
    /// is the standing structural guard for this.
    bodies: BTreeMap<Entity, BodyRef>,
    /// Last fixed tick the world was stepped **to** — the physics analog
    /// of the Motion pump's `last_cooked_tick`. The play/scrub decision in
    /// [`dispatch`](PhysicsBridge::dispatch) reads it.
    last_stepped: u64,
    /// Built lazily on first dispatch (needs `&mut World`); `None` after
    /// [`rebuild`](PhysicsBridge::rebuild) so it re-binds to the world.
    query: Option<BodyQuery>,
    /// The joints this bridge holds, keyed by the entity that authors each one.
    /// `BTreeMap` for the determinism reason `bodies` documents.
    joints: BTreeMap<Entity, JointRef>,
    joint_query: Option<JointQuery>,
    // Reusable scratch — cleared+refilled each frame so the steady-state
    // hot path never reallocates (HR-3; proven by the capacity gate).
    /// Entities carrying physics components this frame (for stale detection).
    seen: Vec<Entity>,
    /// Name-hash → body entity, rebuilt each frame a joint needs resolving.
    names: BTreeMap<u64, Entity>,
    joints_seen: Vec<Entity>,
    joints_to_spawn: Vec<PendingJoint>,
    joints_to_remove: Vec<Entity>,
    /// Where each kinematic body stood when this dispatch began, so a
    /// multi-tick dispatch can spread its move across the ticks it owes
    /// instead of teleporting it on the first one. Scratch: cleared and
    /// refilled per dispatch, so the steady state never reallocates.
    kin_start: Vec<(Entity, [f32; 3])>,
    /// Ancestor chain buffer for `space::world_transform` / `write_world_pose`.
    /// Persistent so the per-body, per-frame conversion allocates nothing
    /// (HR-3; `hot_path_no_alloc` is the gate).
    pub(super) chain: Vec<Transform>,
    /// New entities to spawn a body for, this frame.
    to_spawn: Vec<(Entity, BodyDesc, BodyKind)>,
    /// Entities whose body must be removed this frame (component gone).
    to_remove: Vec<Entity>,
    /// The world's authored settings, kept so a rewind can rebuild a fresh
    /// world that still has them: `PhysicsWorld::new` starts from the engine
    /// defaults, so anything not carried here is **silently reset** by a scrub
    /// backwards. (This field used to be gravity alone, which is exactly that
    /// bug scoped to nine other knobs.)
    settings: PhysicsSettings,
    /// Sparse cache of past states, so scrubbing backwards replays at most
    /// `STRIDE` steps instead of `target` of them (W1.5). Purely an
    /// accelerator: on a miss the rest-pose rebuild below runs, which is the
    /// path that shipped in W1.
    ring: PhysicsCheckpointRing,
    /// Every `world.step()` this bridge has ever run. The scrub gate is a
    /// COUNT, not a stopwatch: the claim being defended is "a scrub replays
    /// at most `STRIDE` steps regardless of how far in it lands", and steps
    /// are exactly that quantity — deterministic, and immune to the machine
    /// the test happens to run on.
    steps_taken: u64,
    /// **Trigger state** (W7): each sensor entity → the entities inside it,
    /// rebuilt every dispatch (`bridge::triggers`). Empty without sensors, so a
    /// non-trigger scene pays nothing. `BTreeMap` for the determinism reason
    /// `bodies` documents.
    triggers: BTreeMap<Entity, Vec<Entity>>,
    /// The pairs touching this frame (`bridge::contacts`). A flat list, not a map:
    /// a contact is a RELATIONSHIP with no owner, unlike a trigger, which is asked
    /// about one sensor. Cleared and refilled per dispatch; empty in free fall.
    contacts: Vec<contacts::BodyContact>,
}

impl Default for PhysicsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsBridge {
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            bodies: BTreeMap::new(),
            last_stepped: 0,
            query: None,
            joints: BTreeMap::new(),
            joint_query: None,
            seen: Vec::new(),
            names: BTreeMap::new(),
            joints_seen: Vec::new(),
            joints_to_spawn: Vec::new(),
            joints_to_remove: Vec::new(),
            kin_start: Vec::new(),
            chain: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
            settings: PhysicsSettings::default(),
            ring: PhysicsCheckpointRing::new(),
            steps_taken: 0,
            triggers: BTreeMap::new(),
            contacts: Vec::new(),
        }
    }

    /// Total `step()` calls since this bridge was created — the ruler the
    /// scrub gate reads (see [`PhysicsBridge::steps_taken`]'s field docs).
    #[doc(hidden)]
    pub fn steps_taken(&self) -> u64 {
        self.steps_taken
    }

    /// How many past states the scrub cache is holding, and what they cost
    /// (for the memory gate and diagnostics).
    #[doc(hidden)]
    pub fn ring_stats(&self) -> (usize, usize) {
        (self.ring.len(), self.ring.approx_bytes())
    }

    /// The last fixed tick the world has been stepped to (for the shell's
    /// play/scrub decision, and for tests).
    pub fn last_stepped(&self) -> u64 {
        self.last_stepped
    }

    /// Where the SOLVER has this entity's body, `(x, y, rotation)` — not where
    /// the entity's `Transform` says it is.
    ///
    /// Exists for gates about the drive stage. The two agree for a dynamic
    /// body (the readback copies one into the other), which is exactly why a
    /// test that asks the `Transform` cannot see whether a KINEMATIC body's
    /// aim reached rapier at all: nothing writes that `Transform`, so it holds
    /// whatever the test put there either way.
    #[doc(hidden)]
    #[must_use]
    pub fn body_pose(&self, entity: Entity) -> Option<(f32, f32, f32)> {
        let b = self.bodies.get(&entity)?;
        let pose = self.world.body_pose(b.handle)?;
        Some((
            pose.translation.x,
            pose.translation.y,
            pose.rotation.angle(),
        ))
    }

    /// Number of live rapier bodies (for tests / diagnostics).
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Number of live rapier joints (for tests / diagnostics).
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Both anchors of every live joint, in **world** meters — what the
    /// collider overlay draws. A joint is as invisible as a collider is, and
    /// the answer to that was the same one both times: draw it.
    pub fn joint_anchors(&self) -> impl Iterator<Item = ([f32; 2], [f32; 2])> + '_ {
        self.joints
            .values()
            .filter_map(|j| self.world.joint_anchors(j.handle))
    }

    /// Set world gravity (m/s²). Default is `(0, -9.81)` (Y-up).
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.set_settings(PhysicsSettings {
            gravity_x: x,
            gravity_y: y,
            ..self.settings
        });
    }

    /// The world's authored settings (what the panel paints, and what the
    /// project file stores).
    pub fn settings(&self) -> PhysicsSettings {
        self.settings
    }

    /// Replace the world's authored settings and push them into rapier.
    ///
    /// Clamps on the way in: a range that only lives in a slider is not a
    /// range, and this is also the door a loaded project file comes through.
    ///
    /// ⚠️ **Clears the checkpoint ring**, for the same reason gravity always
    /// did: every cached state was simulated under the OLD settings, so
    /// replaying from one would splice two different worlds together and
    /// publish the result as if nothing happened. Asked once, for all ten
    /// knobs, instead of per-knob — one door cannot forget one of them.
    pub fn set_settings(&mut self, settings: PhysicsSettings) {
        let settings = settings.clamped();
        if settings == self.settings {
            // Idempotent: the panel republishes every frame, and waking every
            // body (which `set_body_defaults` does) on a frame where nothing
            // changed would keep a settled stack from ever sleeping.
            return;
        }
        self.settings = settings;
        settings.apply_to(&mut self.world);
        self.ring.clear();
    }

    /// Throw away the derived world. Call on project load / undo restore —
    /// entity bits are recycled there, so the handle map dangles. The world
    /// is rebuilt from components on the next [`dispatch`](Self::dispatch)
    /// (runtime-truth: the components are the truth, the world is derived).
    pub fn rebuild(&mut self) {
        self.world = PhysicsWorld::new();
        // A fresh world starts from the ENGINE defaults, not this document's
        // settings — re-push them or a project load quietly reverts every knob.
        self.settings.apply_to(&mut self.world);
        self.bodies.clear();
        self.joints.clear();
        self.last_stepped = 0;
        self.query = None; // re-bind to the (possibly fresh) world
        self.joint_query = None;
        self.ring.clear(); // cached states belong to the document being left
    }

    /// The per-frame entry point (mirrors `motion_bridge::dispatch`).
    /// `playing` / `target` come from the shell's `Playhead`
    /// (`target = round(playhead.time() / fixed_dt)`).
    ///
    /// - **Playing:** step the owed ticks (`target - last_stepped`,
    ///   sequential) then read poses back into `Transform`.
    /// - **Not playing:** settle bodies to the authored `Transform` (no
    ///   step) — the artist is posing. Scrub-*back* re-sim is W1.5.
    pub fn dispatch(&mut self, sim: &mut SimWorld, playing: bool, target: u64) {
        self.dispatch_with_scene(sim, playing, target, &mut FrozenScene);
    }

    /// Bind the queries and bring the rapier world's *structure* level with the
    /// components — the prologue every entry point shares.
    ///
    /// Bodies BEFORE joints, always: a joint binds body handles and derives its
    /// local anchors from where those bodies stand, so it cannot be built before
    /// them (`bridge::joints`).
    fn prepare(&mut self, sim: &mut SimWorld) {
        if self.query.is_none() {
            self.query = Some(sim.world_mut().query());
        }
        if self.joint_query.is_none() {
            self.joint_query = Some(sim.world_mut().query());
        }
        self.reconcile_structure(sim);
        self.reconcile_joints(sim);
        self.restamp_damping();
    }

    /// [`PhysicsBridge::dispatch`], told where the scene's scene-driven bodies
    /// are at each tick it runs (see [`SceneAtTick`]). The plain `dispatch` is
    /// this with nothing to consult; there is ONE implementation so the two
    /// cannot drift.
    pub fn dispatch_with_scene(
        &mut self,
        sim: &mut SimWorld,
        playing: bool,
        target: u64,
        scene: &mut dyn SceneAtTick,
    ) {
        self.prepare(sim);
        match target.cmp(&self.last_stepped) {
            // The clock went BACKWARDS — Reset, or a scrub. rapier has no
            // rewind, so replay from the rest state (see `rewind_to`).
            std::cmp::Ordering::Less => self.rewind_to(sim, target, scene),
            // Forward: advance the owed ticks, sequentially. This is play,
            // and it is equally a scrub FORWARD while paused — the sim state
            // is a function of the tick, not of the play button.
            std::cmp::Ordering::Greater => {
                // A kinematic body's pose is an INPUT, and the scene supplies
                // it once per FRAME while a frame may owe several ticks. So the
                // move is spread across them — the same slicing the wrapper
                // does across sub-steps, one level up, through the same law
                // (`PhysicsWorld::slice_pose`).
                //
                // ⚠️ Neither half of that is optional, and both were wrong once.
                // Aiming once per DISPATCH feeds the first step and leaves the
                // rest with none (an aim is spent by the step it is for), so
                // the body crosses the whole span in one tick at N× speed:
                // measured on a platform carrying a box, stepping to tick 60
                // one tick at a time leaves the cargo riding at x = 1.049,
                // while asking for tick 60 in ONE dispatch FLINGS it to
                // x = -0.520 and off the edge. Aiming at the same TARGET every
                // tick is no better — the aim is absolute, so the body arrives
                // on the first step and then has zero velocity for the rest.
                let owed = target - self.last_stepped;
                self.capture_kinematic_start();
                let mut tick = self.last_stepped;
                for i in 0..owed {
                    // Ask the scene for THIS tick first. When it answers, the
                    // pose is exact and there is nothing to interpolate; when it
                    // does not, spread the frame's move across the ticks owed.
                    let exact = scene.put(sim, self.last_stepped + i + 1);
                    let f = if exact {
                        1.0
                    } else {
                        (i + 1) as f32 / owed as f32
                    };
                    self.drive_kinematic(sim, f);
                    self.world.step();
                    self.steps_taken += 1;
                    tick += 1;
                    // Asking is free; capturing costs about one step, which
                    // is why the ring is sparse (see `PhysicsCheckpointRing`).
                    if self.ring.should_record(tick) {
                        self.ring.record(tick, self.world.checkpoint());
                    }
                }
                self.readback(sim);
                self.last_stepped = target;
            }
            // The clock is standing still. While PAUSED, let bodies follow a
            // Transform the artist moved (authoring). While PLAYING, do
            // nothing: a frame faster than the tick must not touch the world
            // — `settle` would zero the velocity and the fall would stutter.
            std::cmp::Ordering::Equal => {
                if !playing {
                    self.settle(sim);
                }
            }
        }
        // Read the sensor overlaps of the world in its final state for this
        // frame, whichever branch produced it (`bridge::triggers`). No-op (and
        // no alloc) when the scene has no sensors.
        self.rebuild_triggers();
        // And the SOLID half of the same question (`bridge::contacts`): who is
        // actually touching whom, where, and under how much load. Read from the same
        // final world state as the triggers, for the same reason — whichever branch
        // above produced it. No-op (and no alloc) when nothing touches.
        self.rebuild_contacts();
    }

    /// Put the world back at tick 0 and replay forward to `target`.
    ///
    /// rapier cannot step backwards, and the live `Transform` is no help —
    /// the readback has already overwritten it with the simulated pose. So
    /// each body carries the description it was SPAWNED with
    /// ([`BodyRef::rest`]); a fresh world built from those, replayed
    /// `target` steps, reproduces the state exactly (the sim is
    /// deterministic). `target == 0` is the common case — Reset — and costs
    /// no steps at all.
    ///
    /// **W1.5:** the checkpoint ring makes this `O(STRIDE)` instead of
    /// `O(target)`. The ring seeds the world from the newest cached state at
    /// or before `target` and only the remainder is replayed; on a miss (a
    /// target older than the cached window, and Reset, which is `target = 0`)
    /// the world is rebuilt from the bodies' rest descriptions — the path
    /// that shipped in W1, still the only correctness-critical one.
    fn rewind_to(&mut self, sim: &mut SimWorld, target: u64, scene: &mut dyn SceneAtTick) {
        let anchor = self.ring.seed(&mut self.world, target);
        let (from, replayed) = match anchor {
            Some(tick) => (tick, target - tick),
            None => {
                self.rebuild_from_rest();
                (0, target)
            }
        };
        for i in 0..replayed {
            // ⚠️ The replay drives scene-owned bodies too, and skipping this
            // was the defect that made a scrub disagree with a play. A
            // kinematic body frozen at its rest pose for the whole replay is a
            // platform that is not where it was, so every dynamic body that
            // touched it lands somewhere else — measured at 3.4 cm on a partial
            // replay, and at "the box never travelled at all" on a ring miss,
            // which meant the answer also depended on whether the cache
            // happened to hold the anchor.
            //
            // With a scene that answers, this restores the invariant the whole
            // bridge rests on: the world is a function of the tick, given the
            // authored rest state AND the authored curves — both reproducible.
            scene.put(sim, from + i + 1);
            self.drive_kinematic(sim, 1.0);
            self.world.step();
            self.steps_taken += 1;
        }
        self.readback(sim);
        self.last_stepped = target;
    }

    /// Put the world back at tick 0 from the descriptions the bodies were
    /// spawned with.
    ///
    /// The live `Transform` is no help here — the readback has already
    /// overwritten it with the simulated pose — which is why every body
    /// carries its [`BodyRef::rest`].
    ///
    /// **Clears the ring**, because this hands out fresh rapier handles: a
    /// checkpoint captured before the rebuild indexes bodies through the old
    /// arena, and restoring it would leave the bridge's handles addressing
    /// nothing. The pose published would then be stale, in silence — the
    /// worst kind of wrong. (The handles very likely come back identical,
    /// insertion order being the same; "very likely" is not a thing to build
    /// a cache on, and clearing here costs nothing since the ring already
    /// missed.)
    fn rebuild_from_rest(&mut self) {
        self.ring.clear();
        self.world = PhysicsWorld::new();
        self.settings.apply_to(&mut self.world);
        // BTreeMap → entity order, so the fresh handles are assigned in the
        // same deterministic order as the original spawn (HR-5).
        for b in self.bodies.values_mut() {
            b.handle = self.world.spawn_body(b.rest);
        }
        // Joints come back in the SAME call: the rewind replays its owed steps
        // immediately, and a replay missing the joints is a different
        // simulation — the chain would fall apart and re-assemble a frame later.
        self.respawn_joints_from_rest();
    }

    /// Spawn bodies for new physics entities, remove bodies for despawned
    /// ones. New bodies are created in a **stable (entity-sorted) order** so
    /// the rapier handle assignment — and thus the whole simulation — does
    /// not depend on ECS archetype/insertion order (HR-5, the same sort
    /// `propagate_transforms` uses for roots). Structural only; it does not
    /// touch existing bodies' poses (that is `step`/`settle`).
    fn reconcile_structure(&mut self, sim: &SimWorld) {
        self.seen.clear();
        self.to_spawn.clear();
        self.to_remove.clear();

        let world = sim.world();
        // Take the cached query out so the loop body can push into the
        // other scratch fields without a borrow clash.
        let mut q = self.query.take().expect("query built in dispatch");
        let at_rest = self.last_stepped == 0;
        for (e, rb, col, _local) in q.iter(world) {
            self.seen.push(e);
            // ⚠️ WORLD, not the local `Transform` the query just handed us. The
            // solver has no hierarchy: a body authored under a parent must be
            // DESCRIBED where it is actually drawn, or it spawns (and rests, and
            // collides) at its local coordinates read as world ones.
            let Some(t) = space::world_transform(world, e, &mut self.chain) else {
                continue;
            };
            // Optional per-body gravity multiplier (W8); absent = full gravity.
            // Read here because this is the half holding the `World`; folded into
            // the `BodyDesc` so it survives rewind.
            let gravity_scale = world
                .get::<crate::GravityScale>(e)
                .map_or(crate::GravityScale::NEUTRAL, |g| g.0);
            // Optional authored initial velocity (W9); absent = at rest. Folded
            // into the `BodyDesc` for the same reason — a rewind re-arms it.
            let iv = world
                .get::<crate::InitialVelocity>(e)
                .copied()
                .unwrap_or(crate::InitialVelocity::REST);
            // Optional CCD marker (W-CCD); its PRESENCE is the flag (absent =
            // discrete, rapier's default). Folded into the `BodyDesc` so a rewind
            // re-arms it, exactly like the two above.
            let ccd = world.get::<crate::Ccd>(e).is_some();
            // Optional LockRotation marker (Freeze Rotation); presence is the flag.
            let lock_rotation = world.get::<crate::LockRotation>(e).is_some();
            // Optional LockPositionX/Y markers (Freeze Position); each presence is a
            // flag that ORs its axis into the same `LockedAxes` fold, so a rewind
            // re-arms it exactly like the rotation lock above.
            let lock_x = world.get::<crate::LockPositionX>(e).is_some();
            let lock_y = world.get::<crate::LockPositionY>(e).is_some();
            // Optional MassOverride (W-Mass); its VALUE is the explicit mass in kg
            // (`None` = auto, density-derived). Folded into the `BodyDesc` so a
            // rewind re-arms it, exactly like gravity scale.
            let mass_override = world.get::<crate::MassOverride>(e).map(|m| m.0);
            // Optional Dominance (W-Dominance); its VALUE is the collision priority
            // (absent = neutral `0`). Folded in and rides the `BodyDesc` for rewind.
            let dominance = world.get::<crate::Dominance>(e).map_or(0, |d| d.0);
            // Optional MaterialCombine (W-Material); absent = both `Average`. The
            // collider's friction/restitution combine policy, folded in and riding
            // the `BodyDesc` for rewind like the rest.
            let material = world
                .get::<crate::MaterialCombine>(e)
                .copied()
                .unwrap_or_default();
            // Optional DampingOverride (W-Damping); absent = the world default drag.
            // Folded into the `BodyDesc` for rewind, AND re-stamped each dispatch by
            // `restamp_damping` so a mid-play global-drag change cannot clobber it.
            let damping = world.get::<crate::DampingOverride>(e).copied();
            // Optional OneWayPlatform marker (W-OneWay); its PRESENCE is the flag. A
            // collider property (a platform is usually Static), folded in and riding
            // the `BodyDesc` for the rewind like the rest.
            let one_way = world.get::<crate::OneWayPlatform>(e).is_some();
            // Optional AreaEffector (W-Area); its VALUE is the force in newtons the
            // zone applies to whatever overlaps it. Folded in and riding the
            // `BodyDesc` for the rewind; the wrapper refuses it on a solid collider.
            // Optional AreaEffector / AreaDrag (W-Area, W-AreaDrag); the VALUES are
            // what the area does to whatever overlaps it. Two components, bundled here
            // into the one `AreaEffect` the world takes — they are separate on this
            // side because a component blob is positional and a second FIELD would be
            // a schema bump, while a second COMPONENT is additive.
            let zone_force = world.get::<crate::AreaEffector>(e).map(|a| a.force);
            let zone_drag = world.get::<crate::AreaDrag>(e).map(|d| d.0);
            let zone_density = world.get::<crate::AreaBuoyancy>(e).map(|b| b.0);
            let any = zone_force.is_some() || zone_drag.is_some() || zone_density.is_some();
            let effector = any.then(|| ph2d_physics::AreaEffect {
                force: zone_force.unwrap_or([0.0, 0.0]),
                drag: zone_drag.unwrap_or(0.0),
                density: zone_density.unwrap_or(0.0),
            });
            let desc = crate::scale::body_desc(
                rb,
                col,
                &t,
                gravity_scale,
                iv.linvel,
                iv.angvel,
                ccd,
                lock_rotation,
                lock_x,
                lock_y,
                mass_override,
                dominance,
                material,
                damping,
                one_way,
                effector,
            );
            match self.bodies.get(&e) {
                None => self.to_spawn.push((e, desc, rb.kind)),
                // **The rest pose is the authored pose at tick 0** — read, not
                // remembered. Without this, `rest` froze at the pose the body
                // happened to be spawned with, so moving an object and pressing
                // Reset threw the artist's placement away and jumped it back to
                // where it first appeared. It also covers a shape or density
                // edited at tick 0 (the Inspector, W2): re-describing the body
                // is one rule instead of a growing list of fields to watch.
                Some(b) if at_rest && b.rest != desc => {
                    self.to_spawn.push((e, desc, rb.kind));
                    self.to_remove.push(e);
                }
                Some(_) => {}
            }
        }
        self.query = Some(q);

        // Stale bodies: entity no longer carries the components. O(N²) is
        // fine at W1 body counts and allocates nothing; only fires on
        // despawn (steady state leaves `to_remove` empty).
        for (&e, _) in self.bodies.iter() {
            if !self.seen.contains(&e) {
                self.to_remove.push(e);
            }
        }
        // Any structural change makes every cached state a snapshot of a
        // DIFFERENT world: restoring one would hand back rapier handles that
        // no longer address the entities this bridge is holding, and the
        // wrong pose would be published without anything looking broken.
        // Asked once, for both directions.
        if !self.to_spawn.is_empty() || !self.to_remove.is_empty() {
            self.ring.clear();
        }

        self.to_remove.sort_unstable_by_key(|e| e.to_bits());
        for i in 0..self.to_remove.len() {
            let e = self.to_remove[i];
            if let Some(b) = self.bodies.remove(&e) {
                self.world.remove_body(b.handle);
            }
        }
        self.to_remove.clear();

        // Deterministic spawn order (HR-5): sort by entity bits so handle
        // assignment is a pure function of the entity set.
        self.to_spawn.sort_unstable_by_key(|(e, _, _)| e.to_bits());
        for i in 0..self.to_spawn.len() {
            let (e, desc, kind) = self.to_spawn[i];
            let handle = self.world.spawn_body(desc);
            // `desc` was built from the Transform as it is RIGHT NOW, before
            // any stepping — that is the rest state a rewind replays from.
            self.bodies.insert(
                e,
                BodyRef {
                    handle,
                    kind,
                    rest: desc,
                },
            );
        }
        self.to_spawn.clear();
    }

    /// Aim every kinematic body at the pose its entity's `Transform` now
    /// holds — the mirror image of [`Self::readback`], and the reason a baked
    /// body still shoves the sim instead of ghosting through it.
    ///
    /// Nothing here decides WHERE the body goes; the scene already did, and
    /// this stage only tells rapier. That is the whole contract of the kind:
    /// the pose is an input, so a curve, a gizmo drag and a parent's motion
    /// all drive it identically, with no branch per author.
    /// Read each dynamic body's pose back into its entity's `Transform`
    /// (meters, radians CCW, Y-up — no conversion; ADR-0131 D4). Static
    /// bodies never move and kinematic ones are DRIVEN by that same
    /// `Transform` ([`Self::drive_kinematic`]), so both are skipped — asked
    /// through [`BodyKind::solver_owns_pose`], the one door, because a body
    /// that both stages claimed would have its pose written twice a tick.
    /// ⚠️ The solver answers in WORLD space and `Transform` is LOCAL, so the
    /// pose goes back through [`space::write_world_pose`]. Assigning it raw
    /// works for a root (where the two coincide) and is wrong for every child:
    /// the renderer composes the parent onto it again, so the body simulates
    /// in one place and draws in another. A parent that cannot be inverted
    /// (scaled to zero) leaves the `Transform` untouched rather than storing
    /// a non-finite pose that would poison the whole subtree.
    fn readback(&mut self, sim: &mut SimWorld) {
        let world = sim.world_mut();
        for (&e, b) in self.bodies.iter() {
            if !b.kind.solver_owns_pose() {
                continue;
            }
            if let Some(pose) = self.world.body_pose(b.handle) {
                space::write_world_pose(
                    world,
                    e,
                    pose.translation.x,
                    pose.translation.y,
                    pose.rotation.angle(),
                    &mut self.chain,
                );
            }
        }
    }
}
