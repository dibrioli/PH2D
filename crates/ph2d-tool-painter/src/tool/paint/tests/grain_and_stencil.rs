//! **O grão e o estêncil.** O slot Grain (fonte, mapeamento, rake, offset aleatório, o giro por dab do
//! Jitter Rotate) e o Stencil: a moldura que recorta o dab, as suas alças de mover/redimensionar/rodar,
//! e a pré-visualização que aparece durante a transformação.

use super::*;

#[test]
fn jitter_rotate_reaches_smear_on_a_flattened_untextured_dab() {
    // Sweep (2026-07-12): `has_per_dab_rotation()` demanded `texture.is_active()`, so a FLATTENED dab with
    // no Shape and no Grain looked "constant" to the guard — and Smear/Blur/Clone served it the cached,
    // constant-orientation StampMask. Every dab smeared with the SAME ellipse angle: Jitter Rotate did
    // nothing there. (The paint path never had it — with both slots off it has no cache to serve, so
    // `jitter_rotate_spins_a_flattened_falloff_with_no_texture` has always passed. This is its Smear twin.)
    // Jitter Rotate spins the whole FOOTPRINT, so an anisotropic footprint alone makes it visible.
    // RED without the fix: the two canvases are byte-identical.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::StrokeMethod;
    let size = 48u32;
    let smear = |seed: u64| -> Vec<u8> {
        let mut t = PainterTool::default();
        // Left half black, right half white — the SAME fixture the working Smear gate uses: the dab has to
        // straddle a BOUNDARY, because smearing inside a uniform region is a no-op at any angle.
        let mut src = vec![255u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size / 2 {
                let i = ((y * size + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            space_attenuation: false,
            stroke_method: StrokeMethod::Space, // allows_jitter
            dab_flatten: 0.6,                   // anisotropic ⇒ a per-dab rotation is VISIBLE
            jitter_rotate: 1.0,
            ..Default::default()
        };
        // The tool keeps a brush PER MODE — seed every slot, or selecting Smear swaps the settings out.
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.paint.seed = seed; // the jitter draw
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_PAINT_MODE,
            "smear".to_string(),
        ));
        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([(size / 2 - 6) as f32, mid], PointerPhase::Down));
        t.on_canvas_pointer(cp([(size / 2 + 8) as f32, mid], PointerPhase::Move));
        t.on_canvas_pointer(cp([(size / 2 + 8) as f32, mid], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    // Guard against a no-op fixture proving nothing: the smear must actually have dragged the boundary.
    let mut pristine = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            pristine[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
    let a = smear(1);
    assert_ne!(a, pristine, "the smear actually dragged pixels");
    assert_ne!(
        a,
        smear(999),
        "Jitter Rotate must reach the Smear of a flattened, untextured dab"
    );
}

#[test]
fn granulation_re_bakes_the_coloured_stamp() {
    // Sweep (2026-07-12): `render_color_stamp_mask` folds `effective_granulation()` into the baked Grain
    // coverage — so Granulation is an INPUT of the bake — but `ColorStampKey` did not carry it. The
    // grayscale `StampKey` always has (with a comment saying exactly why); the coloured twin was written
    // without it. Dragging Granulation left the coloured stamp STALE until some other field moved the key.
    //
    // ★ The test MUST reuse ONE tool. Baking on two fresh tools proves nothing — each starts with a cold
    // cache, so it re-bakes either way and the test goes green with the bug alive (it did, for me, until I
    // ran the RED). A cache-key gate has to exercise the CACHE HIT.
    // RED without the fix: the second bake is byte-identical — the key matched and the stale stamp was reused.
    use ph2d_painter_brush::{TextureKind, TextureSettings};
    let mut t = white_canvas(64, 9.0);
    t.paint.brush.watercolor = true; // `effective_granulation()` is 0 unless watercolor is on
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        size: [6.0, 6.0],
        ..t.paint.brush.texture
    };
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    let stamp_bytes = |t: &PainterTool| -> Vec<u8> {
        t.paint
            .color_stamp_cache
            .as_ref()
            .expect("stamp baked")
            .0
            .iter()
            .flat_map(|s| s.data().to_vec())
            .collect()
    };
    t.paint.brush.granulation = 0.0;
    let brush = t.paint.brush;
    t.ensure_color_stamp_cache(&brush, 64);
    let before = stamp_bytes(&t);
    // The user drags Granulation — nothing else moves.
    t.paint.brush.granulation = 0.9;
    let brush = t.paint.brush;
    t.ensure_color_stamp_cache(&brush, 64); // must MISS the cache and re-bake
    assert_ne!(
        before,
        stamp_bytes(&t),
        "Granulation is folded into the baked Grain coverage — it MUST be in the stamp's key"
    );
}

#[test]
fn stencil_overlay_outlines_the_rect_only_for_stencil() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    // No texture → no overlay.
    assert!(t.stencil_overlay().is_none());
    // A texture but a non-Stencil mapping → still no overlay.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    };
    assert!(t.stencil_overlay().is_none());
    // Stencil, centred, full-canvas size (stencil_size 1), no rotation → corners at the canvas corners.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0],
        ..Default::default()
    };
    let o = t.stencil_overlay().expect("stencil overlay present");
    assert_eq!(
        o.corners,
        [[0.0, 0.0], [64.0, 0.0], [64.0, 64.0], [0.0, 64.0]],
        "centred full-canvas stencil outlines the whole canvas"
    );
    // The DEFAULT stencil (stencil_size 0.5) is a centred rect at 50 % of the sprite.
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        ..Default::default()
    };
    assert_eq!(
        t.stencil_overlay().expect("overlay").corners,
        [[16.0, 16.0], [48.0, 16.0], [48.0, 48.0], [16.0, 48.0]],
        "the default stencil rect is 50% of the sprite"
    );
}

#[test]
fn stencil_dab_paints_only_inside_the_rect() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    // A hard black dab whose Stencil rect covers only the centre: a corner well outside the rect
    // stays white (masked), proving the engine mask reaches the canvas via stamp_dabs.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.4, 0.4], // a central rect ≈ [19.2 .. 44.8]
        ..Default::default()
    };
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert_eq!(
        px(&t, 64, 2, 2),
        [255, 255, 255, 255],
        "a corner outside the stencil rect is untouched"
    );
}

#[test]
fn per_layer_color_grain_stencil_masks_to_the_rect_not_the_whole_dab() {
    use ph2d_painter_brush::{Dab, StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    // Regression (Enio 2026-06-27): with Per-Layer Color ON, a Grain mapped Stencil was baked into the
    // dab-LOCAL cached coloured stamp, which can't represent the canvas-fixed rect → the colour leaked
    // OUTSIDE the Stencil (worst on a big Anchored dab). Canvas-fixed Grain now routes to the per-pixel
    // dynamic path, which masks the rect. A pixel inside the dab but outside the rect must stay white.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]);
    t.set_brush_shape_layer_color(1, [0.0, 1.0, 0.0]);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.3, 0.3], // central rect ≈ [22.4 .. 41.6]
        ..Default::default()
    };
    let dab = Dab {
        center: [32.0, 32.0],
        radius_px: 30.0, // covers the canvas; the silhouette (full square) reaches well past the rect
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 30.0,
    };
    t.stamp_dabs(&[dab]);
    // (32,10): inside the dab radius (dy = 22 < 30) but ABOVE the rect (y < 22.4) → masked = white.
    assert_eq!(
        px(&t, 64, 32, 10),
        [255, 255, 255, 255],
        "Per-Layer Color + Grain Stencil masks to the rect — inside the dab but outside the rect stays white"
    );
}

// Diagnostics (Enio 2026-06-28): does each per-dab rotation actually reach the painted result? Two
// strokes with DIFFERENT seeds must differ — the seed only feeds the rotation here, so identical = the
// rotation was DROPPED (the cached-vs-per-pixel path confound is avoided: both runs use the SAME path).
fn directional_bar() -> Vec<u8> {
    let mut bar = vec![0u8; 64]; // 8×8, top 3 rows white = directional under rotation
    for px in bar.iter_mut().take(3 * 8) {
        *px = 255;
    }
    bar
}

#[test]
fn jitter_rotate_reaches_curve_fill() {
    use ph2d_painter_brush::StrokeMethod;
    let bar = directional_bar();
    let run = |seed: u64| {
        let mut t = white_canvas(64, 8.0);
        t.set_brush_shape_image(bar.clone(), 8, 8);
        t.paint.brush.stroke_method = StrokeMethod::Arc;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([54.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate must vary the Curve fill with the seed"
    );
}

#[test]
fn jitter_rotate_spins_a_flattened_falloff_with_no_texture() {
    use ph2d_painter_brush::StrokeMethod;
    // Enio 2026-06-28: Jitter Rotate spins the brush FOOTPRINT (the flatten + rotation circle), so a
    // flattened round brush with NO Texture and NO Shape still rotates per dab (the `Texture: None` case).
    let run = |seed: u64| {
        let mut t = white_canvas(64, 10.0);
        t.paint.brush.dab_flatten = 0.5; // elliptical footprint (anisotropic under rotation)
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([14.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate spins the flattened footprint even with no Texture / Shape"
    );
}

#[test]
fn jitter_rotate_reaches_the_paint() {
    use ph2d_painter_brush::StrokeMethod;
    let bar = directional_bar();
    let run = |seed: u64| {
        let mut t = white_canvas(48, 8.0);
        t.set_brush_shape_image(bar.clone(), 8, 8);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate must vary with the seed (rotation reaches the paint)"
    );
}

#[test]
fn jitter_rotate_is_the_grains_random_spin_now_that_per_slot_random_angle_is_gone() {
    // The per-slot "Random Angle" was retired 2026-07-19: the Stroke Jitter Rotate spins the WHOLE stamp
    // (Shape + View-Grain together) by a random per-dab angle — the coherent superset, and more expressive
    // (it has an amount). This pins that removing the per-slot toggle lost NO capability on the Grain: a
    // random per-dab Grain rotation still reaches the paint through Jitter Rotate. `jitter_rotate_reaches_
    // the_paint` above is the Shape twin. If someone re-adds a per-slot Random Angle "because we lost it",
    // these two say we did not.
    use ph2d_painter_brush::{StrokeMethod, TextureKind};
    let run = |seed: u64| {
        let mut t = white_canvas(48, 8.0);
        t.paint.brush.texture.kind = TextureKind::Stripes; // a directional grain (a spin is visible)
        t.paint.brush.texture.size = [0.5, 0.5];
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.set_brush_jitter_rotate(1.0);
        t.paint.seed = seed;
        t.on_canvas_pointer(cp([10.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([38.0, 24.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    assert_ne!(
        run(1),
        run(999),
        "Jitter Rotate spins the Grain per dab — the retired per-slot Random Angle's replacement"
    );
}

#[test]
fn jitter_rotate_panel_event_sets_the_field() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = white_canvas(48, 8.0);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_ROTATE,
        0.7,
    ));
    assert!(
        (t.brush_settings().jitter_rotate - 0.7).abs() < 1e-4,
        "the panel slider event must set jitter_rotate: {}",
        t.brush_settings().jitter_rotate
    );
}

#[test]
fn grain_ramp_stencil_does_not_paint_outside_the_rect() {
    use ph2d_painter_brush::texture::{TextureKind, TextureMapping};
    // Regression (Enio 2026-06-28): a Grain **Color Ramp** indexed by the grain value painted `ramp[0]`
    // OUTSIDE the Stencil rect — `sample()` returns 0 there, which the ramp read as a colour (not "no
    // paint"). The rect must mask the ramp path too. A central rect; a dab covers the canvas; a pixel
    // inside the dab but outside the rect must stay white.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture.kind = TextureKind::Noise;
    t.paint.brush.texture.mapping = TextureMapping::Stencil;
    t.paint.brush.texture.stencil_size = [0.3, 0.3]; // central rect ≈ [22.4 .. 41.6]
    t.set_texture_ramp_enabled(true); // grain value → ramp colour
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)); // radius 30 dab over the canvas
    assert_eq!(
        px(&t, 64, 32, 8),
        [255, 255, 255, 255],
        "the Grain ramp is masked by the Stencil — inside the dab but outside the rect stays white"
    );
}

#[test]
fn loading_a_grain_image_fits_the_aspect_for_each_mapping() {
    use ph2d_painter_brush::TextureMapping;
    // Enio 2026-06-28: a Grain Image is never squashed. STENCIL shapes the rect to the image (Size 1:1);
    // the other mappings put the aspect in the Grain Size (`sx:sy = h:w`).
    let mut t = white_canvas(64, 8.0);

    // Stencil: a 2:1 image → stencil_size aspect 2:1 (wider axis at the 0.5 box); Size stays 1:1.
    t.set_brush_texture_mapping(TextureMapping::Stencil.to_u8());
    t.set_brush_texture_image(vec![128u8; 32 * 16], 32, 16); // 2:1
    let b = t.brush_settings();
    assert!(
        (b.stencil_size[0] - 0.5).abs() < 1e-4 && (b.stencil_size[1] - 0.25).abs() < 1e-4,
        "2:1 image → stencil_size [0.5, 0.25]: {:?}",
        b.stencil_size
    );
    assert_eq!(
        b.texture_size,
        [1.0, 1.0],
        "Stencil image fills the rect once (Size 1:1)"
    );

    // View Plane: the aspect goes into the Grain Size — a 2:1 image → [0.5, 1.0] (h:w), so it's not squashed.
    t.set_brush_texture_mapping(TextureMapping::ViewPlane.to_u8());
    let s = t.brush_settings().texture_size;
    assert!(
        (s[0] - 0.5).abs() < 1e-4 && (s[1] - 1.0).abs() < 1e-4,
        "2:1 image (View) → Size [0.5, 1.0]: {s:?}"
    );
    // A tall 1:2 image flips it.
    t.set_brush_texture_image(vec![128u8; 16 * 32], 16, 32); // 1:2
    let s = t.brush_settings().texture_size;
    assert!(
        (s[0] - 1.0).abs() < 1e-4 && (s[1] - 0.5).abs() < 1e-4,
        "1:2 image (View) → Size [1.0, 0.5]: {s:?}"
    );
}

#[test]
fn stencil_corner_drag_with_shift_scales_uniformly() {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    // Shift held → a Stencil corner scale preserves the grab-time aspect ratio (the Sprite gizmo's
    // aspect-lock): dragging mostly along X grows BOTH axes by the X factor, keeping the 2:1 rect 2:1.
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.4, 0.2], // a 2:1 rect (centred)
        ..Default::default()
    };
    let corner = t.stencil_overlay().unwrap().corners[2]; // [++] bottom-right
    t.set_uniform_scale(true);
    assert!(
        t.on_canvas_pointer(cp(corner, PointerPhase::Down)),
        "grab the corner"
    );
    t.on_canvas_pointer(cp([60.0, 34.0], PointerPhase::Move)); // grow X a lot, Y barely
    let s = t.brush_settings();
    let aspect = s.stencil_size[0] / s.stencil_size[1];
    assert!(
        (aspect - 2.0).abs() < 0.05,
        "uniform scale keeps the 2:1 aspect: {aspect} ({:?})",
        s.stencil_size
    );
}

#[test]
fn anchored_stencil_does_not_leak_outside_the_rect_during_the_drag() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind, TextureMapping, TextureSettings};
    // Regression (Enio 2026-06-27): an Anchored stroke with a Grain mapped Stencil leaked colour OUTSIDE
    // the rect while dragging (the interactive preview stamped texture-free for speed → no stencil mask).
    // A small central rect; the Anchored anchor sits in a corner well outside it; after the size-drag the
    // anchor pixel (dab centre, falloff = 1) must stay white — the stencil masks the live preview too.
    let mut t = white_canvas(64, 30.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.3, 0.3], // central rect ≈ [22.4 .. 41.6]
        ..Default::default()
    };
    t.set_brush_stroke_method(StrokeMethod::Anchored.to_u8());
    assert!(t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down))); // anchor in the corner (outside rect)
    let _ = t.on_canvas_pointer(cp([8.0, 50.0], PointerPhase::Move)); // grow radius ≈ 42 (covers the canvas)
    assert_eq!(
        px(&t, 64, 8, 8),
        [255, 255, 255, 255],
        "the Anchored preview is masked by the stencil — the corner outside the rect stays white"
    );
}

/// A `PainterTool` with a centred full-canvas Stencil texture (handles at the canvas corners +
/// centre), ready for the drag-gesture tests.
fn stencil_tool() -> PainterTool {
    use ph2d_painter_brush::{TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0], // stencil_offset 0, stencil_size 1 → rect = whole canvas
        ..Default::default()
    };
    t
}

#[test]
fn stencil_centre_handle_drag_moves_the_rect() {
    let mut t = stencil_tool();
    let center = t.stencil_overlay().expect("overlay").center; // (32, 32)
    assert!(
        t.on_canvas_pointer(cp(center, PointerPhase::Down)),
        "grab the centre handle"
    );
    let _ = t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Move));
    let _ = t.on_canvas_pointer(cp([40.0, 36.0], PointerPhase::Up));
    let s = t.brush_settings();
    // new centre (40,36) → stencil_offset = (px/64*2 − 1) = (0.25, 0.125). The gizmo writes the
    // dedicated stencil field, NOT the texture offset.
    assert!(
        (s.stencil_offset[0] - 0.25).abs() < 1e-3,
        "x {}",
        s.stencil_offset[0]
    );
    assert!(
        (s.stencil_offset[1] - 0.125).abs() < 1e-3,
        "y {}",
        s.stencil_offset[1]
    );
    assert_eq!(
        s.texture_offset,
        [0.0, 0.0],
        "texture offset untouched by the gizmo"
    );
}

#[test]
fn stencil_corner_handle_drag_resizes_the_rect() {
    let mut t = stencil_tool();
    // Grab the bottom-right corner (64, 64) and drag in to (48, 48).
    assert!(
        t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down)),
        "grab a corner handle"
    );
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Move));
    let _ = t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    let s = t.brush_settings();
    // half = |(48,48) − centre(32,32)| = 16 each → stencil_size = 2·16/64 = 0.5.
    assert!(
        (s.stencil_size[0] - 0.5).abs() < 1e-3,
        "x {}",
        s.stencil_size[0]
    );
    assert!(
        (s.stencil_size[1] - 0.5).abs() < 1e-3,
        "y {}",
        s.stencil_size[1]
    );
    assert_eq!(
        s.texture_size,
        [1.0, 1.0],
        "texture size untouched by the gizmo"
    );
}

#[test]
fn stencil_corner_ring_drag_rotates_the_rect() {
    let mut t = stencil_tool();
    t.set_shape_grab_tol_px(5.0); // scale ≤ 5 px; the rotate ring is 5..13 px past a corner
    // A point just OUTSIDE the bottom-right corner (64, 64): in the rotate ring, not the scale disc.
    let down = [70.0, 70.0]; // dist from the corner ≈ 8.5 px
    assert!(
        t.on_canvas_pointer(cp(down, PointerPhase::Down)),
        "grab the rotate ring just outside a corner"
    );
    assert!(
        t.stencil_overlay().expect("overlay").rotating,
        "the active grab is a rotation (square→circle cue)"
    );
    // Drag from 45° to 135° about the centre (32, 32) → +90°.
    let _ = t.on_canvas_pointer(cp([-6.0, 70.0], PointerPhase::Move));
    let deg = i32::from(t.brush_settings().stencil_angle_deg);
    assert!((deg - 90).abs() <= 2, "stencil rotated ~90°, got {deg}");
    let _ = t.on_canvas_pointer(cp([-6.0, 70.0], PointerPhase::Up));
    assert_eq!(
        t.brush_settings().texture_angle_deg,
        0,
        "the texture's own angle is untouched by the gizmo"
    );
}

#[test]
fn stencil_preview_shows_during_transform_and_fades_when_idle() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = stencil_tool();
    assert!(t.stencil_preview().is_none(), "no preview when idle");
    // A panel param change (Stencil card) arms the transient in-gizmo preview.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
        30.0,
    ));
    assert!(
        t.stencil_preview().is_some(),
        "preview shows after a stencil param change"
    );
    // It fades after the hold window (decayed by the per-frame tick).
    t.paint_tick(1.0); // 1 s ≫ the hold
    assert!(
        t.stencil_preview().is_none(),
        "preview fades once the user stops changing params"
    );
    // A handle drag shows it live; releasing hides it crisply.
    let center = t.stencil_overlay().expect("overlay").center;
    let _ = t.on_canvas_pointer(cp(center, PointerPhase::Down));
    assert!(
        t.stencil_preview().is_some(),
        "preview shows during a handle drag"
    );
    let _ = t.on_canvas_pointer(cp(center, PointerPhase::Up));
    assert!(
        t.stencil_preview().is_none(),
        "preview hides the moment the drag ends"
    );
}

#[test]
fn grain_assign_auto_enables_shape_bw_and_resets_the_grain_ramp() {
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(64, 10.0);
    // A coloured Shape ramp (no Grain yet): enable it, B&W off, so it owns colour.
    t.set_shape_ramp_enabled(true);
    assert!(
        !t.shape_ramp_bw(),
        "Shape ramp starts as a colour ramp (B&W off)"
    );
    // A coloured GRAIN ramp too, enabled — so we can observe it reset.
    t.toggle_texture_ramp_enabled();
    assert!(t.brush_settings().texture_ramp_enabled);

    // Assign a Grain texture → the Shape ramp's B&W auto-enables (it becomes the tone), and the (now
    // Grain-owned) colour ramp resets to its default off state (Enio 2026-06-26).
    t.set_brush_texture_kind(TextureKind::Noise.to_u8());
    let b = t.brush_settings();
    assert!(
        b.shape_color_ramp_bw,
        "assigning a Grain auto-enabled the Shape ramp's B&W (tone) filter"
    );
    assert!(
        b.shape_color_ramp_enabled,
        "the Shape ramp stays enabled (now as tone)"
    );
    assert!(
        !b.texture_ramp_enabled && !b.texture_ramp_bw,
        "assigning a Grain reset the Grain colour ramp to defaults"
    );
}

#[test]
fn texture_image_request_then_modulates_the_dab() {
    use ph2d_painter_brush::TextureKind;
    let mut t = white_canvas(64, 12.0);
    // Picking the Image kind requests a file load (the shell polls this); consumed once.
    t.set_brush_texture_kind(TextureKind::Image.to_u8());
    assert!(
        t.take_brush_texture_image_request(),
        "picking Image requests a file load"
    );
    assert!(
        !t.take_brush_texture_image_request(),
        "the request is consumed once"
    );
    // All-black luminance → mask 0 → the dab paints nothing.
    t.set_brush_texture_image(vec![0u8; 16], 4, 4);
    assert_eq!(t.brush_settings().texture_kind, TextureKind::Image.to_u8());
    let _ = t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 64, 32, 32),
        [255, 255, 255, 255],
        "an all-black image mask paints nothing"
    );
    // All-white luminance → mask 1 → paints fully (hard brush → black centre).
    t.set_brush_texture_image(vec![255u8; 16], 4, 4);
    let _ = t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    assert_eq!(
        px(&t, 64, 20, 20),
        [0, 0, 0, 255],
        "an all-white image mask paints fully"
    );
}

#[test]
fn stencil_down_away_from_handles_paints_not_edits() {
    let mut t = stencil_tool();
    let before = t.brush_settings();
    // (16, 8) is well clear of every handle (corners + centre) → no grab → it paints.
    let _ = t.on_canvas_pointer(cp([16.0, 8.0], PointerPhase::Down));
    let after = t.brush_settings();
    assert_eq!(
        before.stencil_offset, after.stencil_offset,
        "a Down away from handles must not move the stencil"
    );
    assert_eq!(
        before.stencil_size, after.stencil_size,
        "a Down away from handles must not resize the stencil"
    );
    assert!(t.dirty_rect.is_some(), "it painted instead of editing");
}

#[test]
fn stencil_card_panel_events_drive_the_stencil_frame_not_the_texture() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::{TEX_SIZE_MAX, TextureKind, TextureMapping, TextureSettings};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.texture = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        ..Default::default()
    };
    // The Stencil card's number boxes write the REAL value to the dedicated stencil_* fields.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        0.3,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_Y,
        -0.5,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
        90.0,
    ));
    let s = t.brush_settings();
    assert!(
        (s.stencil_size[0] - 0.3).abs() < 1e-6,
        "{}",
        s.stencil_size[0]
    );
    assert!(
        (s.stencil_offset[1] + 0.5).abs() < 1e-6,
        "{}",
        s.stencil_offset[1]
    );
    assert_eq!(s.stencil_angle_deg, 90);
    // The texture tiling is independent state — the card leaves it alone.
    assert_eq!(s.texture_size, [1.0, 1.0], "texture size untouched");
    assert_eq!(s.texture_offset, [0.0, 0.0], "texture offset untouched");
    assert_eq!(s.texture_angle_deg, 0, "texture angle untouched");
    // Real-value clamp to the size bound (not a 0..1 remap).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        999.0,
    ));
    assert!((t.brush_settings().stencil_size[0] - TEX_SIZE_MAX).abs() < 1e-6);
}

// ── Per-dab randomize seam (Jitter Scale / Rotate + Randomize Color) ─────────────────────────
// These prove the panel controls are WIRED end-to-end: the generic PanelEvent reaches the brush
// state (not silently dropped — the dead-control class) AND the per-dab jitter actually alters the
// painted pixels. Unit-green ≠ product-green; only this e2e drive catches a missing register/route.

#[test]
fn randomize_controls_reach_the_brush_and_snapshot() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    let mut t = PainterTool::default();
    // Enable toggle (Click) + the five 0..1 sliders (SetValue) — exactly what the panel emits.
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
        0.3,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
        0.2,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        0.1,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_SCALE,
        0.7,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_ROTATE,
        0.4,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_JITTER_SPACING,
        0.6,
    ));
    // (a) the events reached the brush model (would all be 0/false if dropped).
    assert!(t.paint.brush.color_jitter_enabled, "enable toggle wired");
    assert_eq!(t.paint.brush.color_jitter_hue, 0.3, "Hue slider wired");
    assert_eq!(t.paint.brush.color_jitter_sat, 0.2, "Sat slider wired");
    assert_eq!(t.paint.brush.color_jitter_val, 0.1, "Value slider wired");
    assert_eq!(t.paint.brush.jitter_scale, 0.7, "Jitter Scale slider wired");
    assert_eq!(
        t.paint.brush.jitter_rotate, 0.4,
        "Jitter Rotate slider wired"
    );
    assert_eq!(
        t.paint.brush.jitter_spacing, 0.6,
        "Jitter Spacing slider wired"
    );
    // (b) the published snapshot the panel reads back mirrors them (slider positions).
    let s = t.brush_settings();
    assert!(s.color_jitter_enabled);
    assert_eq!(s.color_jitter, [0.3, 0.2, 0.1]);
    assert_eq!(s.jitter_scale, 0.7);
    assert_eq!(s.jitter_rotate, 0.4);
    assert_eq!(s.jitter_spacing, 0.6);
    // A second enable Click toggles it back off.
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    assert!(!t.paint.brush.color_jitter_enabled, "enable toggle flips");
}

#[test]
fn randomize_color_varies_the_painted_pixels_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    // Mid-grey base + hard disk so each dab fully replaces its footprint with its (jittered) colour.
    let mut t = white_canvas(64, 3.0);
    t.paint.brush.color = [0.5, 0.5, 0.5];
    // Drive Randomize Color ON with a strong Value amount via the PANEL events (the wiring proof).
    t.handle_panel_event(PanelEvent::Click(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
    ));
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        1.0,
    ));
    // Paint a multi-dab horizontal stroke; per-dab Value jitter must yield >1 painted shade.
    t.on_canvas_pointer(cp([6.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Up));
    let shades: std::collections::BTreeSet<u8> = (6..58).map(|x| px(&t, 64, x, 32)[0]).collect();
    assert!(
        shades.len() > 1,
        "Randomize Color must vary the painted shades end-to-end, got {shades:?}"
    );

    // Control: with Randomize Color OFF the same stroke paints a single uniform shade.
    let mut t0 = white_canvas(64, 3.0);
    t0.paint.brush.color = [0.5, 0.5, 0.5];
    t0.on_canvas_pointer(cp([6.0, 32.0], PointerPhase::Down));
    t0.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Move));
    t0.on_canvas_pointer(cp([58.0, 32.0], PointerPhase::Up));
    let base: std::collections::BTreeSet<u8> = (6..58).map(|x| px(&t0, 64, x, 32)[0]).collect();
    assert_eq!(base.len(), 1, "no jitter ⟹ one uniform shade, got {base:?}");
}
