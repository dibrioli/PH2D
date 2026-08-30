//! **O slot Shape: de onde a SILHUETA vem e o que a colore.** A fonte (procedural, imagem, camadas),
//! o dropdown de follow, a máscara pelo falloff, as rampas de valor e de cor, a opacidade e o blend da
//! camada de origem — e o anel do cursor, que tem de apontar para onde o próximo dab vai apontar.

use super::*;

#[test]
fn shape_follow_dropdown_selects_off_rake_flow_mutually_exclusively() {
    // The Follow dropdown (Off/Rake/Flow) drives the two engine flags `shape.rake`/`shape.flow` from a
    // SINGLE control: exactly one (or neither) is on. Picking Flow must set flow and CLEAR rake; picking
    // Rake must set rake and CLEAR flow; picking Off clears both. Drives the real panel `SelectOption`,
    // and reads the engine flags directly (not just the snapshot) so the wiring can't be green while the
    // engine sees the wrong state.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let mut t = PainterTool::default();
    let pick = |t: &mut PainterTool, mode: &str| {
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_SHAPE_FOLLOW,
            mode.to_string(),
        ));
    };
    pick(&mut t, "2"); // Flow
    assert!(t.paint.brush.shape.flow, "Flow selected → shape.flow on");
    assert!(!t.paint.brush.shape.rake, "Flow selected → shape.rake off");
    assert_eq!(t.brush_settings().shape_follow, 2, "snapshot reports Flow");
    pick(&mut t, "1"); // Rake
    assert!(t.paint.brush.shape.rake, "Rake selected → shape.rake on");
    assert!(
        !t.paint.brush.shape.flow,
        "Rake selected → shape.flow off (mutually exclusive)"
    );
    assert_eq!(t.brush_settings().shape_follow, 1, "snapshot reports Rake");
    pick(&mut t, "0"); // Off
    assert!(
        !t.paint.brush.shape.rake && !t.paint.brush.shape.flow,
        "Off → both flags cleared"
    );
    assert_eq!(t.brush_settings().shape_follow, 0, "snapshot reports Off");
}

#[test]
fn shape_source_dropdown_requests_image_and_clears_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    let mut t = PainterTool::default();
    // Picking "Image" in the Shape source dropdown requests a file load (the shell polls it); the engine
    // does no I/O, so the silhouette stays the falloff until pixels arrive. Mirrors the Grain Kind flow.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Image.to_u8().to_string(),
    ));
    assert!(
        t.take_brush_shape_image_request(),
        "picking Image requests a Shape file load"
    );
    assert!(
        !t.take_brush_shape_image_request(),
        "the Shape request is consumed once"
    );
    assert!(
        !t.brush_settings().shape_has_image,
        "no pixels yet ⇒ silhouette is still the falloff"
    );

    // The shell delivers the pixels ⇒ shape_has_image flips (the dropdown then reads "Image").
    t.set_brush_shape_image(vec![255u8; 16], 4, 4);
    assert!(t.brush_settings().shape_has_image, "shape image assigned");

    // Picking "None" clears the image (→ falloff), the same as the section reset.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::None.to_u8().to_string(),
    ));
    assert!(
        !t.brush_settings().shape_has_image,
        "picking None cleared the shape image"
    );

    // Picking a PROCEDURAL kind installs that pattern (no pixels) — the panel's "Texture" picker.
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    assert_eq!(
        t.brush_settings().shape_kind,
        TextureKind::Checker.to_u8(),
        "procedural Shape kind installed"
    );
    assert!(
        !t.brush_settings().shape_has_image,
        "a procedural Shape never holds pixels"
    );

    // The procedural Shape exposes the kind's per-pattern params (like the Grain): a SetValue on a
    // PAINTER_SHAPE_PARAMS slider tunes only the Shape pattern.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_PARAMS[0], 0.9));
    assert!(
        (t.brush_settings().shape_params[0] - 0.9).abs() < 1e-6,
        "Shape per-pattern param routed to the Shape slot"
    );
}

#[test]
fn procedural_shape_is_masked_by_the_falloff_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    // A soft round falloff so the envelope actually attenuates toward the dab edge.
    let mut a = white_canvas(64, 24.0);
    a.paint.brush.falloff = Falloff::Smooth;
    let _ = a.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));

    // Same brush + a procedural Checker Shape, selected via the panel "Texture" picker.
    let mut b = white_canvas(64, 24.0);
    b.paint.brush.falloff = Falloff::Smooth;
    b.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    let _ = b.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));

    // The procedural silhouette is `falloff × pattern ≤ falloff`, so the Checker dab deposits strictly
    // LESS total ink than the bare-falloff dab (the pattern carves out ~half), yet still paints. Total
    // coverage is the robust invariant (a per-pixel bound is foiled by the cached stamp's bilinear blit
    // of the sharp checker edge). Proves the masking end-to-end (panel "Texture" pick → engine).
    let ink = |t: &PainterTool| -> u64 {
        let mut s = 0u64;
        for yy in 0..64 {
            for xx in 0..64 {
                s += 255 - u64::from(px(t, 64, xx, yy)[0]); // darkness on white = deposited ink
            }
        }
        s
    };
    let (ink_falloff, ink_checker) = (ink(&a), ink(&b));
    assert!(ink_checker > 0, "the Checker Shape must still paint");
    assert!(
        ink_checker < ink_falloff * 9 / 10,
        "the falloff must MASK the Checker (less ink than the bare falloff): {ink_checker} vs {ink_falloff}"
    );
}

#[test]
fn shape_value_ramp_remaps_the_silhouette_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    use ph2d_painter_brush::TextureKind;

    let ink = |t: &PainterTool| -> u64 {
        let mut s = 0u64;
        for yy in 0..64 {
            for xx in 0..64 {
                s += 255 - u64::from(px(t, 64, xx, yy)[0]); // darkness on white = deposited ink
            }
        }
        s
    };

    // The Shape ramp acts as the B&W **tone** remap when its B&W filter is on — which auto-enables
    // when a Grain is assigned (Enio 2026-06-26) — so assign a Noise Grain in each case below.
    // White tip (silhouette 1) + identity ramp (luma(v)=v) ⇒ the tip paints under the Grain.
    let mut t2 = white_canvas(64, 12.0);
    t2.set_brush_shape_image(vec![255u8; 16], 4, 4);
    t2.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t2.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_ENABLE));
    t2.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let ink_identity = ink(&t2);
    assert!(
        ink_identity > 0,
        "identity tone ramp still paints (with a Grain)"
    );

    // INVERT the ramp (white→black) ⇒ the value-1 tip maps to 0 BEFORE the Grain multiply ⇒ the tip is
    // zeroed: the centre stays pure white, and far less ink overall.
    let mut t3 = white_canvas(64, 12.0);
    t3.set_brush_shape_image(vec![255u8; 16], 4, 4);
    t3.set_brush_texture_kind(TextureKind::Noise.to_u8());
    t3.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_ENABLE));
    t3.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_SHAPE_RAMP_INVERT));
    t3.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    assert_eq!(
        px(&t3, 64, 32, 32),
        [255, 255, 255, 255],
        "the inverted tone ramp zeroes the white tip (before the Grain multiply)"
    );
    assert!(
        ink(&t3) < ink_identity / 2,
        "inverted tone ramp deposits far less ink: {} vs {ink_identity}",
        ink(&t3)
    );
}

#[test]
fn shape_colour_ramp_colourises_the_silhouette_when_grain_is_none() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};

    // No Grain ⇒ the SHAPE colour ramp OWNS the painted colour (B&W off): the silhouette coverage
    // indexes it (Enio 2026-06-26). A solid-red ramp over a BLUE brush base ⇒ a RED dab — proving the
    // Shape ramp colourises (without it the centre would be the brush's blue).
    let mut t = white_canvas(64, 12.0);
    t.set_brush_color_channel(2, 1.0); // brush base = blue [0,0,1]
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(true); // B&W stays off ⇒ the ramp colourises
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(
        c[0] > 200 && c[1] < 80 && c[2] < 80,
        "grain-None Shape colour ramp paints red (not the brush's blue): {c:?}"
    );
}

#[test]
fn resetting_the_shape_clears_the_per_layer_color_state() {
    // Reset OR removing the Shape image (dropdown → None) must drop the captured layers + the Per-Layer
    // Color mode, so the panel rows disappear AND a now-None Shape never routes into the coloured path
    // (which left it un-paintable). Both the section Reset and the kind→None dropdown are covered.
    for clear_via_kind in [false, true] {
        let mut t = white_canvas(64, 6.0);
        t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
        t.toggle_brush_shape_per_layer_color();
        let on = t.brush_settings();
        assert!(
            on.shape_layer_count == 2 && on.shape_per_layer_color,
            "armed"
        );
        if clear_via_kind {
            t.set_brush_shape_kind(0); // TextureKind::None — "remove from the slot"
        } else {
            t.reset_brush_shape(); // the Shape section Reset button
        }
        let off = t.brush_settings();
        assert_eq!(off.shape_layer_count, 0, "captured layers dropped");
        assert!(!off.shape_per_layer_color, "Per-Layer Color mode dropped");
        // And painting still works — a plain dab lands (no stale coloured-path routing).
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        let c = px(&t, 64, 32, 32);
        assert!(c[3] > 0, "a normal dab still paints after the reset: {c:?}");
    }
}

#[test]
fn shape_layer_opacity_reaches_the_flattened_silhouette() {
    // Sweep (2026-07-12): `ShapeLayers::flatten()` scales each layer by `opacity[i]` — but `set_layers()`
    // RESETS the opacities to 1.0, and the capture only installs the real ones afterwards
    // (`set_layers_meta`). So the flatten always baked against all-1.0. And `set_opacity` never re-flattened
    // at all: the per-layer **Opacity** box was DEAD everywhere except Per-Layer Color mode (which applies
    // opacity at recomposite time and so bypasses the flatten entirely — which is exactly why nobody
    // noticed). The `op` term inside `flatten()` was unreachable code.
    // RED without the fix: the flattened silhouette is byte-identical after zeroing a layer's opacity.
    let mut t = white_canvas(64, 9.0);
    // Two DISJOINT halves. (A full bottom layer would over-composite to saturation and the top layer's
    // opacity could not change the result at ALL — the fixture would prove nothing.)
    let mut left = vec![0u8; 64];
    let mut right = vec![0u8; 64];
    for y in 0..8 {
        for x in 0..4 {
            left[y * 8 + x] = 255;
            right[y * 8 + x + 4] = 255;
        }
    }
    t.set_brush_shape_layers(vec![(left, 8, 8), (right, 8, 8)]);
    let before = t
        .brush_shape_image()
        .expect("a shape image was flattened")
        .0
        .to_vec();
    // Drop the top layer to fully transparent — it must vanish from the silhouette.
    t.set_brush_shape_layer_opacity(1, 0.0);
    let after = t
        .brush_shape_image()
        .expect("a shape image was flattened")
        .0
        .to_vec();
    assert_ne!(
        before, after,
        "layer Opacity scales the flattened silhouette — the box must re-bake it"
    );
}

#[test]
fn editing_the_shape_source_re_captures_and_keeps_colours() {
    use ph2d_painter_brush::StrokeMethod;
    // Capture a multi-layer sprite as the Shape + colour layer 0; painting on that SAME sprite (the Shape
    // source) auto-re-captures the Shape at pointer-up WITHOUT wiping the colours (no manual re-assign).
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    t.layers.add_raster("L2", 64, 64); // make it multi-layer
    t.capture_layers_as_brush_shape();
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_color(0, [1.0, 0.0, 0.0]); // red on layer 0
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up));
    t.refresh_shape_source_if_changed(); // the bridge calls this each frame; the paint changed the source
    let s = t.brush_settings();
    assert!(
        s.shape_layer_color_on[0],
        "layer 0 stays coloured after the auto re-capture"
    );
    assert_eq!(
        s.shape_layer_color[0],
        [1.0, 0.0, 0.0],
        "the per-layer red is preserved across the re-capture"
    );
    assert!(
        s.shape_per_layer_color,
        "per-layer-colour mode survives the re-capture"
    );
}

#[test]
fn changing_source_layer_opacity_re_captures_the_shape() {
    // Editing the reference sprite WITHOUT painting — here a layer's opacity — must still update the brush
    // Shape. The per-frame revision poll catches opacity / visibility / undo, not only paint strokes.
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    let l2 = t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape();
    let ver0 = t.brush_shape_image_version();
    t.set_layer_opacity(l2, 0.5); // edit the source, no painting
    t.refresh_shape_source_if_changed();
    assert_ne!(
        t.brush_shape_image_version(),
        ver0,
        "an opacity change on the source re-captures the Shape"
    );
}

#[test]
fn shape_layer_opacity_edits_its_source_document_layer_two_way() {
    // The per-layer opacity box is TWO-WAY with the Shape SOURCE layer's opacity slider: editing the box
    // edits exactly that source layer's opacity (Enio 2026-06-29). Uses `white_canvas` → `set_source`, so
    // the painter is NOT document-bound (`bound_doc == None`) — the real case where the box wasn't updating
    // the sprite layer; the guard is `bound_doc == shape_source_doc`, which holds for the unbound doc too.
    let mut t = white_canvas(64, 6.0);
    t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape(); // shape source == the (unbound) painted document
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_opacity(0, 0.5);
    let s = t.brush_settings();
    assert!(
        (s.shape_layer_opacity[0] - 0.5).abs() < 1e-3,
        "the box value mirrors into the brush snapshot: {}",
        s.shape_layer_opacity[0]
    );
    let changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| (l.opacity - 0.5).abs() < 1e-3)
        })
        .count();
    assert_eq!(
        changed, 1,
        "exactly the ONE source document layer's opacity changed (two-way), not the others"
    );
}

#[test]
#[allow(non_snake_case)]
fn shape_opacity_and_blend_remote_control_the_STASHED_source_sprite() {
    use ph2d_painter_effects::BlendMode;
    // The shape source sprite is used to paint OTHER sprites. After capturing sprite 1 as the Shape and
    // switching to paint sprite 2, sprite 1 is STASHED — yet editing opacity/blend in the brush must still
    // update sprite 1's layer (Enio 2026-06-29: "a sprite usada como shape, que agora não está mais
    // selecionada, deve ser atualizada"), never sprite 2's. Verified by switching back to sprite 1.
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
    t.layers.add_raster("L2", 64, 64).expect("add"); // multi-layer ⇒ sprite 1 gets stashed on switch
    t.capture_layers_as_brush_shape(); // shape source = sprite 1
    t.toggle_brush_shape_per_layer_color();
    t.bind_document(2, vec![0u8; 32 * 32 * 4], 32, 32); // paint sprite 2 — sprite 1 is now stashed
    let two_layers = t.brush_settings().shape_layer_count >= 2;
    t.set_brush_shape_layer_opacity(0, 0.4);
    if two_layers {
        t.set_brush_shape_layer_blend(1, BlendMode::Multiply.to_u8());
    }
    // Sprite 2's layers must be UNTOUCHED (we edited the stashed sprite 1, not the painted sprite 2).
    assert!(
        t.layers.root().iter().all(|&id| t
            .layers
            .get(id)
            .is_some_and(|l| (l.opacity - 0.4).abs() > 1e-3)),
        "the painted sprite 2 must NOT have its opacity changed"
    );
    // Switch BACK to sprite 1 — its stashed stack (restored) must carry the brush edits.
    t.bind_document(1, vec![0u8; 4], 1, 1);
    let op_changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| (l.opacity - 0.4).abs() < 1e-3)
        })
        .count();
    assert_eq!(
        op_changed, 1,
        "the stashed shape-source sprite's layer opacity was remote-controlled"
    );
    if two_layers {
        let blend_changed = t
            .layers
            .root()
            .iter()
            .filter(|&&id| {
                t.layers
                    .get(id)
                    .is_some_and(|l| l.blend_mode == BlendMode::Multiply)
            })
            .count();
        assert_eq!(
            blend_changed, 1,
            "the stashed shape-source sprite's layer blend was remote-controlled"
        );
    }
}

#[test]
fn shape_layer_blend_edits_its_source_document_layer_two_way() {
    use ph2d_painter_effects::BlendMode;
    // The blend dropdown is a REMOTE CONTROL of the source layer's blend mode (Enio 2026-06-29). Editing it
    // edits that source layer's `blend_mode` (and the Layers panel shows it). Layer index 1 is a non-base
    // layer (the base, index 0, has no blend).
    let mut t = white_canvas(64, 6.0);
    t.layers.add_raster("L2", 64, 64).expect("add layer");
    t.capture_layers_as_brush_shape();
    t.toggle_brush_shape_per_layer_color();
    let s = t.brush_settings();
    if s.shape_layer_count < 2 {
        return; // capture grabbed a single layer — the 2-layer blend path isn't exercised here
    }
    t.set_brush_shape_layer_blend(1, BlendMode::Multiply.to_u8());
    let changed = t
        .layers
        .root()
        .iter()
        .filter(|&&id| {
            t.layers
                .get(id)
                .is_some_and(|l| l.blend_mode == BlendMode::Multiply)
        })
        .count();
    assert_eq!(
        changed, 1,
        "exactly the ONE source layer's blend mode changed (remote control), not the others"
    );
    assert_eq!(
        t.brush_settings().shape_layer_blend[1],
        BlendMode::Multiply.to_u8(),
        "the brush snapshot mirrors the picked blend"
    );
}

#[test]
fn manual_blend_and_opacity_reflect_in_the_snapshot_and_paint() {
    use ph2d_painter_brush::StrokeMethod;
    // The "B" blend pick + the per-layer opacity box land in the snapshot the panel reads, and the
    // stroke paints. (Texture Color is the default — `color_on` off — so a custom colour is opt-in.)
    let mut t = white_canvas(64, 6.0);
    t.set_brush_shape_layers(vec![(vec![255u8; 64], 8, 8), (vec![255u8; 64], 8, 8)]);
    t.toggle_brush_shape_per_layer_color();
    t.set_brush_shape_layer_blend(1, 1); // Multiply on the top layer
    t.set_brush_shape_layer_opacity(0, 0.4); // brush-only opacity on the bottom layer
    let s = t.brush_settings();
    assert_eq!(
        s.shape_layer_blend[1], 1,
        "manual blend reflects in the snapshot"
    );
    assert!(
        (s.shape_layer_opacity[0] - 0.4).abs() < 1e-3,
        "per-layer opacity reflects in the snapshot: {}",
        s.shape_layer_opacity[0]
    );
    // The stroke still lands (no dead no-op).
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(c[3] > 0, "a per-layer-colour dab paints something: {c:?}");
}

#[test]
fn shape_ramp_swatch_select_option_sets_the_stop_colour() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    // The Shape ramp swatch picker forwards `"id,r,g,b,a"` (sRGB bytes) to PAINTER_SHAPE_RAMP_SWATCH;
    // the tool sets THAT stop's colour. Stop id 0 defaults to black → drive it to pure red.
    let mut t = PainterTool::default();
    let id0 = t.shape_color_ramp().stops()[0].id;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_SHAPE_RAMP_SWATCH,
        format!("{id0},255,0,0,255"),
    ));
    let s0 = *t
        .shape_color_ramp()
        .stops()
        .iter()
        .find(|s| s.id == id0)
        .unwrap();
    assert!(
        s0.color[0] > 0.9 && s0.color[1] < 0.1 && s0.color[2] < 0.1,
        "swatch set stop {id0} to red, got {:?}",
        s0.color
    );
}

#[test]
fn shape_image_paints_the_silhouette_end_to_end() {
    // A full-white 4×4 Shape image makes the dab a SQUARE silhouette: a footprint corner that the round
    // falloff disc would leave blank gets painted. Proves the tool routes the Shape slot end-to-end
    // (set image → stamp route → engine composition), not just the unit sampler.
    let size = 16u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(6.0);
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.falloff = Falloff::Smooth; // a SOFT disc — the corner is far below the rim

    // Control: no Shape image ⇒ the corner (3,3) stays white (round disc).
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    assert_eq!(
        px(&t, size, 3, 3),
        [255, 255, 255, 255],
        "round falloff leaves the corner blank"
    );

    // Assign the square Shape image and paint again ⇒ the corner is now painted (square silhouette).
    let mut t2 = PainterTool::default();
    t2.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t2.set_brush_size_px(6.0);
    t2.paint.brush.color = [0.0, 0.0, 0.0];
    t2.paint.brush.falloff = Falloff::Smooth;
    t2.set_brush_shape_image(vec![255u8; 16], 4, 4); // 4×4 all-white → full-square silhouette
    t2.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t2.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
    let corner = px(&t2, size, 3, 3);
    assert!(
        corner[0] < 80,
        "square shape paints the footprint corner (got {corner:?})"
    );
}

#[test]
fn shape_colour_ramp_paints_cached_and_colourises_the_silhouette() {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    // The no-Grain Shape Colour Ramp blits the cached coverage mask applying `ramp[coverage]` (the
    // fast path) — this proves it colourises correctly. With a Shape silhouette + Strength 1 + no Grain
    // + no per-dab rotation + B&W off, the router takes `stamp_dabs_cached_ramped`.
    let mut t = white_canvas(64, 16.0);
    t.set_brush_shape_image(vec![255u8; 16], 4, 4); // full-coverage silhouette ⇒ cacheable
    t.set_brush_strength(1.0); // no Accumulate cap ⇒ keeps the cacheable path
    // Shape colour ramp ON (B&W off), red high stop (full coverage → top colour).
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(true);

    // Paint a dab; the full-coverage silhouette centre takes the top ramp colour (red).
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    let c = px(&t, 64, 32, 32);
    assert!(
        c[0] > 150 && c[1] < 90 && c[2] < 90,
        "Shape colour-ramp centre is ramp red via the cached blit, got {c:?}"
    );
}

/// **A following pattern is a fact of the STROKE, not of the canvas** — so rotating the whole stroke by
/// 90 degrees must rotate the painting by 90 degrees. That is the property Enio reported missing (*"nenhum
/// deles consegue o ângulo em direção ao traço, para que as linhas permanecessem paralelas mesmo nas
/// curvas"*): a Shape pinned to the canvas keeps its pattern pointing the same way while the path turns
/// under it, and on a curve the stamps cross instead of continuing.
///
/// The oracle is a whole-canvas comparison, which needs no feature detection and cannot be fooled by a
/// pattern whose lines happen to run along or across the stroke. A 90-degree turn about the canvas centre
/// maps pixel centres exactly onto pixel centres (`B[py][px]` is `A[px][N-1-py]`), so no resampling enters
/// and the residual is only the engine's own float asymmetry.
///
/// ⚠️ This replaces a pair of `assert_ne!` gates. Byte-inequality is satisfied by ANY difference — and it
/// was: with the frame stuck on the static Angle, Flow still added its arc-length term and still skipped
/// the footprint, so the canvas always differed from both Off and Rake, and both gates passed over a Flow
/// that rendered identically to Off. Measured, the "difference" the old gate accepted was **0.55 %** of
/// texels. The gate measured *that something changed*; the artist measured *which way the lines point*.
#[test]
fn a_following_shape_paints_the_stroke_not_the_canvas() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind};
    const N: usize = 96;
    // The quarter circle, and the SAME curve turned 90 degrees about the canvas centre.
    let path = |k: f32| {
        let a = k * std::f32::consts::FRAC_PI_2;
        [14.0 + 56.0 * a.cos(), 14.0 + 56.0 * a.sin()]
    };
    let turned = |k: f32| {
        let p = path(k);
        [p[1], N as f32 - p[0]]
    };
    let paint = |rake: bool, flow: bool, turn: bool| {
        let mut t = white_canvas(N as u32, 9.0);
        t.paint.brush.shape.kind = TextureKind::Stripes;
        t.paint.brush.shape.size = [2.0, 2.0];
        t.paint.brush.shape.rake = rake;
        t.paint.brush.shape.flow = flow;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        let at = |k: f32| if turn { turned(k) } else { path(k) };
        t.on_canvas_pointer(cp(at(0.0), PointerPhase::Down));
        for i in 1..=60 {
            t.on_canvas_pointer(cp(at(i as f32 / 60.0), PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(at(1.0), PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    // Mean |difference| over the painted region between the turned painting and the turn of the painting.
    let residual = |rake: bool, flow: bool| {
        let a = paint(rake, flow, false);
        let b = paint(rake, flow, true);
        let (mut sum, mut n) = (0.0f64, 0usize);
        for py in 0..N {
            for px in 0..N {
                let vb = f32::from(b[(py * N + px) * 4]);
                let va = f32::from(a[(px * N + (N - 1 - py)) * 4]);
                // Only where SOMETHING was painted in either image (white canvas = 255).
                if va < 250.0 || vb < 250.0 {
                    sum += f64::from((va - vb).abs());
                    n += 1;
                }
            }
        }
        assert!(n > 500, "fixture painted almost nothing ({n} texels)");
        sum / n as f64
    };
    let flow = residual(false, true);
    let rake = residual(true, false);
    let off = residual(false, false);
    println!("PROBE flow={flow:.1} rake={rake:.1} off={off:.1}");
    assert!(
        flow < 0.35 * off,
        "FLOW must paint the STROKE, not the canvas: turning the stroke left a residual of {flow:.1} \
         against {off:.1} for a static Shape"
    );
    assert!(
        rake < 0.35 * off,
        "RAKE must paint the STROKE, not the canvas: residual {rake:.1} against {off:.1}"
    );
    assert!(
        off > 8.0,
        "control: a NON-following Shape is pinned to the canvas, so turning the stroke must NOT turn \
         the painting — if this residual is small the oracle cannot tell following from static ({off:.1})"
    );
}

/// **The brush-cursor ring turns with the stroke, in real time** (Enio 2026-07-19). With a slot following
/// the stroke, the ellipse the cursor draws must wear the orientation the NEXT dab will wear — otherwise
/// the artist is aiming a calligraphic nib with a picture of it pointing somewhere else.
///
/// The rotor is published by the tool (`BrushSettings::dab_rotor`) from the ENGINE's own live heading, so
/// this also pins that the ring never re-derives a direction of its own: a second estimate would drift
/// from the paint, and a cursor that disagrees with the mark is worse than one that does not move.
/// Jitter Rotate is excluded on purpose — per-dab randomness in a cursor reads as flicker, not as aim.
#[test]
fn the_brush_ring_rotor_turns_with_the_stroke_only_when_a_slot_follows() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind};
    const N: u32 = 96;
    let path = |k: f32| {
        let a = k * std::f32::consts::FRAC_PI_2;
        [14.0 + 56.0 * a.cos(), 14.0 + 56.0 * a.sin()]
    };
    let tangent_deg = |k: f32| {
        let a = k * std::f32::consts::FRAC_PI_2;
        (a.cos()).atan2(-a.sin()).to_degrees().rem_euclid(360.0)
    };
    // Drive the real stroke and read the published rotor at a few points along the curve.
    let rotors = |rake: bool| {
        let mut t = white_canvas(N, 9.0);
        t.paint.brush.shape.kind = TextureKind::Stripes;
        t.paint.brush.shape.rake = rake;
        t.paint.brush.dab_angle_deg = 0;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.on_canvas_pointer(cp(path(0.0), PointerPhase::Down));
        let mut out = Vec::new();
        for i in 1..=60 {
            let k = i as f32 / 60.0;
            t.on_canvas_pointer(cp(path(k), PointerPhase::Move));
            if i % 10 == 0 && i >= 20 {
                out.push((k, t.brush_settings().dab_rotor));
            }
        }
        t.on_canvas_pointer(cp(path(1.0), PointerPhase::Up));
        out
    };
    // FOLLOWING: the rotor tracks the path tangent all the way round.
    for (k, r) in rotors(true) {
        let deg = r[1].atan2(r[0]).to_degrees().rem_euclid(360.0);
        let want = tangent_deg(k);
        let d = (deg - want).rem_euclid(360.0);
        let err = if d > 180.0 { 360.0 - d } else { d };
        assert!(
            err < 8.0,
            "the ring must wear the stroke's orientation at k={k}: {deg:.1}deg vs tangent {want:.1}deg"
        );
        assert!(
            (r[0] * r[0] + r[1] * r[1] - 1.0).abs() < 1e-3,
            "the rotor stays unit-length"
        );
    }
    // NOT FOLLOWING: the ring rests at the brush Angle, bit-for-bit, however the stroke curves.
    for (_, r) in rotors(false) {
        assert_eq!(
            r,
            ph2d_painter_brush::texture::rotate_by_degrees(0),
            "a non-following brush's ring must not turn with the stroke"
        );
    }
}

/// **The ring aims BEFORE you click.** Hovering — cursor moving, no button down — must already turn the
/// brush-cursor rotor, because aiming a calligraphic nib is something you do on the way to the canvas
/// (Enio 2026-07-19: *"permita rodar em tempo real mesmo antes de clicar"*).
///
/// Also pins the two halves that make it safe: a brush with nothing following is **bit-identical** to the
/// resting Angle however you wave the cursor, and a hover with no motion holds rather than resets.
#[test]
fn hovering_aims_the_brush_ring_before_the_stroke_starts() {
    use ph2d_painter_brush::TextureKind;
    let hover_rotor = |rake: bool, dir: [f32; 2]| {
        let mut t = white_canvas(96, 9.0);
        t.paint.brush.shape.kind = TextureKind::Stripes;
        t.paint.brush.shape.rake = rake;
        t.paint.brush.dab_angle_deg = 0;
        // A run of hover samples along `dir` — enough travel for the EMA to settle.
        for i in 0..40u8 {
            let k = f32::from(i) * 3.0;
            t.on_canvas_hover([20.0 + dir[0] * k, 20.0 + dir[1] * k]);
        }
        t.brush_settings().dab_rotor
    };
    // FOLLOWING: the rotor lands on the hover direction, with no click anywhere in sight.
    let down = hover_rotor(true, [0.0, 1.0]);
    assert!(
        down[0].abs() < 0.05 && (down[1] - 1.0).abs() < 0.05,
        "hovering downward must aim the ring downward, got {down:?}"
    );
    let right = hover_rotor(true, [1.0, 0.0]);
    assert!(
        (right[0] - 1.0).abs() < 0.05 && right[1].abs() < 0.05,
        "hovering rightward must aim the ring rightward, got {right:?}"
    );
    // NOT FOLLOWING: bit-identical to the resting Angle, however the cursor moves.
    assert_eq!(
        hover_rotor(false, [0.0, 1.0]),
        ph2d_painter_brush::texture::rotate_by_degrees(0),
        "a non-following brush's ring must not react to hover at all"
    );
}

/// **A live stroke's heading BEATS the hover preview** — and the order is load-bearing. The engine's
/// heading is the one the PAINT uses; the hover value exists only to fill the gap before pen-down and
/// during the warm-up (where the engine's heading is still unset and the opening dabs are held anyway).
/// Preferring the hover would let the cursor keep pointing where the artist was *approaching* from while
/// the tip has already committed to the stroke.
#[test]
fn a_live_stroke_beats_the_hover_preview() {
    use ph2d_painter_brush::{StrokeMethod, TextureKind};
    let mut t = white_canvas(96, 9.0);
    t.paint.brush.shape.kind = TextureKind::Stripes;
    t.paint.brush.shape.rake = true;
    t.paint.brush.dab_angle_deg = 0;
    t.paint.brush.stroke_method = StrokeMethod::Space;
    // Approach the canvas heading DOWN...
    for i in 0..40u8 {
        t.on_canvas_hover([20.0, 20.0 + f32::from(i) * 3.0]);
    }
    let aimed = t.brush_settings().dab_rotor;
    assert!(
        aimed[1] > 0.9,
        "fixture: the hover should have aimed down, got {aimed:?}"
    );
    // ...then paint a long stroke to the RIGHT. The ring must follow the paint, not the approach.
    t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Down));
    for i in 1..=40u8 {
        t.on_canvas_pointer(cp([20.0 + f32::from(i) * 1.5, 60.0], PointerPhase::Move));
    }
    let painting = t.brush_settings().dab_rotor;
    assert!(
        painting[0] > 0.9 && painting[1].abs() < 0.2,
        "the live stroke's heading must win over the hover preview, got {painting:?}"
    );
}
