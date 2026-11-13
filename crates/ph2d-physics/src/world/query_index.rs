//! **As consultas vêem o mundo de AGORA, não o do último passo** (Bug #13, `docs/Physics/BUGS_physics.md`).
//!
//! As consultas ([`super::cast`], [`super::sweep`], [`super::character`]) descem o BVH do broad
//! phase, e o rapier só o actualiza dentro do `step`: um collider inserido (ou um corpo
//! teleportado) antes do passo não está nele. O player Snap nascido a 0,9 perguntava pelo chão no
//! tique 1, antes do primeiro passo, e o encaixe do nascimento não achava nada — assentava no 16.
//!
//! A cura é a do Box2D (a forma entra na árvore ao ser criada; `b2Body_SetTransform` move a
//! folha): quem muda a geometria fora do passo marca o índice ([`PhysicsWorld::mark_query_index`]),
//! e [`PhysicsWorld::refresh_query_index`] — um `bool` quando nada mudou — põe cada collider na
//! pose que o passo lhe daria (corpo · pose no corpo) e a folha dele pelo `BroadPhaseBvh::set_aabb`,
//! a porta que o rapier documenta para consultar sem passo («change detection accounts for it
//! during the next broad-phase update»). O passo seguinte recomputa tudo do mesmo jeito.
//!
//! ⚠️ **Não é chamado pelo `step` nem por quem insere** — mexer na árvore muda a ordem dos pares
//! do primeiro passo. Só o chamador que vai CONSULTAR fora do passo o pede (a ponte, à cabeça dos
//! controladores), e o mundo sem consulta fica byte-idêntico ao de antes.

use rapier2d::geometry::{Aabb, BoundingVolume};

use super::PhysicsWorld;

impl PhysicsWorld {
    /// A geometria mudou fora do passo: a próxima [`Self::refresh_query_index`] trabalha.
    pub(crate) fn mark_query_index(&mut self) {
        self.query_index_stale = true;
    }

    /// **O índice das consultas está atrás do mundo?** — `true` depois de inserir ou teleportar,
    /// `false` depois de um passo ou de um refresh.
    #[must_use]
    pub fn query_index_stale(&self) -> bool {
        self.query_index_stale
    }

    /// **Põe o índice das consultas no mundo de agora.** No-op (um `bool`) quando nada mudou desde
    /// o último passo ou refresh; senão uma passada pelos colliders na ordem da arena
    /// (determinista), e só as folhas que saíram da sua caixa gorda se mexem.
    pub fn refresh_query_index(&mut self) {
        if !self.query_index_stale {
            return;
        }
        self.query_index_stale = false;
        let half_prediction = self.integration_parameters.prediction_distance() / 2.0;
        let mut moved = std::mem::take(&mut self.query_index_scratch);
        moved.clear();
        for (h, c) in self.colliders.iter() {
            if !c.is_enabled() {
                continue;
            }
            let pose = match (
                c.parent().and_then(|b| self.bodies.get(b)),
                c.position_wrt_parent(),
            ) {
                (Some(rb), Some(local)) => *rb.position() * *local,
                _ => *c.position(),
            };
            let aabb: Aabb = c
                .shape()
                .compute_aabb(&pose)
                .loosened(c.contact_skin() + half_prediction);
            moved.push((h, pose, aabb));
        }
        for &(h, pose, aabb) in &moved {
            // A pose que o passo lhe daria: a consulta testa a forma em `position()`, não na folha.
            if let Some(c) = self.colliders.get_mut(h)
                && *c.position() != pose
            {
                c.set_position(pose);
            }
            self.broad_phase
                .set_aabb(&self.integration_parameters, h, aabb);
        }
        self.query_index_scratch = moved;
    }
}

#[cfg(test)]
mod tests {
    use super::PhysicsWorld;

    /// Sonda de custo: o refresh percorre todos os colliders quando um só se teleporta (o pior caso
    /// por tique), contra o passo do mesmo mundo; mínimo de 200 tiques intercalados.
    #[test]
    #[ignore = "sonda de custo"]
    fn measure_the_refresh_cost() {
        use std::time::Instant;
        for n in [100_usize, 1000, 5000] {
            let mut w = PhysicsWorld::new();
            let side = (n as f32).sqrt().ceil() as usize;
            for i in 0..n {
                w.add_static_cuboid((i % side) as f32 * 1.5, (i / side) as f32 * 1.5, 0.5, 0.5);
            }
            let (ball, _) = w.add_dynamic_circle(-5.0, 0.0, 0.2, 1.0);
            w.step();
            let (mut refresh, mut step) = (f64::MAX, f64::MAX);
            for k in 0..200 {
                w.set_body_pose(ball, -5.0, (k % 7) as f32, 0.0, true);
                let t = Instant::now();
                w.refresh_query_index();
                refresh = refresh.min(t.elapsed().as_secs_f64());
                let t = Instant::now();
                w.step();
                step = step.min(t.elapsed().as_secs_f64());
            }
            println!(
                "{n:>6} colliders: refresh {:.1} us, step {:.1} us",
                refresh * 1e6,
                step * 1e6
            );
        }
    }
}
