//! O `ExtDryBrush` (Tuning «Dry brush») é uma lei VIVA, não morta: o `dry_gate` do dab é `0` com o `b` do
//! toque inteiro (a fábrica), mas a cobertura que cai (a velocidade → opacidade) baixa o `b` e abre o portão.

use crate::LinhaDaMao;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;
use std::sync::{Arc, Mutex};

const W: u32 = 360;
const H: u32 = 80;

/// Um traço RÁPIDO (24 px por quadro a 60 Hz ≈ 1 440 px/s) e o maior `dry_gate` que o motor viu.
fn pinta(rapido_seco: bool, dry_brush: f64) -> (Vec<u8>, f64) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    if rapido_seco {
        t.set_mao(LinhaDaMao::Knob(Knob::PressureOpacity), 0.0);
        t.set_mao(LinhaDaMao::SpeedOpacity, LinhaDaMao::SpeedOpacity.faixa().1);
    }
    t.paint.wetpaint.knobs.set(Knob::ExtDryBrush, dry_brush);
    assert!(t.ensure_wet_session());
    let visto = Arc::new(Mutex::new(0.0f64));
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        let v = Arc::clone(&visto);
        s.engine.on_dab = Some(Box::new(move |_, d| {
            let mut m = v.lock().unwrap();
            *m = m.max(d.dry_gate);
        }));
    }
    let y = H as f32 / 2.0;
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=13 {
        t.on_canvas_pointer(cp(20.0 + 24.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(20.0 + 24.0 * 13.0, PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_bring_home();
    let g = *visto.lock().unwrap();
    (t.canvas_rgba.to_vec(), g)
}

/// ⭐ Na fábrica o portão está fechado (`dry_gate = 0`) e o Dry brush não muda um byte; com a cobertura a cair
/// (a velocidade → opacidade, sem a pressão na opacidade) o portão abre e o Dry brush tira tinta.
#[test]
fn o_extdrybrush_so_mexe_com_dry_gate_positivo() {
    let (fabrica_0, g) = pinta(false, 0.0);
    assert_eq!(g, 0.0, "a fábrica: o b do toque inteiro fecha o portão");
    let (fabrica_1, _) = pinta(false, 1.0);
    assert!(fabrica_0 != vec![255u8; fabrica_0.len()], "o traço pinta");
    assert_eq!(fabrica_0, fabrica_1, "portão fechado: o Dry brush é inerte");

    let (seco_0, g) = pinta(true, 0.0);
    assert!(g > 0.0, "a cobertura que cai abre o portão: dry_gate {g}");
    let (seco_1, _) = pinta(true, 1.0);
    let tinta = |v: &[u8]| v.iter().map(|&c| u64::from(255 - c)).sum::<u64>();
    assert!(
        tinta(&seco_1) < tinta(&seco_0),
        "o Dry brush tira tinta com o portão aberto: {} -> {}",
        tinta(&seco_0),
        tinta(&seco_1)
    );
}
