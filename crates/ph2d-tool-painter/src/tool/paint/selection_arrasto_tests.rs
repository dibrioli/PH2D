//! B16.7 (PL-18): o ARRASTO de uma seleção (Marquee, Lasso, Automatic) não recompõe a tela por Move.
//! O compositor não lê a seleção (o tracejado é um overlay do shell, por quadro); só o Up, que
//! grava a entrada estrutural, invalida.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, RasterEditTool};
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

struct Arrasto {
    /// Drenagens da prévia que subiram a tela INTEIRA durante os Moves.
    telas_inteiras: usize,
    /// Quantas vezes a revisão da pilha andou durante os Moves.
    revisoes: u64,
    /// A máscara depois de cada Move (o que o overlay mostra).
    mascaras: Vec<Vec<u8>>,
    /// A prévia depois do Up.
    final_rgba: Vec<u8>,
}

fn arrasta(mode: u8, oraculo: bool) -> Arrasto {
    super::selection_input::INVALIDA_POR_MOVE.set(oraculo);
    let (w, h) = (96u32, 64u32);
    let mut t = PainterTool::default();
    let mut px = vec![0u8; (w * h * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        p.copy_from_slice(&[(i % 251) as u8, (i % 97) as u8, 40, 255]);
    }
    t.set_source(px, w, h);
    let fundo = t.layers.root()[0];
    t.add_raster_layer("cima").expect("camada");
    t.select_layer(fundo); // o Automatic inunda a camada ativa: a de baixo tem píxeis

    t.set_paint_tool_mode("selection");
    t.set_selection_mode(mode);
    let _ = t.take_preview_arc();
    let _ = t.take_preview_upload_bbox();
    t.on_canvas_pointer(cp([10.0, 8.0], PointerPhase::Down));
    let _ = t.take_preview_arc();
    let _ = t.take_preview_upload_bbox();
    let rev0 = t.layers_revision();
    let mut telas_inteiras = 0;
    let mut mascaras = Vec::new();
    for k in 1..=8 {
        let k = k as f32;
        t.on_canvas_pointer(cp([10.0 + 8.0 * k, 8.0 + 0.7 * k * k], PointerPhase::Move));
        if t.take_preview_arc().is_some() && t.take_preview_upload_bbox().is_none() {
            telas_inteiras += 1;
        }
        mascaras.push(t.paint.selection_mask.as_slice().to_vec());
    }
    let revisoes = t.layers_revision() - rev0;
    t.on_canvas_pointer(cp([80.0, 50.0], PointerPhase::Up));
    let final_rgba = t
        .take_preview_arc()
        .expect("o Up recompõe")
        .0
        .as_slice()
        .to_vec();
    super::selection_input::INVALIDA_POR_MOVE.set(false);
    Arrasto {
        telas_inteiras,
        revisoes,
        mascaras,
        final_rgba,
    }
}

#[test]
fn um_arrasto_de_selecao_nao_recompoe_a_tela_por_move() {
    // 0 = Automatic, 1 = Lasso, 2 = Rect, 3 = Ellipse.
    for mode in 0..=3u8 {
        let antes = arrasta(mode, true);
        let depois = arrasta(mode, false);
        // CONTROLO: o caminho de antes recompõe a tela inteira a cada Move.
        assert_eq!(
            antes.telas_inteiras, 8,
            "modo {mode}: o controlo deixou de recompor"
        );
        assert!(
            antes.revisoes >= 8,
            "modo {mode}: o controlo deixou de publicar"
        );
        assert_eq!(
            depois.telas_inteiras, 0,
            "modo {mode}: o arrasto recompôs a tela {} vezes",
            depois.telas_inteiras
        );
        assert_eq!(
            depois.revisoes, 0,
            "modo {mode}: o arrasto republicou a pilha"
        );
        assert!(
            depois
                .mascaras
                .last()
                .is_some_and(|m| m.iter().any(|&v| v > 0)),
            "modo {mode}: a seleção não aparece durante o arrasto"
        );
        assert!(
            antes.mascaras == depois.mascaras,
            "modo {mode}: a máscara ao vivo mudou"
        );
        assert!(
            antes.final_rgba == depois.final_rgba,
            "modo {mode}: a prévia final mudou"
        );
    }
}
