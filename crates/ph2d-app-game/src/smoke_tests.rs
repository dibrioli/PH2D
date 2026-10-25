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
    assert_eq!(
        world.get::<Name>(jogador).map(Name::as_str),
        Some("Player")
    );
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
