//! ⭐⭐⭐ **A cena `=3` CONTÉM o fenómeno** — jogada inteira, sem ecrã, pela ordem do quadro da shell
//! (rotas e tweens · relógios · a marca dos moldes · o tique da física · os sinais · os cérebros · a
//! tabela · a entrega à ponte · as fábricas · o `settle`; parado no início, o rebobinar a cada quadro). ⚠️ *Um arnês que não corre o que o quadro corre não afirma nada sobre o
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
    registo: ph2d_ecs::scene::ComponentRegistry,
    mapa: ph2d_vec_entities::entities::VecEntityMap,
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
            registo: crate::component_registry_for_tests::registo(),
            mapa,
            g,
            tique: 0,
            dos_cerebros: Vec::new(),
        }
    }

    /// Um quadro com o relógio a ANDAR, com o herói a andar `para` (as setas).
    fn quadro(&mut self, para: [f32; 2]) -> Vec<String> {
        self.tique += 1;
        self.um_quadro(para, true)
    }

    /// Um quadro com o relógio PARADO no início (o que o Rewind deixa): a física segura o tique 0,
    /// as fábricas não nascem, e a porta do rebobinar corre no fim (a `fase_fabrica_e_morte`).
    fn quadro_parado(&mut self) -> Vec<String> {
        self.tique = 0;
        self.um_quadro([0.0, 0.0], false)
    }

    /// O quadro da shell. Devolve os sinais que correram.
    fn um_quadro(&mut self, para: [f32; 2], a_correr: bool) -> Vec<String> {
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
        // ⚠️ A marca de MESTRE antes da ponte (a família da física fá-lo): sem ela o molde da porta
        // seria uma parede no vão desde o início.
        ph2d_ecs::assign_master_pieces(self.sim.world_mut());
        self.ponte.dispatch(&mut self.sim, a_correr, self.tique);
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
        // As FÁBRICAS (só com o relógio a andar — a `fase_fabrica_e_morte`).
        if a_correr && !fila.is_empty() {
            let nomes: Vec<&str> = fila.iter().map(String::as_str).collect();
            let tick = ph2d_ecs::tick_factories(self.sim.world_mut(), &self.tags, &nomes);
            let mut docs = crate::instance_docs::OwnedDocs {
                vec_scene: &mut self.cena,
                vec_entities: &mut self.mapa,
            };
            crate::factory_bridge::apply_births(
                &mut self.sim,
                &self.registo,
                &mut docs,
                &tick.births,
                self.tique,
            );
        }
        // Parado no início: a porta do rebobinar, A CADA QUADRO (o invariante da shell).
        if !a_correr {
            crate::factory_bridge::sweep_spawned(&mut self.sim);
            ph2d_ecs::rewind_runtime::rewind_runtime_state(
                self.sim.world_mut(),
                ph2d_ecs::rewind_runtime::Renascimento::Rebobinar,
            );
        }
        self.drive.settle();
        fila
    }

    /// A porta que a fábrica fez nascer (`None` = o vão está aberto). ⚠️ Nesta cena só a porta
    /// nasce, e a cópia chama-se `Door (1)` (a fábrica numera-as).
    fn porta_nascida(&mut self) -> Option<Entity> {
        let w = self.sim.world_mut();
        let mut q = w.query::<(Entity, &ph2d_ecs::Spawned)>();
        q.iter(w).map(|(e, _)| e).next()
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
    assert!(
        j.porta_nascida().is_none(),
        "a cena abriu com a porta fechada"
    );

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
        fechou |= j.porta_nascida().is_some();
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
    assert!(fechou, "a porta nunca fechou");
    let nascida = j.porta_nascida().expect("a porta");
    assert_eq!(j.pos(nascida), j.pos(porta), "a porta nasceu fora do vão");
    assert!(desistiu, "a porta fechou e o guarda não desistiu");
    j.quadro([0.0, 0.0]);
    assert_eq!(
        j.ponte.nav_ordem(guarda).map(|o| o.alvo),
        Some(0),
        "ao desistir ele não voltou ao alvo autorado (a ronda)"
    );

    // (4) ⭐⭐ O REWIND (report do dono, 02/10): parado no início, a porta abre (o que nasceu é
    //     varrido), os cérebros voltam ao princípio — e o guarda anuncia a ronda UMA vez, não a
    //     cada quadro.
    let mut patrulhar = 0;
    for _ in 0..30 {
        patrulhar += j
            .quadro_parado()
            .iter()
            .filter(|n| n.as_str() == "patrulhar")
            .count();
    }
    assert!(j.porta_nascida().is_none(), "o Rewind não abriu a porta");
    assert_eq!(j.estado(porta), 0, "o cérebro da porta não voltou a «Open»");
    assert_eq!(j.estado(guarda), 0);
    assert!(
        patrulhar <= 1,
        "parado no início, o guarda anunciou a ronda {patrulhar} vezes em 30 quadros"
    );
    // E a corrida seguinte volta a fechá-la (a fábrica tem `total_max = 1` POR CORRIDA).
    let mut viu_outra_vez = false;
    for _ in 0..240 {
        j.quadro([-1.0, 0.0]);
        if j.estado(guarda) == 1 {
            viu_outra_vez = true;
            break;
        }
    }
    assert!(viu_outra_vez, "na 2.ª corrida o guarda não viu o herói");
}
