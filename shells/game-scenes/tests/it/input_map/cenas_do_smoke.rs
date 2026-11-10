//! As cenas de smoke da aba (`PH2D_GAME_SMOKE` 6..13, plano §8): o código que o roteiro do jogo dá é o mesmo que a
//! família arma, cada cena põe a aba no estado da maquete, e a cena tem os leitores do fire e do laser (o reload não).

use super::Mesa;
use ph2d_app_game::smoke::input_map::{CODIGOS as CENAS, DEAD, PRIMEIRA, arma, codigo, leitores};
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::Key;
use ph2d_input::readers::Reader;

/// A composição da shell (`janela_do_jogo.rs::game_smoke`): arma a aba, instala o mapa e pede a aba.
fn armada(cena: &str) -> Mesa {
    let mut m = Mesa::new(View::Keyboard);
    let base = std::env::temp_dir().join(format!("ph2d-cenas-{}-{cena}", std::process::id()));
    m.tab_mut().doc.tryit.base = Some(base); // a C4 grava o ficheiro do jogador: nunca no HOME de quem corre
    let map = arma(&mut m.tab_mut().doc, cena).expect("código do roteiro");
    m.hero.input_map = map;
    m.hero.store.request_input_map_tab();
    m.frame();
    m
}

fn id(m: &Mesa, n: &str) -> ph2d_input::ActionId {
    m.hero.input_map.id(n).expect("a acção do mapa do dono")
}

#[test]
fn as_cenas_do_input_map_fecham_o_roteiro_pela_ordem() {
    let todas = ph2d_app_game::smoke::CENAS;
    assert_eq!(codigo(PRIMEIRA - 1), None);
    assert_eq!(codigo(PRIMEIRA), Some("A1"));
    assert_eq!(codigo(todas), CENAS.last().copied());
    assert_eq!(codigo(todas + 1), None);
    assert!(arma(&mut Mesa::new(View::List).tab_mut().doc, "Z9").is_none());
}

#[test]
fn cada_cena_abre_a_aba_na_vista_dela_com_as_11_accoes() {
    for c in CENAS {
        let m = armada(c);
        assert!(m.active(), "{c}: a aba aberta e à frente");
        assert_eq!(m.hero.input_map.len(), 11, "{c}");
        let want = match c.as_bytes()[0] {
            b'A' => View::List,
            b'B' => View::Keyboard,
            _ => View::TryIt,
        };
        assert_eq!(m.tab().doc.view(), want, "{c}");
    }
}

#[test]
fn as_cenas_do_try_it_e_do_play_poem_a_tecla_a_manche_e_a_tela_do_jogador() {
    use ph2d_app_game::smoke::input_map::{
        GAME_ID, MANCHE_C3, NA_TELA, abre_a_aba, manche, tecla_do_p1, troca_do_jogador,
    };
    let up = Key::named_in_script("up").and_then(|v| v.first().copied());
    let m = armada("C1");
    assert_eq!(
        m.tab().doc.tryit.confirm,
        None,
        "C1: as receitas, sem pergunta"
    );
    let m = armada("C2");
    assert_eq!(
        m.tab().doc.tryit.held_keys(),
        up.into_iter().collect::<Vec<_>>()
    );
    let c3 = PRIMEIRA + 10;
    assert_eq!(codigo(c3), Some("C3"));
    let (eixo, v) = manche(c3).expect("C3 empurra a manche");
    assert!(eixo == ph2d_input::GamepadAxis::LeftStickX && (v - MANCHE_C3).abs() < 1e-6);
    assert!(manche(c3 - 1).is_none() && manche(c3 + 1).is_none());
    assert!(
        armada("C3")
            .hero
            .input_map
            .actions()
            .iter()
            .any(|a| a.is_analog())
    );
    for c in ["C4", "P1"] {
        let m = armada(c);
        let map = &m.hero.input_map;
        assert!(map.controls().players_can_change_keys, "{c}");
        let vistas: Vec<_> = (map.actions().iter())
            .filter(|a| map.controls().shown_in_game(a.id))
            .map(|a| a.name.as_str())
            .collect();
        assert_eq!(vistas, NA_TELA, "{c}");
    }
    let m = armada("C4");
    assert_eq!(
        troca_do_jogador(&m.hero.input_map).changed(&m.hero.input_map),
        1
    );
    let mut sim = ph2d_ecs::SimWorld::new();
    let _ = leitores(&mut sim, PRIMEIRA + 11, |s| s.world_mut().spawn(()).id());
    let id = sim
        .world()
        .get_resource::<ph2d_app_input_map::ph2d_player_controls::GameId>();
    assert_eq!(id.map(|g| g.0), Some(GAME_ID));
    let p1 = PRIMEIRA + 12;
    assert!(!abre_a_aba("P1") && abre_a_aba("C4"));
    let teclas: Vec<_> = (0..400).filter_map(|q| tecla_do_p1(p1, q)).collect();
    assert_eq!(teclas.len(), 5, "abre, desce, desce, direita, Enter");
    assert!((0..400).all(|q| tecla_do_p1(p1 - 1, q).is_none()));
    // As teclas do condutor, pela porta da sessão do Play: a tela abre, desce ao Fire, 2.ª tecla, à escuta.
    let mut m = armada("P1");
    let map = m.hero.input_map.clone();
    m.jogador.start_with(&map, troca_do_jogador(&map), None);
    for k in teclas {
        let _ = m.jogador.key(k, true, &map);
    }
    let tela = m.jogador.screen();
    assert!(tela.is_open() && tela.listening(), "P1: aberta e à escuta");
    assert_eq!(m.jogador.rows(&map)[tela.selected()].action, "fire");
    assert_eq!(tela.slot(), 1, "a 2.ª tecla do Fire");
}

#[test]
fn as_cenas_da_lista_escutam_filtram_e_mostram_a_manche() {
    let m = armada("A2");
    assert_eq!(m.tab().doc.snap.listening, Some(id(&m, "reload")));
    let m = armada("A3");
    let shown = ph2d_app_input_map::view_list::shown(&m.tab().doc);
    assert_eq!(shown, vec![id(&m, "jump"), id(&m, "move_up")]);
    let m = armada("A4");
    let ml = m
        .hero
        .input_map
        .get(id(&m, "move_left"))
        .expect("move_left");
    assert!(ml.is_analog() && (ml.dead_zone - DEAD).abs() < 1e-6);
    let a1 = armada("A1");
    assert!(a1.hero.input_map.actions().iter().all(|a| !a.is_analog()));
}

#[test]
fn as_cenas_do_teclado_abrem_a_accao_e_o_desenho_delas() {
    let up = Key::named_in_script("up").and_then(|v| v.first().copied());
    let m = armada("B1");
    assert_eq!(m.tab().doc.kb.open, Some(id(&m, "jump")));
    let m = armada("B2");
    assert_eq!(m.tab().doc.kb.hover, up.map(KbHit::Key));
    let m = armada("B3");
    assert_eq!(m.tab().doc.kb.sub, Sub::Gamepad);
    assert_eq!(m.tab().doc.kb.open, Some(id(&m, "move_left")));
    let m = armada("B4");
    let reload = id(&m, "reload");
    assert_eq!(m.tab().doc.kb.open, Some(reload));
    assert!(
        m.hero
            .input_map
            .get(reload)
            .is_some_and(|a| a.bindings.is_empty())
    );
}

#[test]
fn a_cena_tem_um_leitor_do_fire_e_do_laser_e_nenhum_do_reload() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let _ = leitores(&mut sim, PRIMEIRA, |s| s.world_mut().spawn(()).id());
    let readers = ph2d_app_input_map::readers::of_world(sim.world());
    let fire = readers.get("fire").map_or(&[][..], Vec::as_slice);
    assert!(fire.iter().any(|r| matches!(r, Reader::Trigger { .. })));
    let laser = readers.get("laser").map_or(&[][..], Vec::as_slice);
    assert!(laser.iter().any(|r| matches!(r, Reader::Script { .. })));
    assert!(readers.get("reload").is_none_or(Vec::is_empty));
}
