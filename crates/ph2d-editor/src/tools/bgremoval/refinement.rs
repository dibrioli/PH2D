//! Mask refinement passes — applied AFTER the algorithm produces a
//! raw mask, BEFORE composite with RGB.
//!
//! Order matters: `apply_opening_closing` (optional auto-clean) →
//! `apply_expansion` (user knob) → `apply_smoothing` (guided filter)
//! → `apply_feather` (F-H exact distance transform).
//!
//! All passes operate in-place on the mask and pre-allocate every
//! scratch buffer they need — HR-3 compliant.

/// Morphological expand (`amount > 0` → dilate) or contract
/// (`amount < 0` → erode). Magnitude is the number of 8-connected
/// iterations. `scratch` must be `w*h`.
pub fn apply_expansion(mask: &mut [f32], w: u32, h: u32, amount: f32, scratch: &mut [f32]) {
    let iters = amount.round().abs() as u32;
    if iters == 0 {
        return;
    }
    let expand = amount > 0.0;
    let (w, h) = (w as usize, h as usize);
    debug_assert_eq!(mask.len(), w * h);
    debug_assert_eq!(scratch.len(), w * h);

    for _ in 0..iters {
        scratch.copy_from_slice(mask);
        for y in 0..h {
            for x in 0..w {
                let pos = y * w + x;
                let mut best = scratch[pos];
                for dy in -1i32..=1 {
                    let ny = y as i32 + dy;
                    if ny < 0 || ny as usize >= h {
                        continue;
                    }
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        if nx < 0 || nx as usize >= w {
                            continue;
                        }
                        let v = scratch[ny as usize * w + nx as usize];
                        if expand {
                            if v > best {
                                best = v;
                            }
                        } else if v < best {
                            best = v;
                        }
                    }
                }
                mask[pos] = best;
            }
        }
    }
}

/// Opening (erode then dilate) followed by closing (dilate then
/// erode), each with a single 3×3 iteration. Removes salt-and-pepper
/// noise without touching well-formed silhouettes.
pub fn apply_opening_closing(mask: &mut [f32], w: u32, h: u32, scratch: &mut [f32]) {
    apply_expansion(mask, w, h, -1.0, scratch); // erode
    apply_expansion(mask, w, h, 1.0, scratch); // dilate → opening done
    apply_expansion(mask, w, h, 1.0, scratch); // dilate
    apply_expansion(mask, w, h, -1.0, scratch); // erode → closing done
}

/// Edge-preserving guided-filter smoothing (He 2010), guidance =
/// the source RGBA's BT.601 luminance. `amount` 0..=10 maps to a
/// filter radius 1..=10 px. Higher `amount` = smoother mask without
/// blurring across the subject's silhouette edges.
///
/// `scratch_a..scratch_f` are all `w*h` f32 buffers — caller reuses
/// them across calls (HR-3).
// HR-3: six caller-owned scratch slices is the price of zero-alloc
// guided filter (each holds a distinct intermediate of the He 2010
// recurrence). Packing them into a struct just renames the count.
#[allow(clippy::too_many_arguments)]
pub fn apply_smoothing(
    mask: &mut [f32],
    rgba: &[u8],
    w: u32,
    h: u32,
    amount: f32,
    scratch_a: &mut [f32], // I (guidance)
    scratch_b: &mut [f32], // mean_I
    scratch_c: &mut [f32], // mean_p
    scratch_d: &mut [f32], // corr_I + later var_I + later mean_a
    scratch_e: &mut [f32], // corr_Ip + later cov_Ip + later mean_b
    scratch_f: &mut [f32], // generic scratch
) {
    if amount <= 0.0 {
        return;
    }
    let (wu, hu) = (w as usize, h as usize);
    let total = wu * hu;
    let r = amount.clamp(0.0, 10.0).round() as usize;
    if r == 0 {
        return;
    }
    let eps: f32 = 1e-3;

    debug_assert_eq!(mask.len(), total);
    debug_assert_eq!(scratch_a.len(), total);

    // Build guidance I = luma(rgba) ∈ [0, 1].
    for (i, s) in scratch_a.iter_mut().enumerate().take(total) {
        let idx = i * 4;
        *s = (rgba[idx] as f32 * 0.299
            + rgba[idx + 1] as f32 * 0.587
            + rgba[idx + 2] as f32 * 0.114)
            / 255.0;
    }

    // mean_I = boxfilter(I, r) ; mean_p = boxfilter(p, r)
    box_filter(scratch_a, wu, hu, r, scratch_b, scratch_f); // → mean_I
    box_filter(mask, wu, hu, r, scratch_c, scratch_f); // → mean_p

    // I*I and I*p go into scratch_d, scratch_e then box-filtered.
    for i in 0..total {
        scratch_d[i] = scratch_a[i] * scratch_a[i];
    }
    box_filter_inplace(scratch_d, wu, hu, r, scratch_f); // → mean_II

    for i in 0..total {
        scratch_e[i] = scratch_a[i] * mask[i];
    }
    box_filter_inplace(scratch_e, wu, hu, r, scratch_f); // → mean_Ip

    // var_I = mean_II - mean_I² ;  cov_Ip = mean_Ip - mean_I*mean_p
    for i in 0..total {
        let mi = scratch_b[i];
        let mp = scratch_c[i];
        let var_i = scratch_d[i] - mi * mi;
        let cov_ip = scratch_e[i] - mi * mp;
        // a = cov / (var + ε), b = mean_p - a*mean_I.
        let a = cov_ip / (var_i + eps);
        let b = mp - a * mi;
        scratch_d[i] = a; // reuse: scratch_d → a
        scratch_e[i] = b; // reuse: scratch_e → b
    }

    box_filter_inplace(scratch_d, wu, hu, r, scratch_f); // mean_a
    box_filter_inplace(scratch_e, wu, hu, r, scratch_f); // mean_b

    // q = mean_a * I + mean_b
    for i in 0..total {
        let q = scratch_d[i] * scratch_a[i] + scratch_e[i];
        mask[i] = q.clamp(0.0, 1.0);
    }
}

/// 1D-then-1D box filter with radius `r` (so box size = `2r+1`).
/// Output goes into `out`; `temp` is a `w*h` scratch.
fn box_filter(src: &[f32], w: usize, h: usize, r: usize, out: &mut [f32], temp: &mut [f32]) {
    box_horizontal(src, w, h, r, temp);
    box_vertical(temp, w, h, r, out);
}

/// In-place variant: writes the result back into `buf`.
fn box_filter_inplace(buf: &mut [f32], w: usize, h: usize, r: usize, temp: &mut [f32]) {
    box_horizontal(buf, w, h, r, temp);
    box_vertical(temp, w, h, r, buf);
}

fn box_horizontal(src: &[f32], w: usize, h: usize, r: usize, out: &mut [f32]) {
    if w == 0 || h == 0 {
        return;
    }
    for y in 0..h {
        let row = y * w;
        let mut sum = 0.0_f32;
        // Prime window: [0..=r] clipped to width.
        let init_hi = r.min(w - 1);
        for x in 0..=init_hi {
            sum += src[row + x];
        }
        let mut count = (init_hi + 1) as f32;
        out[row] = sum / count;

        for x in 1..w {
            // Add new element on the right.
            let add_x = x + r;
            if add_x < w {
                sum += src[row + add_x];
                count += 1.0;
            }
            // Drop element off the left.
            if x > r {
                let drop_x = x - r - 1;
                sum -= src[row + drop_x];
                count -= 1.0;
            }
            out[row + x] = sum / count;
        }
    }
}

fn box_vertical(src: &[f32], w: usize, h: usize, r: usize, out: &mut [f32]) {
    if w == 0 || h == 0 {
        return;
    }
    for x in 0..w {
        let mut sum = 0.0_f32;
        let init_hi = r.min(h - 1);
        for y in 0..=init_hi {
            sum += src[y * w + x];
        }
        let mut count = (init_hi + 1) as f32;
        out[x] = sum / count;

        for y in 1..h {
            let add_y = y + r;
            if add_y < h {
                sum += src[add_y * w + x];
                count += 1.0;
            }
            if y > r {
                let drop_y = y - r - 1;
                sum -= src[drop_y * w + x];
                count -= 1.0;
            }
            out[y * w + x] = sum / count;
        }
    }
}

/// Apply feather using Felzenszwalb-Huttenlocher exact Euclidean
/// distance transform.
///
/// Caller pre-allocates:
/// - `dist_scratch` (`w*h` f32) — holds squared then linear distances.
/// - `col_f` (`max(w, h)` f32) — current 1D column buffer.
/// - `col_prev` (`max(w, h)` f32) — previous-pass copy used during the
///   output sweep (avoids in-place RAW hazard).
/// - `col_v` (`max(w, h)` usize) — parabola origin indices.
/// - `col_z` (`max(w, h) + 1` f32) — parabola intersection bounds.
// HR-3: five caller-owned scratch buffers is the cost of a zero-alloc
// F-H exact EDT. Each is a distinct intermediate of the recurrence;
// fusing them into a struct just renames the count.
#[allow(clippy::too_many_arguments)]
pub fn apply_feather(
    mask: &mut [f32],
    w: u32,
    h: u32,
    width: f32,
    strength: f32,
    dist_scratch: &mut [f32],
    col_f: &mut Vec<f32>,
    col_prev: &mut Vec<f32>,
    col_v: &mut Vec<usize>,
    col_z: &mut Vec<f32>,
) {
    if width <= 0.0 || strength <= 0.0 {
        return;
    }
    let (wu, hu) = (w as usize, h as usize);
    let total = wu * hu;
    debug_assert_eq!(mask.len(), total);
    debug_assert_eq!(dist_scratch.len(), total);

    let radius = width.max(0.0);
    let strength_norm = strength.clamp(0.0, 100.0) / 100.0;

    // Build a binary "edge or not" indicator in `dist_scratch`:
    // 0 where the pixel sits on a silhouette edge, large² elsewhere.
    let big = (wu + hu) as f32;
    let big_sq = big * big;
    for d in dist_scratch.iter_mut() {
        *d = big_sq;
    }
    for y in 0..hu {
        for x in 0..wu {
            let pos = y * wu + x;
            let v = mask[pos];
            let mut is_edge = false;
            if x > 0 && (v - mask[pos - 1]).abs() > 0.3 {
                is_edge = true;
            }
            if !is_edge && x + 1 < wu && (v - mask[pos + 1]).abs() > 0.3 {
                is_edge = true;
            }
            if !is_edge && y > 0 && (v - mask[pos - wu]).abs() > 0.3 {
                is_edge = true;
            }
            if !is_edge && y + 1 < hu && (v - mask[pos + wu]).abs() > 0.3 {
                is_edge = true;
            }
            if is_edge {
                dist_scratch[pos] = 0.0;
            }
        }
    }

    let max_dim = wu.max(hu);
    col_f.resize(max_dim, 0.0);
    col_prev.resize(max_dim, 0.0);
    col_v.resize(max_dim, 0);
    col_z.resize(max_dim + 1, 0.0);

    // Pass 1 — columns.
    for x in 0..wu {
        for y in 0..hu {
            col_f[y] = dist_scratch[y * wu + x];
        }
        edt_1d(col_f, col_prev, col_v, col_z, hu);
        for y in 0..hu {
            dist_scratch[y * wu + x] = col_f[y];
        }
    }
    // Pass 2 — rows. Take sqrt to convert squared → linear distance.
    for y in 0..hu {
        for x in 0..wu {
            col_f[x] = dist_scratch[y * wu + x];
        }
        edt_1d(col_f, col_prev, col_v, col_z, wu);
        for x in 0..wu {
            dist_scratch[y * wu + x] = col_f[x].sqrt();
        }
    }

    for i in 0..total {
        let d = dist_scratch[i];
        if d >= radius {
            continue;
        }
        let t = d / radius;
        let s = smootherstep(t);
        let original = mask[i];
        if original > 0.5 {
            mask[i] = original * (s + (1.0 - s) * (1.0 - strength_norm));
        } else {
            mask[i] = original + (1.0 - original) * (1.0 - s) * strength_norm * 0.3;
        }
    }
}

/// 1D Felzenszwalb-Huttenlocher exact squared-EDT.
/// `f` is input/output (squared distances). `prev`, `v`, `z` are
/// caller-owned scratch buffers reused across calls — zero alloc.
fn edt_1d(f: &mut [f32], prev: &mut [f32], v: &mut [usize], z: &mut [f32], n: usize) {
    if n == 0 {
        return;
    }
    prev[..n].copy_from_slice(&f[..n]);

    // Build lower envelope of parabolas.
    let mut k: usize = 0;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        let qf = q as f32;
        let fq = prev[q];
        // Pop parabolas from the envelope until the new one fits.
        loop {
            let vk = v[k];
            let vkf = vk as f32;
            let s = ((fq + qf * qf) - (prev[vk] + vkf * vkf)) / (2.0 * (qf - vkf));
            if s <= z[k] {
                if k == 0 {
                    // Replace the bottom parabola — it's fully shadowed.
                    v[0] = q;
                    z[0] = f32::NEG_INFINITY;
                    z[1] = f32::INFINITY;
                    break;
                }
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
        }
    }

    // Sweep + write the resulting EDT into `f`.
    let mut k2: usize = 0;
    for (q, fq) in f.iter_mut().enumerate().take(n) {
        while z[k2 + 1] < q as f32 {
            k2 += 1;
        }
        let vk = v[k2];
        let d = q as f32 - vk as f32;
        *fq = d * d + prev[vk];
    }
}

#[inline]
fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mask(w: usize, h: usize, fill: f32) -> Vec<f32> {
        vec![fill; w * h]
    }

    #[test]
    fn expansion_zero_amount_is_noop() {
        let mut mask = vec![0.5; 16];
        let mut scratch = vec![0.0; 16];
        let before = mask.clone();
        apply_expansion(&mut mask, 4, 4, 0.0, &mut scratch);
        assert_eq!(mask, before);
    }

    #[test]
    fn dilate_grows_foreground() {
        // 4×4 mask, single 1.0 pixel at center.
        let mut mask = make_mask(4, 4, 0.0);
        mask[5] = 1.0;
        let mut scratch = mask.clone();
        apply_expansion(&mut mask, 4, 4, 1.0, &mut scratch);
        // After 1 dilation, all 8-neighbors of (1,1) should be 1.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let y = (1 + dy) as usize;
                let x = (1 + dx) as usize;
                assert_eq!(mask[y * 4 + x], 1.0);
            }
        }
    }

    #[test]
    fn erode_shrinks_foreground() {
        // 4×4 mask, fully 1.0.
        let mut mask = make_mask(4, 4, 1.0);
        // Put a single hole.
        mask[5] = 0.0;
        let mut scratch = mask.clone();
        apply_expansion(&mut mask, 4, 4, -1.0, &mut scratch);
        // The 8-neighborhood of the hole should now be 0.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let y = (1 + dy) as usize;
                let x = (1 + dx) as usize;
                assert_eq!(mask[y * 4 + x], 0.0);
            }
        }
    }

    #[test]
    fn opening_closing_removes_isolated_pixel() {
        // 8×8 mostly 0.0, single pixel 1.0 in the middle (salt).
        let mut mask = make_mask(8, 8, 0.0);
        mask[3 * 8 + 3] = 1.0;
        let mut scratch = mask.clone();
        apply_opening_closing(&mut mask, 8, 8, &mut scratch);
        // Opening (erode→dilate) wipes salt. The single pixel should
        // be gone.
        assert_eq!(mask[3 * 8 + 3], 0.0);
    }

    #[test]
    fn box_filter_uniform_field_unchanged() {
        let w = 8;
        let h = 8;
        let src = vec![0.5; w * h];
        let mut out = vec![0.0; w * h];
        let mut temp = vec![0.0; w * h];
        box_filter(&src, w, h, 2, &mut out, &mut temp);
        for v in &out {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn smoothing_zero_amount_is_noop() {
        let mut mask = vec![0.5; 16];
        let rgba = vec![128u8; 64];
        let before = mask.clone();
        let mut a = vec![0.0; 16];
        let mut b = vec![0.0; 16];
        let mut c = vec![0.0; 16];
        let mut d = vec![0.0; 16];
        let mut e = vec![0.0; 16];
        let mut f = vec![0.0; 16];
        apply_smoothing(
            &mut mask, &rgba, 4, 4, 0.0, &mut a, &mut b, &mut c, &mut d, &mut e, &mut f,
        );
        assert_eq!(mask, before);
    }

    #[test]
    fn smoothing_uniform_mask_stays_uniform() {
        let w = 8u32;
        let h = 8u32;
        let total = (w * h) as usize;
        let mut mask = vec![1.0; total];
        let rgba = vec![128u8; total * 4];
        let mut a = vec![0.0; total];
        let mut b = vec![0.0; total];
        let mut c = vec![0.0; total];
        let mut d = vec![0.0; total];
        let mut e = vec![0.0; total];
        let mut f = vec![0.0; total];
        apply_smoothing(
            &mut mask, &rgba, w, h, 3.0, &mut a, &mut b, &mut c, &mut d, &mut e, &mut f,
        );
        for v in &mask {
            assert!((v - 1.0).abs() < 0.05);
        }
    }

    #[test]
    fn feather_zero_width_is_noop() {
        let mut mask = vec![0.5; 16];
        let mut dist = vec![0.0; 16];
        let mut cf = Vec::new();
        let mut cp = Vec::new();
        let mut cv = Vec::new();
        let mut cz = Vec::new();
        let before = mask.clone();
        apply_feather(
            &mut mask, 4, 4, 0.0, 100.0, &mut dist, &mut cf, &mut cp, &mut cv, &mut cz,
        );
        assert_eq!(mask, before);
    }

    #[test]
    fn feather_creates_soft_band_around_silhouette() {
        // 16×16 mask, hard-edged 1.0 square at center 4..=11.
        let w = 16usize;
        let h = 16usize;
        let mut mask = vec![0.0; w * h];
        for y in 4..=11 {
            for x in 4..=11 {
                mask[y * w + x] = 1.0;
            }
        }
        let mut dist = vec![0.0; w * h];
        let mut cf = Vec::new();
        let mut cp = Vec::new();
        let mut cv = Vec::new();
        let mut cz = Vec::new();
        apply_feather(
            &mut mask, w as u32, h as u32, 3.0, 100.0, &mut dist, &mut cf, &mut cp, &mut cv,
            &mut cz,
        );
        // Center stays close to 1, outside-of-radius stays 0,
        // immediately outside the silhouette becomes a soft value.
        assert!(mask[7 * w + 7] > 0.5); // deep center
        assert!(mask[0] < 0.1); // far corner
        // One pixel just outside (3, 7): should be ≥ 0.
        let outside = 7 * w + 3;
        assert!(mask[outside] >= 0.0);
    }
}
