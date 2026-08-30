//! Upscale — pure resampling algorithms (CPU, 100 % first-party).
//!
//! Three algorithms cover the design space (decision-record-style
//! rationale in [`crate::params`]):
//!
//! - [`upscale_nearest`] — pixel replication. Trivial. Preserves the
//!   source grid exactly. Accepts any factor (the destination has
//!   rectangular runs).
//! - [`upscale_lanczos3`] — sinc-based separable resample (Duchon
//!   1979). Two-pass: horizontal first (`src_w × src_h` →
//!   `dst_w × src_h`), then vertical (`dst_w × src_h` → `dst_w ×
//!   dst_h`). Per-pixel weight normalization keeps edge brightness
//!   stable when the kernel partially overhangs the source. Accepts
//!   any factor.
//! - [`upscale_epx`] — edge-directed corner replacement (EPX / Scale2x
//!   family, Johnson 1992 / Mazzoleni 2001) evaluated as a *continuous*
//!   reconstruction, so it accepts **any** factor. Byte-identical to
//!   Scale2x at `2×`. ⛔ It is NOT Hyllian xBR and no longer claims to
//!   be — the previous version clamped to `{2, 3, 4}`, which left
//!   `4×…16×` of the slider dead while the chip printed the raw value.
//!
//! All three operate on straight-alpha RGBA8 (`length = w * h * 4`)
//! and return `(Vec<u8>, dst_w, dst_h)`. No external image deps —
//! the kernels are pure Rust on raw byte slices.
//!
//! HR-5 (determinism): kernels are pure (input → output is a function),
//! no global state, no `mul_add`, no allocation beyond the output and
//! one intermediate buffer in the separable pass.

use crate::params::SCALE_FULL_SCALE;
use ph2d_color::SrgbRgba;

/// Lanczos kernel support: kernel half-width in source pixels. Lanczos3
/// reads `2 * SUPPORT = 6` source samples per destination pixel per axis.
const LANCZOS_SUPPORT: f32 = 3.0;

/// Result of a resample run: owned RGBA8 buffer + destination
/// dimensions (each guaranteed ≥ 1).
pub struct UpscaleResult {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Compute destination dimensions for a given factor, clamping each
/// axis to at least 1 px so degenerate inputs don't underflow. The
/// factor is clamped against [`SCALE_FULL_SCALE`] to honour the
/// slider ceiling even if a caller bypasses [`crate::params`].
fn dst_dims(src_w: u32, src_h: u32, factor: f32) -> (u32, u32) {
    let f = factor.clamp(1.0, SCALE_FULL_SCALE);
    let dw = ((src_w as f32 * f).round() as u32).max(1);
    let dh = ((src_h as f32 * f).round() as u32).max(1);
    (dw, dh)
}

// ──────────────────────────────────────────────────────────────────
// Nearest neighbour
// ──────────────────────────────────────────────────────────────────

/// Nearest-neighbour pixel replication. Each destination pixel
/// `(x_d, y_d)` reads the source pixel at
/// `(floor(x_d / factor), floor(y_d / factor))` (no filtering).
///
/// Accepts any factor; the destination has rectangular runs for
/// non-integer factors.
pub fn upscale_nearest(pixels: &[SrgbRgba], src_w: u32, src_h: u32, factor: f32) -> UpscaleResult {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    debug_assert_eq!(rgba.len(), (src_w as usize) * (src_h as usize) * 4);
    let (dw, dh) = dst_dims(src_w, src_h, factor);
    let mut out = vec![0u8; (dw as usize) * (dh as usize) * 4];
    if src_w == 0 || src_h == 0 {
        return UpscaleResult {
            pixels: out,
            width: dw,
            height: dh,
        };
    }
    let inv_fx = src_w as f32 / dw as f32;
    let inv_fy = src_h as f32 / dh as f32;
    let src_stride = (src_w as usize) * 4;
    let dst_stride = (dw as usize) * 4;
    for y in 0..dh as usize {
        let sy = ((y as f32 + 0.5) * inv_fy) as usize;
        let sy = sy.min(src_h as usize - 1);
        for x in 0..dw as usize {
            let sx = ((x as f32 + 0.5) * inv_fx) as usize;
            let sx = sx.min(src_w as usize - 1);
            let so = sy * src_stride + sx * 4;
            let dot = y * dst_stride + x * 4;
            out[dot..dot + 4].copy_from_slice(&rgba[so..so + 4]);
        }
    }
    UpscaleResult {
        pixels: out,
        width: dw,
        height: dh,
    }
}

// ──────────────────────────────────────────────────────────────────
// Lanczos3 (separable)
// ──────────────────────────────────────────────────────────────────

/// Normalized sinc function: `sin(π·x) / (π·x)`, with the removable
/// singularity at `x = 0` returning `1.0`.
fn sinc(x: f32) -> f32 {
    if x.abs() < 1.0e-8 {
        1.0
    } else {
        let px = std::f32::consts::PI * x;
        px.sin() / px
    }
}

/// 1-D Lanczos3 reconstruction kernel: `sinc(t) · sinc(t / 3)` for
/// `|t| < 3`, zero outside.
fn lanczos3_kernel(t: f32) -> f32 {
    if t.abs() >= LANCZOS_SUPPORT {
        0.0
    } else {
        sinc(t) * sinc(t / LANCZOS_SUPPORT)
    }
}

/// One contribution row: which source columns feed a single
/// destination column, and with what weights (already normalized so
/// `weights.sum() == 1.0` for that destination column).
struct Contribution {
    /// First source index this destination reads from (clamped to
    /// `[0, src_dim)` — entries past the edge are mirrored at the
    /// boundary).
    start: i32,
    /// Per-source weight. `weights.len() == count` (caller stride).
    weights: Vec<f32>,
}

/// Build the per-axis contribution table: for each of `dst_dim`
/// destination pixels, list which source pixels (indices + weights)
/// feed it under Lanczos3 with a scale of `dst_dim / src_dim`.
///
/// Mirror-edge handling: a kernel that overhangs the source mirrors
/// the source index back into range (so a corner pixel sees the same
/// neighbourhood as an interior pixel does, with no DC drop).
fn build_contributions(src_dim: u32, dst_dim: u32) -> Vec<Contribution> {
    let mut out = Vec::with_capacity(dst_dim as usize);
    let scale = dst_dim as f32 / src_dim as f32;
    // Upsampling (scale ≥ 1) keeps the kernel at its native support 3.
    // Downsampling would widen it (`SUPPORT / scale`) for prefiltering;
    // upscale is the only case here, so this stays simple.
    let support = LANCZOS_SUPPORT;
    let inv_scale = 1.0 / scale;
    for d in 0..dst_dim {
        // Source-space centre of this destination pixel (half-pixel
        // offset so dst index 0 hits the centre of src index 0).
        let center = (d as f32 + 0.5) * inv_scale - 0.5;
        let left = (center - support).floor() as i32;
        let right = (center + support).floor() as i32;
        let count = (right - left + 1).max(0) as usize;
        let mut weights = Vec::with_capacity(count);
        let mut wsum = 0.0_f32;
        for i in 0..count {
            let src_x = left + i as i32;
            let t = src_x as f32 - center;
            let w = lanczos3_kernel(t);
            weights.push(w);
            wsum += w;
        }
        // Normalize (HR-5: avoid `mul_add`).
        if wsum.abs() > 1.0e-8 {
            let inv = 1.0 / wsum;
            for w in &mut weights {
                *w *= inv;
            }
        }
        out.push(Contribution {
            start: left,
            weights,
        });
    }
    out
}

/// Mirror a possibly-OOB source index back into `[0, dim)`. Reflects
/// at both edges (a `-1` becomes `0`, a `dim` becomes `dim - 1`, etc.).
fn mirror_index(i: i32, dim: i32) -> usize {
    if dim <= 1 {
        return 0;
    }
    let period = 2 * dim - 2;
    let mut m = i.rem_euclid(period);
    if m >= dim {
        m = period - m;
    }
    m as usize
}

/// Lanczos3 separable upscale. Two passes (horizontal → vertical),
/// each using the contribution table from [`build_contributions`].
/// Per-channel arithmetic in `f32` linearly over straight-alpha RGBA8
/// (no premultiply — matches the rest of the Image Tools pipeline).
/// Final clamp to `0..=255` is `round()` then `clamp`, never `as u8`
/// truncation (which would bias dark).
pub fn upscale_lanczos3(pixels: &[SrgbRgba], src_w: u32, src_h: u32, factor: f32) -> UpscaleResult {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    debug_assert_eq!(rgba.len(), (src_w as usize) * (src_h as usize) * 4);
    let (dw, dh) = dst_dims(src_w, src_h, factor);
    if src_w == 0 || src_h == 0 {
        return UpscaleResult {
            pixels: vec![0u8; (dw as usize) * (dh as usize) * 4],
            width: dw,
            height: dh,
        };
    }
    let h_contrib = build_contributions(src_w, dw);
    let v_contrib = build_contributions(src_h, dh);
    let src_w_i = src_w as i32;
    let src_h_i = src_h as i32;

    // Horizontal pass: src_w × src_h → dw × src_h, RGBA f32.
    let mut h_pass: Vec<f32> = vec![0.0; (dw as usize) * (src_h as usize) * 4];
    let src_stride = (src_w as usize) * 4;
    let h_stride = (dw as usize) * 4;
    for y in 0..src_h as usize {
        let src_row = &rgba[y * src_stride..(y + 1) * src_stride];
        let dst_row = &mut h_pass[y * h_stride..(y + 1) * h_stride];
        for (x, c) in h_contrib.iter().enumerate() {
            let mut acc = [0.0_f32; 4];
            for (i, w) in c.weights.iter().enumerate() {
                let sx = mirror_index(c.start + i as i32, src_w_i);
                let so = sx * 4;
                acc[0] += src_row[so] as f32 * w;
                acc[1] += src_row[so + 1] as f32 * w;
                acc[2] += src_row[so + 2] as f32 * w;
                acc[3] += src_row[so + 3] as f32 * w;
            }
            let dot = x * 4;
            dst_row[dot] = acc[0];
            dst_row[dot + 1] = acc[1];
            dst_row[dot + 2] = acc[2];
            dst_row[dot + 3] = acc[3];
        }
    }

    // Vertical pass: dw × src_h → dw × dh, f32 → u8 with round+clamp.
    let mut out = vec![0u8; (dw as usize) * (dh as usize) * 4];
    let dst_stride = (dw as usize) * 4;
    for (y, c) in v_contrib.iter().enumerate() {
        let dst_row = &mut out[y * dst_stride..(y + 1) * dst_stride];
        for x in 0..dw as usize {
            let mut acc = [0.0_f32; 4];
            for (i, w) in c.weights.iter().enumerate() {
                let sy = mirror_index(c.start + i as i32, src_h_i);
                let so = sy * h_stride + x * 4;
                acc[0] += h_pass[so] * w;
                acc[1] += h_pass[so + 1] * w;
                acc[2] += h_pass[so + 2] * w;
                acc[3] += h_pass[so + 3] * w;
            }
            let dot = x * 4;
            dst_row[dot] = clamp_u8(acc[0]);
            dst_row[dot + 1] = clamp_u8(acc[1]);
            dst_row[dot + 2] = clamp_u8(acc[2]);
            dst_row[dot + 3] = clamp_u8(acc[3]);
        }
    }

    UpscaleResult {
        pixels: out,
        width: dw,
        height: dh,
    }
}

/// Round + clamp a possibly-negative or overshoot `f32` into a valid
/// `u8`. Lanczos's negative lobes produce small undershoots near edges
/// and overshoots at peaks; both must clamp, not wrap.
fn clamp_u8(v: f32) -> u8 {
    let r = v.round();
    if r <= 0.0 {
        0
    } else if r >= 255.0 {
        255
    } else {
        r as u8
    }
}

// ──────────────────────────────────────────────────────────────────
// EPX — edge-directed pixel-art upscale at ANY factor
//
// ⚠️ The kernel, its Scale2x oracle gates and its rationale live in the
// sibling [`crate::algorithm_epx`]; they are re-exported here so every
// caller still reaches the three algorithms through one module.
// ──────────────────────────────────────────────────────────────────

// ⚠️ A CHILD module, not a sibling: a descendant sees this module's
// private items, so `read_px` / `write_px` / `scale2x` / `scale3x` stay
// private instead of being promoted to `pub(crate)` just to be reached.
// Promoting them would newly expose raw `&[u8]` colour data to the whole
// crate, which `arch_color_space_typed` is right to refuse.
#[path = "algorithm_epx.rs"]
pub mod epx;

pub use epx::{epx_resample, upscale_epx};

/// Read a source pixel with mirror-edge handling. Returns an RGBA8
/// 4-byte array.
fn read_px(rgba: &[u8], w: i32, h: i32, x: i32, y: i32) -> [u8; 4] {
    let xi = mirror_index(x, w);
    let yi = mirror_index(y, h);
    let o = (yi * (w as usize) + xi) * 4;
    [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

/// Write a 4-byte RGBA8 pixel into the destination at `(x, y)`.
fn write_px(dst: &mut [u8], dw: u32, x: u32, y: u32, px: [u8; 4]) {
    let o = (y as usize * dw as usize + x as usize) * 4;
    dst[o..o + 4].copy_from_slice(&px);
}

/// Scale2x — Mazzoleni 2001. Doubles each axis.
///
/// ⚠️ **Test-only ORACLE, deliberately not on the product path.** The
/// product runs [`epx_resample`] at every factor; this block emitter is
/// kept so `epx_at_two_is_byte_identical_to_scale2x` compares the
/// continuous law against an *independent construction* rather than
/// against itself. A gate that compares a thing to a rearrangement of
/// itself is blind to a shared mutation.
#[cfg(test)]
fn scale2x(rgba: &[u8], sw: u32, sh: u32) -> UpscaleResult {
    let dw = sw * 2;
    let dh = sh * 2;
    let mut out = vec![0u8; (dw as usize) * (dh as usize) * 4];
    let sw_i = sw as i32;
    let sh_i = sh as i32;
    for y in 0..sh_i {
        for x in 0..sw_i {
            let e = read_px(rgba, sw_i, sh_i, x, y);
            let b = read_px(rgba, sw_i, sh_i, x, y - 1);
            let d = read_px(rgba, sw_i, sh_i, x - 1, y);
            let f = read_px(rgba, sw_i, sh_i, x + 1, y);
            let h = read_px(rgba, sw_i, sh_i, x, y + 1);
            let (tl, tr, bl, br) = if b != h && d != f {
                (
                    if d == b { d } else { e },
                    if b == f { f } else { e },
                    if d == h { d } else { e },
                    if h == f { f } else { e },
                )
            } else {
                (e, e, e, e)
            };
            let dx = (x as u32) * 2;
            let dy = (y as u32) * 2;
            write_px(&mut out, dw, dx, dy, tl);
            write_px(&mut out, dw, dx + 1, dy, tr);
            write_px(&mut out, dw, dx, dy + 1, bl);
            write_px(&mut out, dw, dx + 1, dy + 1, br);
        }
    }
    UpscaleResult {
        pixels: out,
        width: dw,
        height: dh,
    }
}

/// Scale3x — Mazzoleni 2001. Triples each axis. Same per-block
/// decision as Scale2x but emits a 3×3 output decided by the full
/// 3×3 source neighbourhood (corners + edges + centre).
///
/// ⚠️ **Test-only ORACLE** — see [`scale2x`]. The product's declared
/// divergence from this one (the four conditional *edge* cells) is what
/// `epx_at_three_matches_scale3x_corner_cells` pins.
#[cfg(test)]
fn scale3x(rgba: &[u8], sw: u32, sh: u32) -> UpscaleResult {
    let dw = sw * 3;
    let dh = sh * 3;
    let mut out = vec![0u8; (dw as usize) * (dh as usize) * 4];
    let sw_i = sw as i32;
    let sh_i = sh as i32;
    for y in 0..sh_i {
        for x in 0..sw_i {
            let a = read_px(rgba, sw_i, sh_i, x - 1, y - 1);
            let b = read_px(rgba, sw_i, sh_i, x, y - 1);
            let c = read_px(rgba, sw_i, sh_i, x + 1, y - 1);
            let d = read_px(rgba, sw_i, sh_i, x - 1, y);
            let e = read_px(rgba, sw_i, sh_i, x, y);
            let f = read_px(rgba, sw_i, sh_i, x + 1, y);
            let g = read_px(rgba, sw_i, sh_i, x - 1, y + 1);
            let h = read_px(rgba, sw_i, sh_i, x, y + 1);
            let i = read_px(rgba, sw_i, sh_i, x + 1, y + 1);
            let (e0, e1, e2, e3, e4, e5, e6, e7, e8) = if b != h && d != f {
                (
                    if d == b { d } else { e },
                    if (d == b && e != c) || (b == f && e != a) {
                        b
                    } else {
                        e
                    },
                    if b == f { f } else { e },
                    if (d == b && e != g) || (d == h && e != a) {
                        d
                    } else {
                        e
                    },
                    e,
                    if (b == f && e != i) || (h == f && e != c) {
                        f
                    } else {
                        e
                    },
                    if d == h { d } else { e },
                    if (d == h && e != i) || (h == f && e != g) {
                        h
                    } else {
                        e
                    },
                    if h == f { f } else { e },
                )
            } else {
                (e, e, e, e, e, e, e, e, e)
            };
            let dx = (x as u32) * 3;
            let dy = (y as u32) * 3;
            write_px(&mut out, dw, dx, dy, e0);
            write_px(&mut out, dw, dx + 1, dy, e1);
            write_px(&mut out, dw, dx + 2, dy, e2);
            write_px(&mut out, dw, dx, dy + 1, e3);
            write_px(&mut out, dw, dx + 1, dy + 1, e4);
            write_px(&mut out, dw, dx + 2, dy + 1, e5);
            write_px(&mut out, dw, dx, dy + 2, e6);
            write_px(&mut out, dw, dx + 1, dy + 2, e7);
            write_px(&mut out, dw, dx + 2, dy + 2, e8);
        }
    }
    UpscaleResult {
        pixels: out,
        width: dw,
        height: dh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convert a packed-bytes Vec<u8> to Vec<SrgbRgba> (1:1 chunking).
    fn pack(bytes: Vec<u8>) -> Vec<SrgbRgba> {
        assert!(bytes.len().is_multiple_of(4));
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| SrgbRgba([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// Build a uniform `w × h` opaque solid `rgb` for setup.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    // ── Common ──────────────────────────────────────────────────────

    #[test]
    fn dst_dims_clamp_factor_to_slider_range() {
        assert_eq!(dst_dims(4, 4, 0.1), (4, 4)); // clamped up to 1×
        assert_eq!(dst_dims(4, 4, 99.0), (64, 64)); // clamped to 16×
    }

    #[test]
    fn dst_dims_round_non_integer_factors() {
        assert_eq!(dst_dims(10, 10, 1.5), (15, 15));
        assert_eq!(dst_dims(10, 10, 2.0), (20, 20));
    }

    // ── Nearest ─────────────────────────────────────────────────────

    #[test]
    fn nearest_at_factor_one_is_identity() {
        let src = solid(3, 2, [10, 20, 30]);
        let r = upscale_nearest(&pack(src.clone()), 3, 2, 1.0);
        assert_eq!((r.width, r.height), (3, 2));
        assert_eq!(r.pixels, src);
    }

    #[test]
    fn nearest_doubles_dims_and_replicates_pixels() {
        // 2×2 input: TL=red, others=blue; @ 2× expect 4×4 with red TL block.
        let mut src = solid(2, 2, [0, 0, 255]);
        src[0..4].copy_from_slice(&[255, 0, 0, 255]); // (0,0) = red
        let r = upscale_nearest(&pack(src.clone()), 2, 2, 2.0);
        assert_eq!((r.width, r.height), (4, 4));
        // The red pixel must occupy the full TL 2×2 block.
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(px(&r.pixels, 4, x, y), [255, 0, 0, 255]);
            }
        }
        // And the blue covers the rest.
        assert_eq!(px(&r.pixels, 4, 3, 3), [0, 0, 255, 255]);
    }

    #[test]
    fn nearest_handles_non_integer_factor() {
        let src = solid(4, 4, [100, 100, 100]);
        let r = upscale_nearest(&pack(src.clone()), 4, 4, 1.5);
        assert_eq!((r.width, r.height), (6, 6));
        // Every output pixel is the uniform source colour.
        for y in 0..6 {
            for x in 0..6 {
                assert_eq!(px(&r.pixels, 6, x, y), [100, 100, 100, 255]);
            }
        }
    }

    // ── Lanczos3 ────────────────────────────────────────────────────

    #[test]
    fn lanczos3_at_factor_one_is_near_identity_on_flat() {
        let src = solid(8, 8, [120, 80, 40]);
        let r = upscale_lanczos3(&pack(src.clone()), 8, 8, 1.0);
        assert_eq!((r.width, r.height), (8, 8));
        // Solid colour must come back inside ±1 (round/clamp).
        for y in 0..8 {
            for x in 0..8 {
                let p = px(&r.pixels, 8, x, y);
                for (a, b) in p.iter().zip([120u8, 80, 40, 255].iter()) {
                    assert!(
                        (*a as i32 - *b as i32).abs() <= 1,
                        "diff at ({x},{y}): got {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn lanczos3_doubles_dims_and_keeps_dc_on_flat() {
        // A flat colour must stay flat (DC preserved) and the alpha
        // channel must clamp at 255 — no negative-lobe undershoot.
        let src = solid(4, 4, [50, 150, 200]);
        let r = upscale_lanczos3(&pack(src.clone()), 4, 4, 2.0);
        assert_eq!((r.width, r.height), (8, 8));
        // Centre pixel — well clear of any edge mirroring; should equal
        // the source DC exactly within round/clamp tolerance.
        let p = px(&r.pixels, 8, 4, 4);
        for (a, b) in p.iter().zip([50u8, 150, 200, 255].iter()) {
            assert!((*a as i32 - *b as i32).abs() <= 1);
        }
    }

    #[test]
    fn lanczos3_alpha_clamps_correctly_on_fully_opaque() {
        // Fully opaque input must yield fully opaque output (alpha
        // doesn't drift). Lanczos negative lobes can push slightly past
        // 255 — clamp_u8 must catch.
        let src = solid(8, 8, [200, 50, 25]);
        let r = upscale_lanczos3(&pack(src.clone()), 8, 8, 2.5);
        for p in r.pixels.chunks(4) {
            assert_eq!(p[3], 255, "alpha must clamp at 255, got {}", p[3]);
        }
    }

    // ⚠️ The EPX gates (the Scale2x oracle, the whole-course gate, the
    // not-an-alias gate) live in `algorithm_epx.rs`, next to the kernel
    // they measure. `scale2x` / `scale3x` stay HERE as `pub(crate)`
    // test-only oracles: they are an independent construction, which is
    // the only reason the parity gate means anything.

    // ── Mirror index ───────────────────────────────────────────────

    #[test]
    fn mirror_index_handles_oob_and_dim_one() {
        assert_eq!(mirror_index(0, 4), 0);
        assert_eq!(mirror_index(3, 4), 3);
        assert_eq!(mirror_index(-1, 4), 1); // reflect
        assert_eq!(mirror_index(4, 4), 2); // reflect
        assert_eq!(mirror_index(-3, 4), 3);
        assert_eq!(mirror_index(7, 4), 1);
        assert_eq!(mirror_index(7, 1), 0); // degenerate dim
    }
}
