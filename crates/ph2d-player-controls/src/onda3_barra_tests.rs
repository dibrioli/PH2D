//! ⭐ **O botão «Controls» da barra do Play** (dono 10/11): ele abre e fecha a tela pelo MESMO caminho da tecla que a
//! abre ([`host::toggle`] chama [`host::key`] com a tecla do mapa), e a shell só o traduz.

use crate::{PlaySession, host};
use ph2d_input::{InputMap, Key};

const F4: Key = Key(0xF707);

fn mapa(pode: bool) -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = pode;
    m.controls_mut().open_key = F4;
    m
}

/// No Play: o 1.º clique abre e pausa, o 2.º fecha e retoma — como a tecla; e a tecla é a do mapa (F4), não o F1.
/// Mutação: fixar a tecla do `toggle` no F1 ⇒ nada abre.
#[test]
fn o_botao_abre_e_o_segundo_fecha_pela_tecla_do_mapa() {
    let m = mapa(true);
    let mut s = PlaySession::default();
    s.start(&m, None);
    assert_eq!(host::toggle(&mut s, &m, true), Some(false), "abre e pausa");
    assert!(s.screen().is_open());
    assert_eq!(
        host::toggle(&mut s, &m, false),
        Some(true),
        "fecha e retoma"
    );
    assert!(!s.screen().is_open());
    let mut pela_tecla = PlaySession::default();
    pela_tecla.start(&m, None);
    let tecla = host::key(
        &mut pela_tecla,
        Some(F4),
        (true, false),
        true,
        (Some(&m), true),
    );
    assert_eq!(tecla.1, Some(false), "a tecla faz o mesmo");
}

/// Fora do Play, ou num jogo sem a tela, o botão não faz nada (a barra também o mostra desligado).
#[test]
fn fora_do_play_ou_sem_tela_o_botao_nao_faz_nada() {
    let m = mapa(true);
    let mut s = PlaySession::default();
    assert_eq!(host::toggle(&mut s, &m, false), None);
    assert!(!s.screen().is_open());
    let fechado = mapa(false);
    s.start(&fechado, None);
    assert_eq!(host::toggle(&mut s, &fechado, true), None);
    assert!(!s.screen().is_open());
}

/// A shell só traduz: o pedido da barra vai ao `host::toggle`, e o clique sobre o botão com a tela aberta passa ao
/// hero (sem isto a tela, dona do rato, comeria o 2.º clique e o botão nunca a fecharia).
#[test]
fn a_shell_traduz_o_botao_pelo_host_e_deixa_passar_o_clique_nele() {
    let bus = include_str!("../../../shells/desktop/src/render_loop/fase_bus_chrome.rs");
    let arm = bus
        .find("EditorAction::PlayerControlsToggle")
        .expect("o braço do bus");
    assert!(bus[arm..].contains("host::toggle("));
    let rato = include_str!("../../../shells/desktop/src/input_dispatch/controles_do_jogador.rs");
    let passa = rato
        .find("TOPBAR_PLAYER_CONTROLS)")
        .expect("o clique no botão passa");
    assert!(passa < rato.find("host::click(").expect("o clique da tela"));
}
