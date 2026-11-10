//! **B12.A1: o tique não compra a espera do `TICK_WAIT` fora do `mostra`** (filho de [`super`] por `#[path]`).
//!
//! A fixture põe o worker SEM o motor (o slot diz `Away`, o canal de volta fica vazio para sempre): é o pior caso
//! da espera — cada pedido com `TICK_WAIT` paga os 4 ms inteiros, quadro após quadro, enquanto os factos mudam.
//! Mutação que sangra: a ternária de `wetpaint_tick` (`ciclo.rs`) de volta ao `try_bring_home()` de sempre.

use super::*;
use ph2d_painter_brush::Falloff;
use std::time::{Duration, Instant};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma poça de 256² com um traço — o custo do passo não entra: o que se mede é a espera.
fn poca() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
    let b = BrushSpec {
        radius_px: 20.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::WetPaint);
    t.on_canvas_pointer(cp([30.0, 128.0], PointerPhase::Down));
    for k in 1..=12 {
        t.on_canvas_pointer(cp([30.0 + 8.0 * k as f32, 128.0], PointerPhase::Move));
        let _ = t.take_preview_arc();
    }
    t.on_canvas_pointer(cp([126.0, 128.0], PointerPhase::Up));
    t
}

/// **Os factos a mudar com o motor longe custam ≤ 1 ms de espera por quadro** (antes: o `TICK_WAIT` inteiro, 4 ms,
/// em cada quadro). O MÍNIMO de N quadros é a régua: a versão que espera nunca desce dos 4 ms.
#[test]
fn os_factos_com_o_motor_longe_nao_esperam_o_tick_wait() {
    const QUADROS: u8 = 24;
    let mut t = poca();
    t.wetpaint_tick(1.0 / 60.0);
    t.wet_bring_home();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let EngineSlot::Here(motor) = std::mem::replace(&mut sess.engine, EngineSlot::Away) else {
        panic!("CONTROLO: o wet_bring_home não trouxe o motor para casa");
    };
    // CONTROLO: a porta que espera paga o `TICK_WAIT` nesta fixture — a régua discrimina.
    let a = Instant::now();
    assert!(
        !sess.try_bring_home(),
        "CONTROLO: o canal vazio devolveu um motor"
    );
    let espera_antiga = a.elapsed();
    assert!(
        espera_antiga >= Duration::from_millis(3),
        "CONTROLO: a porta com espera voltou em {espera_antiga:?} — a fixture não mede o TICK_WAIT"
    );
    let mut minimo = Duration::MAX;
    let mut soma = Duration::ZERO;
    for k in 0..QUADROS {
        t.paint.wetpaint.tilt_ring = 1 + k % 7;
        let a = Instant::now();
        t.wetpaint_tick(1.0 / 60.0);
        let d = a.elapsed();
        minimo = minimo.min(d);
        soma += d;
    }
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    assert!(
        sess.motor_pedido(),
        "CONTROLO: o tique deixou de pedir o motor — os factos não chegaram à porta"
    );
    sess.engine = EngineSlot::Here(motor);
    eprintln!(
        "B12.A1: mínimo {minimo:?}, média {:?} por quadro (a porta com espera: {espera_antiga:?})",
        soma / u32::from(QUADROS)
    );
    assert!(
        minimo <= Duration::from_millis(1),
        "o tique esperou o motor com os factos a mudar: mínimo {minimo:?}, média {:?} por quadro (a porta com \
         espera: {espera_antiga:?})",
        soma / u32::from(QUADROS)
    );
}
