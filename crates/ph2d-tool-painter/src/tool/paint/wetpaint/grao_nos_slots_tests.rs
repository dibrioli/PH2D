//! **O Grain aplicado e os caminhos fora do traço do Pincel** (Bug #65, auditoria 05/11): a Borracha (A1), a macro (A2),
//! o slot único da imagem (A4) e o valor lido do ficheiro (D4). Medições: `docs/Painter/BUGS_painter.md` #65.
use super::same_as_paper_volume_tests::{H, W, le, ponto, traco, traco_com};
use crate::tool::PainterTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

fn imagem(v: u8) -> Vec<u8> {
    vec![v; 64 * 64]
}

/// Uma imagem com desenho, de média ~0,5.
fn imagem_com_desenho() -> Vec<u8> {
    (0..64 * 64)
        .map(|i| {
            if (i % 64) / 8 % 2 == (i / 64) / 8 % 2 {
                240
            } else {
                16
            }
        })
        .collect()
}

fn difs(a: &PainterTool, b: &PainterTool) -> usize {
    a.canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .filter(|(x, y)| x != y)
        .count()
}

fn risca(t: &mut PainterTool, x0: f32, n: usize) {
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(ponto(x0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(x0 + 4.0 * n as f32, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tinta na tela e a Borracha na mão, `muda`da, a passar por cima: `(massa apagada, a ferramenta)`.
fn apaga(muda: impl FnOnce(&mut PainterTool)) -> (f64, PainterTool) {
    let (antes, mut t) = traco(&de_fabrica_ou_candidato("Gouache Opaque"), false);
    t.set_paint_tool_mode("eraser");
    t.set_brush_size_px(40.0);
    muda(&mut t);
    risca(&mut t, 100.0, 50);
    (antes[0] - le(&t)[0], t)
}
