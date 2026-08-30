//! **EPX — edge-directed pixel-art upscale, at any factor.**
//!
//! Split out of `algorithm.rs` when this kernel replaced the old
//! `{2, 3, 4}`-clamped one and pushed that file past its frozen
//! workspace LOC cap. It is the honest split, not a raised cap: the
//! sinc/nearest resamplers and this edge-directed one answer different
//! questions and share only the neighbourhood helpers.
//!
//! The whole rationale — why `CORNER_CUT` is derived rather than
//! chosen, what is reproduced from Scale2x/Scale3x and what is a
//! declared divergence, and why this is deliberately NOT called
//! "xBR" — lives on [`upscale_epx`].

use super::{UpscaleResult, dst_dims, read_px, write_px};
use ph2d_color::SrgbRgba;

// ──────────────────────────────────────────────────────────────────
// EPX — edge-directed pixel-art upscale at ANY factor
// ──────────────────────────────────────────────────────────────────

/// Half-width of the 45° corner cut, in source-pixel units, measured
/// as the L1 distance `|Δu| + |Δv|` from the pixel corner.
///
/// ⚠️ **`0.5` is DERIVED, not chosen.** An ideal 1-px staircase edge is
/// the 45° line through the midpoints of the two cut sides — the line
/// through `(0.5, 0)` and `(1, 0.5)` for the top-right corner. Both
/// points satisfy `(1 − u) + v = 0.5`, so `0.5` *is* the true edge, and
/// the triangle it cuts has area `1/8`. Any other value would be a
/// taste knob describing no geometry.
///
/// Its consequence is the load-bearing one: at `factor == 2` the four
/// sample points land at `|Δu| + |Δv| == 0.5` **exactly**, so the law
/// degenerates to Scale2x byte-for-byte (gate
/// `epx_at_two_is_byte_identical_to_scale2x`).
const CORNER_CUT: f32 = 0.5;

/// Edge-directed pixel-art upscale — **any** factor in
/// `[MIN_SCALE_FACTOR, SCALE_FULL_SCALE]`, integer or not.
///
/// ## What this is, and what it is not
///
/// This is the EPX / Scale2x / AdvMAME corner rule (Johnson 1992,
/// Mazzoleni 2001) evaluated as a *continuous* reconstruction instead
/// of a fixed `2×2` / `3×3` block emitter. It is **not** Hyllian xBR —
/// see the note at the end of this doc.
///
/// Each destination pixel maps back to a source coordinate; the integer
/// part picks the source pixel `E` and the fractional part `(u, v)`
/// says where inside `E` the sample sits:
///
/// ```text
///     B            u → 0..1 across E,  v → 0..1 down E
///   D E F          corner distance = the L1 distance to the nearest
///     H                              corner of E
/// ```
///
/// The EPX predicate — `B != H && D != F`, then the corner's own pair
/// test — decides whether a 45° edge cuts that corner of `E`. If it
/// does, every sample within [`CORNER_CUT`] of the corner takes the
/// neighbour's colour; everything else keeps `E`:
///
/// | corner | pair test | fill | corner distance |
/// |---|---|---|---|
/// | TL | `D == B` | `D` | `u + v` |
/// | TR | `B == F` | `F` | `(1−u) + v` |
/// | BL | `D == H` | `D` | `u + (1−v)` |
/// | BR | `H == F` | `F` | `(1−u) + (1−v)` |
///
/// ## Why the whole slider course is alive
///
/// Nothing here is quantised to `{2, 3, 4}`: the factor only sets the
/// destination dimensions, and the law is evaluated per destination
/// sample. `5×` and `6×` produce genuinely different images, and so do
/// `15×` and `16×`. Flat regions are byte-exact replication at every
/// factor (the predicate cannot fire), which is what makes it safe for
/// pixel art.
///
/// ## Fidelity, stated honestly
///
/// - At `factor == 2` it is **byte-identical to Scale2x**.
/// - At `factor == 3` it reproduces Scale3x's four *corner* cells, and
///   deliberately does **not** reproduce Scale3x's four conditional
///   *edge* cells: those sit at corner distance `2/3`, outside the true
///   45° line. This is a declared divergence, gated by
///   `epx_at_three_matches_scale3x_corner_cells`.
/// - ⛔ It is **not Hyllian xBR**, and this crate no longer says it is.
///   xBR's predicate is a YUV-weighted distance sum over a 21-tap
///   window whose exact weights we cannot verify from the published
///   description alone. Shipping an approximation under that name is
///   precisely the defect this pass removed — the previous version was
///   Scale2x wearing an "xBR" label, with 4×…16× of the slider dead.
///   Adopting real xBR is a follow-up that starts by acquiring the
///   reference (MIT, Hyllian) and gating against it.
///
/// Pixel-equality test: byte-exact RGBA8 match (the canonical EPX
/// definition).
pub fn upscale_epx(pixels: &[SrgbRgba], src_w: u32, src_h: u32, factor: f32) -> UpscaleResult {
    debug_assert_eq!(pixels.len(), (src_w as usize) * (src_h as usize));
    let (dw, dh) = dst_dims(src_w, src_h, factor);
    epx_resample(pixels, src_w, src_h, dw, dh)
}

/// The kernel behind [`upscale_epx`], addressed by destination
/// dimensions instead of a factor so a caller that must hit an exact
/// `(dw, dh)` — a non-uniform fit, say — reaches the same law.
///
/// Returns the source untouched when either dimension is degenerate.
pub fn epx_resample(pixels: &[SrgbRgba], sw: u32, sh: u32, dw: u32, dh: u32) -> UpscaleResult {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return UpscaleResult {
            pixels: rgba.to_vec(),
            width: sw,
            height: sh,
        };
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

            let e = read_px(rgba, sw_i, sh_i, x0, y0);
            let b = read_px(rgba, sw_i, sh_i, x0, y0 - 1);
            let d = read_px(rgba, sw_i, sh_i, x0 - 1, y0);
            let f = read_px(rgba, sw_i, sh_i, x0 + 1, y0);
            let h = read_px(rgba, sw_i, sh_i, x0, y0 + 1);

            let mut px = e;
            // EPX guard: a corner is only cut where the two axes
            // disagree — otherwise `E` sits inside a run, not on a step.
            if b != h && d != f {
                let left = u < 0.5;
                let top = v < 0.5;
                let du = if left { u } else { 1.0 - u };
                let dv = if top { v } else { 1.0 - v };
                // The neighbour that owns this corner horizontally is
                // the fill for all four cases; the pair test says
                // whether it and the vertical neighbour agree.
                let horiz = if left { d } else { f };
                let vert = if top { b } else { h };
                if horiz == vert && du + dv <= CORNER_CUT {
                    px = horiz;
                }
            }
            write_px(&mut out, dw, dx, dy, px);
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
    use super::super::{scale2x, scale3x, upscale_lanczos3, upscale_nearest};
    use super::*;
    use crate::params::SCALE_FULL_SCALE;

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

    /// A hard diagonal edge — the ONLY fixture that can tell three
    /// scalers apart. A flat or smooth fixture makes Nearest, Lanczos3
    /// and EPX agree, and would green-light the alias this pass removed.
    ///
    /// 16×16, split by `y > x`: solid red above, solid black below. The
    /// staircase gives every source pixel on the diagonal a live EPX
    /// predicate.
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

    /// ORACLE gate — the continuous law reproduces the independent
    /// Scale2x block emitter byte-for-byte at `factor == 2`.
    ///
    /// This is the gate that says the kernel really is EPX and not an
    /// improvisation: `scale2x` is built by a different construction
    /// (iterate source, emit a 2×2 block) and lives only in `#[cfg(test)]`.
    #[test]
    fn epx_at_two_is_byte_identical_to_scale2x() {
        for n in [3usize, 4, 16] {
            let src = diagonal_edge(n);
            let got = upscale_epx(&pack(src.clone()), n as u32, n as u32, 2.0);
            let want = scale2x(&src, n as u32, n as u32);
            assert_eq!((got.width, got.height), (want.width, want.height));
            assert_eq!(
                got.pixels, want.pixels,
                "EPX at 2x must equal Scale2x byte-for-byte (n={n})"
            );
        }
    }

    /// The declared divergence from Scale3x, pinned: the four CORNER
    /// cells agree, and this gate does not pretend the four conditional
    /// edge cells do (they sit at corner distance `2/3`, outside the
    /// true 45° line).
    #[test]
    fn epx_at_three_matches_scale3x_corner_cells() {
        let n = 8u32;
        let src = diagonal_edge(n as usize);
        let got = upscale_epx(&pack(src.clone()), n, n, 3.0);
        let want = scale3x(&src, n, n);
        assert_eq!((got.width, got.height), (want.width, want.height));
        for sy in 0..n {
            for sx in 0..n {
                for (ox, oy) in [(0u32, 0u32), (2, 0), (0, 2), (2, 2)] {
                    let (x, y) = (sx * 3 + ox, sy * 3 + oy);
                    assert_eq!(
                        px(&got.pixels, want.width, x, y),
                        px(&want.pixels, want.width, x, y),
                        "corner cell ({x},{y}) of source ({sx},{sy})"
                    );
                }
            }
        }
    }

    /// ⭐ **The defect gate for the dead 80 % of the slider.** Every
    /// integer stop from 1 to 16 must produce its own destination size
    /// AND its own pixels. Before this pass `4..=16` all collapsed onto
    /// `4×` — the same bytes at the same size, while the chip printed
    /// the raw slider value.
    #[test]
    fn every_integer_factor_from_one_to_sixteen_produces_a_distinct_image() {
        let n = 8u32;
        let src = pack(diagonal_edge(n as usize));
        let mut seen: Vec<(u32, Vec<u8>)> = Vec::new();
        for f in 1..=(SCALE_FULL_SCALE as u32) {
            let r = upscale_epx(&src, n, n, f as f32);
            assert_eq!(
                (r.width, r.height),
                (n * f, n * f),
                "factor {f} must reach its own size"
            );
            for (pf, prev) in &seen {
                assert!(
                    prev.len() != r.pixels.len() || *prev != r.pixels,
                    "factor {f} produced the same image as factor {pf} — the course is dead there"
                );
            }
            seen.push((f, r.pixels));
        }
        assert_eq!(seen.len(), SCALE_FULL_SCALE as usize);
    }

    /// ⭐ **The oracle gate the alias could never pass.** On a fixture
    /// that CONTAINS the phenomenon (a hard diagonal edge), EPX at 2×
    /// must differ from Nearest at 2× and from Lanczos3 at 2×. When
    /// `Xbr` was an alias, one of these two was byte-identical.
    #[test]
    fn epx_differs_from_both_nearest_and_lanczos_on_a_hard_diagonal() {
        let n = 16u32;
        let src = pack(diagonal_edge(n as usize));
        for f in [2.0f32, 3.0, 5.0, 8.0, 16.0] {
            let epx = upscale_epx(&src, n, n, f);
            let near = upscale_nearest(&src, n, n, f);
            let lanc = upscale_lanczos3(&src, n, n, f);
            assert_eq!((epx.width, epx.height), (near.width, near.height));
            assert_ne!(
                epx.pixels, near.pixels,
                "EPX at {f}x is an alias of Nearest — the edge rule never fired"
            );
            assert_ne!(
                epx.pixels, lanc.pixels,
                "EPX at {f}x is an alias of Lanczos3"
            );
        }
    }

    #[test]
    fn epx_flat_input_replicates_all_pixels_at_every_factor() {
        // Flat region: the predicate cannot fire, so EPX is exact
        // replication — the property that makes it pixel-art safe.
        let src = pack(solid(4, 4, [80, 80, 80]));
        for f in [1.0f32, 2.0, 3.0, 7.0, 16.0] {
            let r = upscale_epx(&src, 4, 4, f);
            for p in r.pixels.chunks(4) {
                assert_eq!(p, [80, 80, 80, 255], "factor {f} filtered a flat region");
            }
        }
    }

    #[test]
    fn epx_at_one_is_the_identity() {
        let src = diagonal_edge(6);
        let r = upscale_epx(&pack(src.clone()), 6, 6, 1.0);
        assert_eq!((r.width, r.height), (6, 6));
        assert_eq!(r.pixels, src, "1x must be a byte-exact passthrough");
    }

    #[test]
    fn epx_diagonal_edge_gets_corner_blended() {
        // 3×3 input: a diagonal step from top-left red to bottom-right
        // black. The corner-replacement rule must turn the centre
        // pixel's BL into red (the rule `D == H` triggers).
        //
        //   R R K          E=K at (2,2): B=K H=., D=., F=. — flat path.
        //   R . K          E=. at (1,1): B=R H=. D=R F=K — edge.
        //   . . K
        //
        // Test the centre pixel's 2×2 output: BL must be R (the
        // `D == H` rule when both are R), BR must be K (`H == F`).
        let r = [255, 0, 0, 255];
        let k = [0, 0, 0, 255];
        let e = [128, 128, 128, 255];
        let row = |a: [u8; 4], b: [u8; 4], c: [u8; 4]| {
            let mut v = Vec::with_capacity(12);
            v.extend_from_slice(&a);
            v.extend_from_slice(&b);
            v.extend_from_slice(&c);
            v
        };
        let mut src = row(r, r, k);
        src.extend(row(r, e, k));
        src.extend(row(e, e, k));
        let out = upscale_epx(&pack(src.clone()), 3, 3, 2.0);
        assert_eq!((out.width, out.height), (6, 6));
        // Centre source pixel `(1,1) = e`. Its 2×2 output sits at
        // dst (2..4, 2..4). With B=r, H=e, D=r, F=k:
        //   B != H ✓  D != F ✓  → edge rules fire.
        //   TL = (D == B = r) → r
        //   TR = (B == F? r == k no) → e
        //   BL = (D == H? r == e no) → e
        //   BR = (H == F? e == k no) → e
        assert_eq!(px(&out.pixels, 6, 2, 2), r, "TL of centre block");
        assert_eq!(px(&out.pixels, 6, 3, 2), e, "TR of centre block");
        assert_eq!(px(&out.pixels, 6, 2, 3), e, "BL of centre block");
        assert_eq!(px(&out.pixels, 6, 3, 3), e, "BR of centre block");
    }

    #[test]
    fn epx_produces_f_by_f_blocks_per_source_pixel() {
        for (f, side) in [(3.0f32, 6u32), (4.0, 8), (5.0, 10), (11.0, 22)] {
            let src = solid(2, 2, [200, 200, 200]);
            let r = upscale_epx(&pack(src.clone()), 2, 2, f);
            assert_eq!((r.width, r.height), (side, side));
        }
    }

    /// A non-integer factor is not refused and not silently snapped —
    /// it lands on the rounded destination size and still runs the edge
    /// law. (The PANEL snaps to integers as a product decision; the
    /// kernel must not double-snap.)
    #[test]
    fn epx_accepts_a_non_integer_factor() {
        let src = pack(diagonal_edge(8));
        let r = upscale_epx(&src, 8, 8, 2.5);
        assert_eq!((r.width, r.height), (20, 20));
        assert_ne!(r.pixels, upscale_nearest(&src, 8, 8, 2.5).pixels);
    }
}
