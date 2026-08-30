//! **EPX — edge-directed pixel-art upscale, any destination size.**
//!
//! Split out of `algorithm.rs` when this kernel replaced the mode's
//! permanent alias and pushed that file past its frozen workspace LOC
//! cap. It is the honest split, not a raised cap.
//!
//! ⚠️ Each Image Tools drop-crate owns its kernels by charter — the
//! Lanczos3 and Nearest resamplers are duplicated between this crate
//! and `ph2d-tool-upscale` for the same reason, and this one joins
//! them. What is NOT duplicated is the law: the two implementations are
//! each gated against an independent Scale2x oracle at an exact `2x`,
//! so they cannot drift into different algorithms wearing one name.

/// Half-width of the 45° corner cut, in source-pixel units (L1 distance
/// from the corner of the source pixel).
///
/// ⚠️ **DERIVED, not chosen.** The ideal 1-px staircase edge is the 45°
/// line through the midpoints of the two cut sides — `(0.5, 0)` and
/// `(1, 0.5)` for the top-right corner — and both satisfy
/// `(1 − u) + v = 0.5`. Its consequence: at an exact `2×` the four
/// sample points land on the line, so the law degenerates to Scale2x
/// byte-for-byte.
const CORNER_CUT: f32 = 0.5;

/// Edge-directed pixel-art resample (EPX / Scale2x family, Johnson
/// 1992 / Mazzoleni 2001) evaluated as a *continuous* reconstruction,
/// so it accepts any `(dw, dh)` — including the non-uniform ratios a
/// fit-to-target produces.
///
/// For each destination sample, the integer part of the back-projected
/// coordinate picks the source pixel `E` and the fractional part
/// `(u, v)` says where inside `E` the sample sits. The EPX predicate
/// (`B != H && D != F`, then the corner's own pair test) decides
/// whether a 45° edge cuts that corner; samples within [`CORNER_CUT`]
/// of a cut corner take the neighbour's colour, everything else keeps
/// `E`. Flat regions are byte-exact replication at every ratio, which
/// is the property that makes it pixel-art safe.
///
/// ⛔ **Not Hyllian xBR** — the label this mode used to wear. xBR's
/// predicate is a YUV-weighted distance sum over a 21-tap window; an
/// approximation shipped under that name is the defect this replaced.
pub(super) fn epx_resample(rgba: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return rgba.to_vec();
    }
    let sw_i = sw as i32;
    let sh_i = sh as i32;
    let step_x = sw as f32 / dw as f32;
    let step_y = sh as f32 / dh as f32;
    let mut out = vec![0u8; (dw as usize) * (dh as usize) * 4];
    for dy in 0..dh {
        let sy = (dy as f32 + 0.5) * step_y;
        let y0f = sy.floor();
        let v = sy - y0f;
        let y0 = y0f as i32;
        for dx in 0..dw {
            let sx = (dx as f32 + 0.5) * step_x;
            let x0f = sx.floor();
            let u = sx - x0f;
            let x0 = x0f as i32;

            let e = clamped_px(rgba, sw_i, sh_i, x0, y0);
            let b = clamped_px(rgba, sw_i, sh_i, x0, y0 - 1);
            let d = clamped_px(rgba, sw_i, sh_i, x0 - 1, y0);
            let f = clamped_px(rgba, sw_i, sh_i, x0 + 1, y0);
            let h = clamped_px(rgba, sw_i, sh_i, x0, y0 + 1);

            let mut px = e;
            if b != h && d != f {
                let left = u < 0.5;
                let top = v < 0.5;
                let du = if left { u } else { 1.0 - u };
                let dv = if top { v } else { 1.0 - v };
                let horiz = if left { d } else { f };
                let vert = if top { b } else { h };
                if horiz == vert && du + dv <= CORNER_CUT {
                    px = horiz;
                }
            }
            let o = (dy as usize * dw as usize + dx as usize) * 4;
            out[o..o + 4].copy_from_slice(&px);
        }
    }
    out
}

/// Read a source pixel with clamp-to-edge — the same edge rule
/// `nearest_upscale` and `resample_axis` use in this crate, so the
/// three kernels agree about what lies outside the image.
fn clamped_px(rgba: &[u8], w: i32, h: i32, x: i32, y: i32) -> [u8; 4] {
    let xi = x.clamp(0, w - 1) as usize;
    let yi = y.clamp(0, h - 1) as usize;
    let o = (yi * (w as usize) + xi) * 4;
    [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

#[cfg(test)]
mod tests {
    use super::super::upscale_to_at_least;
    use super::*;
    use crate::params::UpscaleAlgorithm;

    /// A `w`x`h` canvas where every pixel is `[r,g,b,255]`.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    /// A hard diagonal edge — the ONLY fixture that can tell three
    /// scalers apart. A flat or smooth fixture makes Nearest, Lanczos3
    /// and EPX agree, and would green-light the alias this pass removed.
    fn diagonal_edge(n: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(n * n * 4);
        for y in 0..n {
            for x in 0..n {
                let c: [u8; 4] = if y > x {
                    [0, 0, 0, 255]
                } else {
                    [255, 0, 0, 255]
                };
                v.extend_from_slice(&c);
            }
        }
        v
    }

    /// Independent ORACLE: the canonical Scale2x block emitter (iterate
    /// the SOURCE, emit a 2×2 block), with this crate's clamp edge
    /// rule. Built by a different construction from [`epx_resample`] on
    /// purpose — a gate that compares a thing to a rearrangement of
    /// itself is blind to a shared mutation.
    fn scale2x_oracle(rgba: &[u8], sw: u32, sh: u32) -> Vec<u8> {
        let (w, h) = (sw as i32, sh as i32);
        let dw = (sw * 2) as usize;
        let mut out = vec![0u8; dw * (sh as usize) * 2 * 4];
        for y in 0..h {
            for x in 0..w {
                let e = clamped_px(rgba, w, h, x, y);
                let b = clamped_px(rgba, w, h, x, y - 1);
                let d = clamped_px(rgba, w, h, x - 1, y);
                let f = clamped_px(rgba, w, h, x + 1, y);
                let hh = clamped_px(rgba, w, h, x, y + 1);
                let (tl, tr, bl, br) = if b != hh && d != f {
                    (
                        if d == b { d } else { e },
                        if b == f { f } else { e },
                        if d == hh { d } else { e },
                        if hh == f { f } else { e },
                    )
                } else {
                    (e, e, e, e)
                };
                for (ox, oy, c) in [(0, 0, tl), (1, 0, tr), (0, 1, bl), (1, 1, br)] {
                    let o = (((y * 2 + oy) as usize) * dw + (x * 2 + ox) as usize) * 4;
                    out[o..o + 4].copy_from_slice(&c);
                }
            }
        }
        out
    }

    /// ORACLE gate — the continuous law reproduces Scale2x byte-for-byte
    /// at an exact `2×`. This is what says the kernel is really EPX and
    /// not an improvisation.
    #[test]
    fn epx_at_two_is_byte_identical_to_scale2x() {
        for n in [3usize, 4, 12] {
            let src = diagonal_edge(n);
            let got = epx_resample(&src, n as u32, n as u32, n as u32 * 2, n as u32 * 2);
            assert_eq!(
                got,
                scale2x_oracle(&src, n as u32, n as u32),
                "EPX at 2x must equal Scale2x byte-for-byte (n={n})"
            );
        }
    }

    /// ⭐ **The alias gate.** On a fixture that CONTAINS the phenomenon,
    /// picking EPX must produce pixels that are neither Nearest's nor
    /// Lanczos3's — at an INTEGER ratio (where it used to alias Nearest)
    /// and at a FRACTIONAL one (where it used to alias Lanczos3).
    #[test]
    fn epx_is_not_an_alias_of_nearest_or_lanczos_at_either_kind_of_ratio() {
        let n = 12u32;
        let src = diagonal_edge(n as usize);
        // (min_w, min_h) pairs: an exact integer ratio, then two
        // fractional ones, then a NON-UNIFORM fit (the normal case).
        for (mw, mh) in [(24u32, 24u32), (30, 30), (19, 19), (24, 30)] {
            let (epx, ew, eh) = upscale_to_at_least(&src, n, n, mw, mh, UpscaleAlgorithm::Epx);
            let (near, nw, nh) = upscale_to_at_least(&src, n, n, mw, mh, UpscaleAlgorithm::Nearest);
            let (lanc, lw, lh) =
                upscale_to_at_least(&src, n, n, mw, mh, UpscaleAlgorithm::Lanczos3);
            assert!(ew >= mw && eh >= mh, "EPX must still reach the target");
            assert!(
                (ew, eh) != (nw, nh) || epx != near,
                "EPX at ({mw},{mh}) is an alias of Nearest"
            );
            assert!(
                (ew, eh) != (lw, lh) || epx != lanc,
                "EPX at ({mw},{mh}) is an alias of Lanczos3"
            );
        }
    }

    #[test]
    fn epx_leaves_a_flat_region_byte_exact() {
        // The predicate cannot fire on a flat region, at any ratio —
        // the property that makes EPX pixel-art safe.
        let src = solid(6, 6, [70, 90, 110]);
        for (mw, mh) in [(12u32, 12u32), (17, 23), (96, 96)] {
            let (out, _, _) = upscale_to_at_least(&src, 6, 6, mw, mh, UpscaleAlgorithm::Epx);
            for p in out.chunks(4) {
                assert_eq!(p, [70, 90, 110, 255], "EPX filtered a flat region");
            }
        }
    }
}
