use super::*;

#[test]
fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE").then(|| v.to_owned());
    assert_eq!(armed_level_in(com("1")), Some(1));
    assert_eq!(armed_level_in(com("2")), None);
    assert_eq!(armed_level_in(com("x")), None);
    assert_eq!(armed_level_in(|_| None), None);
}

#[test]
fn o_condutor_joga_corre_e_para_por_esta_ordem() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| v.to_owned());
    assert_eq!(conducao_in(com("window")), Some(GameStage::Window));
    assert_eq!(conducao_in(com("fullscreen")), Some(GameStage::Fullscreen));
    assert_eq!(conducao_in(com("editor")), Some(GameStage::Editor));
    assert_eq!(conducao_in(com("x")), None);
    let s = GameStage::Fullscreen;
    let g: Vec<_> = (0..1000).filter_map(|q| gesto_no_quadro(s, q)).collect();
    assert_eq!(
        g,
        [
            Gesto::Joga(s),
            Gesto::Direita(true),
            Gesto::Direita(false),
            Gesto::Para
        ]
    );
}

/// A cena tem o que o smoke manda olhar: o jogador, a câmara que o segue e as moedas com script.
#[test]
fn o_nivel_tem_jogador_camara_e_moedas_com_script() {
    let mut world = ph2d_ecs::World::new();
    let jogador = montar(&mut world, 1);
    assert_eq!(world.get::<Name>(jogador).map(Name::as_str), Some("Player"));
    let mut q = world.query::<(&Name, Option<&LuauScript>, Option<&CameraFollow>)>();
    let (mut moedas, mut seguem) = (0, 0);
    for (n, s, f) in q.iter(&world) {
        if n.as_str().starts_with("Coin") {
            assert!(s.is_some(), "{} sem script", n.as_str());
            moedas += 1;
        }
        if f.is_some_and(|f| f.target == "Player") {
            seguem += 1;
        }
    }
    assert_eq!((moedas, seguem), (MOEDAS.len(), 1));
    assert!(
        world
            .resource::<ProjectScripts>()
            .0
            .by_name(MOEDA_NAME)
            .is_some()
    );
}

/// ⭐ A moeda SOME quando o jogador lhe chega: a física paira-o e corre-o para a direita (o horário do
/// condutor), os scripts correm a cada passo, e as duas primeiras moedas saem. A corrida é sobre uma
/// cópia (R0a), por isso apagar um objecto que o artista pôs é do jogo — o Stop devolve-o.
#[test]
fn a_moeda_some_quando_o_jogador_lhe_chega() {
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};
    let mut sim = ph2d_ecs::SimWorld::new();
    let jogador = montar(sim.world_mut(), 1);
    let moeda = |sim: &mut ph2d_ecs::SimWorld, nome: &str| {
        let w = sim.world_mut();
        let mut q = w.query::<&Name>();
        q.iter(w).any(|n| n.as_str() == nome)
    };
    crate::marca_a_copia(sim.world_mut(), true);
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    let mut bridge = PhysicsBridge::new();
    let (mut caiu, mut sumiram) = (Vec::new(), Vec::new());
    for tick in 1..=240_u64 {
        let drive = if tick > 60 { 1.0 } else { 0.0 };
        bridge.set_player_input(
            jogador,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(&mut sim, 1.0 / 60.0);
        for e in r.despawned {
            sim.world_mut().entity_mut(e).despawn();
            sumiram.push(tick);
        }
        if tick == 60 {
            caiu.push(
                sim.world()
                    .get::<Transform>(jogador)
                    .expect("pose")
                    .translation
                    .y,
            );
        }
    }
    let consola: Vec<String> = host.drain_console().into_iter().map(|l| l.text).collect();
    // A altura a que paira (o topo do chão é y = 0): o alcance de 0,6 m tem de a cobrir.
    assert!((0.7..1.1).contains(&caiu[0]), "paira a {caiu:?}");
    assert!(
        !moeda(&mut sim, "Coin 1") && !moeda(&mut sim, "Coin 2"),
        "{consola:?}"
    );
    assert!(moeda(&mut sim, "Coin 3"), "a da plataforma fica");
    assert_eq!(sumiram.len(), 2, "{consola:?}");
}

/// Sem cópia (o transporte da timeline a tocar), a moeda do artista fica — e a consola diz porquê.
#[test]
fn sem_copia_a_moeda_do_artista_fica_e_a_consola_diz_porque() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let jogador = montar(sim.world_mut(), 1);
    crate::marca_a_copia(sim.world_mut(), false);
    let mut host = ph2d_script::ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    sim.world_mut()
        .get_mut::<Transform>(jogador)
        .expect("pose")
        .translation = Vec2::new(-4.0, 0.9);
    let r = host.scene_tick(&mut sim, 1.0 / 60.0);
    assert!(r.despawned.is_empty());
    assert!(
        host.drain_console()
            .iter()
            .any(|l| l.text.contains("ph2d.despawn: 'Coin 1'"))
    );
}
