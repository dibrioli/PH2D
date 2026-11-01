//! **Sonda (ignorada) da Absorbency** (E10d2 §3): um traço Wet Round e um Ink Brush, papel por omissão, a lei do
//! produto, com a Absorbency em {0 · 0,25 · 1 · 4 · 16 · 64}, os casos intercalados no MESMO processo.
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib absorcao_sonda -- --ignored --nocapture`
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 400;
const H: usize = 200;
const Y: f32 = 100.0;

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, Y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// (Σ film, células molhadas, passos do motor).
fn agua(t: &mut PainterTool) -> (f64, usize, u64) {
    let mut r = (0.0, 0usize, 0u64);
    t.wet_sonda_motor(|e| {
        let g = e.active_grid();
        for y in 1..g.h {
            for x in 1..g.w {
                let f = f64::from(g.film[x + y * g.s]);
                r.0 += f;
                if f > 0.01 {
                    r.1 += 1;
                }
            }
        }
        r.2 = e.sim.frame;
    });
    r
}

/// A largura 10–90 % da orla de cima do traço no ecrã, na coluna do meio (px): a borda macia é larga.
fn orla(t: &PainterTool) -> usize {
    let x = W / 2;
    let escuro: Vec<f64> = (0..H)
        .map(|y| 255.0 - f64::from(t.canvas_rgba[(x + y * W) * 4]))
        .collect();
    let pico = escuro.iter().copied().fold(0.0, f64::max);
    if pico <= 0.0 {
        return 0;
    }
    let y10 = escuro.iter().position(|&v| v >= 0.1 * pico).unwrap_or(0);
    let y90 = escuro.iter().position(|&v| v >= 0.9 * pico).unwrap_or(0);
    y90.saturating_sub(y10)
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn absorcao_sonda() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    println!(
        "pincel|A|agua_t0|agua_t30|agua_t120|molhadas_t0|t30|t120|orla_px_t120|orla_px_seco|seca_em_ticks|passos"
    );
    for nome in ["Wet Round", "Ink Brush"] {
        let p = &b
            .entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset;
        for a in [0.0, 0.25, 1.0, 4.0, 16.0, 64.0] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
            t.set_paint_tool_mode("wetpaint");
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(p).is_empty());
            t.wet_preset_aplicar_volumes(&Volumes {
                size_px: Some(40.0),
                ..p.parametros.volumes.clone()
            });
            t.paint.wetpaint.tilt_on = false;
            t.set_wet_absorcao(a);
            t.set_brush_color_srgb8([23, 101, 179]);
            t.on_canvas_pointer(ponto(100.0, PointerPhase::Down));
            for k in 1..=50 {
                t.on_canvas_pointer(ponto(100.0 + 4.0 * k as f32, PointerPhase::Move));
                t.on_tick(8.0);
            }
            t.on_canvas_pointer(ponto(300.0, PointerPhase::Up));
            t.on_tick(1000.0 / 60.0);
            let a0 = agua(&mut t);
            for _ in 0..30 {
                t.on_tick(1000.0 / 60.0);
            }
            let a30 = agua(&mut t);
            for _ in 30..120 {
                t.on_tick(1000.0 / 60.0);
            }
            let a120 = agua(&mut t);
            let o120 = orla(&t);
            let mut seca = None;
            for k in 120..60 * 60 {
                t.on_tick(1000.0 / 60.0);
                if k % 30 == 0 && agua(&mut t).1 == 0 {
                    seca = Some(k);
                    break;
                }
            }
            let fim = agua(&mut t);
            println!(
                "{nome}|{a}|{:.0}|{:.0}|{:.0}|{}|{}|{}|{o120}|{}|{}|{}",
                a0.0,
                a30.0,
                a120.0,
                a0.1,
                a30.1,
                a120.1,
                orla(&t),
                seca.map_or("> 3600".to_owned(), |k| k.to_string()),
                fim.2
            );
        }
    }
}
