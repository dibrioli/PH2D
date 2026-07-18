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

use std::collections::HashMap;

use bevy_ecs::query::QueryState;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

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
}

/// The ECS ↔ rapier bridge. One per document, held on `AppGfx.physics`.
pub struct PhysicsBridge {
    world: PhysicsWorld,
    bodies: HashMap<Entity, BodyRef>,
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
            bodies: HashMap::new(),
            last_stepped: 0,
            query: None,
            seen: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
        }
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
        self.world.set_gravity(x, y);
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
        if playing {
            if target > self.last_stepped {
                let owed = target - self.last_stepped;
                for _ in 0..owed {
                    self.world.step();
                }
                self.readback(sim);
                self.last_stepped = target;
            }
            // target <= last_stepped while playing = no new fixed tick this
            // frame: leave the world as-is. Settling here would yank bodies
            // back to the authored Transform and fight the sim.
        } else {
            self.settle(sim);
            self.last_stepped = target;
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
        for (e, rb, col, t) in q.iter(world) {
            self.seen.push(e);
            if !self.bodies.contains_key(&e) {
                self.to_spawn.push((e, body_desc(rb, col, t), rb.kind));
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
            self.bodies.insert(e, BodyRef { handle, kind });
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
            if let Some(pose) = self.world.body_pose(b.handle) {
                if let Some(mut t) = world.get_mut::<Transform>(e) {
                    t.translation.x = pose.translation.x;
                    t.translation.y = pose.translation.y;
                    t.rotation = pose.rotation.angle();
                }
            }
        }
    }

    /// While paused: make every body track the authored `Transform` (and
    /// zero its velocity), so play starts from exactly where the artist
    /// left it. No stepping.
    fn settle(&mut self, sim: &SimWorld) {
        let world = sim.world();
        for (&e, b) in self.bodies.iter() {
            if let Some(t) = world.get::<Transform>(e) {
                self.world
                    .set_body_pose(b.handle, t.translation.x, t.translation.y, t.rotation, false);
            }
        }
    }

    /// A blake3 digest over the readback poses (the ECS-visible result of
    /// the whole bridge: sync + step + our conversion + readback), sorted
    /// so the HashMap iteration order can never leak in. This is what the
    /// `physics_ecs_c9` harness prints and CI compares cross-OS (ADR-0130
    /// D7) — it proves OUR code on the deterministic path, not just the
    /// wrapper's internal `deterministic_hash`.
    pub fn deterministic_hash(&self, sim: &SimWorld) -> [u8; 32] {
        let world = sim.world();
        let mut rows: Vec<[u8; 12]> = self
            .bodies
            .keys()
            .filter_map(|&e| {
                world.get::<Transform>(e).map(|t| {
                    let mut b = [0u8; 12];
                    b[0..4].copy_from_slice(&t.translation.x.to_bits().to_le_bytes());
                    b[4..8].copy_from_slice(&t.translation.y.to_bits().to_le_bytes());
                    b[8..12].copy_from_slice(&t.rotation.to_bits().to_le_bytes());
                    b
                })
            })
            .collect();
        // Sort by pose bytes: order-independent of HashMap iteration, so
        // the digest is a pure function of the SET of poses.
        rows.sort_unstable();
        let mut hasher = blake3::Hasher::new();
        for b in &rows {
            hasher.update(b);
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
