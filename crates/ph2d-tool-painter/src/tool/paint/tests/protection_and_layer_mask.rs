//! **A máscara — a proteção e a da camada.** O pincel de proteção que congela pixels contra o pincel
//! de tinta, o scratch e o Apply que o transforma numa máscara de camada, o Clear/Invert, e a
//! sobreposição que tinge o que está protegido.

use super::*;

#[test]
fn mask_brush_protects_and_keeps_the_layer_fully_visible() {
    // The Mask BRUSH paints a TEMPORARY PROTECTION scratch (Blender Sculpt-mask style). It must NOT create
    // a stack layer, keep the current layer active, leave the layer's pixels untouched (non-destructive),
    // and — critically — NEVER make anything invisible: the layer stays fully opaque; the overlay only
    // TINTS the protected region so you can see it.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    let raster = t.layers.active().expect("a raster is active");
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    assert!(t.is_mask_mode());
    assert_eq!(t.mask_brush(), 0, "default sub-brush is Paint (protect)");
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down)); // protect the centre (into the scratch)
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    // No layer created; still on the raster; a transient scratch is live.
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "the mask brush creates NO layer"
    );
    assert_eq!(t.layers.active(), Some(raster), "still on the raster");
    assert!(t.mask_scratch_active(), "a transient scratch is live");
    // Non-destructive: the layer's own pixels (canvas_rgba) are untouched (still white).
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "the layer pixels are untouched (non-destructive)"
    );
    // NOTHING is invisible: the protected centre stays FULLY OPAQUE (a = 255) — the opposite of a
    // visibility mask. The overlay only tints the RGB so you can see it; an unprotected corner is pristine.
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let c = ((12 * w + 12) * 4) as usize;
    let corner = ((2 * w + 2) * 4) as usize;
    assert_eq!(
        buf[c + 3],
        255,
        "the protected pixel is NOT hidden — still fully opaque"
    );
    assert!(
        buf[c] < 255,
        "the overlay tints the protected region, got {}",
        buf[c]
    );
    assert_eq!(
        [buf[corner], buf[corner + 3]],
        [255, 255],
        "an unprotected corner keeps the pristine image"
    );
}

#[test]
fn a_mask_stroke_takes_the_partial_lane_byte_identical_to_a_full_recompose() {
    // While a Mask protection scratch is live, `take_preview_arc` used to `force_full` EVERY painted
    // frame — a whole-canvas recompose + full 16 MiB upload for a dab-sized change (measured 17 ms
    // preview + 6.9 ms upload @ 2048², CPU: the mask FPS drop Enio reported). But the overlay tint is
    // PER-PIXEL and a mask dab changes coverage only inside its dirty rect, so the partial fast lane
    // re-tints just that region (`apply_mask_overlay_region`) and is byte-identical to a full recompose.
    //
    // Mutations that must bleed: (a) re-adding `force_full = mask_scratch_active()` → the 2nd dab takes
    // the FULL arm, not partial (the branch assert); (b) dropping `apply_mask_overlay_region` from the
    // partial arm → the 2nd dab's region loses its tint and `partial != full`.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 64u32;
    let mut t = white_canvas(size, 6.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    assert!(t.is_mask_mode());
    // First dab: creating the scratch invalidates the composite, so this drain SEEDS the cache with
    // the overlay via the full arm (composite is `None`).
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    let _seed = t.take_preview_arc().expect("seed drain");
    assert_eq!(
        t.preview_drain_diag().0,
        crate::tool::DrainBranch::FullComposite,
        "the first mask dab seeds the cache via a full recompose",
    );
    // Second dab elsewhere: the cache is seeded now, so this MUST take the partial fast lane.
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    let (partial, _, _) = t.take_preview_arc().expect("partial drain");
    assert_eq!(
        t.preview_drain_diag().0,
        crate::tool::DrainBranch::PartialComposite,
        "a mask dab over a seeded cache must take the partial fast lane, not force full",
    );
    // The COMPOSITE is partial (fast), but the UPLOAD is forced FULL for the translucent mask overlay:
    // the shell's partial GPU upload leaves visible seams for it on the real device (Enio smoke
    // 2026-07-24), so the mask drain reports NO upload bbox → the shell uploads the whole texture.
    // (Mutation: `Some(bbox)` unconditionally → this is Some → the seam-prone partial upload returns.)
    assert!(
        t.take_preview_upload_bbox().is_none(),
        "a live mask scratch forces a FULL upload (no partial-upload seam), even on the partial composite",
    );
    let partial = (*partial).clone();
    // Force a full recompose of the EXACT same scratch+layer state and compare byte-for-byte.
    t.invalidate_composite();
    let (full, _, _) = t.take_preview_arc().expect("full drain");
    assert_eq!(
        t.preview_drain_diag().0,
        crate::tool::DrainBranch::FullComposite,
        "invalidate → the next drain is a full recompose",
    );
    assert_eq!(
        *full, partial,
        "the partial-lane mask preview must equal a full recompose to the byte",
    );
}

#[test]
fn mask_stroke_undoes_and_redoes_with_the_global_timeline() {
    // A mask stroke mutates only the transient scratch (the layer's own pixels stay put), so the undo
    // model must capture that scratch — else the stroke produces a no-op undo entry and can't be rolled
    // back. The reported bug. Paint a mask dab, then undo/redo and check the scratch flips
    // concealed↔cleared in lock-step with the global painter timeline.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 24u32;
    let mut t = white_canvas(size, 6.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    let center = ((12 * size + 12) * 4) as usize; // R channel of the scratch centre
    // Paint a mask dab → the scratch is created + the centre concealed (R drops from white).
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let painted = t.paint.mask_scratch_rgba[center];
    assert!(
        painted < 200,
        "the mask dab concealed the centre (R={painted})"
    );
    // Undo → the scratch rolls back to white (the stroke IS undoable with the global timeline).
    assert!(t.undo_last(), "the mask stroke is an undo step");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], 255,
        "undo cleared the scratch back to white"
    );
    // Redo → the conceal comes back, identically.
    assert!(t.redo_last(), "redo re-applies the mask stroke");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], painted,
        "redo restored the concealed scratch"
    );
}

#[test]
fn mask_canvas_op_is_undoable() {
    // A whole-canvas mask Modifier (Clear / Invert / …) mutates the scratch and must be undoable too.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let size = 24u32;
    let mut t = white_canvas(size, 6.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    let center = ((12 * size + 12) * 4) as usize;
    // A mask dab conceals the centre.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let concealed = t.paint.mask_scratch_rgba[center];
    assert!(concealed < 200, "the dab concealed the centre");
    // Clear (op 5) whitens the whole scratch.
    t.mask_canvas_op(5);
    assert_eq!(
        t.paint.mask_scratch_rgba[center], 255,
        "Clear whitened the scratch"
    );
    // Undo rolls the Clear back to the concealed dab (the canvas op is its own undo step).
    assert!(t.undo_last(), "the canvas op is an undo step");
    assert_eq!(
        t.paint.mask_scratch_rgba[center], concealed,
        "undo restored the concealed dab"
    );
}

#[test]
fn mask_brush_freezes_pixels_against_the_paint_brush() {
    // The CORE of the protection mask: a scratch-protected region is FROZEN — the paint Brush (and every
    // other paint tool) cannot alter it. Protect the centre, switch to the Brush, then paint over both the
    // protected centre and an unprotected corner: the centre keeps its pixel, the corner paints normally.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(32, 4.0); // black brush on white
    // Protect the centre.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Switch to the normal Brush (protection persists) and stroke the protected centre — it must not move.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 16, 16),
        [255, 255, 255, 255],
        "the protected centre is FROZEN — the brush could not paint it"
    );
    // An unprotected corner paints normally (black).
    t.on_canvas_pointer(cp([28.0, 28.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([28.0, 28.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 32, 28, 28),
        [0, 0, 0, 255],
        "an unprotected pixel paints normally"
    );
}

#[test]
fn clear_then_repaint_makes_a_fresh_protection() {
    // Bug: after Clear the app could no longer create new temporary masks (the mask leaked the user's
    // brush colour, so a light colour painted an invisible/weak mask). Clear must leave the scratch able
    // to take a NEW full-strength protection regardless of the brush colour.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 5.0);
    t.paint.brush.color = [0.9, 0.9, 0.9]; // a light brush colour must not weaken the fresh mask
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Clear the mask.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5]));
    // Paint a NEW protection.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        t.mask_scratch_active(),
        "a scratch is live after Clear + repaint"
    );
    // The fresh protection must freeze the brush at the centre.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "the freshly-painted protection freezes the brush after Clear"
    );
}

// ── Audit: layer-system mask via Apply (Photoshop-style visibility mask) ──────────────────────────

#[test]
fn apply_mask_is_one_undo_step_and_redoable() {
    // Apply is a single structural undo step: undo removes the created Mask layer + restores the parent's
    // full visibility (snapshot_model captures both layers AND images); redo re-creates it with its pixels.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().unwrap();
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert!(t.layers.get(target).and_then(|l| l.mask).is_some());
    assert_eq!(t.layers.all_ids().count(), n_before + 1);
    // Undo → the mask layer is gone and the parent is unmasked again.
    assert!(t.can_undo());
    assert!(t.undo_last());
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_none(),
        "undo removed the layer mask"
    );
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "the Mask layer is gone after undo"
    );
    // Redo → the mask is back and conceals the centre again.
    assert!(t.redo_last());
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_some(),
        "redo restored the layer mask"
    );
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "redo restored the mask concealment, got a = {}",
        buf[i + 3]
    );
}

#[test]
fn apply_copies_the_scratch_into_the_mask_faithfully() {
    // Apply must copy the scratch coverage 1:1 into the layer mask — the mask pixel at a point equals the
    // scratch coverage there (no re-threshold / re-colour). Verified at a protected centre + a clear corner.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().unwrap();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    let idx_c = (8 * 16 + 8) as usize;
    let idx_corner = (16 + 1) as usize; // (1,1)
    let sc_c = crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx_c);
    let sc_corner = crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx_corner);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    let mask = t.layers.get(target).and_then(|l| l.mask).unwrap();
    let img = t.images.get(&mask).expect("the mask pixels live in images");
    assert!(
        (crate::compositor::mask_value(&img.rgba8, idx_c) - sc_c).abs() < 0.004,
        "mask centre coverage matches the scratch (faithful copy)"
    );
    assert!(
        (crate::compositor::mask_value(&img.rgba8, idx_corner) - sc_corner).abs() < 0.004,
        "mask corner coverage matches the scratch (faithful copy)"
    );
}

#[test]
fn apply_twice_merges_into_the_existing_mask() {
    // The merge branch: once a layer has a mask, painting a NEW protection + Apply again multiplies the
    // scratch INTO the existing mask (NO second Mask layer; the same mask id refines its coverage).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 5.0);
    let target = t.layers.active().unwrap();
    let n0 = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    // First Apply: protect + apply at spot A.
    t.on_canvas_pointer(cp([6.0, 6.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([6.0, 6.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    let mask = t.layers.get(target).and_then(|l| l.mask).unwrap();
    assert_eq!(t.layers.all_ids().count(), n0 + 1);
    // Second protection at spot B + Apply again → merge in place (no new layer, same mask id).
    t.on_canvas_pointer(cp([18.0, 18.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([18.0, 18.0], PointerPhase::Up));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert_eq!(
        t.layers.all_ids().count(),
        n0 + 1,
        "merge: no second Mask layer created"
    );
    assert_eq!(
        t.layers.get(target).and_then(|l| l.mask),
        Some(mask),
        "the same mask id refined in place"
    );
    // Both spots A and B are hidden (black) in the merged mask.
    let img = t.images.get(&mask).unwrap();
    assert!(
        crate::compositor::mask_value(&img.rgba8, (6 * 24 + 6) as usize) < 0.5,
        "spot A stays hidden after the merge"
    );
    assert!(
        crate::compositor::mask_value(&img.rgba8, (18 * 24 + 18) as usize) < 0.5,
        "spot B is hidden after the merge"
    );
}

#[test]
fn erase_mask_sub_brush_removes_protection() {
    // Bug: the Erase sub-brush was broken (the stamp reads each dab's OWN colour, baked from the user's
    // brush colour, so the white override was a no-op → Erase painted the wrong coverage). Erase (white)
    // over a protected area must UNPROTECT it, so the paint brush can then modify those pixels again.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    let idx = (12 * 24 + 12) as usize;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    // Paint (protect) the centre → scratch fully black there.
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) < 0.01,
        "Paint protected the centre (scratch black)"
    );
    // Erase sub-brush over the same spot → scratch back to white (unprotected).
    t.set_mask_brush(1);
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) > 0.99,
        "Erase unprotected the centre (scratch white again)"
    );
    // End-to-end: the centre is now unprotected → the paint brush CAN modify it.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [0, 0, 0, 255],
        "Erase removed the protection → the brush paints the centre again"
    );
}

#[test]
fn mask_paint_ignores_the_brush_colour() {
    // Root cause of the "mask is much lighter than normal" + Clear/Erase bugs: the mask must paint a PURE
    // coverage (black = protect), ignoring the user's brush colour. A light/coloured brush used to leak
    // its luma into the scratch → partial protection → a faint overlay. Paint with a light colour and a
    // Screen blend and assert FULL protection (scratch black, brush frozen).
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(24, 6.0);
    t.paint.brush.color = [1.0, 0.85, 0.2]; // a light yellow — must NOT weaken the mask
    t.paint.brush.blend = ph2d_painter_brush::BrushBlend::Screen; // a non-Mix blend must not break it
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    let idx = (12 * 24 + 12) as usize;
    assert!(
        crate::compositor::mask_value(&t.paint.mask_scratch_rgba[..], idx) < 0.01,
        "the mask painted FULL black protection regardless of the light brush colour + blend"
    );
    // And it freezes the brush at the centre (full protection, not partial).
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([12.0, 12.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 24, 12, 12),
        [255, 255, 255, 255],
        "full protection freezes the brush"
    );
}

#[test]
fn mask_apply_creates_a_layer_mask_from_the_scratch() {
    // Apply promotes the transient scratch to a REAL layer-system mask attached to the current layer:
    // a Mask layer appears (count up), `target.mask` points at it, the scratch clears, the parent's OWN
    // pixels are UNTOUCHED (non-destructive — the mask lives in the stack, not baked into the alpha), and
    // the composite still conceals through the new mask.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    let target = t.layers.active().expect("a raster is active");
    let n_before = t.layers.all_ids().count();
    assert!(
        t.layers.get(target).and_then(|l| l.mask).is_none(),
        "the raster starts with no layer mask"
    );
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // conceal centre (scratch only)
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Apply → a real layer mask is created from the scratch.
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_APPLY));
    assert!(!t.mask_scratch_active(), "Apply cleared the scratch");
    assert_eq!(
        t.layers.all_ids().count(),
        n_before + 1,
        "Apply added exactly one Mask layer"
    );
    let mask = t
        .layers
        .get(target)
        .and_then(|l| l.mask)
        .expect("the target now owns a layer mask");
    assert!(
        matches!(
            t.layers.get(mask).map(|l| &l.kind),
            Some(LayerKind::Mask(_))
        ),
        "the attached layer is a Mask"
    );
    assert_eq!(
        t.layers.active(),
        Some(target),
        "the parent raster stays the active edit layer (not the mask)"
    );
    // Non-destructive: the parent's OWN pixels are untouched (still opaque white — the mask is separate).
    assert_eq!(
        px(&t, 16, 8, 8),
        [255, 255, 255, 255],
        "Apply did NOT bake into the layer alpha (the mask is a separate stack layer)"
    );
    // The composite still conceals the masked centre through the new layer mask (scratch cleared → no
    // overlay film now, so the alpha drops to ~0).
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "the layer mask conceals the centre, got a = {}",
        buf[i + 3]
    );
}

#[test]
fn mask_scratch_persists_across_a_tool_switch() {
    // The scratch is PERSISTENT (correção #1): switching the rail tool does NOT discard it. After painting
    // the scratch and switching to the Brush, it stays live (its target layer is still active) and keeps
    // PROTECTING the region — so you can paint freely around the frozen area with the Brush. (Switching
    // LAYERS is the only thing that makes it go dormant.)
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(16, 5.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down)); // protect centre (scratch only)
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(t.mask_scratch_active());
    // Switch to the Brush — the scratch must NOT be discarded.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "brush".to_string(),
    ));
    assert!(
        t.mask_scratch_active(),
        "switching tools keeps the scratch alive (its target layer is still active)"
    );
    // The composite still TINTS the protected centre while an unprotected corner keeps the pristine image.
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let c = ((8 * w + 8) * 4) as usize;
    let corner = ((w + 1) * 4) as usize;
    assert!(
        buf[c] < 128,
        "the scratch still marks the protected centre after the tool switch, got {}",
        buf[c]
    );
    assert_eq!(buf[corner], 255, "the unprotected corner keeps the image");
}

#[test]
fn mask_canvas_op_clear_then_invert() {
    // The whole-canvas Modifiers edit the transient SCRATCH (no layer). Clear → nothing protected (no
    // overlay tint → pristine layer); Invert → everything protected (fully tinted). Verified via composite.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = white_canvas(16, 4.0);
    let n_before = t.layers.all_ids().count();
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5])); // Clear → scratch white
    assert_eq!(
        t.layers.all_ids().count(),
        n_before,
        "canvas ops create NO layer"
    );
    assert!(
        t.mask_scratch_active(),
        "a scratch is live after a Modifier"
    );
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]],
        [255, 255, 255, 255],
        "Clear → nothing protected → pristine (no overlay tint)"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[4])); // Invert → scratch black
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i] < 128,
        "Invert → all protected, composite tinted dark, got {}",
        buf[i]
    );
}

#[test]
fn layer_mask_paintable_by_brush_and_grayscale_view_eye() {
    // A LAYER-SYSTEM mask (Layers "Mask" button) is paintable by the NORMAL brush (any tool), and its
    // grayscale-view eye toggles the canvas between the masked effect (closed) and the mask channel (open).
    let mut t = white_canvas(16, 4.0);
    let mask = t.add_mask_to_active().expect("layer mask created + active");
    // Normal Paint stroke (black default) on the active mask → conceal centre.
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert!(
        px(&t, 16, 8, 8)[0] < 128,
        "the brush painted the layer mask"
    );
    // Eye closed (default): composite shows the EFFECT — concealed centre hidden (low alpha).
    assert_eq!(t.mask_view_grayscale(), None);
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i + 3] < 128,
        "effect view hides the concealed centre, got a = {}",
        buf[i + 3]
    );
    // Eye open: composite shows the mask GRAYSCALE — concealed centre opaque black.
    t.toggle_mask_view_grayscale(mask);
    assert_eq!(t.mask_view_grayscale(), Some(mask.0));
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 3]],
        [0, 255],
        "grayscale view shows the mask channel (opaque black centre)"
    );
}

#[test]
fn mask_overlay_tints_the_protected_composite() {
    // The overlay is a quick-mask film over the PROTECTED region: an all-unprotected (white) mask shows
    // nothing, so Clear→Invert (all protected / black) + the fluorescent-yellow overlay pulls the
    // composite's blue down (yellow = low blue), proving the film renders on the frozen area.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = white_canvas(16, 4.0);
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_COLOR[1])); // fluorescent yellow
    assert_eq!(t.mask_overlay_color(), 1);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[5])); // Clear → white (unprotected)
    // An all-unprotected mask must NOT tint (no flood).
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    assert_eq!(
        [buf[i], buf[i + 1], buf[i + 2]],
        [255, 255, 255],
        "an all-unprotected mask shows NO overlay flood"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_MASK_OP[4])); // Invert → black (protected)
    let (buf, w, _h) = t.take_preview_arc().expect("a composite preview");
    let i = ((8 * w + 8) * 4) as usize;
    let (r, g, b) = (buf[i], buf[i + 1], buf[i + 2]);
    assert!(
        b < r && b < g,
        "yellow overlay tints the protected area, pulling blue below red/green: ({r}, {g}, {b})"
    );
}

// ── Layer mask painting (bug: a selected mask couldn't be painted — the event fell through to
//    the move tool and dragged the sprite instead) ────────────────────────────────────────────

#[test]
fn a_selected_mask_is_paintable_e2e() {
    let mut t = white_canvas(64, 6.0);
    // Add a mask to the active raster layer; it becomes active with a white (fully-visible) buffer.
    let _mask = t
        .add_mask_to_active()
        .expect("mask added to the active raster layer");
    assert!(t.active_is_mask(), "the new mask is the active layer");
    // The bug: `paint_target_ready` rejected masks, so `on_canvas_pointer` returned `false` and the
    // event fell through to the move tool (dragging the sprite). It must now CONSUME the event...
    let consumed = t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert!(
        consumed,
        "painting a selected mask must consume the canvas event (not fall through to move/drag)"
    );
    // ...and paint the mask's coverage: the default black brush conceals (luma → 0) at the centre.
    assert_eq!(
        px(&t, 64, 32, 32),
        [0, 0, 0, 255],
        "the mask was painted (black = conceal)"
    );
    // An unpainted corner stays white (fully visible).
    assert_eq!(
        px(&t, 64, 0, 0),
        [255, 255, 255, 255],
        "unpainted mask area stays white"
    );
}
