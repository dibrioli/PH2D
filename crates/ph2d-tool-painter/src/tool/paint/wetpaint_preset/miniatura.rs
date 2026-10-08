//! **A miniatura de um preset** — o traço dele pintado pelo motor, numa tela branca de
//! [`LADO_MINIATURA`]², no relógio FIXO: o mesmo preset dá os mesmos bytes, e por isso as miniaturas
//! da fábrica são o corpus de regressão do motor (uma mudança de lei acusa em todos de uma vez).
//!
//! O gesto: um S de margem a margem, a passo de rato (3 px), com a pressão da rampa da bancada
//! (`0,15 → 1 → 0,15`, doc 03), e 2 s de água a correr depois do pen-up.

use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::{LADO_MINIATURA, Miniatura, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// O diâmetro do traço da miniatura quando o preset não fixa o tamanho — o `size_px` da bancada
/// (doc 03: o Rebelle e a sonda pintam a 40).
pub const DIAMETRO_DA_MINIATURA: f32 = 40.0;
/// O maior diâmetro que cabe no S sem tocar as margens (o S oscila ±[`AMPLITUDE`] em torno do meio).
const DIAMETRO_MAX: f32 = 96.0;
const DIAMETRO_MIN: f32 = 2.0;
const MARGEM: f32 = 28.0;
const AMPLITUDE: f32 = 40.0;
const PASSO_PX: f32 = 3.0;
const TICK_MS: f32 = 8.0;
/// 2 s de água depois do pen-up, a 60 Hz.
const QUADROS_DEPOIS: u32 = 120;
/// A tinta da miniatura (o azul da bancada, `sonda_gestos::AZUL`).
const AZUL: [u8; 3] = [23, 101, 179];

fn ponto(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Pinta a miniatura de `p`.
#[must_use]
pub fn miniatura_do_preset(p: &Preset) -> Miniatura {
    let lado = LADO_MINIATURA;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (lado * lado * 4) as usize], lado, lado);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    // Uma textura que não se leu já foi dita a quem aplicou o preset; a miniatura pinta o que há.
    let _ = t.wet_preset_aplicar(p);
    let d = p
        .parametros
        .volumes
        .size_px
        .map_or(DIAMETRO_DA_MINIATURA, |d| d as f32)
        .clamp(DIAMETRO_MIN, DIAMETRO_MAX);
    t.set_brush_size_px(d * 0.5);
    t.set_brush_color_srgb8(AZUL);

    let l = lado as f32;
    let (x0, x1, cy) = (MARGEM, l - MARGEM, l * 0.5);
    let n = ((x1 - x0) / PASSO_PX).ceil() as u32;
    let em = |f: f32| {
        [
            x0 + (x1 - x0) * f,
            cy + AMPLITUDE * (f * std::f32::consts::TAU).sin(),
        ]
    };
    let pressao = |f: f32| 0.15 + 0.85 * (f * std::f32::consts::PI).sin();
    t.on_canvas_pointer(ponto(em(0.0), pressao(0.0), PointerPhase::Down));
    for k in 1..=n {
        let f = k as f32 / n as f32;
        t.on_canvas_pointer(ponto(em(f), pressao(f), PointerPhase::Move));
        t.on_tick(TICK_MS);
    }
    t.on_canvas_pointer(ponto(em(1.0), pressao(1.0), PointerPhase::Up));
    for _ in 0..QUADROS_DEPOIS {
        t.on_tick(1000.0 / 60.0);
    }
    let (rgba, w, h) = t.run_full();
    Miniatura {
        largura: w,
        altura: h,
        rgba,
    }
}
