//! Resample math for [`super`]'s Merge-Sprites composite pass — the
//! inverse world→source-pixel mapping + the premultiplied bilinear
//! sampler. Split out of `sprite_merge.rs` (HR-18 file-LOC decomposition)
//! as a `#[path]` child module so `super::SrcRecord` (+ its private
//! fields) stays reachable. Pure CPU math: fully covered by the unit
//! tests at the bottom (no GPU smoke needed).

use super::SrcRecord;

/// World-px → source-image-px via inverse of the forward chain
/// `T * R * Ta * S * P_local` (matches `sprite_image_to_screen_affine`
/// minus the screen projection step). Returns `None` on degenerate
/// scale / size that would divide by zero.
///
/// Returns coordinates in PIXEL-INDEX convention — `img_x = i` lands
/// exactly on the centre of source pixel `i` (matches the GPU
/// `textureSample` convention where UV `(0.5/W, 0.5/H)` is the centre
/// of texel `(0, 0)`). The `- 0.5` half-pixel correction converts the
/// forward chain's edge-coordinate output (where `img_x = 0` is the
/// LEFT edge of pixel 0, not its centre) into the centre-aligned
/// coordinate the bilinear sampler expects. Without it, every output
/// pixel landed at fractional `img_x = N + 0.5` → bilinear blended
/// pixel `N` with pixel `N + 1` 50/50 → half-pixel blur everywhere,
/// even when the grid was perfectly snapped (Enio 2026-05-27 "ainda
/// borra imagem").
pub(super) fn world_to_image(wx: f32, wy: f32, src: &SrcRecord) -> (f32, f32) {
    let dx = wx - src.tx;
    let dy = wy - src.ty;
    // Inverse rotation = transpose for a pure rotation matrix. Uses
    // the pre-cached `cos_t` / `sin_t` (computed once per source at
    // read-time, not per-pixel).
    let post_anchor_x = dx * src.cos_t + dy * src.sin_t;
    let post_anchor_y = -dx * src.sin_t + dy * src.cos_t;
    let post_scale_x = post_anchor_x - src.anchor_x;
    let post_scale_y = post_anchor_y - src.anchor_y;
    // Degenerate scale / size are filtered at the read pass (audit
    // A4/A5), so the cached reciprocals are always finite here.
    let local_x = post_scale_x * src.inv_scale_x;
    let local_y = post_scale_y * src.inv_scale_y;
    let img_x = (local_x * src.inv_size_w + 0.5) * src.w as f32 - 0.5;
    let img_y = (0.5 - local_y * src.inv_size_h) * src.h as f32 - 0.5;
    (img_x, img_y)
}

/// Bilinear sample of an RGBA8 buffer in PREMULTIPLIED space — the
/// caller is responsible for handing in premul bytes (see Step 1's
/// `into_premultiplied`). Returns `None` when `(x, y)` lies outside
/// `[0, w-1] × [0, h-1]` — caller treats that as fully transparent
/// (skips compositing for that source at this output pixel).
///
/// Sampling premul instead of straight at edge pixels avoids the
/// "dark fringe" classical straight-bilinear produces: a half-pixel
/// between an opaque colour and full transparency reads in straight
/// space as half-brightness colour at half coverage, which composes
/// to quarter-brightness; in premul space it reads as half-brightness
/// premul at half coverage, which composes to half-brightness — the
/// physically correct partial coverage of full-brightness pixels.
pub(super) fn bilinear_sample_premul(
    rgba: &[u8],
    w: u32,
    h: u32,
    x: f32,
    y: f32,
) -> Option<(u8, u8, u8, u8)> {
    if w == 0 || h == 0 {
        return None;
    }
    // Pixel-index convention with half-pixel border on each edge —
    // `(-0.5, w - 0.5)` is the half-pixel "clamp-to-edge" zone
    // (matches GPU `textureSample` with `address_mode = ClampToEdge`).
    // Outside that range, the source's footprint doesn't cover this
    // world position; return `None` so the caller skips it.
    if x < -0.5 || x > (w as f32 - 0.5) || y < -0.5 || y > (h as f32 - 0.5) {
        return None;
    }
    // Clamp into the integer-index range `[0, w-1] × [0, h-1]` before
    // splitting into the bilinear quad. Values in `[-0.5, 0)` fold
    // onto pixel 0 (clamp-to-edge); similarly for the right border.
    let xc = x.clamp(0.0, (w - 1) as f32);
    let yc = y.clamp(0.0, (h - 1) as f32);
    let x0 = xc.floor() as u32;
    let y0 = yc.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = xc - x0 as f32;
    let fy = yc - y0 as f32;
    let sample = |xx: u32, yy: u32| -> (f32, f32, f32, f32) {
        let base = ((yy as usize) * (w as usize) + (xx as usize)) * 4;
        (
            rgba[base] as f32,
            rgba[base + 1] as f32,
            rgba[base + 2] as f32,
            rgba[base + 3] as f32,
        )
    };
    let s00 = sample(x0, y0);
    let s10 = sample(x1, y0);
    let s01 = sample(x0, y1);
    let s11 = sample(x1, y1);
    let lerp = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let r0 = lerp(s00.0, s10.0, fx);
    let g0 = lerp(s00.1, s10.1, fx);
    let b0 = lerp(s00.2, s10.2, fx);
    let a0 = lerp(s00.3, s10.3, fx);
    let r1 = lerp(s01.0, s11.0, fx);
    let g1 = lerp(s01.1, s11.1, fx);
    let b1 = lerp(s01.2, s11.2, fx);
    let a1 = lerp(s01.3, s11.3, fx);
    let r = (r0 * (1.0 - fy) + r1 * fy).clamp(0.0, 255.0);
    let g = (g0 * (1.0 - fy) + g1 * fy).clamp(0.0, 255.0);
    let b = (b0 * (1.0 - fy) + b1 * fy).clamp(0.0, 255.0);
    let a = (a0 * (1.0 - fy) + a1 * fy).clamp(0.0, 255.0);
    Some((r as u8, g as u8, b as u8, a as u8))
}

#[cfg(test)]
mod tests {
    use super::super::SrcRecord;
    use super::*;

    /// Build an axis-aligned-at-unit-scale `SrcRecord` for tests.
    /// World-rect: `[tx − size_w/2, tx + size_w/2] × [...]` (Y-up).
    fn unit_src(tx: f32, ty: f32, size_w: f32, size_h: f32, w: u32, h: u32) -> SrcRecord {
        SrcRecord {
            bits: 0,
            rgba: vec![0u8; (w as usize) * (h as usize) * 4],
            w,
            h,
            tx,
            ty,
            rot: 0.0,
            cos_t: 1.0,
            sin_t: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            inv_scale_x: 1.0,
            inv_scale_y: 1.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
            size_w,
            size_h,
            inv_size_w: 1.0 / size_w,
            inv_size_h: 1.0 / size_h,
            world_min_x: tx - size_w * 0.5,
            world_max_x: tx + size_w * 0.5,
            world_min_y: ty - size_h * 0.5,
            world_max_y: ty + size_h * 0.5,
        }
    }

    /// The `-0.5` half-pixel correction in `world_to_image` MUST land
    /// integer `img_x` for an output-pixel world centre that aligns
    /// with the source pixel grid. Audit + Enio 2026-05-27 "ainda
    /// borra imagem": before the correction, `img_x` was `i + 0.5` →
    /// bilinear blended pixel `i` with pixel `i+1` 50/50 → uniform
    /// half-pixel blur everywhere even on perfectly snapped grids.
    /// This test guards the lossless invariant.
    #[test]
    fn axis_aligned_centre_maps_to_integer_pixel_index() {
        // 10×10 m sprite at origin, 100×100 px (10 px/m).
        let src = unit_src(0.0, 0.0, 10.0, 10.0, 100, 100);
        // World-px-centre of source pixel 0 (top-left): `(p_left +
        // 0.5/pm, p_top - 0.5/pm)`.
        let wx = -5.0 + 0.5 / 10.0;
        let wy = 5.0 - 0.5 / 10.0;
        let (img_x, img_y) = world_to_image(wx, wy, &src);
        assert!(
            (img_x - 0.0).abs() < 1e-3,
            "img_x should be exactly 0 (pixel-index of pixel 0 centre), got {img_x}"
        );
        assert!(
            (img_y - 0.0).abs() < 1e-3,
            "img_y should be exactly 0, got {img_y}"
        );

        // World-centre of source pixel 50 along X.
        let wx_50 = -5.0 + (50.5) / 10.0;
        let (img_x_50, _) = world_to_image(wx_50, 0.0, &src);
        assert!(
            (img_x_50 - 50.0).abs() < 1e-3,
            "img_x at pixel 50 centre should be 50, got {img_x_50}"
        );
    }

    /// World-edge (where `img_x` should be exactly `−0.5`, the
    /// clamp-to-edge zone boundary) must NOT exceed the sampler's
    /// reject threshold of `−0.5` (`bilinear_sample_premul` line ~441).
    #[test]
    fn left_edge_lands_at_neg_half() {
        let src = unit_src(0.0, 0.0, 10.0, 10.0, 100, 100);
        // Exact world left edge of the sprite.
        let wx = -5.0;
        let (img_x, _) = world_to_image(wx, 0.0, &src);
        assert!(
            (img_x - (-0.5)).abs() < 1e-3,
            "img_x at world left-edge should be -0.5, got {img_x}"
        );
        // Sampler must accept -0.5 (clamp-to-edge zone). It does — the
        // check is `x < -0.5`, strict less than.
        let s = bilinear_sample_premul(&src.rgba, src.w, src.h, img_x, 0.0);
        assert!(s.is_some(), "sampler must accept -0.5 (clamp-to-edge)");
    }

    /// Negative scale (mirrored sprite) flips the inverse mapping
    /// correctly. Audit hand-trace said this works; assert it.
    #[test]
    fn negative_scale_mirror_inverse_is_correct() {
        // size 100 m, scale -1 → effective world range `[-50, 50]` still
        // (corners swap which side they map from in source).
        let mut src = unit_src(0.0, 0.0, 100.0, 100.0, 100, 100);
        src.scale_x = -1.0;
        src.inv_scale_x = -1.0;
        // World x = 25 (right of centre). For a mirrored sprite, this
        // should sample the LEFT half of the source. local_x = 25 / -1
        // = -25 → img_x = ((-25)/100 + 0.5) * 100 - 0.5 = 25 - 0.5 =
        // 24.5 → pixel ~24 (half between 24 and 25). The OLD code (no
        // -0.5) returned 25, but pixel-index 25 is centre of pixel 25,
        // not pixel 24.5. The new convention is the correct one.
        let (img_x, _) = world_to_image(25.0, 0.0, &src);
        assert!(
            (img_x - 24.5).abs() < 1e-3,
            "mirrored sprite: world x=25 -> img_x=24.5, got {img_x}"
        );
    }

    /// Sampler bounds: outside `[-0.5, w-0.5]` should reject, inside
    /// should accept including the half-pixel border. Defensive
    /// regression — audit flagged that the old `[0, w-1]` range
    /// underflows `floor()` on `-0.5`.
    #[test]
    fn sampler_bounds_include_half_pixel_border() {
        let rgba = vec![255u8; 4 * 4];
        // Inside.
        assert!(bilinear_sample_premul(&rgba, 2, 2, 0.0, 0.0).is_some());
        // Left clamp-to-edge border.
        assert!(bilinear_sample_premul(&rgba, 2, 2, -0.4, 0.0).is_some());
        // Right clamp-to-edge border.
        assert!(bilinear_sample_premul(&rgba, 2, 2, 1.4, 0.0).is_some());
        // Strictly outside.
        assert!(bilinear_sample_premul(&rgba, 2, 2, -0.6, 0.0).is_none());
        assert!(bilinear_sample_premul(&rgba, 2, 2, 1.6, 0.0).is_none());
    }
}
