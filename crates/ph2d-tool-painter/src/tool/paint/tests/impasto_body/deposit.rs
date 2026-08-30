//! **O relevo que o DEPÓSITO deixa.** O interruptor mestre (desligado = byte a byte igual), o ladrilho
//! e a simetria, a borracha que leva o relevo junto com a tinta, o Draw To Depth, o undo, a cor
//! por-camada, o grão, a invariância ao espaçamento dos dabs, o tamanho do pincel e o teto de
//! empilhamento.

use super::*;

// ── Impasto (#16) — the foundation gate: the master switch is the ONLY gate ───────────────────────

/// Paint the SAME rich stroke — Shape + Grain + Randomize Color + Jitter Scale/Rotate + Symmetry +
/// Tiling, i.e. every feature Enio asked to be integrated — through a brush the caller may tweak.
/// Returns the canvas bytes. One tool per call, so the two runs share nothing but the code.
fn impasto_rich_stroke(tweak: impl FnOnce(&mut BrushSpec)) -> Vec<u8> {
    use ph2d_painter_brush::{MirrorAxis, TextureKind, TextureMapping};
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let mut b = BrushSpec {
        radius_px: 9.0,
        hardness: 0.3,
        color: [0.2, 0.5, 0.9],
        space_attenuation: false,
        // Everything that must ride the SAME dab list / SAME stamp mask as the height will.
        dab_flatten: 0.4,
        dab_angle_deg: 20,
        jitter_scale: 0.3,
        jitter_rotate: 0.4,
        jitter_spacing: 0.2,
        color_jitter_enabled: true,
        color_jitter_hue: 0.3,
        color_jitter_sat: 0.2,
        color_jitter_val: 0.2,
        grain_depth: 0.8,
        ..Default::default()
    };
    b.shape.kind = TextureKind::Checker; // procedural silhouette (no image pixels needed)
    b.shape.mapping = TextureMapping::ViewPlane;
    b.texture.kind = TextureKind::Noise; // Grain
    b.texture.mapping = TextureMapping::ViewPlane;
    b.symmetry.enabled = true;
    b.symmetry.axis = MirrorAxis::X;
    b.symmetry.center = [24.0, 24.0];
    tweak(&mut b);
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.paint.tiling = [true, true]; // wrap on both axes
    t.on_canvas_pointer(cp([6.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 22.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([44.0, 41.0], PointerPhase::Up));
    (*t.canvas_rgba).clone()
}

#[test]
fn impasto_off_is_byte_identical() {
    // T1.3, the foundation of #16: while the master switch is off, NONE of the impasto knobs may be
    // read. If any of them leaks into the colour path — even as a rounding difference — this fails.
    //
    // The knobs deliberately carry live *when-enabled* defaults (depth 0.5, smoothing 0.2), so the
    // default is inert because of the SWITCH, not because the values happen to be neutral. This gate
    // is what says so: it drives every knob to a wild value and demands the same bytes.
    //
    // The stroke is the rich one on purpose — Shape, Grain, Randomize Color, Jitter Scale/Rotate,
    // Symmetry and Tiling all active. Those are exactly the features the height channel is going to
    // share the dab list and the stamp mask with, so if wiring the height ever perturbs the dab
    // stream (a re-ordered RNG draw, an extra dab, a differently-shaped mask), the COLOUR moves too
    // and this gate catches it — in the one configuration where it is hardest to notice by eye.
    let baseline = impasto_rich_stroke(|_| {});
    let wild = impasto_rich_stroke(|b| {
        b.impasto_depth = 1.0;
        b.impasto_smoothing = 1.0;
        b.impasto_source = DepthSource::Grain;
        b.impasto_draw_to = DrawTo::Depth; // would suppress ALL pigment if it were read
        // ...but the master switch stays OFF.
        b.impasto = false;
    });
    assert_eq!(
        baseline, wild,
        "with Impasto off, the impasto settings must not reach a single pixel"
    );
    assert!(
        baseline.iter().any(|&b| b != 255),
        "sanity: the fixture actually painted (an all-white canvas would make this gate vacuous)"
    );
}

#[test]
fn impasto_tiling_sculpts_the_opposite_edge() {
    // THE structural gate (plan §5, T3.4.4). The height must consume the dab list the COLOUR consumes —
    // already wrapped by Tiling. A height pass that walked its own geometry would paint relief only
    // where the brush physically is, and the wrapped edge would come out flat: paint on one side of the
    // seam, no thickness on the other. That is how "Tiling doesn't work in Impasto" gets born, and this
    // is the test that refuses to let it.
    let size = 40u32;
    let mut t = impasto_canvas(size);
    t.paint.tiling = [true, false]; // wrap on X
    t.on_canvas_pointer(cp([1.0, 20.0], PointerPhase::Down)); // hard against the LEFT edge
    t.on_canvas_pointer(cp([1.0, 20.0], PointerPhase::Up));
    let h = relief(&t);
    assert!(!h.is_empty(), "the stroke laid down relief");
    let at = |x: u32, y: u32| h[(y * size + x) as usize];
    assert!(at(0, 20) > 0.0, "relief where the brush is");
    assert!(
        at(size - 1, 20) > 0.0,
        "and relief on the WRAPPED edge — the height rides the same tiled dab list as the colour"
    );
    assert_eq!(at(20, 20), 0.0, "and nowhere the brush never went");
}

#[test]
fn impasto_symmetry_mirrors_the_relief() {
    // The Symmetry twin of the Tiling gate: `push_symmetric` mirrors dabs INTO the list, so the mirrored
    // dab carries its height for free — if, and only if, the height reads the list.
    let size = 40u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.symmetry.enabled = true;
    b.symmetry.axis = ph2d_painter_brush::MirrorAxis::X; // mirror across the vertical centre line
    b.symmetry.center = [20.0, 20.0];
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([8.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 12.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([8.0, 12.0], PointerPhase::Up));
    let h = relief(&t);
    let at = |x: u32, y: u32| h[(y * size + x) as usize];
    assert!(at(8, 20) > 0.0, "relief under the brush");
    assert!(
        at(32, 20) > 0.0,
        "and its mirror image — 8 and 32 straddle the axis at x=20"
    );
    // And NOTHING in between. This half of the gate exists because a mutation found the hole: the body
    // is swept back to the PREVIOUS dab along the path, and `push_symmetric` INTERLEAVES its copies
    // (`[base, mirror, base, mirror, …]`). Link the immediate neighbour in the list and you sweep a
    // capsule from every dab to its own MIRROR — a bar of paint straight across the canvas. The
    // assertions above pass happily with that bar present, which is exactly the kind of gate that lets a
    // bug ship. The path predecessor is `copies` entries back, and this is what says so.
    for y in 18..23 {
        assert_eq!(
            at(20, y),
            0.0,
            "no relief on the mirror axis — the stroke and its reflection must not be joined by a bar \
             (x=20, y={y})"
        );
    }
}

#[test]
fn impasto_one_stroke_is_one_thickness_but_two_strokes_add() {
    // The envelope (T1.2). Scrubbing back and forth WITHIN a stroke must not build a staircase of
    // paint — a loaded brush passing over its own line leaves one thickness. But a SECOND stroke
    // genuinely piles more on. Both halves matter: envelope-everything would make impasto unbuildable;
    // add-everything would make a single slow stroke pile up under the cursor.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Move)); // pass 2 over the same point
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Move)); // pass 3
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let one = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (one - 0.5).abs() < 1e-5,
        "one stroke = one depth, got {one}"
    );

    // A second, separate stroke over the same paint.
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let two = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (two - 1.0).abs() < 1e-5,
        "a second stroke lays MORE paint on top, got {two}"
    );
}

#[test]
fn impasto_eraser_takes_the_relief_with_the_paint() {
    // T1.6 — not optional, a correction: without it the eraser removes the pigment and the light pass
    // keeps reporting a ridge. Ghost relief.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert!(relief(&t)[(20 * 40 + 20) as usize] > 0.0, "relief is there");

    t.paint.eraser = true;
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let left = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        left.abs() < 1e-5,
        "the eraser scrubbed the relief away with the paint, got {left}"
    );
}

#[test]
fn impasto_draw_to_depth_sculpts_without_painting() {
    // T1.8 — the palette knife: thickness, no pigment. The canvas must come out BYTE-identical while
    // the relief changes. (A "sculpt" that quietly tinted the canvas would be a lie.)
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto_draw_to = DrawTo::Depth;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let before = (*t.canvas_rgba).clone();
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert_eq!(
        *t.canvas_rgba, before,
        "Draw To = Depth deposits no pigment — the canvas is untouched"
    );
    assert!(
        relief(&t)[(20 * 40 + 20) as usize] > 0.0,
        "...but the relief is there"
    );
}

#[test]
fn impasto_undo_takes_back_the_relief_with_the_pixels() {
    // The relief lives in the undo snapshot. If it didn't, Ctrl+Z would restore the colour and leave
    // the thickness — paint that is gone but still catches the light.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    assert!(relief(&t)[(20 * 40 + 20) as usize] > 0.0);
    assert!(t.undo_last(), "the stroke is one undo step");
    let h = relief(&t);
    assert!(
        h.is_empty() || h[(20 * 40 + 20) as usize].abs() < 1e-6,
        "undo took the relief back with the pixels"
    );
    assert!(t.redo_last(), "and it redoes");
    assert!(
        relief(&t)[(20 * 40 + 20) as usize] > 0.0,
        "and redo brings it back"
    );
}

#[test]
fn watercolor_is_untouched_by_impasto() {
    // ★ THE BARRIER (plan §2, Enio's explicit order: "Watercolor é uma implementação à parte e não deve
    // ser tocada ou ferida"). With the wash on, an impasto brush must be INERT: the canvas byte-identical
    // to the same wash with Impasto off, and not one texel of relief deposited. The architecture already
    // guarantees it (`stamp_dabs` short-circuits into the optical path before the router, where the
    // height choke point lives) — this test is what will notice the day someone changes that.
    let wash = |impasto: bool| -> (Vec<u8>, Vec<f32>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
        let b = BrushSpec {
            radius_px: 8.0,
            color: [0.8, 0.2, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            impasto,
            impasto_depth: 1.0,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([16.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([32.0, 24.0], PointerPhase::Up));
        let h = relief(&t);
        ((*t.canvas_rgba).clone(), h)
    };
    let (plain, h_off) = wash(false);
    let (with_impasto, h_on) = wash(true);
    assert!(
        plain.iter().any(|&b| b != 255),
        "sanity: the wash actually painted"
    );
    assert_eq!(
        plain, with_impasto,
        "Impasto must not move a single pixel of the watercolor path"
    );
    assert!(h_off.is_empty(), "and the wash deposits no relief...");
    assert!(
        h_on.iter().all(|&v| v == 0.0),
        "...with Impasto ticked either — the card is hidden there, and the code path never runs"
    );
}

#[test]
fn impasto_on_does_not_disturb_the_pigment() {
    // RULE 2, and the gate that guards it. The Grain's per-dab random frame is drawn from a PERSISTENT
    // rng stream (`tex_rng`). The height pass has to resolve the same frames the colour pass will — so
    // it reads a COPY of that stream and throws it away. If it ever wrote the stream back (the obvious,
    // wrong thing), every colour dab would draw the NEXT random frame instead of its own: the relief and
    // the pigment would carry different grain, and the artist would see the texture change the moment
    // they ticked Impasto — a checkbox for RELIEF silently repainting the COLOUR.
    //
    // **The isolation had to move** (Enio 2026-07-12). The gate used to say "turning Impasto ON changes
    // not one pixel of pigment", and that premise died the day a body-laying brush began cutting its own
    // pigment to the film it lays (`film_coverage`): Enio's whole complaint was that the pigment DIDN'T
    // follow the body. So the rule is now stated where it is still true and still sharp — on a brush whose
    // height pass RUNS (it must, or the gate is vacuous) but which lays no BODY, so no film: Impasto on,
    // `DrawTo::Color`, Push up. The height kernel resolves every grain frame exactly as before; if it
    // consumed the stream, the pigment would shift. It may not move one byte.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let stroke = |arm: &dyn Fn(&mut BrushSpec)| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
        let mut b = BrushSpec {
            radius_px: 7.0,
            color: [0.2, 0.6, 0.3],
            space_attenuation: false,
            impasto_depth: 1.0,
            impasto_source: DepthSource::Grain,
            ..Default::default()
        };
        // A Grain that DRAWS from the rng every dab — the whole point. A static grain would make this
        // gate vacuous: with nothing consuming the stream, a stream bug is invisible. Random Offset is the
        // per-dab rng draw (the per-slot Random Angle was retired 2026-07-19).
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Random;
        arm(&mut b);
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let without = stroke(&|b| b.impasto = false);
    // The height pass RUNS (Push makes it touch the field) and lays NO body (`DrawTo::Color`) ⇒ no film.
    let running_no_body = stroke(&|b| {
        b.impasto = true;
        b.impasto_draw_to = DrawTo::Color;
        b.impasto_push = 0.5;
    });
    assert!(
        without.iter().any(|&b| b != 255),
        "sanity: the fixture painted"
    );
    assert_eq!(
        without, running_no_body,
        "the height pass must not consume the colour's random stream: it resolved every grain frame here \
         (Push is up), and the pigment moved anyway — every colour dab is drawing the NEXT frame"
    );
    // …and the gate is not vacuous the other way: a brush that DOES lay body cuts its pigment to the film,
    // on purpose (Enio 2026-07-12). If this ever stops being true, the film is gone and nobody noticed.
    assert_ne!(
        without,
        stroke(&|b| b.impasto = true),
        "a body-laying brush lays its pigment as a FILM — the cut is the fix, not a regression"
    );
}

#[test]
fn impasto_per_layer_color_leaves_one_coherent_relief() {
    // T1.7 — the plan called this "the one place `for free` does not hold", because the Per-Layer Color
    // route BYPASSES the ordinary cached routes: it composites N tinted shape layers onto the canvas
    // itself. It turned out to be free after all, and for a reason worth writing down: the height is
    // taken at the ONE choke point ABOVE the whole route dispatch, from the union silhouette that all N
    // layers already flatten into. So the relief is ONE coherent body — the thickness of the paint the
    // brush laid — not N stacked steps, one per shape layer, which is exactly the artefact the plan
    // feared. Nothing about this is guaranteed by the code reading well; it is guaranteed by this test.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    // Two shape layers, each a solid 8×8 tile → the union silhouette is the whole tip.
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    assert!(
        t.brush_settings().shape_per_layer_color,
        "the fixture really is in Per-Layer Color mode"
    );
    let mut b = t.paint.brush;
    b.radius_px = 8.0;
    b.impasto = true;
    b.impasto_depth = 0.6;
    b.space_attenuation = false;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    let h = relief(&t);
    assert!(!h.is_empty(), "the Per-Layer Color route laid down relief");
    let centre = h[(32 * 64 + 32) as usize];
    assert!(centre > 0.0, "there IS a body under the dab ({centre})");
    // ONE dab of depth 0.6 ⇒ at most 0.6 anywhere. Two layers each contributing their own step would
    // land at ~1.2 — the "N stacked steps" artefact, and the whole reason this task existed.
    let peak = h.iter().fold(0.0f32, |m, &v| m.max(v));
    assert!(
        peak > 0.3,
        "sanity: a real body, not a sliver — else the bound below would pass vacuously (peak {peak})"
    );
    assert!(
        peak <= 0.6 + 1e-5,
        "the relief is ONE body of the brush's depth, not one step per shape layer (peak {peak})"
    );
}

#[test]
#[ignore = "perf measurement - run with --release --ignored"]
fn impasto_perf_kill_criterion() {
    // The kill-criterion frozen BEFORE the build (plan 7, DIRETIVA 5): canvas 2048, r=100, a dragged
    // stroke, Show Impasto on. Target <= 4 ms/move for the whole impasto cost (deposit + light over the
    // dirty rect); KILL at 8 ms. Numbers, in ms, in --release. No verdict by vibes.
    //
    // It also times the PEN-UP now, and at 4096 as well as 2048, because the criterion as frozen was
    // watching the wrong half. It timed the Move and never the Up - and the Up was where the commit ran a
    // box blur over the WHOLE CANVAS for a stroke that touched a corner of it: 146 ms at 2048 and
    // **1010 ms at 4096**, a full second of freeze at the end of every stroke, invisible to a gate that
    // only ever watched the drag. A budget whose other half nobody spends is not a budget.
    use std::time::Instant;
    const MOVES: u32 = 20;
    static PROBE_PUSH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    // The same stroke with the feature OFF and ON. The number that matters is the DELTA - the frame
    // already costs something without Impasto, and charging that to Impasto would flatter it.
    let run = |size: u32, impasto: bool| -> (f64, f64, f64) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 100.0,
            color: [0.2, 0.4, 0.8],
            space_attenuation: false,
            impasto,
            impasto_depth: 0.7,
            impasto_push: if impasto {
                f32::from_bits(PROBE_PUSH.load(std::sync::atomic::Ordering::Relaxed))
            } else {
                0.0
            },
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        let mid = (size / 2) as f32;
        // A GROUND stroke first: without paint already on the canvas, Push has nothing to shove and the
        // measurement would be of a code path that never ran. (It flattered the first run of this probe.)
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        for i in 0..MOVES {
            t.on_canvas_pointer(cp(
                [220.0 + f64::from(i) as f32 * 40.0, mid],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([1020.0, mid], PointerPhase::Up));
        let _ = t.take_preview_arc();

        t.on_canvas_pointer(cp([200.0, mid + 30.0], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let (mut worst, mut total) = (0.0f64, 0.0f64);
        for i in 0..MOVES {
            let x = 220.0 + f64::from(i) * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x as f32, mid + 30.0], PointerPhase::Move));
            let _ = t.take_preview_arc(); // deposit + composite + light: what a frame really costs
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            worst = worst.max(ms);
            total += ms;
        }
        // The other half of the budget: the commit (derive + settle + re-base + the re-light of what moved).
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([1020.0, mid + 30.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let up = t0.elapsed().as_secs_f64() * 1000.0;
        (total / f64::from(MOVES), worst, up)
    };
    for (size, push) in [(2048u32, 0.0f32), (4096, 0.0), (4096, 1.0)] {
        PROBE_PUSH.store(push.to_bits(), std::sync::atomic::Ordering::Relaxed);
        let (off_mean, off_worst, off_up) = run(size, false);
        let (on_mean, on_worst, on_up) = run(size, true);
        println!("--- Push = {push} ---");
        println!(
            "@{size}px r100 - impasto OFF: mean {off_mean:.2} ms/move, worst {off_worst:.2}, pen-up {off_up:.2} ms\n\
             @{size}px r100 - impasto  ON: mean {on_mean:.2} ms/move, worst {on_worst:.2}, pen-up {on_up:.2} ms\n\
             >>> IMPASTO COST @{size}px: mean {:.2} ms/move, worst {:.2} ms/move (target <=4, kill 8) | PEN-UP {:.2} ms",
            on_mean - off_mean,
            on_worst - off_worst,
            on_up - off_up
        );
    }
}

#[test]
fn impasto_hides_itself_in_every_mode_it_does_not_apply_to() {
    // §1.2 of the plan, as an EXECUTABLE gate. The card is painted only when `impasto_applies`, and a
    // card that is not painted registers no hit — so this one predicate is what makes the whole matrix
    // real. A prose checklist in a doc does not bite; this does.
    //
    // Watercolor is the one that matters most: Enio's order was that it "é uma implementação à parte e
    // não deve ser tocada ou ferida". Impasto must not so much as APPEAR there.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    assert!(
        t.impasto_applies(),
        "a plain Paint brush is where Impasto lives"
    );
    assert!(
        t.brush_settings().impasto_applies,
        "and the panel is told so"
    );

    // Watercolor: the wash is a separate implementation and thin paint besides.
    t.paint.brush.watercolor = true;
    assert!(!t.impasto_applies(), "hidden under the Watercolor wash");
    assert!(!t.brush_settings().impasto_applies);
    t.paint.brush.watercolor = false;

    // Eraser: it TAKES relief away (that is wired), but it has no body of its own to configure.
    t.paint.eraser = true;
    assert!(!t.impasto_applies(), "hidden for the Eraser");
    t.paint.eraser = false;

    // The pixel-processing modes: Smear / Blur / Clone move paint that is already down; Mask is a
    // grayscale channel with no body; Inpaint is a heal disc that ignores the brush entirely. None of
    // them deposits paint, so none of them has a Body to configure.
    for mode in [
        PaintMode::Smear,
        PaintMode::Blur,
        PaintMode::Clone,
        PaintMode::Mask,
        PaintMode::Inpaint,
        PaintMode::Selection,
    ] {
        t.paint.paint_mode = mode;
        assert!(
            !t.impasto_applies(),
            "the Body card must not show up in {mode:?} — it deposits no fresh paint"
        );
        assert!(!t.brush_settings().impasto_applies);
    }

    // …the exception the matrix always named is the one that deposits nothing and yet MOVES paint, so it
    // gets the knife — `Plow`, and nothing else. ⚠️ **That is the KNIFE, not the Smear** (Enio,
    // 2026-07-19: *"o modo Smear do Impasto (knife) deve ser único e não compartilhado com o smear dos
    // outros tipos de pintura … Smear com botão no painel lateral é o smear dos outros modos de
    // pintura"*). They were one mode until then, which meant one `BrushSpec` slot and one Plow between
    // them; the plain smear now drags the colour and leaves the body where it is.
    t.paint.paint_mode = PaintMode::Knife;
    assert!(
        t.impasto_plow_applies() && t.brush_settings().impasto_plow_applies,
        "the Knife has a knife"
    );
    for mode in [
        PaintMode::Paint,
        PaintMode::Smear,
        PaintMode::Blur,
        PaintMode::Clone,
        PaintMode::Mask,
        PaintMode::Inpaint,
        PaintMode::Selection,
    ] {
        t.paint.paint_mode = mode;
        assert!(
            !t.impasto_plow_applies(),
            "…and only the Knife does — {mode:?} has no impasto volume to displace"
        );
        assert!(!t.brush_settings().impasto_plow_applies);
    }
    // The two are mutually exclusive by construction: a mode never shows both cards, so the artist is
    // never offered a Depth they cannot deposit nor a knife with nothing to push.
    for mode in [PaintMode::Paint, PaintMode::Knife] {
        t.paint.paint_mode = mode;
        assert!(
            t.impasto_applies() != t.impasto_plow_applies(),
            "{mode:?}: exactly one of the two Impasto surfaces is live"
        );
    }
    t.paint.paint_mode = PaintMode::Paint;
    assert!(t.impasto_applies(), "and it comes back in Paint");
}

#[test]
fn impasto_panel_events_reach_the_brush() {
    // The seam test in the panel proves the widget forwards the event; this proves the TOOL consumes it
    // and the value lands in the spec. Both halves are needed: either one alone leaves a knob that looks
    // wired and is not (`feedback_tool_unit_green_integration_dead`).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);

    // The medium is picked from the Paint Mode dropdown (2026-07-22), not a section checkbox.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_MEDIA,
        "2".into(),
    ));
    assert!(t.paint.brush.impasto, "picking Impasto reached the brush");

    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_IMPASTO_DEPTH, -0.4));
    assert!(
        (t.paint.brush.impasto_depth + 0.4).abs() < 1e-6,
        "Depth, negative (carving) and all"
    );

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_SOURCE_GRAIN));
    assert_eq!(t.paint.brush.impasto_source, DepthSource::Grain);

    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_DRAW_DEPTH));
    assert_eq!(t.paint.brush.impasto_draw_to, DrawTo::Depth);

    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_SMOOTHING,
        0.9,
    ));
    assert!((t.paint.brush.impasto_smoothing - 0.9).abs() < 1e-6);

    // The canvas half.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_SHOW));
    assert!(!t.paint.impasto_show, "Show Impasto toggled off");
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_LIGHT_ANGLE,
        200.0,
    ));
    assert_eq!(t.paint.impasto_rig.lights[0].angle_deg, 200);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_IMPASTO_LIGHT_ELEV,
        1.0,
    ));
    assert_eq!(
        t.paint.impasto_rig.lights[0].elev_deg, 5,
        "elevation floors at 5° — a grazing light divides by ~0"
    );
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_IMPASTO_SHINE, 0.7));
    assert!((t.paint.brush.impasto_shine - 0.7).abs() < 1e-6);

    // Reset restores the settings — and must NOT delete relief the artist already sculpted.
    t.paint.brush.impasto = true;
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    let sculpted = relief(&t);
    assert!(
        sculpted.iter().any(|&v| v != 0.0),
        "there is relief on the canvas"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_IMPASTO_RESET));
    assert!(!t.paint.brush.impasto, "Reset restored the defaults");
    assert_eq!(
        relief(&t),
        sculpted,
        "...and did NOT delete the artist's sculpting — Reset is for the SETTINGS"
    );
}

/// The relief a straight Grain-sourced stroke lays down under `mapping` — the shared fixture for the
/// two questions below (is there relief at all, and does it corrugate).
fn grain_relief_stroke(mapping: ph2d_painter_brush::TextureMapping) -> (Vec<f32>, usize) {
    use ph2d_painter_brush::TextureKind;
    let size = 320u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let mut b = BrushSpec {
        radius_px: 40.0, // spacing = 0.10 × 2 × 40 = 8 px exactly
        color: [0.9, 0.1, 0.1],
        space_attenuation: false,
        impasto: true,
        impasto_depth: 0.7,
        impasto_source: DepthSource::Grain,
        impasto_smoothing: 0.0,
        ..Default::default()
    };
    b.texture.kind = TextureKind::Noise;
    b.texture.mapping = mapping;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let step = b.dab_spacing_px().round().max(2.0) as usize;
    t.on_canvas_pointer(cp([60.0, 160.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([280.0, 160.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([280.0, 160.0], PointerPhase::Up));
    let h = relief(&t);
    // The height straight down the centreline of the stroke.
    (
        (70..270).map(|x| h[(160 * size + x) as usize]).collect(),
        step,
    )
}

/// Peak relief of that stroke — used to keep the ratio below from being computed over an empty field.
fn relief_peak(mapping: ph2d_painter_brush::TextureMapping) -> f32 {
    grain_relief_stroke(mapping)
        .0
        .iter()
        .fold(0.0f32, |m, &v| m.max(v.abs()))
}

/// How much of the height variance along a straight stroke is a pure function of the DAB PHASE
/// (`x mod spacing`). 1.0 = the relief is corrugated at exactly the dab pitch; ~0 = it is not.
fn dab_phase_variance(mapping: ph2d_painter_brush::TextureMapping) -> f32 {
    let (line, step) = grain_relief_stroke(mapping);
    let mean = line.iter().sum::<f32>() / line.len() as f32;
    let total: f32 = line.iter().map(|x| (x - mean) * (x - mean)).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut explained = 0.0f32;
    for ph in 0..step {
        let bin: Vec<f32> = line.iter().skip(ph).step_by(step).copied().collect();
        if bin.is_empty() {
            continue;
        }
        let bm = bin.iter().sum::<f32>() / bin.len() as f32;
        explained += bin.len() as f32 * (bm - mean) * (bm - mean);
    }
    explained / total
}

#[test]
fn impasto_grain_relief_corrugates_unless_the_grain_is_anchored_to_the_canvas() {
    // The ribs Enio saw across every stroke (2026-07-12), quantified — and NOT an engine bug, which
    // is exactly why it needs a gate rather than a fix.
    //
    // A **ViewPlane** grain is DAB-relative: each dab stamps the identical noise in its own frame. At
    // 10% spacing the dabs overlap tenfold, so the height envelope repeats at the dab pitch and the
    // relief corrugates across the travel. Under the dome kernel that was ~100% of the along-stroke
    // variance; the body curve attenuates it (every dab whose SOLID band covers the pixel bids full
    // body, so the envelope keeps more of the grain and less of the silhouette's phase) — measured
    // **0.70** now. Still corduroy, still the wrong arming for `DepthSource::Grain`.
    //
    // Anchor the grain to the CANVAS (Tiled) and consecutive dabs bite different noise: **~0.02** —
    // the marks read as bristle streaks ALONG the path, which is what a loaded brush leaves. The
    // smoke arms it that way; this gate is here so the day someone "simplifies" the mapping, the
    // corduroy does not come back silently.
    use ph2d_painter_brush::TextureMapping;
    // ANTI-VACUITY, twice. (1) `dab_phase_variance` divides by the total variance — a zero relief
    // returns 0 and the "must not corrugate" assertions pass while proving nothing (this gate shipped
    // green in exactly that state for one commit, pre-`GRAIN_GROOVE`). (2) A relief SATURATED flat by
    // the envelope-of-many-bids would also pass them — so the centreline must still carry real groove
    // texture, not a ceiling.
    assert!(
        relief_peak(TextureMapping::ViewPlane) > 0.15 && relief_peak(TextureMapping::Tiled) > 0.15,
        "sanity: both configurations actually lay down relief — else the ratios below are vacuous"
    );
    let (line, _) = grain_relief_stroke(TextureMapping::Tiled);
    let (lo, hi) = line
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    assert!(
        hi - lo > 0.02,
        "sanity: the canvas-anchored grooves are alive on the centreline (spread {:.3}) — a saturated \
         ceiling would sit at ~0.000 and make the phase ratios below vacuous. (Measured 0.043 — the \
         same under the dome kernel, since on the spine w = 1 and body(1) = 1: the grain-coverage fold \
         compresses Noise more than a first guess says.)",
        hi - lo
    );
    let dab_relative = dab_phase_variance(TextureMapping::ViewPlane);
    let canvas_anchored = dab_phase_variance(TextureMapping::Tiled);
    assert!(
        dab_relative > 0.5,
        "a dab-relative grain DOES still corrugate at the dab pitch — that is the physics this gate \
         records (got {dab_relative:.2}; ~1.0 under the dome kernel, 0.70 under the body curve)"
    );
    assert!(
        canvas_anchored < 0.2,
        "a canvas-anchored grain must NOT corrugate: consecutive dabs bite different noise \
         (got {canvas_anchored:.2}, expected ~0.02)"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn flat_probe_exact_smoke_arming() {
    // Enio's second smoke came out completely FLAT. Reproduce the smoke's arming EXACTLY — through the
    // same public setters, in the same order, not by hand-building a BrushSpec — and report what the
    // relief and the shading actually are. (Hand-building the spec is how a probe agrees with itself and
    // misses the product.)
    let size = 240u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t.set_brush_texture_kind(ph2d_painter_brush::TextureKind::Noise.to_u8());
    t.set_brush_texture_mapping(ph2d_painter_brush::TextureMapping::Tiled.to_u8());
    t.toggle_brush_impasto();
    t.set_brush_impasto_depth(0.7);
    t.set_brush_impasto_source(DepthSource::Grain.to_u8());
    t.set_brush_impasto_smoothing(0.15);

    let b = t.paint.brush;
    println!(
        "spec: impasto={} depth={} source={:?} grain_kind={:?} grain_mapping={:?} grain_active={} \
         radius={}",
        b.impasto,
        b.impasto_depth,
        b.impasto_source,
        b.texture.kind,
        b.texture.mapping,
        b.texture.is_active(),
        b.radius_px
    );
    println!(
        "deposits_height={} deposits_color={}",
        b.deposits_height(),
        b.deposits_color()
    );

    t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));

    let h = relief(&t);
    let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let nonzero = h.iter().filter(|v| v.abs() > 1e-6).count();
    println!("relief: {} pixels, max |h| = {hmax:.4}", nonzero);

    println!("impasto_visible = {}", t.impasto_visible());
    t.invalidate_composite();
    let shaded = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);
    let mut worst = 0i32;
    let mut moved = 0u32;
    for i in (0..plain.len()).step_by(4) {
        let d = (i32::from(shaded[i + 1]) - i32::from(plain[i + 1])).abs();
        if d > 2 {
            moved += 1;
        }
        worst = worst.max(d);
    }
    println!("light: {moved} pixels moved >2 levels, worst {worst} levels");

    // Compare against the mapping that DID show relief, and against Grain off entirely — the height is
    // `depth × coverage × w × g`, so a weak `g` alone can gut it.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    for (name, kind, mapping, tex_size) in [
        (
            "Grain OFF (Uniform src)",
            TextureKind::None,
            TextureMapping::Tiled,
            1.0f32,
        ),
        (
            "Noise ViewPlane",
            TextureKind::Noise,
            TextureMapping::ViewPlane,
            1.0,
        ),
        (
            "Noise Tiled size 1.0",
            TextureKind::Noise,
            TextureMapping::Tiled,
            1.0,
        ),
        (
            "Noise Tiled size 0.2",
            TextureKind::Noise,
            TextureMapping::Tiled,
            0.2,
        ),
        (
            "Noise Tiled size 0.1",
            TextureKind::Noise,
            TextureMapping::Tiled,
            0.1,
        ),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: if matches!(kind, TextureKind::None) {
                DepthSource::Uniform
            } else {
                DepthSource::Grain
            },
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = kind;
        b.texture.mapping = mapping;
        b.texture.size = [tex_size, tex_size];
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));
        let h = relief(&t);
        let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        // Steepest local slope — what the light actually reads.
        let mut slope = 0.0f32;
        for y in 1..size as usize - 1 {
            for x in 1..size as usize - 1 {
                let g = (h[y * size as usize + x + 1] - h[y * size as usize + x - 1]).abs() * 0.5;
                slope = slope.max(g);
            }
        }
        t.invalidate_composite();
        let sh = lit(&mut t);
        t.paint.impasto_show = false;
        t.invalidate_composite();
        let pl = lit(&mut t);
        let worst = (0..pl.len())
            .step_by(4)
            .map(|i| (i32::from(sh[i + 1]) - i32::from(pl[i + 1])).abs())
            .max()
            .unwrap_or(0);
        println!(
            "  {name:24} max|h|={hmax:.3}  steepest slope={slope:.4}/px  light moves up to {worst} levels"
        );
    }
}

#[test]
fn impasto_grain_textures_the_body_instead_of_removing_it() {
    // Enio's second smoke came out FLAT (2026-07-12), and this was half the reason. The funnel is
    // `h = depth · coverage · w · g`, so `DepthSource::Grain` was MULTIPLYING the body by the grain —
    // and a Noise grain's samples average well under half. The artist asked for Depth 0.7 and got 0.21:
    // a bristle brush laying a third of the paint it should. A tuft does not deposit a third of the
    // paint; it deposits the paint, with GROOVES in it.
    //
    // (The other half was `SLOPE_GAIN`, which I had picked by taste at 8 — a real stroke's steepest
    // slope is 0.026/px, so it tilted the normal 6° and lit nothing. Both were mine. `SLOPE_GAIN` has
    // since been retired for the physical `DEPTH_UNIT_PX` — impasto_light.rs tells that story.)
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let body = |grain: bool| -> (f32, f32) {
        let size = 240u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            // The REFERENCE radius: size-scaling is off (scale 1) so `uniform_peak` is the bare Depth the
            // sanity assert expects. The Grain-vs-Uniform comparison is a shape property, unaffected by size.
            radius_px: 10.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: if grain {
                DepthSource::Grain
            } else {
                DepthSource::Uniform
            },
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        if grain {
            b.texture.kind = TextureKind::Noise;
            b.texture.mapping = TextureMapping::Tiled;
        }
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([80.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([140.0, 120.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 180.0], PointerPhase::Up));
        let h = relief(&t);
        let peak = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        // Spread of the relief INSIDE the stroke — the striations. A body with no variation is not a
        // bristle mark; a gate that only checked the peak would happily accept flat paint.
        let inside: Vec<f32> = h.iter().copied().filter(|v| v.abs() > 0.05).collect();
        let (lo, hi) = inside
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        (peak, hi - lo)
    };
    let (uniform_peak, _) = body(false);
    let (grain_peak, grain_spread) = body(true);
    assert!(
        (uniform_peak - 0.7).abs() < 0.02,
        "sanity: Uniform lays the Depth the artist asked for ({uniform_peak:.2})"
    );
    assert!(
        grain_peak > uniform_peak * 0.5,
        "the Grain must TEXTURE the body, not remove it: peak {grain_peak:.2} vs Uniform's \
         {uniform_peak:.2} — the artist asked for thick paint and got a film"
    );
    assert!(
        grain_spread > 0.1,
        "...and it must still carry striations ({grain_spread:.2}) — a smooth body is not a bristle mark"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn spacing_probe_relief_must_not_depend_on_dab_pitch() {
    // Enio's experiment (2026-07-12): the SAME brush at spacing 0.1 / 0.05 / 0.01 produces three
    // visibly different reliefs — heavy corduroy, mild, then a smooth tube. That cannot be right: the
    // thickness of paint is a property of the brush and the path, not of how finely the engine chose to
    // sample it.
    //
    // Thesis: the envelope is a `max` of DISCRETE domes. Between two dab centres the distance to either
    // grows, so the max DIPS — and the dip is deepest where the falloff is steep, i.e. on the FLANKS.
    // (My earlier probe sampled the CENTRELINE, where the falloff sits on its plateau and barely dips —
    // which is exactly why it reported "no corrugation" while the screen was corrugated.)
    let size = 300u32;
    let scan = |spacing: f32, off_axis: i32| -> (f32, f32) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform, // NO grain — isolate the geometry
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let y = (150 + off_axis) as usize;
        let line: Vec<f32> = (60..240).map(|x| h[y * size as usize + x]).collect();
        // Ripple: peak-to-peak of the height along a line that SHOULD be perfectly flat (a straight
        // stroke of constant width), and the steepest along-path slope (what the light reads).
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        let ripple = hi - lo;
        let slope = line
            .windows(3)
            .map(|w| ((w[2] - w[0]) * 0.5).abs())
            .fold(0.0f32, f32::max);
        (ripple, slope)
    };
    for spacing in [0.10f32, 0.05, 0.01] {
        let (r_axis, s_axis) = scan(spacing, 0);
        let (r_flank, s_flank) = scan(spacing, 30);
        println!(
            "UNIFORM  spacing {spacing:.2} ({:>4.1} px)  centre: ripple {r_axis:.4} slope {s_axis:.4}  \
             flank: ripple {r_flank:.4} slope {s_flank:.4}",
            spacing * 2.0 * 40.0
        );
    }
    // Now the SMOKE's actual configuration — Grain source over a canvas-anchored noise. If the ribs are
    // here and not in Uniform, the geometry is not the culprit.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let scan_grain = |spacing: f32, mapping: TextureMapping, off_axis: i32| -> (f32, f32) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = mapping;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let y = (150 + off_axis) as usize;
        let line: Vec<f32> = (60..240).map(|x| h[y * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        let slope = line
            .windows(3)
            .map(|w| ((w[2] - w[0]) * 0.5).abs())
            .fold(0.0f32, f32::max);
        (hi - lo, slope)
    };
    for spacing in [0.10f32, 0.05, 0.01] {
        for (name, m) in [
            ("Grain/Tiled    ", TextureMapping::Tiled),
            ("Grain/ViewPlane", TextureMapping::ViewPlane),
        ] {
            let (r, sl) = scan_grain(spacing, m, 0);
            let (rf, slf) = scan_grain(spacing, m, 30);
            println!(
                "{name} spacing {spacing:.2}  centre: ripple {r:.4} slope {sl:.4}  flank: ripple {rf:.4} slope {slf:.4}"
            );
        }
    }
}

#[test]
fn impasto_relief_is_the_same_at_any_dab_spacing() {
    // Enio's experiment, 2026-07-12, and one of the best bug reports this line got: the same brush at
    // spacing 0.1 / 0.05 / 0.01 produced heavy corduroy, mild corduroy, and a smooth tube. Three
    // different paintings from one brush.
    //
    // The thickness of paint is a property of the brush and the PATH. It cannot depend on how finely the
    // engine chose to sample that path — that is an implementation detail leaking onto the canvas.
    //
    // The cause was geometric, not the grain: the envelope was a `max` of discrete DISCS, and between
    // two centres the distance to either grows, so the maximum DIPS. Wider spacing, deeper scallops.
    // (My first probe measured the centreline of a Grain stroke and reported "no corrugation" — it was
    // looking at the wrong thing. The geometry shows up with the grain OFF.)
    //
    // Now each dab sweeps its body BACK along its own heading, so the union is the stroke's true
    // distance field. Measured before: ripple 0.0148 / 0.0025 / 0.0000. After: 0.0000 at every spacing.
    let size = 300u32;
    let stroke = |spacing: f32| -> Vec<f32> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform, // grain OFF — this is about the GEOMETRY
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        relief(&t)
    };
    // A straight stroke of constant width must leave a relief that is FLAT along its length. Any ripple
    // here is the dab pitch printing itself onto the paint.
    let ripple = |h: &[f32], row: usize| {
        let line: Vec<f32> = (60..240).map(|x| h[row * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        hi - lo
    };
    let coarse = stroke(0.10);
    let fine = stroke(0.01);
    assert!(
        coarse.iter().fold(0.0f32, |m, &v| m.max(v)) > 0.6,
        "sanity: the coarse stroke really did lay down a body"
    );
    for row in [150usize, 180] {
        // 180 = 30 px off the axis: the flank, where the falloff is steep.
        assert!(
            ripple(&coarse, row) < 0.002,
            "at spacing 0.10 the relief ripples {:.4} along a stroke that should be flat — the dab \
             pitch is printing itself onto the paint (row {row})",
            ripple(&coarse, row)
        );
    }
    // And the two spacings must agree: same brush, same path, same paint.
    let peak = |h: &[f32]| h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let (pc, pf) = (peak(&coarse), peak(&fine));
    assert!(
        (pc - pf).abs() < 0.02,
        "coarse and fine sampling of the SAME stroke must deposit the same thickness ({pc:.3} vs {pf:.3})"
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn sweep_probe_jitter_spacing() {
    // The sweep reaches back exactly one nominal pitch. Jitter Spacing scatters the dabs, so a gap can
    // open wider than that — does the corrugation come back?
    let size = 300u32;
    for js in [0.0f32, 0.5, 1.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 40.0,
            spacing: 0.10,
            jitter_spacing: js,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Uniform,
            impasto_smoothing: 0.0,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));
        let h = relief(&t);
        let line: Vec<f32> = (60..240).map(|x| h[150 * size as usize + x]).collect();
        let (lo, hi) = line
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &v| (l.min(v), h.max(v)));
        println!("jitter_spacing {js:.1} → ripple {:.4}", hi - lo);
    }
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn spacing_probe_curved_full_field() {
    // The strokes STILL differ with spacing on screen. My gate measured a straight stroke's peak and
    // ripple — which is not the same as "the two paintings are the same painting". Compare the whole
    // field, on a CURVE, and split it: is the difference in the COLOUR, in the RELIEF, or in the LIGHT?
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 260u32;
    let run = |spacing: f32| -> (Vec<u8>, Vec<f32>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        // A curve, like the smoke's S.
        t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
        for p in [[120.0, 90.0], [90.0, 140.0], [130.0, 190.0], [110.0, 225.0]] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([110.0, 225.0], PointerPhase::Up));
        let canvas = (*t.canvas_rgba).clone();
        let h = relief(&t);
        let (comp, _, _) = t.take_preview_arc().expect("preview");
        (canvas, h, (*comp).clone())
    };
    let (c_coarse, h_coarse, l_coarse) = run(0.10);
    let (c_fine, h_fine, l_fine) = run(0.01);
    let _ = (&h_coarse, &h_fine);

    let du8 = |a: &[u8], b: &[u8]| {
        let mut worst = 0i32;
        let mut n = 0u32;
        for i in (0..a.len()).step_by(4) {
            let d = (0..3)
                .map(|c| (i32::from(a[i + c]) - i32::from(b[i + c])).abs())
                .max()
                .unwrap_or(0);
            if d > 8 {
                n += 1;
            }
            worst = worst.max(d);
        }
        (n, worst)
    };
    let (cn, cw) = du8(&c_coarse, &c_fine);
    let (ln, lw) = du8(&l_coarse, &l_fine);
    let hw = h_coarse
        .iter()
        .zip(h_fine.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    let hn = h_coarse
        .iter()
        .zip(h_fine.iter())
        .filter(|(a, b)| (*a - *b).abs() > 0.05)
        .count();
    println!("COLOUR  (canvas_rgba): {cn} px differ >8 levels, worst {cw}");
    println!("RELIEF  (height):      {hn} px differ >0.05,     worst {hw:.3}");
    println!("LIGHT   (composite):   {ln} px differ >8 levels, worst {lw}");

    // Is the colour difference the engine's ratified "spacing changes deposit density" — the thing
    // "Adjust Strength for Spacing" exists to normalise, and which Enio turned OFF by default in
    // 2026-06-24? Turn it back on and see whether the three strokes converge.
    let run_att = |spacing: f32| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            spacing,
            color: [0.9, 0.1, 0.1],
            space_attenuation: true, // <- the only change
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
        for p in [[120.0, 90.0], [90.0, 140.0], [130.0, 190.0], [110.0, 225.0]] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([110.0, 225.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let (an, aw) = du8(&run_att(0.10), &run_att(0.01));
    println!("COLOUR with Adjust Strength ON: {an} px differ >8 levels, worst {aw}");
}

#[test]
fn impasto_depth_and_smoothing_are_live_on_the_stroke_already_painted() {
    // Enio 2026-07-12: "Depth e Smooth devem atualizar em tempo real após o traço ser feito como as
    // outras propriedades fazem." An artist lays a stroke and then dials the thickness in while LOOKING
    // at it — a knob that only affects the next stroke is a knob you have to guess with.
    //
    // The bar is not "something changes". It is: dragging Depth after the fact must land on EXACTLY the
    // relief you would have got by painting with that Depth from the start. A live edit that merely
    // approximates the real thing is a second, silently-divergent code path — and this line has already
    // paid for one of those.
    let paint = |depth: f32, smoothing: f32, retune: Option<(f32, f32)>| -> Vec<f32> {
        let mut t = impasto_canvas(60);
        // The claim is that the knobs REACH the finished stroke, so the fixture asks for that explicitly
        // rather than riding the default (which is OFF since 2026-07-19).
        t.paint.impasto_live_edit = true;
        let mut b = t.paint.brush;
        b.radius_px = 8.0;
        b.impasto_depth = depth;
        b.impasto_smoothing = smoothing;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
        if let Some((d, s)) = retune {
            // The stroke is DONE. Now move the sliders, exactly as the panel would.
            t.set_brush_impasto_depth(d);
            t.set_brush_impasto_smoothing(s);
        }
        relief(&t)
    };
    // Painted at 0.3/0.0, then re-tuned to 0.8/0.6 — versus painted at 0.8/0.6 in the first place.
    let retuned = paint(0.3, 0.0, Some((0.8, 0.6)));
    let native = paint(0.8, 0.6, None);
    assert!(
        native.iter().fold(0.0f32, |m, &v| m.max(v)) > 0.3,
        "sanity: the reference stroke has a real body"
    );
    let worst = retuned
        .iter()
        .zip(native.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-5,
        "re-tuning Depth/Smoothing after the stroke must land on the same relief as painting with them \
         from the start (worst divergence {worst})"
    );

    // Carving live, too — Depth is signed, and flipping it must flip the paint already down.
    let carved = paint(0.5, 0.0, Some((-0.5, 0.0)));
    assert!(
        carved.iter().any(|&v| v < -0.1),
        "dragging Depth negative carves the stroke that is already on the canvas"
    );

    // ...but a SECOND stroke ends the live edit: only the last one is re-derivable, and re-tuning must
    // never resurrect or rescale the ones before it.
    let mut t = impasto_canvas(60);
    t.paint.impasto_live_edit = true; // the claim is about WHICH stroke is live, so live editing is on
    let mut b = t.paint.brush;
    b.radius_px = 8.0;
    b.impasto_depth = 0.5;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([15.0, 15.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([15.0, 15.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([45.0, 45.0], PointerPhase::Up));
    let first_before = relief(&t)[(15 * 60 + 15) as usize];
    // SANITY, and it is load-bearing: committing the second stroke must not have erased the first. I
    // wrote this gate without it, and a mutation that made the live buffer FORGET its ground sailed
    // straight through — because it destroyed the first stroke's relief BEFORE the reference was read,
    // so the test compared zero against zero and approved. Read the reference, then check it is real.
    assert!(
        first_before > 0.2,
        "the first stroke is still on the layer after the second one commits ({first_before})"
    );
    t.set_brush_impasto_depth(1.0);
    let h = relief(&t);
    assert!(
        (h[(15 * 60 + 15) as usize] - first_before).abs() < 1e-5,
        "the FIRST stroke is committed paint — a later Depth drag must not reach back and re-sculpt it"
    );
    assert!(
        h[(45 * 60 + 45) as usize] > first_before * 1.5,
        "...while the last stroke, the live one, does follow the slider"
    );
}

#[test]
fn impasto_strokes_pile_up_only_to_the_glass() {
    // T4.2 — Corel Painter documents the same limit: accumulated impasto "top[s] out and appear[s] as if the
    // strokes are pressed against glass". Strokes ADD (a second stroke genuinely piles more on —
    // `impasto_one_stroke_is_one_thickness_but_two_strokes_add` pins that), but not forever.
    //
    // **Rewritten 2026-07-14 (Enio's smoke).** This used to read the STORED field and demand it stop at
    // exactly 2.0 — i.e. it pinned a hard `clamp`. That clamp was not glass, it was an ERASER: it mapped
    // every height above the ceiling to the same number, so the brush-marks up there were deleted rather
    // than compressed, the plateau's gradient went to zero, and the light rendered the artist's hardest work
    // as a dead flat plate (see `impasto_ceiling::soft_ceiling`). Two strokes of Inflate and the sculpture was a mesa.
    //
    // So the ceiling moved to the LIGHT and the buffer tells the truth. The gate follows: the paint really
    // is three loads thick, and it really does *top out* — the two claims are no longer the same claim.
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto_depth = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    for _ in 0..3 {
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    }
    let h = relief(&t)[(20 * 40 + 20) as usize];
    assert!(
        (h - 3.0).abs() < 1e-4,
        "three full loads of paint ARE three loads — the buffer keeps the relief the artist built, so the \
         sculpt's plane fits and ball offsets reason about a real surface. Got {h}."
    );
    // …and the APPEARANCE is three loads too: **weight-proportional, no glass** (Enio's 2nd correction,
    // 2026-07-14 — *"subir na proporção real do peso da ferramenta"*). Three loads is far below the knee,
    // so the ceiling is the identity here and a stroke adds exactly its weight.
    let seen = super::impasto_ceiling::soft_ceiling(h);
    assert!(
        (seen - h).abs() < 1e-4,
        "three loads should LOOK like three loads (linear, weight-proportional), not {seen:.3}. The old \
         ceiling topped out at 2 and made stacking a fight; the artist wanted paint that piles."
    );
    // The runaway guard is still THERE — it just lives far out of reach: an absurd pile is bounded (the
    // light never sees an infinite slope) and bounded SMOOTHLY (never a flat clamp).
    assert!(
        super::impasto_ceiling::soft_ceiling(1e6) < super::impasto_ceiling::H_ASYMPTOTE,
        "the far-field guard is gone: an unbounded pile would hand the light an infinite normal"
    );
    assert!(
        super::impasto_ceiling::soft_ceiling(1e6) > super::impasto_ceiling::H_KNEE + 1.0,
        "…but it is a GUARD, not a low ceiling — it must sit far above anything an artist reaches"
    );
}

#[test]
fn the_glass_ceiling_compresses_the_marks_it_does_not_erase_them() {
    // **Enio's screenshot, as a number** (2026-07-14): *"em 3 pinceladas toda escultura é achatada no teto"*.
    //
    // A hard clamp maps EVERY height above the ceiling to the same value. So two texels that differ by a
    // brush-mark — the whole reason to sculpt at all — come out IDENTICAL, the plateau's gradient is exactly
    // zero, and the light (which shades from `∇h`) has nothing to draw. The tool erased the work.
    //
    // The soft ceiling's slope is `1/(1+t)²`: small on a huge pile, never zero. Marks survive with less
    // contrast, which is what *pressed against glass* actually means.
    //
    // **Mutation that must bleed:** put the clamp back (`h.clamp(-H_KNEE, H_KNEE)` in `soft_ceiling`).
    let mark = 0.08f32; // a brush-mark's worth of relief, in loads
    for base in [2.5f32, 4.0, 8.0, 20.0] {
        let a = super::impasto_ceiling::soft_ceiling(base);
        let b = super::impasto_ceiling::soft_ceiling(base + mark);
        assert!(
            b > a,
            "at {base} loads a {mark}-load mark VANISHED under the ceiling ({a} vs {b}). That is not a \
             ceiling, it is an eraser — and it erases exactly where the artist worked hardest."
        );
    }
    // …and it really is a CEILING: the higher the pile, the less of the mark survives.
    let low = super::impasto_ceiling::soft_ceiling(30.0 + mark)
        - super::impasto_ceiling::soft_ceiling(30.0);
    let high = super::impasto_ceiling::soft_ceiling(120.0 + mark)
        - super::impasto_ceiling::soft_ceiling(120.0);
    assert!(
        high < low * 0.5 && high > 0.0,
        "the mark should read fainter on a tall pile than on a short one (got {low:e} vs {high:e}) — that \
         IS the glass, and a ceiling that compressed nothing would be no ceiling at all"
    );
}

#[test]
fn below_the_knee_the_ceiling_is_the_identity_byte_for_byte() {
    // Everything painted before the ceiling changed must render EXACTLY as it did. The knee sits where the
    // old hard clamp did, so the whole of the old linear range passes through untouched — no goldens move,
    // no canvas shifts by a level. (A ceiling you cannot introduce without repainting the artist's work is a
    // ceiling you cannot introduce.)
    //
    // **Mutation that must bleed:** start the compression at zero (drop the `if a <= H_KNEE` early return).
    for i in 0..=2000 {
        let h =
            (f32::from(i16::try_from(i).unwrap_or(0)) / 2000.0) * super::impasto_ceiling::H_KNEE;
        for h in [h, -h] {
            assert_eq!(
                super::impasto_ceiling::soft_ceiling(h).to_bits(),
                h.to_bits(),
                "the ceiling touched a height of {h}, which is below the knee"
            );
        }
    }
    // …and it is C¹ at the knee: no crease where the two halves meet (a crease IS a slope, and the light
    // would draw a ring around every pile at exactly two loads).
    let k = super::impasto_ceiling::H_KNEE;
    let e = 1e-3f32;
    let below =
        (super::impasto_ceiling::soft_ceiling(k) - super::impasto_ceiling::soft_ceiling(k - e)) / e;
    let above =
        (super::impasto_ceiling::soft_ceiling(k + e) - super::impasto_ceiling::soft_ceiling(k)) / e;
    assert!(
        (below - 1.0).abs() < 1e-2 && (above - 1.0).abs() < 2e-2,
        "the ceiling creases at the knee (slope {below} below, {above} above) — the light would draw a ring \
         around every pile at exactly {k} loads"
    );
}

#[test]
fn a_pile_past_the_ceiling_still_catches_the_light() {
    // The same failure as `the_glass_ceiling_compresses_the_marks…`, measured where Enio measured it: **on
    // the screen**. The unit gate above says the numbers survive; this one says the PIXELS do.
    //
    // A relief three loads high, with a fine ridge on top of it. Under the hard clamp every one of those
    // texels became 2.0, the gradient vanished, and the plateau came out as one flat colour — which is the
    // screenshot. It has to shade.
    //
    // **Mutation that must bleed:** the clamp, again — this time through the whole light pass.
    let size = 40u32;
    let mut t = impasto_canvas(size);
    let layer = t.layers.active().expect("a layer");
    let n = (size * size) as usize;
    // Three loads of paint, with a 0.15-load ridge running down it. Well past the old ceiling.
    let field: Vec<f32> = (0..n)
        .map(|i| {
            let x = (i as u32 % size) as f32;
            3.0 + if (x - 20.0).abs() < 3.0 { 0.15 } else { 0.0 }
        })
        .collect();
    t.heights.insert(layer, std::sync::Arc::new(field));
    t.covers.insert(layer, std::sync::Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    t.mark_dirty(super::Region {
        x: 0,
        y: 0,
        w: size,
        h: size,
    });
    t.invalidate_composite();

    let px = lit(&mut t);
    let row = 20usize;
    let lum = |x: usize| -> i32 {
        let i = (row * size as usize + x) * 4;
        i32::from(px[i]) + i32::from(px[i + 1]) + i32::from(px[i + 2])
    };
    // The ridge's two walls are at x ≈ 17 and x ≈ 23. Somewhere across them the light must MOVE.
    let flat = lum(6);
    let swing = (10..=30).map(|x| (lum(x) - flat).abs()).max().unwrap_or(0);
    assert!(
        swing >= 6,
        "a ridge sitting three loads up rendered as FLAT ({swing} levels of swing across it). Every texel \
         there was clamped to the same height, so the surface has no slope and the light has nothing to \
         draw — the artist's work is erased exactly where they worked hardest. This is Enio's screenshot."
    );
}

#[test]
fn impasto_body_zero_obeys_the_falloff() {
    // Enio's smoke on the Fase 4 body (2026-07-12): "parece ter perdido a capacidade de obedecer
    // toda a suavidade do falloff — não consigo relevos perfeitamente arredondados como antes."
    // He is right: the body curve crushed EVERY profile to plateau + wall, and the plan §0 promise
    // ("a Shape-Tone ramp vira escultura") died with it. The state of the art ships both schools
    // behind a control (PS Technique Smooth↔Chisel; Blender Draw vs Layer brushes) — so does the
    // brush now: **Body = 0** must hand the cross-section back to the silhouette, exactly.
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
    b.impasto_source = DepthSource::Uniform;
    b.impasto_smoothing = 0.0; // the raw deposit IS the claim — no settling on top
    b.impasto_body = 0.0; // the round school
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [40.0 + 10.0 * f32::from(i as u8), 80.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));

    let h = relief(&t);
    let at = |d: u32| h[((80 + d) * size + 80) as usize];
    let spine = at(0);
    assert!(spine > 0.6, "sanity: full depth on the spine");
    // A dome, not a mesa: the height falls from the very centre (no plateau)...
    assert!(
        at(10) < 0.97 * spine,
        "no plateau — the falloff's curve starts at the spine ({} vs {spine})",
        at(10)
    );
    // ...keeps falling monotonically...
    assert!(
        at(10) > at(20) && at(20) > at(30),
        "monotone rounded flank ({} > {} > {})",
        at(10),
        at(20),
        at(30)
    );
    // ...and the soft tail CARRIES relief again (the wall is gone; body 1 zeroes this pixel).
    assert!(
        at(30) > 0.01,
        "the falloff's soft tail sculpts the relief at Body 0 ({})",
        at(30)
    );
    // And the round school must not resurrect the halo: the tail has height now, but the light
    // still ignores paint that is not there — bare-white pixels do not move.
    t.paint.impasto_rig.lights[0].angle_deg = 90;
    t.paint.impasto_rig.lights[0].elev_deg = 45;
    t.set_impasto_shine(0.0);
    t.invalidate_composite();
    let img = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base = lit(&mut t);
    let mut worst = 0i32;
    for i in (0..base.len()).step_by(4) {
        if 255 - i32::from(base[i + 1]) > 10 {
            continue; // real paint — allowed to shade
        }
        worst = worst.max((i32::from(img[i + 1]) - i32::from(base[i + 1])).abs());
    }
    assert!(
        worst <= 8,
        "rounded relief must not shade the near-invisible tail (worst drift {worst} levels)"
    );
}

/// Paint one stroke on a fresh canvas with `arm` applied to the brush, then apply `edit` through the
/// PUBLIC setters (the panel's own route) and return the relief. With `edit` a no-op this is simply
/// "what the brush painted".
fn impasto_stroke_then_edit(
    arm: impl FnOnce(&mut BrushSpec),
    edit: impl FnOnce(&mut PainterTool),
) -> Vec<f32> {
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 120u32;
    let mut t = impasto_canvas(size);
    // This helper exists to prove the knobs are LIVE, so it must ASK for live editing rather than inherit
    // it: "Adjust Last Stroke" is the artist's default and it went OFF on 2026-07-19. The capability and
    // the default are different claims, and only the latter belongs to `a_fresh_brush_does_not_adjust…`.
    t.paint.impasto_live_edit = true;
    let mut b = t.paint.brush;
    b.radius_px = 24.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 0.6;
    b.impasto_body = 1.0;
    b.impasto_smoothing = 0.0;
    b.impasto_source = DepthSource::Uniform;
    b.texture.kind = TextureKind::Noise; // a grain to carve, so Depth Source has something to say
    b.texture.mapping = TextureMapping::Tiled;
    arm(&mut b);
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 70.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    edit(&mut t);
    relief(&t)
}

#[test]
fn impasto_every_body_knob_edits_the_last_stroke_live() {
    // Enio, 2026-07-12: "coloque todos os parâmetros vivos em tempo real para ajustes depois do traço."
    //
    // THE claim, stated so it can only be true one way: for every knob in the Body card, dialling it
    // AFTER the stroke gives the same relief as having painted the stroke with it from the start. That
    // is only possible because the stroke stores its INGREDIENTS (the paint it laid + the grain it
    // sampled) and the relief is a pure function of them — bake anything into the height that
    // `derive_height` cannot see and this goes red, which is exactly what Body and Depth Source did
    // before this gate existed (they were dead after pen-up, and the panel said nothing).
    type Arm = fn(&mut BrushSpec);
    type Edit = fn(&mut PainterTool);
    let cases: [(&str, Arm, Edit); 4] = [
        (
            "Depth",
            |b| b.impasto_depth = -0.9,
            |t| t.set_brush_impasto_depth(-0.9),
        ),
        (
            "Body",
            |b| b.impasto_body = 0.0,
            |t| t.set_brush_impasto_body(0.0),
        ),
        (
            "Depth Source",
            |b| b.impasto_source = DepthSource::Grain,
            |t| t.set_brush_impasto_source(DepthSource::Grain.to_u8()),
        ),
        (
            "Smoothing",
            |b| b.impasto_smoothing = 0.8,
            |t| t.set_brush_impasto_smoothing(0.8),
        ),
    ];
    let baseline = impasto_stroke_then_edit(|_| {}, |_| {});
    for (name, arm, edit) in cases {
        let painted_with_it = impasto_stroke_then_edit(arm, |_| {});
        let edited_after = impasto_stroke_then_edit(|_| {}, edit);
        // The knob must actually DO something (else the equality below is vacuous — the trap that let
        // a dead knob ship green once already).
        let moved = painted_with_it
            .iter()
            .zip(baseline.iter())
            .filter(|(a, b)| (*a - *b).abs() > 1e-4)
            .count();
        assert!(
            moved > 200,
            "{name}: the knob changes the deposit at all ({moved} px moved) — else this gate is vacuous"
        );
        assert_eq!(
            painted_with_it.len(),
            edited_after.len(),
            "{name}: same canvas"
        );
        let worst = painted_with_it
            .iter()
            .zip(edited_after.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 1e-5,
            "{name}: dialling it AFTER the stroke must give the relief of having painted with it \
             (worst pixel differs by {worst})"
        );
    }
}

// ── The relief obeys the brush SIZE (Enio's smoke of 2026-07-14) ─────────────────────────────────────

/// Paint one dab at `radius`, on its own canvas, and hand back the peak relief and the whole height field.
fn one_dab_relief(radius: f32, depth: f32) -> (f32, Vec<f32>, u32) {
    let size = 220u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 1.0,
        falloff: Falloff::Smooth, // a rounded body, so the slope — and thus the light — is the whole story
        color: [0.7, 0.1, 0.1],
        space_attenuation: false,
        impasto: true,
        impasto_depth: depth,
        impasto_body: 0.0, // the paint's own profile: relief follows the falloff, which is the point here
        impasto_smoothing: 0.0,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let c = f32::from(u8::try_from(size / 2).unwrap_or(110));
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c, c], PointerPhase::Up));
    let h = relief(&t);
    let peak = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    (peak, h, size)
}

/// **A bigger brush lays proportionally taller relief.** (Enio: *"a altura do relevo não está vinculada ao
/// tamanho do pincel, mas é fixa."*)
///
/// The peak height scales with the radius, so the mound's aspect ratio (height ÷ width) is constant — which
/// is what makes the falloff read at every scale instead of flattening out under a big brush. A dab at the
/// reference radius is still exactly its Depth, so nothing an artist painted with a default brush changed.
///
/// **Mutation that must bleed:** drop the `size_scale` from `derive_height` — every radius peaks at the same
/// Depth again, which is the bug.
#[test]
fn the_relief_height_scales_with_the_brush_size() {
    let depth = 0.5f32;
    let (small, _, _) = one_dab_relief(10.0, depth); // the reference radius
    let (big, _, _) = one_dab_relief(40.0, depth);

    assert!(
        (small - depth).abs() < 0.03,
        "a dab at the reference radius should peak at its Depth ({depth}), got {small} — the scaling must be \
         1 there, or every canvas painted with the default brush just changed height"
    );
    // Four times the radius, four times the peak (± the settle/quantisation slack).
    let ratio = big / small;
    assert!(
        (ratio - 4.0).abs() < 0.4,
        "a 4×-bigger brush laid {ratio:.2}× the relief, not ~4×. The height is not tracking the size, so a \
         big brush's mound spreads its Depth over a huge footprint and reads as flat paint — which is \
         exactly Enio's report."
    );
}

/// **A big brush still shows RELIEF — it is not just flat paint.** The appearance half of Enio's report,
/// measured where he measured it: on the screen.
///
/// A flat disc of paint has `n_z = 1` everywhere, so the light — which is RELATIVE (a pixel divided by the
/// flat response) — does **nothing** inside it: lit and unlit are the same pixels. That was the big dab under
/// the old fixed-height deposit: its Depth smeared over a huge footprint, `n_z ≈ 1`, the light drew a flat
/// disc — *"apenas tinta"*. So the oracle is exactly that: **how far does turning the light on move the
/// interior of the dab?** With the height tracking the size, a big brush is a dome and the light has a great
/// deal to say; without it, the interior barely moves.
///
/// The oracle is the light, not the buffer ([[feedback_oracle_must_model_appearance_not_implementation]]).
///
/// **Mutation that must bleed** (checked): drop the `size_scale` from `derive_height` — the big dab flattens,
/// the light stops moving its interior, and the shading collapses to the rim.
#[test]
fn a_big_brush_still_shows_relief_it_is_not_just_flat_paint() {
    // How much the RELIEF's light changes a dab, cancelling the paint underneath. Two identical dabs —
    // one with impasto (relief + light), one without (`impasto: false`, the same colour and the same
    // coverage, no relief) — differ ONLY by the shading. Diff them per pixel and the paint blend, the paper
    // edge, everything but the light drops out. A flat disc has `n_z = 1` and the two are equal; a dome's
    // walls fall away and they diverge.
    //
    // (This is the honest form of the light-toggle the first draft used: that toggled `impasto_show` on ONE
    // tool, whose preview cached across the relight, so the "unlit" pass came back equal to the lit one and
    // every dab read as flat. Two fresh tools cannot cache into each other.)
    let shading_departure = |radius: f32| -> i32 {
        let size = 220u32;
        let render = |impasto: bool| -> Vec<u8> {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            let b = BrushSpec {
                radius_px: radius,
                // SOFT, so the relief is a rounded dome and the slope lives across the whole dab — a hard
                // brush is a mesa, flat-topped at every size, and would hide the very thing Enio reported.
                hardness: 0.0,
                falloff: Falloff::Smooth,
                color: [0.7, 0.1, 0.1],
                space_attenuation: false,
                impasto,
                impasto_depth: 0.6,
                impasto_body: 0.0,
                impasto_smoothing: 0.0,
                ..Default::default()
            };
            t.paint.brush = b;
            t.paint.brush_by_mode.fill(b);
            let c = f32::from(u8::try_from(size / 2).unwrap_or(110));
            t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
            t.on_canvas_pointer(cp([c, c], PointerPhase::Up));
            lit(&mut t)
        };
        let on = render(true);
        let off = render(false);
        let w = size as usize;
        let cc = (size / 2) as usize;
        let span = (radius * 0.95) as usize;
        let mut worst = 0i32;
        for x in (cc - span)..=(cc + span) {
            let i = (cc * w + x) * 4;
            let d = (0..3)
                .map(|k| (i32::from(on[i + k]) - i32::from(off[i + k])).abs())
                .max()
                .unwrap_or(0);
            worst = worst.max(d);
        }
        worst
    };

    let small = shading_departure(10.0);
    let big = shading_departure(60.0);
    assert!(
        small > 10,
        "sanity: the small dab's interior is shaded ({small} levels) — if not, the light is off and the \
         comparison below is vacuous"
    );
    assert!(
        big >= small,
        "the light barely touched the big dab's INTERIOR ({big} levels, vs {small} for the small one). A \
         flat disc has n_z = 1 and the relative light does nothing inside it — which is a big brush reading \
         as *\"apenas tinta\"*. The relief has to track the size, or a big brush is paint with no body."
    );
}

/// **Paint stacks in proportion to its weight — no glass to fight.** (Enio: *"o fato de ficar
/// progressivamente mais difícil de subir não é desejável … subir na proporção real do peso da ferramenta."*)
///
/// Six full-Depth strokes over the same spot pile to ≈ six loads, linearly — each stroke adds its whole
/// weight, and the apparent height equals the real height all the way up. The old ceiling topped out at two
/// loads, so the third stroke onward did almost nothing: the wall Enio hit. Now the ceiling is a far-field
/// guard, and nothing in the reachable range presses against it.
///
/// **Mutation that must bleed:** drop `H_KNEE` back to 2 (or 3) — the sixth stroke's apparent height stops
/// tracking the real one and the ratio collapses.
#[test]
fn stacking_is_linear_and_weight_proportional() {
    let mut t = impasto_canvas(48); // reference radius, so a full-Depth stroke is exactly one load
    let mut b = t.paint.brush;
    b.impasto_depth = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let centre = (24 * 48 + 24) as usize;
    let mut last = 0.0f32;
    for n in 1..=6u32 {
        t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
        let stored = relief(&t)[centre];
        let seen = super::impasto_ceiling::soft_ceiling(stored);
        // The stored relief is n loads (stacking adds), and the LIGHT sees all of it — linear, no topping.
        assert!(
            (stored - n as f32).abs() < 0.05,
            "after {n} strokes the relief is {stored}, not ~{n} loads — stacking is not adding a full weight \
             per stroke"
        );
        assert!(
            (seen - stored).abs() < 1e-4,
            "the {n}th load is being COMPRESSED in the display ({seen} vs {stored}) — the artist asked for \
             weight-proportional stacking, and the reachable range must be pure linear"
        );
        // Each stroke moved the surface up by a real, undiminished step — no "progressively harder".
        assert!(
            seen - last > 0.9,
            "the {n}th stroke lifted the surface by only {:.3} of a load — that is the glass Enio rejected",
            seen - last
        );
        last = seen;
    }
}

/// **The relief of scattered dabs is BEADS, not bars — the capsule law** (Enio's live smoke,
/// 2026-07-15, twice: "relevo fora tinta com jitter" and, with the Airbrush, "de um ponto de relevo
/// a outro, uma reta de relevo ligasse os pontos").
///
/// The height pass sweeps each dab's body back to the previous dab's centre — a capsule — so
/// overlapping stamps join into the stroke's true distance field instead of sagging between
/// centres. The sweep's premise is that the segment between the two centres is GUARANTEED PAINT,
/// and three product knobs break it: per-dab position scatter, Jitter-Scale-shrunken dabs, and the
/// Airbrush's timer dabs under a fast cursor. The colour paints beads there; the height swept a
/// TUBE across bare canvas — film + relief with no pigment under them, which the light dutifully
/// shades: grey bars beside the paint. The law now: sweep only when each disc contains the other's
/// centre (`dist <= min(r, r_prev)`); otherwise the dab is a bead, exactly like its pigment.
///
/// The oracle models the APPEARANCE: no strong film (the coverage the light weighs, > 8/255)
/// farther than 8 px from any pigment. The 8 px allowance is the soft-rim rounding band that soft
/// edges always had (measured: every residual sits within 8 px of paint, most within 2); the bars
/// this gate exists for sat 10-80 px out at FULL film, so the gap between artifact and allowance
/// is an order of magnitude.
///
/// **Mutation that must bleed:** drop the `sweepable` gate on `prev_center` in
/// `stamp_dabs_height` (sweep unconditionally, the old behaviour) — both fixtures light up bars.
#[test]
fn the_relief_of_scattered_dabs_is_beads_not_bars() {
    let size = 300u32;
    // Fixture A: the smoke's brush + full position scatter (the first screenshot).
    // Fixture B: the Airbrush with a fast cursor — consecutive timer dabs far apart (the second
    // screenshot: "uma reta de relevo ligando os pontos").
    for (name, method, jitter, step) in [
        ("scatter", 3u8, 1.0f32, 22.0f32),
        ("airbrush jump", 1, 0.0, 60.0),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(40.0);
        t.toggle_brush_impasto();
        t.set_brush_stroke_method(method);
        t.set_brush_jitter_norm(jitter);
        t.on_canvas_pointer(cp([40.0, 150.0], PointerPhase::Down));
        let moves = (220.0 / step) as u32;
        for i in 1..=moves {
            t.on_canvas_pointer(cp([40.0 + step * i as f32, 150.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([260.0, 150.0], PointerPhase::Up));

        let id = t.layers.active().expect("a layer");
        let cov = t.covers.get(&id).map(|c| (**c).clone()).unwrap_or_default();
        assert!(
            cov.iter().filter(|&&c| c > 8).count() > 500,
            "{name}: fixture laid no film — the absence below would be vacuous"
        );
        let w = size as usize;
        let px = &t.canvas_rgba;
        let is_pig = |x: i64, y: i64| -> bool {
            if x < 0 || y < 0 || x >= size as i64 || y >= size as i64 {
                return false;
            }
            let i = (y as usize * w + x as usize) * 4;
            px[i] != 255 || px[i + 1] != 255 || px[i + 2] != 255
        };
        let mut bars = 0usize;
        let mut worst: Option<(i64, i64, u8)> = None;
        for y in 0..size as i64 {
            for x in 0..size as i64 {
                let i = y as usize * w + x as usize;
                if cov[i] <= 8 || is_pig(x, y) {
                    continue;
                }
                let near = (1..=8i64).any(|r| {
                    (-r..=r).any(|dy| {
                        (-r..=r).any(|dx| dx.abs().max(dy.abs()) == r && is_pig(x + dx, y + dy))
                    })
                });
                if !near {
                    bars += 1;
                    if worst.is_none_or(|(_, _, c)| cov[i] > c) {
                        worst = Some((x, y, cov[i]));
                    }
                }
            }
        }
        assert_eq!(
            bars, 0,
            "{name}: {bars} texels carry film > 8 farther than 8 px from ANY pigment (worst \
             {worst:?}) — the light will shade bare canvas there: the grey bar linking the beads. \
             The capsule swept a segment that is not paint."
        );
    }
}

/// **An Anchored ball is as tall committed as it was live — the radius is the third ingredient.**
///
/// Enio's live smoke (2026-07-15, two screenshots): drag an Anchored ball and the relief reads as a
/// strong sphere; release, and it flattens. The arithmetic was in plain sight: the live envelope
/// derives each texel at the DAB's radius (`d.radius_px` — for Anchored, the drag distance), while
/// the commit re-derived the same ingredients at the PANEL brush's radius — and the height scales
/// with the radius (`IMPASTO_REFERENCE_RADIUS_PX`), so a 100 px ball drawn with a 40 px brush came
/// back `40/100` as tall. (`derive_height`'s own comment called per-dab radius "a refinement…, not
/// a visible error" — true for pressure taper, false by 2.5-10x for the drag-sized methods.) The
/// radius is now stored per texel with the same envelope winner as `paint`/`grain`, and the commit
/// derives each texel at the radius that made it.
///
/// The oracle is BIT-equality: with Smoothing 0 the settle is skipped, and the rule of this module
/// is that the relief is *always exactly* `derive_height(ingredients)` — so the committed field
/// must equal the live envelope to the bit. A corollary rides along: the Size slider AFTER a stroke
/// no longer re-scales relief that is already on the canvas.
///
/// **Mutation that must bleed:** in `rebuild_live_layer_relief`, derive at `brush.radius_px`
/// instead of `live_radius[i]` (the old code) — the ball comes back flattened by `brush/drag`.
#[test]
fn an_anchored_ball_is_as_tall_committed_as_it_was_live() {
    let size = 300u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t.toggle_brush_impasto();
    t.set_brush_impasto_smoothing(0.0);
    t.set_brush_stroke_method(2); // Anchored: the drag sizes the disc

    t.on_canvas_pointer(cp([150.0, 150.0], PointerPhase::Down));
    for i in 1u8..=5 {
        t.on_canvas_pointer(cp([150.0 + 20.0 * f32::from(i), 150.0], PointerPhase::Move));
    }
    // The live relief the artist is looking at, one frame before release (nothing committed yet, so
    // the view IS the stroke envelope).
    let live = relief(&t);
    let live_peak = live.iter().fold(0.0f32, |m, &v| m.max(v));
    assert!(
        live_peak > 2.0 * t.paint.brush.impasto_depth.abs() * 40.0 / 10.0,
        "fixture: the drag-sized dab must be derived at the DRAG's radius (live peak {live_peak:.2} \
         should far exceed a brush-radius dab's) — else this gate is comparing two flat balls"
    );
    // Release at the same point: the final stamp is the same disc the artist saw.
    t.on_canvas_pointer(cp([250.0, 150.0], PointerPhase::Up));
    let committed = relief(&t);

    let differing = live
        .iter()
        .zip(&committed)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        differing,
        0,
        "with Smoothing 0 the committed relief must BE the live envelope, to the bit — {differing} \
         texels differ (live peak {live_peak:.2}, committed peak {:.2}). The commit re-derived the \
         stroke at a radius that is not the one that made it: the ball flattens the moment the mouse \
         is released.",
        committed.iter().fold(0.0f32, |m, &v| m.max(v))
    );
}

/// **An undone Line takes its relief with it** (Enio, live smoke 2026-07-15: *"ao usar linha, ao
/// fazer a última etapa do undo, desfez a cor mas restou o relevo"*).
///
/// The shape editors reset the stroke's relief envelope before each re-stamp, so every undo that
/// restores a snapshot WITH a shape rebuilds envelope and pixels in lock-step. The LAST undo
/// restores a snapshot with no shape — that path only cleared the editors, and the previous
/// re-stamp's envelope survived with no pigment under it: a lit crest floating over bare canvas,
/// the same ghost the eraser gate refuses, entering through Ctrl+Z. `restore_model` now resets the
/// envelope unconditionally (a gesture cannot be in flight during an undo, so there is never a
/// live envelope the reset could legitimately lose).
///
/// Both commit paths are covered — the OPEN shape (undo across the live preview) and the applied
/// one — because they die through different code and each once had its own ghost.
///
/// **Mutation that must bleed:** drop the `reset_stroke_height()` call in `restore_model` — the
/// open-shape variant keeps a 4-load crest over zero painted pixels.
#[test]
fn an_undone_line_takes_its_relief_with_it() {
    let size = 300u32;
    for (name, apply) in [("open shape", false), ("applied", true)] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(40.0);
        t.toggle_brush_impasto();
        t.set_brush_stroke_method(5); // Line: authored click-by-click
        for p in [[80.0f32, 150.0], [220.0, 150.0]] {
            t.on_canvas_pointer(cp(p, PointerPhase::Down));
            t.on_canvas_pointer(cp(p, PointerPhase::Up));
        }
        if apply {
            assert!(t.commit_open_shape(), "{name}: fixture applies the shape");
        }
        let id = t.layers.active().expect("a layer");
        let before = t.layer_height_view(id).unwrap_or_default();
        assert!(
            before.iter().any(|&v| v > 0.5),
            "{name}: fixture laid relief — else the absence below is vacuous"
        );

        let mut steps = 0;
        while t.undo_last() && steps < 10 {
            steps += 1;
        }
        assert!(steps > 0, "{name}: fixture had something to undo");
        assert_eq!(
            t.canvas_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
                .count(),
            0,
            "{name}: sanity — the undo took every painted pixel back"
        );
        let after = t.layer_height_view(id).unwrap_or_default();
        let (mut peak, mut n) = (0.0f32, 0usize);
        for &v in &after {
            if v.abs() > 0.05 {
                n += 1;
            }
            peak = peak.max(v.abs());
        }
        assert_eq!(
            n, 0,
            "{name}: the colour is undone but {n} texels still carry relief (peak {peak:.2}) — a \
             lit crest floating over bare canvas. The last undo cleared the shape editors without \
             clearing the stroke envelope they had stamped."
        );
    }
}
