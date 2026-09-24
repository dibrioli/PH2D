//! **A PONTE do mover de VISTA DE CIMA** (TOP-20 #13) — a lei pura devolve um
//! plano, e aqui ele é gasto contra o mundo.
//!
//! Plano: `docs/Components/10_plano_topdown_player.md`. A lei:
//! [`ph2d_topdown`], com o corpus do oráculo (Godot MIT, `motion_mode = FLOATING`)
//! a defendê-la.
//!
//! # ⭐⭐⭐ O que esta ponte faz que a do platformer não faz
//!
//! Ela chama o controlador **mais do que uma vez por tique**. É a lei do
//! ORÇAMENTO: o que não coube na direcção pedida é re-emitido na tangente com o
//! resto intacto, e isso obriga a perguntar *«e a partir DAQUI, quanto cabe?»*.
//! ⛔ Escrever a pose entre as perguntas não serve (sem um `step()` pelo meio o
//! colisor fica com a pose velha), e um `step()` por deslize seria simular o
//! mundo quatro vezes por quadro ⇒ a porta é o
//! [`PhysicsWorld::move_character_from`], que desloca a pose do **cast**.
//!
//! # ⚠️ A ordem dentro do tique, e por que ela é esta
//!
//! `intenção → velocidade → plano → mundo → pose → rotação`. A rotação é a
//! ÚLTIMA e lê a direcção de **mundo** (depois do viewpoint): o desenho está no
//! ecrã, não no tabuleiro.
//!
//! # ⚠️ `max_slope_deg = 0` é o que faz TODA superfície ser parede
//!
//! É o `motion_mode = FLOATING` do oráculo escrito no vocabulário da casa: numa
//! vista de cima não há «cima», logo não há chão — há obstáculo. ⛔ Com o limite
//! de rampa do platformer, o controlador classificaria metade das paredes como
//! chão e **absorveria** o movimento contra elas.

use super::*;
use crate::components::{PlatformPlayer, TopDownPlayer};
use ph2d_physics::{CharacterHit, CharacterParams, RigidBodyHandle};
use ph2d_topdown::{TopDownState, intent, rotation, slide};

/// O que um player de vista de cima deve ao mundo neste tique — colhido com
/// `&self` e aplicado com `&mut self`, como todas as listas desta ponte.
struct TopDownMove {
    entity: Entity,
    handle: RigidBodyHandle,
    /// O deslocamento que o plano inteiro conseguiu.
    moved: [f32; 2],
    /// **A memória inteira depois deste tique** — a velocidade das rampas e quem
    /// mandava.
    ///
    /// ⚠️ **O struct inteiro e não só a velocidade:** desde a ordem do dono de
    /// 2026-09-15 (*«no 4 dir a última seta manda»*) a memória tem duas metades, e
    /// carregar só uma faria a dominância nascer de novo a cada tique — o boneco
    /// nunca trocaria de eixo. *Levar o TIPO é o que impede a próxima metade de
    /// ser esquecida.*
    state: TopDownState,
    /// O ângulo novo, se este modo roda.
    facing: Option<f32>,
}

impl PhysicsBridge {
    /// **Um tique do mover de vista de cima.** Ver o cabeçalho do módulo.
    pub(super) fn drive_topdown(&mut self, sim: &SimWorld) {
        let world = sim.world();
        let dt = self.world.dt();
        if !(dt.is_finite() && dt > 0.0) {
            return;
        }
        let mut planos: Vec<TopDownMove> = Vec::new();
        // ⚠️ **Um buffer para o laço inteiro** — a porta limpa-o a cada chamada.
        let mut hits: Vec<CharacterHit> = Vec::new();

        // ── COLHER (o cast toma `&self`) ─────────────────────────────────────
        // A ordem é a do `BTreeMap` de corpos: determinística cross-OS, a lei do
        // módulo.
        for (&entity, b) in self.bodies.iter() {
            let handle = b.handle;
            // ⚠️ **A CAMADA é a do corpo, nunca um zero escrito à mão** — o filtro
            // do cast é `groups_for(layer, matriz)`, e um literal aqui faria o
            // controlador consultar a máscara de OUTRA camada: o corpo atravessa
            // o cenário com `hits = 0` e o gate mais próximo não diz nada, porque
            // «andou o orçamento inteiro» é exactamente o que esta wave quer ler.
            let layer = b.rest.layer;
            let Some(cfg) = world.get::<TopDownPlayer>(entity) else {
                continue;
            };
            // ⚠️ **Dois movers na mesma entidade: o de plataforma GANHA, e este
            // cala-se.** É a lei transversal 2 da síntese (*«um dono do transform
            // por vez»*) com a falha do lado SEGURO — o comportamento indefinido
            // seria os dois escreverem a pose no mesmo tique. Quem AVISA o artista
            // é o Inspector; aqui só se decide, e decide-se sempre igual.
            if world.get::<PlatformPlayer>(entity).is_some() {
                continue;
            }
            let law = cfg.law();
            let entrada = self.player_input.get(&entity).copied().unwrap_or_default();
            let bruto = [entrada.drive, entrada.drive_y];

            // ⚠️ **A memória é lida INTEIRA e avança AQUI**, uma vez por tique e por
            // corpo: o `world_direction` observa a transição das setas para saber
            // qual chegou por último (ordem do dono, 2026-09-15). Observá-la duas
            // vezes come a transição, e a seta nova deixa de roubar o comando.
            let mut st: TopDownState = self.topdown_state.get(&entity).copied().unwrap_or_default();
            let dir = ph2d_topdown::world_direction(bruto, &law, &mut st.dominance);

            let v = intent::advance(
                st.velocity,
                dir,
                law.speed,
                law.acceleration,
                law.deceleration,
                dt,
            );
            st.velocity = v;

            let params = CharacterParams {
                // Ver o cabeçalho: numa vista de cima não há chão.
                up: [0.0, 1.0],
                snap_distance: 0.0,
                max_slope_deg: 0.0,
                step_height: 0.0,
            };
            let mut andado = [0.0_f32, 0.0];
            let mut passo = slide::first_step(v, dt, law.max_slides);
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
                let coube = ph2d_topdown::len(got.translation);
                if diagnostico() {
                    eprintln!(
                        "TD dir=({:.4},{:.4}) budget={:.5} got=({:.6},{:.6}) hits={} restam={}",
                        s.dir[0],
                        s.dir[1],
                        s.budget,
                        got.translation[0],
                        got.translation[1],
                        hits.len(),
                        s.steps_left
                    );
                }
                // ⚠️ **A normal é a do PRIMEIRO contacto, com o sinal normalizado
                // para se OPOR ao movimento** — a lei não pode adivinhar de que
                // lado o `normal1` da `rapier` aponta, e uma lei que adivinhasse
                // deslizaria **para dentro** da parede metade das vezes.
                let Some(n) = primeira_normal_oposta(&hits, s.dir) else {
                    break;
                };
                passo = slide::next_step(s, coube, n, law.min_slide_angle_deg);
            }

            let facing = if matches!(law.rotation, rotation::RotationMode::None) {
                None
            } else {
                let atual = self
                    .world
                    .body_pose(handle)
                    .map_or(0.0, |p| p.rotation.angle());
                Some(rotation::rotate_toward(
                    atual,
                    dir,
                    law.rotation,
                    law.rotation_speed_deg,
                    dt,
                ))
            };

            planos.push(TopDownMove {
                entity,
                handle,
                moved: andado,
                state: st,
                facing,
            });
        }

        // ── APLICAR (escrever a pose toma `&mut self`) ───────────────────────
        for m in planos {
            if let Some(pose) = self.world.body_pose(m.handle) {
                // ⚠️ **`set_next_kinematic_pose`, nunca uma escrita directa:** é ela
                // que faz o solver tratar o corpo como MOVENDO-SE, e é isso que o
                // faz empurrar o que toca em vez de o atravessar (o mesmo idioma
                // do `apply_kin_moves`).
                self.world.set_next_kinematic_pose(
                    m.handle,
                    pose.translation.x + m.moved[0],
                    pose.translation.y + m.moved[1],
                    m.facing.unwrap_or_else(|| pose.rotation.angle()),
                );
            }
            // ⚠️ **O que o mundo NÃO deixou acontecer não volta como velocidade
            // aqui**, ao contrário do `kinematic_settle` do irmão: ali a
            // velocidade é a do SOLVER e uma parede tem de a matar, e aqui ela é
            // a INTENÇÃO com rampa — matá-la contra a parede faria o corpo perder
            // a embalagem que o deslize acabou de preservar, que é a lei desta
            // wave ao contrário.
            self.topdown_state.insert(m.entity, m.state);
        }
    }
}

/// **`PH2D_TOPDOWN_LOG=1`** — imprime cada passo do plano de deslize.
///
/// ⚠️ A pergunta é lida **uma vez**: um `var_os` por iteração vive no laço que
/// corre por jogador por tique, e um diagnóstico não pode ser um custo.
///
/// ⭐ Ele ganhou o seu lugar: foram precisas QUATRO rondas para achar que uma
/// parede de meia-altura `20 000` faz o cast devolver `hits = 0` e o corpo
/// atravessar o cenário, e a tabela lia-se como um resultado.
fn diagnostico() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| std::env::var_os("PH2D_TOPDOWN_LOG").is_some())
}

#[cfg(test)]
#[path = "topdown_tests.rs"]
mod tests;
