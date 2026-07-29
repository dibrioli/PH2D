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
use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route;
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
///
/// `pub` (re-exported from the crate root) because it is the documented place
/// where "which parameters does this kind actually use?" is answered, and that is
/// a question a gate needs to ask directly: the alternative is inferring the
/// answer from a body's motion, which cannot distinguish *the threshold was not
/// passed* from *the threshold was passed and the reading is structurally zero* —
/// exactly the pair the torque row exists to keep apart.
///
/// ⚠️ **`None` para uma [`JointKind::Pulley`]**, e não é um caso de erro: o
/// rapier não tem polia, então aquele tipo não tem `JointDesc` nenhum a
/// produzir — ele é roteado para o passe de impulso de
/// `ph2d_physics::world::pulley`. Devolver `Option` em vez de mapear a polia
/// para o tipo "mais parecido" é o que impede uma corda de virar uma corda-de-
/// rapier em silêncio.
pub fn joint_desc(
    j: &PhysicsJoint,
    local_a: [f32; 2],
    local_b: [f32; 2],
    axis: ([f32; 2], [f32; 2]),
) -> Option<JointDesc> {
    Some(JointDesc {
        kind: match j.kind {
            JointKind::Pulley => return None,
            JointKind::Pin => ph2d_physics::JointKind::Pin,
            JointKind::Spring => ph2d_physics::JointKind::Spring,
            JointKind::Rope => ph2d_physics::JointKind::Rope,
            JointKind::Weld => ph2d_physics::JointKind::Weld,
            JointKind::Slider => ph2d_physics::JointKind::Slider,
            JointKind::Rod => ph2d_physics::JointKind::Rod,
            JointKind::Wheel => ph2d_physics::JointKind::Wheel,
        },
        anchor_a: local_a,
        anchor_b: local_b,
        // A parameter the kind ignores is not *passed* to the solver either:
        // `is_hinge` is asked here exactly as the Inspector asks it to decide
        // which rows to paint, so a limit left over from a previous kind
        // cannot quietly still be in force.
        // `has_limits`, not `is_hinge`: a Slider is limited too, and its range is
        // a stroke in metres (`limits_in_metres`) rather than an angle. The
        // Inspector asks the same door to decide which label to paint.
        limits: (j.kind.has_limits() && j.limits_enabled).then_some([j.limit_min, j.limit_max]),
        // `has_motor`, not `is_hinge`: a Slider's rail and a Rope's distance are
        // driven too (W-J6). The wrapper asks the SAME question through
        // `ph2d_physics::motor_axis` — which is where a Spring's exclusion is
        // load-bearing rather than cosmetic (rapier models a spring AS a motor,
        // so a second one there would eat its stiffness).
        motor: (j.kind.has_motor() && j.motor_enabled).then_some(MotorDesc {
            mode: match j.motor_mode {
                crate::joint::MotorMode::Velocity => ph2d_physics::MotorMode::Velocity,
                crate::joint::MotorMode::Position => ph2d_physics::MotorMode::Position,
            },
            speed: j.motor_speed,
            target: j.motor_target,
            max_force: j.motor_max_force,
        }),
        rest_length: j.rest_length,
        stiffness: j.stiffness,
        damping: j.damping,
        max_length: j.max_length,
        axis_a: axis.0,
        axis_b: axis.1,
        // ⚠️ `∞` is what "off" IS (W-J7, P7), so the checkbox is resolved HERE
        // rather than carried into the solver: the wrapper never learns that a
        // joint *could* be breakable, only what it can take. A joint with the box
        // clear packs to the same `user_data` a joint from before this wave has,
        // which is why every scene that predates it is byte-identical.
        break_force: if j.break_enabled {
            j.break_force
        } else {
            f32::INFINITY
        },
        // The torque threshold is offered on the Pin alone (`breaks_on_torque` —
        // rapier reports no reaction for a LOCKED angular axis, so a Weld's would
        // be a control that can never fire). Asked here as well as in the panel,
        // so a stale value left over from a kind change cannot quietly still be
        // in force — the same reason `limits` asks `has_limits`.
        break_torque: if j.break_enabled && j.kind.breaks_on_torque() {
            j.break_torque
        } else {
            f32::INFINITY
        },
        // W-J8. Both are carried verbatim — they are the same fact on both sides
        // of this boundary, unlike the break thresholds two lines up (a checkbox
        // here, an infinity there). An inactive joint is still BUILT and still
        // reconciled; what stops is the constraint.
        enabled: j.active,
        contacts_enabled: j.collide_connected,
    })
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
            // ⚠️ Clear the published list on this path too, and it is NOT
            // redundant: a DORMANT joint is seen without ever being built, so
            // deleting one leaves `self.joints` empty and the scene joint-less —
            // straight into this early-out, with last frame's list still holding
            // the corpse. `joint_entities` is what the canvas handles are drawn
            // from, so that is a grabbable dot for something that no longer
            // exists. (Deleting a BUILT joint takes the slow path, which clears
            // below; that case is why this looked redundant.)
            self.joints_seen.clear();
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
        self.pulley_records.clear();
        self.harvest_rope_wheels(world);
        self.wheel_entities.clear();
        self.wheel_spin.clear();

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
            // As poses de REPOUSO, copiadas aqui: tudo abaixo fala delas, e
            // segurar as referências de `self.bodies` até lá colidiria com o
            // `drop_if_live`, que precisa de `&mut self`.
            let (rest_a, rest_b) = (rest_pose(ba), rest_pose(bb));
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
                let centre_b = [rest_b[0], rest_b[1]];
                let (wa, wb) = anchor_points(
                    joint,
                    [transform.translation.x, transform.translation.y],
                    centre_b,
                );
                let la = PhysicsWorld::local_anchor_at_pose(rest_a, wa);
                let lb = PhysicsWorld::local_anchor_at_pose(rest_b, wb);
                // Persist after the query borrow releases (below), and flip
                // `anchored` so the next reconcile — and every body move — reads
                // these instead of re-deriving.
                self.joints_to_seed.push((e, la, lb, None));
                (la, lb)
            };
            // ⚠️ **A POLIA sai daqui**, antes do `joint_desc`: ela não é um joint
            // do rapier, então não tem descritor, não entra no `ImpulseJointSet` e
            // não toca as arenas — e é por isso que ela nunca invalida o ring de
            // checkpoints. `drop_if_live` cobre a troca de tipo: quem era um Pin
            // e virou polia tem de sair do solver.
            if joint.kind == JointKind::Pulley {
                drop_if_live(self);
                if joint.active {
                    // As âncoras em REPOUSO. A rota da corda é resolvida contra
                    // elas — e não contra a pose viva — porque é isso que congela
                    // o LADO de cada roldana durante o play: uma corda real não
                    // troca de lado da polia no meio da corrida, e um lado
                    // recomputado por frame pisca perto do degenerado, o que muda
                    // o comprimento e dá um puxão. Ao mesmo tempo, mover um corpo
                    // COM O RELÓGIO PARADO re-resolve, que é o que autoria quer.
                    let wa = PhysicsWorld::world_from_local_at_pose(rest_a, la);
                    let wb = PhysicsWorld::world_from_local_at_pose(rest_b, lb);
                    // A corda é apontada pelo NOME dela, que é a mesma chave
                    // por que ela aponta os corpos. Uma corda sem nome não pode
                    // ser apontada — e uma roldana com `rope: 0` não aponta nada.
                    let rope = world
                        .get::<Name>(e)
                        .map_or(0, |n| stable_name_id(n.as_str()));
                    let start = self.pulley_wheels_to_install.len() as u32;
                    let first = self.rope_wheels.partition_point(|r| r.rope < rope);
                    // A taxa da corda é a SOMA das taxas dos tambores dela (§6 do
                    // plano): um só é a própria taxa, dois puxando juntos recolhem
                    // o dobro, dois em sentidos opostos se anulam.
                    let mut motor_rate = 0.0;
                    for row in self.rope_wheels[first..]
                        .iter()
                        .take_while(|r| r.rope == rope)
                    {
                        // ⚠️ **Um eixo que cedeu não entra na arena**, e a arena
                        // é a lista que o DESENHO lê: sem isto o overlay
                        // desenharia a corda passando por uma roldana que o
                        // solver já ignorou — a segunda opinião que o doc do
                        // `physics_overlay_pulley` proíbe em voz alta. O passe
                        // tem o próprio filtro (é o que mantém o `PhysicsWorld`
                        // correto sozinho); esta é a outra camada, e ela tem
                        // gate próprio.
                        if !self.world.pulley_wheel_is_intact(row.wheel.id) {
                            continue;
                        }
                        motor_rate += row.reel_rate;
                        self.pulley_wheels_to_install.push(row.wheel);
                        self.wheel_entities.push(row.entity);
                        // O ângulo que esta roda já tinha: a arena é reconstruída
                        // todo dispatch, então sem esta memória por ENTIDADE toda
                        // roldana voltaria a zero a cada frame.
                        self.wheel_spin.push(
                            self.wheel_spin_by_entity
                                .get(&row.entity)
                                .copied()
                                .unwrap_or(0.0),
                        );
                    }
                    let count = self.pulley_wheels_to_install.len() as u32 - start;
                    let wheels = &mut self.pulley_wheels_to_install[start as usize..];
                    rope_route::resolve_sides(wa, wb, wheels, &mut self.route_scratch);
                    // O que o artista escolheu vence o que o algoritmo achou — o
                    // escape manual que a lição do Flip exige ao lado de todo
                    // matcher automático.
                    for (w, row) in wheels.iter_mut().zip(&self.rope_wheels[first..]) {
                        w.side = super::rope::wrap_side(row.wrap, w.side, w.centre, wa, wb);
                    }
                    // O comprimento da corda: autorado, ou semeado da rota que a
                    // montagem tem em REPOUSO — a corda nasce esticada, e o
                    // primeiro frame não dá um puxão.
                    let total_length = if joint.anchored {
                        joint.clamped().max_length
                    } else {
                        let l0 = rope_route::route(wa, wb, wheels, &mut self.route_scratch)
                            .map_or(joint.clamped().max_length, |r| r.length);
                        self.joints_to_seed.push((e, la, lb, Some(l0)));
                        l0
                    };
                    self.pulley_records.push(super::views::PulleyRecord {
                        entity: e,
                        entities: (ea, eb),
                        desc: PulleyDesc {
                            // A corda é ela mesma através das trocas de tabela
                            // pelo NOME dela — a mesma chave por que ela aponta os
                            // corpos e por que as roldanas a apontam.
                            id: rope,
                            body_a: handles.0,
                            body_b: handles.1,
                            local_a: la,
                            local_b: lb,
                            wheel_start: start,
                            wheel_count: count,
                            total_length,
                            motor_rate,
                            // O MESMO campo que todo joint carrega (W-J7), agora
                            // honrado por um vínculo que não é do rapier: a §12
                            // deixou de excluir a polia do card de ruptura.
                            break_force: if joint.break_enabled {
                                joint.clamped().break_force
                            } else {
                                f32::INFINITY
                            },
                        },
                    });
                }
                continue;
            }
            // `clamped()` here and not only in the Inspector: a component is
            // serde and arrives from the project file too, and this is the last
            // door before rapier.
            let Some(desc) = joint_desc(
                &joint.clamped(),
                la,
                lb,
                PhysicsWorld::axis_locals(transform.rotation, rest_a[2], rest_b[2]),
            ) else {
                drop_if_live(self);
                continue;
            };
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

        // Instalar as polias deste frame. A tabela do solver é DERIVADA do
        // registro — uma lista, duas leituras (a outra é o DESENHO) — e a troca
        // devolve a do frame anterior ao scratch, que é limpo mantendo a
        // capacidade: zero alocação em regime.
        self.pulleys_to_install
            .extend(self.pulley_records.iter().map(|r| r.desc));
        self.world.swap_pulleys(
            &mut self.pulleys_to_install,
            &mut self.pulley_wheels_to_install,
        );
        self.pulleys_to_install.clear();
        self.pulley_wheels_to_install.clear();

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
            let (e, la, lb, rope_length) = self.joints_to_seed[i];
            if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
                j.local_a = la;
                j.local_b = lb;
                // Só uma POLIA tem comprimento a semear, e ele vem da rota que as
                // roldanas de fato desenham — não de uma segunda regra aqui.
                if let Some(l0) = rope_length {
                    j.max_length = l0;
                }
                j.anchored = true;
            }
        }
        self.joints_to_seed.clear();

        // W-Pulley W3: o mesmo, uma família adiante. O eixo de uma roldana
        // montada foi convertido para o frame do corpo na colheita; persistir com
        // `mounted = true` é o que faz um move de corpo LER o local em vez de o
        // re-derivar — o eixo segue o bloco em vez de deslizar por ele.
        for i in 0..self.wheels_to_seed.len() {
            let (e, local) = self.wheels_to_seed[i];
            if let Some(mut wheel) = sim.world_mut().get_mut::<crate::PulleyWheel>(e) {
                wheel.local = local;
                wheel.mounted = true;
            }
        }
        self.wheels_to_seed.clear();
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
            // Through the SAME door the handles and the Inspector read
            // (`bridge::anchors`) — the displayed pivot and the grabbable dot
            // cannot describe different points if only one function answers
            // "where is the A anchor?". An un-seeded joint answers with its own
            // `Transform`, so this is a no-op there rather than a special case.
            for &e in self.joints.keys() {
                if let Some(pivot) = self.joint_anchor_world(sim, e, super::anchors::JointSide::A) {
                    scratch.push((e, pivot));
                }
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
