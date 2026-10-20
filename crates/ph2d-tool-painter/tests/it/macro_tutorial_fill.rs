//! W13 (`docs/Macros/handoffs/PROMPT_W13_o_tutorial_ensina_o_fill_2026-10-20.md`): o tutorial «fazer»
//! ensina o **Fill** — o ColorDrop gravado (W11) é uma etapa com critério entre as pinceladas, e só se
//! cumpre com a região certa na cor certa.

use super::macro_colordrop::{Limiar, N, NOVA, PRETO, grava_o_colordrop};
use super::macro_comum::cp;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::inicial;
use ph2d_tool_painter::macro_track::tutorial::{EnsinoDoPainter, mostra};

/// Um quadro do editor.
fn quadro(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(1000.0 / 60.0);
}

/// O aluno solta a cor `cor` do C&F em `(u, v)` (fracção da tela) e confirma — o gesto da shell, um
/// quadro por evento.
fn solta(t: &mut PainterTool, cor: [u8; 3], u: f32, v: f32) {
    let antes = t.brush_color_srgb8();
    t.set_brush_color_srgb8(cor);
    let volta = t.active_paint_mode_id();
    t.begin_colordrop_fill(volta);
    let p = [N as f32 * u, N as f32 * v];
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    quadro(t);
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
    quadro(t);
    t.fill_commit();
    quadro(t);
    t.set_brush_color_srgb8(antes);
}

/// ⭐ **Pincelada · Fill · Pincelada** pelo tutorial: o Fill é a 2.ª etapa, a pincelada seguinte não o
/// salta, a cor errada não o cumpre, a certa sim, o Ctrl+Z volta-o, e o «Mostra-me» fá-lo com a cor
/// gravada e devolve a do aluno.
#[test]
fn o_tutorial_ensina_o_fill_entre_as_pinceladas() {
    for limiar in [Limiar::Modal, Limiar::Parado] {
        let (_, doc) = grava_o_colordrop(limiar);
        let track = &doc.tracks[0];
        let e = EnsinoDoPainter::medido(track).expect("ensino");
        let ks: Vec<usize> = e.criterios.iter().map(|c| c.regiao.k).collect();
        assert_eq!(
            ks,
            vec![0, 1, 2],
            "{limiar:?}: o Fill é uma etapa do tutorial"
        );
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        assert_eq!(e.por_cumprir(&t), Some(0), "{limiar:?}: nada feito");
        mostra(&mut t, track, 0).expect("mostra a 1.ª pincelada");
        assert_eq!(e.por_cumprir(&t), Some(1), "{limiar:?}: falta o Fill");
        mostra(&mut t, track, 2).expect("mostra a 2.ª pincelada");
        assert_eq!(
            e.por_cumprir(&t),
            Some(1),
            "{limiar:?}: a pincelada seguinte não cumpre o Fill"
        );
        assert!(t.undo_last(), "a 2.ª pincelada desfaz-se");
        // Solto noutro ponto do mesmo papel: a região é a mesma.
        solta(&mut t, PRETO, 0.6, 0.6);
        assert_eq!(
            e.por_cumprir(&t),
            Some(1),
            "{limiar:?}: o Fill na cor errada não cumpre"
        );
        assert!(t.undo_last(), "o Fill errado desfaz-se");
        solta(&mut t, NOVA, 0.6, 0.6);
        assert_eq!(
            e.por_cumprir(&t),
            Some(2),
            "{limiar:?}: o Fill do aluno na cor certa cumpre"
        );
        mostra(&mut t, track, 2).expect("mostra a 2.ª pincelada");
        assert_eq!(
            e.por_cumprir(&t),
            None,
            "{limiar:?}: a 2.ª pincelada sobre o Fill cumpre-se"
        );
        assert!(
            t.undo_last() && t.undo_last(),
            "a pincelada e o Fill desfazem-se"
        );
        assert_eq!(
            e.por_cumprir(&t),
            Some(1),
            "{limiar:?}: o desfazer volta o passo"
        );
        let cor_do_aluno = [10, 200, 10];
        t.set_brush_color_srgb8(cor_do_aluno);
        mostra(&mut t, track, 1).expect("mostra o Fill");
        quadro(&mut t);
        assert_eq!(
            e.por_cumprir(&t),
            Some(2),
            "{limiar:?}: o «Mostra-me» do Fill cumpre-o"
        );
        assert_eq!(
            t.brush_color_srgb8(),
            cor_do_aluno,
            "{limiar:?}: a cor do aluno volta"
        );
        assert_eq!(
            t.active_paint_mode_id(),
            "brush",
            "{limiar:?}: o modo volta"
        );
    }
}

/// Apagar a etapa Fill no painel leva o C&F inteiro: a entrada em Fill antes do toque era deixada para
/// trás, e a pincelada seguinte tocava como um preenchimento (a tela inteira na cor dela).
#[test]
fn apagar_o_fill_nao_deixa_a_pincelada_seguinte_em_fill() {
    use super::macro_comum::toca;
    use ph2d_macro::MacroDoc;
    use ph2d_tool_painter::macro_track::etapas::{LeitorDoPainter, TipoDaEtapa, apaga, etapas};
    for limiar in [Limiar::Modal, Limiar::Parado] {
        let (_, mut doc) = grava_o_colordrop(limiar);
        apaga(&mut doc.tracks[0], 1).expect("apaga o Fill");
        let tipos: Vec<TipoDaEtapa> = etapas(&doc.tracks[0])
            .expect("etapas")
            .into_iter()
            .map(|e| e.tipo)
            .collect();
        assert_eq!(
            tipos,
            [TipoDaEtapa::Pincelada(1), TipoDaEtapa::Pincelada(2)],
            "{limiar:?}"
        );
        let tocavel = doc.tracks[0].tocavel(&LeitorDoPainter).expect("tocável");
        let sem = MacroDoc {
            tracks: vec![tocavel],
            ..doc.clone()
        };
        let mut toc = toca(&sem).expect("toca sem o Fill");
        let tela = toc.tool_mut().run_full().0;
        let texel = |u: f32, v: f32| {
            let i = (((v * N as f32) as usize) * N as usize + (u * N as f32) as usize) * 4;
            [tela[i], tela[i + 1], tela[i + 2]]
        };
        assert_eq!(
            texel(0.6, 0.6),
            [255; 3],
            "{limiar:?}: o papel ficou branco"
        );
        assert_eq!(texel(0.5, 0.85), PRETO, "{limiar:?}: a 2.ª pincelada");
        assert_eq!(
            toc.tool().active_paint_mode_id(),
            "brush",
            "{limiar:?}: o modo no fim"
        );
    }
}
