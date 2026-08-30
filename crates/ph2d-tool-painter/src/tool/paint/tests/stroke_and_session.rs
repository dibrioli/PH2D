//! **Do ponteiro ao pixel, e a sessão à volta dele.** As fases do ponteiro (Down/Move/Up/Hover), a
//! região suja que o traço declara, o passo de undo que um traço vale, os métodos que depositam sem
//! abrir editor (Space, Airbrush, Anchored, a linha reta e o snap de 45°) — e a sessão da ferramenta:
//! o dock, as configurações partilhadas ou independentes entre ferramentas, e o que a troca de sprite
//! larga.

use super::*;

/// **`canvas_version` moves only when the preview actually changed.**
///
/// The shell keys its GPU-slot upload on this instead of the drained `Arc`'s pointer — which is what
/// lets it own its preview buffer and leave the tool the sole owner of `canvas_rgba` (so a stamp
/// writes in place instead of copying the whole plane per move). The contract: a DIRTY drain bumps
/// the version (pixels moved ⇒ the shell uploads); an IDLE drain returns `None` and leaves it put
/// (⇒ the shell's plan Skips). Break the bump and the shell freezes on a changing canvas; bump it on
/// idle and it re-uploads a static one every frame.
///
/// Mutation that must bleed: drop the `self.preview_version += 1` in `take_preview_arc`.
#[test]
fn canvas_version_advances_on_a_dirty_drain_and_holds_on_an_idle_one() {
    let mut t = white_canvas(64, 4.0);
    let _ = t.take_preview_arc(); // clear any bind-time dirty so v0 is a settled baseline
    let v0 = t.canvas_version();
    assert!(
        t.take_preview_arc().is_none(),
        "a clean canvas has nothing to drain"
    );
    assert_eq!(
        t.canvas_version(),
        v0,
        "an idle frame must not advance the version"
    );

    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    assert!(
        t.take_preview_arc().is_some(),
        "the dab dirtied the preview"
    );
    let v1 = t.canvas_version();
    assert!(
        v1 > v0,
        "a dirty drain must advance the version ({v0} -> {v1})"
    );

    assert!(
        t.take_preview_arc().is_none(),
        "no new paint since the drain"
    );
    assert_eq!(
        t.canvas_version(),
        v1,
        "a second idle frame holds the version"
    );

    t.on_canvas_pointer(cp([40.0, 20.0], PointerPhase::Move));
    assert!(t.take_preview_arc().is_some());
    assert!(
        t.canvas_version() > v1,
        "a second dirty drain advances the version again"
    );
}

#[test]
fn down_paints_into_active_raster_and_marks_dirty() {
    let mut t = white_canvas(64, 6.0);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert_eq!(px(&t, 64, 32, 32), [0, 0, 0, 255], "centre painted black");
    assert!(t.preview_dirty, "preview flagged dirty");
    assert!(t.dirty_rect.is_some(), "dirty rect accumulated");
    // A far corner is untouched.
    assert_eq!(px(&t, 64, 0, 0), [255, 255, 255, 255]);
}

#[test]
fn trivial_stack_stroke_uploads_only_the_dab_bbox_not_the_whole_canvas() {
    // Regression: a single-layer (trivial) stroke must hand the bridge the dab's
    // dirty bbox so it patches only that sub-rect. Forcing `None` here made every
    // painted frame a full clone + premul + full GPU texture upload, O(W×H)
    // regardless of the 10px brush — the 300→150 fps drop.
    let mut t = white_canvas(64, 4.0);
    assert!(
        t.is_trivial_stack(),
        "single opaque Normal raster is trivial"
    );

    // First drain is the source-push seed (no paint yet) → `None` → the bridge
    // does one full upload to seed the GPU texture.
    assert!(t.take_preview_arc().is_some(), "source-push marks dirty");
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "seed frame uploads the full canvas (no dab yet)"
    );

    // Now paint one dab and drain again — the bbox must be present and strictly
    // smaller than the canvas (the dab footprint), not the whole 64×64.
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert!(
        t.take_preview_arc().is_some(),
        "the dab re-dirtied the preview"
    );
    let (bx, by, bw, bh) = t
        .take_preview_upload_bbox()
        .expect("a trivial-stack stroke must carry its dab bbox, not None");
    assert!(bw > 0 && bh > 0, "bbox is non-empty");
    assert!(
        bw < 64 && bh < 64,
        "partial upload, not the full canvas: got {bw}×{bh}"
    );
    assert!(
        bx <= 32 && by <= 32 && bx + bw >= 32 && by + bh >= 32,
        "bbox contains the dab centre (32,32): ({bx},{by},{bw},{bh})"
    );
}

#[test]
fn hover_never_paints() {
    let mut t = white_canvas(32, 4.0);
    let _ = t.take_preview_dirty(); // clear the dirty flag `set_source` raised
    assert!(!t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Hover)));
    assert_eq!(
        px(&t, 32, 16, 16),
        [255, 255, 255, 255],
        "hover left canvas untouched"
    );
    assert!(!t.preview_dirty, "hover did not re-dirty the preview");
}

#[test]
fn drag_dot_follows_cursor_leaving_no_trail() {
    // Blender Drag Dot: one dab follows the cursor (no trail) and only the dab at the release point
    // is committed. The tool restores the pixels under the previous position before re-stamping.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::DragDot;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down)); // dot appears at the press point
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Move)); // dot moves — previous erased
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // dot moves again
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up)); // commit at the release point
    assert_eq!(
        px(&t, 64, 56, 32),
        [0, 0, 0, 255],
        "the dot is committed at the release point"
    );
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "no trail left at the press point"
    );
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "no trail left at the intermediate point"
    );
    assert!(t.paint.stroke.is_none());
    assert!(
        t.paint.drag_preview.is_none(),
        "the restore record is cleared once the dot is committed"
    );
}

#[test]
fn stroke_down_move_up_paints_a_line() {
    let mut t = white_canvas(64, 3.0);
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    // Spacing emits many dabs along the horizontal segment → the midpoint is
    // painted, while a point well off the line stays white.
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "midpoint of the stroke painted"
    );
    assert_eq!(
        px(&t, 64, 32, 10),
        [255, 255, 255, 255],
        "off-line pixel untouched"
    );
    // Stroke ended → no stroke in progress.
    assert!(t.paint.stroke.is_none());
}

#[test]
fn move_without_down_is_ignored() {
    let mut t = white_canvas(32, 4.0);
    assert!(
        !t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Move)),
        "stray move"
    );
    assert_eq!(px(&t, 32, 16, 16), [255, 255, 255, 255]);
}

#[test]
fn alpha_lock_blocks_paint_on_transparency() {
    // Canvas: left half opaque white, right half transparent.
    let size = 16u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 3.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false, // full coverage for the alpha-lock assertion
        ..Default::default()
    };
    // Enable alpha lock on the active layer.
    let active = t.layers.active().expect("active layer");
    t.layers.get_mut(active).expect("layer").alpha_locked = true;

    // Paint on the transparent side → blocked (no alpha created).
    t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 12, 8)[3],
        0,
        "alpha-lock blocked paint on transparency"
    );

    // Paint on the opaque side → recoloured, alpha preserved.
    t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 3, 8),
        [0, 0, 0, 255],
        "recoloured the opaque side"
    );
}

#[test]
fn switching_sprite_while_the_paint_is_still_wet_does_not_index_the_old_moisture_map() {
    // Sweep finding (2026-07-12), same family as Bug #12: `canvas_wet` is the ONE canvas-sized buffer that
    // SURVIVES pen-up (the moisture map dries on the heartbeat, over ~10 s). `dry_canvas_wet` guards it
    // with `is_empty()` — "does it exist?" — and then indexes it with the CURRENT sprite's stride (`fw`)
    // and a `canvas_wet_rect` recorded in the OLD sprite's coordinates. Bind a BIGGER sprite inside the
    // drying window and the very next tick slices past the end of the old buffer.
    // RED without the fix: `paint_tick` PANICS (`range end index … out of range for slice of length 4096`)
    // — the same signature class as Enio's Rake crash, from the same root: a guard that asks "exists?"
    // instead of "does the SHAPE match?".
    let mut t = white_canvas(64, 8.0);
    t.paint.brush.watercolor = true;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));
    assert_eq!(
        t.paint.canvas_wet.len(),
        64 * 64,
        "the wet stroke left a moisture map sized for the 64² sprite"
    );
    assert!(
        t.paint.canvas_wet_rect.is_some(),
        "and a wet rect in the 64² sprite's coordinates"
    );
    // The user clicks a BIGGER sprite while the paint is still drying — one click, nothing else.
    t.bind_document(2, vec![255u8; 512 * 512 * 4], 512, 512);
    t.paint_tick(0.1); // the heartbeat the shell runs every frame → dry_canvas_wet
    assert!(
        t.paint.canvas_wet.is_empty() || t.paint.canvas_wet.len() == 512 * 512,
        "the new sprite must not inherit a moisture map shaped for the old one"
    );
}

#[test]
fn switching_sprite_drops_the_compositor_cut_cache() {
    // Sweep finding (2026-07-12) — the THIRD instance of the Bug #12 family, and the nastiest, because the
    // same-size case does not crash: it corrupts in silence.
    //
    // The compositor caches a "cut point" per Adjustment layer: the composited accumulator BELOW it, a
    // `Vec<[f32;4]>` sized for THAT document's canvas. `set_source` (bind another sprite) builds a fresh
    // `LayerStack` — and `LayerStack::new()` restarts `next_id` at 1, so the new document's layer ids
    // COLLIDE with the old one's by construction. The cut cache was never cleared, and its guard only asks
    // "is there a cut for this id?", never "does that cut have the shape of THIS canvas?".
    //
    // The asymmetry is the tell: `restore_doc` clears compositor_cache / adjustment_cache_pending /
    // dirty_rect / preview_upload_bbox — `set_source` cleared none of them. Same seam, two doors, one
    // locked. Bigger sprite ⇒ the accumulator is indexed past its end (panic). Same size ⇒ the new sprite's
    // Adjustment composites over the OLD sprite's cached layers-below: a silently wrong preview.
    // RED without the fix: `cuts` is non-empty after the rebind (and the 1024² step panics).
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 256 * 256 * 4], 256, 256);
    let adj = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .expect("adjustment added");
    t.set_adjustment_param(adj, 0, 0.8);
    let _ = t.take_preview_arc(); // drains the composite → seeds the cut cache for sprite 1
    assert!(
        !t.compositor_cache.cuts.is_empty(),
        "sprite 1 seeded a cut-point cache sized for its 256\u{b2} canvas"
    );
    // The user clicks a BIGGER sprite.
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    assert!(
        t.compositor_cache.cuts.is_empty(),
        "the cut cache is DOCUMENT-scoped — binding another sprite must drop it"
    );
    // And the new document must composite without reading the old canvas's accumulator.
    let adj2 = t
        .add_adjustment_layer(AdjustmentKind::BrightnessContrast)
        .expect("adjustment added on the new sprite");
    t.set_adjustment_param(adj2, 0, 0.8); // same recycled LayerId as sprite 1's adjustment
    let _ = t.take_preview_arc(); // panicked here before the fix (index past the 256² accumulator)
}

#[test]
fn switching_sprite_does_not_carry_the_old_sprites_selection() {
    // Sweep finding (2026-07-12): the pixel Selection is TOOL-global — it is not in `StashedDoc` (which
    // stashes the LAYER selection) and was never registered in `reset_transient_edit_state`. And
    // `selection_restricts_paint()` asks only "is the mask non-empty?", never "does it belong to THIS
    // sprite?". So the new sprite silently inherited the old one's selection and every stroke outside it
    // was reverted: the "it just doesn't paint and I don't know why" class.
    // RED without the fix: the dab at (48,48) is restored to white by `restore_deselected_region`.
    let mut t = white_canvas(64, 6.0);
    t.set_rect_selection(0, 0, 16, 16); // select a corner of sprite 1
    assert!(t.paint.selection_active);
    t.bind_document(2, vec![255u8; 64 * 64 * 4], 64, 64); // click another sprite (same size = the silent case)
    assert!(
        !t.paint.selection_active,
        "the new sprite starts unselected — the old sprite's selection must not gate its paint"
    );
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down)); // far OUTSIDE sprite 1's selection
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    assert_ne!(
        px(&t, 64, 48, 48),
        [255, 255, 255, 255],
        "the stroke paints — it is not gated by a selection that belongs to another sprite"
    );
}

#[test]
fn dab_bbox_covers_the_paint_write_bounds() {
    // Regression (Enio 2026-06-27): `dab_bbox` is the drag-preview SAVE/RESTORE + dirty-upload region for
    // the fill methods. It MUST be a superset of every paint path's write bounds — `floor(c−r)..ceil(c+r)+1`
    // (the blit/accumulate loop) — or an edge row can paint outside the saved region and never get restored
    // (a CPU trail) / never get re-uploaded (a stale row on the upscaled GPU texture: the thin horizontal
    // lines). The old `round(c)±(ceil(r)+1)` box violated this by 1px for fractional centres (e.g. c=0.4,
    // r=1.7). This pins the invariant directly.
    let t = white_canvas(64, 3.0);
    for &c in &[0.1f32, 0.4, 0.5, 0.6, 0.9, 12.3, 31.5, 47.7] {
        for &r in &[1.0f32, 1.7, 2.5, 3.0, 5.5, 8.2] {
            let want_x0 = (c - r).floor().max(0.0) as i64;
            let want_x1 = ((c + r).ceil() as i64 + 1).min(64);
            let bb = t.dab_bbox([c, c], r).expect("dab in-canvas has a bbox");
            assert!(
                (bb.x as i64) <= want_x0 && (bb.x + bb.w) as i64 >= want_x1,
                "dab_bbox x [{},{}) must cover paint bounds [{want_x0},{want_x1}) for c={c} r={r}",
                bb.x,
                bb.x + bb.w
            );
        }
    }
}

#[test]
fn eraser_removes_alpha_from_opaque_pixels() {
    // Opaque white canvas, hard brush; eraser on → a dab clears alpha.
    let mut t = white_canvas(32, 6.0);
    t.toggle_brush_eraser();
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 16, 16)[3],
        0,
        "eraser cleared alpha at the centre"
    );
    // A far corner is untouched (still opaque).
    assert_eq!(px(&t, 32, 0, 0)[3], 255);
}

#[test]
fn dock_defaults_to_brush_then_toggles() {
    let mut t = PainterTool::default();
    assert!(
        !t.dock_shows_layers(),
        "dock opens on the Brush-properties view (Enio 2026-07-04)"
    );
    t.toggle_dock();
    assert!(
        t.dock_shows_layers(),
        "header toggle flips to the Layers/Effects view"
    );
    t.toggle_dock();
    assert!(!t.dock_shows_layers(), "toggling back returns to Brush");
}

#[test]
fn stroke_is_one_undo_step_and_redoable() {
    let mut t = white_canvas(64, 6.0);
    let pristine = Vec::clone(&t.canvas_rgba); // white, pre-stroke
    assert!(!t.can_undo(), "fresh source has nothing to undo");

    // One stroke (down → up).
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
    assert_ne!(*t.canvas_rgba, pristine, "stroke changed pixels");
    assert!(t.can_undo(), "stroke pushed exactly one undo step");

    // Undo restores the pre-stroke pixels byte-for-byte.
    assert!(t.undo_last());
    assert_eq!(*t.canvas_rgba, pristine, "undo restored the canvas");
    assert!(!t.can_undo(), "one stroke == one undo step");

    // Redo repaints.
    assert!(t.redo_last());
    assert_ne!(*t.canvas_rgba, pristine, "redo repainted the stroke");
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "stroke start back to black"
    );
}

#[test]
fn unbaked_edits_tracked_and_deactivate_defers_the_bake() {
    // Persistence (Enio 2026-06-24): the painter flags unbaked edits so the shell auto-persists them
    // on leave/deactivate. A fresh bind has none; an edit sets the flag; deactivating with edits KEEPS
    // the canvas + defers the bake (so the shell can write it back before teardown).
    use ph2d_editor_core::tool::{RasterEditTool, Tool};
    let mut t = PainterTool::default();
    (&mut t as &mut dyn RasterEditTool).set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    assert!(!t.has_unbaked_edits(), "fresh bind has no unbaked edits");

    // A structural edit (add a layer) marks the canvas unbaked.
    t.add_raster_layer("Layer 2");
    assert!(t.has_unbaked_edits(), "an edit flags unbaked work");

    // Deactivating with unbaked edits defers the bake + keeps the canvas for the shell.
    t.on_deactivate();
    assert!(
        t.take_deferred_bake(),
        "deactivate defers the bake when edited"
    );
    assert!(
        t.has_unbaked_edits(),
        "canvas kept until the shell bakes it"
    );

    // The shell signals the bake landed.
    t.mark_baked();
    assert!(!t.has_unbaked_edits());
}

#[test]
fn deactivate_without_edits_tears_down_immediately() {
    use ph2d_editor_core::tool::{RasterEditTool, Tool};
    let mut t = PainterTool::default();
    (&mut t as &mut dyn RasterEditTool).set_source(vec![0u8; 4 * 4 * 4], 4, 4);
    t.on_deactivate();
    assert!(!t.take_deferred_bake(), "no edits → no deferred bake");
    assert!(!t.has_unbaked_edits());
}

#[test]
fn airbrush_deposits_on_the_tick_at_the_tracked_cursor_not_on_a_bare_move() {
    // End-to-end (tool layer): the airbrush is a timer method — a bare move lays NO dab; the
    // per-frame `on_tick` fires the timer and deposits at the cursor the moves tracked to. Wire
    // path: `on_canvas_pointer(Down/Move)` tracks position, `on_tick(dt_ms)` → `paint_tick` →
    // `Stroke::tick` → `stamp_dabs`. This is the behaviour the §2.1 handoff deferred until `on_tick`
    // drove the timer (it now does).
    use ph2d_editor_core::tool::Tool;
    let mut t = white_canvas(48, 4.0);
    t.paint.brush.stroke_method = StrokeMethod::Airbrush;
    t.paint.brush.airbrush_rate_s = 0.1; // 10 Hz
    t.paint.brush.stabilizer = 0.0; // raw, so the tick lands exactly at the moved-to point
    t.paint.brush.space_attenuation = false; // full coverage for the pixel assertion

    // Down at A: the begin dab paints A (airbrush `emits_on_begin`).
    t.on_canvas_pointer(cp([8.0, 24.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 48, 8, 24),
        [0, 0, 0, 255],
        "down paints the first dab"
    );

    // Move to B with NO tick: the airbrush must not paint on the bare move (timer-only).
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Move));
    assert_eq!(
        px(&t, 48, 40, 24),
        [255, 255, 255, 255],
        "a bare move left a dab — airbrush must deposit only on the timer"
    );

    // One frame of 100 ms = one rate period → the timer deposits one dab at the tracked cursor B.
    t.on_tick(100.0);
    assert_eq!(
        px(&t, 48, 40, 24),
        [0, 0, 0, 255],
        "the tick deposited the airbrush dab at the tracked cursor"
    );

    // Closing the stroke stops the spray: a later tick paints nothing new.
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
    t.on_tick(100.0);
    assert_eq!(
        px(&t, 48, 24, 24),
        [255, 255, 255, 255],
        "no spray after pointer-up"
    );
}

#[test]
fn anchored_stamps_a_drag_sized_disc_centred_on_the_press_point() {
    // Anchored end-to-end (tool layer): press anchors (no paint), the drag sizes a single disc
    // centred on the press point (restore+re-stamp preview — no trail), pen-up commits it.
    let mut t = white_canvas(48, 4.0);
    t.paint.brush.stroke_method = StrokeMethod::Anchored;
    t.paint.brush.edge_to_edge = false;
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    // Press at the anchor — nothing painted yet (interactive).
    t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 48, 10, 24),
        [255, 255, 255, 255],
        "the press alone paints nothing"
    );

    // An intermediate small drag then a larger one: the preview restores between moves, so only the
    // final disc survives — proving the resize leaves no trail.
    t.on_canvas_pointer(cp([16.0, 24.0], PointerPhase::Move)); // small (r≈6)
    t.on_canvas_pointer(cp([26.0, 24.0], PointerPhase::Move)); // grow (r≈16)
    t.on_canvas_pointer(cp([26.0, 24.0], PointerPhase::Up)); // commit

    // Committed disc: centre = anchor (10,24), radius = final drag distance 16.
    assert_eq!(px(&t, 48, 10, 24), [0, 0, 0, 255], "anchor painted");
    assert_eq!(
        px(&t, 48, 22, 24),
        [0, 0, 0, 255],
        "12 px from the anchor is inside the disc"
    );
    assert_eq!(
        px(&t, 48, 0, 0),
        [255, 255, 255, 255],
        "a far corner is outside the disc"
    );
}

#[test]
fn line_paints_a_straight_committed_line_with_no_trail() {
    // Line end-to-end (tool layer), polyline model: click the first corner (a lone point paints nothing),
    // then PRESS the second corner and drag it to a WRONG spot then the final spot (each move previews the
    // straight line, restore + re-stamp → no trail), release, Enter bakes. The wrong drag leaves no trace.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;

    // First corner: a lone point paints nothing (< 2 points).
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 64, 8, 8),
        [255, 255, 255, 255],
        "one corner paints nothing"
    );

    // Second corner: press in empty space (creates it), drag to a WRONG spot (vertical) then the final
    // spot (horizontal), release. Enter bakes the line.
    t.on_canvas_pointer(cp([8.0, 56.0], PointerPhase::Down)); // create corner 1 (wrong: vertical)
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Move)); // drag to final (horizontal)
    t.on_canvas_pointer(cp([56.0, 8.0], PointerPhase::Up));
    assert!(t.commit_open_shape(), "Enter baked the open line");

    // The committed line is horizontal at y=8 from (8,8) to (56,8).
    assert_eq!(px(&t, 64, 8, 8), [0, 0, 0, 255], "anchor end painted");
    assert_eq!(
        px(&t, 64, 32, 8),
        [0, 0, 0, 255],
        "midpoint of the committed line painted"
    );
    assert_eq!(px(&t, 64, 56, 8), [0, 0, 0, 255], "release end painted");
    // The discarded vertical drag left no trail (restored before the horizontal re-stamp).
    assert_eq!(
        px(&t, 64, 8, 32),
        [255, 255, 255, 255],
        "the discarded vertical drag left no trail"
    );
}

#[test]
fn snap_to_45_projects_onto_the_eight_rays() {
    let a = [0.0, 0.0];
    assert_eq!(
        brush_settings::snap_to_45(a, [10.0, 1.0]),
        [10.0, 0.0],
        "near-horizontal → flat"
    );
    assert_eq!(
        brush_settings::snap_to_45(a, [1.0, 10.0]),
        [0.0, 10.0],
        "near-vertical → vertical"
    );
    assert_eq!(
        brush_settings::snap_to_45(a, [-1.0, 10.0]),
        [0.0, 10.0],
        "sign of the cursor picks the ray"
    );
    let d = brush_settings::snap_to_45(a, [10.0, 9.0]); // near-diagonal
    assert!(
        (d[0] - d[1]).abs() < 1e-4,
        "snapped onto the y=x diagonal: {d:?}"
    );
    assert!(d[0] > 0.0);
}

/// Timing: the FULL per-move tool cost of an Anchored size-drag (restore + save + stamp), plain vs
/// textured, on a large canvas. Tells us where the per-move CPU goes. Run:
/// `cargo test -p ph2d-tool-painter --release perf_anchored -- --ignored --nocapture`.
#[test]
#[ignore]
fn perf_anchored_drag_per_move_cost() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    use std::time::Instant;
    // `hold_preview` simulates the shell bridge retaining the preview Arc across frames (it drains
    // `take_preview_arc` each frame and keeps it). With it held, the tool's next mutation hits
    // refcount=2 → `Arc::make_mut` deep-clones the whole 16.8MB canvas EVERY move. That clone is
    // invisible to a bench that doesn't hold the Arc — the bench-vs-live gap.
    let run = |label: &str, kind: TextureKind, mapping: TextureMapping, hold_preview: bool| {
        let mut t = white_canvas(2048, 10.0);
        t.paint.brush.texture = TextureSettings {
            kind,
            mapping,
            ..Default::default()
        };
        t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
        let _ = t.on_canvas_pointer(cp([1024.0, 1024.0], PointerPhase::Down));
        let moves = 20u32;
        let mut held = None;
        let t0 = Instant::now();
        for k in 1..=moves {
            let r = 60.0 + k as f32 * 45.0; // radius grows to ~960 px
            let _ = t.on_canvas_pointer(cp([1024.0, 1024.0 + r], PointerPhase::Move));
            if hold_preview {
                held = t.take_preview_arc(); // retain across the next move (bridge behaviour)
            }
        }
        let _ = held;
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(moves);
        eprintln!("  anchored {label:<22} {ms:6.2} ms/move");
    };
    eprintln!(
        "perf: Anchored size-drag on 2048², radius→960 px, per-move tool cost (preview held):"
    );
    run("plain", TextureKind::None, TextureMapping::ViewPlane, true);
    run(
        "voronoi View (cached)",
        TextureKind::Voronoi,
        TextureMapping::ViewPlane,
        true,
    );
    run(
        "voronoi Tiled (cached)",
        TextureKind::Voronoi,
        TextureMapping::Tiled,
        true,
    );
    run(
        "noise Tiled (cached)",
        TextureKind::Noise,
        TextureMapping::Tiled,
        true,
    );
}

#[test]
fn anchored_textured_stroke_commits_a_textured_result() {
    // Perf fix: the interactive Anchored preview stamps texture-FREE (fast), then re-applies the
    // texture once on pen-up. Assert the COMMITTED result is still textured — a hard Checker dab
    // leaves a mix of painted (black) and masked (white) pixels in its footprint.
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(96, 6.0);
    t.set_brush_texture_kind(TextureKind::Checker.to_u8());
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::ViewPlane,
        size: [0.2, 0.2], // big cells across the anchored footprint
        ..Default::default()
    };
    t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
    // Anchored: press at the centre, drag out (radius = drag distance), release.
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    let _ = t.on_canvas_pointer(cp([48.0, 78.0], PointerPhase::Move)); // radius ≈ 30
    let _ = t.on_canvas_pointer(cp([48.0, 78.0], PointerPhase::Up));
    // Scan the footprint for both fully-painted (black) and masked (white) pixels.
    let (mut black, mut white) = (0, 0);
    for y in 20..76 {
        for x in 20..76 {
            match px(&t, 96, x, y) {
                [0, 0, 0, 255] => black += 1,
                [255, 255, 255, 255] => white += 1,
                _ => {}
            }
        }
    }
    assert!(black > 0, "the committed Anchored dab painted");
    assert!(
        white > 0,
        "the committed Anchored dab is textured (checker masked some texels)"
    );
}

#[test]
fn sample_composite_at_uv_reads_the_painted_colour() {
    // The colour-picker eyedropper samples this (not the transparent Vello overlay). A painted
    // pixel must read its colour; an unpainted pixel reads the canvas background — never transparent.
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.color = [1.0, 0.0, 0.0]; // red
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let centre = t
        .sample_composite_at_uv(0.5, 0.5)
        .expect("composite sample at centre");
    assert_eq!(
        [centre[0], centre[1], centre[2], centre[3]],
        [255, 0, 0, 255],
        "eyedropper reads the painted (opaque) colour, not transparent"
    );
    let corner = t
        .sample_composite_at_uv(0.0, 0.0)
        .expect("composite sample at corner");
    assert_eq!(
        [corner[0], corner[1], corner[2], corner[3]],
        [255, 255, 255, 255],
        "an unpainted pixel reads the opaque white canvas, not transparent"
    );
}

/// `coalesces_canvas_motion` gates per-frame pointer coalescing in the shell. It must be true EXACTLY for
/// the restore + whole-shape re-stamp fill methods (latest-position-only, so coalescing is byte-identical)
/// and false for the incremental / capture methods (Space/Dots/Airbrush/Free Hand) that need every event.
/// Guards the FPS-drop fix (`HANDOFF_per_layer_color_perf_artifacts` §1.R) against a method slipping into
/// the wrong bucket (e.g. coalescing Free Hand would drop captured path points).
#[test]
fn coalesces_canvas_motion_is_true_only_for_restore_based_fill_methods() {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(8, 2.0);
    let cases = [
        (StrokeMethod::Arc, true),
        (StrokeMethod::Ellipse, true),
        (StrokeMethod::Polygon, true),
        (StrokeMethod::Line, true),
        (StrokeMethod::Anchored, true),
        (StrokeMethod::DragDot, true),
        (StrokeMethod::Space, false),
        (StrokeMethod::Dots, false),
        (StrokeMethod::Airbrush, false),
        (StrokeMethod::FreeHand, false),
    ];
    for (method, want) in cases {
        t.paint.brush.stroke_method = method;
        assert_eq!(
            t.coalesces_canvas_motion(),
            want,
            "{method:?} coalesce bucket"
        );
    }
    // Selection mode: gizmo drags / Rectangle / Ellipse / Automatic act on the latest position only →
    // coalesce (each raw Move paid a full boolean recompose — the P4 storm, Enio 2026-07-04). The
    // Freehand lasso (mode 1) CAPTURES the path → every event, regardless of the brush method.
    t.paint.brush.stroke_method = StrokeMethod::Space; // would NOT coalesce as a stroke
    t.set_paint_tool_mode("selection");
    for (mode, want) in [(0u8, true), (1, false), (2, true), (3, true)] {
        t.set_selection_mode(mode);
        assert_eq!(
            t.coalesces_canvas_motion(),
            want,
            "selection mode {mode} coalesce bucket"
        );
    }
}

// ── Rail Shapes ⟷ Stroke:Method wiring (the tool half of the seam) ────────────────────────────────

#[test]
fn stroke_method_channel_sets_shapes_and_the_brush_sentinel_restores_the_last_non_shape() {
    // The tool rail drives the SAME PAINTER_BRUSH_STROKE_METHOD channel as the Method dropdown: a shape's
    // wire u8 selects it; the sentinel "brush" (the rail Brush button) restores the last NON-shape method.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let sm = |t: &PainterTool| t.paint.brush.stroke_method;
    let set = |t: &mut PainterTool, v: &str| {
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            v.to_string(),
        ));
    };
    // Choose a non-shape method (Dots = 0) — becomes the remembered "resting" method.
    set(&mut t, "0");
    assert_eq!(sm(&t), StrokeMethod::Dots);
    // Pick a shape (Ellipse = 7) — the method switches, but the non-shape memory is untouched.
    set(&mut t, "7");
    assert_eq!(sm(&t), StrokeMethod::Ellipse);
    // The Brush button (sentinel "brush") restores the last non-shape method (Dots), NOT the default.
    set(&mut t, "brush");
    assert_eq!(
        sm(&t),
        StrokeMethod::Dots,
        "Brush restored the last non-shape method"
    );
    // Another shape, then Brush again → still Dots (the memory persists across shape excursions).
    set(&mut t, "9"); // FreeHand
    assert_eq!(sm(&t), StrokeMethod::FreeHand);
    set(&mut t, "brush");
    assert_eq!(sm(&t), StrokeMethod::Dots);
}

#[test]
fn brush_sentinel_restores_space_when_no_non_shape_was_chosen() {
    // Fresh tool → a shape → Brush restores the default resting method (Space), never a shape.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        "8".to_string(), // Polygon
    ));
    assert_eq!(t.paint.brush.stroke_method, StrokeMethod::Polygon);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        "brush".to_string(),
    ));
    assert_eq!(t.paint.brush.stroke_method, StrokeMethod::Space);
}

#[test]
fn tools_keep_independent_brush_settings_by_default() {
    // Default model: each paint tool has its OWN BrushSpec; a mode switch swaps slots, so editing one
    // tool never bleeds into another. (`white_canvas` seeds every slot to the fixture 6.0, so the split
    // below is created by the test's own edits, not the fixture.)
    let mut t = white_canvas(32, 6.0);
    assert!(
        !t.link_shared_settings(),
        "independent (unlinked) by default"
    );
    t.paint.brush.radius_px = 20.0; // Brush (Paint) size
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 6.0,
        "Mask uses its own size (fixture 6), not the Brush's 20"
    );
    t.paint.brush.radius_px = 3.0; // edit Mask only
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.paint.brush.radius_px, 20.0,
        "the Brush size survived the Mask detour"
    );
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 3.0,
        "Mask kept its own edited size"
    );
}

#[test]
fn syncing_shares_settings_and_seeds_from_the_checked_panel() {
    let mut t = white_canvas(32, 6.0);
    // Give Brush and Mask independent sizes.
    t.paint.brush.radius_px = 20.0; // Brush
    t.set_paint_tool_mode("mask");
    t.paint.brush.radius_px = 3.0; // Mask
    t.set_paint_tool_mode("brush");
    assert_eq!(t.paint.brush.radius_px, 20.0);
    // Check "Sync with other tools" on the Brush panel → it configures the others.
    t.toggle_link_shared_settings();
    assert!(t.link_shared_settings());
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 20.0,
        "linked: Mask now shows the checked (Brush) panel's size, not its old 3"
    );
    // While linked, editing any tool changes the shared value seen by all.
    t.paint.brush.radius_px = 12.0;
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.paint.brush.radius_px, 12.0,
        "linked: editing Mask also changed the Brush"
    );
    // Uncheck → every tool keeps the current shared value, then diverges.
    t.toggle_link_shared_settings();
    assert!(!t.link_shared_settings());
    t.paint.brush.radius_px = 7.0; // edit Brush only
    t.set_paint_tool_mode("mask");
    assert_eq!(
        t.paint.brush.radius_px, 12.0,
        "unlinked: Mask kept the last shared value (12), not the Brush's new 7"
    );
}

#[test]
fn sync_checkbox_click_routes_to_the_link_toggle() {
    // Guards the panel→tool wiring: a Click on PAINTER_BRUSH_SYNC reaches toggle_link_shared_settings
    // through route_brush_dab_event.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 4.0);
    assert!(!t.link_shared_settings());
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYNC));
    assert!(
        t.link_shared_settings(),
        "the Sync checkbox click toggled the link on"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYNC));
    assert!(!t.link_shared_settings(), "clicking again toggled it off");
}

#[test]
fn rebinding_a_sprite_abandons_a_pending_fill_and_disarms_the_eyedropper() {
    // The Enio 2026-07-02 lifecycle bug: deleting a sprite (Painter active) then selecting another used
    // to carry a pending Fill ColorDrop + an armed Eyedropper onto the new sprite — the Fill flooded it
    // BLACK and the pick swallowed the next Down ("can't paint"). Binding a new document must clear both.
    let mut t = white_canvas(16, 4.0); // black brush, white canvas
    t.set_paint_tool_mode("fill");
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // arm a ColorDrop (fill_begin_drop)
    assert!(
        t.has_active_fill(),
        "a ColorDrop is pending on the old sprite"
    );
    t.paint.eyedropper_armed = true; // also arm the Eyedropper
    // Model delete-then-select-another by binding a fresh mid-grey document.
    <PainterTool as RasterEditTool>::set_source(&mut t, vec![128u8; 16 * 16 * 4], 16, 16);
    assert!(
        !t.has_active_fill(),
        "the stale ColorDrop was abandoned on rebind"
    );
    assert!(
        !t.paint.eyedropper_armed,
        "the Eyedropper was disarmed on rebind"
    );
    // A stray Fill modal slider drag can no longer flood the newly-bound sprite (fill_seed is gone).
    t.set_fill_threshold(0.9);
    assert!(
        t.canvas_rgba.iter().all(|&b| b == 128),
        "the new sprite is intact — not flooded black by the leaked fill"
    );
}

/// The paint COLOUR is one shared foreground colour across every paint mode (Photoshop/Procreate model):
/// a colour set in one mode survives a mode switch, so the C&F ColorDrop (which switches to Fill mode) and
/// switching tools no longer revert it to the previous / default black (Enio 2026-07-04).
#[test]
fn paint_colour_is_shared_across_modes() {
    let mut t = white_canvas(32, 4.0);
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([200, 50, 20]);
    assert_eq!(t.brush_color_srgb8(), [200, 50, 20]);
    // Switching to Fill mode (what the ColorDrop does) must keep the colour, not swap in Fill's black slot.
    t.set_paint_tool_mode("fill");
    assert_eq!(
        t.brush_color_srgb8(),
        [200, 50, 20],
        "the colour survives the switch to Fill mode (ColorDrop)"
    );
    // And through Selection + back to Brush.
    t.set_paint_tool_mode("selection");
    assert_eq!(t.brush_color_srgb8(), [200, 50, 20]);
    t.set_paint_tool_mode("brush");
    assert_eq!(
        t.brush_color_srgb8(),
        [200, 50, 20],
        "colour is shared, not per-mode"
    );
}
