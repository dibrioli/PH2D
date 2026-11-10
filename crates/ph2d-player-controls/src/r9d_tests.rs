//! R9d: a prévia sem foco, o painel que abraça as linhas (maquete C4), o ficheiro do jogador fora do smoke e do
//! `game_id` 0 (plano InputMap §4).

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::rect_for_label;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{Binding, InputMap, Key, PlayerKeys};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, VectorScene};

use crate::layout::{FOOT_PX, HINTS, foot_lines};
use crate::{
    ControlsScreen, Layout, Outcome, PlaySession, player_file, player_file_of, rebind, rows,
};

const F1: Key = Key(0xF704);
const ENTER: Key = Key(0x0D);
const DOWN: Key = Key(0xF701);
const J: Key = Key(0x4A);
const K: Key = Key(0x4B);
const AREA: Rect = Rect::new(0.0, 0.0, 1600.0, 1000.0); // LITERAL-PX-OK: a janela medida da foto P1

fn game() -> InputMap {
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    m
}

/// `(preenchimentos, contornos)` da pintura.
fn ops(paint: impl FnOnce(&mut VectorScene)) -> (usize, usize) {
    let mut scene = VectorScene::new();
    scene.record();
    paint(&mut scene);
    let rec = scene.take_record();
    let fills = rec
        .iter()
        .filter(|o| matches!(o, DrawOp::Fill { .. }))
        .count();
    let strokes = rec
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { .. }))
        .count();
    (fills, strokes)
}

/// A prévia é o painel do jogo sem a faixa da linha escolhida nem o anel de foco da caixa — e com a MESMA disposição.
#[test]
fn a_previa_nao_acende_linha_nem_foco() {
    let g = game();
    let r = rows(&g);
    let mut ts = TextSystem::new();
    let s = ControlsScreen::default();
    let lay = Layout::natural(0.0, 0.0, &r, &mut ts);
    let th = Theme::default();
    let play = ops(|sc| crate::paint::paint_with(&lay, &r, &s, sc, &mut ts, th, false));
    let mut lay_prev = None;
    let prev = ops(|sc| lay_prev = Some(crate::paint::paint_preview_natural(&r, sc, &mut ts, th)));
    assert_eq!(lay_prev, Some(lay), "a mesma disposição");
    assert_eq!(play.0, prev.0 + 1, "só a faixa da linha escolhida sai");
    assert_eq!(play.1, prev.1 + 1, "só o anel de foco sai");
}

/// ⛔ Foto P1: a dica alargava o painel muito além das linhas. O painel tem a largura das linhas (ou do título) e a
/// dica parte-se nela, reservando as linhas da MAIOR das três frases.
#[test]
fn o_painel_abraca_as_linhas_e_o_pe_parte_se() {
    let g = game();
    let r = rows(&g);
    let mut ts = TextSystem::new();
    let lay = Layout::of(AREA, &r, &mut ts);
    let longest = HINTS
        .iter()
        .map(|k| rect_for_label(ts.prefix_width(tr(k), FOOT_PX).ceil()))
        .fold(0.0_f32, f32::max);
    assert!(
        lay.hint.w < longest,
        "a dica ({longest}) não alarga o painel ({})",
        lay.hint.w
    );
    let n = HINTS
        .iter()
        .map(|k| foot_lines(&mut ts, tr(k), lay.hint.w).len())
        .max()
        .expect("frases");
    assert!(n >= 2, "a frase maior parte-se");
    assert!(
        (lay.hint.h - crate::layout::HINT_LINE_H * n as f32).abs() < 1e-3,
        "uma faixa por linha"
    );
    assert!(
        lay.hint.y + lay.hint.h <= lay.panel.y + lay.panel.h,
        "cabe no painel"
    );
    for k in HINTS {
        for l in foot_lines(&mut ts, tr(k), lay.hint.w) {
            assert!(
                rect_for_label(ts.prefix_width(&l, FOOT_PX)) <= lay.hint.w + 1e-3,
                "{l:?} cabe"
            );
        }
    }
    let rows_right = lay
        .rows
        .iter()
        .map(|x| x.band.x + x.band.w)
        .fold(0.0_f32, f32::max);
    assert!((rows_right - (lay.panel.x + lay.panel.w - m::CARD_PAD_X)).abs() < 1e-3);
}

/// `game_id` 0 (projecto gravado sem mundo) não tem ficheiro: partilharia o `0000000000000000.txt` entre projectos.
/// A tela funciona na mesma, em memória.
#[test]
fn o_game_id_zero_nao_tem_ficheiro_e_a_tela_funciona_em_memoria() {
    let base = std::env::temp_dir().join(format!("ph2d-r9d-zero-{}", std::process::id()));
    assert_eq!(player_file_of(Some(&base), 0), None);
    assert_eq!(player_file_of(None, 0xAB), None);
    assert_eq!(
        player_file_of(Some(&base), 0xAB),
        Some(player_file(&base, 0xAB))
    );
    let g = game();
    let mut s = PlaySession::default();
    s.start(&g, player_file_of(Some(&base), 0));
    assert_eq!(troca_o_jump(&mut s, &g), Outcome::Changed);
    assert!(
        s.map(&g)
            .actions()
            .iter()
            .any(|a| a.name == "jump" && a.bindings.contains(&Binding::Key(J)))
    );
    assert!(!base.join("players").exists(), "nada no disco");
}

/// O smoke nunca lê nem grava o ficheiro do jogador real, nem depois de parar e voltar a jogar.
#[test]
fn a_sessao_fora_do_disco_nao_le_nem_grava() {
    let base = std::env::temp_dir().join(format!("ph2d-r9d-off-{}", std::process::id()));
    let path = player_file(&base, 0x1A_9E3C);
    let g = game();
    let mut real = PlayerKeys::new();
    assert_eq!(rebind(&g, &mut real, "move_left", 1, K), Outcome::Changed);
    crate::file::save(&path, &real).expect("grava");
    let before = std::fs::read_to_string(&path).expect("lê");
    let mut s = PlaySession::default();
    s.keep_off_disk();
    for _ in 0..2 {
        s.start(&g, Some(path.clone()));
        assert!(s.keys().is_empty(), "não leu as teclas do dono");
        assert_eq!(troca_o_jump(&mut s, &g), Outcome::Changed);
        s.stop();
    }
    assert_eq!(
        std::fs::read_to_string(&path).expect("lê"),
        before,
        "não gravou"
    );
    let mut normal = PlaySession::default();
    normal.start(&g, Some(path.clone()));
    assert_eq!(normal.keys(), &real, "fora do smoke, o ficheiro lê-se");
    let _ = std::fs::remove_dir_all(&base);
}

fn troca_o_jump(s: &mut PlaySession, g: &InputMap) -> Outcome {
    s.key(F1, true, g);
    let at = rows(g)
        .iter()
        .position(|r| r.action == "jump")
        .expect("jump");
    for _ in 0..at {
        s.key(DOWN, true, g);
    }
    s.key(ENTER, true, g);
    s.key(J, true, g)
}
