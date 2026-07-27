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

pub mod anchors;
pub mod contacts;
mod damping;
mod diagnostics;
mod grab;
mod hold;
pub mod ik;
mod inspect;
pub mod joint_break;
pub mod joints;
mod kinematic;
mod readback;
mod rewind;
mod space;
mod triggers;
pub mod views;

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
    /// A sessão de POSE viva (W-IK), ou `None` fora de um gesto de IK.
    ///
    /// Fora do checkpoint e fora de tudo que é persistido, de propósito: uma
    /// árvore de multibody é **ferramenta**, não estado da cena — ela nasce num
    /// press e morre num release, e o que sobrevive ao gesto é o `Transform`
    /// autorado que o chamador escreveu.
    pub(super) ik: Option<ik::IkSession>,
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
    /// Joints whose body-local anchors were just derived from their world
    /// `Transform` (the seed) — written back after the query borrow releases so
    /// a body move never re-derives them again. Scratch: cleared per reconcile.
    joints_to_seed: Vec<(Entity, [f32; 2], [f32; 2])>,
    /// (joint, derived world pivot) pairs — `sync_joint_pivots` writes each into
    /// the joint's `Transform.translation` so the anchor dot follows the body.
    /// Scratch: taken out and refilled per sync, so the steady state never allocs.
    joints_to_sync: Vec<(Entity, [f32; 2])>,
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
    /// The bodies a settle pass is about to walk. Collected first because
    /// `follow_authored_pose` borrows the `chain` scratch mutably while
    /// `self.bodies` is being iterated; retained for the same zero-alloc reason
    /// as every other buffer here.
    to_settle: Vec<(Entity, RigidBodyHandle)>,
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
    /// What was touching at the END of the previous TICK, with where and how hard it
    /// last hit — the memory that turns a per-tick set into TRANSITIONS
    /// (`bridge::contacts`). `BTreeMap` for the determinism reason `bodies`
    /// documents; the key order is the order `Ended` events come out in.
    contact_since: BTreeMap<(Entity, Entity), contacts::ContactMemo>,
    /// The transitions of this dispatch — cleared at its start, appended to per tick.
    contact_events: Vec<contacts::ContactEvent>,
    /// The joints that gave way in this dispatch (`bridge::joint_break`). Same
    /// shape and the same lifetime as `contact_events`, and for the same reason:
    /// a break is a transition, and a dispatch can owe several ticks.
    joint_breaks: Vec<joint_break::JointBreakEvent>,
    /// The hardest each joint has been pulled since the clock last restarted
    /// (`bridge::joint_break`) — the number a break threshold is TUNED against.
    /// Unlike `joint_breaks` this is not per-dispatch: it is the memory of the run.
    joint_peaks: BTreeMap<Entity, ph2d_physics::JointLoad>,
    /// The live begin-flashes (`bridge::contacts`) — the visible half. Seeded from
    /// `Began` transitions and decayed in SIM ticks; PERSISTS across dispatches (a
    /// flash outlives the tick it was born in), so it is not cleared per dispatch, only
    /// aged and dropped, or cleared by a discontinuity.
    flashes: Vec<contacts::ContactFlash>,
    /// Whether `contact_since` describes the tick immediately before this one.
    ///
    /// False after any discontinuous clock move, which makes the next forward tick adopt
    /// the set in SILENCE instead of reporting a scrub as a hundred collisions. Starts
    /// TRUE over an empty baseline, so the first stepped tick does report what it finds
    /// — the Unity reading, argued in `accumulate_contact_events`.
    contacts_continuous: bool,
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
            ik: None,
            joint_query: None,
            seen: Vec::new(),
            names: BTreeMap::new(),
            joints_seen: Vec::new(),
            joints_to_spawn: Vec::new(),
            joints_to_remove: Vec::new(),
            joints_to_seed: Vec::new(),
            joints_to_sync: Vec::new(),
            kin_start: Vec::new(),
            chain: Vec::new(),
            to_spawn: Vec::new(),
            to_remove: Vec::new(),
            to_settle: Vec::new(),
            settings: PhysicsSettings::default(),
            ring: PhysicsCheckpointRing::new(),
            steps_taken: 0,
            triggers: BTreeMap::new(),
            contacts: Vec::new(),
            contact_since: BTreeMap::new(),
            contact_events: Vec::new(),
            joint_breaks: Vec::new(),
            joint_peaks: BTreeMap::new(),
            flashes: Vec::new(),
            contacts_continuous: true,
        }
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
        // A static body has ONE author — the authored `Transform` — so it tracks
        // it on every dispatch, not only on the paused ones. Before this, a wall
        // dragged with the clock running moved the drawing and left the collider
        // behind (`bridge::hold::settle_static`). Runs BEFORE the step, so the
        // tick is solved against the wall the artist can see; and it is a no-op
        // for every body nobody touched, so a settled scene is unaffected.
        self.settle_static(sim);
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
        // Transitions are a fresh list per dispatch; the forward loop appends to it,
        // tick by tick. (Flashes are NOT cleared here — they outlive their tick.)
        self.contact_events.clear();
        self.joint_breaks.clear();
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
                // The handle→entity map for the per-tick event diff. Built ONCE here:
                // the forward branch never respawns bodies (only a rewind does), so the
                // handles are stable across the loop.
                let by_handle = self.handle_map();
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
                    // Diff this tick's touching union against the standing set — the
                    // only place the clock stepped through the transitions, and the one
                    // that catches a touch shorter than a whole tick (W-TickContacts).
                    self.accumulate_contact_events(&by_handle);
                    // And the joints that parted during that same tick (W-J7) —
                    // the wrapper clears its own list every `step`, so a break in
                    // an early tick of a multi-tick dispatch is gone by the last.
                    self.accumulate_joint_breaks();
                    // And the high-water mark of the RUN (the tuning signal): the
                    // wrapper's peak is per-TICK and a yank is over before it can
                    // be read.
                    self.accumulate_joint_peaks();
                    tick += 1;
                    // Asking is free; capturing costs about one step, which
                    // is why the ring is sparse (see `PhysicsCheckpointRing`).
                    //
                    // ⚠️ **Nada é gravado enquanto um CUTUCÃO está em voo** (a
                    // mão do W-Grab ou o campo de atração do W-Hand; regra 1 de
                    // `bridge::grab`): um checkpoint tirado sob o
                    // cutucão descreve uma corrida que nenhum replay reproduz,
                    // então semear um scrub com ele faria a resposta para um tick
                    // depender de o cache tê-lo ou não. O `grab` já limpou o que
                    // havia; isto impede que a janela se re-encha.
                    if !self.is_poking() && self.ring.should_record(tick) {
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
        // actually touching whom, where, and under how much load — the STANDING set,
        // read from the same final world state as the triggers, for the same reason.
        // The transitions and flashes are the forward loop's job above; this only
        // publishes what touches AT the tick the artist is now looking at, which is a
        // question even a scrub must answer. No-op (and no alloc) when nothing touches.
        self.rebuild_standing_contacts();
        // Sync each joint's DISPLAY pivot (its `Transform.translation`) to where
        // its authored body-local anchor now sits — so the anchor dot and the
        // Inspector Position follow the body when a body moves. Rest-only: during
        // play the dot is not shown and the overlay draws the live solver anchors
        // (`joint_anchors`), so a per-frame write there would be a stale display
        // value for no reader. `!playing` also covers a scrub that lands paused.
        if !playing {
            self.sync_joint_pivots(sim);
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
            let zone_form = world.get::<crate::AreaFormDrag>(e).map(|f| f.0);
            // The rotational half (W-AreaTorque): a fifth optional component, folded into
            // the same `AreaEffect` bundle for the same reason as its siblings.
            let zone_torque = world.get::<crate::AreaTorque>(e).map(|t| t.0);
            // The FRAME of the force (W-AreaFrame): a sixth optional component, and a
            // MARKER — its presence pins the force to world axes, its absence (the
            // default) authors it in the zone's own frame so turning the sensor turns
            // the wind.
            //
            // ⚠️ It is deliberately NOT part of `any` — a marker alone describes the
            // frame of a force that is not there — but that exclusion is HYGIENE, not
            // correctness, and a mutation proved it: putting it in `any` leaves every
            // gate green, because `effector::zone_effect` refuses a wholly inert zone
            // anyway (zero force, zero torque, no drag). Same shape as the two refusals
            // that function documents about itself, and worth writing down for the same
            // reason: ask what a layer buys ALONE and be willing to answer "nothing"
            // ([[feedback_layered_defenses_need_per_layer_gates]]). What it does buy is
            // a `BodyDesc` that does not claim to be a zone when it is not.
            let zone_world_axes = world.get::<crate::AreaForceWorldAxes>(e).is_some();
            // How much of the push reaches the far side (W-AreaFalloff): a seventh
            // optional component, folded into the same bundle. Like the marker above it is
            // deliberately NOT part of `any` — a falloff is a MODIFIER, so a zone carrying
            // nothing else attenuates nothing and is not a zone.
            //
            // ⚠️ And, like the marker above, that exclusion is HYGIENE rather than
            // correctness — a mutation proved it: putting it in `any` leaves both gates
            // green, because `effector::zone_effect` refuses the wholly inert zone anyway.
            // I first wrote here that "the reason is not merely hygiene"; the measurement
            // says otherwise, and the honest version is the one that survives being tested
            // ([[feedback_layered_defenses_need_per_layer_gates]]). What it buys is the
            // same thing its sibling buys: a `BodyDesc` that does not claim to be a zone.
            let zone_falloff = world.get::<crate::AreaFalloff>(e).map(|f| f.0);
            let any = zone_force.is_some()
                || zone_drag.is_some()
                || zone_density.is_some()
                || zone_form.is_some()
                || zone_torque.is_some();
            let effector = any.then(|| ph2d_physics::AreaEffect {
                force: zone_force.unwrap_or([0.0, 0.0]),
                drag: zone_drag.unwrap_or(0.0),
                density: zone_density.unwrap_or(0.0),
                form_drag: zone_form.unwrap_or(0.0),
                torque: zone_torque.unwrap_or(0.0),
                world_axes: zone_world_axes,
                falloff: zone_falloff.unwrap_or(0.0),
                // A lateralidade do frame é função da POSE, não dos componentes, então ela
                // não se decide aqui: `scale::body_desc` a dobra ao lado da linha que já
                // dobra a escala sincada no offset (W-Offset), que é a mesma regra.
                mirror: ph2d_physics::AreaEffect::UNMIRRORED,
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
}
