//! **O desfazer do composite Wet guarda a caixa que ele escreve, não a tela** (n.º 6, 12/11): antes o
//! `fork_canvas(.., None)` capturava a tela inteira por traço (33 MB a 4K, ~6 ms) e, na camada nova (a tela com
//! outro dono), o fork copiava-a OUTRA vez. Controlo: o caminho antigo (`CAPTURA_PELA_CAIXA` e
//! `CAPTURA_PARTILHADA` desligados) captura a tela inteira. Os dois caminhos dão os mesmos bytes, e o
//! desfazer/refazer devolve-os.

use super::agua_sobre_oleo_tests::ponteiro;
use crate::tool::PainterTool;
use crate::tool::paint::plane_fork::{CAPTURA_PARTILHADA, CAPTURA_PELA_CAIXA};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

const W: u32 = 512;
const H: u32 = 384;

/// Um traço Wet em `y`; devolve o pico dos bytes que o journal da tela reteve.
fn traco(t: &mut PainterTool, y: f32) -> usize {
    let mut pico = 0;
    t.on_canvas_pointer(ponteiro(120.0, y, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=8 {
        t.on_canvas_pointer(ponteiro(120.0 + 14.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
        pico = pico.max(t.undo.write_state.journal_heap_bytes());
    }
    t.on_canvas_pointer(ponteiro(240.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    let _ = t.take_preview_arc();
    pico
}

struct Corrida {
    picos: [usize; 4],
    antes: Vec<u8>,
    depois: Vec<u8>,
    desfeito: Vec<u8>,
    refeito: Vec<u8>,
}

fn corre(nova: bool, aguada: bool) -> Corrida {
    CAPTURA_PELA_CAIXA.set(nova);
    CAPTURA_PARTILHADA.set(nova);
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (W * H * 4) as usize], W, H);
    if aguada {
        // O Watercolor Basic: a aguada óptica do Paint (`watercolor_render`).
        t.paint.brush.watercolor = true;
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        assert!(t.watercolor_render_active(), "a fixture: a aguada");
    } else {
        t.set_paint_tool_mode("wetpaint");
    }
    t.set_brush_size_px(24.0);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    let b1 = traco(&mut t, 80.0);
    let b2 = traco(&mut t, 140.0);
    let camada = t.add_raster_layer("nova").expect("a camada nova");
    t.select_layer(camada);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    let n1 = traco(&mut t, 220.0);
    let antes = t.canvas_rgba.to_vec();
    let n2 = traco(&mut t, 300.0);
    let depois = t.canvas_rgba.to_vec();
    assert!(t.undo_last(), "o 2.º traço da camada nova desfaz-se");
    let desfeito = t.canvas_rgba.to_vec();
    assert!(t.redo_last(), "e refaz-se");
    let refeito = t.canvas_rgba.to_vec();
    CAPTURA_PELA_CAIXA.set(true);
    CAPTURA_PARTILHADA.set(true);
    Corrida {
        picos: [b1, b2, n1, n2],
        antes,
        depois,
        desfeito,
        refeito,
    }
}

#[test]
fn o_desfazer_do_composite_wet_guarda_a_caixa_e_nao_a_tela() {
    confere(false);
}

/// O mesmo no Watercolor Basic (`watercolor_render`, n.º 6): 2.º pen-down da base a 4K 28–29 → 16–17 ms.
#[test]
fn o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela() {
    confere(true);
}

fn confere(aguada: bool) {
    let tela = (W * H * 4) as usize;
    let velho = corre(false, aguada);
    let novo = corre(true, aguada);
    eprintln!(
        "pico do journal da tela (B; tela {tela}): antigo {:?} · novo {:?}",
        velho.picos, novo.picos
    );
    assert!(
        velho.picos.iter().any(|&p| p >= tela),
        "CONTROLO: o caminho antigo captura a tela inteira ({:?} de {tela})",
        velho.picos
    );
    for (i, &p) in novo.picos.iter().enumerate() {
        assert!(
            p < tela / 2,
            "traço {i}: o journal reteve {p} B de {tela} (a caixa, não a tela)"
        );
    }
    assert_ne!(novo.antes, novo.depois, "o traço pinta");
    assert!(
        novo.depois == velho.depois,
        "os dois caminhos pintam os mesmos bytes"
    );
    assert!(
        novo.desfeito == novo.antes,
        "o desfazer devolve a tela de antes do traço"
    );
    assert!(velho.desfeito == velho.antes, "CONTROLO: o antigo também");
    assert!(novo.refeito == novo.depois, "o refazer devolve o traço");
}
