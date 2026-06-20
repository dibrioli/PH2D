use super::*;

// ── sRGB transfer LUTs (PERF, byte/bit-exact) ─────────────────────────

#[test]
fn decode_lut_is_bit_exact_with_srgb_to_linear_byte() {
    // The decode LUT must be the SAME bits as the powf path for every byte
    // (the GPU compositor's `.to_bits()` gate decodes identically).
    for b in 0u16..=255 {
        let b = b as u8;
        assert_eq!(
            SRGB_DECODE_LUT[b as usize].to_bits(),
            srgb_to_linear_byte(b).to_bits(),
            "decode LUT drifted at byte {b}"
        );
    }
}

#[test]
fn encode_via_threshold_matches_linear_to_srgb_byte() {
    use ph2d_color::srgb::linear_to_srgb_byte;
    // The LUT encode must produce the SAME byte as the powf round-to-nearest
    // across a dense linear sweep (real pixel data) — byte-exact, no powf.
    let thresh = &*SRGB_ENCODE_THRESH;
    let coarse = &*SRGB_ENCODE_COARSE;
    for i in 0..=300_000u32 {
        let v = i as f32 / 300_000.0;
        assert_eq!(
            encode_byte(thresh, coarse, v),
            linear_to_srgb_byte(v),
            "encode mismatch at v={v}"
        );
    }
    // Endpoints + out-of-range clamp.
    assert_eq!(encode_byte(thresh, coarse, 0.0), linear_to_srgb_byte(0.0));
    assert_eq!(encode_byte(thresh, coarse, 1.0), linear_to_srgb_byte(1.0));
    assert_eq!(encode_byte(thresh, coarse, -1.0), 0);
    assert_eq!(encode_byte(thresh, coarse, 2.0), 255);
}

#[test]
fn decode_then_encode_round_trips_every_byte() {
    // A pixel that is only decoded + re-encoded (no blend) must survive
    // unchanged for every byte — proves the two LUTs are mutual inverses.
    let thresh = &*SRGB_ENCODE_THRESH;
    let coarse = &*SRGB_ENCODE_COARSE;
    for b in 0u16..=255 {
        let b = b as u8;
        assert_eq!(
            encode_byte(thresh, coarse, SRGB_DECODE_LUT[b as usize]),
            b,
            "round-trip byte {b}"
        );
    }
}

fn solid(w: u32, h: u32, rgba: [u8; 4]) -> LayerImage {
    LayerImage {
        width: w,
        height: h,
        rgba8: rgba.repeat((w * h) as usize),
    }
}

#[test]
fn two_rasters_normal_top_over_bottom() {
    // SMOKE-INTRA Day 4: top layer (opaque) fully covers bottom.
    let (w, h) = (2, 2);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [200, 50, 50, 255]));
    src.insert(top, solid(w, h, [50, 50, 200, 255]));
    let out = composite(&s, &src, w, h);
    // Every pixel == top color (opaque over).
    for px in out.chunks_exact(4) {
        assert_eq!(&px[0..3], &[50, 50, 200], "expected top color");
        assert_eq!(px[3], 255);
    }
}

#[test]
fn transparent_top_reveals_bottom() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [200, 50, 50, 255]));
    src.insert(top, solid(w, h, [50, 50, 200, 0])); // alpha 0
    let out = composite(&s, &src, w, h);
    assert_eq!(&out[0..3], &[200, 50, 50]);
    assert_eq!(out[3], 255);
}

#[test]
fn invisible_layer_is_skipped() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.set_visible(top, false);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [10, 20, 30, 255]));
    src.insert(top, solid(w, h, [200, 200, 200, 255]));
    let out = composite(&s, &src, w, h);
    assert_eq!(&out[0..3], &[10, 20, 30]);
}

#[test]
fn opacity_half_blends_toward_bottom() {
    // 50% white over black → mid gray (in *linear*, then sRGB-encoded
    // → ~188, NOT 128). Verifies opacity folds into alpha + linear blend.
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.set_opacity(top, 0.5);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
    src.insert(top, solid(w, h, [255, 255, 255, 255]));
    let out = composite(&s, &src, w, h);
    // linear 0.5 → sRGB ≈ 188.
    assert!(
        (out[0] as i32 - 188).abs() <= 1,
        "expected ~188 (linear-space half), got {}",
        out[0]
    );
    assert_eq!(out[3], 255);
}

#[test]
fn multiply_blend_darkens() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.set_blend_mode(top, BlendMode::Multiply);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [180, 180, 180, 255]));
    src.insert(top, solid(w, h, [180, 180, 180, 255]));
    let out = composite(&s, &src, w, h);
    // Multiply of equal mid-grays is strictly darker than either input.
    assert!(out[0] < 180, "multiply should darken, got {}", out[0]);
}

#[test]
fn group_composites_children_then_blends() {
    // A group holding one opaque raster, group opacity 1.0 → same as
    // the raster composited directly.
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let child = s.add_raster("child", w, h).unwrap();
    let g = s.add_group("group").unwrap();
    s.move_into_group(child, g);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
    src.insert(child, solid(w, h, [120, 60, 200, 255]));
    let out = composite(&s, &src, w, h);
    assert_eq!(&out[0..3], &[120, 60, 200]);
}

#[test]
fn group_opacity_attenuates_the_stack() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let child = s.add_raster("child", w, h).unwrap();
    let g = s.add_group("group").unwrap();
    s.move_into_group(child, g);
    s.set_opacity(g, 0.5);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
    src.insert(child, solid(w, h, [255, 255, 255, 255]));
    let out = composite(&s, &src, w, h);
    assert!(
        (out[0] as i32 - 188).abs() <= 1,
        "group 50% → ~188, got {}",
        out[0]
    );
}

#[test]
fn mask_black_hides_parent_reveals_below() {
    // T3.5: a black mask on the top raster fully hides it → the layer below
    // shows through (mask multiplies the parent's alpha to 0).
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mask = s.add_mask(top).unwrap();
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 200, 0, 255])); // green below
    src.insert(top, solid(w, h, [200, 0, 0, 255])); // red on top
    src.insert(mask, solid(w, h, [0, 0, 0, 255])); // black → hide top
    let out = composite(&s, &src, w, h);
    assert_eq!(
        &out[0..3],
        &[0, 200, 0],
        "black mask hides top → green shows"
    );
}

#[test]
fn mask_white_keeps_parent_fully_visible() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mask = s.add_mask(top).unwrap();
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 200, 0, 255]));
    src.insert(top, solid(w, h, [200, 0, 0, 255]));
    src.insert(mask, solid(w, h, [255, 255, 255, 255])); // white → full visible
    let out = composite(&s, &src, w, h);
    assert_eq!(&out[0..3], &[200, 0, 0], "white mask keeps top visible");
}

#[test]
fn mask_inverted_flips_black_to_visible() {
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mask = s.add_mask(top).unwrap();
    s.set_mask_inverted(mask, true);
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 200, 0, 255]));
    src.insert(top, solid(w, h, [200, 0, 0, 255]));
    src.insert(mask, solid(w, h, [0, 0, 0, 255])); // black, but inverted → visible
    let out = composite(&s, &src, w, h);
    assert_eq!(
        &out[0..3],
        &[200, 0, 0],
        "inverted black mask → top visible"
    );
}

#[test]
fn mask_gray_is_partial_visibility() {
    // A mid-gray mask (~50%) partially reveals: result is between top and
    // bottom. Use Normal blend; assert the red channel lands strictly between
    // the fully-hidden (green's 0 red) and fully-shown (top's 200) extremes.
    let (w, h) = (1, 1);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    let mask = s.add_mask(top).unwrap();
    let mut src = MapPixelSource::default();
    src.insert(bottom, solid(w, h, [0, 0, 0, 255])); // black below
    src.insert(top, solid(w, h, [255, 0, 0, 255])); // red on top
    src.insert(mask, solid(w, h, [128, 128, 128, 255])); // ~50% visible
    let out = composite(&s, &src, w, h);
    assert!(
        out[0] > 0 && out[0] < 255,
        "gray mask → partial red, got {}",
        out[0]
    );
}

#[test]
fn clipping_layer_paints_only_where_base_is_opaque() {
    // T3.6: a clipping raster is masked by the base's straight alpha —
    // visible only where the base below it is opaque.
    let (w, h) = (2, 1);
    let mut s = LayerStack::new();
    let base = s.add_raster("base", w, h).unwrap();
    let clip = s.add_raster("clip", w, h).unwrap();
    s.get_mut(clip).unwrap().clipping = true;
    let mut src = MapPixelSource::default();
    let mut bimg = LayerImage::transparent(w, h);
    bimg.rgba8[0..4].copy_from_slice(&[0, 200, 0, 255]); // left opaque green
    bimg.rgba8[4..8].copy_from_slice(&[0, 0, 0, 0]); // right transparent
    src.insert(base, bimg);
    src.insert(clip, solid(w, h, [200, 0, 0, 255])); // opaque red everywhere
    let out = composite(&s, &src, w, h);
    assert_eq!(&out[0..4], &[200, 0, 0, 255], "clip shows over opaque base");
    assert_eq!(out[7], 0, "clip hidden where base is transparent");
}

#[test]
fn consecutive_clipping_layers_share_one_base() {
    // T3.6: two clipping layers chain to the SAME base (the first
    // non-clipping raster below), not to each other.
    let (w, h) = (2, 1);
    let mut s = LayerStack::new();
    let base = s.add_raster("base", w, h).unwrap();
    let c1 = s.add_raster("c1", w, h).unwrap();
    let c2 = s.add_raster("c2", w, h).unwrap();
    s.get_mut(c1).unwrap().clipping = true;
    s.get_mut(c2).unwrap().clipping = true;
    let mut src = MapPixelSource::default();
    let mut bimg = LayerImage::transparent(w, h);
    bimg.rgba8[0..4].copy_from_slice(&[10, 10, 10, 255]); // left opaque
    bimg.rgba8[4..8].copy_from_slice(&[0, 0, 0, 0]); // right transparent
    src.insert(base, bimg);
    src.insert(c1, solid(w, h, [0, 0, 0, 0])); // c1 transparent (no cover)
    src.insert(c2, solid(w, h, [200, 0, 0, 255])); // c2 opaque red
    let out = composite(&s, &src, w, h);
    // c2 visible on the left proves it clips to the BASE, not to (transparent) c1.
    assert_eq!(
        &out[0..3],
        &[200, 0, 0],
        "c2 clips to the base → red on left"
    );
    assert_eq!(out[7], 0, "c2 hidden where the base is transparent");
}

#[test]
fn dirty_rect_matches_full_recompose() {
    // layers_dirty_rect_correctness: recompositing a sub-rect equals
    // the same rect cropped from the full composite.
    let (w, h) = (4, 4);
    let mut s = LayerStack::new();
    let bottom = s.add_raster("bottom", w, h).unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.set_blend_mode(top, BlendMode::Screen);
    s.set_opacity(top, 0.7);
    let mut src = MapPixelSource::default();
    // Vary pixels so a region crop is a real test (not uniform).
    let mut b = LayerImage::transparent(w, h);
    let mut t = LayerImage::transparent(w, h);
    for i in 0..(w * h) as usize {
        b.rgba8[i * 4] = (i * 11 % 256) as u8;
        b.rgba8[i * 4 + 1] = (i * 7 % 256) as u8;
        b.rgba8[i * 4 + 2] = (i * 5 % 256) as u8;
        b.rgba8[i * 4 + 3] = 255;
        t.rgba8[i * 4] = (i * 13 % 256) as u8;
        t.rgba8[i * 4 + 1] = (i * 3 % 256) as u8;
        t.rgba8[i * 4 + 2] = (i * 17 % 256) as u8;
        t.rgba8[i * 4 + 3] = 200;
    }
    src.insert(bottom, b);
    src.insert(top, t);

    let full = composite(&s, &src, w, h);
    let region = Region {
        x: 1,
        y: 1,
        w: 2,
        h: 2,
    };
    let part = composite_region(&s, &src, w, h, region);
    for ly in 0..region.h {
        for lx in 0..region.w {
            let gx = region.x + lx;
            let gy = region.y + ly;
            let fi = ((gy * w + gx) * 4) as usize;
            let pi = ((ly * region.w + lx) * 4) as usize;
            assert_eq!(
                &full[fi..fi + 4],
                &part[pi..pi + 4],
                "dirty-rect pixel ({gx},{gy}) diverged from full recompose"
            );
        }
    }
}

#[test]
fn dirty_rect_matches_full_with_group_and_blend() {
    // audit W3 F2: exercise the GROUP sub-window recursion in the dirty-rect
    // path (the flat-raster test above doesn't). A nested group with a
    // blended child must crop bit-for-bit too.
    let (w, h) = (4, 4);
    let mut s = LayerStack::new();
    let base = s.add_raster("base", w, h).unwrap();
    let group = s.add_group("group").unwrap();
    let child = s.add_raster("child", w, h).unwrap();
    s.move_into_group(child, group);
    s.set_blend_mode(child, BlendMode::Multiply);
    s.set_opacity(child, 0.6);
    s.set_blend_mode(group, BlendMode::Screen);
    s.set_opacity(group, 0.8);
    let mut src = MapPixelSource::default();
    let mut bimg = LayerImage::transparent(w, h);
    let mut cimg = LayerImage::transparent(w, h);
    for i in 0..(w * h) as usize {
        bimg.rgba8[i * 4] = (i * 11 % 256) as u8;
        bimg.rgba8[i * 4 + 1] = (i * 7 % 256) as u8;
        bimg.rgba8[i * 4 + 2] = (i * 5 % 256) as u8;
        bimg.rgba8[i * 4 + 3] = 255;
        cimg.rgba8[i * 4] = (i * 13 % 256) as u8;
        cimg.rgba8[i * 4 + 1] = (i * 3 % 256) as u8;
        cimg.rgba8[i * 4 + 2] = (i * 17 % 256) as u8;
        cimg.rgba8[i * 4 + 3] = 200;
    }
    src.insert(base, bimg);
    src.insert(child, cimg);

    let full = composite(&s, &src, w, h);
    let region = Region {
        x: 1,
        y: 0,
        w: 2,
        h: 3,
    };
    let part = composite_region(&s, &src, w, h, region);
    for ly in 0..region.h {
        for lx in 0..region.w {
            let gx = region.x + lx;
            let gy = region.y + ly;
            let fi = ((gy * w + gx) * 4) as usize;
            let pi = ((ly * region.w + lx) * 4) as usize;
            assert_eq!(
                &full[fi..fi + 4],
                &part[pi..pi + 4],
                "group dirty-rect pixel ({gx},{gy}) diverged"
            );
        }
    }
}

#[test]
fn adjustment_layer_noop_stub_is_identity() {
    // W4 T4.2: an adjustment layer composites end-to-end (the no-op
    // `apply_adjustment` stub leaves the layers below unchanged) — verifies
    // the LayerKind::Adjustment arm runs + the path is wired. A real arm
    // goes live the next frame once the implementer fills the compute.
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    let (w, h) = (2, 2);
    let mut s = LayerStack::new();
    let base = s.add_raster("base", w, h).unwrap();
    let _adj = s
        .add_adjustment(AdjustmentKind::HueSaturationBrightness)
        .unwrap();
    let mut src = MapPixelSource::default();
    src.insert(base, solid(w, h, [120, 60, 200, 255]));
    let out = composite(&s, &src, w, h);
    for px in out.chunks_exact(4) {
        assert_eq!(
            &px[0..3],
            &[120, 60, 200],
            "no-op adjustment changed the pixel"
        );
        assert_eq!(px[3], 255);
    }
}

#[test]
fn compositor_cache_skeleton_round_trips() {
    let mut c = CompositorCache::new();
    assert!(c.dirty_rect().is_none());
    c.mark_dirty(Region {
        x: 0,
        y: 0,
        w: 4,
        h: 4,
    });
    assert_eq!(c.dirty_rect().map(|r| r.w), Some(4));
    c.invalidate_from(LayerId(1), &LayerStack::new()); // skeleton: clears
}

#[test]
#[ignore = "W4 soft perf gate (ADR-0045 §2.11): inline full-recompose is bandwidth-bound; W5 wires CompositorCache cut-points into composite + un-ignores (hard ≤1ms @4K)"]
fn adjustment_layer_recomposition_perf_4k() {
    // Budget: slider-drag recompose @ 4K, 10 adjustment layers ≤ 1 ms.
    // Fleshed when the CompositorCache cut-point lands in the hot path.
}

// ── W5 CompositorCache cut-point cache (ADR-0045 §2.7) ────────────────
/// A per-pixel-varied opaque raster so blends + adjustments are non-trivial
/// (a uniform fill would hide a slicing/ordering bug).
fn varied(w: u32, h: u32, seed: u32) -> LayerImage {
    let mut img = LayerImage::transparent(w, h);
    for i in 0..(w * h) as usize {
        let i = i as u32;
        img.rgba8[(i * 4) as usize] = ((i * 11 + seed * 3) % 256) as u8;
        img.rgba8[(i * 4 + 1) as usize] = ((i * 7 + seed * 5) % 256) as u8;
        img.rgba8[(i * 4 + 2) as usize] = ((i * 5 + seed * 13) % 256) as u8;
        img.rgba8[(i * 4 + 3) as usize] = 255;
    }
    img
}

fn bc(brightness: f32, contrast: f32) -> ph2d_painter_effects::adjustments::AdjustmentParams {
    use ph2d_painter_effects::adjustments::{AdjustmentParams, BrightnessContrastParams};
    AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
        brightness,
        contrast,
        legacy: false,
    })
}

#[test]
fn cache_matches_full_recompose() {
    // The cut-point cache MUST be bit-identical to the reference full
    // recompose — cold, and after a param-only change of either a lower or a
    // higher adjustment (the slider-drag hot path). Creation order is
    // bottom→top, so the panel root ends up
    // [top, adj_high, mid, adj_low, base] (index 0 = topmost).
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    let (w, h) = (4, 4);
    let mut s = LayerStack::new();
    let base = s.add_raster("base", w, h).unwrap();
    let adj_low = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    let mid = s.add_raster("mid", w, h).unwrap();
    let adj_high = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.adjustment_mut(adj_low).unwrap().params = bc(0.15, 0.10);
    s.adjustment_mut(adj_high).unwrap().params = bc(-0.20, 0.25);
    s.set_opacity(top, 0.6);
    s.set_blend_mode(top, BlendMode::Screen);
    let mut src = MapPixelSource::default();
    src.insert(base, varied(w, h, 1));
    src.insert(mid, varied(w, h, 2));
    src.insert(top, varied(w, h, 3));

    // Cold cache == full recompose (and populates both cuts).
    let mut cache = CompositorCache::new();
    let full = composite(&s, &src, w, h);
    let cold = composite_with_cache(&s, &src, w, h, &mut cache);
    assert_eq!(cold, full, "cold cache diverged from full recompose");

    // Param change on the LOWER adjustment: its cut (below it) stays valid,
    // the higher cut is dropped → restart from adj_low.
    s.adjustment_mut(adj_low).unwrap().params = bc(0.40, -0.10);
    cache.invalidate_above(adj_low, &s);
    let full_low = composite(&s, &src, w, h);
    let warm_low = composite_with_cache(&s, &src, w, h, &mut cache);
    assert_eq!(warm_low, full_low, "lower-adj param-change cache diverged");

    // Param change on the HIGHER adjustment: reuses the lower cut entirely
    // → restart from adj_high (fewest layers recomposed).
    s.adjustment_mut(adj_high).unwrap().params = bc(0.05, 0.50);
    cache.invalidate_above(adj_high, &s);
    let full_high = composite(&s, &src, w, h);
    let warm_high = composite_with_cache(&s, &src, w, h, &mut cache);
    assert_eq!(
        warm_high, full_high,
        "higher-adj param-change cache diverged"
    );
}

#[test]
fn cache_hit_skips_below_layers() {
    // The bandwidth win: a param-only change of an adjustment must NOT re-read
    // the layers below its cut (only the adjustment + layers above recompose).
    use ph2d_painter_effects::adjustments::AdjustmentKind;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// Counts `layer_rgba` reads per layer. Interior mutability keeps the
    /// trait's `&self`; the returned slice borrows `inner` (not the cell), so
    /// there is no borrow conflict.
    struct CountingSource<'a> {
        inner: &'a MapPixelSource,
        reads: RefCell<BTreeMap<LayerId, usize>>,
    }
    impl LayerPixelSource for CountingSource<'_> {
        fn layer_rgba(&self, id: LayerId) -> Option<&[u8]> {
            *self.reads.borrow_mut().entry(id).or_default() += 1;
            self.inner.layer_rgba(id)
        }
    }

    let (w, h) = (4, 4);
    let mut s = LayerStack::new();
    // root = [top, adj, mid, base] (index 0 = topmost).
    let base = s.add_raster("base", w, h).unwrap();
    let mid = s.add_raster("mid", w, h).unwrap();
    let adj = s
        .add_adjustment(AdjustmentKind::BrightnessContrast)
        .unwrap();
    let top = s.add_raster("top", w, h).unwrap();
    s.adjustment_mut(adj).unwrap().params = bc(0.2, 0.1);
    let mut inner = MapPixelSource::default();
    inner.insert(base, varied(w, h, 1));
    inner.insert(mid, varied(w, h, 2));
    inner.insert(top, varied(w, h, 3));
    let src = CountingSource {
        inner: &inner,
        reads: RefCell::new(BTreeMap::new()),
    };

    // Cold compose reads every raster ≥ once.
    let mut cache = CompositorCache::new();
    let _ = composite_with_cache(&s, &src, w, h, &mut cache);
    assert!(src.reads.borrow().get(&base).copied().unwrap_or(0) >= 1);
    assert!(src.reads.borrow().get(&mid).copied().unwrap_or(0) >= 1);
    assert!(src.reads.borrow().get(&top).copied().unwrap_or(0) >= 1);

    // Param-only change → cache hit restarts from the adjustment's cut; the
    // below-layers (base, mid) are not re-read, the above-layer (top) is.
    src.reads.borrow_mut().clear();
    s.adjustment_mut(adj).unwrap().params = bc(0.5, -0.2);
    cache.invalidate_above(adj, &s);
    let _ = composite_with_cache(&s, &src, w, h, &mut cache);
    let reads = src.reads.borrow();
    assert_eq!(
        reads.get(&base).copied().unwrap_or(0),
        0,
        "below layer `base` was re-read on a cache hit"
    );
    assert_eq!(
        reads.get(&mid).copied().unwrap_or(0),
        0,
        "below layer `mid` was re-read on a cache hit"
    );
    assert!(
        reads.get(&top).copied().unwrap_or(0) >= 1,
        "above layer `top` must recompose"
    );
}
