//! Onda 3, item 3, rodada 2 da auditoria: o hover DERIVA do último ponto do ponteiro sobre a disposição ACTUAL
//! (H1: a lista que rola com o ponteiro parado), sair da janela apaga (H2), o Down arma o premido a partir do ponto
//! dele (H3) e só o botão principal age (H4).

use ph2d_editor_core::zones::Rect;
use ph2d_input::{InputMap, Key};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::{Hover, Layout, PlaySession, host};

const F1: Key = Key(0xF704);
const ENTER: Key = Key(0x0D);
const DOWN: Key = Key(0xF701);
const J: Key = Key(0x4A);
/// Baixa de propósito: a lista não cabe e rola.
const SHORT: Rect = Rect::new(0.0, 0.0, 1600.0, 300.0); // LITERAL-PX-OK: janela baixa que obriga a rolar
const AREA: Rect = Rect::new(0.0, 0.0, 1600.0, 1000.0); // LITERAL-PX-OK: a janela das fotos da tela

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn key(s: &mut PlaySession, g: &InputMap, k: Key) {
    let _ = host::key(s, Some(k), (true, false), true, (Some(g), false));
}

/// Pinta e devolve a disposição desse quadro.
fn frame(s: &mut PlaySession, g: &InputMap, ts: &mut TextSystem, area: Rect) -> Layout {
    s.paint(area, g, &mut VectorScene::new(), ts, Theme::default());
    Layout::scrolled(area, &s.rows(g), s.screen(), ts)
}

fn aberta(g: &InputMap) -> PlaySession {
    let mut s = PlaySession::default();
    s.keep_off_disk();
    s.start(g, None);
    key(&mut s, g, F1);
    assert!(s.screen().is_open(), "F1 abre a tela");
    s
}

#[test]
fn h1_a_lista_que_rola_com_o_ponteiro_parado_acende_a_linha_sob_ele() {
    let (g, mut ts) = (game(), TextSystem::new());
    let mut s = aberta(&g);
    let lay = frame(&mut s, &g, &mut ts, SHORT);
    assert!(lay.bar.is_some(), "pré-condição: a lista não cabe e rola");
    let at = centre(lay.rows[0].band);
    let _ = host::pointer_move(&mut s, Some(at));
    assert_eq!(s.screen().hovered(), Some(Hover::Row(lay.first)));
    for _ in 1..s.rows(&g).len() {
        key(&mut s, &g, DOWN);
    }
    let depois = frame(&mut s, &g, &mut ts, SHORT);
    assert!(depois.first > lay.first, "↑/↓ rolou a lista");
    assert_eq!(
        s.screen().hovered(),
        depois.hover(at.0, at.1),
        "o hover é o que está sob o ponteiro parado no quadro NOVO"
    );
}

#[test]
fn h2_sair_da_janela_apaga_o_hover() {
    let (g, mut ts) = (game(), TextSystem::new());
    let mut s = aberta(&g);
    let lay = frame(&mut s, &g, &mut ts, AREA);
    let _ = host::pointer_move(&mut s, Some(centre(lay.rows[1].band)));
    assert!(s.screen().hovered().is_some());
    assert_eq!(host::pointer_move(&mut s, None), (true, true));
    assert_eq!(s.screen().hovered(), None);
    let _ = frame(&mut s, &g, &mut ts, AREA);
    assert_eq!(
        s.screen().hovered(),
        None,
        "o quadro seguinte não o reacende"
    );
}

#[test]
fn h3_o_down_no_reset_sem_mexer_mostra_o_premido() {
    let (g, mut ts) = (game(), TextSystem::new());
    let mut s = aberta(&g);
    let lay = frame(&mut s, &g, &mut ts, AREA);
    let _ = host::click(&mut s, (true, true), Some((centre(lay.reset), &g)), false);
    assert_eq!(
        s.screen().hovered(),
        Some(Hover::Reset),
        "o hover sai do ponto do Down"
    );
    assert!(
        s.screen().reset_pressed(),
        "o Down sem movimento arma o premido"
    );
}

#[test]
fn h3_a_tela_que_abre_sob_o_ponteiro_parado_acende_o_que_esta_sob_ele() {
    let (g, mut ts) = (game(), TextSystem::new());
    let mut s = aberta(&g);
    let lay = frame(&mut s, &g, &mut ts, AREA);
    let at = centre(lay.rows[1].band);
    let _ = host::pointer_move(&mut s, Some(at));
    key(&mut s, &g, F1);
    assert!(!s.screen().is_open());
    key(&mut s, &g, F1);
    let lay = frame(&mut s, &g, &mut ts, AREA);
    assert_eq!(s.screen().hovered(), lay.hover(at.0, at.1));
    assert!(s.screen().hovered().is_some());
}

#[test]
fn h4_so_o_botao_principal_faz_reset() {
    let (g, mut ts) = (game(), TextSystem::new());
    let mut s = aberta(&g);
    let lay = frame(&mut s, &g, &mut ts, AREA);
    for k in [ENTER, J] {
        key(&mut s, &g, k);
    }
    assert!(!s.keys().is_empty(), "o J entrou no jump");
    let reset = Some((centre(lay.reset), &g));
    for down in [true, false] {
        assert_eq!(
            host::click(&mut s, (down, false), reset, false),
            (true, None),
            "a tela continua dona do rato"
        );
    }
    assert!(!s.keys().is_empty(), "direito/meio não repõem");
    assert!(!s.screen().reset_pressed());
    let _ = host::click(&mut s, (true, true), reset, false);
    let _ = host::click(&mut s, (false, true), reset, false);
    assert!(s.keys().is_empty(), "o principal repõe");
}
