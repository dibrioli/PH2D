//! As cenas de smoke da aba (`PH2D_GAME_SMOKE` 6..13, plano §8): o código que o roteiro do jogo dá é o mesmo que a
//! família arma, cada cena põe a aba no estado da maquete, e a cena tem os leitores do fire e do laser (o reload não).

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_app_input_map::smoke::{CENAS, DEAD, arma};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::Key;
use ph2d_input::readers::Reader;

fn armada(cena: &str) -> Mesa {
    let mut m = Mesa::new(View::Keyboard);
    assert!(arma(&mut m.hero, cena), "{cena} arma");
    m.frame();
    m
}

fn id(m: &Mesa, n: &str) -> ph2d_input::ActionId {
    m.hero.input_map.id(n).expect("a acção do mapa do dono")
}

#[test]
fn o_roteiro_do_jogo_e_a_familia_contam_as_mesmas_cenas() {
    assert_eq!(CENAS, ph2d_app_game::smoke::input_map::CODIGOS);
    assert!(!arma(&mut Mesa::new(View::List).hero, "Z9"));
}

#[test]
fn cada_cena_abre_a_aba_na_vista_dela_com_as_11_accoes() {
    for c in CENAS {
        let m = armada(c);
        assert!(m.active(), "{c}: a aba aberta e à frente");
        assert_eq!(m.hero.input_map.len(), 11, "{c}");
        let want = if c.starts_with('A') {
            View::List
        } else {
            View::Keyboard
        };
        assert_eq!(m.tab().doc.view(), want, "{c}");
    }
}

#[test]
fn as_cenas_da_lista_escutam_filtram_e_mostram_a_manche() {
    let m = armada("A2");
    assert_eq!(m.tab().doc.snap.listening, Some(id(&m, "reload")));
    let m = armada("A3");
    let shown = ph2d_app_input_map::view_list::shown(&m.tab().doc);
    assert_eq!(shown, vec![id(&m, "jump"), id(&m, "move_up")]);
    let m = armada("A4");
    let ml = m.hero.input_map.get(id(&m, "move_left")).expect("move_left");
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
    assert!(m.hero.input_map.get(reload).is_some_and(|a| a.bindings.is_empty()));
}

#[test]
fn a_cena_tem_um_leitor_do_fire_e_do_laser_e_nenhum_do_reload() {
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_app_game::smoke::input_map::leitores(&mut sim, ph2d_app_game::smoke::input_map::PRIMEIRA);
    let readers = ph2d_app_input_map::readers::of_world(sim.world());
    let fire = readers.get("fire").map_or(&[][..], Vec::as_slice);
    assert!(fire.iter().any(|r| matches!(r, Reader::Trigger { .. })));
    let laser = readers.get("laser").map_or(&[][..], Vec::as_slice);
    assert!(laser.iter().any(|r| matches!(r, Reader::Script { .. })));
    assert!(readers.get("reload").is_none_or(Vec::is_empty));
}
