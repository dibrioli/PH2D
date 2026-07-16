//! RENDER-AND-LOOK probe for the Push phase (the handoff's explicit method: start by rendering and
//! looking, not from theory). Writes lit PNGs of deposit-Push strokes — plus the Scrape+Conserve
//! reference whose drawing Enio approved — to `PH2D_PUSH_LOOK_DIR`. Diagnostic, `#[ignore]`d.

use ph2d_editor::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tool(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t.toggle_brush_impasto();
    t.set_brush_color_srgb8([168, 36, 36]); // Enio's dark red — black crushes the shading
    t
}

/// Thick ground: three fat horizontal strokes across the middle band.
fn lay_ground(t: &mut PainterTool) {
    for y in [150.0f32, 190.0, 230.0] {
        t.on_canvas_pointer(cp([50.0, y], PointerPhase::Down));
        for i in 1u8..=10 {
            t.on_canvas_pointer(cp([50.0 + 30.0 * f32::from(i), y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([350.0, y], PointerPhase::Up));
    }
}

fn save(t: &mut PainterTool, dir: &str, name: &str, size: u32) {
    let active = t.layers().active().expect("layer");
    let cur = t.layers().get(active).expect("layer").opacity;
    t.set_layer_opacity(active, cur); // byte-neutral invalidate → full recompose + light
    let (rgba, w, h) = t.take_preview_arc().expect("composite");
    let _ = image::save_buffer(
        format!("{dir}/{name}.png"),
        &rgba[..],
        w,
        h,
        image::ColorType::Rgba8,
    );
    assert_eq!((w, h), (size, size));
}

#[test]
#[ignore = "render-and-look probe; PH2D_PUSH_LOOK_DIR=<dir> cargo test ... -- --ignored"]
fn probe_push_render_and_look() {
    let Some(dir) = std::env::var("PH2D_PUSH_LOOK_DIR").ok() else {
        eprintln!("set PH2D_PUSH_LOOK_DIR");
        return;
    };
    let size = 400u32;

    // 1. Ground only (the baseline everything else is judged against).
    let mut t = tool(size);
    lay_ground(&mut t);
    save(&mut t, &dir, "0_ground", size);

    // 2. Push=1 straight stroke crossing the ground vertically (the classic plough).
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_brush_size_px(24.0);
    t.set_brush_impasto_push(1.0);
    t.on_canvas_pointer(cp([200.0, 60.0], PointerPhase::Down));
    for i in 1u8..=10 {
        t.on_canvas_pointer(cp([200.0, 60.0 + 26.0 * f32::from(i)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([200.0, 320.0], PointerPhase::Up));
    save(&mut t, &dir, "1_push_straight", size);

    // 3. Same stroke, Push=0 (contrast: pure pile-on).
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_brush_size_px(24.0);
    t.on_canvas_pointer(cp([200.0, 60.0], PointerPhase::Down));
    for i in 1u8..=10 {
        t.on_canvas_pointer(cp([200.0, 60.0 + 26.0 * f32::from(i)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([200.0, 320.0], PointerPhase::Up));
    save(&mut t, &dir, "2_push_off", size);

    // 4. Push=1 stroke that STOPS in the middle of the ground (where a bow wave would stand).
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_brush_size_px(24.0);
    t.set_brush_impasto_push(1.0);
    t.on_canvas_pointer(cp([120.0, 190.0], PointerPhase::Down));
    for i in 1u8..=6 {
        t.on_canvas_pointer(cp([120.0 + 20.0 * f32::from(i), 190.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([240.0, 190.0], PointerPhase::Up));
    save(&mut t, &dir, "3_push_stops_midground", size);

    // 5. Push=1 CURVE through the ground.
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_brush_size_px(24.0);
    t.set_brush_impasto_push(1.0);
    t.on_canvas_pointer(cp([120.0, 90.0], PointerPhase::Down));
    for i in 1u8..=12 {
        let a = f32::from(i) * 0.22;
        t.on_canvas_pointer(cp(
            [
                120.0 + 90.0 * a.sin() + 12.0 * f32::from(i),
                90.0 + 20.0 * f32::from(i),
            ],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([260.0, 330.0], PointerPhase::Up));
    save(&mut t, &dir, "4_push_curve", size);

    // 6. THE COLLAR SCENE: a BIG SOFT brush (radius 45, Smooth default) at Push=1 stopping in the
    //    middle of thick paint. This is where the 2026-07-15 bug is loudest — the old rim anchored at
    //    the geometric rim (45 px) while the soft paint's body ends at ~27 px (t0 ≈ 0.60), so a hard
    //    circular collar stood ~18 px of bare canvas out from the paint. After the fix the ridge rises
    //    from the body edge and butts the paint.
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_brush_size_px(45.0);
    t.set_brush_impasto_push(1.0);
    t.on_canvas_pointer(cp([70.0, 190.0], PointerPhase::Down));
    for i in 1u8..=7 {
        t.on_canvas_pointer(cp([70.0 + 20.0 * f32::from(i), 190.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([210.0, 190.0], PointerPhase::Up));
    save(&mut t, &dir, "5_push_stops_big_soft", size);

    // 6. THE REFERENCE: Scrape + Conserve through the same ground (the drawing Enio approved).
    let mut t = tool(size);
    lay_ground(&mut t);
    t.set_paint_tool_mode("sculpt");
    t.set_sculpt_mode(3); // Scrape
    t.toggle_sculpt_conserve();
    t.set_brush_size_px(24.0);
    t.on_canvas_pointer(cp([200.0, 60.0], PointerPhase::Down));
    for i in 1u8..=10 {
        t.on_canvas_pointer(cp([200.0, 60.0 + 26.0 * f32::from(i)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([200.0, 320.0], PointerPhase::Up));
    save(&mut t, &dir, "6_reference_scrape_conserve", size);

    eprintln!("PNGs written to {dir}");
}
