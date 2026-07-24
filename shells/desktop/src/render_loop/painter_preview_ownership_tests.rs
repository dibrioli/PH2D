//! **The shell OWNS its preview buffer — it never holds a clone of the tool's live canvas.**
//!
//! Split from `painter_preview_pipeline_tests.rs` (HR-18 file-LOC cap). Those gates prove the slot
//! the sprite samples equals the tool's composite; these prove the *ownership* invariant that keeps a
//! plain stroke fast: the shell's preview buffer is INDEPENDENT of the tool's `canvas_rgba`, so the
//! tool stays the sole owner and its `stamp_dabs` writes in place instead of copying the whole plane
//! per move. Shared harness (`ENTITY`, `cp`) lives in the pipeline module.

use super::painter_bridge::own_preview_buffer;
use super::painter_preview_pipeline_tests::ENTITY;
use crate::app_state::PainterPreview;
use ph2d_editor::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::sync::Arc;

/// **The shell owns its preview buffer — it NEVER holds a clone of the tool's live canvas.**
///
/// This is the whole fix for the CPU-bound FPS drop. If `own_preview_buffer` handed back the tool's
/// `Arc` (the old `painter_preview = Some(cache { rgba: drained })`), the tool's next `stamp_dabs`
/// `Arc::make_mut` would see a second owner and copy the WHOLE canvas — measured 0.34 ms/move @ 2048²,
/// 10 ms/move @ 4096², flat across brush size, to deposit one dab. The helper must instead return an
/// INDEPENDENT buffer (a full copy on seed, a prior-buffer-plus-region-patch on a partial frame), so
/// the drained `Arc` drops and the tool is left the sole owner ⇒ its write is in place.
///
/// Mutation that must bleed: `own_preview_buffer` returning `Arc::clone(drained)` (the regression) —
/// `Arc::ptr_eq` then holds on both the seed and the partial arm.
#[test]
fn the_shell_owns_its_preview_buffer_never_the_tools_canvas() {
    let (w, h) = (8u32, 8u32);
    // Seed (no prior buffer): a full, INDEPENDENT copy of the composite.
    let drained: Arc<Vec<u8>> = Arc::new((0..w * h * 4).map(|i| i as u8).collect());
    let seeded = own_preview_buffer(None, ENTITY, w, h, &drained, None);
    assert!(
        !Arc::ptr_eq(&seeded, &drained),
        "the seed must be the shell's OWN buffer, not the tool's canvas Arc — holding the tool's Arc \
         is what forced the per-move whole-canvas copy"
    );
    assert_eq!(
        *seeded, *drained,
        "the seed must mirror the composite byte for byte"
    );

    // Partial (prior buffer + a dirty bbox): patch only the region, keep the rest, still independent.
    let prior = PainterPreview {
        entity_bits: ENTITY,
        rgba: Arc::new(vec![0u8; (w * h * 4) as usize]),
        width: w,
        height: h,
    };
    let next: Arc<Vec<u8>> = Arc::new(vec![9u8; (w * h * 4) as usize]);
    let patched = own_preview_buffer(Some(prior), ENTITY, w, h, &next, Some((2, 2, 4, 4)));
    assert!(
        !Arc::ptr_eq(&patched, &next),
        "the partial frame must not alias the tool's canvas either"
    );
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let inside = (2..6).contains(&x) && (2..6).contains(&y);
            assert_eq!(
                patched[i],
                if inside { 9 } else { 0 },
                "patch keeps the dirty bbox from the new composite ({inside}) and the prior pixels \
                 elsewhere, at ({x},{y})"
            );
        }
    }
}

/// **The fix on the clock: a plain stroke is FOOTPRINT-bound once the shell owns its buffer.**
///
/// The bug was the shell holding the tool's canvas `Arc` across the frame, so `stamp_dabs`'
/// `Arc::make_mut` copied the whole plane per move — O(canvas), 0.34 ms/move @ 2048², 10 ms/move @
/// 4096², flat across brush size. Routing the drain through `own_preview_buffer` (driven here, the
/// REAL fn) leaves the tool the sole owner ⇒ the write is in place ⇒ the per-move cost tracks the DAB,
/// not the canvas.
///
/// Asserted as a RATIO (4096² ÷ 2048²), not a wall-clock bar: `ci-test` is opt-level=1 and this
/// machine drifts across a session, but a copy-per-move quadruples with the canvas while an in-place
/// write barely moves. The same run measures the OLD path (stash the drained Arc) as the control —
/// it must still show the ~4× the fix removes, or the fixture stopped exercising the copy.
/// `#[ignore]`: perf, run with `--release --ignored`.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn a_plain_stroke_is_footprint_bound_when_the_shell_owns_its_buffer() {
    use ph2d_editor::tool::CanvasPointer;
    use std::time::Instant;

    fn cpt(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    // `own` = the fix (route the drain through `own_preview_buffer`, drop the tool's Arc). `!own` =
    // the bug (stash the drained tool Arc, as the shell did before this wave).
    fn per_move(size: u32, own: bool) -> f64 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(16.0);
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cpt([40.0, mid], PointerPhase::Down));
        // The shell's stash, one buffer at a time (its `painter_preview.rgba`).
        let mut held: Option<PainterPreview> = None;
        if let Some((d, w, h)) = t.take_preview_arc() {
            let bbox = t.take_preview_upload_bbox();
            let rgba = if own {
                own_preview_buffer(held.take(), ENTITY, w, h, &d, bbox)
            } else {
                d
            };
            held = Some(PainterPreview {
                entity_bits: ENTITY,
                rgba,
                width: w,
                height: h,
            });
        }
        let mut moves = Vec::new();
        for i in 1..=30u32 {
            let x = 40.0 + 20.0 * (i as f32);
            let t0 = Instant::now();
            // The stamp: `Arc::make_mut` copies the whole canvas iff `held` still owns the tool's Arc.
            t.on_canvas_pointer(cpt([x, mid], PointerPhase::Move));
            let drained = t.take_preview_arc();
            moves.push(t0.elapsed().as_secs_f64() * 1e3);
            if let Some((d, w, h)) = drained {
                let bbox = t.take_preview_upload_bbox();
                let rgba = if own {
                    own_preview_buffer(held.take(), ENTITY, w, h, &d, bbox)
                } else {
                    d
                };
                held = Some(PainterPreview {
                    entity_bits: ENTITY,
                    rgba,
                    width: w,
                    height: h,
                });
            }
        }
        let _ = held;
        moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        moves[moves.len() / 2]
    }

    let (own_2k, own_4k) = (per_move(2048, true), per_move(4096, true));
    let (hold_2k, hold_4k) = (per_move(2048, false), per_move(4096, false));
    let ratio_own = own_4k / own_2k.max(1e-6);
    let ratio_hold = hold_4k / hold_2k.max(1e-6);
    eprintln!(
        "[paint-move] OWN  2048²={own_2k:.3} ms 4096²={own_4k:.3} ms  ratio {ratio_own:.1}x\n\
         [paint-move] HOLD 2048²={hold_2k:.3} ms 4096²={hold_4k:.3} ms  ratio {ratio_hold:.1}x  (the bug)"
    );
    assert!(
        ratio_own < 2.0,
        "owning the buffer must keep a plain stroke footprint-bound (flat in canvas size); got \
         {ratio_own:.1}x ({own_2k:.3} -> {own_4k:.3} ms). A ratio near 4x means the per-move \
         whole-canvas copy is back."
    );
    assert!(
        ratio_hold > 2.5,
        "control: stashing the tool's Arc must still show the plane-bound copy the fix removes; got \
         {ratio_hold:.1}x. If it went flat, the fixture stopped exercising the copy (a brush that \
         deposits nothing, or the trivial drain changed)."
    );
}
