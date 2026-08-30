//! **A cor por-camada do Shape (Per-Layer Color).** Cada camada da silhueta pinta a própria cor: a
//! ordem de empilhamento ao longo do traço, o blend, o randomize por dab, a virada de rota no meio do
//! traço, a assinatura de aparência que serve de chave do cache — e o arnês de perf que mede o custo
//! por Move deste caminho (`per_layer_perf`).

use super::*;

#[test]
fn per_layer_color_top_layer_paints_above_all_lower_painting_across_the_stroke() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // 2-layer Shape: layer 0 (bottom) = a full square, layer 1 (top) = its RIGHT half only. Colours red
    // bottom, green top. Two overlapping dabs (B to the right of A) emitted in SEPARATE `stamp_dabs`
    // calls — exactly how a real freehand stroke arrives (one batch per pointer move). At a pixel inside
    // A's right-half (green) that B's left-half (red bottom, no green) re-covers, a direct per-dab
    // composite lets B's later red bury A's green (only the tip's highlight survives, worse the slower
    // the stroke). The per-stroke accumulate + recomposite keeps it GREEN across batches (Enio 2026-06-26).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space; // incremental → accumulate across batches
    let full = vec![255u8; 64]; // 8×8 full coverage (the body)
    let mut right = vec![0u8; 64]; // 8×8, right half = 255 (the highlight)
    for row in 0..8 {
        for col in 4..8 {
            right[row * 8 + col] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(full, 8, 8), (right, 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom = red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top = green
    let dab = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    };
    t.stamp_dabs(&[dab(20.0)]); // batch 1 (dab A)
    t.stamp_dabs(&[dab(26.0)]); // batch 2 (dab B overlapping A's right half from the left)
    let [r, g, b, _] = px(&t, 64, 22, 32); // inside A's right-half-green, re-covered by B's left-half
    assert!(
        g > 200 && r < 80,
        "the top (green) layer survives the lower (red) layer across batches: {:?}",
        [r, g, b]
    );
}

#[test]
fn per_layer_color_respects_brush_blend_mode() {
    use ph2d_painter_brush::{BrushBlend, Dab, StrokeMethod};
    // The per-layer-colour tip must blend onto the canvas via the **Brush blend mode** (applied to the
    // whole composite, once). On a 50% grey canvas a solid RED tip with Multiply yields ~half-red
    // (grey×red), NOT the pure red that Normal gives — the old per-layer `blend_over` mis-applied it.
    let mut t = white_canvas(64, 6.0);
    t.set_source(vec![128u8; 64 * 64 * 4], 64, 64); // 50% grey
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.blend = BrushBlend::Multiply;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [1.0, 0.0, 0.0]); // solid red composite
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    };
    t.stamp_dabs(&[dab]);
    let [r, g, b, _] = px(&t, 64, 32, 32);
    assert!(
        (100..=150).contains(&r) && g < 30 && b < 30,
        "Multiply grey×red is ~half-red, not pure red: {:?}",
        [r, g, b]
    );
}

#[test]
fn per_layer_color_dynamic_randomize_color_tints_per_dab() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Randomize Color on → the DYNAMIC per-layer path, which tints by each dab's own `d.color`. Two
    // non-overlapping dabs carrying red / blue paint red / blue (the static cached path baked one colour
    // for the whole stroke and would ignore `d.color`).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color_jitter_hue = 0.5; // Randomize Color active (amount > 0) → routes to the dynamic path
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color(); // layers un-coloured → they take the per-dab base colour
    let dab = |cx: f32, col: [f32; 3]| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: col,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    };
    t.stamp_dabs(&[dab(16.0, [1.0, 0.0, 0.0])]); // red dab
    t.stamp_dabs(&[dab(48.0, [0.0, 0.0, 1.0])]); // blue dab
    let red = px(&t, 64, 16, 32);
    let blue = px(&t, 64, 48, 32);
    assert!(red[0] > 200 && red[2] < 80, "first dab is red: {red:?}");
    assert!(
        blue[2] > 200 && blue[0] < 80,
        "second dab is blue: {blue:?}"
    );
}

#[test]
fn per_layer_color_fill_method_uses_canvas_base_and_self_clears() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Fill methods (Line/Curve/Ellipse/Polygon) take the no-snapshot / self-clearing per-layer path: the
    // canvas is the recomposite base (the drag preview restores it to the pre-shape each move) and the
    // maps self-clear, so there's no per-move full-canvas clone + N-map re-allocation (the FPS fix). Two
    // full layers (red bottom, green top) → green on top; re-stamping the identical fill onto the same
    // restored canvas must be STABLE — proving the maps self-cleared and the canvas-base didn't double-
    // composite (a stale-map or double-composite bug would change the second result).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Line; // a fill method → the non-incremental path
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top green
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    };
    let pristine = (*t.canvas_rgba).clone(); // the pre-shape the drag preview restores to
    t.stamp_dabs(&[dab]); // first fill
    let a = px(&t, 64, 32, 32);
    *std::sync::Arc::make_mut(&mut t.canvas_rgba) = pristine; // emulate the drag-preview restore
    t.stamp_dabs(&[dab]); // re-fill the identical shape onto the restored canvas
    let b = px(&t, 64, 32, 32);
    assert!(
        a[1] > 200 && a[0] < 80,
        "the top (green) layer wins on the fill: {a:?}"
    );
    assert_eq!(
        a, b,
        "re-filling the restored canvas is stable (maps self-clear, canvas as base)"
    );
}

/// Build the reported context: a live Per-Layer Colour freehand stroke on the CACHED route (two layers
/// with custom colours ⇒ 1 B/px coverage maps, no per-dab dynamics yet).
#[cfg(test)]
fn per_layer_live_stroke() -> PainterTool {
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space; // incremental freehand
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // bottom red
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]); // top green
    t
}

#[cfg(test)]
fn live_dab(x: f32) -> ph2d_painter_brush::Dab {
    ph2d_painter_brush::Dab {
        center: [x, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    }
}

#[test]
fn per_layer_color_route_flip_mid_stroke_reshapes_the_maps() {
    // Enio's PANIC (2026-07-12), painting a live freehand stroke in Per-Layer Colour and pressing Shape
    // **Rake**: `range end index 3911680 out of range for slice of length 1048576`
    // (`accumulate_batch.rs`) → SIGSEGV. Rake flips the route from the cached path (maps = 1 B/px coverage)
    // to the per-dab dynamic path (maps = 4 B/px premul RGBA), and the reuse guard only asked
    // "initialised?" (`pre.is_empty()`) / "layer count changed?" (`cov.len() != n`) — never the ELEMENT
    // SIZE. The dynamic route then sliced the previous route's `w*h` maps as if they were `w*h*4`: an
    // out-of-bounds slice, not a wrong pixel. (3911680 = row 955 × stride 4096; 1048576 = 1024².)
    // RED without the fix: the second `stamp_dabs` PANICS here.
    let mut t = per_layer_live_stroke();
    t.stamp_dabs(&[live_dab(24.0)]); // batch 1 — the cached route allocates the 1 B/px maps
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64,
        "the cached route's maps are 1 B/px coverage"
    );
    t.paint.brush.shape.rake = true; // the user presses Rake with the stroke STILL LIVE → the route flips
    t.stamp_dabs(&[live_dab(40.0)]); // batch 2 — the dynamic route: this is where it blew up
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64 * 4,
        "the flipped-to dynamic route re-shaped the maps to 4 B/px premul RGBA"
    );
    assert!(
        px(&t, 64, 40, 32)[3] > 0,
        "the dab painted after the mid-stroke route flip"
    );
}

#[test]
fn flipping_per_layer_color_mid_stroke_keeps_what_was_already_painted() {
    // Sweep (2026-07-12). Same seam as Bug #12 — the panel is live while the canvas is — but a different
    // failure: not the maps' SHAPE, the stroke's CONTINUITY. `pre` is the PRE-stroke canvas snapshot; a dab
    // painted while Per-Layer Color was OFF went straight to `canvas_rgba`, so it is in neither `pre` nor
    // the coverage maps. Turning the mode back ON, the next batch recomposites its bbox from `pre` — and
    // the off-interval dab EVAPORATES.
    // RED without the fix: the pixel painted with the mode off is white again.
    use ph2d_painter_brush::{Dab, StrokeMethod};
    let mut t = white_canvas(64, 8.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.0, 0.0, 1.0]; // blue — the plain-route dab
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32, coverage: f32| Dab {
        center: [cx, 32.0],
        radius_px: 8.0,
        coverage,
        color: [0.0, 0.0, 1.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 8.0,
    };
    t.stamp_dabs(&[dab(16.0, 1.0)]); // per-layer ON — this seeds `pre` (the canvas BEFORE it: all white)
    t.toggle_brush_shape_per_layer_color(); // OFF, stroke still live
    t.stamp_dabs(&[dab(32.0, 1.0)]); // opaque BLUE, straight to the canvas — in neither `pre` nor the maps
    assert!(
        px(&t, 64, 32, 32)[2] > 200 && px(&t, 64, 32, 32)[0] < 60,
        "the mode-off dab painted blue"
    );
    t.toggle_brush_shape_per_layer_color(); // back ON
    // A SEMI-TRANSPARENT dab over the same spot. Its recomposite rebuilds the region as `pre ⊕ layers` —
    // and where the layers are only 30% opaque, 70% of what shows through is the BASE. If the base is the
    // stale pre-stroke snapshot, that 70% is WHITE and the blue dab is gone. (An opaque dab would hide the
    // bug: it overwrites what is under it anyway, which is just normal painting.)
    t.stamp_dabs(&[dab(32.0, 0.3)]);
    let p = px(&t, 64, 32, 32);
    assert!(
        p[0] < 60,
        "the dab painted with the mode OFF must survive the flip back ON — its blue must still be the \
         base under the translucent dab, not rebuilt away from a stale `pre` (red channel {}, so the base \
         went back to WHITE)",
        p[0]
    );
}

#[test]
fn appearance_signature_tracks_tiling_and_ramp_alpha() {
    // Sweep (2026-07-12): `AppearanceSig` is the change detector that re-fills an OPEN shape editor's
    // preview (`on_panel_event` compares it and calls `refill_open_shape`). It missed the two ramps'
    // **Alpha Mode** and **Tiling** — both because their setters deliberately say "no re-bake needed, it
    // only affects future stamps". True of the LUT and of the canvas; NOT true of an open shape, whose
    // preview IS a stamp that has not landed yet. Toggling Tiling (or Alpha Mode) with a Curve on screen
    // changed nothing until some unrelated knob moved.
    //
    // The gate is on the SIGNATURE, which is precisely the defect: if it does not move, the refill is never
    // even called. (Driving the repaint end-to-end needs an open-shape state this harness does not
    // reproduce, and a test whose behaviour I cannot explain is worth less than one that pins the cause.)
    // RED without the fix: the signature is unchanged and the preview stays stale.
    use ph2d_painter_brush::RampAlphaMode;
    let mut t = white_canvas(64, 8.0);
    let sig = t.appearance_sig();
    t.toggle_brush_tiling(0);
    assert!(
        sig != t.appearance_sig(),
        "Tiling wraps an open shape's stamp — it MUST be in the appearance signature"
    );
    let sig = t.appearance_sig();
    t.set_texture_ramp_alpha_mode(RampAlphaMode::Strength.to_u8());
    assert!(
        sig != t.appearance_sig(),
        "the Grain ramp's Alpha Mode is applied at STAMP time — it MUST be in the appearance signature"
    );
    let sig = t.appearance_sig();
    t.set_shape_ramp_alpha_mode(RampAlphaMode::Strength.to_u8());
    assert!(
        sig != t.appearance_sig(),
        "the Shape ramp's Alpha Mode is applied at STAMP time — it MUST be in the appearance signature"
    );
}

#[test]
fn per_layer_color_grain_random_offset_takes_the_per_dab_route() {
    // Sweep (2026-07-12): the per-layer route hand-rolled its "can the constant coloured stamp express this
    // Grain?" test as `Rake || Random-Angle || canvas-fixed`. Grain **Mapping = Random Offset** randomises
    // the per-dab OFFSET, not the angle — it matched none of those clauses, so the CONSTANT stamp was baked
    // once and blitted for every dab: the texture "sticks" to the dab instead of jittering. The canonical
    // predicate is `!texture.is_cacheable()`, which the grayscale routes had always used and which covers
    // Rake, Random-Angle, Random-Offset, Tiled and Stencil in one place.
    //
    // ORACLE = the ROUTE, read from real state, not a re-derived predicate: the per-layer maps are 4 B/px
    // premul RGBA on the per-dab dynamic route and 1 B/px coverage on the constant cached one (the very
    // asymmetry behind Bug #12). Asserting the canvas instead would be nicer, but a full-opacity two-layer
    // silhouette swamps the grain's contribution in this fixture — and a test whose green I cannot explain
    // is worse than one that pins exactly the defect.
    // RED without the fix: the maps come back 1 B/px — the constant stamp served a per-dab Grain.
    use ph2d_painter_brush::{Dab, StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 9.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Random, // randomises the OFFSET per dab — no angle involved
        size: [6.0, 6.0],
        ..t.paint.brush.texture
    };
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 9.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 9.0,
    };
    t.stamp_dabs(&[dab(18.0), dab(46.0)]);
    assert_eq!(
        t.paint
            .per_layer_stroke
            .cov
            .first()
            .map_or(0, std::vec::Vec::len),
        64 * 64 * 4,
        "Random Offset must route to the per-dab dynamic path (4 B/px maps), not the constant cached stamp"
    );
}

#[test]
fn per_layer_color_grain_rake_flip_mid_stroke_reshapes_the_maps() {
    // Enio (2026-07-12): "temos outro rake em grain e paper". The **Grain** Rake reaches the SAME route
    // predicate as the Shape Rake — `grain_has_per_dab_rotation()` is one of `per_dab_dynamic`'s disjuncts
    // (`stamp_route.rs`) — so it flips a live Per-Layer Colour stroke from the cached (1 B/px) route to the
    // dynamic (4 B/px) one exactly like Shape Rake did, and panicked the same way. The shape guard is
    // route-agnostic, so it covers this too; this test PINS that, because "it's the same code path" is a
    // claim, and a claim is not a gate.
    let mut t = per_layer_live_stroke();
    t.paint.brush.texture = ph2d_painter_brush::TextureSettings {
        kind: ph2d_painter_brush::TextureKind::Noise, // an active Grain, so Rake is meaningful
        ..t.paint.brush.texture
    };
    t.stamp_dabs(&[live_dab(24.0)]); // batch 1 — cached route (1 B/px maps)
    let before = t.paint.per_layer_stroke.cov[0].len();
    t.paint.brush.texture.rake = true; // GRAIN Rake, stroke still live → the route flips
    t.stamp_dabs(&[live_dab(40.0)]); // batch 2 — panicked here before the fix
    assert_eq!(
        before,
        64 * 64,
        "the cached route's maps start at 1 B/px coverage"
    );
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64 * 4,
        "Grain Rake re-shaped the maps to the dynamic route's 4 B/px premul RGBA"
    );
}

#[test]
fn per_layer_color_route_flip_back_reshapes_the_maps_too() {
    // The REVERSE flip (Rake turned back off: dynamic → cached) never panicked — the 4 B/px maps are big
    // enough to index at 1 B/px — it CORRUPTED in silence: the cached recomposite read the leftover
    // premul-RGBA bytes as coverage. Same root cause (the guard ignored the element size), so the same
    // guard has to catch this direction too, or the fix would only have moved the bug.
    let mut t = per_layer_live_stroke();
    t.paint.brush.shape.rake = true; // start on the dynamic route (4 B/px maps)
    t.stamp_dabs(&[live_dab(24.0)]);
    assert_eq!(t.paint.per_layer_stroke.cov[0].len(), 64 * 64 * 4);
    t.paint.brush.shape.rake = false; // Rake back off, stroke still live → back to the cached route
    t.stamp_dabs(&[live_dab(40.0)]);
    assert_eq!(
        t.paint.per_layer_stroke.cov[0].len(),
        64 * 64,
        "the flipped-back cached route re-shaped the maps to 1 B/px coverage"
    );
    let p = px(&t, 64, 40, 32);
    assert!(
        p[1] > 200 && p[0] < 80,
        "the post-flip dab paints the TOP layer's green — not RGBA bytes read as coverage: {p:?}"
    );
}

#[test]
fn line_per_layer_color_moving_endpoint_leaves_no_trail() {
    use ph2d_painter_brush::StrokeMethod;
    // Draw a Line (press at A, drag the endpoint around) with Per-Layer Color on. Each move re-stamps the
    // whole line via the drag-preview restore; an earlier endpoint position must be fully restored — no
    // thin trail survives along where the line used to be (Enio 2026-06-27). Drives the real pointer path.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.stroke_method = StrokeMethod::Line;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8)]); // full square silhouette → coverage to the edge
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    let a = [10.0, 31.0];
    // Polyline model: click corner A, then press corner B and sweep it around (the line pivots on A),
    // settling near A. Each move re-stamps the whole line via the drag-preview restore.
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    t.on_canvas_pointer(cp(a, PointerPhase::Up)); // corner A
    t.on_canvas_pointer(cp([52.0, 12.0], PointerPhase::Down)); // create corner B
    for b in [[52.0, 50.0], [52.0, 31.0], [16.0, 31.0]] {
        t.on_canvas_pointer(cp(b, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([16.0, 31.0], PointerPhase::Up));
    // The final line is the short A=(10,31)→(16,31) segment (y≈31, x≈7..19). Any painted pixel well away
    // from it (e.g. y<24 or y>38, or x>26) is a trail from an earlier endpoint the move failed to restore.
    let mut trail = Vec::new();
    for y in 0..64u32 {
        for x in 0..64u32 {
            let far = !(24..=38).contains(&y) || x > 26;
            if far && px(&t, 64, x, y) != [255, 255, 255, 255] {
                trail.push((x, y));
            }
        }
    }
    assert!(
        trail.is_empty(),
        "moving the Line endpoint left a trail at {} pixels, e.g. {:?}",
        trail.len(),
        &trail[..trail.len().min(8)]
    );
}

#[test]
fn per_layer_color_randomize_jitters_custom_layer_colours() {
    use ph2d_painter_brush::{Dab, StrokeMethod};
    // Randomize Color must jitter the per-layer CUSTOM colours too (the artist's case), not only the
    // un-coloured layers. Brush base grey; both layers a custom green. Two dabs with different `d.color`
    // shift the green by different HSV offsets → the two locations differ (the path used to ignore the
    // custom colours, so Randomize Color had no effect).
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.5, 0.5, 0.5];
    t.paint.brush.color_jitter_hue = 0.5; // Randomize Color active (amount > 0)
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [0.0, 1.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    let dab = |cx: f32, col: [f32; 3]| Dab {
        center: [cx, 32.0],
        radius_px: 6.0,
        coverage: 1.0,
        color: col,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 6.0,
    };
    t.stamp_dabs(&[dab(16.0, [1.0, 0.0, 0.0])]);
    t.stamp_dabs(&[dab(48.0, [0.0, 0.0, 1.0])]);
    let a = px(&t, 64, 16, 32);
    let b = px(&t, 64, 48, 32);
    assert_ne!(a, b, "custom layer colours jitter per dab: {a:?} vs {b:?}");
}

// ============================================================================
// FASE A — Per-Layer Color perf-measurement harness.
// Tracker: docs/Painter/handoffs/HANDOFF_per_layer_color_perf_artifacts.md §1 (owed numbers).
// Ignored by default. Run in RELEASE — dev (opt-0) lies about perf
// (project_painter_composite_perf):
//   cargo test -p ph2d-tool-painter --release per_layer_perf -- --ignored --nocapture
//
// Design: drive the real pointer API (no GUI needed for timing). A Curve in
// DRAW mode re-fills the whole anchor->cursor line every Move, so K identical
// Moves at a fixed cursor = pure per-move cost at a fixed geometry. Comparing a
// DIAGONAL vs a HORIZONTAL line of the SAME length (same dab count D) isolates
// the bbox-bound cost (save/restore memcpy + O(bbox.N) recomposite + the
// composite_region in take_preview_arc) from the dab-count-bound cost (the
// whole-shape re-stamp + the O(D.N.S) accumulate). N-scaling (2 vs 16 shape
// layers) isolates the x N loops from the N-independent memcpy.
// ============================================================================

#[test]
fn per_layer_texture_color_paints_each_layers_own_rgb() {
    // The capture DEFAULT: a layer without a custom pick paints its OWN captured RGB (Texture Color).
    // With a constant orientation this routes through the cached RGBA path (baked premul stamps) — the
    // per-dab dynamic path only runs for Rake/Random/Jitter/Randomize/canvas-fixed Grain. Two layers:
    // bottom all-RED full mask, top all-GREEN on the LEFT half only → the painted tip is green on the
    // left (top over bottom) and red on the right (bottom alone).
    use ph2d_painter_brush::StrokeMethod;
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    let s = 16u32;
    let full = vec![255u8; (s * s) as usize];
    let mut left = vec![0u8; (s * s) as usize];
    for y in 0..s {
        for x in 0..s / 2 {
            left[(y * s + x) as usize] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(full, s, s), (left, s, s)]);
    let red = vec![[255u8, 0, 0]; (s * s) as usize]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();
    let green = vec![[0u8, 255, 0]; (s * s) as usize]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();
    t.paint
        .shape_layers
        .set_layers_meta(vec![red, green], vec![1.0; 2], vec![0; 2], vec![1, 2]);
    t.toggle_brush_shape_per_layer_color();
    assert!(t.paint.shape_layers.is_color_mode());
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let l = px(&t, 64, 27, 32); // left of centre — the GREEN top layer covers here
    let r = px(&t, 64, 37, 32); // right of centre — only the RED bottom layer covers
    assert!(
        l[1] > 180 && l[0] < 80,
        "left half paints the TOP layer's green rgb: {l:?}"
    );
    assert!(
        r[0] > 180 && r[1] < 80,
        "right half paints the BOTTOM layer's red rgb: {r:?}"
    );
}

mod per_layer_perf {
    use super::*;
    use ph2d_painter_brush::{Falloff, StrokeMethod};
    use std::time::Instant;

    /// White `size`x`size` canvas, `n_shape` full 16x16 Shape layers (distinct colours), Per-Layer
    /// Color ON, Curve method, hard disk `radius`. `doc_extra` extra raster doc layers make the
    /// document stack non-trivial (exercises `take_preview_arc`'s `composite_region` lane).
    fn setup(size: u32, n_shape: usize, doc_extra: usize, radius: f32) -> PainterTool {
        let mut t = white_canvas(size, radius);
        t.paint.brush.stroke_method = StrokeMethod::Arc;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.falloff = Falloff::Constant;
        t.paint.brush.space_attenuation = false;
        let layers: Vec<(Vec<u8>, u32, u32)> = (0..n_shape)
            .map(|_| (vec![255u8; 16 * 16], 16, 16))
            .collect();
        t.set_brush_shape_layers(layers);
        t.toggle_brush_shape_per_layer_color();
        for i in 0..n_shape {
            let f = i as f32;
            t.set_brush_shape_layer_color(
                i,
                [(f * 0.13) % 1.0, (f * 0.27) % 1.0, (f * 0.41) % 1.0],
            );
        }
        for k in 0..doc_extra {
            t.add_raster_layer(format!("L{k}"));
        }
        assert!(
            t.paint.shape_layers.is_color_mode(),
            "harness must be in per-layer-colour mode"
        );
        t
    }

    /// Time K identical full-line re-fills (the Curve draw branch re-fills anchor->p1 every Move) and
    /// the matching `take_preview_arc` drains. Returns `(avg_move_us, avg_preview_us)`.
    fn measure(t: &mut PainterTool, p0: [f32; 2], p1: [f32; 2], k: usize) -> (f64, f64) {
        t.on_canvas_pointer(cp(p0, PointerPhase::Down));
        t.on_canvas_pointer(cp(p1, PointerPhase::Move)); // establish the full line
        let _ = t.take_preview_arc(); // drain the establish frame
        let mut move_ns = 0u128;
        let mut prev_ns = 0u128;
        for _ in 0..k {
            let a = Instant::now();
            t.on_canvas_pointer(cp(p1, PointerPhase::Move));
            move_ns += a.elapsed().as_nanos();
            let b = Instant::now();
            let _ = t.take_preview_arc();
            prev_ns += b.elapsed().as_nanos();
        }
        let kf = k as f64;
        (move_ns as f64 / kf / 1000.0, prev_ns as f64 / kf / 1000.0)
    }

    /// Diagonal vs horizontal endpoints of EQUAL length `len`, inset `m` from the canvas origin.
    fn endpoints(size: u32, m: f32, len: f32, diagonal: bool) -> ([f32; 2], [f32; 2]) {
        if diagonal {
            let d = len / std::f32::consts::SQRT_2;
            ([m, m], [m + d, m + d])
        } else {
            ([m, size as f32 * 0.5], [m + len, size as f32 * 0.5])
        }
    }

    #[test]
    #[ignore = "perf measurement — run explicitly in --release"]
    fn per_layer_perf_sweep() {
        println!(
            "\n{:>6} {:>5} {:>4} {:>4} | {:>10} {:>10} | {:>10} {:>10} | {:>7}",
            "size", "r", "N", "doc", "D.move", "D.prev", "H.move", "H.prev", "D/H"
        );
        for &size in &[256u32, 1024u32] {
            let m = size as f32 * 0.1;
            let len = size as f32 * 0.6;
            for &radius in &[8.0_f32, 40.0, 100.0] {
                for &n in &[2usize, 16usize] {
                    for &doc in &[0usize, 1usize] {
                        let mut td = setup(size, n, doc, radius);
                        let (d0, d1) = (
                            endpoints(size, m, len, true).0,
                            endpoints(size, m, len, true).1,
                        );
                        let (dm, dp) = measure(&mut td, d0, d1, 30);
                        let mut th = setup(size, n, doc, radius);
                        let (h0, h1) = (
                            endpoints(size, m, len, false).0,
                            endpoints(size, m, len, false).1,
                        );
                        let (hm, hp) = measure(&mut th, h0, h1, 30);
                        println!(
                            "{size:>6} {radius:>5.0} {n:>4} {doc:>4} | {dm:>10.1} {dp:>10.1} | {hm:>10.1} {hp:>10.1} | {:>6.1}x",
                            if hm > 0.0 { dm / hm } else { 0.0 }
                        );
                    }
                }
            }
        }
        println!(
            "\nus per Move. move=curve_fill+save/restore+accumulate+recomposite; prev=take_preview_arc."
        );
        println!(
            "D/H = diagonal/horizontal at EQUAL dab count: >>1 => bbox-bound; ~1 => D.S.N-bound."
        );
        println!("time ~prop N => the xN per-layer loops dominate.\n");
    }

    /// The LIVE config (Enio 2026-07-04: "FPS 60→10 com Line/Arc/Ellipse/Polygon/Freehand"): captured
    /// layers WITHOUT a custom colour pick default to **Texture Color** → the route takes the per-pixel
    /// DYNAMIC path (`stamp_dabs_per_layer_dynamic`), not the cached one the batch kernel accelerated.
    /// Times the EDIT phase (drag the Arc's mid control point — a whole-shape re-stamp per move) at
    /// 2048², dynamic (texture-colour default) vs cached (all colours picked), N 3/16:
    ///   cargo test -p ph2d-tool-painter --release per_layer_perf_live -- --ignored --nocapture
    #[test]
    #[ignore = "perf measurement — run explicitly in --release"]
    fn per_layer_perf_live() {
        let size = 2048u32;
        let radius = 100.0f32;
        for &n in &[3usize, 16] {
            for &custom_colors in &[false, true] {
                let mut t = white_canvas(size, radius);
                t.paint.brush.stroke_method = StrokeMethod::Arc;
                t.paint.brush.hardness = 1.0;
                t.paint.brush.falloff = Falloff::Constant;
                t.paint.brush.space_attenuation = false;
                // 128² soft-disc layer masks (a real captured-layer silhouette, not a flat square).
                let layers: Vec<(Vec<u8>, u32, u32)> = (0..n)
                    .map(|_| {
                        let s = 128u32;
                        let mut m = vec![0u8; (s * s) as usize];
                        for y in 0..s {
                            for x in 0..s {
                                let dx = x as f32 - 64.0;
                                let dy = y as f32 - 64.0;
                                let d = (dx * dx + dy * dy).sqrt() / 64.0;
                                m[(y * s + x) as usize] = ((1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
                            }
                        }
                        (m, s, s)
                    })
                    .collect();
                t.set_brush_shape_layers(layers);
                // Real captured layers carry per-pixel RGB (`w·h·3`) — WITHOUT it `any_texture_color()`
                // is false and the route silently falls back to the cached path, hiding the dynamic
                // kernel (the live default) from the measurement.
                let rgb: Vec<Vec<u8>> = (0..n)
                    .map(|i| {
                        let s = 128usize;
                        let mut v = vec![0u8; s * s * 3];
                        for p in 0..s * s {
                            v[p * 3] = ((p * 7 + i * 31) % 256) as u8;
                            v[p * 3 + 1] = ((p * 13 + i * 17) % 256) as u8;
                            v[p * 3 + 2] = ((p * 3 + i * 53) % 256) as u8;
                        }
                        v
                    })
                    .collect();
                t.paint.shape_layers.set_layers_meta(
                    rgb,
                    vec![1.0; n],
                    vec![0; n],
                    (0..n as u64).collect(),
                );
                t.toggle_brush_shape_per_layer_color();
                if custom_colors {
                    for i in 0..n {
                        let f = i as f32;
                        t.set_brush_shape_layer_color(
                            i,
                            [(f * 0.13) % 1.0, (f * 0.27) % 1.0, (f * 0.41) % 1.0],
                        );
                    }
                }
                assert!(t.paint.shape_layers.is_color_mode());
                t.add_raster_layer("doc"); // non-trivial doc stack → real preview lane
                // Create the Arc: a long horizontal drag (chord 1800 px → mid bows to y≈754).
                t.on_canvas_pointer(cp([124.0, 1024.0], PointerPhase::Down));
                t.on_canvas_pointer(cp([1924.0, 1024.0], PointerPhase::Move));
                t.on_canvas_pointer(cp([1924.0, 1024.0], PointerPhase::Up));
                let _ = t.take_preview_arc();
                // EDIT phase: grab the (bowed) mid anchor and wiggle it — one whole-shape re-stamp per move.
                let mid = t.curve_overlay().expect("arc open").points[1];
                t.on_canvas_pointer(cp(mid, PointerPhase::Down));
                let moves = 10u32;
                let mut held = None;
                let mut move_ns = 0u128;
                let mut prev_ns = 0u128;
                for k in 0..moves {
                    let d = ((k % 5) as f32) - 2.0;
                    let a = std::time::Instant::now();
                    let _ = t.on_canvas_pointer(cp([mid[0] + d, mid[1] + d], PointerPhase::Move));
                    move_ns += a.elapsed().as_nanos();
                    let b = std::time::Instant::now();
                    if let Some(p) = t.take_preview_arc() {
                        held = Some(p); // bridge retainer
                    }
                    prev_ns += b.elapsed().as_nanos();
                }
                let _ = held;
                t.on_canvas_pointer(cp(mid, PointerPhase::Up));
                let kf = f64::from(moves);
                eprintln!(
                    "  live 2048² r100 N{n:<2} {}  move {:>9.1} us   prev {:>9.1} us",
                    if custom_colors {
                        "cached (colours picked)"
                    } else {
                        "texture-colour (default)"
                    },
                    move_ns as f64 / kf / 1000.0,
                    prev_ns as f64 / kf / 1000.0,
                );
            }
        }
    }

    /// Worst observed config (1024 r100 N16, diagonal) in isolation so `PH2D_PAINT_PROF=1` prints a
    /// clean accumulate-vs-recomposite split:
    ///   PH2D_PAINT_PROF=1 cargo test -p ph2d-tool-painter --release per_layer_perf_worst -- --ignored --nocapture
    #[test]
    #[ignore = "perf measurement — run explicitly in --release with PH2D_PAINT_PROF=1"]
    fn per_layer_perf_worst() {
        let size = 1024u32;
        let m = size as f32 * 0.1;
        let len = size as f32 * 0.6;
        let mut t = setup(size, 16, 0, 100.0);
        let (p0, p1) = (
            endpoints(size, m, len, true).0,
            endpoints(size, m, len, true).1,
        );
        let (mv, pv) = measure(&mut t, p0, p1, 8);
        println!("worst: move_us={mv:.1} prev_us={pv:.1}");
    }

    /// REGRESSION (Enio 2026-07-11 smoke, clean `--release`): editing a per-layer-colour shape left "lines
    /// at the edges of rectangles, in the brush's own colours" — and `PH2D_PAINT_FULL_UPLOAD=1` did NOT
    /// clear them, so it is NOT the partial GPU upload; it is upstream (the canvas restore/recomposite, or
    /// the `composite_region`/`blit_region` cache lane). The trigger is a shape that MOVES so the new
    /// footprint does NOT cover the old (the screenshot's off-centre ghost) — the [[watercolor Drag-Dot
    /// "moving preview restores the old position"]] class, unguarded for the per-layer-colour shape path.
    /// A line whose endpoint SWEEPS across the canvas must leave the SAME preview as the final line drawn
    /// directly; any pixel that differs is residue the sweep failed to revert (the stale rectangle). The
    /// diff bbox tells whether it is an axis-aligned rectangle (the Bug #9 signature). Cached path (colours
    /// picked) AND the per-dab dynamic path (Randomize Colour — Enio's 3D-look brushes) are both checked.
    #[test]
    fn per_layer_moving_shape_leaves_no_stale_rectangle() {
        let size = 256u32;
        let a = [40.0f32, 40.0]; // fixed line start
        // Endpoint sweeps a wide arc → successive lines barely overlap (the moving-preview case).
        let sweep = [
            [220.0f32, 60.0],
            [210.0, 200.0],
            [70.0, 215.0],
            [200.0, 130.0],
        ];
        let run = |doc: usize, randomize: bool| -> (usize, (u32, u32, u32, u32)) {
            let mk = || {
                let mut t = setup(size, 3, doc, 12.0);
                t.paint.brush.stroke_method = StrokeMethod::Line;
                if randomize {
                    // Route through the per-dab DYNAMIC path (`stamp_dabs_per_layer_dynamic`): a Hue jitter
                    // makes `has_colour_jitter_amount()` true — the gate the 3D-look brushes trip.
                    t.paint.brush.color_jitter_hue = 0.5;
                }
                t
            };
            let last = *sweep.last().unwrap();
            // TRUTH: the final line, drawn directly.
            let mut truth = mk();
            truth.on_canvas_pointer(cp(a, PointerPhase::Down));
            truth.on_canvas_pointer(cp(last, PointerPhase::Move));
            let (tb, w, h) = truth.take_preview_arc().expect("truth preview");
            // ACTUAL: the endpoint sweeps through every point, ending at the SAME final line.
            let mut actual = mk();
            actual.on_canvas_pointer(cp(a, PointerPhase::Down));
            let mut ab_opt = None;
            for p in sweep {
                actual.on_canvas_pointer(cp(p, PointerPhase::Move));
                if let Some(v) = actual.take_preview_arc() {
                    ab_opt = Some(v); // drain each frame like the bridge; keep the last non-empty
                }
            }
            let (ab, _, _) = ab_opt.expect("actual preview");
            let mut n = 0usize;
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    if tb[i..i + 4] != ab[i..i + 4] {
                        n += 1;
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (n, (x0, y0, x1, y1))
        };
        let (n0, b0) = run(0, false);
        let (n1, b1) = run(1, false);
        let (nd, bd) = run(0, true);
        assert_eq!(
            n0, 0,
            "cached, trivial stack: {n0} residue px on the CANVAS, bbox={b0:?}"
        );
        assert_eq!(
            n1, 0,
            "cached, doc stack: {n1} residue px in the COMPOSITE cache lane, bbox={b1:?}"
        );
        assert_eq!(
            nd, 0,
            "DYNAMIC (Randomize Colour): {nd} residue px, bbox={bd:?}"
        );
    }

    /// The REAL context (Enio's screenshot): a PARKED shape (drawn earlier) plus an ACTIVE shape being
    /// EDITED — the `restamp_shapes_preview` multi-shape path (active + every parked re-stamped onto one
    /// baseline each frame), NOT the single `stamp_drag_preview` a plain stroke uses. As the active shape's
    /// handle sweeps, the union footprint shifts; any pixel that ends up different from the same final
    /// two-shape scene built directly is residue (the off-centre ghost). Trivial + doc stack.
    #[test]
    fn per_layer_multishape_edit_leaves_no_stale_rectangle() {
        let size = 256u32;
        // Shape 1 (parked): a small ellipse in the top-left. Shape 2 (active): drawn bottom-right, then its
        // right handle is dragged around. The sweep + the final resting handle position.
        let c1 = [70.0f32, 70.0];
        let c2 = [165.0f32, 165.0];
        let sweep = [
            [235.0f32, 165.0],
            [165.0, 235.0],
            [120.0, 120.0],
            [210.0, 150.0],
        ];
        let run = |doc: usize| -> (usize, (u32, u32, u32, u32)) {
            // Returns the LIVE preview captured mid-drag (no pen-up — the artifact is a live-preview residue
            // the final commit would otherwise hide).
            let build = |edits: &[[f32; 2]]| -> (Arc<Vec<u8>>, u32, u32) {
                let mut t = setup(size, 3, doc, 12.0);
                t.paint.brush.stroke_method = StrokeMethod::Ellipse;
                // Shape 1 → parked once shape 2 begins.
                t.on_canvas_pointer(cp(c1, PointerPhase::Down));
                t.on_canvas_pointer(cp([c1[0] + 25.0, c1[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp([c1[0] + 25.0, c1[1]], PointerPhase::Up));
                // Shape 2 (empty Down parks shape 1) → radius 40, then editable.
                t.on_canvas_pointer(cp(c2, PointerPhase::Down));
                t.on_canvas_pointer(cp([c2[0] + 40.0, c2[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp([c2[0] + 40.0, c2[1]], PointerPhase::Up));
                // Edit shape 2: grab the right handle (at centre + rx) and drag it through `edits` — NO Up.
                let h = [c2[0] + 40.0, c2[1]];
                t.on_canvas_pointer(cp(h, PointerPhase::Down));
                let mut prev = None;
                for &e in edits {
                    t.on_canvas_pointer(cp(e, PointerPhase::Move));
                    if let Some(v) = t.take_preview_arc() {
                        prev = Some(v);
                    }
                }
                prev.or_else(|| {
                    t.preview_dirty = true;
                    t.take_preview_arc()
                })
                .expect("a live preview mid-edit")
            };
            let last = *sweep.last().unwrap();
            let (tb, w, h) = build(&[last]); // edit straight to the final handle position
            let (ab, _, _) = build(&sweep); // sweep through every intermediate position
            let mut n = 0usize;
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    if tb[i..i + 4] != ab[i..i + 4] {
                        n += 1;
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (n, (x0, y0, x1, y1))
        };
        let (n0, b0) = run(0);
        let (n1, b1) = run(1);
        assert_eq!(
            n0, 0,
            "multi-shape, trivial stack: {n0} residue px, bbox={b0:?}"
        );
        assert_eq!(
            n1, 0,
            "multi-shape, doc stack: {n1} residue px, bbox={b1:?}"
        );
    }

    /// The DECISIVE oracle: gesture-vs-gesture cancels a bug that is geometry-dependent (both go through
    /// the partial `composite_region`/`blit_region` cache lane). Compare the incrementally-blitted
    /// `composited` CACHE against a FULL recompose of the SAME final state — that is the exact difference
    /// `PH2D_PAINT_FULL_UPLOAD` cannot fix (it uploads the stale cache). Edits sweep a shape toward the
    /// canvas EDGE so the dirty bbox can clamp (`composite_region` returns `rw < bbox.w` while `blit_region`
    /// strides by `bbox.w` — the §3-B shear). Non-trivial doc stack (the composite lane only runs there).
    #[test]
    fn per_layer_composite_cache_matches_full_recompose_during_shape_edit() {
        let size = 128u32;
        let mut t = setup(size, 3, 1, 10.0);
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        // Draw an ellipse near the right edge, then drag its handle across the boundary and back.
        let c = [96.0f32, 64.0];
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        t.on_canvas_pointer(cp([c[0] + 24.0, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp([c[0] + 24.0, c[1]], PointerPhase::Up));
        let handle = [c[0] + 24.0, c[1]];
        t.on_canvas_pointer(cp(handle, PointerPhase::Down));
        let mut partial = None;
        for &e in &[[124.0f32, 64.0], [110.0, 30.0], [70.0, 64.0], [118.0, 90.0]] {
            t.on_canvas_pointer(cp(e, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial.expect("a partial-lane composited preview mid-edit");
        // FULL recompose of the SAME final state (drop the incremental cache).
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "partial composite cache != full recompose: {n} stale px, bbox=({x0},{y0})..({x1},{y1}) \
             — the incremental blit lane left a stale rectangle (FULL_UPLOAD can't fix this)"
        );
    }

    /// The screenshot's actual shape: a self-overlapping FREE HAND stroke (the un-incrementalised
    /// whole-path re-fill) with per-layer colour + Randomize Colour (the dynamic recomposite path) +
    /// a doc stack. Mid-draw, the partial composite CACHE must equal a FULL recompose of the same state.
    #[test]
    fn per_layer_freehand_selfoverlap_cache_matches_full_recompose() {
        let size = 160u32;
        let mut t = setup(size, 3, 1, 8.0);
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5; // → the dynamic per-dab recomposite path
        // A figure-8 that crosses itself (the pretzel), captured point by point.
        let path = [
            [40.0f32, 80.0],
            [70.0, 40.0],
            [110.0, 40.0],
            [120.0, 80.0],
            [90.0, 120.0],
            [60.0, 120.0],
            [40.0, 80.0],
            [70.0, 60.0],
            [110.0, 100.0],
        ];
        t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
        let mut partial = None;
        for &p in &path[1..] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview mid free-hand");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "free-hand: {n} stale px, bbox=({x0},{y0})..({x1},{y1})"
        );
    }

    /// The MISSING context (Enio 2026-07-11: "esqueci de colocar na freehand"): FreeHand is the ONLY shape
    /// method that does NOT coalesce (`coalesces_canvas_motion == false`) — the real app processes SEVERAL
    /// pointer Moves per frame, then ONE `take_preview_arc`. Every prior harness drained once per Move, so
    /// the multi-Move-per-frame accumulation of the growing whole-path re-fill was never exercised. Compare
    /// the partial composite CACHE built that way against a FULL recompose of the same final state.
    #[test]
    fn per_layer_freehand_multimove_per_frame_matches_full_recompose() {
        let size = 200u32;
        let mut t = setup(size, 3, 1, 6.0);
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5; // dynamic per-dab path (Enio's 3D brushes)
        // A growing, self-overlapping scribble captured point by point (spaced > min-capture).
        let pts: Vec<[f32; 2]> = (0..40)
            .map(|i| {
                let f = i as f32;
                // A lissajous-ish curve that crosses itself, no RNG, deterministic.
                let x = 100.0 + 70.0 * ((f * 0.5).sin());
                let y = 100.0 + 60.0 * ((f * 0.31).sin());
                [x, y]
            })
            .collect();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        let mut partial = None;
        // 4 Moves per "frame", ONE drain per frame (the un-coalesced FreeHand cadence).
        for frame in pts[1..].chunks(4) {
            for &p in frame {
                t.on_canvas_pointer(cp(p, PointerPhase::Move));
            }
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "free-hand multi-move/frame: {n} stale px, bbox=({x0},{y0})..({x1},{y1}) \
             — the partial cache diverged from a full recompose"
        );
    }

    /// Enio's ACTUAL screenshot scene: a FREE HAND scribble already drawn (→ PARKED) with an ELLIPSE editor
    /// active ON TOP of it, being edited. `restamp_shapes_preview` then re-stamps the parked free-hand's
    /// LONG dab list + the active ellipse onto one baseline EVERY move — the heaviest multi-shape path, and
    /// the one the screenshot shows. Partial composite cache vs FULL recompose of the same final state.
    #[test]
    fn per_layer_parked_freehand_plus_active_ellipse_matches_full_recompose() {
        let size = 220u32;
        let mut t = setup(size, 3, 1, 7.0);
        // 1) Draw a self-overlapping FREE HAND scribble, pen-up → it parks when the next shape starts.
        t.paint.brush.stroke_method = StrokeMethod::FreeHand;
        t.paint.brush.color_jitter_hue = 0.5;
        let pts: Vec<[f32; 2]> = (0..32)
            .map(|i| {
                let f = i as f32;
                [
                    70.0 + 55.0 * (f * 0.5).sin(),
                    80.0 + 45.0 * (f * 0.31).sin(),
                ]
            })
            .collect();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for frame in pts[1..].chunks(4) {
            for &p in frame {
                t.on_canvas_pointer(cp(p, PointerPhase::Move));
            }
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        let _ = t.take_preview_arc();
        // 2) An ELLIPSE on top (empty Down parks the free-hand), then EDIT its handle across a sweep.
        t.paint.brush.stroke_method = StrokeMethod::Ellipse;
        let c = [150.0f32, 150.0];
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        t.on_canvas_pointer(cp([c[0] + 45.0, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp([c[0] + 45.0, c[1]], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let handle = [c[0] + 45.0, c[1]];
        t.on_canvas_pointer(cp(handle, PointerPhase::Down));
        let mut partial = None;
        for &e in &[
            [205.0f32, 150.0],
            [150.0, 205.0],
            [95.0, 120.0],
            [190.0, 175.0],
        ] {
            t.on_canvas_pointer(cp(e, PointerPhase::Move));
            if let Some(v) = t.take_preview_arc() {
                partial = Some(v);
            }
        }
        let (pb, w, h) = partial
            .or_else(|| {
                t.preview_dirty = true;
                t.take_preview_arc()
            })
            .expect("a partial-lane preview mid-edit");
        t.composited = None;
        t.preview_dirty = true;
        let (fb, _, _) = t.take_preview_arc().expect("a full recompose");
        let mut n = 0usize;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                if pb[i..i + 4] != fb[i..i + 4] {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        assert_eq!(
            n, 0,
            "parked free-hand + active ellipse: {n} stale px, bbox=({x0},{y0})..({x1},{y1})"
        );
    }
}
