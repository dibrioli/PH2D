//! **Sonda (ignorada): a mistura de pigmento do Wet Paint é em tempo real? Molhado ≠ seco?** (dono 09/10:
//! *«pigment mixing em wet paint não acontece em tempo real mas apenas no mouse up. Além disso, parece que a
//! mistura acontece da mesma maneira sobre tinta seca e tinta úmida»*). Um traço VERMELHO horizontal; depois
//! um AZUL vertical que o cruza, com o vermelho ainda MOLHADO (0,5 s) ou SECO (Dry canvas). Lê-se o pixel no
//! cruzamento e o rasto do azul DEPOIS do cruzamento (o pincel sujo arrasta vermelho ⇒ roxo) DURANTE o traço
//! (antes do Up) e depois do Up + 2 s.
//!
//! `PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --lib sonda_mistura -- --ignored
//! --nocapture` e `python3 -I docs/Painter/NewFeaturesWetPaint/ferramentas/velatura.py
//! /tmp/x/mistura_seco_durante.ppm /tmp/x/mistura_seco_depois.ppm` (e o molhado).
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};

const W: usize = 600;
const H: usize = 300;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A tela em PPM (`PH2D_SONDA_OUT`) para a régua `ferramentas/velatura.py`.
fn grava(t: &PainterTool, nome: &str) {
    let Ok(dir) = std::env::var("PH2D_SONDA_OUT") else {
        return;
    };
    let mut px = format!("P6 {W} {H} 255\n").into_bytes();
    for k in 0..W * H {
        px.extend_from_slice(&t.canvas_rgba[k * 4..k * 4 + 3]);
    }
    std::fs::write(format!("{dir}/mistura_{nome}.ppm"), px).expect("grava a tela");
}

fn media(t: &PainterTool, x0: usize, x1: usize, y0: usize, y1: usize) -> [f64; 3] {
    let mut s = [0.0; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * W + x) * 4;
            for c in 0..3 {
                s[c] += f64::from(t.canvas_rgba[i + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| (v / n).round())
}

#[test]
#[ignore = "sonda: a mistura do Wet Paint; corre-se à mão"]
fn sonda_mistura() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    for seco in [false, true] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
        t.wet_preset_aplicar_volumes(&p.parametros.volumes);
        t.paint.wetpaint.tilt_on = false;
        t.set_brush_size_px(30.0);
        // As VARIANTES (W1e): `PH2D_SONDA_VELATURA=<γ>` e `PH2D_SONDA_NO_TRACO=0|1` sobrepõem-se ao produto.
        assert!(t.ensure_wet_session());
        if let Some(s) = t.paint.wetpaint.session.as_mut() {
            if let Some(g) = std::env::var("PH2D_SONDA_VELATURA")
                .ok()
                .and_then(|v| v.parse().ok())
            {
                s.engine.sim.glaze = g;
            }
            if let Ok(v) = std::env::var("PH2D_SONDA_NO_TRACO") {
                s.engine.sim.sim_while_painting = v == "1";
            }
        }
        // O vermelho, horizontal.
        t.set_brush_color_srgb8([200, 30, 30]);
        t.on_canvas_pointer(cp([100.0, 150.0], PointerPhase::Down));
        for k in 1..=133 {
            t.on_canvas_pointer(cp([100.0 + 3.0 * k as f32, 150.0], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp([500.0, 150.0], PointerPhase::Up));
        for _ in 0..30 {
            t.on_tick(1000.0 / 60.0);
        }
        if seco {
            assert!(t.route_brush_wetpaint_event(&PanelEvent::Click(
                crate::ids::PAINTER_WETPAINT_DRYCANVAS
            )));
            for _ in 0..30 {
                t.on_tick(1000.0 / 60.0);
            }
        }
        let antes = media(&t, 295, 306, 145, 156);
        // O azul, vertical, de cima para baixo, a cruzar o vermelho em y = 150.
        t.set_brush_color_srgb8([30, 60, 210]);
        t.on_canvas_pointer(cp([300.0, 40.0], PointerPhase::Down));
        let mut k: u16 = 0;
        while 40.0 + 3.0 * f32::from(k) < 260.0 {
            k += 1;
            t.on_canvas_pointer(cp([300.0, 40.0 + 3.0 * f32::from(k)], PointerPhase::Move));
            t.on_tick(8.0);
        }
        for _ in 0..6 {
            t.on_tick(1000.0 / 60.0);
        }
        let estado = if seco { "seco" } else { "molhado" };
        grava(&t, &format!("{estado}_durante"));
        let (cruz_vivo, rasto_vivo, azul_vivo) = (
            media(&t, 295, 306, 145, 156),
            media(&t, 295, 306, 185, 225),
            media(&t, 295, 306, 60, 100),
        );
        t.on_canvas_pointer(cp([300.0, 260.0], PointerPhase::Up));
        for _ in 0..120 {
            t.on_tick(1000.0 / 60.0);
        }
        grava(&t, &format!("{estado}_depois"));
        let (cruz_fim, rasto_fim, azul_fim) = (
            media(&t, 295, 306, 145, 156),
            media(&t, 295, 306, 185, 225),
            media(&t, 295, 306, 60, 100),
        );
        println!(
            "[mistura] vermelho {:<7} | cruzamento antes do azul {antes:?} | DURANTE: cruz {cruz_vivo:?} rasto depois {rasto_vivo:?} azul limpo {azul_vivo:?} | DEPOIS do Up: cruz {cruz_fim:?} rasto {rasto_fim:?} azul {azul_fim:?}",
            if seco { "SECO" } else { "MOLHADO" }
        );
    }
}
