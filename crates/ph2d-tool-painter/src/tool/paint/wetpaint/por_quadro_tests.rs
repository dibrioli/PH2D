//! **R3: a tela do Wet compõe-se por quadro** ([`super::por_quadro`]) — e é a MESMA tela, ao byte, que a
//! composição por evento: depois de cada traço, no meio de um (a drenagem do quadro), e depois do Ctrl+Z.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

/// Eventos por quadro (o oráculo mandou um a cada 2 ms).
const EV: usize = 8;

/// As fotografias da tela de uma cena: cada drenagem a meio do 1.º traço, o fim de cada traço e o Ctrl+Z.
fn cena(por_quadro: bool, oleo: bool) -> Vec<Vec<u8>> {
    let (w, h) = (320usize, 200usize);
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; w * h * 4], w as u32, h as u32);
    t.set_compor_por_quadro(por_quadro);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    if oleo {
        t.set_wet_oleo_real_ligado(true);
    }
    let mut fotos = Vec::new();
    let foto = |t: &PainterTool| t.imagem_aplicada().expect("tela").0;
    for (k, (raio, y, cor)) in [
        (14.0, 60.0, [180u8, 40, 30]),
        (40.0, 110.0, [30, 60, 190]),
        (6.0, 90.0, [20, 150, 60]),
    ]
    .into_iter()
    .enumerate()
    {
        t.set_brush_size_px(raio);
        t.set_brush_color_srgb8(cor);
        let p = |x: f32, phase| CanvasPointer {
            pos: [x, y + 0.2 * x],
            pressure: 0.9,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(p(30.0, PointerPhase::Down));
        for i in 1..=80 {
            t.on_canvas_pointer(p(30.0 + 3.0 * i as f32, PointerPhase::Move));
            if i % EV == 0 {
                t.on_tick(16.7);
                let _ = t.take_preview_arc();
                if k == 0 {
                    fotos.push(foto(&t));
                }
            }
        }
        t.on_canvas_pointer(p(270.0, PointerPhase::Up));
        let _ = t.take_preview_arc();
        fotos.push(foto(&t));
    }
    assert!(t.undo_last(), "o Ctrl+Z não desfez");
    let _ = t.take_preview_arc();
    fotos.push(foto(&t));
    fotos
}

fn confere(oleo: bool) {
    let evento = cena(false, oleo);
    let quadro = cena(true, oleo);
    assert_eq!(evento.len(), quadro.len());
    // Controlo: a cena pinta (as fotos mudam), senão a igualdade não prova nada.
    assert_ne!(evento[0], evento[evento.len() - 2], "a cena não pintou");
    for (i, (a, b)) in evento.iter().zip(&quadro).enumerate() {
        let n = a.iter().zip(b).filter(|(x, y)| x != y).count();
        assert_eq!(n, 0, "foto {i} (óleo {oleo}): {n} bytes diferem entre evento e quadro");
    }
}

#[test]
fn a_tela_por_quadro_e_a_tela_por_evento_ao_byte() {
    confere(false);
}

#[test]
fn a_tela_por_quadro_e_a_tela_por_evento_ao_byte_com_o_oleo_real() {
    confere(true);
}

/// O adiamento acontece de facto: com o hospedeiro a drenar, um Move não compõe (a régua do corte).
#[test]
fn com_o_hospedeiro_a_drenar_o_move_nao_compoe() {
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; 64 * 64 * 4], 64, 64);
    t.set_compor_por_quadro(true);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(8.0);
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, 32.0],
        pressure: 0.9,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(10.0, PointerPhase::Down));
    let mut adiou = false;
    for i in 1..=10 {
        t.on_canvas_pointer(p(10.0 + 3.0 * i as f32, PointerPhase::Move));
        adiou |= t.paint.wetpaint.composite_pendente;
    }
    assert!(adiou, "nenhum Move adiou a composição");
    let _ = t.take_preview_arc();
    assert!(!t.paint.wetpaint.composite_pendente, "a drenagem não compôs o pendente");
    t.on_canvas_pointer(p(50.0, PointerPhase::Up));
}
