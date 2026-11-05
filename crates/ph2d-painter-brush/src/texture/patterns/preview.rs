//! The live preview of a texture strip (child of [`super`], LOC cap).

use super::sample_kind_px;
use crate::ramp_alpha::RampAlphaMode;
use crate::texture::{ImageMask, TextureKind};

/// Fill `out` (RGBA8, `w`×`h`, row-major) with a live preview of texture `kind` shaped by
/// `params` / `size` / `offset` — the scalar pattern as it modulates the brush, View-plane mapped at
/// identity rotation (the preview ignores per-dab rotation + jitter). ~3 tiles across with square
/// cells. `image` backs [`TextureKind::Image`]; without it that kind previews flat.
///
/// With `ramp = None` the preview is **grayscale** (the raw scalar). With `ramp = Some((lut, mode))`
/// the scalar indexes the 256-entry **sRGB-straight RGBA** Color-Ramp `lut` and the result is shown
/// **with its alpha** over a checker (so transparency reads): `mode` picks how the alpha shows —
/// [`RampAlphaMode::None`] ignores it (opaque), the other two reveal the translucency. No-op if `out`
/// is too small.
#[allow(clippy::too_many_arguments)]
pub fn render_texture_preview(
    kind: TextureKind,
    params: [f32; crate::texture::MAX_TEX_PARAMS],
    size: [f32; 2],
    offset: [f32; 2],
    angle_deg: u16,
    image: Option<&ImageMask>,
    ramp: Option<(&[[f32; 4]], RampAlphaMode)>,
    out: &mut [u8],
    w: u32,
    h: u32,
) {
    if w == 0 || h == 0 || out.len() < (w as usize) * (h as usize) * 4 {
        return;
    }
    let sx = size[0].clamp(crate::texture::TEX_SIZE_MIN, crate::texture::TEX_SIZE_MAX);
    let sy = size[1].clamp(crate::texture::TEX_SIZE_MIN, crate::texture::TEX_SIZE_MAX);
    let step = 3.0 / w as f32; // procedural: ~3 tiles across, square cells
    // A largura de um píxel da faixa em células: os degraus leem-se pela área, como na tela.
    let fw = if crate::texture::pela_area::le_pela_area(kind) {
        step * sx.max(sy)
    } else {
        0.0
    };
    // An imported image previews as ONE centred picture filling the strip (footprint `−1..1`, matching
    // the dab at size 1), not the procedural 3-tile field — else it reads repeated + offset.
    let is_image = matches!(kind, TextureKind::Image);
    // The **Angle** rotates the sampled pattern about the strip centre (so the preview reflects the
    // texture rotation, matching the on-canvas render). Centre = 1.5 tile units (procedural) / 0 (image).
    let rot = crate::texture::rotate_by_degrees(angle_deg);
    let (ca, sa) = (rot[0], rot[1]);
    let centre = if is_image { 0.0 } else { 1.5 };
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    for py in 0..h {
        let bv0 = if is_image {
            (py as f32 + 0.5) / h as f32 * 2.0 - 1.0
        } else {
            (py as f32 + 0.5) * step
        } - centre;
        for px in 0..w {
            let bu0 = if is_image {
                (px as f32 + 0.5) / w as f32 * 2.0 - 1.0
            } else {
                (px as f32 + 0.5) * step
            } - centre;
            // Rotate the centred base coords, restore the centre, then apply Size + Offset.
            let bu = bu0 * ca - bv0 * sa + centre;
            let bv = bu0 * sa + bv0 * ca + centre;
            let u = bu * sx + offset[0];
            let v = bv * sy + offset[1];
            let s = sample_kind_px(kind, [u, v], params, image, [0, 0], fw).clamp(0.0, 1.0);
            let [r, g, b] = match ramp {
                Some((lut, mode)) if !lut.is_empty() => {
                    let idx = ((s * (lut.len() - 1) as f32) + 0.5) as usize;
                    let c = lut[idx.min(lut.len() - 1)];
                    // Off ignores the ramp alpha (opaque); Strength / Sprite Alpha show it as opacity
                    // over a checker so the transparency reads.
                    let alpha = if matches!(mode, RampAlphaMode::None) {
                        1.0
                    } else {
                        c[3].clamp(0.0, 1.0)
                    };
                    let bg = if (((px / 6) + (py / 6)) & 1) == 0 {
                        0.80
                    } else {
                        0.55
                    };
                    let over = |ch: f32| bg * (1.0 - alpha) + ch * alpha;
                    [over(c[0]), over(c[1]), over(c[2])]
                }
                _ => [s, s, s], // grayscale (the raw scalar)
            };
            let i = ((py * w + px) * 4) as usize;
            out[i] = enc(r);
            out[i + 1] = enc(g);
            out[i + 2] = enc(b);
            out[i + 3] = 255;
        }
    }
}
