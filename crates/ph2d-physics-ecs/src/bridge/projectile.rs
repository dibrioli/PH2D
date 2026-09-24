//! **A PONTE do PROJÉCTIL** (TOP-20 #14) — a lei pura devolve um plano, e aqui ele é gasto contra
//! o mundo.
//!
//! Plano: `docs/Components/11_plano_projectile_motion.md`. A lei: [`ph2d_projectile`], com o corpus
//! do oráculo (Godot MIT, `move_and_collide` + `get_remainder` + `bounce`) a defendê-la.
//!
//! # ⭐⭐ O que esta ponte faz que a do mover de vista de cima não faz
//!
//! Ela **conta o que andou** e **mata o voo**. As duas coisas são a mesma: o alcance é medido em
//! metros **percorridos**, e é ele que separa este componente do `Lifetime` do #12, que mata por
//! TEMPO — *uma bala lenta e uma rápida com o mesmo tempo de vida têm alcances diferentes*.
//!
//! ⚠️ **E o que o mundo não deixou andar NÃO conta para o alcance:** uma bala barrada por uma
//! parede não percorreu nada, e cobrar-lhe alcance mata-a mais cedo por ter batido.
//!
//! # ⚠️ A ordem dentro do tique
//!
//! `lei → plano → mundo (com ricochetes) → pose → rotação → o voo acabou?`
//!
//! ⛔ A pergunta do fim vem **depois** do movimento, e não antes: uma bala que gasta o último metro
//! a chegar ao alvo tem de lá chegar.

use super::*;
use crate::components::ProjectileMotion;
use ph2d_physics::{CharacterHit, CharacterParams, RigidBodyHandle};
use ph2d_projectile::{Ended, ProjectileState, bounce};

/// O que um projéctil deve ao mundo neste tique.
struct ProjectileMove {
    entity: Entity,
    handle: RigidBodyHandle,
    /// O deslocamento que o plano inteiro conseguiu.
    moved: [f32; 2],
    /// **A memória inteira depois deste tique** — ⚠️ o TIPO, nunca um campo: ela tem três metades
    /// (velocidade, metros, saltos) e carregar duas seria um voo que nunca acaba.
    state: ProjectileState,
    /// O ângulo novo, se este projéctil vira.
    facing: Option<f32>,
    /// Por que o voo acabou, se acabou.
    ended: Option<Ended>,
}

impl PhysicsBridge {
    /// **Os projécteis cujo voo ACABOU neste dispatch**, com o motivo.
    ///
    /// ⚠️⚠️ **Quem lê isto NÃO pode apagar toda a gente.** Um projéctil que o artista pôs na cena
    /// à mão é **documento**: matá-lo por ter percorrido o alcance destruiria trabalho autorado.
    /// A porta que separa os dois é a [`ph2d_ecs::is_transient`] — *o que nasce numa corrida não é
    /// documento* —, e ela já tem dois leitores nesta casa. ⛔ Este é o terceiro, e não uma
    /// segunda resposta à mesma pergunta.
    ///
    /// ⭐ Um projéctil de documento cujo voo acabou **pára** e fica na cena; rebobinar devolve-lhe
    /// a pose autorada, como a todo corpo cinemático.
    #[must_use]
    pub fn projectile_done(&self) -> &[(Entity, ph2d_projectile::Ended)] {
        &self.projectile_done
    }

    /// **Os projécteis cujo voo ACABOU e que ainda estão na cena** — o ESTADO, não o evento.
    ///
    /// ⚠️⚠️ **Não confundir com [`Self::projectile_done`], e a diferença mordeu:** aquele é um
    /// **acontecimento** (*«morreu NESTE dispatch»*) e serve o dreno, que apaga a cópia uma vez;
    /// este é um **facto do mundo** (*«o voo acabou»*) e serve o Inspector, que o pinta enquanto
    /// for verdade. O Inspector lia o canal do evento, e por isso a etiqueta dependia de o
    /// relógio estar a andar: parado ela ficava (nada limpava o canal), a andar ela sumia no
    /// quadro seguinte. *Um evento lido como estado acerta pelo tempo que ninguém o apagar.*
    ///
    /// ⚠️ Iterador e não `Vec`: quem chama já tem o buffer dele (`PhysicsState::projectile_over`),
    /// e devolver uma lista nova alocaria uma por quadro.
    pub fn projectiles_finished(&self) -> impl Iterator<Item = Entity> + '_ {
        self.projectile_state
            .iter()
            .filter(|(_, st)| st.finished)
            .map(|(&e, _)| e)
    }

    /// **Esquece as mortes anunciadas** — irmão dos `discard_*` desta ponte.
    ///
    /// ⚠️⚠️ **O canal é do DISPATCH, e até 2026-09-15 ele era limpo por TIQUE** — dentro do
    /// `drive_projectiles`, ou seja uma vez por passo. Numa moldura que deve 3 tiques, uma morte
    /// no primeiro era apagada pelo segundo, e a bala transitória **nunca saía da cena**: é o
    /// irmão exacto do `accumulate_joint_breaks`, cujo doc já escreve a mesma frase sobre o mesmo
    /// laço. E um salto de relógio deixava-o de pé, a descrever uma corrida que acabou.
    pub(super) fn discard_projectile_deaths(&mut self) {
        self.projectile_done.clear();
    }

    /// **Um tique dos projécteis.** Ver o cabeçalho do módulo.
    pub(super) fn drive_projectiles(&mut self, sim: &SimWorld) {
        let world = sim.world();
        let dt = self.world.dt();
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }
        // ⛔ **A limpeza do canal NÃO mora aqui** — ela é do DISPATCH, e escrevê-la neste laço
        // fazia uma morte de um tique ser apagada pelo tique seguinte da mesma moldura. Ver
        // [`Self::discard_projectile_deaths`].
        let mut planos: Vec<ProjectileMove> = Vec::new();
        let mut hits: Vec<CharacterHit> = Vec::new();

        for (&entity, b) in self.bodies.iter() {
            let handle = b.handle;
            let layer = b.rest.layer;
            let Some(cfg) = world.get::<ProjectileMotion>(entity) else {
                continue;
            };
            let law = cfg.law();

            let pose = self.world.body_pose(handle);
            let posicao = pose.map_or([0.0, 0.0], |p| [p.translation.x, p.translation.y]);
            let angulo = pose.map_or(0.0, |p| p.rotation.angle());

            // ⚠️ **O alvo resolve-se pelo NOME**, com a porta que a casa já tem — os bits de uma
            // entidade não sobrevivem a um `Ctrl+Z`.
            let alvo = (cfg.homing_target != 0)
                .then(|| self.posicao_por_nome(sim, cfg.homing_target))
                .flatten();

            let mut st: ProjectileState = self
                .projectile_state
                .get(&entity)
                .copied()
                .unwrap_or_default();
            // ⭐ **Um voo acabado não anda mais.** ⚠️ Ele fica na lista de corpos (a entidade pode
            // ser documento, e aí ninguém a apaga) — o que acaba é o MOVIMENTO.
            if st.finished {
                continue;
            }
            let v = ph2d_projectile::advance(&mut st, &law, angulo, alvo, posicao, dt);

            let params = CharacterParams {
                // ⚠️ Um projéctil não tem chão: toda superfície é obstáculo (o mesmo
                // `motion_mode = FLOATING` que o irmão de vista de cima escreve).
                up: [0.0, 1.0],
                snap_distance: 0.0,
                max_slope_deg: 0.0,
                step_height: 0.0,
            };
            let mut andado = [0.0_f32, 0.0];
            let mut bateu = false;
            let mut passo = ph2d_projectile::first_step(v, dt, law.max_bounces);
            let mut velocidade = v;
            while let Some(s) = passo {
                let pedido = [s.dir[0] * s.budget, s.dir[1] * s.budget];
                let got = self
                    .world
                    .move_character_from(handle, andado, pedido, params, None, layer, &mut hits);
                // ⭐ E o que ele BATEU vira toque para a VIDA (plano 28 §8.1) — o mover pára rente
                // ao obstáculo, logo o solver nunca verá este par.
                self.toques_do_mover
                    .extend(hits.iter().filter_map(|h| h.body.map(|b| (entity, b))));
                andado = [
                    andado[0] + got.translation[0],
                    andado[1] + got.translation[1],
                ];
                let coube = ph2d_projectile::len(got.translation);
                let Some(n) = primeira_normal_oposta(&hits, s.dir) else {
                    break;
                };
                bateu = true;
                let Some(b) = bounce::next_step(s, velocidade, coube, n, &law) else {
                    break;
                };
                st.bounces_used = st.bounces_used.saturating_add(1);
                velocidade = b.velocity;
                passo = Some(b.step);
            }
            st.velocity = velocidade;
            // ⚠️ **O alcance conta o que ANDOU**, não o orçamento pedido.
            st.travelled += ph2d_projectile::len(andado);

            let facing = ph2d_projectile::facing_of(velocidade, &law);
            // ⚠️ **A pergunta do fim vem DEPOIS do movimento:** uma bala que gasta o último metro a
            // chegar ao alvo tem de lá chegar.
            let ended = ph2d_projectile::ended(&st, &law, bateu);
            if ended.is_some() {
                st.finished = true;
                // ⚠️ E a velocidade morre com o voo — senão um scrub para trás devolveria um corpo
                // parado com uma velocidade guardada, e ele arrancaria sozinho.
                st.velocity = [0.0, 0.0];
            }

            planos.push(ProjectileMove {
                entity,
                handle,
                moved: andado,
                state: st,
                facing,
                ended,
            });
        }

        // ── APLICAR ─────────────────────────────────────────────────────────
        for m in planos {
            if let Some(pose) = self.world.body_pose(m.handle) {
                self.world.set_next_kinematic_pose(
                    m.handle,
                    pose.translation.x + m.moved[0],
                    pose.translation.y + m.moved[1],
                    m.facing.unwrap_or_else(|| pose.rotation.angle()),
                );
            }
            self.projectile_state.insert(m.entity, m.state);
            if let Some(porque) = m.ended {
                // ⚠️ **A ponte não despacha a morte; ela ANUNCIA-A.** Quem apaga uma entidade é a
                // shell, no sítio onde o `Lifetime` do #12 já o faz — dois despachantes de morte
                // seriam duas respostas à pergunta *«quando é que isto sai da cena?»*.
                self.projectile_done.push((m.entity, porque));
            }
        }
    }

    /// A posição de mundo de quem tem este `stable_name_id` — `None` se ninguém tem.
    fn posicao_por_nome(&self, sim: &SimWorld, nome: u64) -> Option<[f32; 2]> {
        let world = sim.world();
        let mut q = world.try_query::<(Entity, &ph2d_ecs::Name)>()?;
        for (e, n) in q.iter(world) {
            if ph2d_ecs::stable_name_id(n.as_str()) == nome {
                let t = world.get::<ph2d_ecs::Transform>(e)?;
                return Some([t.translation.x, t.translation.y]);
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "projectile_tests.rs"]
mod tests;
