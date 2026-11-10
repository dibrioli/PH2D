//! Onda 3, item 3: o hover da tela «Controls» no Play (D32 do dono: acende no quadro, sem fade) e o «Reset to
//! defaults» com premido — o Down captura, o Up dentro dispara, fora cancela (o idioma do `interact` da aba).

use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::zones::Rect;
use ph2d_input::{InputMap, Key};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Brush, DrawOp, VectorScene};

use crate::paint::{HOVER_INK, PRESS_INK};
use crate::{Hover, Layout, PlaySession, host};

const F1: Key = Key(0xF704);
const ENTER: Key = Key(0x0D);
const J: Key = Key(0x4A);
const AREA: Rect = Rect::new(0.0, 0.0, 1600.0, 1000.0); // LITERAL-PX-OK: a janela das fotos da tela

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// A sessão a correr, com a tela aberta e pintada uma vez (o ponteiro lê a disposição do último quadro).
fn aberta(g: &InputMap, ts: &mut TextSystem) -> (PlaySession, Layout) {
    let mut s = PlaySession::default();
    s.keep_off_disk();
    s.start(g, None);
    let abre = host::key(&mut s, Some(F1), (true, false), true, (Some(g), true));
    assert!(abre.0, "F1 abre a tela");
    s.paint(AREA, g, &mut VectorScene::new(), ts, Theme::default());
    let lay = Layout::scrolled(AREA, &s.rows(g), s.screen(), ts);
    (s, lay)
}

/// Quantos preenchimentos da pintura saem na cor de `token`.
fn fills_in(s: &mut PlaySession, g: &InputMap, ts: &mut TextSystem, token: ColorToken) -> usize {
    let th = Theme::default();
    let want = resolve(token, th);
    let mut scene = VectorScene::new();
    scene.record();
    s.paint(AREA, g, &mut scene, ts, th);
    scene
        .take_record()
        .iter()
        .filter(|o| matches!(o, DrawOp::Fill { brush: Brush::Solid(c), .. } if *c == want))
        .count()
}

#[test]
fn a_linha_sob_o_ponteiro_acende_no_quadro_e_apaga_ao_sair() {
    let (g, mut ts) = (game(), TextSystem::new());
    let (mut s, lay) = aberta(&g, &mut ts);
    let frio = fills_in(&mut s, &g, &mut ts, HOVER_INK);
    let at = centre(lay.rows[1].band);
    assert_eq!(
        host::pointer_move(&mut s, Some(at)),
        (true, true),
        "o alvo mudou: repinta"
    );
    assert_eq!(s.screen().hovered(), Some(Hover::Row(lay.first + 1)));
    assert_eq!(
        host::pointer_move(&mut s, Some(at)),
        (true, false),
        "o mesmo alvo não repinta"
    );
    assert_eq!(
        fills_in(&mut s, &g, &mut ts, HOVER_INK),
        frio + 1,
        "a faixa da linha pairada acende no MESMO quadro"
    );
    assert_eq!(
        host::pointer_move(&mut s, Some((-10.0, -10.0))),
        (true, true)
    );
    assert_eq!(s.screen().hovered(), None, "fora do painel apaga");
    assert_eq!(fills_in(&mut s, &g, &mut ts, HOVER_INK), frio);
}

#[test]
fn o_reset_tem_hover_e_premido_e_o_up_fora_cancela() {
    let (g, mut ts) = (game(), TextSystem::new());
    let (mut s, lay) = aberta(&g, &mut ts);
    // Uma troca do jogador, para o Reset ter o que repor.
    for k in [ENTER, J] {
        let _ = host::key(&mut s, Some(k), (true, false), true, (Some(&g), false));
    }
    assert!(!s.keys().is_empty(), "o J entrou no jump");
    let (frio_h, frio_p) = (
        fills_in(&mut s, &g, &mut ts, HOVER_INK),
        fills_in(&mut s, &g, &mut ts, PRESS_INK),
    );
    let reset = centre(lay.reset);
    assert_eq!(host::pointer_move(&mut s, Some(reset)), (true, true));
    assert_eq!(s.screen().hovered(), Some(Hover::Reset));
    assert_eq!(
        fills_in(&mut s, &g, &mut ts, HOVER_INK),
        frio_h + 1,
        "o Reset pairado"
    );

    let _ = host::click(&mut s, (true, true), Some((reset, &g)), false);
    assert!(s.screen().reset_pressed(), "o Down captura");
    assert!(!s.keys().is_empty(), "o Down NÃO dispara");
    assert_eq!(
        fills_in(&mut s, &g, &mut ts, PRESS_INK),
        frio_p + 1,
        "o premido vê-se"
    );

    let fora = (-10.0, -10.0);
    let _ = host::pointer_move(&mut s, Some(fora));
    assert!(!s.screen().reset_pressed(), "fora do botão o premido apaga");
    let _ = host::click(&mut s, (false, true), Some((fora, &g)), false);
    assert!(!s.keys().is_empty(), "o Up fora cancela");

    let _ = host::pointer_move(&mut s, Some(reset));
    let _ = host::click(&mut s, (true, true), Some((reset, &g)), false);
    let _ = host::click(&mut s, (false, true), Some((reset, &g)), false);
    assert!(s.keys().is_empty(), "o Up dentro dispara");
    assert!(!s.screen().reset_pressed());
}

#[test]
fn fechada_a_tela_nao_e_dona_do_ponteiro() {
    let g = game();
    let mut s = PlaySession::default();
    s.keep_off_disk();
    s.start(&g, None);
    assert_eq!(
        host::pointer_move(&mut s, Some((10.0, 10.0))),
        (false, false)
    );
    assert_eq!(s.screen().hovered(), None);
}
