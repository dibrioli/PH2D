//! Gates for the Sculpt **session** — when it is born, when it is parked, when it dies, and what it
//! costs while it lives (`docs/Painter/18_plano_sculpt_relevo.md` §4, §10.4).
//!
//! A child of [`super`] rather than a sibling, so the fixtures (`sculpt_canvas`, `arm_sculpt`, `drag`)
//! are shared and cannot drift into two subtly different canvases. Split out for the workspace file-LOC
//! cap, and along the seam that was already there: [`super`] gates what the kernel DOES to the relief;
//! this gates the lifetime of the state that lets it.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::sync::Arc;

/// **An applied shape's carving survives the next shape.**
///
/// The shape editors keep the stroke OPEN at pen-up (the shape stays editable until Apply), so
/// `close_stroke` — and with it the session's parking — never runs for them. The deposit already paid for
/// this once: consequence #3 in `stamp_preview::commit_drag_preview`'s doc, where the next pen-down wiped
/// the relief a committed curve had laid.
///
/// For the sculpt it is worse, and this gate exists because the same question was asked of the new
/// channel: a session left open past its Apply means the NEXT shape's very first preview frame calls
/// `restamp_reset_sculpt`, which restores the relief from a `pre` frozen before the applied shape — and
/// silently **erases carving that was already on the canvas**.
///
/// **Mutation that must bleed:** drop `self.commit_stroke_sculpt()` from `commit_drag_preview`.
#[test]
fn an_applied_shape_keeps_its_carving_when_the_next_shape_starts() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 1.0, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

    // Shape 1: a two-point line, then Apply (bake it).
    t.on_canvas_pointer(cp([50.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([50.0, 60.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([150.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([150.0, 60.0], PointerPhase::Up));
    assert!(
        t.line_commit(),
        "fixture: Apply must actually bake the shape"
    );
    let applied = heights_of(&t, layer);
    let carved = before
        .iter()
        .zip(&applied)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        carved > 200,
        "fixture: the first shape carved only {carved} texels — nothing to lose, so nothing to prove"
    );

    // Shape 2, somewhere else entirely. Its first preview frame must not reach back and undo shape 1.
    t.on_canvas_pointer(cp([50.0, 160.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([50.0, 160.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([150.0, 160.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([150.0, 160.0], PointerPhase::Up));
    let after = heights_of(&t, layer);

    // The band the FIRST shape carved (rows well away from the second) must be untouched.
    let mut lost = 0u32;
    for y in 0..110u32 {
        for x in 0..size {
            let i = (y * size + x) as usize;
            if applied[i].to_bits() != after[i].to_bits() {
                lost += 1;
            }
        }
    }
    assert_eq!(
        lost, 0,
        "starting a second shape un-carved {lost} texels of the FIRST one — the sculpt session outlived \
         its Apply, so the new shape's first preview frame restored the relief from a `pre` frozen before \
         the carving that is already on the canvas"
    );
}

/// **Strength 0 is byte-identical — and does not even allocate.**
///
/// The `Arc::ptr_eq` is the sharper half: a sculpt that wrote `h = pre` everywhere would be *value*-
/// identical while quietly forking a canvas-sized plane on every stroke. This says the layer's relief was
/// never touched at all — the strongest form of "no-op" there is, and it needs no number.
///
/// **Mutation that must bleed:** delete the `k_scale <= 0.0` early-out in `render_sculpt`.
#[test]
fn strength_zero_never_touches_the_relief() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    let before = Arc::clone(t.heights.get(&layer).unwrap());
    let before_rgba = Arc::clone(&t.canvas_rgba);

    arm_sculpt(&mut t, 0, 0.5, 0.0); // Strength 0
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);

    let after = t.heights.get(&layer).expect("the relief is still there");
    assert!(
        Arc::ptr_eq(&before, after),
        "at Strength 0 the sculpt still forked the layer's relief plane — it wrote the same values back \
         over themselves, which is byte-identical and still a canvas-sized copy on every stroke"
    );
    assert!(
        Arc::ptr_eq(&before_rgba, &t.canvas_rgba),
        "at Strength 0 the sculpt touched the canvas"
    );
}

/// **Radius and Smooth↔Sharpen re-render the stroke that is already down.**
///
/// This is the property the whole per-stroke session buys (§4.3), and it is the one the artist feels
/// first: a knob that only arms the NEXT stroke is the failure mode this section keeps producing (the
/// Body card was rebuilt twice over exactly this).
///
/// **Mutation that must bleed:** make `commit_stroke_sculpt` call `end_sculpt_session()` instead of
/// parking — the session dies at pen-up and both knobs go inert.
#[test]
fn the_sculpt_knobs_re_render_the_finished_stroke() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.25, 1.0);
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);
    let small_radius = heights_of(&t, layer);

    // Pen is UP. Turn the Radius knob — the stroke on screen must change.
    t.set_sculpt_radius(1.0);
    let big_radius = heights_of(&t, layer);
    let moved = small_radius
        .iter()
        .zip(&big_radius)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        moved > 200,
        "turning Radius after the stroke re-rendered only {moved} texels — the knob is arming the NEXT \
         stroke instead of adjusting the one the artist is looking at"
    );

    // And the verb, too.
    t.set_sculpt_mode(1); // Sharpen
    let sharpened = heights_of(&t, layer);
    let flipped = big_radius
        .iter()
        .zip(&sharpened)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        flipped > 200,
        "switching Smooth → Sharpen after the stroke re-rendered only {flipped} texels — the verb is \
         inert on the finished stroke"
    );
}

/// **The kill criterion**, frozen before the build and mirroring the impasto's: canvas 2048² and 4096²,
/// brush radius 100, a dragged stroke, the widest kernel (Radius 16). Target **≤ 4 ms/move** for the
/// sculpt's own cost, **kill at 8**.
///
/// The number that matters is the DELTA. A frame already costs something without the sculpt, and charging
/// that to the sculpt would flatter it — so the same stroke runs twice, and the baseline is the SAME code
/// path on a layer that carries no relief (where the session never opens and the sculpt is a true no-op).
/// That isolates exactly the arithmetic this wave added and nothing else.
///
/// This is where the tile memo earns its keep. Without it the blur is recomputed inside every dab's
/// footprint, and at 10% spacing that is ~10× the same work; with it, each texel is blurred once per
/// stroke however slowly the artist moves.
///
/// Unlike its impasto sibling, this one ASSERTS. A budget nobody enforces is a number in a comment.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn sculpt_perf_kill_criterion() {
    use std::time::Instant;
    const MOVES: u32 = 20;
    const KILL_MS: f64 = 8.0;

    let run = |size: u32, with_relief: bool| -> (f64, f64, f64) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let layer = t.layers.active().expect("a layer");
        if with_relief {
            let n = (size * size) as usize;
            let relief: Vec<f32> = (0..n)
                .map(|i| {
                    let x = (i as u32 % size) as f32;
                    let y = (i as u32 / size) as f32;
                    0.4 * ((x * 0.031).fract() + (y * 0.017).fract())
                })
                .collect();
            t.heights.insert(layer, Arc::new(relief));
            t.covers.insert(layer, Arc::new(vec![255u8; n]));
            t.sync_relief_flags();
        }
        arm_sculpt(&mut t, 0, 1.0, 1.0); // the WIDEST kernel — the worst case, not the comfortable one
        let mut b = t.paint.brush;
        b.radius_px = 100.0;
        t.paint.brush = b;
        t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let (mut worst, mut total) = (0.0f64, 0.0f64);
        for i in 0..MOVES {
            let x = 220.0 + f64::from(i) * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x as f32, mid], PointerPhase::Move));
            let _ = t.take_preview_arc(); // sculpt + composite + light: what a frame really costs
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            worst = worst.max(ms);
            total += ms;
        }
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([1020.0, mid], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let up = t0.elapsed().as_secs_f64() * 1000.0;
        (total / f64::from(MOVES), worst, up)
    };

    for size in [2048u32, 4096] {
        let (off_mean, off_worst, off_up) = run(size, false);
        let (on_mean, on_worst, on_up) = run(size, true);
        let (mean, worst, up) = (on_mean - off_mean, on_worst - off_worst, on_up - off_up);
        println!(
            "@{size}px r100 radius16 — no relief: mean {off_mean:.2} ms/move, worst {off_worst:.2}, pen-up {off_up:.2}\n\
             @{size}px r100 radius16 — SCULPT : mean {on_mean:.2} ms/move, worst {on_worst:.2}, pen-up {on_up:.2}\n\
             >>> SCULPT COST @{size}px: mean {mean:.2} ms/move, worst {worst:.2} ms/move (target <=4, kill {KILL_MS}) | PEN-UP {up:.2} ms"
        );
        assert!(
            mean < KILL_MS,
            "the sculpt costs {mean:.2} ms/move at {size}px — past the kill criterion of {KILL_MS} ms. \
             The first suspect is the blur memo: if a tile is being recomputed per dab instead of once \
             per stroke, the cost scales with the SPACING slider."
        );
    }
}

/// **The sculpt session costs what this comment says it costs.**
///
/// *A rule that never observes cannot fire* ([[feedback_a_rule_that_never_observes_cannot_fire]]): the
/// audio editor reached 4351 MB while HR-13 sat there adding up numbers nobody had measured. So the
/// planes are counted, on the real thing, rather than reasoned about.
///
/// **During a stroke**: `amount` (4 B/px) + `blurred` (4 B/px) + the forked `pre` (4 B/px) = **12 B/px**.
/// **Parked** (pen-up, waiting for a knob): the memo is freed, so **8 B/px** — less than the impasto's own
/// parked ingredients, which is the bar this had to clear.
#[test]
fn the_sculpt_session_costs_twelve_bytes_per_pixel_and_parks_at_eight() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.5, 1.0);

    // Mid-stroke: pen down, one move, NOT up.
    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));
    let s = &t.paint.sculpt;
    assert!(s.open, "fixture: the stroke is open");
    let live = s.amount.len() * 4 + s.blurred.len() * 4 + s.pre.len() * 4;
    assert_eq!(
        (s.amount.len(), s.blurred.len(), s.pre.len()),
        (n, n, n),
        "the session's planes are not canvas-sized — the arithmetic below is measuring the wrong thing"
    );
    assert_eq!(
        live / n,
        12,
        "a live sculpt stroke costs {} B/px, not the 12 the docs promise",
        live / n
    );

    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Up));
    let s = &t.paint.sculpt;
    assert!(!s.open, "pen-up parks the session");
    assert!(
        s.blurred.is_empty() && s.blur_done.is_empty(),
        "the blur memo survived the pen-up — it is pure derived data, and a live Radius edit would \
         throw it away anyway, so parking it is 4 B/px of pure waste"
    );
    let parked = s.amount.len() * 4 + s.pre.len() * 4;
    assert_eq!(
        parked / n,
        8,
        "a parked sculpt session costs {} B/px, not 8",
        parked / n
    );

    // …and the next pen-down lets go of it entirely: a document you have stopped sculpting pays nothing.
    t.on_canvas_pointer(cp([40.0, 90.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 90.0], PointerPhase::Up));
    t.set_paint_tool_mode("brush");
    let s = &t.paint.sculpt;
    assert!(
        s.amount.is_empty() && s.pre.is_empty() && s.blurred.is_empty(),
        "leaving Sculpt kept the session's planes alive — the tool is gone and the memory is not"
    );
}

/// **Undo and redo inside an open shape leaves the relief where it was.**
///
/// The shape editors' undo rests on one invariant: a mid-preview snapshot can be peeled back to pristine.
/// For pixels that peel is `preview_patch`. For the impasto DEPOSIT it is free — the deposit stages its
/// relief in `stroke_height` and only merges at commit, so a snapshot's `heights` is always pristine.
///
/// **The sculpt breaks it**, and that is precisely why doc 18 §10.4 says *"a sessão entra no
/// `ModelSnapshot`"* — a line this wave read and then did not act on. The sculpt writes the layer's plane
/// LIVE, so every `begin_shape_txn()` captures an already-carved `heights`. Drop the session on restore
/// and the shape editor's next re-stamp opens a FRESH one whose frozen source is that carved plane — and
/// runs the kernel on top of it. `heights = K(K(H₀, a), a)`: smoothed twice, and it compounds on every
/// undo/redo cycle. The PIXELS are perfect the whole time, so nothing else in the system so much as
/// blinks. It is the melt of §4 arriving through the one door §4 did not check.
///
/// **Mutation that must bleed:** replace `self.restore_sculpt(m.sculpt)` in `restore_model` with
/// `self.end_sculpt_session()` — which is exactly what this wave shipped until the audit.
#[test]
fn undo_and_redo_inside_a_shape_do_not_sculpt_twice() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 1.0, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

    for p in [[60.0, 100.0], [140.0, 100.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    let carved = heights_of(&t, layer);

    assert!(t.undo_last(), "fixture: there is something to undo");
    assert!(t.redo_last(), "fixture: …and to redo");
    let after = heights_of(&t, layer);

    let moved = carved
        .iter()
        .zip(&after)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        moved, 0,
        "undo→redo re-smoothed {moved} texels. The sculpt session did not roll back with the relief, so \
         the re-stamp froze the ALREADY-CARVED plane as its source and ran the kernel again. Do it a few \
         more times and the ridge melts away — while the pixels stay perfect and nothing warns you."
    );
}

/// **A parked session does not follow the artist to the next sprite.**
///
/// `reset_transient_edit_state` is the rebind teardown, and its own comments name this bug class and kill
/// the Deform twin for it. The sculpt is that structure again: `pre` is the OLD sprite's relief, and the
/// two things standing between it and the new one are exactly the two that do not hold across a rebind —
/// the active-layer id (`LayerStack` restarts `next_id` at 1, so the ids COLLIDE by construction) and a
/// length check (which matches whenever the two sprites are the same size).
///
/// The hazard window is *switch sprite, then touch the card* — a pen-down on the new sprite would have
/// opened a fresh session and cured it. And a knob edit records no undo entry, so there is no way back.
///
/// **Mutation that must bleed:** delete `self.end_sculpt_session()` from `lifecycle::reset_transient_edit_state`.
#[test]
fn a_parked_session_does_not_follow_the_artist_to_the_next_sprite() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer_a, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.5, 1.0);
    drag(&mut t, &[[30.0, 64.0], [60.0, 64.0], [90.0, 64.0]]);
    assert!(
        t.paint.sculpt.layer.is_some(),
        "fixture: the session is parked, holding sprite A's relief"
    );

    // Sprite B — the SAME size, which is the case that corrupts in silence.
    t.bind_document(2, vec![255u8; n * 4], size, size);
    let layer_b = t.layers.active().expect("a layer on the new sprite");
    let relief_b: Vec<f32> = (0..n).map(|i| ((i % 97) as f32) / 97.0).collect();
    t.heights.insert(layer_b, Arc::new(relief_b.clone()));
    t.sync_relief_flags();

    // …and now, WITHOUT painting a thing, the artist nudges Radius.
    t.set_sculpt_radius(0.9);

    let after = heights_of(&t, layer_b);
    let moved = relief_b
        .iter()
        .zip(&after)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        moved, 0,
        "turning the Radius knob after switching sprites rewrote {moved} texels of the NEW sprite's \
         relief — with the OLD one's. The session outlived the document it was measured against."
    );
}

/// **A feathered selection attenuates the spatula once, not once per pointer event.**
///
/// The selection used to be folded onto the ACCUMULATED `amount` after each batch. `amount` carries every
/// earlier batch, so a texel touched by *k* batches had its first contribution scaled *k* times:
/// `((a₁·s) + a₂)·s`, not `(a₁ + a₂)·s`.
///
/// With a HARD selection `s ∈ {0, 1}` and the multiply is idempotent — which is why the hard-edged gate
/// `the_sculpt_respects_the_selection` stayed green through all of it. **Feather** is a real, shipped
/// slider, and in its boundary band `s` is partial: there the spatula's strength became a function of how
/// many pointer events the OS happened to deliver. It is `a_faster_mouse_does_not_sculpt_deeper` wearing a
/// different hat, and it is why the fold now happens inside the kernel, as the dab lands.
///
/// **Mutation that must bleed:** move the `× sel` out of `accumulate_dab_sculpt` and multiply
/// `amount[i] *= allow` over the batch's rect afterwards.
#[test]
fn a_feathered_selection_does_not_attenuate_once_per_pointer_batch() {
    let size = 200u32;
    let run = |samples: u32| -> Vec<f32> {
        let (mut t, layer, _) = sculpt_canvas(size);
        t.set_rect_selection(0, 0, 120, size);
        t.set_selection_feather(0.35); // a soft edge: the band where `s` is partial
        arm_sculpt(&mut t, 0, 1.0, 0.5);
        let mut path = vec![[40.0, 100.0]];
        for k in 1..=samples {
            path.push([40.0 + 120.0 * (k as f32 / samples as f32), 100.0]);
        }
        drag(&mut t, &path);
        heights_of(&t, layer)
    };
    let coarse = run(1);
    let fine = run(8);

    let differing = coarse
        .iter()
        .zip(&fine)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "through a FEATHERED selection, the same gesture delivered in 1 batch and in 8 left two different \
         reliefs ({differing} texels). The mask is being applied to the running total instead of to each \
         dab, so it compounds once per pointer event — and the artist's polling rate becomes a setting."
    );
}
