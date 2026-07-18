//! [`PhysicsBridge`] — the per-frame seam between the ECS and rapier.
//!
//! Owns the transient [`PhysicsWorld`] + the `Entity -> body handle` map.
//! **Not serialized** (ADR-0130 D2): the live world is *derived* from the
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

use std::collections::BTreeMap;

use bevy_ecs::query::QueryState;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics::{
    BodyDesc, PhysicsCheckpointRing, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc,
};

use crate::components::{BodyKind, Collider, ColliderShape, RigidBody};

/// The query the bridge iterates each frame. Cached (built once) because
/// `World::query` allocates a fresh `QueryState` per call — the cached
/// state is the zero-alloc idiom (HR-3), mirroring `TransformPropagationState`.
type BodyQuery = QueryState<(
    Entity,
    &'static RigidBody,
    &'static Collider,
    &'static Transform,
)>;

/// A live rapier body owned by the bridge, keyed by its ECS entity.
#[derive(Copy, Clone)]
struct BodyRef {
    handle: RigidBodyHandle,
    /// Cached so `readback` knows which bodies to read (only dynamic ones
    /// move; static bodies keep their authored pose).
    kind: BodyKind,
    /// The spawn description captured when this body was FIRST created —
    /// i.e. the **rest state at tick 0**. rapier cannot rewind, so a
    /// backwards clock replays from here (see
    /// [`PhysicsBridge::rewind_to`]). Without it, Reset could not put the
    /// ball back where it started: the live `Transform` has already been
    /// overwritten by the readback.
    rest: BodyDesc,
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
    // Reusable scratch — cleared+refilled each frame so the steady-state
    // hot path never reallocates (HR-3; proven by the capacity gate).
    /// Entities carrying physics components this frame (for stale detection).
    seen: Vec<Entity>,
    /// New entities to spawn a body for, this frame.
    to_spawn: Vec<(Entity, BodyDesc, BodyKind)>,
    /// Entities whose body must be removed this frame (component gone).
    to_remove: Vec<Entity>,
    /// Kept so a rewind can rebuild a fresh world with the same gravity
    /// (`PhysicsWorld::new` would silently reset it to the default).
    gravity: (f32, f32),
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
            seen: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
            gravity: (0.0, ph2d_physics::PhysicsWorld::DEFAULT_GRAVITY_Y),
            ring: PhysicsCheckpointRing::new(),
            steps_taken: 0,
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

    /// Number of live rapier bodies (for tests / diagnostics).
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Set world gravity (m/s²). Default is `(0, -9.81)` (Y-up).
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = (x, y);
        self.world.set_gravity(x, y);
        // Every cached state was simulated under the OLD gravity. Replaying
        // from one would mix two worlds — the same "an edit invalidates the
        // cache" rule Motion applies on `mark_dirty`.
        self.ring.clear();
    }

    /// Throw away the derived world. Call on project load / undo restore —
    /// entity bits are recycled there, so the handle map dangles. The world
    /// is rebuilt from components on the next [`dispatch`](Self::dispatch)
    /// (runtime-truth: the components are the truth, the world is derived).
    pub fn rebuild(&mut self) {
        self.world = PhysicsWorld::new();
        self.bodies.clear();
        self.last_stepped = 0;
        self.query = None; // re-bind to the (possibly fresh) world
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
        if self.query.is_none() {
            self.query = Some(sim.world_mut().query());
        }
        self.reconcile_structure(sim);
        match target.cmp(&self.last_stepped) {
            // The clock went BACKWARDS — Reset, or a scrub. rapier has no
            // rewind, so replay from the rest state (see `rewind_to`).
            std::cmp::Ordering::Less => self.rewind_to(sim, target),
            // Forward: advance the owed ticks, sequentially. This is play,
            // and it is equally a scrub FORWARD while paused — the sim state
            // is a function of the tick, not of the play button.
            std::cmp::Ordering::Greater => {
                let mut tick = self.last_stepped;
                for _ in 0..(target - self.last_stepped) {
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
    fn rewind_to(&mut self, sim: &mut SimWorld, target: u64) {
        let anchor = self.ring.seed(&mut self.world, target);
        let replayed = match anchor {
            Some(tick) => target - tick,
            None => {
                self.rebuild_from_rest();
                target
            }
        };
        for _ in 0..replayed {
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
        self.world.set_gravity(self.gravity.0, self.gravity.1);
        // BTreeMap → entity order, so the fresh handles are assigned in the
        // same deterministic order as the original spawn (HR-5).
        for b in self.bodies.values_mut() {
            b.handle = self.world.spawn_body(b.rest);
        }
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
        for (e, rb, col, t) in q.iter(world) {
            self.seen.push(e);
            let desc = body_desc(rb, col, t);
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

    /// Read each dynamic body's pose back into its entity's `Transform`
    /// (meters, radians CCW, Y-up — no conversion; ADR-0130 D4). Static
    /// bodies never move, so they are skipped. Only touches root-level
    /// bodies' local Transform == world for W1 (child bodies land in W2).
    fn readback(&self, sim: &mut SimWorld) {
        let world = sim.world_mut();
        for (&e, b) in self.bodies.iter() {
            if b.kind != BodyKind::Dynamic {
                continue;
            }
            if let Some(pose) = self.world.body_pose(b.handle)
                && let Some(mut t) = world.get_mut::<Transform>(e)
            {
                t.translation.x = pose.translation.x;
                t.translation.y = pose.translation.y;
                t.rotation = pose.rotation.angle();
            }
        }
    }

    /// While paused: make every body track the authored `Transform` (and
    /// zero its velocity), so play starts from exactly where the artist
    /// left it. No stepping.
    /// **A drag past tick 0 is transient, and that is a decision.** The
    /// simulation is a function of `(tick, authored rest state)`, so any
    /// rewind — through the ring or through the rest rebuild — reproduces the
    /// timeline as authored and drops a mid-sim nudge. Both paths discard it
    /// identically, which is why there is no cache invalidation here: a
    /// `ring.clear()` on a hand-move would be a defensive line protecting
    /// nothing, and its comment would be claiming otherwise. (Unity and Godot
    /// discard play-mode edits on stop for the same reason. Making a
    /// mid-timeline pose *stick* is authoring a keyframe, which is what W4's
    /// bake is for.)
    fn settle(&mut self, sim: &SimWorld) {
        let world = sim.world();
        for (&e, b) in self.bodies.iter() {
            let Some(t) = world.get::<Transform>(e) else {
                continue;
            };
            let (ax, ay, ar) = (t.translation.x, t.translation.y, t.rotation);
            // Only teleport when the AUTHORED pose actually differs from where
            // the body is. `set_body_pose` zeroes the velocity, so doing this
            // unconditionally every paused frame would make Pause → Play
            // restart the fall from a standstill. The readback writes the
            // body's pose into `Transform` exactly, so an untouched pair
            // compares equal; a gizmo drag makes them differ.
            let moved_by_hand = match self.world.body_pose(b.handle) {
                Some(pose) => {
                    pose.translation.x != ax
                        || pose.translation.y != ay
                        || pose.rotation.angle() != ar
                }
                None => false,
            };
            if moved_by_hand {
                self.world.set_body_pose(b.handle, ax, ay, ar, true);
            }
        }
    }

    /// A blake3 digest over the readback poses (the ECS-visible result of
    /// the whole bridge: sync + step + our conversion + readback). This is
    /// what the `physics_ecs_c9` harness prints and CI compares cross-OS
    /// (ADR-0130 D7) — it proves OUR code on the deterministic path, not
    /// just the wrapper's internal `deterministic_hash`.
    ///
    /// The `bodies` `BTreeMap` iterates in `Entity` order, which is
    /// deterministic per run and identical cross-OS (sequential entity
    /// allocation), so no sort is needed to pin the order.
    pub fn deterministic_hash(&self, sim: &SimWorld) -> [u8; 32] {
        let world = sim.world();
        let mut hasher = blake3::Hasher::new();
        for &e in self.bodies.keys() {
            if let Some(t) = world.get::<Transform>(e) {
                hasher.update(&t.translation.x.to_bits().to_le_bytes());
                hasher.update(&t.translation.y.to_bits().to_le_bytes());
                hasher.update(&t.rotation.to_bits().to_le_bytes());
            }
        }
        *hasher.finalize().as_bytes()
    }

    /// Capacity of the main per-frame scratch buffer — the zero-alloc gate
    /// asserts this is stable across steady-state frames (HR-3, capacity
    /// stability rather than a flaky global allocation counter).
    #[doc(hidden)]
    pub fn scratch_capacity(&self) -> usize {
        self.seen.capacity()
    }
}

/// Translate the authored components + current pose into a plain
/// [`BodyDesc`] for `PhysicsWorld::spawn_body`. The one place ECS types
/// meet rapier's — everything downstream is rapier-free.
fn body_desc(rb: &RigidBody, col: &Collider, t: &Transform) -> BodyDesc {
    BodyDesc {
        body_type: match rb.kind {
            BodyKind::Dynamic => RigidBodyType::Dynamic,
            BodyKind::Static => RigidBodyType::Fixed,
        },
        x: t.translation.x,
        y: t.translation.y,
        rotation: t.rotation,
        density: col.density,
        shape: match col.shape {
            ColliderShape::Ball { radius } => ShapeDesc::Ball { radius },
            ColliderShape::Cuboid { half_x, half_y } => ShapeDesc::Cuboid { half_x, half_y },
        },
    }
}
