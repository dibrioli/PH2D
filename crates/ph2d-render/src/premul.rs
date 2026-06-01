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

/// Alpha storage representation of an RGBA8 buffer / texture. Making
/// this a TYPE (not a loose `bool premultiplied`) means a caller can
/// never silently mistake premultiplied bytes for straight ones — the
/// class of bug that corrupted Image-Tools results run after a
/// BG-Removal Apply (the result was un-premultiplied, the flag dropped,
/// and the fringe came back).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    /// RGB is independent of alpha — the editor's default texture format.
    Straight,
    /// RGB is already multiplied by alpha — the BG-Removal fringe-fix bake.
    Premultiplied,
}

impl AlphaMode {
    pub fn is_premultiplied(self) -> bool {
        matches!(self, AlphaMode::Premultiplied)
    }
    /// Map from `Sprite.premultiplied`.
    pub fn from_premultiplied_flag(flag: bool) -> Self {
        if flag {
            AlphaMode::Premultiplied
        } else {
            AlphaMode::Straight
        }
    }
}

/// An owned RGBA8 image that CARRIES its alpha representation. Image
/// tools read a sprite's pixels as a `SpriteImage` and hand one back; the
/// upload chokepoint derives `Sprite.premultiplied` from `alpha`, so the
/// representation can never drift from the bytes. Conversions are
/// explicit + reversible (see [`premultiply_rgba8`] / [`unpremultiply_rgba8`]).
#[derive(Clone, Debug)]
pub struct SpriteImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub alpha: AlphaMode,
}

impl SpriteImage {
    /// Construct from owned `SrgbRgba` pixels. Wave 11 color-space
    /// migration (ADR-0042 §6 #2): the public boundary is typed; the
    /// inner storage stays `Vec<u8>` for downstream byte-level
    /// consumers (GPU upload). The cast is zero-copy via
    /// `bytemuck::allocation::cast_vec` (`SrgbRgba` is repr-transparent
    /// + `Pod` over `[u8; 4]`).
    pub fn new(
        width: u32,
        height: u32,
        pixels: Vec<ph2d_color::SrgbRgba>,
        alpha: AlphaMode,
    ) -> Self {
        assert_eq!(
            pixels.len(),
            (width as usize) * (height as usize),
            "pixel count must equal width * height",
        );
        Self {
            width,
            height,
            pixels: bytemuck::allocation::cast_vec(pixels),
            alpha,
        }
    }

    /// Construct from owned byte buffer. Use [`Self::new`] (typed) at
    /// every IO boundary; this method is for downstream consumers that
    /// produce bytes natively (image decoders, GPU readback) where a
    /// typed-wrapper round-trip would be pure ceremony. Param is named
    /// `bytes` (not `pixels`) so the `arch_color_space_typed` gate
    /// recognises this is intentionally the byte path.
    pub fn from_bytes(width: u32, height: u32, bytes: Vec<u8>, alpha: AlphaMode) -> Self {
        Self {
            width,
            height,
            pixels: bytes,
            alpha,
        }
    }

    /// Convert to STRAIGHT alpha (no-op if already straight). Call before
    /// any algorithm that reasons about true colours (segmentation).
    ///
    /// Gamma-correct: uses [`unpremultiply_rgba8_in_linear`], the matched
    /// inverse of the linear bake in [`Self::into_premultiplied`], so a
    /// bake → re-edit round trip stays correct.
    pub fn into_straight(mut self) -> Self {
        if self.alpha == AlphaMode::Premultiplied {
            unpremultiply_rgba8_in_linear(&mut self.pixels);
            self.alpha = AlphaMode::Straight;
        }
        self
    }

    /// Convert to PREMULTIPLIED alpha (no-op if already premultiplied).
    ///
    /// Gamma-correct premultiply ([`premultiply_rgba8_in_linear`]): the bake
    /// stores `srgb_encode(rgb_linear · a_linear)`, which the sprite shader
    /// (sampling `Rgba8UnormSrgb` → hw sRGB-decode) reads back as the correct
    /// linear premultiplied value `rgb_linear · a_linear`. The old byte-space
    /// `rgb · a` undershot after hw-decode, darkening translucent edges (the
    /// "halo"). This is the canonical bake for every image-tool Apply path, so
    /// the fix lands once here.
    pub fn into_premultiplied(mut self) -> Self {
        if self.alpha == AlphaMode::Straight {
            premultiply_rgba8_in_linear(&mut self.pixels);
            self.alpha = AlphaMode::Premultiplied;
        }
        self
    }
}

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

/// Gamma-correct premultiplied alpha: sRGB-decode each channel → multiply
/// by linear alpha → sRGB-encode. The result is bytes whose interpretation
/// as sRGB-encoded RGB equals `rgb_linear * a_linear` — exactly what a
/// gamma-correct compositor expects.
///
/// The canonical premultiply for every image-tool Apply path (via
/// [`SpriteImage::into_premultiplied`]) and the live previews that mirror
/// it (Painter, BG-Removal). The straight `premultiply_rgba8` produced
/// bytes that the sprite shader (sampling `Rgba8UnormSrgb` — hw decode →
/// linear bilinear) read as `srgb_decode(srgb(rgb)·a) ≠ rgb_linear·a`,
/// undershooting and darkening translucent edges (the silhouette "halo").
/// Pre-encoding in linear closes that gap: the sprite hw-decode recovers
/// exactly `rgb_linear · a_linear`, the value premultiplied-alpha blending
/// expects. Its matched inverse is [`unpremultiply_rgba8_in_linear`].
///
/// ~10× the CPU cost of `premultiply_rgba8` (3 f32 srgb_to_linear +
/// 3 multiplies + 3 linear_to_srgb per pixel) — still <50 ms at 1K²
/// on M-series, well within the per-frame budget already paid for the
/// full-res preview pipeline.
pub fn premultiply_rgba8_in_linear(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as f32 / 255.0;
        for ch in px[..3].iter_mut() {
            let srgb = *ch as f32 / 255.0;
            let linear = srgb_to_linear(srgb);
            let premul = linear * a;
            let out_srgb = linear_to_srgb(premul);
            *ch = (out_srgb * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
    }
}

/// IEC 61966-2-1 sRGB → linear transfer.
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// IEC 61966-2-1 linear → sRGB transfer.
#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
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

/// Gamma-correct inverse of [`premultiply_rgba8_in_linear`]: sRGB-decode each
/// channel → divide by linear alpha → sRGB-encode. Recovers the straight sRGB
/// colour from bytes that encode `rgb_linear · a_linear`.
///
/// This is the matched partner of [`premultiply_rgba8_in_linear`]: the
/// straight↔premultiplied round trip used by the re-edit path (bake →
/// `into_premultiplied`, later read back → `into_straight`) stays gamma-correct
/// only if BOTH halves use the linear variants. Fully-transparent texels carry
/// no recoverable colour and collapse to `(0,0,0,0)`. At low alpha the 8-bit
/// premultiplied storage cannot retain full colour precision (the division
/// amplifies quantisation — the same inherent loss [`unpremultiply_rgba8`] has);
/// alpha is always exact.
pub fn unpremultiply_rgba8_in_linear(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            continue;
        }
        let a_lin = a as f32 / 255.0;
        for ch in px[..3].iter_mut() {
            let premul = srgb_to_linear(*ch as f32 / 255.0);
            // Clamp guards a stored premul byte that rounded slightly above
            // `linear · a_lin`, which would otherwise yield linear > 1.
            let linear = (premul / a_lin).min(1.0);
            let out_srgb = linear_to_srgb(linear);
            *ch = (out_srgb * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
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
                for (ch, &v) in px[..3].iter().enumerate() {
                    let d = v as i32 - orig;
                    assert!(
                        d.abs() <= 1,
                        "channel {ch}: a={a} c={c} round-trip {v} vs {orig} (delta {d})",
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
                for (ch, &v) in px[..3].iter().enumerate() {
                    let d = (v as i32 - orig).abs();
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
            for &v in &px[..3] {
                assert!(v as u32 <= a, "rgb {v} > a {a}");
            }
        }
    }

    #[test]
    fn sprite_image_conversions_flip_mode_and_are_idempotent() {
        let px = vec![10u8, 200, 30, 128, 0, 0, 0, 0];
        let straight = SpriteImage::from_bytes(2, 1, px.clone(), AlphaMode::Straight);
        // Straight -> premultiplied flips the mode and premultiplies RGB.
        let pre = straight.clone().into_premultiplied();
        assert_eq!(pre.alpha, AlphaMode::Premultiplied);
        assert_ne!(pre.pixels, px, "RGB should have been premultiplied");
        // Idempotent: into_premultiplied again is a no-op.
        let pre2 = pre.clone().into_premultiplied();
        assert_eq!(pre2.pixels, pre.pixels);
        assert_eq!(pre2.alpha, AlphaMode::Premultiplied);
        // Round-trip back to straight (a>=128 interior is lossless to ±1).
        let back = pre.into_straight();
        assert_eq!(back.alpha, AlphaMode::Straight);
        assert!((back.pixels[0] as i32 - 10).abs() <= 1);
        // into_straight on an already-straight image is a no-op.
        let s2 = straight.clone().into_straight();
        assert_eq!(s2.pixels, px);
        assert_eq!(s2.alpha, AlphaMode::Straight);
    }

    #[test]
    fn alpha_mode_flag_round_trip() {
        assert_eq!(
            AlphaMode::from_premultiplied_flag(true),
            AlphaMode::Premultiplied
        );
        assert_eq!(
            AlphaMode::from_premultiplied_flag(false),
            AlphaMode::Straight
        );
        assert!(AlphaMode::Premultiplied.is_premultiplied());
        assert!(!AlphaMode::Straight.is_premultiplied());
    }

    #[test]
    fn linear_premul_round_trips_for_visible_alpha() {
        // The gamma-correct pair (premultiply_rgba8_in_linear ↔
        // unpremultiply_rgba8_in_linear) must round-trip straight colour for
        // the visible-interior alpha range, just like the byte-space pair —
        // otherwise the bake → re-edit cycle (into_premultiplied →
        // into_straight) would corrupt colour. Allow ±2 (one extra step for
        // the two sRGB transfer round-trips on top of the alpha-division).
        for a in 128u32..=255 {
            for c in (0u32..=255).step_by(5) {
                let mut px = [c as u8, c as u8, c as u8, a as u8];
                let orig = c as i32;
                premultiply_rgba8_in_linear(&mut px);
                unpremultiply_rgba8_in_linear(&mut px);
                for (ch, &v) in px[..3].iter().enumerate() {
                    let d = (v as i32 - orig).abs();
                    assert!(
                        d <= 2,
                        "channel {ch}: a={a} c={c} linear round-trip {v} vs {orig} (delta {d})",
                    );
                }
                assert_eq!(px[3], a as u8, "alpha must be untouched");
            }
        }
    }

    #[test]
    fn linear_premul_recovers_linear_premultiplied_invariant() {
        // After hw sRGB-decode the stored bytes satisfy the premultiplied
        // invariant in LINEAR space: rgb_linear <= a_linear — what the sprite
        // shader's blend needs. The BYTE-space invariant (rgb_byte <= a_byte)
        // deliberately does NOT hold for the linear bake (linear→sRGB encode
        // brightens), and must not — that byte-space bound was the halo bug.
        // Tolerance allows one byte of sRGB rounding (~0.004 linear at the top,
        // where srgb_to_linear's slope ≈ 2.3).
        for a in 0u32..=255 {
            let mut px = [255u8, 255, 255, a as u8];
            premultiply_rgba8_in_linear(&mut px);
            let a_lin = a as f32 / 255.0;
            for &v in &px[..3] {
                let rgb_lin = srgb_to_linear(v as f32 / 255.0);
                assert!(
                    rgb_lin <= a_lin + 0.01,
                    "decoded rgb_linear {rgb_lin} > a_linear {a_lin} (a={a}, byte={v})",
                );
            }
        }
    }

    #[test]
    fn linear_fully_transparent_collapses_to_zero() {
        let mut px = [123u8, 45, 200, 0];
        premultiply_rgba8_in_linear(&mut px);
        assert_eq!(px, [0, 0, 0, 0]);
        unpremultiply_rgba8_in_linear(&mut px);
        assert_eq!(px, [0, 0, 0, 0]);
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
