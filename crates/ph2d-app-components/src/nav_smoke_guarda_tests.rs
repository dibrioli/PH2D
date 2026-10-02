//! ⭐⭐⭐ **A cena `=3` CONTÉM o fenómeno** — jogada inteira, sem ecrã, pela ordem do quadro da shell
//! (rotas e tweens · relógios · o tique da física · os sinais · os cérebros · a tabela · a entrega à
//! ponte · o `settle`). ⚠️ *Um arnês que não corre o que o quadro corre não afirma nada sobre o
//! quadro* (a lição do recomeço): cada uma das metades da corrente passa por aqui.

use super::*;
use crate::signal_actions_bridge::{Som, apply, entrega_a_fisica};
use ph2d_ecs::{Disparo, SignalEffect, StateMachineRuntime, resolve_signal_actions};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
use ph2d_preview_drive::PreviewDrive;
use ph2d_tags::TagTree;

const DT: f64 = 1.0 / 60.0;

struct Jogo {
    sim: SimWorld,
    cena: ph2d_vec_scene::VecScene,
    ponte: PhysicsBridge,
    drive: PreviewDrive,
    tags: TagTree,
    g: Guarda,
    tique: u64,
    /// O que os cérebros anunciaram no quadro ANTERIOR: na shell vai ao barramento, e os cérebros
    /// leem-no no quadro seguinte (`fase_signal_outbox` — *uma emissão chega a quem a escuta no
    /// quadro SEGUINTE*); a tabela lê-o no MESMO quadro.
    dos_cerebros: Vec<String>,
}

impl Jogo {
    fn novo() -> Self {
        let mut sim = SimWorld::new();
        let mut cena = ph2d_vec_scene::VecScene::new();
        let mut mapa = ph2d_vec_entities::entities::VecEntityMap::default();
        let g = monta_em(&mut sim, &mut cena, &mut mapa);
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        Self {
            sim,
            cena,
            ponte: PhysicsBridge::new(),
            drive: PreviewDrive::default(),
            tags: TagTree::default(),
            g,
            tique: 0,
            dos_cerebros: Vec::new(),
        }
    }

    /// Um quadro, com o herói a andar `para` (as setas). Devolve os sinais que correram.
    fn quadro(&mut self, para: [f32; 2]) -> Vec<String> {
        let caminhos = crate::path_follow_bridge::a_escrever(&mut self.sim, &self.cena);
        crate::tween_bridge::drive_tweens(&mut self.sim, &mut self.drive, &caminhos);
        ph2d_ecs::reconcile_timers(self.sim.world_mut());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let dt_us = (DT * 1e6) as u64;
        let mut fila: Vec<String> = Vec::new();
        {
            let mundo = self.sim.world_mut();
            let mut q = mundo.query::<(&ph2d_ecs::Timers, &mut ph2d_ecs::TimerRuntime)>();
            for (cfg, mut rt) in q.iter_mut(mundo) {
                for (t, st) in cfg.0.iter().zip(rt.0.iter_mut()) {
                    if ph2d_ecs::timer_advance(t, st, dt_us).fires > 0 && !t.signal.is_empty() {
                        fila.push(t.signal.clone());
                    }
                }
            }
        }
        self.ponte.set_player_input(
            self.g.heroi,
            PlayerInput {
                drive: para[0],
                drive_y: para[1],
                ..PlayerInput::default()
            },
        );
        self.tique += 1;
        self.ponte.dispatch(&mut self.sim, true, self.tique);
        fila.extend(
            self.ponte
                .signal_events(&self.sim, &self.tags)
                .into_iter()
                .map(|s| s.name),
        );
        let ouvidos: Vec<&str> = fila
            .iter()
            .chain(&self.dos_cerebros)
            .map(String::as_str)
            .collect();
        let anunciados: Vec<String> =
            crate::state_machine_tick::advance_machines(&mut self.sim, &ouvidos)
                .into_iter()
                .map(|s| s.name)
                .collect();
        fila.extend(anunciados.iter().cloned());
        self.dos_cerebros = anunciados;
        if !fila.is_empty() {
            let disparos: Vec<Disparo<'_>> = fila
                .iter()
                .map(|n| Disparo {
                    nome: n,
                    quem: None,
                    outro: None,
                })
                .collect();
            let efeitos: Vec<SignalEffect> =
                resolve_signal_actions(self.sim.world_mut(), &self.tags, &disparos);
            let mut mudo = |_: &mut SimWorld, _: Som, _: Entity| false;
            let r = apply(&mut self.sim, &efeitos, &mut self.drive, &mut mudo);
            entrega_a_fisica(r.pedidos_de_vida, r.pedidos_de_navegacao, &mut self.ponte);
        }
        self.drive.settle();
        fila
    }

    fn pos(&self, e: Entity) -> [f32; 2] {
        let t = self.sim.world().get::<Transform>(e).expect("o corpo");
        [t.translation.x, t.translation.y]
    }

    fn estado(&self, e: Entity) -> u8 {
        self.sim
            .world()
            .get::<StateMachineRuntime>(e)
            .map_or(u8::MAX, |r| r.current)
    }
}

#[test]
fn o_guarda_patrulha_ve_persegue_e_a_porta_fecha_se_lhe_na_cara() {
    let mut j = Jogo::novo();
    let (guarda, controlo, heroi, porta) = (j.g.guarda, j.g.controlo, j.g.heroi, j.g.porta);

    // (1) A RONDA: os dois dão a volta às formas desenhadas, e nenhum persegue ninguém.
    let mut pontos_vermelho = std::collections::BTreeSet::new();
    let mut pontos_cinzento = std::collections::BTreeSet::new();
    for _ in 0..600 {
        j.quadro([0.0, 0.0]);
        if let Some(r) = j.ponte.nav_ronda(guarda) {
            pontos_vermelho.insert(r.ponto);
        }
        if let Some(r) = j.ponte.nav_ronda(controlo) {
            pontos_cinzento.insert(r.ponto);
        }
    }
    assert_eq!(pontos_vermelho.len(), 4, "o guarda não deu a volta à ronda");
    assert_eq!(
        pontos_cinzento.len(),
        4,
        "o controlo não deu a volta à ronda"
    );
    assert_eq!(
        j.estado(guarda),
        0,
        "o guarda saiu da ronda sem ver ninguém"
    );

    // (2) O herói entra pela porta e pisa a zona amarela: o guarda VÊ-O e persegue-o.
    let mut viu = false;
    for _ in 0..240 {
        let s = j.quadro([-1.0, 0.0]);
        viu |= s.iter().any(|n| n == "viu_heroi");
        if j.estado(guarda) == 1 {
            break;
        }
    }
    assert!(
        viu,
        "a zona amarela nunca viu o herói (em {:?})",
        j.pos(heroi)
    );
    assert_eq!(j.estado(guarda), 1, "o guarda viu e não passou a perseguir");
    j.quadro([1.0, 0.0]);
    assert_eq!(
        j.ponte.nav_ordem(guarda).map(|o| o.alvo),
        Some(stable_name_id("Hero")),
        "a perseguição não pôs o guarda atrás do herói"
    );
    assert_eq!(
        j.ponte.nav_ordem(controlo),
        None,
        "o CONTROLO ouviu o cérebro do outro"
    );

    // (3) O herói foge para a zona verde: a porta fecha-se atrás dele, na cara do guarda, e o guarda
    //     desiste e volta à ronda.
    let mut fechou = false;
    let mut desistiu = false;
    for _ in 0..480 {
        let para = if j.pos(heroi)[0] < 4.5 { 1.0 } else { 0.0 };
        j.quadro([para, 0.0]);
        fechou |= (j.pos(porta)[1] - Y_PORTA_FECHADA).abs() < 1e-3;
        assert!(
            j.pos(guarda)[0] < X_DIVISAO,
            "o guarda passou a porta: {:?}",
            j.pos(guarda)
        );
        if fechou && j.estado(guarda) == 0 {
            desistiu = true;
            break;
        }
    }
    assert!(fechou, "a porta nunca fechou (está em {:?})", j.pos(porta));
    assert!(desistiu, "a porta fechou e o guarda não desistiu");
    j.quadro([0.0, 0.0]);
    assert_eq!(
        j.ponte.nav_ordem(guarda).map(|o| o.alvo),
        Some(0),
        "ao desistir ele não voltou ao alvo autorado (a ronda)"
    );
    // O mapa mudou: a área andável perdeu o vão.
    let fecho = j.ponte.nav_meshes().map(|(_, _, m)| m.area()).sum::<f64>();
    assert!(fecho > 0.0);
}
