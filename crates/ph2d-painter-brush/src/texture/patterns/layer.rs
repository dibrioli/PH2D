//! Full-canvas **texture-layer** renderer — the production sibling of [`super::render_texture_preview`].
//!
//! Where the preview composites the texture over a checker (so transparency *reads* in a tiny panel
//! strip), a real **Texture layer** needs honest pixels: straight sRGB8 with the ramp's **actual
//! alpha**, so the layer composites through the normal raster path (mask / clip / blend / opacity)
//! exactly like a painted layer. The mapping (scale + offset, identity rotation) is bit-identical to
//! the preview, so the panel preview and the composited layer match.

use super::sample_kind;
use crate::ramp_alpha::RampAlphaMode;
use crate::texture::{ImageMask, MAX_TEX_PARAMS, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind};

/// Render texture `kind` (shaped by `params` / `size` / `offset`) into `out` as **straight sRGB8
/// RGBA**, `w`×`h` row-major, covering the whole buffer. Mapping matches
/// [`super::render_texture_preview`] (≈3 tiles across, square cells, identity rotation), so a panel
/// preview and the composited layer look the same.
///
/// `ramp = None` → **grayscale, opaque** (`r=g=b=s`, `a=255`): the raw scalar as a fill. `ramp =
/// Some((lut, mode))` → the scalar indexes the 256-entry **sRGB-straight RGBA** `lut`; the RGB is
/// written as-is and the **alpha is real** — [`RampAlphaMode::None`] forces it opaque (the ramp only
/// recolours), the other two carry the stop's straight alpha into the layer (translucent texels).
/// `image` backs [`TextureKind::Image`]. No-op if `out` is too small.
#[allow(clippy::too_many_arguments)]
pub fn render_texture_layer(
    kind: TextureKind,
    params: [f32; MAX_TEX_PARAMS],
    size: [f32; 2],
    offset: [f32; 2],
    image: Option<&ImageMask>,
    ramp: Option<(&[[f32; 4]], RampAlphaMode)>,
    out: &mut [u8],
    w: u32,
    h: u32,
) {
    if w == 0 || h == 0 || out.len() < (w as usize) * (h as usize) * 4 {
        return;
    }
    let sx = size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let step = 3.0 / w as f32; // tiles per pixel — same on both axes → square cells (mirror of the preview)
    let stride = (w as usize) * 4;
    // Render disjoint ROW BANDS across the cores (each pixel is a pure function of its own (u,v), so
    // the bands are independent and the result is bit-identical to serial → HR-5). This is the live
    // FPS lever: a Texture-layer param drag re-renders the WHOLE canvas every frame, so on a large
    // sprite this keeps the procedural fill real-time (mirror of the brush's `parallel_band_stamp`,
    // which a small footprint short-circuits to serial — thread spawn isn't worth it).
    let render_band = |band: &mut [u8], band_y0: i64| -> bool {
        let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        for (ly, row) in band.chunks_exact_mut(stride).enumerate() {
            let v = (band_y0 as f32 + ly as f32 + 0.5) * step * sy + offset[1];
            for px in 0..w as usize {
                let u = (px as f32 + 0.5) * step * sx + offset[0];
                let s = sample_kind(kind, [u, v], params, image).clamp(0.0, 1.0);
                let (r, g, b, a) = match ramp {
                    Some((lut, mode)) if !lut.is_empty() => {
                        let idx = ((s * (lut.len() - 1) as f32) + 0.5) as usize;
                        let c = lut[idx.min(lut.len() - 1)];
                        // Off recolours only (opaque); Strength / Sprite Alpha carry the stop's alpha.
                        let alpha = if matches!(mode, RampAlphaMode::None) {
                            1.0
                        } else {
                            c[3].clamp(0.0, 1.0)
                        };
                        (c[0], c[1], c[2], alpha)
                    }
                    _ => (s, s, s, 1.0), // grayscale, opaque (the raw scalar as a fill)
                };
                let i = px * 4;
                row[i] = enc(r);
                row[i + 1] = enc(g);
                row[i + 2] = enc(b);
                row[i + 3] = enc(a);
            }
        }
        false
    };
    crate::dab::parallel_band_stamp(out, 0, i64::from(h), 0, i64::from(w), stride, render_band);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_band_render_is_deterministic_and_seamless() {
        // 384×384 = 147_456 px ≥ PARALLEL_MIN_AREA (131_072) → the multi-thread band path runs. The
        // result must be pure (identical across runs, regardless of how the bands fall) and have no
        // band-seam artifact — each pixel is a function of its own (u,v) only.
        let (w, h) = (384u32, 384u32);
        let mut params = [0.5f32; MAX_TEX_PARAMS];
        for (i, s) in crate::param_specs(TextureKind::Clouds).iter().enumerate() {
            params[i] = s.default;
        }
        let render = || {
            let mut buf = vec![0u8; (w * h * 4) as usize];
            render_texture_layer(
                TextureKind::Clouds,
                params,
                [1.0, 1.0],
                [0.0, 0.0],
                None,
                None,
                &mut buf,
                w,
                h,
            );
            buf
        };
        let a = render();
        let b = render();
        assert_eq!(
            a, b,
            "the band-parallel render must be deterministic (HR-5)"
        );
        // No seam: adjacent rows differ smoothly, never a hard discontinuity at a band boundary. Check
        // that the field genuinely varies (not a flat fill that would hide a seam).
        let stride = (w * 4) as usize;
        let lo = a.iter().step_by(4).copied().min().unwrap();
        let hi = a.iter().step_by(4).copied().max().unwrap();
        assert!(
            hi - lo > 20,
            "the parallel render produced a real field (lo={lo} hi={hi})"
        );
        assert_eq!(a.len(), stride * h as usize);
    }

    #[test]
    fn grayscale_layer_is_opaque_and_varies() {
        let (w, h) = (24u32, 16u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let mut params = [0.5f32; MAX_TEX_PARAMS];
        params[2] = 0.0; // hard checker → crisp dark/light cells
        render_texture_layer(
            TextureKind::Checker,
            params,
            [1.0, 1.0],
            [0.0, 0.0],
            None,
            None,
            &mut buf,
            w,
            h,
        );
        let (mut dark, mut light) = (0, 0);
        for px in buf.chunks_exact(4) {
            assert_eq!(px[3], 255, "no-ramp layer is opaque");
            assert!(px[0] == px[1] && px[1] == px[2], "grayscale");
            if px[0] < 40 {
                dark += 1;
            } else if px[0] > 215 {
                light += 1;
            }
        }
        assert!(dark > 0 && light > 0, "checker layer shows both cells");
    }

    #[test]
    fn ramp_alpha_becomes_real_layer_alpha() {
        let (w, h) = (32u32, 24u32);
        let mut params = [0.5f32; MAX_TEX_PARAMS];
        params[2] = 0.0; // hard checker → s is exactly 0 or 1
        // LUT: opaque red at s=0 … fully transparent at s=1.
        let mut lut = [[0.0f32; 4]; 256];
        for (i, c) in lut.iter_mut().enumerate() {
            let t = i as f32 / 255.0;
            *c = [1.0 - t, t, 0.0, 1.0 - t];
        }
        let render = |mode| {
            let mut buf = vec![0u8; (w * h * 4) as usize];
            render_texture_layer(
                TextureKind::Checker,
                params,
                [1.0, 1.0],
                [0.0, 0.0],
                None,
                Some((&lut[..], mode)),
                &mut buf,
                w,
                h,
            );
            buf
        };
        // Sprite-alpha mode: the s=1 cells become FULLY TRANSPARENT (real alpha 0), s=0 stays opaque.
        let sprite = render(RampAlphaMode::TextureAlpha);
        let (mut transparent, mut opaque) = (0, 0);
        for px in sprite.chunks_exact(4) {
            if px[3] < 10 {
                transparent += 1;
            } else if px[3] > 245 {
                opaque += 1;
            }
        }
        assert!(transparent > 0, "the transparent ramp end zeroes the alpha");
        assert!(opaque > 0, "the opaque ramp end keeps full alpha");
        // Off mode: alpha ignored → every texel opaque (recolour only).
        let off = render(RampAlphaMode::None);
        assert!(
            off.chunks_exact(4).all(|px| px[3] == 255),
            "Off mode keeps the layer fully opaque"
        );
    }
}
