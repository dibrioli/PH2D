//! **O que se PERGUNTA à ponte** — os readouts, ao lado do que ela FAZ.
//!
//! Irmão de `bridge.rs`, separado dele quando ele passou do cap de 700 LOC, e o
//! corte é o mesmo que `world/tuning.rs` fez do outro lado da fronteira: tudo
//! aqui é leitura pura (quantos passos, onde está um corpo, quantos joints, onde
//! eles ancoram), nada aqui roda no `dispatch`.

use ph2d_ecs::Entity;

use super::PhysicsBridge;

impl PhysicsBridge {
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
    ///
    /// ⚠️ **Conta o mapa entidade→corpo da PONTE**, não a arena: um corpo sem
    /// entidade (a âncora de um pino de mundo, a tralha da mão) não está aqui.
    /// Quem quer o número da arena — um gate de vazamento, por exemplo — tem de
    /// perguntar a [`Self::arena_body_count`], senão o gate não pode falhar pelo
    /// motivo que alega.
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// **Quantos corpos a ARENA tem**, âncoras sem entidade incluídas.
    pub fn arena_body_count(&self) -> usize {
        self.world.arena_body_count()
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
}
