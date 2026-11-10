//! F3 (fotos C4/P1): a caixa vazia «Add» legível num contorno tracejado, a dica no entrelinhado normal (uma linha
//! quando cabe) e a primeira tecla de cada linha numa coluna de largura fixa, encostada à esquerda.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::rect_for_label;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{InputMap, Key, PlayerKeys};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, TextRendering, Theme};
use ph2d_vector::{DrawOp, VectorScene};

use crate::layout::{FOOT_PX, HINT_LINE_H, HINTS, foot_lines};
use crate::paint::{ADD_DASH_INK, ADD_INK};
use crate::{ControlsScreen, Layout, rows};

const AREAS: [Rect; 2] = [
    Rect::new(0.0, 0.0, 1600.0, 1000.0), // LITERAL-PX-OK: a janela da foto P1
    Rect::new(0.0, 0.0, 1280.0, 720.0),  // LITERAL-PX-OK: a janela do plano §8
];

fn game() -> InputMap {
    let mut g = InputMap::with_player_defaults();
    g.controls_mut().players_can_change_keys = true;
    g
}

/// Corre `f` no modo de texto dos testes e no de fábrica do app.
fn em_cada_modo(mut f: impl FnMut()) {
    for modo in [TextRendering::Default, TextRendering::CrispHeavyPlus] {
        ph2d_text::set_active_text_rendering(modo);
        f();
    }
    ph2d_text::set_active_text_rendering(TextRendering::Default);
}

fn coluna_fixa(l: &Layout, at: &str) {
    let first = l.rows[0].slots[0];
    for (i, r) in l.rows.iter().enumerate() {
        let s = r.slots[0];
        assert!(
            (s.x - first.x).abs() < 1e-3,
            "{at} linha {i}: x {} × {}",
            s.x,
            first.x
        );
        assert!(
            (s.w - first.w).abs() < 1e-3,
            "{at} linha {i}: w {} × {}",
            s.w,
            first.w
        );
        assert!(s.x >= r.name.x + r.name.w, "{at} linha {i}: pisa o nome");
        for p in r.slots.windows(2) {
            assert!(
                p[1].x >= p[0].x + p[0].w,
                "{at} linha {i}: caixas sobrepostas"
            );
        }
    }
}

/// ⛔ Foto C4/P1: as primeiras caixas começavam cada uma num x (borda dentada). Agora são uma coluna: o mesmo x e a
/// mesma largura em todas as linhas, no jogo, na prévia e à escuta (a coluna reserva «Press a key»).
#[test]
fn a_primeira_tecla_e_uma_coluna_de_largura_fixa() {
    em_cada_modo(|| {
        let g = game();
        let r = rows(&g);
        let mut ts = TextSystem::new();
        for area in AREAS {
            coluna_fixa(&Layout::of(area, &r, &mut ts), "jogo");
        }
        coluna_fixa(&Layout::natural(0.0, 0.0, &r, &mut ts), "prévia");
        let mut s = ControlsScreen::default();
        let mut keys = PlayerKeys::new();
        s.handle_key(Key(0xF704), true, &g, &mut keys); // F1 abre
        s.handle_key(Key(0x0D), true, &g, &mut keys); // Enter escuta a primeira tecla
        assert!(s.listening());
        let rest = Layout::of(AREAS[0], &r, &mut ts);
        let heard = Layout::scrolled(AREAS[0], &r, &s, &mut ts);
        coluna_fixa(&heard, "à escuta");
        assert_eq!(rest.panel, heard.panel, "a caixa não salta ao escutar");
    });
}

/// ⛔ Foto C4: a dica tinha as linhas afastadas pela faixa de um botão. O passo é o entrelinhado normal, e a frase que
/// cabe na largura vai numa linha só.
#[test]
fn a_dica_tem_entrelinhado_normal_e_uma_linha_quando_cabe() {
    em_cada_modo(|| {
        let r = rows(&game());
        let mut ts = TextSystem::new();
        const {
            assert!(
                HINT_LINE_H < m::FIELD_H - m::SEG_INSET,
                "não é a faixa do botão"
            );
            assert!(HINT_LINE_H > FOOT_PX, "as linhas não se pisam");
        }
        for area in AREAS {
            let l = Layout::of(area, &r, &mut ts);
            let shown = foot_lines(&mut ts, tr(HINTS[0]), l.hint.w).len();
            assert!((l.hint.h - HINT_LINE_H * shown as f32).abs() < 1e-3);
            for k in HINTS {
                let whole = rect_for_label(ts.prefix_width(tr(k), FOOT_PX).ceil());
                let n = foot_lines(&mut ts, tr(k), l.hint.w).len();
                if whole <= l.hint.w {
                    assert_eq!(n, 1, "{k}: cabe, uma linha");
                }
            }
        }
    });
}

/// ⛔ Foto C4: «Add» no texto terciário sobre a caixa cheia mal se lia. A caixa vazia é um contorno tracejado com o
/// texto secundário: um tracejado por caixa vazia, e nenhum preenchimento dela.
#[test]
fn a_caixa_vazia_e_tracejada_e_legivel() {
    assert_eq!(ADD_INK, ColorToken::Text2);
    assert_ne!(
        ADD_DASH_INK,
        ColorToken::Bg1,
        "o tracejado vê-se sobre o painel"
    );
    let g = game();
    let empty_before = rows(&g)
        .iter()
        .map(|x| x.slots() - x.keys.len())
        .sum::<usize>();
    assert!(empty_before > 0, "o mapa de fábrica tem caixas vazias");
    let r = rows(&g);
    let mut ts = TextSystem::new();
    let mut scene = VectorScene::new();
    scene.record();
    crate::paint::paint_preview_natural(&r, &mut scene, &mut ts, Theme::default());
    let rec = scene.take_record();
    let dashed = rec
        .iter()
        .filter(|o| matches!(o, DrawOp::Stroke { style, .. } if !style.dash_pattern.is_empty()))
        .count();
    assert_eq!(dashed, empty_before, "um tracejado por caixa vazia");
}
