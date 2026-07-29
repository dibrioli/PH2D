//! **O que UM TIQUE faz** — o laço de sub-passos de [`super::PhysicsWorld::step`].
//!
//! Irmão de `world.rs` pelo cap de LOC, e o corte é por responsabilidade: lá mora
//! o que o mundo É (os campos, os acessores, as portas de autoria), aqui o que ele
//! FAZ quando o relógio anda. Módulo FILHO, então os campos privados do pai
//! seguem alcançáveis — nada precisou virar `pub` para este corte.
//!
//! A ordem dentro do laço é significativa e cada passe diz o porquê no lugar:
//! mira cinemática · arrasto · zonas · atração · POLIAS · o pipeline do rapier ·
//! os dois livros-razão (contatos e joints) que só existem ENTRE os sub-passos.

use super::{PhysicsWorld, blast, contacts, drag, effector, joint_break, oneway, pulley};

impl PhysicsWorld {
    /// Advance one fixed step. Always uses
    /// [`PhysicsWorld::dt`] — never accept an external dt at the
    /// wrapper boundary (HR-5).
    pub fn step(&mut self) {
        // W-ImpactForce: the impact peak is the hardest a pair pushes at any
        // single sub-step, and it is gone by the time this returns. Start the tick
        // with an empty ledger and `max` into it after each sub-step below. Cleared
        // (capacity kept) rather than reallocated — the hot-path zero-alloc gate.
        self.contact_peaks.clear();
        // W-J7: the same shape, one field over — a joint's reaction also lives
        // between the sub-steps, and a break decided a frame late is a different
        // simulation. Capacity kept for the same zero-alloc reason.
        self.joint_peaks.clear();
        self.joint_breaks.clear();
        // W-Pulley W2: a mesma forma, uma família adiante — a carga de uma corda
        // vive ENTRE os sub-passos, e uma ruptura decidida um quadro depois é
        // outra simulação. Capacidade mantida (zero-alloc).
        self.pulley_tension.clear();
        self.pulley_axle.clear();
        self.pulley_breaks.clear();
        for sub in 0..self.substeps {
            // Kinematic bodies advance a SLICE of their tick per sub-step, for
            // the same reason the drag below is applied per sub-step: rapier
            // derives a kinematic body's velocity from how far it was told to
            // move THIS sub-step, so handing it the whole tick at once reports
            // a speed the body does not have. Empty unless something is
            // kinematic, which is what keeps this loop byte-identical for every
            // world that has none.
            let f = (sub + 1) as f32 / self.substeps as f32;
            for i in 0..self.kinematic_targets.len() {
                let (handle, start, target) = self.kinematic_targets[i];
                if let Some(b) = self.bodies.get_mut(handle) {
                    b.set_next_kinematic_position(Self::kinematic_slice(&start, &target, f));
                }
            }
            // Per SUBSTEP, not per tick: a force applied once per tick would be
            // wrong by the substep count.
            drag::apply(
                &mut self.bodies,
                &self.colliders,
                self.air_drag,
                self.integration_parameters.dt,
            );
            // Force zones (W-Area): a sensor's force, applied to whatever is inside
            // it. Per SUBSTEP for the same reason as the drag above. Empty in every
            // scene without a zone, which is what keeps this byte-identical.
            effector::apply(
                &mut self.bodies,
                &self.colliders,
                &self.narrow_phase,
                &self.effectors,
                self.gravity,
                self.integration_parameters.dt,
            );
            // The attract field (W-Hand): the artist holding the pointer down with
            // the Attract tool. Per SUBSTEP for the same reason as the two above,
            // and a no-op for every world nobody is poking.
            blast::apply_attract(
                &mut self.bodies,
                &self.attract,
                self.integration_parameters.dt,
            );
            // W-Pulley W3: onde as roldanas MONTADAS estão AGORA. Antes do passe,
            // e por SUB-PASSO, porque uma cadernal móvel se move junto com o corpo
            // que a carrega — geometria de um sub-passo atrás puxaria numa direção
            // que já não é a da corda. No-op para toda roldana pregada no cenário.
            pulley::refresh_mounts(&self.bodies, &mut self.pulley_wheels);
            // The pulleys (W-Pulley): a rope through two wheels, imposed as a
            // velocity projection. Per SUBSTEP for the same reason as the three
            // above, and a no-op for every world without one.
            pulley::apply(
                &mut self.bodies,
                &self.pulleys,
                &self.pulley_wheels,
                &mut super::rope_load::PulleyLedger {
                    payout: &mut self.pulley_payout,
                    tension: &mut self.pulley_tension,
                    axle: &mut self.pulley_axle,
                    broken_ropes: &mut self.pulley_broken_ropes,
                    broken_wheels: &mut self.pulley_broken_wheels,
                    breaks: &mut self.pulley_breaks,
                },
                &mut self.pulley_live,
                &mut self.route_scratch,
                self.integration_parameters.dt,
                self.pulley_bias,
                self.pulley_lag,
            );
            // The one-way platform hook. Installing it is byte-neutral for every scene
            // without a one-way collider: rapier only calls `modify_solver_contacts`
            // for pairs where a collider carries `MODIFY_SOLVER_CONTACTS`, which only
            // a one-way collider sets.
            let physics_hooks = oneway::OneWayHooks;
            let event_handler = ();
            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.bodies,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                &mut self.ccd_solver,
                &physics_hooks,
                &event_handler,
            );
            // W-ImpactForce: fold this sub-step's contact loads into the peak
            // ledger. Measured ≤ 1.93% of the HR-4 budget at 500 contact pairs
            // (`tests/measure_impact.rs`), so it is unconditional — a gating flag
            // that never toggled would be a dead flag. Empty for a scene where
            // nothing touches, which keeps it free there.
            contacts::accumulate_peaks(
                &self.narrow_phase,
                &self.colliders,
                &mut self.contact_peaks,
            );
            // W-J7: fold this sub-step's joint reactions into the peak ledger and
            // sever whatever just exceeded what it was told it could take. Inside
            // the loop, so the remaining sub-steps already run without the broken
            // joint. Empty for a world with no joints.
            let inv_dt = self.joint_impulse_to_force();
            joint_break::accumulate_and_break(
                &mut self.impulse_joints,
                &self.bodies,
                inv_dt,
                &mut self.joint_peaks,
                &mut self.joint_breaks,
            );
        }
        // The aim is spent. Retaining the capacity is why this is a field:
        // the next tick refills it without allocating (the zero-alloc gate on
        // the hot path measures exactly that).
        self.kinematic_targets.clear();
        self.step_count += 1;
    }
}
