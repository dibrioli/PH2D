//! **A tinta viva sobrevive dentro do Wet** ([plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)): o que outra
//! ferramenta escreve enterra só o que mudou, e desfazê-lo devolve a tinta de antes.

use super::adormecida_tests::{borracha_viva, traco_de_outro_meio};
use super::agua_sobre_oleo_tests::{H, W, com_oleo, mostrada, ponteiro, preset};
use super::borracha_de_fabrica_tests::{apaga, diferentes};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

const FUNDO: (u32, u32, u32, u32) = (20, 130, 180, 30);

/// Só as linhas `y0..y1`: a Borracha passa em `y < 125`.
fn linhas(p: &[u8], y0: usize, y1: usize) -> Vec<u8> {
    p[y0 * W * 4..y1 * W * 4].to_vec()
}

fn preenche_longe(t: &mut PainterTool) {
    preenche(t, FUNDO);
}

/// Selecciona, preenche (o passo que interessa, no topo do undo depois do quadro) e larga a selecção como o artista,
/// num passo seguinte: quem desfaz o Fill desfaz os dois ([`desfaz_o_fill`]).
fn preenche(t: &mut PainterTool, r: (u32, u32, u32, u32)) {
    t.set_rect_selection(r.0, r.1, r.2, r.3);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.selection_color_fill();
    t.ticks_em_blocos(1);
    t.clear_selection();
}

fn desfaz_o_fill(t: &mut PainterTool) {
    assert!(t.undo_last(), "largar a selecção desfaz-se");
    assert!(t.undo_last(), "o Fill desfaz-se");
}

fn borracha_e_o_vivo(t: &mut PainterTool) {
    assert!(t.paint.wetpaint.session.is_some(), "a sessão morreu");
    t.set_paint_tool_mode("eraser");
    let r = apaga(t);
    assert_eq!(
        diferentes(&linhas(&borracha_viva(), 0, 125), &linhas(&r, 0, 125)),
        0,
        "a Borracha sobre o óleo não é a da tinta viva"
    );
}

/// ⭐ Fill da selecção, sem sair do Wet: a tinta longe continua viva e o preenchimento fica na tela.
#[test]
fn preencher_a_selecao_nao_mata_a_tinta() {
    let mut t = com_oleo();
    preenche_longe(&mut t);
    t.ticks_em_blocos(30);
    let cheia = mostrada(&mut t);
    assert_eq!(
        &cheia[(140 * W + 60) * 4..(140 * W + 60) * 4 + 3],
        &[220, 30, 30],
        "o preenchimento sumiu da tela"
    );
    borracha_e_o_vivo(&mut t);
    assert_eq!(
        diferentes(&linhas(&cheia, 140, H), &linhas(&mostrada(&mut t), 140, H)),
        0,
        "a Borracha mexeu no preenchimento longe"
    );
}

/// ⭐ Desfazer o Fill: a tela volta e a tinta nem soube.
#[test]
fn desfazer_o_preenchimento_devolve_a_tinta() {
    let mut t = com_oleo();
    let antes = mostrada(&mut t);
    preenche_longe(&mut t);
    t.ticks_em_blocos(10);
    desfaz_o_fill(&mut t);
    assert_eq!(diferentes(&antes, &mostrada(&mut t)), 0, "a tela de antes");
    assert!(
        t.paint.wetpaint.session.is_some(),
        "a sessão morreu no undo"
    );
    assert!(t.redo_last(), "o Fill refaz-se");
    assert_eq!(
        &mostrada(&mut t)[(140 * W + 60) * 4..(140 * W + 60) * 4 + 3],
        &[220, 30, 30],
        "o redo do preenchimento"
    );
    assert!(t.undo_last(), "o Fill desfaz-se de novo");
    assert!(t.paint.wetpaint.session.is_some(), "a sessão, de novo");
    assert!(
        t.undo_last(),
        "a selecção desfaz-se: ela prenderia a Borracha"
    );
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&borracha_viva(), &apaga(&mut t)), 0);
}

/// ⭐ Fill POR CIMA do óleo: a tinta fica enterrada, e desfazê-lo devolve a borracha que amassa.
#[test]
fn desfazer_o_preenchimento_sobre_o_oleo_devolve_o_relevo() {
    let mut t = com_oleo();
    preenche(&mut t, (100, 50, 60, 80));
    t.ticks_em_blocos(10);
    desfaz_o_fill(&mut t);
    assert!(
        t.undo_last(),
        "a selecção desfaz-se: ela prenderia a Borracha"
    );
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&borracha_viva(), &apaga(&mut t)), 0);
}

/// ⭐ O caso do registro: traço de outro modo por cima, voltar ao Wet (enterra), desfazê-lo: o óleo volta inteiro.
#[test]
fn desfazer_um_traco_de_outro_modo_ja_acordado_devolve_o_oleo() {
    let vivo = borracha_viva();
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        let mut t = com_oleo();
        t.set_paint_media(meio);
        traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
        t.set_paint_media(PaintMedia::WetPaint);
        t.ticks_em_blocos(5);
        assert!(t.undo_last(), "{meio:?}: o traço desfaz-se");
        assert!(t.paint.wetpaint.session.is_some(), "{meio:?}: a sessão");
        t.set_paint_tool_mode("eraser");
        assert_eq!(diferentes(&vivo, &apaga(&mut t)), 0, "{meio:?}");
    }
}

/// ⭐ Três traços de outro modo, um enterro só: desfeito um, os outros dois ficam na tela e sobre a tinta; desfeitos
/// todos, o óleo volta; refeitos, voltam os traços.
#[test]
fn varios_tracos_de_outro_modo_desfazem_se_um_a_um() {
    let tracos = [
        ((110.0, 60.0), (110.0, 120.0)),
        ((130.0, 60.0), (130.0, 120.0)),
        ((150.0, 60.0), (150.0, 120.0)),
    ];
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    let mut telas = vec![mostrada(&mut t)];
    for (a, b) in tracos {
        traco_de_outro_meio(&mut t, a, b);
        telas.push(mostrada(&mut t));
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t.ticks_em_blocos(5);
    for i in (0..3).rev() {
        assert!(t.undo_last(), "o traço {i} desfaz-se");
        t.ticks_em_blocos(5);
        assert!(t.paint.wetpaint.session.is_some(), "a sessão, no traço {i}");
        assert_eq!(diferentes(&telas[i], &mostrada(&mut t)), 0, "a tela {i}");
    }
    for (i, tela) in telas.iter().enumerate().skip(1) {
        assert!(t.redo_last(), "o traço {i} refaz-se");
        t.ticks_em_blocos(5);
        assert!(t.paint.wetpaint.session.is_some(), "a sessão, no redo {i}");
        assert_eq!(diferentes(tela, &mostrada(&mut t)), 0, "o redo {i}");
    }
    for _ in 0..3 {
        assert!(t.undo_last(), "desfaz tudo");
    }
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&borracha_viva(), &apaga(&mut t)), 0);
}

/// Um passo sem pixels (criar uma selecção) não mexe na tinta, nem ao fazer nem ao desfazer.
#[test]
fn desfazer_um_passo_sem_pixels_nao_mata_a_tinta() {
    let mut t = com_oleo();
    t.set_rect_selection(FUNDO.0, FUNDO.1, FUNDO.2, FUNDO.3);
    t.ticks_em_blocos(2);
    assert!(t.undo_last(), "a selecção desfaz-se");
    borracha_e_o_vivo(&mut t);
}

/// Desfazer a criação de uma camada volta à da tinta, que acorda com ela.
#[test]
fn desfazer_uma_camada_nova_acorda_a_tinta() {
    let mut t = com_oleo();
    t.add_raster_layer("outra").expect("a outra camada");
    t.ticks_em_blocos(2);
    assert!(t.undo_last(), "a camada desfaz-se");
    t.ticks_em_blocos(2);
    borracha_e_o_vivo(&mut t);
}

/// O balde é uma troca de modo: acorda por cima, com o relevo longe intacto.
#[test]
fn o_balde_nao_mata_a_tinta() {
    let mut t = com_oleo();
    let antes = mostrada(&mut t);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(ponteiro(10.0, 10.0, PointerPhase::Down));
    t.on_canvas_pointer(ponteiro(10.0, 10.0, PointerPhase::Up));
    t.fill_commit();
    let cheia = mostrada(&mut t);
    assert!(
        diferentes(&antes, &cheia) > 1000,
        "CONTROLO: o balde pintou"
    );
    t.set_paint_media(PaintMedia::WetPaint);
    t.ticks_em_blocos(10);
    assert!(t.paint.wetpaint.session.is_some(), "a sessão");
    assert_eq!(
        diferentes(&cheia, &mostrada(&mut t)),
        0,
        "acordar não apaga"
    );
    assert!(t.undo_last(), "o balde desfaz-se");
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&borracha_viva(), &apaga(&mut t)), 0);
}

/// A água que corria continua a correr depois de uma escrita alheia, e a escrita fica.
#[test]
fn a_agua_continua_a_correr_depois_de_um_preenchimento() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Round.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.on_canvas_pointer(ponteiro(40.0, 60.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(40.0 + 5.0 * k as f32, 60.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(190.0, 60.0, PointerPhase::Up));
    t.ticks_em_blocos(5);
    preenche_longe(&mut t);
    t.ticks_em_blocos(1);
    let parada = mostrada(&mut t);
    t.ticks_em_blocos(30);
    let depois = mostrada(&mut t);
    assert!(t.paint.wetpaint.session.is_some(), "a sessão");
    assert!(diferentes(&parada, &depois) > 50, "a água parou");
    assert_eq!(
        &depois[(150 * W + 60) * 4..(150 * W + 60) * 4 + 3],
        &[220, 30, 30],
        "a água apagou o preenchimento"
    );
}

/// ⛔ Sem isto cada composição copiava a tela inteira (9,86 ms a 4096²): o traço molhado nunca retém a tela.
#[test]
fn o_traco_molhado_nao_retem_a_tela() {
    for meio in [
        "Oil/Dabber/Dabber.ph2d-brush.png",
        "Watercolor/Fluid/Wet Round.ph2d-brush.png",
    ] {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.preset_aplicar(&preset(meio)).expect("o meio");
        t.set_brush_size_px(30.0);
        for y in [40.0, 90.0] {
            t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
            for k in 1..=30 {
                t.on_canvas_pointer(ponteiro(40.0 + 5.0 * k as f32, y, PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
                assert!(
                    t.undo.write_state.tela_que_saiu().is_none(),
                    "{meio}: a tela ficou retida a meio do traço"
                );
            }
            t.on_canvas_pointer(ponteiro(190.0, y, PointerPhase::Up));
            t.ticks_em_blocos(3);
            assert!(
                t.undo.write_state.tela_que_saiu().is_none(),
                "{meio}: no fim"
            );
        }
    }
}
