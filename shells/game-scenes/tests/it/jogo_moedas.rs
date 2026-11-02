//! ⭐ **J5 — as moedas montadas à mão com as peças** (GEF 21, `docs/GameEngineFuncional/21_moedas_plano.md`).
//!
//! A cadeia é a do app, pelas portas do produto: a física publica o `coin` da moeda em que o jogador
//! entra (`signal_events`), a tabela resolve-o (`resolve_signal_actions`) e a ponte aplica-o
//! (`signal_actions_bridge::apply`); quem o *Destroy* manda sair sai no fim do quadro, como no dreno da
//! `fase_fabrica_e_morte`.

use ph2d_app_game::smoke_moedas::{CONTADOR, PLACAR, SINAL};
use ph2d_ecs::{Name, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
use ph2d_script::LuauScript;

/// O que uma corrida para a direita deu: quem saiu, quantos sons se pediram e por quem, e o contador.
struct Corrida {
    sairam: Vec<String>,
    sons: Vec<String>,
    contador: Option<i64>,
    ficam: Vec<String>,
}

fn nome(sim: &SimWorld, e: ph2d_ecs::Entity) -> String {
    sim.world()
        .get::<Name>(e)
        .map_or_else(|| "?".into(), |n| n.as_str().to_owned())
}

fn moedas(sim: &mut SimWorld) -> Vec<String> {
    let w = sim.world_mut();
    let mut q = w.query::<&Name>();
    let mut v: Vec<String> = q
        .iter(w)
        .map(|n| n.as_str().to_owned())
        .filter(|n| n.starts_with("Coin "))
        .collect();
    v.sort();
    v
}

/// O nível 2 numa CÓPIA (o Play), com o jogador a correr para a direita pelo chão.
fn corre(ticks: u64) -> Corrida {
    let mut sim = SimWorld::new();
    let jogador = crate::comp::montar(&mut sim, 2);
    ph2d_app_game::marca_a_copia(sim.world_mut(), true);
    let tags = ph2d_tags::TagTree::new();
    let mut bridge = PhysicsBridge::new();
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let (mut sairam, mut sons) = (Vec::new(), Vec::new());
    for tick in 1..=ticks {
        let dedo = if tick > 30 { 1.0 } else { 0.0 };
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive: dedo,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        let eventos = bridge.signal_events(&sim, &tags);
        let disparos: Vec<ph2d_ecs::Disparo<'_>> = eventos
            .iter()
            .map(|ev| ph2d_ecs::Disparo {
                nome: &ev.name,
                quem: Some(ev.source),
                outro: Some(ev.other),
            })
            .collect();
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &tags, &disparos);
        let mut som = |s: &mut SimWorld, _q, alvo| {
            sons.push(nome(s, alvo));
            true
        };
        let r = ph2d_app_components::signal_actions_bridge::apply(
            &mut sim,
            &efeitos,
            tick as f64 / 60.0,
            &mut drive,
            &mut som,
        );
        for morte in r.mortes {
            sairam.push(nome(&sim, morte.entity));
            sim.world_mut().entity_mut(morte.entity).despawn();
        }
    }
    let contador = ph2d_ecs::counter::soma(sim.world(), CONTADOR, ph2d_ecs::counter::Ambito::Mundo);
    Corrida {
        sairam,
        sons,
        contador,
        ficam: moedas(&mut sim),
    }
}

/// ⭐⭐ **Correr pelo chão apanha as duas moedas do chão**: cada uma pede o SEU som, soma 1 ao HUD e sai
/// — UMA vez cada (E5). As das plataformas ficam (o jogador não saltou).
///
/// **Mutações que devem sangrar:** *From: Anyone* (uma moeda tocada leva as cinco) · tirar o *Trigger*
/// (o jogador pára na moeda e nada grita) · o *Add to Counter* para outro nome.
#[test]
fn correr_pelo_chao_apanha_as_duas_moedas_do_chao_uma_vez_cada() {
    let c = corre(240);
    assert_eq!(c.sairam, ["Coin 1", "Coin 2"], "sons {:?}", c.sons);
    assert_eq!(c.sons, ["Coin 1", "Coin 2"], "cada moeda toca o SEU som");
    assert_eq!(c.contador, Some(2), "o HUD conta duas");
    assert_eq!(c.ficam, ["Coin 3", "Coin 4", "Coin 5"]);
}

/// A moeda é feita SÓ de peças: nenhum script, e o HUD chama-se como a tabela o nomeia.
#[test]
fn a_moeda_nao_tem_script_e_o_hud_tem_o_contador() {
    let mut sim = SimWorld::new();
    crate::comp::montar(&mut sim, 2);
    let w = sim.world_mut();
    let mut q = w.query::<(
        &Name,
        Option<&LuauScript>,
        Option<&ph2d_physics_ecs::SignalOnHit>,
    )>();
    let mut moedas = 0;
    for (n, script, grita) in q.iter(w) {
        if n.as_str().starts_with("Coin ") {
            moedas += 1;
            assert!(script.is_none(), "{n:?} ainda tem script");
            assert_eq!(grita.map(|g| g.0.as_str()), Some(SINAL), "{n:?}");
        }
    }
    assert_eq!(moedas, 5);
    let mut q = w.query::<(&Name, &ph2d_ecs::Counter)>();
    let hud: Vec<_> = q
        .iter(w)
        .map(|(n, c)| (n.as_str().to_owned(), c.name.clone()))
        .collect();
    assert_eq!(hud, [(PLACAR.to_owned(), CONTADOR.to_owned())]);
}
