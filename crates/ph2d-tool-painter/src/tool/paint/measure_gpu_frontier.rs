//! **Where the Painter's frame actually goes** — the census behind the GPU assessment.
//!
//! Not a gate: `#[ignore]`d measurements, run by hand, whose output is the table in
//! `docs/Painter/25_avaliacao_gpu.md`. The project's law is *measure before you limit* (`CLAUDE.md` §0),
//! and "port the Painter to the GPU" is a limit-setting decision the size of a module — so the frontier
//! has to be a measured list, not a guessed one.
//!
//! Two axes per medium, because they are different jobs with different verdicts:
//!
//! * **MOVE** — one pointer move: brush footprint rasterisation plus whatever the medium simulates. The
//!   deposit is bounded by the footprint; a medium's simulation may not be.
//! * **COMPOSITE** — turning the layer stack into the bytes the screen shows. Bounded by the CANVAS.
//!
//! And two canvas sizes, because the ratio between them says which of the two a cost belongs to — the
//! same discipline `warp::perf_tests` uses. A cost flat in canvas size is footprint-bound (the GPU wins
//! it by parallelism at best); a cost that quadruples is plane-bound (the GPU wins it outright).
//!
//! Run:
//! ```text
//! cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1
//! ```

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A canvas armed for `media` with a big soft brush — the footprint the artist actually paints with when
/// a frame gets slow, not the 10px one that hides every plane-bound cost.
fn armed(size: u32, media: PaintMedia, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(media);
    if media == PaintMedia::Impasto {
        t.set_brush_impasto_depth(1.0);
    }
    t
}

/// Median of the per-move wall clock over a straight drag, plus the composite that follows it.
///
/// Median and not mean: the first move of a stroke allocates the medium's session (the wet grid, the
/// relief planes), and an average over a handful of samples would report that allocation as if it were
/// the steady state the artist feels.
fn measure(size: u32, media: PaintMedia, radius: f32) -> (f64, f64, f64) {
    let mut t = armed(size, media, radius);
    let mid = (size / 2) as f32;
    let x0 = radius + 20.0;
    // A CONSTANT step, not one that spans the canvas: the artist's hand moves the same distance whatever
    // the document is, and a step that scaled with the canvas would report "2x the dabs" as "2x the cost
    // per move" — which is how the first run of this harness mistook its own fixture for a plane-bound
    // deposit.
    const STEP_PX: f32 = 40.0;
    let x1 = x0 + STEP_PX * 24.0;
    assert!(x1 < (size as f32) - radius, "stroke must fit the canvas");
    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let _ = t.take_preview_arc();

    let step = STEP_PX;
    let mut moves = Vec::new();
    let mut composites = Vec::new();
    let mut x = x0 + step;
    while x <= x1 {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        moves.push(t0.elapsed().as_secs_f64() * 1e3);
        let t1 = Instant::now();
        let _ = t.take_preview_arc();
        composites.push(t1.elapsed().as_secs_f64() * 1e3);
        x += step;
    }
    let t2 = Instant::now();
    t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
    let up = t2.elapsed().as_secs_f64() * 1e3;

    moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    composites.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    (moves[moves.len() / 2], composites[composites.len() / 2], up)
}

/// The census: MOVE, COMPOSITE and the pen-up bake, per medium, at two canvas sizes.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_four_media() {
    println!(
        "\n{:<12} {:>6} {:>10} {:>12} {:>10}",
        "medium", "canvas", "move ms", "composite ms", "pen-up ms"
    );
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for size in [2048u32, 4096] {
            let (mv, comp, up) = measure(size, media, 100.0);
            println!(
                "{:<12} {:>6} {:>10.3} {:>12.3} {:>10.3}",
                media.name(),
                size,
                mv,
                comp,
                up
            );
        }
    }
    println!();
}

/// How the deposit scales with the BRUSH, which is the axis the artist moves when a frame gets slow.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_brush_radius_sweep() {
    println!(
        "\n{:<12} {:>8} {:>10} {:>12}",
        "medium", "radius", "move ms", "composite ms"
    );
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for r in [16.0f32, 50.0, 100.0, 220.0] {
            let (mv, comp, _) = measure(2048, media, r);
            println!(
                "{:<12} {:>8.0} {:>10.3} {:>12.3}",
                media.name(),
                r,
                mv,
                comp
            );
        }
    }
    println!();
}

/// The composite with a REAL stack — the case the GPU compositor exists for.
///
/// A trivial stack hands back the canvas `Arc` and costs nothing; every number the artist complains
/// about comes from a document with layers in it.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_stack_depth() {
    println!("\n{:<10} {:>8} {:>14}", "layers", "canvas", "composite ms");
    for layers in [1usize, 4, 8, 16] {
        for size in [2048u32, 4096] {
            let mut t = armed(size, PaintMedia::Digital, 100.0);
            for _ in 1..layers {
                t.add_raster_layer("L");
            }
            let mid = (size / 2) as f32;
            t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
            t.on_canvas_pointer(cp([400.0, mid], PointerPhase::Move));
            t.on_canvas_pointer(cp([400.0, mid], PointerPhase::Up));
            let mut samples = Vec::new();
            for i in 0..9 {
                t.preview_dirty = true;
                let t0 = Instant::now();
                let _ = t.take_preview_arc();
                samples.push(t0.elapsed().as_secs_f64() * 1e3);
                let _ = i;
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{:<10} {:>8} {:>14.3}",
                t.layers().len(),
                size,
                samples[samples.len() / 2]
            );
        }
    }
    println!();
}

/// The **heartbeat**, isolated — what a frame costs when the pen is NOT down.
///
/// `paint_tick` is where the Wet Paint solver lives (the engine gates itself off while a stroke is down),
/// where the paper dries, and where the airbrush keeps depositing. Measuring only pointer moves therefore
/// measures the medium with its simulation switched off — which is how the first pass of this harness
/// reported the fluid as if it were footprint-bound.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_idle_tick() {
    println!("\n{:<12} {:>8} {:>12}", "medium", "canvas", "tick ms");
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for size in [1024u32, 2048, 4096] {
            let mut t = armed(size, media, 100.0);
            let mid = (size / 2) as f32;
            t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
            let mut x = 240.0;
            while x <= 800.0 {
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
                x += 40.0;
            }
            t.on_canvas_pointer(cp([800.0, mid], PointerPhase::Up));
            let _ = t.take_preview_arc();
            let mut ticks = Vec::new();
            for _ in 0..24 {
                let t0 = Instant::now();
                t.paint_tick(1.0 / 60.0);
                let _ = t.take_preview_arc();
                ticks.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            ticks.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{:<12} {:>8} {:>12.3}",
                media.name(),
                size,
                ticks[ticks.len() / 2]
            );
        }
    }
    println!();
}

/// **The ordinary document falls off the GPU** — the number this whole assessment turns on.
///
/// `painter_gpu_flatten::flatten_for_gpu` refuses a stack the moment any layer carries a mask, is
/// clipped, or is a reference layer. Those are not exotic features; a mask is how you paint inside a
/// shape. So this measures the CPU composite for the stacks the GPU hands back — one layer with a mask,
/// and a small clipped stack — against the same stacks without them.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_cost_of_falling_off_the_gpu() {
    println!(
        "\n{:<28} {:>8} {:>14}",
        "stack", "canvas", "cpu composite ms"
    );
    for size in [2048u32, 4096] {
        for (label, mask, clip, layers) in [
            ("2 layers, plain", false, false, 2usize),
            ("2 layers, one MASKED", true, false, 2),
            ("2 layers, one CLIPPED", false, true, 2),
            ("6 layers, plain", false, false, 6),
            ("6 layers, one MASKED", true, false, 6),
        ] {
            let mut t = armed(size, PaintMedia::Digital, 100.0);
            for _ in 1..layers {
                t.add_raster_layer("L");
            }
            let active = t.layers().active().expect("active layer");
            if mask {
                t.add_mask_to_active();
            }
            if clip {
                t.set_layer_clipping(active, true);
            }
            let mid = (size / 2) as f32;
            t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
            t.on_canvas_pointer(cp([400.0, mid], PointerPhase::Move));
            t.on_canvas_pointer(cp([400.0, mid], PointerPhase::Up));
            let mut samples = Vec::new();
            for _ in 0..7 {
                t.preview_dirty = true;
                let t0 = Instant::now();
                let _ = t.take_preview_arc();
                samples.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{:<28} {:>8} {:>14.3}",
                label,
                size,
                samples[samples.len() / 2]
            );
        }
    }
    println!();
}

/// The **adjustment slider drag** — the one case where the CPU has a cache of its own.
///
/// A param-only change leaves every layer BELOW the adjustment untouched, so the CPU restarts from a
/// cut-point cache (`composite_with_cache`) instead of recomposing the stack cold. Comparing the GPU's
/// full-canvas adjustment number against a COLD CPU recompose would be comparing against a path the
/// product does not take — so this measures the warm one, which is the honest counterpart.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_adjustment_drag() {
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    println!("\n{:<26} {:>8} {:>16}", "stack", "canvas", "cpu drag ms");
    for size in [1024u32, 2048, 4096] {
        for layers in [1usize, 4] {
            let mut t = armed(size, PaintMedia::Digital, 100.0);
            for _ in 1..layers {
                t.add_raster_layer("L");
            }
            let Some(adj) = t.add_adjustment_layer(AdjustmentKind::HueSaturationBrightness) else {
                continue;
            };
            let mut samples = Vec::new();
            for i in 0..9 {
                // A drag: one slider step per frame, then the drain the bridge would do.
                t.set_adjustment_param(adj, 0, 0.3 + (i as f32) * 0.01);
                let t0 = Instant::now();
                let _ = t.take_preview_arc();
                samples.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{:<26} {:>8} {:>16.3}",
                format!("{layers} raster + HSB"),
                size,
                samples[samples.len() / 2]
            );
        }
    }
    println!();
}
