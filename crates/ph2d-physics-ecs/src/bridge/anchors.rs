//! **Where a joint's two anchors are, and how a gesture puts one somewhere** —
//! the authoring door for both ends (W-J2).
//!
//! # One door, two ends
//!
//! W-AnchorFollow made the anchor authored **body-local state** (`local_a` /
//! `local_b`), with the joint's `Transform` demoted to a *derived* display value
//! for the A end. That fixed the slide, and it left one thing lopsided: only the
//! A end had any way to be authored. Body B's anchor was whatever the seed's
//! policy produced — the shared point for a Pin/Weld, B's own centre for a
//! Spring/Rope — and nothing in the editor could move it.
//!
//! This module is the missing half, and it is deliberately **one door for both
//! sides**: [`PhysicsBridge::joint_anchor_world`] answers *where is it*,
//! [`PhysicsBridge::set_joint_anchor_world`] answers *put it here*, and
//! [`PhysicsBridge::joint_snap_targets`] answers *what can it snap to*. The two
//! canvas handles, the Inspector's Position field and (later) any other gesture
//! all speak through them, so "the anchor moved" means the same thing however it
//! was asked for.
//!
//! # Why a reposition writes the local DIRECTLY, instead of re-seeding
//!
//! `PhysicsJoint::anchored` is a **joint-wide** sentinel: clearing it re-derives
//! *both* locals from the policy. That was harmless while B had no authored
//! value to lose — and it stops being harmless the moment the B handle exists,
//! because dragging A would silently throw B's placement away.
//!
//! So a gesture that knows a world point and a side writes that side's local
//! straight through this door and leaves `anchored` alone. Re-seeding survives
//! only where re-deriving BOTH ends is the intent — a fresh create, a kind
//! change (the policy for B itself changes), a body re-pick (the frame the local
//! belongs to is gone). ⚠️ Re-deriving A from the pivot is a **no-op** in those
//! cases anyway, because `sync_joint_pivots` keeps the pivot equal to
//! `bodyA · local_a`; the sentinel only ever really costs B.
//!
//! # The un-seeded joint keeps its old behaviour, through the same door
//!
//! A joint whose bodies are not resolved yet (half-authored, renamed, deleted)
//! has no rest pose to convert against. For the **A** end the honest answer is
//! still the authored `Transform` — *that is precisely what the seed will
//! convert* — so this door reads and writes it there, which is exactly what the
//! editor did before this module existed. For the **B** end there is no body and
//! therefore no anchor: the door says `None`, and the handle is not offered.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics::{PhysicsWorld, ShapeDesc};

use crate::joint::PhysicsJoint;

use super::{BodyRef, PhysicsBridge};

/// Which end of a joint an anchor gesture is talking about.
///
/// Not a `bool`: `set_joint_anchor_world(.., true, ..)` at a call site reads as
/// nothing at all, and the two ends behave differently enough (only A has a
/// display `Transform`) that the distinction has to be visible.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JointSide {
    /// The anchor on body A — the end the joint's `Transform` mirrors.
    A,
    /// The anchor on body B.
    B,
}

/// The pose a body was authored at — the triple the anchor conversions want.
fn rest_pose(b: &BodyRef) -> [f32; 3] {
    [b.rest.x, b.rest.y, b.rest.rotation]
}

impl PhysicsBridge {
    /// **Every entity carrying a [`PhysicsJoint`]**, as of the last reconcile —
    /// the list a canvas handle is offered for (W-J2b).
    ///
    /// ⚠️ Wider than `self.joints`, deliberately: a **dormant** joint (bodies
    /// half-authored, renamed, deleted) is seen here and is absent from there,
    /// and its A anchor is still authorable through
    /// [`Self::joint_anchor_world`]'s `Transform` fallback. Offering handles
    /// only for joints the solver accepted would hide exactly the joint the
    /// artist is in the middle of fixing.
    ///
    /// This is the reconcile's own scratch, published rather than re-queried:
    /// a second `World` query for the same question would need `&mut World`
    /// (the ECS builds query state on demand), and two enumerations of "which
    /// entities are joints" is how the handles come to describe a set the
    /// bridge is not holding.
    #[must_use]
    pub fn joint_entities(&self) -> &[Entity] {
        &self.joints_seen
    }

    /// The body (and the joint component) one side of a joint attaches to, or
    /// `None` if that side is not resolvable — a dormant joint, a renamed body,
    /// a joint the bridge has never built.
    fn joint_side_body(
        &self,
        sim: &SimWorld,
        joint: Entity,
        side: JointSide,
    ) -> Option<(&BodyRef, PhysicsJoint)> {
        let jr = self.joints.get(&joint)?;
        let component = *sim.world().get::<PhysicsJoint>(joint)?;
        // ⚠️ O lado B pode ser o MUNDO (W-JointWorld), e aí não há `BodyRef` a
        // devolver: a âncora de mundo não é um corpo da cena, é tralha na arena.
        // O `?` é a resposta certa — quem pergunta pelo lado B de um pino de
        // parede está perguntando por um corpo que não existe.
        let entity = match side {
            JointSide::A => jr.entities.0,
            JointSide::B => jr.entities.1?,
        };
        Some((self.bodies.get(&entity)?, component))
    }

    /// **Where this side's anchor is**, in world, as AUTHORED — `bodyX · local_x`
    /// against the body's REST pose.
    ///
    /// Rest, not live: this is the value a handle is drawn at and a gesture
    /// edits, and during play the solver's own anchors are what the overlay
    /// shows (they are the same number at rest, and only the overlay wants the
    /// live one).
    ///
    /// Falls back to the joint's `Transform` for the **A** end of a joint that
    /// has not been seeded or whose bodies are not resolved — see the module
    /// docs: that is the value the seed itself will convert, so the fallback is
    /// the same fact and not a second one.
    #[must_use]
    pub fn joint_anchor_world(
        &self,
        sim: &SimWorld,
        joint: Entity,
        side: JointSide,
    ) -> Option<[f32; 2]> {
        if let Some((body, component)) = self.joint_side_body(sim, joint, side)
            && component.anchored
        {
            let local = match side {
                JointSide::A => component.local_a,
                JointSide::B => component.local_b,
            };
            return Some(PhysicsWorld::world_from_local_at_pose(
                rest_pose(body),
                local,
            ));
        }
        match side {
            JointSide::A => {
                let t = sim.world().get::<Transform>(joint)?;
                Some([t.translation.x, t.translation.y])
            }
            JointSide::B => None,
        }
    }

    /// **Put this side's anchor at a world point.** Returns whether it landed.
    ///
    /// Converts against the body's **rest** pose — the same conversion the seed
    /// runs, for the same reason (an anchor derived against a swinging pose bakes
    /// the swing in; that cost 1.771 m of drift before W-AnchorFollow) — and
    /// writes the local straight into the component, leaving `anchored` alone
    /// (module docs).
    ///
    /// ⚠️ It writes the **local**, and nothing else. The A end's display pivot
    /// (the joint's `Transform`) is *derived*, and `sync_joint_pivots` already
    /// owns that derivation — writing it here too would make two functions
    /// responsible for one number, which is the shape a drift takes. The
    /// exception is the un-seeded case below, where the `Transform` **is** the
    /// authored state rather than a view of it.
    pub fn set_joint_anchor_world(
        &self,
        sim: &mut SimWorld,
        joint: Entity,
        side: JointSide,
        world: [f32; 2],
    ) -> bool {
        let resolved = self
            .joint_side_body(sim, joint, side)
            .filter(|(_, c)| c.anchored)
            .map(|(body, _)| PhysicsWorld::local_anchor_at_pose(rest_pose(body), world));
        match (side, resolved) {
            (JointSide::A, Some(local)) => {
                let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint) else {
                    return false;
                };
                j.local_a = local;
                true
            }
            (JointSide::B, Some(local)) => {
                let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint) else {
                    return false;
                };
                j.local_b = local;
                true
            }
            // Un-seeded / unresolved: the A end is still authorable as the world
            // `Transform` the seed will read. The B end is not — there is no body.
            (JointSide::A, None) => {
                if sim.world().get::<PhysicsJoint>(joint).is_none() {
                    return false;
                }
                write_pivot(sim, joint, world)
            }
            (JointSide::B, None) => false,
        }
    }

    /// **The points this side's anchor can snap to**, in world — the collider's
    /// centre and extremes ([`ShapeDesc::snap_points`]), placed by the body's
    /// rest pose and its collider offset. Returns how many were written.
    ///
    /// Reads the body's `rest` description, so it is the **resolved** collider:
    /// the scaled shape (W6) at the authored offset (W-Offset), which is the one
    /// the solver actually collides with and the one the outline draws.
    #[must_use]
    pub fn joint_snap_targets(
        &self,
        sim: &SimWorld,
        joint: Entity,
        side: JointSide,
        out: &mut [[f32; 2]; ShapeDesc::MAX_SNAP_POINTS],
    ) -> usize {
        let Some((body, _)) = self.joint_side_body(sim, joint, side) else {
            return 0;
        };
        Self::body_snap_targets(body, out)
    }

    /// **Os pontos a que o EIXO de uma roldana montada pode colar** (W-Pulley
    /// W6) — os do collider do corpo que a carrega, em mundo.
    ///
    /// Irmã exata da [`Self::joint_snap_targets`], e a razão de existir é que a
    /// pergunta *"de quem é o corpo?"* tem duas respostas diferentes: um joint a
    /// responde pela ponta (`JointSide`), uma roldana pelo `PulleyWheel::body` —
    /// o NOME que ela cita, resolvido pela mesma chave do reconcile. **A
    /// COLOCAÇÃO é uma só** ([`Self::body_snap_targets`]): duas cópias colariam
    /// o pino e o eixo em pontos diferentes do MESMO collider, e nada na tela
    /// diria qual dos dois está errado.
    ///
    /// ⚠️ **`0` para uma roldana de CENÁRIO, e isso não é falha:** ela não
    /// pertence a corpo nenhum, logo não há collider e não há candidato — a
    /// isenção que o gesto de arrasto trazia continua correta *para ela*. O que
    /// o W3 falsificou foi a generalização: uma roldana MONTADA tem corpo, e o
    /// corpo tem os nove pontos.
    #[must_use]
    pub fn wheel_snap_targets(
        &self,
        sim: &SimWorld,
        wheel: Entity,
        out: &mut [[f32; 2]; ShapeDesc::MAX_SNAP_POINTS],
    ) -> usize {
        let Some(w) = sim.world().get::<crate::PulleyWheel>(wheel) else {
            return 0;
        };
        if w.body == 0 {
            return 0;
        }
        let Some(body) = self.names.get(&w.body).and_then(|e| self.bodies.get(e)) else {
            return 0;
        };
        Self::body_snap_targets(body, out)
    }

    /// **A colocação dos nove pontos** — a resposta única que as duas portas
    /// acima compartilham.
    ///
    /// Lê a descrição de REPOUSO do corpo, então é o collider **resolvido**: a
    /// forma escalada (W6) no offset autorado (W-Offset), que é a que o solver de
    /// fato colide e a que o contorno desenha.
    fn body_snap_targets(
        body: &BodyRef,
        out: &mut [[f32; 2]; ShapeDesc::MAX_SNAP_POINTS],
    ) -> usize {
        let n = body.rest.shape.snap_points(out);
        let pose = rest_pose(body);
        let [ox, oy] = body.rest.offset;
        for slot in out.iter_mut().take(n) {
            *slot = PhysicsWorld::world_from_local_at_pose(pose, [slot[0] + ox, slot[1] + oy]);
        }
        n
    }

    // ---------------------------------------------------------------------
    // O lado ESCRITA da mesma pergunta.
    //
    // ⚠️ `sync_joint_pivots` mudou-se de `bridge::joints` para cá (W-JointWorld)
    // porque é isto que ele é: o `joint_anchor_world` acima responde *onde está
    // a âncora deste lado?* e ele PUBLICA essa resposta no `Transform`. No
    // reconcile ele ficava ao lado de *o que o artista autorou vira que
    // restrição?*, que é outra pergunta — e foi o cap de LOC que pediu a hora.
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
            for (&e, j) in self.joints.iter() {
                // ⚠️ **Um pino de MUNDO é o caso INVERSO, e sem esta linha ele
                // seria impossível de autorar:** aqui o `Transform` do joint **É**
                // a fonte — a âncora é um ponto do cenário e quem a segue é o
                // CORPO. Reescrevê-lo a partir de `bodyA · local_a` desfaria o
                // arrasto do dot no frame seguinte, e o artista veria a alça
                // voltar sozinha (o defeito exato que a W-J6-A curou no eixo de
                // uma roldana montada).
                if j.world_anchor.is_some() {
                    continue;
                }
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
}

/// Write the joint's display pivot (its `Transform.translation`), reporting
/// whether the entity had one.
fn write_pivot(sim: &mut SimWorld, joint: Entity, world: [f32; 2]) -> bool {
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(joint) else {
        return false;
    };
    t.translation.x = world[0];
    t.translation.y = world[1];
    true
}
