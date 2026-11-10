//! Rodada do dono 10/11: a faixa da frase do pé mais estreita, com o texto no meio dela (lido na
//! tinta), e as entradas do «In this script» mais juntas que os títulos das secções.

use super::quem;
use crate::{linhas_de_base, meio_das_maiusculas};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::bancada::{Aim, BancadaView, outline_entry_h, phrase_height};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// A faixa é uma linha sem o vão de controlo, e o meio das maiúsculas da frase cai no meio dela.
#[test]
fn a_faixa_da_frase_e_uma_linha_sem_o_vao_e_o_texto_fica_no_meio() {
    assert_eq!(phrase_height(), ROW_H_PX, "a faixa = uma linha, sem o vão");
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let area = Rect::new(0.0, 800.0, 1000.0, phrase_height());
    let mut scene = VectorScene::new();
    v.paint_phrase(
        &who,
        "hero.luau",
        area,
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    let (px, _, base) = *linhas_de_base(&scene)
        .first()
        .expect("a frase pintou letras");
    let meio = meio_das_maiusculas(&mut ts, v.said(), px, base);
    let alvo = area.y + area.h * 0.5;
    assert!(
        (meio - alvo).abs() < 0.75,
        "as maiúsculas em {meio}, o meio da faixa em {alvo}"
    );
}

/// As entradas (`speed`, `lives`) avançam o passo apertado; um título de secção custa uma linha
/// inteira; os alvos do clique são os rects pintados.
#[test]
fn as_entradas_do_in_this_script_ficam_mais_juntas_que_os_titulos() {
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let area = Rect::new(0.0, 0.0, 300.0, 600.0);
    v.paint_outline(
        &who,
        area,
        &mut VectorScene::new(),
        &mut ts,
        &Theme::default(),
    );
    let eh = outline_entry_h();
    assert!(eh < ROW_H_PX, "o passo {eh} não aperta ({ROW_H_PX})");
    let at = |aim| v.target(aim).expect("a entrada tem alvo");
    let (speed, lives) = (at(Aim::Entry(0)), at(Aim::Entry(1)));
    assert!((lives.y - speed.y - eh).abs() < 1e-3, "{speed:?} {lives:?}");
    assert!((speed.h - eh).abs() < 1e-3, "o alvo é a linha pintada");
    // Da última propriedade (`fresh`, linha 5) à primeira função: a entrada mais o título.
    let (fresh, on_signal) = (at(Aim::Entry(5)), at(Aim::Entry(14)));
    assert!((on_signal.y - fresh.y - eh - ROW_H_PX).abs() < 1e-3);
}
