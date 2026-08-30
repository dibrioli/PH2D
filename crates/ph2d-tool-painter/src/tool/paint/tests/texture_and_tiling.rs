//! **As camadas de textura (`LayerKind::Texture`) e o ladrilho sem costura.** A textura como camada do
//! documento (render, edição ao vivo por evento de painel, duplicar, mascarar) e o Seamless Tiling: o
//! que a tinta faz ao atravessar a beira do sprite.

use super::*;

#[test]
fn tiling_wrapped_copies_share_the_dabs_random_frame() {
    // Sweep finding (2026-07-12). Under Tiling the canvas is a TORUS: a dab crossing an edge is drawn as
    // two `Dab`s (`tiling::tiled_dabs` replicates the list), but they are the SAME dab seen from both
    // sides — so they must share one random frame. They did not: the paint routes iterate the already-
    // wrapped list and draw from `tex_rng` PER COPY, so Shape/Grain Random-Angle (and Randomize Color)
    // gave each side of the seam a different draw. The seam stops matching — which breaks the whole
    // promise of seamless Tiling. Smear/Blur/Clone already do it right and say so out loud:
    // "Computed ONCE per dab, so the wrapped Tiling copies share the same random frame."
    //
    // ORACLE: paint the SAME dab twice from a fresh tool — once centred (no wrap) as the reference, once
    // straddling the left edge with Tiling on. On a torus the tiled canvas must be the reference disc,
    // translated. The wrapped band is where the second (spurious) rng draw shows up.
    // RED without the fix: the wrapped band differs from the reference.
    use ph2d_painter_brush::{Dab, TextureKind, TextureMapping, TextureSettings};
    let brush_with_random_grain = |t: &mut PainterTool| {
        t.paint.brush.texture = TextureSettings {
            kind: TextureKind::Noise,
            mapping: TextureMapping::Random, // Random Offset: a per-dab rng draw the wrapped copies must SHARE
            ..t.paint.brush.texture
        };
    };
    let dab_at = |cx: f32| Dab {
        center: [cx, 32.0],
        radius_px: 12.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 12.0,
    };
    // Reference: the same dab, centred, no tiling → the disc's own random frame.
    let mut r = white_canvas(64, 12.0);
    brush_with_random_grain(&mut r);
    r.stamp_dabs(&[dab_at(32.0)]);
    // Under test: the dab straddles the LEFT edge, Tiling X on → a wrapped copy lands on the right edge.
    let mut t = white_canvas(64, 12.0);
    brush_with_random_grain(&mut t);
    t.paint.tiling = [true, false];
    t.stamp_dabs(&[dab_at(2.0)]);
    // Map canvas → dab-local, carefully (an off-by-one here fails on ANY implementation and would be a
    // false RED): the dab straddling the left edge sits at x=2, so its wrapped copy is centred at 2+64=66.
    // A canvas pixel x in the wrapped band is dab-local `x-66`, which the centred reference holds at
    // `32+(x-66) = x-34`. The central band is dab-local `x-2`, held by the reference at `x+30`.
    let mut mismatch = 0;
    for y in 24..40u32 {
        for x in 54..64u32 {
            if px(&t, 64, x, y) != px(&r, 64, x - 34, y) {
                mismatch += 1; // the WRAPPED copy — where the spurious second rng draw lands
            }
        }
        for x in 0..14u32 {
            if px(&t, 64, x, y) != px(&r, 64, x + 30, y) {
                mismatch += 1; // the central copy — must equal the reference too (same first draw)
            }
        }
    }
    assert_eq!(
        mismatch, 0,
        "the wrapped copy must use the SAME random frame as its central self \
         ({mismatch} texels of the seam band disagree with the reference disc)"
    );
}

// ── Texture layers (LayerKind::Texture) — end-to-end through the panel-event path ──

/// `true` when the RGBA buffer is not a flat fill (the texture produced spatial variation).
fn buf_varies(b: &[u8]) -> bool {
    b.as_chunks::<4>().0.iter().any(|p| p != &b[0..4])
}

#[test]
fn texture_layer_renders_composites_and_edits_live_via_panel_events() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::TextureKind;

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32); // opaque white base
    t.toggle_dock(); // texture-layer editing lives in the Layers view (dock now opens on Brush)

    // Add a Texture layer: it becomes active, with its rendered pixels in `canvas_rgba`.
    let id = t.add_texture_layer().expect("texture layer added");
    assert!(
        matches!(
            t.layers().get(id).map(|l| &l.kind),
            Some(LayerKind::Texture(_))
        ),
        "the new layer is a Texture layer"
    );
    assert_eq!(t.layers().active(), Some(id), "the texture layer is active");
    let buf_default = t.canvas_rgba.as_ref().clone();
    assert_eq!(buf_default.len(), 32 * 32 * 4);
    assert!(
        buf_varies(&buf_default),
        "the default texture fills with variation"
    );

    // It composites like a raster (non-trivial stack → the texture covers the white base).
    let (composite, _, _) = t.run_full();
    assert!(
        buf_varies(&composite),
        "the composite shows the texture over the base"
    );

    // Live edit through the FROZEN panel-event channel — change the kind. The active layer is a
    // texture layer, so the tool routes the texture widget to it (not the brush).
    let brush_kind_before = t.brush_settings().texture_kind;
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        TextureKind::Checker.to_u8().to_string(),
    ));
    let buf_checker = t.canvas_rgba.as_ref().clone();
    assert_ne!(
        buf_default, buf_checker,
        "changing the kind re-rendered the layer live"
    );
    match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => assert_eq!(tex.kind, TextureKind::Checker.to_u8()),
        _ => panic!("layer should still be a Texture layer"),
    }
    assert_eq!(
        t.brush_settings().texture_kind,
        brush_kind_before,
        "the edit routed to the LAYER, leaving the brush texture untouched"
    );

    // A per-pattern param edit also re-renders live (Checker defaults to hard Softness 0.0; push it
    // fully soft so the edge pixels change).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_PARAMS[2],
        1.0,
    ));
    let buf_soft = t.canvas_rgba.as_ref().clone();
    assert_ne!(
        buf_checker, buf_soft,
        "editing a per-pattern param re-rendered the layer"
    );

    // A standard layer feature works on a Texture layer: hiding it drops it from the composite,
    // leaving only the opaque white base.
    t.set_layer_visible(id, true);
    t.set_layer_visible(id, false);
    let (hidden, _, _) = t.run_full();
    assert!(
        hidden
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[0] == 255 && p[1] == 255 && p[2] == 255 && p[3] == 255),
        "hiding the texture layer reveals the white base"
    );
    t.set_layer_visible(id, true);
}

#[test]
fn texture_layer_size_and_offset_panel_events_are_real_valued_and_clamp() {
    // Regression (Enio 2026-06-25): the Layers texture-layer editor uses the SAME drag-scrub number
    // fields as the Brush panel — which emit the REAL value — but routed Size/Offset through
    // normalized (`0..1`) setters. So Size 1.0 mapped to TEX_SIZE_MAX (10.0) and any value < 1 to
    // `0.1 + v*9.9` (e.g. 0.1 → 1.09). The layer must store the real value, clamped to the real range,
    // exactly like the brush's `set_brush_texture_size` / `set_brush_texture_offset`.
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::{TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN};

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    t.toggle_dock(); // texture-layer editing lives in the Layers view (dock now opens on Brush)
    let id = t.add_texture_layer().expect("texture layer added");
    let size = |t: &PainterTool, axis: usize| match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.size[axis],
        _ => panic!("texture layer"),
    };
    let offset = |t: &PainterTool, axis: usize| match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.offset[axis],
        _ => panic!("texture layer"),
    };

    // Size: the headline bug — 1.0 must stay 1.0 (used to jump to 10.0), and a sub-1 value stays
    // itself (used to become `0.1 + v*9.9`).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        1.0,
    ));
    assert!(
        (size(&t, 0) - 1.0).abs() < 1e-6,
        "Size 1.0 stays 1.0, got {}",
        size(&t, 0)
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        0.5,
    ));
    assert!(
        (size(&t, 1) - 0.5).abs() < 1e-6,
        "Size 0.5 stays 0.5, got {}",
        size(&t, 1)
    );
    // Size clamps to the real bounds (not the normalized track).
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        999.0,
    ));
    assert!((size(&t, 0) - TEX_SIZE_MAX).abs() < 1e-6);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        -5.0,
    ));
    assert!((size(&t, 0) - TEX_SIZE_MIN).abs() < 1e-6);

    // Offset: real-valued + clamps to ±1 the same way.
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        -0.5,
    ));
    assert!(
        (offset(&t, 0) + 0.5).abs() < 1e-6,
        "Offset -0.5 stays -0.5, got {}",
        offset(&t, 0)
    );
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        5.0,
    ));
    assert!((offset(&t, 1) - TEX_OFFSET_MAX).abs() < 1e-6);
    t.handle_panel_event(PanelEvent::SetValue(
        core_ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        -5.0,
    ));
    assert!((offset(&t, 0) - TEX_OFFSET_MIN).abs() < 1e-6);
}

#[test]
fn texture_layer_compatible_with_duplicate_and_mask() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 32 * 32 * 4], 32, 32);
    let id = t.add_texture_layer().expect("texture layer added");

    // Duplicate (audit fix): a Texture layer duplicates like a raster.
    let dup = t.duplicate_layer(id).expect("texture layer duplicates");
    assert_ne!(dup, id);
    assert!(matches!(
        t.layers().get(dup).map(|l| &l.kind),
        Some(LayerKind::Texture(_))
    ));

    // Mask (audit fix): a Texture layer can take a grayscale mask (the dup is active after duplicate).
    let mask = t.add_mask_to_active().expect("texture layer takes a mask");
    assert_eq!(
        t.layers().get(dup).and_then(|l| l.mask),
        Some(mask),
        "the mask is attached to the texture layer"
    );
}

#[test]
fn brush_texture_section_not_hijacked_when_dock_shows_brush() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::TextureKind;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 16 * 16 * 4], 16, 16);
    let id = t.add_texture_layer().expect("texture layer added"); // active; dock now opens on Brush
    // The dock already shows the Brush view (default) with the texture layer active.
    // A Kind change in the Brush view must hit the BRUSH, not the active texture layer.
    let layer_kind_before = match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => tex.kind,
        _ => panic!("expected a texture layer"),
    };
    t.handle_panel_event(PanelEvent::SelectOption(
        core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        TextureKind::Voronoi.to_u8().to_string(),
    ));
    assert_eq!(
        t.brush_settings().texture_kind,
        TextureKind::Voronoi.to_u8(),
        "the Brush view's Kind edit reaches the brush"
    );
    match t.layers().get(id).map(|l| &l.kind) {
        Some(LayerKind::Texture(tex)) => assert_eq!(
            tex.kind, layer_kind_before,
            "the texture layer is untouched while the Brush view is showing"
        ),
        _ => panic!("expected a texture layer"),
    }
}

// ── Seamless Tiling (wrap-around painting) ───────────────────────────────────────────────────

#[test]
fn tiling_x_wraps_paint_across_the_sprite_edge_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    // Enable Tiling X via the panel (the wiring proof — a dropped Click would leave it off).
    let mut t = white_canvas(64, 6.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_TILING_X));
    assert_eq!(
        t.brush_tiling(),
        [true, false],
        "Tiling X toggle reached the tool"
    );
    // A single dab at the RIGHT edge (x=63). With Tiling X it also paints the wrapped copy that
    // crosses onto the LEFT edge (x=0) — so a stroke over the border is seamless when tiled.
    t.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 64, 63, 32),
        [0, 0, 0, 255],
        "the dab painted the right edge"
    );
    assert_eq!(
        px(&t, 64, 0, 32),
        [0, 0, 0, 255],
        "Tiling X wrapped it onto the left edge"
    );
    // Only X tiles: the top-left corner stays white (no vertical wrap).
    assert_eq!(
        px(&t, 64, 0, 0),
        [255, 255, 255, 255],
        "Tiling Y off ⟹ no vertical wrap"
    );

    // Control: without tiling the same edge dab does NOT appear on the opposite edge.
    let mut t0 = white_canvas(64, 6.0);
    t0.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Down));
    t0.on_canvas_pointer(cp([63.0, 32.0], PointerPhase::Up));
    assert_eq!(
        px(&t0, 64, 0, 32),
        [255, 255, 255, 255],
        "no Tiling ⟹ the left edge is untouched"
    );
}

#[test]
fn repeat_image_toggle_reaches_the_tool_e2e() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;

    let mut t = PainterTool::default();
    assert!(!t.repeat_image(), "off by default");
    // Toggle Repeat Image via the panel (wiring proof — a dropped Click would leave it off).
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_REPEAT_IMAGE));
    assert!(t.repeat_image(), "Repeat Image toggle reached the tool");
    assert!(
        t.brush_settings().repeat_image,
        "snapshot mirrors it for the panel"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_REPEAT_IMAGE));
    assert!(!t.repeat_image(), "toggles back off");
}
