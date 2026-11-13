use super::*;
use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn ev(i: u32) -> Evento {
    let t = i as f32;
    Evento {
        pos: (40.0 + 7.0 * t, 60.0 + 3.0 * (t * 0.7).sin() * 20.0),
        pressure: 0.3 + 0.7 * ((t * 0.37).sin() * 0.5 + 0.5),
        tilt: [(t * 0.11).sin() * 0.5, (t * 0.13).cos() * 0.4],
        shift: false,
        ctrl: false,
        alt: false,
    }
}

fn cpt(e: Evento, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [e.pos.0, e.pos.1],
        pressure: e.pressure,
        tilt: e.tilt,
        phase,
    }
}

fn tela(t: &mut PainterTool) -> Vec<u8> {
    t.take_preview_arc()
        .map(|(d, _, _)| d.to_vec())
        .unwrap_or_default()
}

/// `por_quadro = None`: o caminho antigo (entrega no handler, o oráculo); `Some(k)`: a fila, drenada a
/// cada `k` eventos (o quadro), como a shell.
fn traco(por_quadro: Option<usize>) -> Vec<u8> {
    const N: u32 = 48;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
    t.set_brush_size_px(12.0);
    descarta();
    t.on_canvas_pointer(cpt(ev(0), PointerPhase::Down));
    for i in 1..N {
        match por_quadro {
            None => {
                t.on_canvas_pointer(cpt(ev(i), PointerPhase::Move));
            }
            Some(k) => {
                enfileira(ev(i));
                if pendentes() >= k {
                    drena(|e| t.on_canvas_pointer(cpt(e, PointerPhase::Move)));
                }
            }
        }
    }
    drena(|e| t.on_canvas_pointer(cpt(e, PointerPhase::Move)));
    t.on_canvas_pointer(cpt(ev(N - 1), PointerPhase::Up));
    tela(&mut t)
}

/// B27.1: a mesma sequência de caneta dá o mesmo byte, drenada com 1, 3, 16 ou todos os eventos por quadro.
#[test]
fn a_fila_da_caneta_da_o_mesmo_traco_ao_bit() {
    let oraculo = traco(None);
    assert!(
        oraculo.iter().any(|&b| b != 255),
        "o traço não pintou: a régua não mede nada"
    );
    for k in [1, 3, 16, usize::MAX] {
        assert!(
            traco(Some(k)) == oraculo,
            "{k} eventos/quadro mudaram o traço"
        );
    }
}

/// Um `false` (o traço morreu) descarta o resto: nada atravessa para o traço seguinte.
#[test]
fn a_fila_descarta_o_resto_quando_o_traco_morre() {
    descarta();
    for i in 0..5 {
        enfileira(ev(i));
    }
    let mut vistos = Vec::new();
    let d = drena(|e| {
        vistos.push(e);
        vistos.len() < 2
    });
    assert_eq!((d.entregues, d.vivo, pendentes()), (1, false, 0));
    assert_eq!(vistos, vec![ev(0), ev(1)]);
}
