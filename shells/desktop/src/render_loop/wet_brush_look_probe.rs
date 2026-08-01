//! **RENDER-AND-LOOK do pincel grande do Wet Paint** — a wave que tirou o cap
//! do `TRAIL_HALF` mudou a APARÊNCIA do traço, e aparência gate nenhum julga.
//!
//! ⚠️ **Escrito ANTES de pedir um smoke ao Enio**, que é a disciplina desta
//! linha (*o método foi o prescrito: RENDERIZAR E OLHAR*, doc 28 §5.13). O
//! risco concreto tem nome: a **tile de cerdas do modelo é indexada em CÉLULAS**
//! e foi construída para o teto de raio 35 da referência JS — agora ela é
//! esticada até 400, e ninguém olhou.
//!
//! Escreve um PNG por raio em `PH2D_WET_LOOK_DIR`. Diagnóstico, `#[ignore]`d.

use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

#[test]
#[ignore = "render-and-look; PH2D_WET_LOOK_DIR=<dir> cargo test ... -- --ignored"]
fn probe_wet_brush_render_and_look() {
    let Ok(dir) = std::env::var("PH2D_WET_LOOK_DIR") else {
        eprintln!("set PH2D_WET_LOOK_DIR");
        return;
    };
    const SIDE: u32 = 1400;
    for radius in [35.0f32, 100.0, 200.0, 400.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_brush_size_px(radius);
        t.set_brush_color_srgb8([32, 64, 148]);
        // Um traço em S: a curva é onde uma tile esticada se denuncia (uma
        // reta pode esconder repetição, uma curva não).
        let (x0, y0) = (240.0f32, 700.0f32);
        t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
        for k in 1..=44u32 {
            let d = 20.0 * k as f32;
            let y = y0 + 180.0 * (d / 260.0).sin();
            t.on_canvas_pointer(cp([x0 + d, y], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([x0 + 880.0, y0], PointerPhase::Up));
        for _ in 0..30 {
            t.on_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        // Invalidação byte-neutra: o `take_preview_arc` do laço acima drenou o
        // dirty, e sem isto o pedido final volta VAZIO (foi o que a 1ª corrida
        // desta sonda reportou como "sem composite" nos quatro raios).
        let active = t.layers().active().expect("layer");
        let cur = t.layers().get(active).expect("layer").opacity;
        t.set_layer_opacity(active, cur);
        let Some((rgba, w, h)) = t.take_preview_arc() else {
            eprintln!("  raio {radius:.0}: sem composite");
            continue;
        };
        // O que a MEDIÇÃO diz, ao lado do que o olho vai ver: cobertura da
        // banda e o contraste local (a assinatura de uma tile esticada é
        // cobertura alta com contraste BAIXO — o traço vira chapa).
        let (mut painted, mut sum, mut sq) = (0u64, 0f64, 0f64);
        for o in (0..(w as usize * h as usize * 4)).step_by(4) {
            let v = f64::from(rgba[o]);
            if v < 250.0 {
                painted += 1;
                sum += v;
                sq += v * v;
            }
        }
        let n = painted.max(1) as f64;
        let sd = (sq / n - (sum / n) * (sum / n)).max(0.0).sqrt();
        println!(
            "  raio {radius:>3.0}p  texels {painted:>8}  media {:>6.1}  desvio {sd:>6.1}",
            sum / n
        );
        let _ = image::save_buffer(
            format!("{dir}/wet_r{radius:.0}.png"),
            &rgba[..],
            w,
            h,
            image::ColorType::Rgba8,
        );
    }
    println!("\n  PNGs em {dir}. O desvio e a regua: uma tile ESTICADA cobre muito");
    println!("  e varia pouco -- o traco vira chapa em vez de cerda.");
}
