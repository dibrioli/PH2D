//! RENDER-AND-LOOK da razão da grade do fluido (doc 28 §5.41) — o oráculo de um
//! artefato de APARÊNCIA é a imagem, não um número.
//!
//! Escreve PNGs de traços de Wet Paint em várias razões para
//! `PH2D_WET_LOOK_DIR`. Irmã do `push_look_probe`; diagnóstica, `#[ignore]`d.
//!
//! ⚠️ **Por que ela existe:** o Enio reprovou a borda com uma FOTO (2026-07-29,
//! *"precisaremos de um AA de baixo custo"*), e três métricas minhas em sequência
//! mediram a coisa errada — a serra RMS do contorno dá **2,28 px na razão 1** e
//! 0,45 na razão 4, isto é, ela mede a granulação esparsa do banco de cerdas
//! (~5 % de cobertura por célula) e não a estrutura de grade que o olho vê. *Um
//! número no lugar errado diz o contrário da foto* (doc 25 §13.10).

use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tool(size: u32, ratio: u8, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(radius * 2.0);
    // O ocre da foto do Enio.
    t.set_brush_color_srgb8([196, 150, 42]);
    t.set_wet_grid_ratio(f64::from(ratio));
    t
}

fn save(t: &mut PainterTool, dir: &str, name: &str) {
    let (rgba, w, h) = t.take_preview_arc().expect("composite");
    let _ = image::save_buffer(
        format!("{dir}/{name}.png"),
        &rgba[..],
        w,
        h,
        image::ColorType::Rgba8,
    );
    eprintln!("   {name}.png");
}

/// Os traços PENDENTES da foto: quatro verticais que descem de cima e terminam
/// com ponta arredondada — a figura onde o Enio apontou a borda.
fn hanging_strokes(t: &mut PainterTool, size: f32) {
    for (k, (x, len)) in [(0.22f32, 0.42f32), (0.40, 0.55), (0.58, 0.80), (0.76, 0.35)]
        .into_iter()
        .enumerate()
    {
        let px = size * x;
        let end = size * len;
        t.on_canvas_pointer(cp([px, 0.0], PointerPhase::Down));
        let steps = 24;
        for i in 1..=steps {
            let y = end * i as f32 / steps as f32;
            t.on_canvas_pointer(cp([px, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([px, end], PointerPhase::Up));
        let _ = k;
    }
}

#[test]
#[ignore = "render-and-look probe; PH2D_WET_LOOK_DIR=<dir> cargo test ... -- --ignored"]
fn probe_wet_grid_render_and_look() {
    let Some(dir) = std::env::var("PH2D_WET_LOOK_DIR").ok() else {
        eprintln!("set PH2D_WET_LOOK_DIR");
        return;
    };
    let size = 512u32;
    eprintln!("  os tracos pendentes da foto, por razao de grade:");
    for ratio in [1u8, 4, 8, 16, 30] {
        let mut t = tool(size, ratio, 26.0);
        hanging_strokes(&mut t, size as f32);
        save(&mut t, &dir, &format!("grid_{ratio:02}"));
    }
}
