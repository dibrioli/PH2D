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

/// **The fold the GPU path still pays on the CPU** — `impasto_gpu_planes`, once per dirty frame.
///
/// Every other composite number in this file is the CPU lane, and a sculpted document does not take it
/// any more: the GPU compositor and the GPU light claimed that frame in 2026-07-18. What the product
/// still pays is this — the composed relief materialised CANVAS-WIDE so the shader has planes to read
/// ([`super::impasto_gpu::ImpastoPlanes`], which calls the choice deliberate v1 and names the fix). It is
/// the one impasto cost no measurement here could see, so "which impasto number is the biggest" was being
/// answered without it.
///
/// Two axes, for the same reason as the census: canvas size says whether the cost is plane-bound (it is —
/// the loop is per texel), and layer count says what the fold itself costs on top of the walk.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_impasto_fold() {
    /// A canvas with `layers` sculpted layers, each carrying a stroke of its own — the fold has to have
    /// something to fold, and a single layer would report the walk while hiding the composite modes.
    fn sculpted(size: u32, layers: usize) -> PainterTool {
        let mut t = armed(size, PaintMedia::Impasto, 100.0);
        let mid = (size / 2) as f32;
        for i in 0..layers {
            if i > 0 {
                t.add_raster_layer("L");
            }
            let y = mid + (i as f32) * 40.0;
            t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
            t.on_canvas_pointer(cp([700.0, y], PointerPhase::Move));
            t.on_canvas_pointer(cp([700.0, y], PointerPhase::Up));
        }
        t
    }
    println!(
        "\n{:<10} {:>8} {:>12} {:>14}",
        "layers", "canvas", "fold ms", "MB/frame"
    );
    for layers in [1usize, 4] {
        for size in [2048u32, 4096] {
            let t = sculpted(size, layers);
            assert!(
                t.impasto_gpu_planes().is_some(),
                "the fixture must CARRY relief, or this measures a `None` early-out"
            );
            let mut samples = Vec::new();
            for _ in 0..7 {
                let t0 = Instant::now();
                let p = t.impasto_gpu_planes().expect("sculpted and lit");
                samples.push(t0.elapsed().as_secs_f64() * 1e3);
                // Dropped OUTSIDE the clock on purpose: freeing four canvas-sized `Vec`s is part of the
                // frame, but it is not part of the fold, and folding the free in would report the
                // allocator as if it were the loop.
                drop(p);
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let n = (size as f64) * (size as f64);
            println!(
                "{:<10} {:>8} {:>12.3} {:>14.1}",
                t.layers().len(),
                size,
                samples[samples.len() / 2],
                // relief f32 + cover u8 + mat0 + mat1 = 13 bytes a texel.
                n * 13.0 / (1024.0 * 1024.0)
            );
        }
    }
    println!();
}

/// **What the fold is MADE of** — the three candidate cures, priced before any of them is written.
///
/// [`measure_the_impasto_fold`] says the fold is the biggest per-frame number in the Painter. It does not
/// say *why*, and the three cures are different waves with different risks:
///
/// * **allocation floor** — four canvas-sized `Vec`s built fresh every frame. If the floor is most of the
///   number, the cure is a reused scratch and nothing else.
/// * **the walk** — `height_at`/`cover_at`/`material_at` per texel. Whatever is left above the floor.
/// * **a REGION** — the same walk over the dirty rect instead of the canvas. This is the number that says
///   whether persistent plane textures with a partial re-fold is worth the epoch machinery it needs.
///
/// Measured through the light's OWN samplers, the same ones `impasto_gpu_planes` calls — a loop of my own
/// arithmetic here would price a fold the product does not run.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_the_fold_is_made_of() {
    fn sculpted(size: u32) -> PainterTool {
        let mut t = armed(size, PaintMedia::Impasto, 100.0);
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        t.on_canvas_pointer(cp([700.0, mid], PointerPhase::Move));
        t.on_canvas_pointer(cp([700.0, mid], PointerPhase::Up));
        t
    }
    /// The dirty rect a 100px brush actually marks — the region a partial fold would have to cover.
    const REGION_EDGE: i64 = 512;
    println!(
        "\n{:<8} {:>12} {:>12} {:>12} {:>14}",
        "canvas", "alloc ms", "full ms", "region ms", "region edge"
    );
    for size in [2048u32, 4096] {
        let t = sculpted(size);
        let n = (size as usize) * (size as usize);
        let mut alloc = Vec::new();
        for _ in 0..7 {
            let t0 = Instant::now();
            let planes = (
                vec![0f32; n],
                vec![0u8; n],
                vec![0u8; n * 4],
                vec![0u8; n * 4],
            );
            alloc.push(t0.elapsed().as_secs_f64() * 1e3);
            drop(planes);
        }
        let f = t.impasto_fields().expect("sculpted and lit");
        // The walk, over the whole canvas and over a rect — same samplers, same order, only the bounds
        // differ, so the ratio between the two columns is the region's saving and nothing else.
        let walk = |x0: i64, y0: i64, x1: i64, y1: i64| -> f64 {
            let mut s = Vec::new();
            for _ in 0..5 {
                let t0 = Instant::now();
                let mut sink = 0.0f32;
                for y in y0..y1 {
                    for x in x0..x1 {
                        sink += f.height_at(x, y) + f.cover_at(x, y);
                        sink += f32::from(f.material_at(x, y)[0]);
                    }
                }
                s.push(t0.elapsed().as_secs_f64() * 1e3);
                assert!(
                    sink.is_finite(),
                    "the sink keeps the walk from being optimised away"
                );
            }
            s.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            s[s.len() / 2]
        };
        let edge = i64::from(size);
        let full = walk(0, 0, edge, edge);
        let mid = edge / 2;
        let half = REGION_EDGE / 2;
        let region = walk(mid - half, mid - half, mid + half, mid + half);
        alloc.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        println!(
            "{:<8} {:>12.3} {:>12.3} {:>12.3} {:>14}",
            size,
            alloc[alloc.len() / 2],
            full,
            region,
            REGION_EDGE
        );
    }
    println!();
}

/// **The fold as the PRODUCT runs it** — `impasto_gpu_planes_in`, not a loop of my own.
///
/// ⚠️ [`measure_what_the_fold_is_made_of`] re-implements the walk to dissect it, which is right for
/// *"what is this number made of"* and **blind** to any change in the door itself: it would go on
/// printing the serial cost after the product stopped paying it. This one times the door, so the two
/// columns are what a frame is actually charged.
///
/// The number that matters is the FULL column: the first lit frame at 4096² cannot use a window
/// (`ImpastoLightPass::planes_seeded` is false until the pass has held the whole painting once), so the
/// artist's first relief stroke pays a whole-canvas fold. Measured in the running app by
/// `PH2D_PAINT_PERF`, that frame was **232,7 ms, 100% of it inside `preview`** — and this is the piece.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_fold_the_product_runs() {
    fn sculpted(size: u32) -> PainterTool {
        let mut t = armed(size, PaintMedia::Impasto, 100.0);
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        t.on_canvas_pointer(cp([700.0, mid], PointerPhase::Move));
        t.on_canvas_pointer(cp([700.0, mid], PointerPhase::Up));
        t
    }
    /// ⚠️ **The window sweep is not decoration.** Parallelising by rows made the FULL fold 13,8× cheaper
    /// and made the tiny windows small enough that rayon's own scheduling can dominate them — and a tiny
    /// window is the STEADY STATE of every stroke frame after the first, which is the frame the artist
    /// feels most. A cure for the once-per-document cost that taxed the per-move cost would be a bad
    /// trade made invisibly, so both ends are measured here, in one table.
    const EDGES: [u32; 5] = [64, 128, 256, 512, 1024];
    println!(
        "\n{:<8} {:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "canvas", "full ms", "1024²", "512²", "256²", "128²", "64²"
    );
    for size in [2048u32, 4096] {
        let t = sculpted(size);
        let time = |region: (u32, u32, u32, u32)| -> f64 {
            let mut s = Vec::new();
            for _ in 0..9 {
                let t0 = Instant::now();
                let planes = t.impasto_gpu_planes_in(region).expect("sculpted and lit");
                s.push(t0.elapsed().as_secs_f64() * 1e3);
                assert!(!planes.relief.is_empty(), "the fold has to produce planes");
            }
            s.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            s[s.len() / 2]
        };
        let full = time((0, 0, size, size));
        let mut cols = Vec::new();
        for edge in EDGES.iter().rev() {
            // ⚠️ Centred on the STROKE, never on the canvas. The first version of this table centred the
            // window on the canvas, and since the stroke lives at `x ∈ [200, 700]` the 2048² windows
            // covered paint while the 4096² ones covered bare paper — so the two rows priced different
            // phenomena and the big canvas looked *cheaper*, which is the fixture lying, not the fold.
            let cx = 450u32.max(edge / 2).min(size - edge / 2);
            let cy = size / 2;
            cols.push(time((cx - edge / 2, cy - edge / 2, *edge, *edge)));
        }
        print!("{size:<8} {full:>12.3}");
        for c in &cols {
            print!(" {c:>10.4}");
        }
        println!();
    }
    println!();
}

/// **The shell holds the drained Arc, and that is the whole cost of a plain stroke.**
///
/// Every other MOVE measurement in this file drops the `take_preview_arc` result on the floor
/// (`let _ = …`), so the tool is the SOLE owner of `canvas_rgba` and its next `Arc::make_mut` is
/// free. The real shell is not the sole owner: `painter_bridge::dispatch` stashes that Arc in
/// `painter_preview.rgba` and holds it across the frame, so the tool's next `stamp_dabs`
/// `make_mut` sees refcount 2 and **copies the entire canvas** before blitting the dab. A brush of
/// 0.5 px pays a full-canvas memcpy per move — the "small brush, big canvas, CPU-bound FPS drop"
/// Enio reported.
///
/// This measures the pre-fix shell (`held` kept) against the sole-owner path (`held` dropped),
/// across canvas size and brush radius. The held column quadruples with the canvas while the dropped
/// column stays flat and the radius barely moves it — the cost is the copy-on-write, not the deposit.
///
/// ⚠️ **This is the DIAGNOSIS, not the fix's gate.** The fix (the shell owning its preview buffer via
/// `own_preview_buffer` so the tool stays sole owner) lives across the `ph2d-render`/shell boundary;
/// its proof — driving the real helper, showing the held path go footprint-bound — is
/// `painter_preview_pipeline_tests::a_plain_stroke_is_footprint_bound_when_the_shell_owns_its_buffer`.
/// The `held` column here is exactly the bug that gate's control still reproduces.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_stroke_the_shell_actually_drives() {
    fn stroke(size: u32, radius: f32, hold: bool) -> f64 {
        let mut t = armed(size, PaintMedia::Digital, radius);
        let mid = (size / 2) as f32;
        let x0 = radius + 20.0;
        const STEP_PX: f32 = 20.0;
        let x1 = x0 + STEP_PX * 30.0;
        assert!(x1 < (size as f32) - radius, "stroke must fit the canvas");
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        // The shell holds the LAST drained Arc (its `painter_preview.rgba`), one at a time.
        let mut held: Option<std::sync::Arc<Vec<u8>>> = t.take_preview_arc().map(|(a, _, _)| a);
        let mut moves = Vec::new();
        let mut x = x0 + STEP_PX;
        while x <= x1 {
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            let drained = t.take_preview_arc().map(|(a, _, _)| a);
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
            // Keep it only in the `hold` arm — the drop arm frees the Arc before the next move,
            // so the tool is sole owner and `make_mut` is O(1).
            held = if hold { drained } else { None };
            x += STEP_PX;
        }
        let _ = held;
        moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        moves[moves.len() / 2]
    }
    // Whether a radius actually deposits — the copy only fires when the drain returns `Some` (the
    // shell then holds the Arc). A radius so small it marks nothing dirty pays nothing, and mistaking
    // that for "small brushes are free" is the fixture trap the radius-0.5 row would have set.
    fn deposits(size: u32, radius: f32) -> bool {
        let mut t = armed(size, PaintMedia::Digital, radius);
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([radius + 20.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        t.on_canvas_pointer(cp([radius + 40.0, mid], PointerPhase::Move));
        t.take_preview_arc().is_some()
    }
    println!(
        "\n{:<10} {:>8} {:>9} {:>14} {:>14} {:>10}",
        "canvas", "radius", "paints?", "held ms/move", "dropped ms/move", "ratio"
    );
    for size in [1024u32, 2048, 4096] {
        for radius in [1.0f32, 2.0, 4.0, 16.0, 100.0] {
            let held = stroke(size, radius, true);
            let dropped = stroke(size, radius, false);
            println!(
                "{:<10} {:>8.1} {:>9} {:>14.3} {:>14.3} {:>9.1}x",
                size,
                radius,
                deposits(size, radius),
                held,
                dropped,
                held / dropped.max(1e-6)
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
