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
pub mod bodies;
pub mod contacts;
mod damping;
mod diagnostics;
pub mod fk;
mod grab;
mod hold;
pub mod ik;
mod inspect;
pub mod joint_break;
pub mod joints;
mod kinematic;
mod readback;
mod rewind;
pub mod rope;
/// As settings de MUNDO — módulo irmão pelo cap de 700 LOC.
mod settings;
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

/// O semeio de um joint: a entidade, as duas âncoras body-local e — só para uma
/// polia — o comprimento da corda, que sai da ROTA que as roldanas desenham.
type JointSeed = (Entity, [f32; 2], [f32; 2], Option<f32>);

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
    /// A sessão de CINEMÁTICA DIRETA viva (W-FK), ou `None` fora do gesto.
    ///
    /// Irmã do `ik` e, como ela, fora do checkpoint e de tudo que é persistido:
    /// um gesto de pose é **ferramenta**, e o que sobrevive a ele é o
    /// `Transform` autorado que o chamador escreveu.
    pub(super) fk: Option<fk::FkSession>,
    joint_query: Option<JointQuery>,
    /// A query das roldanas. Separada da dos joints porque uma roldana é uma
    /// ENTIDADE própria (W-Pulley W1) — a corda a aponta pelo nome.
    wheel_query: Option<rope::WheelQuery>,
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
    /// O terceiro elemento é o semeio de POLIA — as duas roldanas e o
    /// comprimento da corda — presente só para uma [`crate::JointKind::Pulley`]
    /// que ainda não foi ancorada. Viaja no mesmo scratch das âncoras porque é o
    /// MESMO sentinela (`anchored`) que governa os dois: uma polia com roldanas
    /// em `[0,0]` é uma que nunca foi semeada, e o semeio acontece uma vez, das
    /// poses de REPOUSO.
    joints_to_seed: Vec<JointSeed>,
    /// **As roldanas cujo eixo local acaba de ser semeado** (W-Pulley W3), com o
    /// local derivado da pose de REPOUSO do corpo em que estão montadas.
    ///
    /// Scratch, limpo a cada colheita. Irmão exato do `joints_to_seed` e pelo
    /// MESMO motivo: a conversão mundo→local acontece uma vez e é persistida,
    /// senão um move de corpo re-derivaria e o eixo deslizaria pelo bloco.
    wheels_to_seed: Vec<(Entity, [f32; 2])>,
    /// A tabela de polias a instalar neste dispatch, reconstruída inteira todo
    /// frame (uma polia não é dona de nada nas arenas do rapier, então não há
    /// diff de spawn/remove a fazer — e é por isso que ela nunca invalida o ring
    /// de checkpoints).
    pulleys_to_install: Vec<ph2d_physics::world::pulley::PulleyDesc>,
    /// **A arena de roldanas** que as faixas de `pulleys_to_install` indexam —
    /// uma lista só para todas as cordas do mundo, pelo motivo escrito no
    /// `PulleyDesc`: um `Vec` por corda alocaria por frame.
    pulley_wheels_to_install: Vec<ph2d_physics::world::rope_route::RopeWheel>,
    /// As roldanas colhidas do mundo neste dispatch, ORDENADAS por
    /// `(corda, order, desempate por nome)` — a chave inteira é estável através
    /// do undo, ao contrário dos bits de entidade, que são id de ALOCAÇÃO.
    ///
    /// Scratch: limpa e repreenchida por dispatch, então o regime não realoca.
    rope_wheels: Vec<rope::RopeWheelRow>,
    /// **O ângulo de cada roldana**, radianos, na MESMA ordem da arena.
    ///
    /// ⚠️ **Estado VIVO, e é por isso que ele mora aqui e não no componente**: um
    /// ângulo serializado seria um passo de undo por frame (o `canonicalize`
    /// ordena pelas bytes do componente), a mesma razão pela qual velocidade e
    /// sono nunca entraram no `RigidBody`. Um replay o reintegra sozinho.
    wheel_spin: Vec<f32>,
    /// O ângulo por ENTIDADE, para ele sobreviver ao reconcile — a arena é
    /// reconstruída todo dispatch, e sem esta memória toda roda voltaria a zero a
    /// cada frame.
    wheel_spin_by_entity: std::collections::BTreeMap<Entity, f32>,
    /// A entidade de cada roldana da arena, na MESMA ordem — é por ela que o
    /// desenho e as alças sabem QUEM é a roda sob o cursor. Paralela em vez de um
    /// campo dentro do `RopeWheel` porque aquele tipo é do motor, que não conhece
    /// entidade nenhuma (e não deve conhecer).
    wheel_entities: Vec<Entity>,
    /// **O que o artista escolheu sobre o LADO**, na MESMA ordem da arena.
    ///
    /// ⚠️ Paralela e não lida direto das linhas colhidas porque a arena passa por
    /// um FILTRO que elas não passam: um eixo que cedeu não é instalado. Enquanto o
    /// override era lido por `zip` contra as linhas cruas, o primeiro eixo partido
    /// de uma corda **deslocava** todos os overrides seguintes para a roldana
    /// vizinha — o defeito nascia só depois de uma ruptura, e só com um abraço
    /// autorado, que é por que ele viveu calado. O override tem de andar pelo MESMO
    /// filtro que a roda que ele governa.
    wheel_wraps: Vec<crate::WrapSide>,
    /// Os trechos que a resolução de LADO escreve. Scratch pelo mesmo motivo.
    route_scratch: Vec<ph2d_physics::world::rope_route::Tangent>,
    /// **A lista de polias VIVAS**, reconstruída todo reconcile — a fonte única
    /// de que tanto a tabela do solver quanto as views de desenho saem. Uma
    /// polia não vive no `ImpulseJointSet`, então sem este registro ela seria
    /// invisível na tela.
    pulley_records: Vec<views::PulleyRecord>,
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
    /// O mesmo, para as CORDAS (W-Pulley W2): elas não vivem no `joints`, então
    /// o laço de cima não passa por elas. Newtons; só a linear, porque uma corda
    /// não transmite torque.
    pulley_peaks: BTreeMap<Entity, f32>,
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
            fk: None,
            joint_query: None,
            wheel_query: None,
            seen: Vec::new(),
            names: BTreeMap::new(),
            joints_seen: Vec::new(),
            joints_to_spawn: Vec::new(),
            joints_to_remove: Vec::new(),
            joints_to_seed: Vec::new(),
            wheels_to_seed: Vec::new(),
            pulleys_to_install: Vec::new(),
            pulley_wheels_to_install: Vec::new(),
            rope_wheels: Vec::new(),
            wheel_entities: Vec::new(),
            wheel_wraps: Vec::new(),
            wheel_spin: Vec::new(),
            wheel_spin_by_entity: std::collections::BTreeMap::new(),
            route_scratch: Vec::new(),
            pulley_records: Vec::new(),
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
            pulley_peaks: BTreeMap::new(),
            flashes: Vec::new(),
            contacts_continuous: true,
        }
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
        self.wheel_query = None;
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
        if self.wheel_query.is_none() {
            self.wheel_query = Some(sim.world_mut().query());
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
                    self.accumulate_pulley_breaks();
                    // And the high-water mark of the RUN (the tuning signal): the
                    // wrapper's peak is per-TICK and a yank is over before it can
                    // be read.
                    self.accumulate_joint_peaks();
                    self.accumulate_pulley_peaks();
                    // E o GIRO das roldanas, que é o que torna uma roda uma roda
                    // na tela (`bridge::rope`). Por TICK, como tudo aqui: o
                    // ângulo é a integral de uma taxa, e integrá-lo por FRAME
                    // deixaria a roda girar mais rápido em máquina rápida.
                    self.spin_rope_wheels();
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
        // W-Pulley W3: e os eixos das roldanas MONTADAS onde os corpos delas
        // ficaram — **incondicional, e é essa a correção**.
        //
        // ⚠️ A arena é reinstalada por `prepare` a cada dispatch com o centro
        // derivado da pose de REPOUSO (o único que a colheita do ECS conhece), e
        // até aqui só o laço de sub-passos a refrescava. Um quadro mais rápido
        // que o tique não dá passo nenhum ⇒ ele publicava a roldana **onde ela
        // foi autorada**: medido em **1,27 m** de salto num bloco que viaja, com
        // a simulação correta o tempo todo. Era o tremor do smoke da talha, e o
        // fato de ser só-desenho é exatamente o que se espera de uma lista que o
        // solver refresca e o pintor lê.
        //
        // Aqui, e não junto da instalação: este é o único ponto por onde as
        // QUATRO saídas passam (replay, laço de tiques, `settle` pausado, e o
        // quadro que não deve tique nenhum), então a arena publicada descreve
        // onde as roldanas ESTÃO sem ninguém ter de enumerar os ramos. Nos que
        // dão passo é idempotente com o que o `step` já fez.
        self.world.refresh_mounted_wheels();
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
            // W-Pulley W3: e o mesmo para o EIXO de uma roldana montada, pela
            // razão idêntica — o dot de centro e a §2 Position leem o
            // `Transform`, então mover o BLOCO tem de levar a roldana junto. Em
            // play a arena já carrega o centro vivo (`refresh_mounts`) e é dela
            // que o desenho lê, então escrever aqui seria trabalho sem leitor.
            self.sync_mounted_wheels(sim);
        }
    }
}
