//! Os cinco gestos da bancada Rebelle × PH2D (`docs/Painter/NewFeaturesWetPaint/03`), numa tela
//! 1000×640 branca: um só desenho para a sonda (`sonda_tracos_por_meio`) e para a cerca
//! (`os_outros_meios_nao_mudam`). Os mesmos gestos vão ao Rebelle pela porta WebSocket
//! (`ferramentas/gera_eventos.py`); mudar uma conta aqui muda as duas metades da comparação.

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

pub const W: u32 = 1000;
pub const H: u32 = 640;
const AZUL: [u8; 3] = [23, 101, 179];
const VERMELHO: [u8; 3] = [191, 0, 18];

fn cp(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um gesto por uma polilinha, com `passo` px entre eventos e a pressão dada por `p(t)` em `t ∈ [0, 1]`.
fn gesto(t: &mut PainterTool, pts: &[[f32; 2]], passo: f32, p: impl Fn(f32) -> f32) {
    let total: f32 = pts
        .windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .sum();
    let mut andado = 0.0;
    t.on_canvas_pointer(cp(pts[0], p(0.0), PointerPhase::Down));
    for w in pts.windows(2) {
        let dx = w[1][0] - w[0][0];
        let dy = w[1][1] - w[0][1];
        let len = (dx * dx + dy * dy).sqrt();
        let n = (len / passo).ceil().max(1.0) as u32;
        for k in 1..=n {
            let f = k as f32 / n as f32;
            andado += len / n as f32;
            t.on_canvas_pointer(cp(
                [w[0][0] + dx * f, w[0][1] + dy * f],
                p(andado / total),
                PointerPhase::Move,
            ));
            t.on_tick(8.0);
        }
    }
    let fim = pts[pts.len() - 1];
    t.on_canvas_pointer(cp(fim, p(1.0), PointerPhase::Up));
}

fn curva_s(x0: f32, x1: f32, cy: f32, amp: f32) -> Vec<[f32; 2]> {
    (0..=120)
        .map(|i| {
            let f = i as f32 / 120.0;
            [
                x0 + (x1 - x0) * f,
                cy + amp * (f * std::f32::consts::TAU * 1.5).sin(),
            ]
        })
        .collect()
}

/// A cena inteira num meio, com o pincel de fábrica dele a `size` px: devolve `run_full()` (RGBA8).
pub fn pinta(meio: PaintMedia, size: f32) -> (Vec<u8>, u32, u32) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(meio);
    t.set_brush_size_px(size);
    t.set_brush_strength(1.0);
    t.set_brush_color_srgb8(AZUL);
    let cheio = |_: f32| 1.0f32;
    // Linha 1 — recto a passo de rato · curva em S.
    gesto(&mut t, &[[60.0, 120.0], [560.0, 120.0]], 3.0, cheio);
    gesto(&mut t, &curva_s(600.0, 960.0, 120.0, 25.0), 3.0, cheio);
    // Linha 2 — rampa de pressão (0,15 → 1 → 0,15) · traço RÁPIDO (25 px por evento).
    gesto(&mut t, &[[60.0, 300.0], [560.0, 300.0]], 3.0, |f| {
        0.15 + 0.85 * (f * std::f32::consts::PI).sin()
    });
    gesto(&mut t, &[[600.0, 300.0], [960.0, 300.0]], 25.0, cheio);
    // Linha 3 — recto, e um cruzamento vermelho (a mistura / o «dirty brush»).
    gesto(&mut t, &[[60.0, 480.0], [560.0, 480.0]], 3.0, cheio);
    t.set_brush_color_srgb8(VERMELHO);
    gesto(&mut t, &[[300.0, 400.0], [300.0, 560.0]], 3.0, cheio);
    gesto(&mut t, &[[700.0, 480.0], [960.0, 480.0]], 3.0, cheio);
    // O tempo passa (a água corre e seca; nos outros meios é inócuo): 8 s a 60 Hz.
    for _ in 0..480 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full()
}

/// FNV-1a de 64 bits — o mesmo hash dos outros gates de pixels desta crate.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
