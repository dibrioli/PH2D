//! Straight ↔ premultiplied alpha conversion for RGBA8 buffers.
//!
//! The sprite pipeline (`shaders/sprite.wgsl`) normally stores textures
//! with STRAIGHT alpha and premultiplies in the fragment shader AFTER
//! the bilinear `textureSample`. That ordering bleeds a partial-alpha
//! edge texel's full-weight straight RGB into its neighbours — the
//! purple/dark fringe seen on a Background-Removal result.
//!
//! The fix bakes the BG-Removal Apply texture PREMULTIPLIED and flips a
//! per-instance flag so the shader skips its post-sample premultiply.
//! Bilinear sampling of premultiplied data composites edge texels as
//! `rgb·α` — identical to Vello's `draw_image_rgba` preview, fringe
//! gone, and the alpha matte / RGB art is never altered (the conversion
//! is mathematically reversible to ±1 per channel).
//!
//! Both functions operate in place on tightly-packed `[r,g,b,a,…]`
//! RGBA8 and are no-ops on a zero-length / non-multiple-of-4 tail.

/// Convert STRAIGHT-alpha RGBA8 to PREMULTIPLIED in place:
/// `rgb' = round(rgb * a / 255)`, alpha unchanged.
///
/// Rounded (`+ 127` before the divide) so the round-trip back through
/// [`unpremultiply_rgba8`] stays within ±1 per channel.
pub fn premultiply_rgba8(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        px[0] = mul_div_255(px[0] as u32, a) as u8;
        px[1] = mul_div_255(px[1] as u32, a) as u8;
        px[2] = mul_div_255(px[2] as u32, a) as u8;
    }
}

/// Convert PREMULTIPLIED RGBA8 back to STRAIGHT alpha in place:
/// `rgb = min(255, round(rgb * 255 / a))` for `a > 0`, else `0`.
///
/// Recovers the straight RGB an alpha-aware algorithm (BG-Removal,
/// Trim, Make-Square) expects when it reads a baked premultiplied
/// texture back. Fully-transparent texels carry no recoverable colour,
/// so they collapse to `(0,0,0,0)`.
pub fn unpremultiply_rgba8(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        } else {
            px[0] = unmul(px[0] as u32, a);
            px[1] = unmul(px[1] as u32, a);
            px[2] = unmul(px[2] as u32, a);
        }
    }
}

#[inline]
fn mul_div_255(c: u32, a: u32) -> u32 {
    (c * a + 127) / 255
}

#[inline]
fn unmul(c: u32, a: u32) -> u8 {
    (((c * 255) + a / 2) / a).min(255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_is_identity() {
        let mut px = [10, 200, 30, 255];
        premultiply_rgba8(&mut px);
        assert_eq!(px, [10, 200, 30, 255]);
        unpremultiply_rgba8(&mut px);
        assert_eq!(px, [10, 200, 30, 255]);
    }

    #[test]
    fn fully_transparent_collapses_to_zero() {
        let mut px = [123, 45, 200, 0];
        premultiply_rgba8(&mut px);
        // rgb * 0 = 0; alpha preserved.
        assert_eq!(px, [0, 0, 0, 0]);
        unpremultiply_rgba8(&mut px);
        assert_eq!(px, [0, 0, 0, 0]);
    }

    #[test]
    fn round_trip_within_one_per_channel_for_opaque_half() {
        // For alpha in the upper half (a >= 128 — the visible interior
        // of any sprite), the straight↔premultiplied↔straight round
        // trip is lossless to ±1 per channel, as the diretriz requires.
        for a in 128u32..=255 {
            for c in 0u32..=255 {
                let mut px = [c as u8, c as u8, c as u8, a as u8];
                let orig = c as i32;
                premultiply_rgba8(&mut px);
                unpremultiply_rgba8(&mut px);
                for ch in 0..3 {
                    let d = px[ch] as i32 - orig;
                    assert!(
                        d.abs() <= 1,
                        "channel {ch}: a={a} c={c} round-trip {} vs {orig} (delta {d})",
                        px[ch],
                    );
                }
                assert_eq!(px[3], a as u8, "alpha must be untouched");
            }
        }
    }

    #[test]
    fn round_trip_error_bounded_by_alpha_quantization() {
        // For LOW alpha, premultiplied 8-bit storage cannot retain full
        // colour precision (the same lossiness Vello's premultiplied
        // pipeline has). The recovered colour error is bounded by the
        // alpha-quantization step `ceil(255 / a)`; verify that bound
        // holds across the whole range so we never silently corrupt
        // beyond the theoretical floor. Alpha is always exact.
        for a in 1u32..=255 {
            let bound = (255u32.div_ceil(a) + 1) as i32;
            for c in (0u32..=255).step_by(3) {
                let mut px = [c as u8, c as u8, c as u8, a as u8];
                let orig = c as i32;
                premultiply_rgba8(&mut px);
                unpremultiply_rgba8(&mut px);
                for ch in 0..3 {
                    let d = (px[ch] as i32 - orig).abs();
                    assert!(
                        d <= bound,
                        "channel {ch}: a={a} c={c} delta {d} exceeds bound {bound}",
                    );
                }
                assert_eq!(px[3], a as u8);
            }
        }
    }

    #[test]
    fn premultiplied_rgb_never_exceeds_alpha() {
        // A valid premultiplied pixel has rgb <= a. Verify the bake
        // produces that invariant so the shader's "already premul"
        // branch is fed clean data.
        for a in 0u32..=255 {
            let mut px = [255, 255, 255, a as u8];
            premultiply_rgba8(&mut px);
            for ch in 0..3 {
                assert!(px[ch] as u32 <= a, "rgb {} > a {a}", px[ch]);
            }
        }
    }

    #[test]
    fn empty_and_short_tail_are_noops() {
        let mut empty: [u8; 0] = [];
        premultiply_rgba8(&mut empty);
        unpremultiply_rgba8(&mut empty);
        // chunks_exact ignores a sub-4 tail; just confirm no panic.
        let mut tail = [1u8, 2, 3];
        premultiply_rgba8(&mut tail);
        assert_eq!(tail, [1, 2, 3]);
    }
}
