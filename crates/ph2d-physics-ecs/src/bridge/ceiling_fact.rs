//! **O TETO MEDIDO DEPOIS DO PASSO** (Bug #12b) — o `is_on_ceiling` do Godot lido dos contactos
//! que o passo produziu, ao lado da sonda de antes do passo.
//!
//! A sonda (`player_probes::probe_ceiling`) corre ANTES de a lei lançar, com a velocidade do
//! tique anterior: com o teto a cabeça + 0,05 o tique do pulo sonda com `rel_up` < 0 (não
//! pergunta), o solver encosta a cabeça no teto nesse MESMO tique, e no seguinte a subida já
//! morreu — ninguém perguntava. Aqui o facto é medido como o `CharacterBody` o mede: no fim do
//! tique, pelos contactos reais do corpo.
//!
//! - **Armar** ([`PhysicsBridge::arm_ceiling_fact`], antes do `step`, depois dos motores): a
//!   subida PRETENDIDA de cada player (a velocidade com o lançamento já aplicado), se a vista
//!   deste tique já acendeu o bit, e se a do tique anterior estava acesa.
//! - **Medir** ([`PhysicsBridge::measure_player_ceilings`], depois do `step`, pelos dois laços):
//!   um par do tique (`tick_contacts`, a união dos sub-passos) que EMPURROU o corpo para baixo,
//!   com a normal dentro do ângulo de teto (`walk.max_slope_deg`, o mesmo número que decide o
//!   chão), sobre uma subida pretendida > 0 — é uma batida. Sob Snap o corpo cinemático não gera
//!   contacto: o facto é o do próprio movimento (subida pedida e cortada contra uma normal de
//!   teto), colhido em `player_kinmove`.
//!
//! **Uma batida por subida, sem duplicar:** o bit acende na vista; o `Bonked` só sai se nem a
//! sonda deste tique nem a vista anterior já estavam acesas — a mesma borda do `events_between`.
//!
//! **Um só `speed`:** nos dois caminhos o `Bonked` leva a subida PRETENDIDA deste tique — a que
//! foi entregue ao solver (ou ao movimento) e que o teto cortou. O da sonda sai do
//! `events_between` com a subida da vista (antes do motor) e é recarimbado aqui ao armar.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics::RigidBodyHandle;
use ph2d_platformer::PlayerEvent;

use super::PhysicsBridge;
use crate::components::PlatformPlayer;

/// O que um player levou ao passo — armado antes do `step`, consumido depois dele.
#[derive(Copy, Clone, Debug)]
pub(super) struct CeilingWatch {
    entity: Entity,
    handle: RigidBodyHandle,
    /// A subida pretendida (m/s, eixo `UP`) com os motores deste tique já aplicados.
    rise: f32,
    /// A sonda deste tique já acendeu o bit (o `Bonked`, se devido, já saiu dela).
    lit_now: bool,
    /// A vista do tique anterior estava acesa — contacto contínuo, não batida nova.
    lit_before: bool,
    /// Sob Snap: o movimento cortou a subida contra uma normal de teto.
    kin_cut: bool,
}

/// A normal (para fora da superfície, para o corpo) é de TETO? `n·up ≤ −cos(max_slope)`.
fn faces_down(n_into_body: [f32; 2], max_slope_deg: f32) -> bool {
    // `libm`, nunca o `std`: o hash do c9 corre em três SOs.
    n_into_body[1] <= -libm::cosf(max_slope_deg.to_radians())
}

/// A linha de uma normal de SENTIDO não contratado (`CharacterHit::normal`) é de teto?
pub(super) fn is_ceiling_line(n: [f32; 2], max_slope_deg: f32) -> bool {
    faces_down(n, max_slope_deg) || faces_down([-n[0], -n[1]], max_slope_deg)
}

impl PhysicsBridge {
    /// **Arma a medição** — chamado por `apply_player_results` depois dos motores e dos movimentos
    /// cinemáticos e ANTES de a tabela de vistas ser reescrita (a anterior ainda é a do tique
    /// passado). `kin` = `(player Snap, subida pedida m/s, cortada contra um teto)`. O `Bonked`
    /// que a sonda deste tique publicou em `events` recebe a subida pretendida (um só critério).
    pub(super) fn arm_ceiling_fact(
        &mut self,
        views: &[(Entity, ph2d_platformer::PlayerView)],
        kin: &[(Entity, f32, bool)],
        events: &mut [(Entity, PlayerEvent)],
    ) {
        self.ceiling_watch.clear();
        for (entity, view) in views {
            let Some(b) = self.bodies.get(entity) else {
                continue;
            };
            // Sob Snap a subida pretendida é a do movimento pedido; dinâmico, a do corpo.
            let mine = kin.iter().find(|(e, ..)| e == entity);
            let kin_cut = mine.is_some_and(|&(_, _, cut)| cut);
            let rise = mine.map_or_else(
                || self.world.body_velocity(b.handle).map_or(0.0, |v| v[1]),
                |&(_, r, _)| r,
            );
            if view.ceiling {
                for (e, ev) in events.iter_mut() {
                    if let (true, PlayerEvent::Bonked { speed }) = (e == entity, ev) {
                        *speed = if rise.is_finite() { rise.max(0.0) } else { 0.0 };
                    }
                }
            }
            self.ceiling_watch.push(CeilingWatch {
                entity: *entity,
                handle: b.handle,
                rise,
                lit_now: view.ceiling,
                lit_before: self.player_views.get(entity).is_some_and(|v| v.ceiling),
                kin_cut,
            });
        }
    }

    /// **Mede o teto DEPOIS do passo** — ver o cabeçalho do módulo. Chamado pelos dois laços
    /// (`depois_do_passo`), e no replay também: a vista é a memória da borda seguinte.
    pub(super) fn measure_player_ceilings(&mut self, sim: &SimWorld) {
        let watch = std::mem::take(&mut self.ceiling_watch);
        for w in &watch {
            // NaN também sai: só uma subida finita e positiva bate.
            if w.lit_now || !w.rise.is_finite() || w.rise <= 0.0 {
                continue;
            }
            let Some(slope) = sim
                .world()
                .get::<PlatformPlayer>(w.entity)
                .map(|p| p.config().walk.max_slope_deg)
            else {
                continue;
            };
            let me = w.handle.into_raw_parts();
            let hit = w.kin_cut
                || self.world.tick_contacts().iter().any(|(key, s)| {
                    // A normal aponta de `body1` para `body2`; para o corpo é −n se ele é o 1.
                    let n = if key.0 == me {
                        [-s.normal[0], -s.normal[1]]
                    } else if key.1 == me {
                        s.normal
                    } else {
                        return false;
                    };
                    s.impact > 0.0 && faces_down(n, slope)
                });
            if !hit {
                continue;
            }
            if let Some(v) = self.player_views.get_mut(&w.entity) {
                v.ceiling = true;
            }
            if !w.lit_before {
                self.player_events
                    .push((w.entity, PlayerEvent::Bonked { speed: w.rise }));
            }
        }
        // A capacidade volta para o próximo tique.
        self.ceiling_watch = watch;
        self.ceiling_watch.clear();
    }
}
