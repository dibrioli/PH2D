//! Gates for the Sculpt **session** — when it is born, how long it lives, every way it can end, and what it
//! costs while it does (`docs/Painter/18_plano_sculpt_relevo.md` §4, §10.4).
//!
//! One sentence governs the whole file: **the session lives exactly as long as the gesture is uncommitted.**
//! Everything here is a way of ending a gesture — pen-up, Apply, Cancel, undo, a sprite rebind, a mode
//! switch — and each one is a door the carving can rot behind, because the sculpt is the one channel that
//! rewrites the layer's plane LIVE. Three of the six were found only by walking the list.
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
/// **Mutation that must bleed:** drop `self.end_sculpt_session()` from `commit_drag_preview`.
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

/// **A finished stroke is finished: the card's knobs do not reach back and rewrite it.**
///
/// This wave shipped the opposite. The session was *parked* at pen-up so Radius and Smooth↔Sharpen would
/// re-render the stroke you had just made, riding the Body card's "Adjust Last Stroke". Enio's smoke killed
/// it in one sentence: picking **Sharpen** — in order to sharpen somewhere *else* — converted the Smooth
/// behind him into its opposite.
///
/// The analogy to the deposit was the error. Paint is a **substance**: Depth / Body are properties *of the
/// paint that stroke laid*, so "keep tuning them" is a coherent offer. A sculpt stroke is an **operation**;
/// it leaves nothing behind that has properties, only the relief as it now is. Operations are undone, not
/// re-dialled — and a *verb* that rewrites history when you select it is not an adjustment at all.
///
/// Byte-identical, not "close": a knob that moved one texel of finished work has already broken the promise.
///
/// **Mutation that must bleed:** restore the parking — make `commit_drag_preview` free only the memo
/// (`blurred` / `blur_done`) instead of calling `end_sculpt_session()`.
#[test]
fn the_sculpt_knobs_do_not_touch_a_finished_stroke() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.25, 1.0);
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);
    let finished = heights_of(&t, layer);
    let carved = finished.iter().filter(|v| **v != 0.0).count();
    assert!(
        carved > 200,
        "fixture: the stroke carved nothing — there is no finished work here to leave alone"
    );

    // The pen is UP. The stroke is on the canvas. Now reach for the other verb, and a different scale.
    t.set_sculpt_mode(1); // Sharpen
    t.set_sculpt_radius(1.0);

    let after = heights_of(&t, layer);
    let moved = finished
        .iter()
        .zip(&after)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        moved, 0,
        "picking Sharpen (and turning Radius) after the stroke rewrote {moved} texels of the SMOOTH that \
         was already down. The knobs arm the next stroke; they are not a live filter on the last one — the \
         artist who reaches for the other verb means the next mark, not the one behind them."
    );
}

/// **…and the presence sibling: an OPEN shape still re-renders.** (Deleting `refresh_live_sculpt` outright
/// would pass the gate above while the shape editor's preview quietly lied.)
///
/// A shape editor's stroke is not canvas — it is a preview with an **Apply** button, precisely because it is
/// not committed. Everything about it is live until then (that is the whole point of the mode), so a Sculpt
/// card whose knobs went inert *there* would leave the curve on screen disagreeing with the card that
/// describes it. That is not the rule bending; it is the rule: **the session lives exactly as long as the
/// gesture is uncommitted.**
///
/// **Mutation that must bleed:** make `refresh_live_sculpt` return immediately.
#[test]
fn the_sculpt_knobs_re_render_an_open_shape() {
    let size = 200u32;
    let (mut t, layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.25, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

    // Two points ⇒ a line. NOT applied: the shape is still open and editable.
    for p in [[50.0, 100.0], [150.0, 100.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    let smooth_preview = heights_of(&t, layer);
    assert!(
        t.paint.sculpt.layer.is_some(),
        "fixture: an un-applied shape must keep its session open — there is nothing live to re-render"
    );

    t.set_sculpt_mode(1); // Sharpen, on a shape the artist has not committed
    let sharpen_preview = heights_of(&t, layer);
    let moved = smooth_preview
        .iter()
        .zip(&sharpen_preview)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        moved > 200,
        "switching the verb on an OPEN shape re-rendered only {moved} texels. The preview on screen now \
         shows a Smooth while the card says Sharpen, and only the Apply would reconcile them."
    );
}

/// **Cancelling a shape un-carves it.**
///
/// `cancel_open_shape`'s own comment states the invariant: *"the pixels are peeled back to pristine — so the
/// relief has to go with them"*. The deposit obeys it for free (it stages relief in an envelope and only
/// merges at commit, so dropping the envelope is enough). The sculpt cannot: it rewrites the layer's plane
/// LIVE. So a cancelled Curve used to leave its carving behind — the shape gone, the smoothing permanent,
/// and **no undo entry**, because the shape was never committed. There was no way back.
///
/// Third door of the same house as the stale-Apply and the sprite-rebind, and found by walking the list of
/// every way a gesture can end rather than only the way it usually does.
///
/// **Mutation that must bleed:** delete `self.cancel_sculpt_session()` from `cancel_open_shape`.
#[test]
fn cancelling_a_shape_un_carves_it() {
    let size = 200u32;
    let (mut t, layer, before) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 1.0, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

    for p in [[50.0, 100.0], [150.0, 100.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    let carved = heights_of(&t, layer);
    let touched = before
        .iter()
        .zip(&carved)
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    assert!(
        touched > 200,
        "fixture: the open shape carved only {touched} texels — nothing to un-carve, so nothing to prove"
    );

    assert!(t.cancel_open_shape(), "fixture: Esc must cancel the shape");

    let after = heights_of(&t, layer);
    let left = before
        .iter()
        .zip(&after)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        left, 0,
        "cancelling the shape left {left} texels of its carving on the layer. The pixels peeled back to \
         pristine and the relief did not — a smoothing with no shape to explain it, and no undo entry to \
         reach it, because the shape was never committed."
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
    /// Moves excluded from the STEADY-STATE mean, because they are not steady state.
    ///
    /// Traced 2026-07-18 (INFLATE @2048, with relief): move 0 costs **0.0 ms** — the sculpt session has
    /// not opened yet, so it does no work at all — move 1 costs **8.8**, and moves 2..19 cost 5.7-6.5.
    /// Move 1 is the session's set-up: its buffers allocated and first-touched, the thread pool spun up.
    ///
    /// The baseline run does none of that (with no relief the sculpt is a true no-op, 0.0 ms on every
    /// move), so ALL of it was landing in the delta and being reported as what every move costs. It
    /// inflated the mean on a quiet machine and, on a loaded one, produced the 40 ms single-move outliers
    /// that made this gate fail against code that had not changed.
    const WARMUP_MOVES: usize = 2;
    /// The one-time cost's own bar. Loose on purpose — it is paid once per STROKE, not once per frame —
    /// but it is asserted rather than discarded: a hitch at the start of every stroke is a thing the
    /// artist feels, and sweeping it out of the mean without asserting it anywhere would be hiding a cost
    /// instead of classifying it. Measured at ~9 ms where steady state is ~6.
    const SETUP_KILL_MS: f64 = 40.0;

    let run = |size: u32, with_relief: bool, mode: u8| -> (f64, f64, f64, f64) {
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
        arm_sculpt(&mut t, mode, 1.0, 1.0); // the WIDEST kernel — the worst case, not the comfortable one
        // …and for Inflate the kernel's radius is its DEPTH (a ball of `Depth · DEPTH_UNIT_PX` px), so the
        // Radius slider above says nothing about it. At the default Depth the ball is 8 px and this would be
        // measuring a quarter of the work the artist can ask for. `O(ρ²)` per texel means the comfortable
        // case and the worst case are a factor of FOUR apart, which is the whole distance to the budget.
        t.set_sculpt_depth(1.0); // +1.0 loads ⇒ a 16-px ball: the widest offset there is
        let mut b = t.paint.brush;
        b.radius_px = 100.0;
        t.paint.brush = b;
        t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;

        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        // Every move is timed; the split into set-up and steady state happens after, so which index pays
        // what is READ OFF rather than assumed. (It was assumed once, on this very gate, and the guess
        // was wrong by one: move 0 looked like the expensive one and is in fact the free one.)
        let mut ms: Vec<f64> = Vec::with_capacity(MOVES as usize);
        for i in 0..MOVES {
            let x = 220.0 + f64::from(i) * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x as f32, mid], PointerPhase::Move));
            let _ = t.take_preview_arc(); // sculpt + composite + light: what a frame really costs
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        let setup = ms[..WARMUP_MOVES].iter().copied().fold(0.0f64, f64::max);
        let steady = &ms[WARMUP_MOVES..];
        let worst = steady.iter().copied().fold(0.0f64, f64::max);
        let total: f64 = steady.iter().sum();
        let mean = total / steady.len() as f64;
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([1020.0, mid], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let up = t0.elapsed().as_secs_f64() * 1000.0;
        (mean, worst, up, setup)
    };

    // All THREE engines: Smooth (the tile-memoised blur), Scrape (the per-dab plane fit) and Inflate (the
    // tile-memoised BALL OFFSET — `O(ρ²)` taps per texel, by far the most arithmetic in the file, and the
    // reason the memo it shares with Smooth is not optional). They are different arithmetic with different
    // failure modes, and a budget measured on one of them is a budget for a tool the artist does not have.
    for (label, mode) in [("SMOOTH", 0u8), ("SCRAPE", 3u8), ("INFLATE", 7u8)] {
        for size in [2048u32, 4096] {
            let (off_mean, off_worst, off_up, off_setup) = run(size, false, mode);
            let (on_mean, on_worst, on_up, on_setup) = run(size, true, mode);
            let (mean, worst, up) = (on_mean - off_mean, on_worst - off_worst, on_up - off_up);
            let setup = on_setup - off_setup;
            println!(
                ">>> {label} COST @{size}px: mean {mean:.2} ms/move (steady state, \
             {WARMUP_MOVES} warm-up moves excluded), worst {worst:.2} ms/move \
             (target <=4, kill {KILL_MS}) | SET-UP {setup:.2} ms | PEN-UP {up:.2} ms \
             [baseline {off_mean:.2}, with-relief {on_mean:.2}]"
            );
            assert!(
                setup < SETUP_KILL_MS,
                "{label} @{size}px: the stroke's SET-UP move costs {setup:.2} ms — past the one-time \
             budget of {SETUP_KILL_MS} ms. Charged once per stroke rather than per move, which is not \
             licence for it to grow without limit: this is the hitch the artist feels when a stroke \
             starts."
            );
            assert!(
                mean < KILL_MS,
                "{label} costs {mean:.2} ms/move at {size}px — past the kill criterion of {KILL_MS} ms. \
             For SMOOTH the first suspect is the blur memo: if a tile is recomputed per dab instead of once \
             per stroke, the cost scales with the SPACING slider. For SCRAPE it is the plane fit: if the \
             silhouette is being sampled twice per dab (once for the fit, once for the write) the whole \
             dab walk doubles — the `scratch` buffer exists to make it once."
            );
        }
    }
}

/// **The sculpt session costs what this comment says it costs — and costs nothing once the stroke is down.**
///
/// *A rule that never observes cannot fire* ([[feedback_a_rule_that_never_observes_cannot_fire]]): the
/// audio editor reached 4351 MB while HR-13 sat there adding up numbers nobody had measured. So the planes
/// are counted, on the real thing, rather than reasoned about.
///
/// **While the gesture is in flight**: `amount` (4 B/px) + `blurred` (4 B/px) + the forked `pre` (4 B/px) =
/// **12 B/px**. **Once it is committed**: zero. Not "the memo is freed and 8 B/px is parked for the live
/// knobs" — that was this wave's first cut, and the knobs it was paying for turned out to be a design bug
/// (see [`the_sculpt_knobs_do_not_touch_a_finished_stroke`]). Killing the feature refunded the memory: a
/// canvas you have finished sculpting holds nothing at all.
#[test]
fn the_sculpt_session_costs_twelve_bytes_per_pixel_and_nothing_once_committed() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.5, 1.0);

    // Mid-stroke: pen down, one move, NOT up.
    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));
    let s = &t.paint.sculpt;
    assert!(s.layer.is_some(), "fixture: the gesture is in flight");
    assert_eq!(
        (s.amount.len(), s.memo.len(), s.pre.len()),
        (n, n, n),
        "the session's planes are not canvas-sized — the arithmetic below is measuring the wrong thing"
    );
    let live = s.amount.len() * 4 + s.memo.len() * 4 + s.pre.len() * 4;
    assert_eq!(
        live / n,
        12,
        "a live sculpt stroke costs {} B/px, not the 12 the docs promise",
        live / n
    );

    // Pen-up commits. The gesture is over, so the session is over — every plane, not just the memo.
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Up));
    let s = &t.paint.sculpt;
    assert!(
        s.layer.is_none()
            && s.pre.is_empty()
            && s.amount.is_empty()
            && s.memo.is_empty()
            && s.memo_done.is_empty(),
        "a finished sculpt stroke left its session behind: {} B of `pre` + {} B of `amount` + {} B of memo. \
         Those planes exist to re-render an UNCOMMITTED gesture; this one is on the canvas.",
        s.pre.len() * 4,
        s.amount.len() * 4,
        s.memo.len() * 4
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

/// **A session does not follow the artist to the next sprite.**
///
/// `reset_transient_edit_state` is the rebind teardown, and its own comments name this bug class and kill
/// the Deform twin for it. The sculpt is that structure again: `pre` is the OLD sprite's relief, and the
/// two things standing between it and the new one are exactly the two that do not hold across a rebind —
/// the active-layer id (`LayerStack` restarts `next_id` at 1, so the ids COLLIDE by construction) and a
/// length check (which matches whenever the two sprites are the same size).
///
/// The carrier is an **open shape**: a committed stroke ends its own session now, so the only thing that can
/// still be holding sprite A's relief when sprite B arrives is a gesture that was never applied. The rebind
/// discards the SHAPE (`discard_open_shape`) — the session it was carving in is a separate drop, and that
/// is the line this gate is about.
///
/// The hazard window is *switch sprite, then touch the card* — a pen-down on B would have opened a fresh
/// session and cured it. And a knob edit records no undo entry, so there is no way back.
///
/// **Mutation that must bleed:** delete `self.end_sculpt_session()` from `lifecycle::reset_transient_edit_state`.
#[test]
fn a_session_does_not_follow_the_artist_to_the_next_sprite() {
    let size = 128u32;
    let n = (size * size) as usize;
    let (mut t, _layer_a, _) = sculpt_canvas(size);
    arm_sculpt(&mut t, 0, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.stroke_method = ph2d_painter_brush::StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
    for p in [[30.0, 64.0], [90.0, 64.0]] {
        t.on_canvas_pointer(cp(p, PointerPhase::Down));
        t.on_canvas_pointer(cp(p, PointerPhase::Up));
    }
    assert!(
        t.paint.sculpt.layer.is_some(),
        "fixture: an un-applied shape must leave the session open, holding sprite A's relief"
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
