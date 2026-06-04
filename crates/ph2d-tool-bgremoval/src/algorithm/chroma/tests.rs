use super::*;

/// Helper: build an RGBA8 buffer of size `w × h` with a solid
/// background colour and an optional inner rectangle of a
/// foreground colour.
fn make_image(w: u32, h: u32, bg: [u8; 3], fg: Option<([u8; 3], u32, u32, u32, u32)>) -> Vec<u8> {
    let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h {
        for x in 0..w {
            let i = ((y as usize) * (w as usize) + x as usize) * 4;
            rgba[i] = bg[0];
            rgba[i + 1] = bg[1];
            rgba[i + 2] = bg[2];
            rgba[i + 3] = 255;
        }
    }
    if let Some((c, fx, fy, fw, fh)) = fg {
        for y in fy..(fy + fh).min(h) {
            for x in fx..(fx + fw).min(w) {
                let i = ((y as usize) * (w as usize) + x as usize) * 4;
                rgba[i] = c[0];
                rgba[i + 1] = c[1];
                rgba[i + 2] = c[2];
            }
        }
    }
    rgba
}

fn default_params() -> ChromaParams {
    ChromaParams::default()
}

#[test]
fn srgb_to_oklab_white_is_near_unit_luma() {
    let lab = srgb_to_oklab(255, 255, 255);
    // Oklab L for pure white ≈ 1.0 (paper).
    assert!((lab[0] - 1.0).abs() < 0.01, "white L = {}", lab[0]);
    assert!(lab[1].abs() < 0.01);
    assert!(lab[2].abs() < 0.01);
}

#[test]
fn srgb_to_oklab_black_is_zero() {
    let lab = srgb_to_oklab(0, 0, 0);
    assert!(lab[0].abs() < 1e-3);
    assert!(lab[1].abs() < 1e-3);
    assert!(lab[2].abs() < 1e-3);
}

#[test]
fn srgb_to_oklab_red_has_positive_a() {
    let lab = srgb_to_oklab(255, 0, 0);
    assert!(lab[1] > 0.1, "red a* should be positive, got {}", lab[1]);
}

#[test]
fn solid_bg_with_no_subject_is_entirely_background() {
    let rgba = make_image(32, 32, [0, 200, 0], None);
    let mut s = BgRemovalScratch::default();
    s.ensure(32, 32, false);
    let r = segment(&rgba, 32, 32, &default_params(), &[], &mut s);
    let SegmentResult::Chroma { bg_oklab } = r;
    // bg detected — the centroid should be very close to green oklab.
    let green = srgb_to_oklab(0, 200, 0);
    assert!(oklab_dist_sq(bg_oklab, green) < 0.01);
    let mask = &s.mask[..32 * 32];
    let bg_count = mask.iter().filter(|&&v| v == 0).count();
    assert!(
        bg_count > 32 * 32 * 99 / 100,
        "≥99% should be bg, got {bg_count}/{} ",
        32 * 32
    );
}

#[test]
fn subject_on_uniform_bg_is_preserved() {
    // 32×32 green bg, 8×8 red subject in the middle.
    let rgba = make_image(32, 32, [0, 200, 0], Some(([200, 30, 30], 12, 12, 8, 8)));
    let mut s = BgRemovalScratch::default();
    s.ensure(32, 32, false);
    let _ = segment(&rgba, 32, 32, &default_params(), &[], &mut s);

    // Subject pixels (12..20 × 12..20) should be fg.
    for y in 12..20 {
        for x in 12..20 {
            let i = (y as usize) * 32 + x as usize;
            assert_eq!(s.mask[i], 255, "subject pixel ({x},{y}) should be fg");
        }
    }
    // Border pixels should be bg.
    for x in 0..32 {
        let top = x as usize;
        let bot = 31 * 32 + x as usize;
        assert_eq!(s.mask[top], 0, "top row pixel {x} should be bg");
        assert_eq!(s.mask[bot], 0, "bottom row pixel {x} should be bg");
    }
}

#[test]
fn interior_bg_color_pocket_is_removed() {
    // 32×32 green bg, 12×12 red ring with an interior green
    // patch (an enclosed bg-coloured "hole" — like the gap between
    // a character's arm and torso). It must be removed: the border
    // flood can't reach it, but the interior-pocket sweep does
    // (hard bg, ΔE² ≤ tol²). Was previously protected as fg, which
    // left bg colour stuck inside the drawing.
    let mut rgba = make_image(32, 32, [0, 200, 0], Some(([200, 30, 30], 10, 10, 12, 12)));
    // Punch a 2×2 green hole at (15,15) inside the red ring.
    for y in 15..17 {
        for x in 15..17 {
            let i = ((y as usize) * 32 + x as usize) * 4;
            rgba[i] = 0;
            rgba[i + 1] = 200;
            rgba[i + 2] = 0;
        }
    }

    let mut s = BgRemovalScratch::default();
    s.ensure(32, 32, false);
    let _ = segment(&rgba, 32, 32, &default_params(), &[], &mut s);

    // The 2×2 interior green patch is bg-coloured → removed.
    for y in 15..17 {
        for x in 15..17 {
            let i = (y as usize) * 32 + x as usize;
            assert_eq!(
                s.mask[i], 0,
                "interior bg-colour pocket ({x},{y}) should be removed"
            );
        }
    }
    // The red ring (subject) stays fg.
    let ring_i = 11 * 32 + 11;
    assert_eq!(s.mask[ring_i], 255, "red ring pixel should stay fg");
    // Exterior green is still bg.
    let edge_i = 0;
    assert_eq!(s.mask[edge_i], 0, "exterior bg pixel should be bg");
}

#[test]
fn reference_color_override_skips_corner_detection() {
    // Make an image where 3 corners are red and 1 is green; if
    // we let corner-auto run, it picks the majority (red). But
    // we override with green, so the green corner should be the
    // one masked.
    let mut rgba = vec![255u8; 32 * 32 * 4]; // all opaque white
    for y in 0..32 {
        for x in 0..32 {
            let i = ((y as usize) * 32 + x as usize) * 4;
            let is_tl = x < 16 && y < 16;
            let (r, g, b) = if is_tl { (0, 200, 0) } else { (200, 0, 0) };
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
        }
    }
    let mut p = default_params();
    // Override: target the *green* corner as bg.
    p.reference_color = Some([0, 200, 0]);
    // Disable flood so we see a pure threshold result.
    p.use_flood = false;
    let mut s = BgRemovalScratch::default();
    s.ensure(32, 32, false);
    let _ = segment(&rgba, 32, 32, &p, &[], &mut s);

    // TL (green) should be bg; BR (red) should be fg.
    let tl = 0;
    let br = 31 * 32 + 31;
    assert_eq!(s.mask[tl], 0, "TL (green-overridden bg) should be bg");
    assert_eq!(s.mask[br], 255, "BR (red, not bg) should be fg");
}

#[test]
fn delta_e_sq_is_zero_on_bg_and_positive_elsewhere() {
    let rgba = make_image(16, 16, [0, 200, 0], Some(([200, 0, 0], 6, 6, 4, 4)));
    let mut s = BgRemovalScratch::default();
    s.ensure(16, 16, false);
    let _ = segment(&rgba, 16, 16, &default_params(), &[], &mut s);

    // A border pixel should have ΔE² ≈ 0.
    assert!(
        s.delta_e[0] < 0.001,
        "border ΔE² should be ~0, got {}",
        s.delta_e[0]
    );
    // The very centre (subject) should have ΔE² >> 0.
    let centre = 7 * 16 + 7;
    assert!(
        s.delta_e[centre] > 0.05,
        "subject ΔE² should be >0.05, got {}",
        s.delta_e[centre]
    );
}

#[test]
fn border_bg_confidence_drops_when_subject_touches_edge() {
    // 32×32 green bg, but the subject (red) is glued to the
    // entire top row. Border-bg-confidence < 60%, so flood
    // should be auto-disabled and the threshold-only path runs
    // (interior subject still ends up fg). Image must clear
    // MIN_BIG_DIM so corner sampling sees 3 green corners and
    // 1 red corner — k-means picks green as the majority bg.
    let rgba = make_image(64, 64, [0, 200, 0], Some(([200, 30, 30], 0, 0, 64, 24)));

    let mut p = default_params();
    p.tolerance = 0.10;
    let mut s = BgRemovalScratch::default();
    s.ensure(64, 64, false);
    let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

    // Top-left pixel is red → fg.
    assert_eq!(
        s.mask[0], 255,
        "top-left red pixel should be fg under fallback"
    );
    // Bottom-left pixel is green → bg.
    let bl = 63 * 64;
    assert_eq!(
        s.mask[bl], 0,
        "bottom-left green pixel should be bg under fallback"
    );
}

#[test]
fn empty_image_returns_default_bg_no_panic() {
    let mut s = BgRemovalScratch::default();
    s.ensure(0, 0, false);
    let r = segment(&[], 0, 0, &default_params(), &[], &mut s);
    let SegmentResult::Chroma { bg_oklab } = r;
    assert_eq!(bg_oklab, [0.0, 0.0, 0.0]);
}

#[test]
fn determinism_same_input_produces_same_mask() {
    let rgba = make_image(48, 48, [10, 180, 30], Some(([220, 40, 40], 18, 18, 12, 12)));

    let mut s1 = BgRemovalScratch::default();
    s1.ensure(48, 48, false);
    let r1 = segment(&rgba, 48, 48, &default_params(), &[], &mut s1);

    let mut s2 = BgRemovalScratch::default();
    s2.ensure(48, 48, false);
    let r2 = segment(&rgba, 48, 48, &default_params(), &[], &mut s2);

    assert_eq!(s1.mask, s2.mask);
    let (SegmentResult::Chroma { bg_oklab: a }, SegmentResult::Chroma { bg_oklab: b }) = (r1, r2);
    assert_eq!(a, b);
}

#[test]
fn scratch_is_reusable_across_calls() {
    let rgba_a = make_image(24, 24, [200, 0, 0], None);
    let rgba_b = make_image(24, 24, [0, 0, 200], None);

    let mut s = BgRemovalScratch::default();
    s.ensure(24, 24, false);

    let _ = segment(&rgba_a, 24, 24, &default_params(), &[], &mut s);
    let bg_count_a = s.mask.iter().filter(|&&v| v == 0).count();

    // No re-ensure between calls — reuse same allocation.
    let _ = segment(&rgba_b, 24, 24, &default_params(), &[], &mut s);
    let bg_count_b = s.mask.iter().filter(|&&v| v == 0).count();

    // Both runs should produce mostly-bg masks (different bg
    // colours but each input is uniform).
    assert!(bg_count_a > 24 * 24 * 99 / 100);
    assert!(bg_count_b > 24 * 24 * 99 / 100);
}

// --- Audit-driven coverage gap tests ---------------------------------

#[test]
fn transparent_border_does_not_poison_corner_bg_detection() {
    // 64×64 light-grey opaque bg with 4-px transparent corners
    // (alpha-rounded sprite) and a 16×16 opaque red subject in
    // the middle. Without the alpha-aware skip, the transparent
    // corner pixels' garbage RGB would poison k-means; with it,
    // k-means samples the actual light-grey bg.
    let dim = 64usize;
    let mut rgba = vec![0u8; dim * dim * 4];
    for i in 0..(dim * dim) {
        let b = i * 4;
        rgba[b] = 200;
        rgba[b + 1] = 200;
        rgba[b + 2] = 200;
        rgba[b + 3] = 255;
    }
    // Punch 4×4 transparent corners.
    for cy in 0..4 {
        for cx in 0..4 {
            for &(x, y) in &[
                (cx, cy),
                (dim - 1 - cx, cy),
                (cx, dim - 1 - cy),
                (dim - 1 - cx, dim - 1 - cy),
            ] {
                rgba[(y * dim + x) * 4 + 3] = 0;
            }
        }
    }
    // 16×16 red subject in the middle.
    for y in 24..40 {
        for x in 24..40 {
            let b = (y * dim + x) * 4;
            rgba[b] = 220;
            rgba[b + 1] = 30;
            rgba[b + 2] = 30;
            rgba[b + 3] = 255;
        }
    }
    let mut s = BgRemovalScratch::default();
    s.ensure(dim as u32, dim as u32, false);
    let _ = segment(
        &rgba,
        dim as u32,
        dim as u32,
        &default_params(),
        &[],
        &mut s,
    );

    // Centre red subject must be fg.
    let centre = 32 * dim + 32;
    assert_eq!(s.mask[centre], 255, "opaque red centre must be fg");
    // Off-centre grey bg pixel must be bg.
    let bg_pixel = 10 * dim + 30;
    assert_eq!(s.mask[bg_pixel], 0, "light-grey bg pixel must be bg");
    // Transparent corner pixel must be bg (alpha-driven via
    // delta_e=0 forced for `a == 0`).
    assert_eq!(s.mask[0], 0, "transparent corner must be bg");
}

#[test]
fn nan_tolerance_does_not_make_everything_foreground() {
    // Without the NaN sanitisation, `delta_e <= NaN` is always
    // false and every pixel would be marked fg silently. Image
    // size must clear MIN_BIG_DIM so corner sampling sees the
    // bg colour.
    let rgba = make_image(64, 64, [0, 200, 0], None);
    let mut p = default_params();
    p.tolerance = f32::NAN;
    let mut s = BgRemovalScratch::default();
    s.ensure(64, 64, false);
    let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

    // Sanitised → tol_sq = 0 + FP_NOISE_FLOOR → uniform bg
    // (exact match within FP noise) all classify as bg.
    let bg = s.mask.iter().filter(|&&v| v == 0).count();
    assert_eq!(
        bg,
        64 * 64,
        "NaN tolerance must collapse to 0, classifying uniform bg as bg"
    );
}

#[test]
fn tolerance_zero_only_marks_exact_match_as_bg() {
    // Pure red 64×64 with a single off-by-one pixel (R=254 vs
    // 255). With tolerance=0 the off-by-one pixel falls outside
    // the threshold → fg; the corner pixel matches exactly
    // within FP noise → bg. Image must clear MIN_BIG_DIM.
    let mut rgba = make_image(64, 64, [255, 0, 0], None);
    let probe = (5 * 64 + 5) * 4;
    rgba[probe] = 254; // R 255→254
    let mut p = default_params();
    p.tolerance = 0.0;
    // Disable flood so the probe pixel is classified on its
    // own colour, not via flood-propagation from neighbours.
    p.use_flood = false;
    let mut s = BgRemovalScratch::default();
    s.ensure(64, 64, false);
    let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

    assert_eq!(
        s.mask[5 * 64 + 5],
        255,
        "off-by-one pixel must be fg at tol=0"
    );
    assert_eq!(s.mask[0], 0, "exact-match corner pixel must be bg at tol=0");
}

#[test]
fn very_tall_image_falls_back_to_small_path_without_panic() {
    // 4 × 200 image — width below MIN_BIG_DIM. The sample path
    // must take the small-image fallback (no panic, sensible
    // mask).
    let rgba = make_image(4, 200, [50, 150, 250], None);
    let mut s = BgRemovalScratch::default();
    s.ensure(4, 200, false);
    let _ = segment(&rgba, 4, 200, &default_params(), &[], &mut s);
    // Uniform input → entire mask should be bg.
    let bg = s.mask.iter().filter(|&&v| v == 0).count();
    assert!(
        bg > 4 * 200 * 95 / 100,
        "≥95% should be bg, got {bg}/{}",
        4 * 200
    );
}

#[test]
fn min_big_dim_boundary_does_not_overflow_or_overlap() {
    // Exactly MIN_BIG_DIM (52×52) is the first size that takes
    // the big-image dispersion path. The top-left and top-right
    // patches must sample disjoint columns by construction.
    // Functional check: a uniform image yields a fully-bg mask
    // and no panic.
    let dim = MIN_BIG_DIM;
    let rgba = make_image(dim, dim, [100, 100, 100], None);
    let mut s = BgRemovalScratch::default();
    s.ensure(dim, dim, false);
    let _ = segment(&rgba, dim, dim, &default_params(), &[], &mut s);
    let n = (dim * dim) as usize;
    let bg = s.mask.iter().filter(|&&v| v == 0).count();
    assert_eq!(bg, n, "uniform MIN_BIG_DIM image must be entirely bg");

    // One below — falls into small-image path.
    let small = dim - 1;
    let rgba2 = make_image(small, small, [100, 100, 100], None);
    let mut s2 = BgRemovalScratch::default();
    s2.ensure(small, small, false);
    let _ = segment(&rgba2, small, small, &default_params(), &[], &mut s2);
    let n2 = (small * small) as usize;
    let bg2 = s2.mask.iter().filter(|&&v| v == 0).count();
    assert_eq!(bg2, n2, "uniform small-fallback image must be entirely bg");
}

#[test]
fn extra_color_masks_a_second_background_region() {
    // 64×64 image: green bg (auto-detected from the corners) with a
    // BLUE 16×16 block in the centre. Blue is far from green, so
    // with auto-only the blue block stays foreground. Adding blue as
    // an extra background colour must knock it out — without
    // disturbing the green-bg classification. Disable flood so each
    // pixel is judged on its own colour (the blue block is interior,
    // not border-connected).
    let blue = [30u8, 30, 220];
    let rgba = make_image(64, 64, [0, 200, 0], Some((blue, 24, 24, 16, 16)));
    let mut p = default_params();
    p.use_flood = false;
    let centre = 32 * 64 + 32;

    // Auto-only: blue centre is foreground.
    let mut s = BgRemovalScratch::default();
    s.ensure(64, 64, false);
    let _ = segment(&rgba, 64, 64, &p, &[], &mut s);
    assert_eq!(
        s.mask[centre], 255,
        "without the extra colour the blue block must stay foreground"
    );

    // With blue as an extra bg colour: the blue centre is removed.
    let mut s2 = BgRemovalScratch::default();
    s2.ensure(64, 64, false);
    let _ = segment(&rgba, 64, 64, &p, &[blue], &mut s2);
    assert_eq!(
        s2.mask[centre], 0,
        "the extra blue colour must mask the blue block as background"
    );
    // The green border is still background.
    assert_eq!(s2.mask[0], 0, "green border must remain background");
}

#[test]
fn extra_color_does_not_cascade_via_flood_into_subject() {
    // Regression for Enio 2026-05-26 "adição de novas cores através
    // do pick colors provoca o desaparecimento de toda imagem".
    //
    // Setup: 80×80 image, green bg (auto-detected at corners), with a
    // SOLID RED 40×40 block in the centre (the "subject"). The user
    // picks a *single* extra colour that happens to be SIMILAR (but
    // not identical) to a different region of the bg — say a slightly
    // brighter green halo. Before the fix, the extras-min-fold
    // lowered ΔE² everywhere along that hue band, `border_bg_confidence`
    // shot above the gate, the flood fired and propagated inward
    // through the band, and the entire red block got swallowed.
    //
    // After the fix: extras are an isolated pixel-wise predicate
    // applied AFTER the flood, so a single distant pick cannot bridge
    // through the subject. The red block must remain foreground.
    let red = [220u8, 30, 30];
    let rgba = make_image(80, 80, [0, 200, 0], Some((red, 20, 20, 40, 40)));
    // Pick a green CLOSE TO the auto bg (within typical tolerance).
    // This is the worst case for the old code: the extras fold would
    // collapse delta_e across the whole green sea, and the flood
    // would walk happily over any green-adjacent pixel.
    let extra_green = [40u8, 220, 30];
    let mut s = BgRemovalScratch::default();
    s.ensure(80, 80, false);
    // Use_flood ON — this is the path the old bug exploded under.
    let _ = segment(&rgba, 80, 80, &default_params(), &[extra_green], &mut s);

    // The centre of the red block MUST still be foreground.
    let centre = 40 * 80 + 40;
    assert_eq!(
        s.mask[centre], 255,
        "subject must survive when a SINGLE extra colour is added \
             (regression: flood used to bridge through any pixel within \
             tol of the extra, devouring the foreground)"
    );
    // And the corner is still bg (sanity).
    assert_eq!(s.mask[0], 0, "green border still bg");
    // The red block as a whole must be foreground (count fg pixels in
    // the 40×40 block — at least 95% should survive; a few edge
    // pixels can be soft-band).
    let mut fg_in_block = 0;
    for y in 20..60 {
        for x in 20..60 {
            if s.mask[y * 80 + x] != 0 {
                fg_in_block += 1;
            }
        }
    }
    assert!(
        fg_in_block >= 40 * 40 * 95 / 100,
        "≥95% of the red block must survive, got {fg_in_block}/{}",
        40 * 40
    );
}

#[test]
fn extras_cache_pixels_oklab_is_populated_for_non_transparent() {
    // Sanity check on the new `pixels_oklab` cache so a future
    // refactor can't silently break the `fill_delta_e_sq` →
    // extras-loop handoff.
    let rgba = make_image(8, 8, [120, 80, 200], None);
    let mut s = BgRemovalScratch::default();
    s.ensure(8, 8, false);
    let _ = segment(&rgba, 8, 8, &default_params(), &[], &mut s);
    // All pixels are non-transparent → all should have a populated
    // OkLab cache (not the all-zero sentinel).
    let any_zero = s
        .pixels_oklab
        .iter()
        .any(|&[l, a, b]| l == 0.0 && a == 0.0 && b == 0.0);
    assert!(
        !any_zero,
        "fill_delta_e_sq must populate pixels_oklab for every \
             non-transparent pixel (else the extras loop re-converts)"
    );
}
