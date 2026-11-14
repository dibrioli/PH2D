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
            let s = sample_kind_px(kind, [u, v], params, image, [0, 0], fw, false).clamp(0.0, 1.0);
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

/// A prévia de um padrão PRESO À TELA (o papel): o recorte `span_px` × (`span_px`·`h`/`w`) píxeis da tela a 100 %,
/// lido pela MESMA [`crate::texture::sample_tiled`] que o motor amostra (B39.6) — a densidade é a da pintura, não
/// «~3 tiles por faixa». Cinzento opaco. No-op se `out` for curto.
pub fn render_tiled_preview(
    s: &crate::texture::TextureSettings,
    image: Option<&ImageMask>,
    out: &mut [u8],
    w: u32,
    h: u32,
    span_px: f32,
) {
    if w == 0 || h == 0 || out.len() < (w as usize) * (h as usize) * 4 {
        return;
    }
    let passo = span_px.max(1.0) / w as f32;
    let tela = |i: u32| ((i as f32 + 0.5) * passo).floor() as i64;
    for py in 0..h {
        for px in 0..w {
            let v = crate::texture::sample_tiled(s, tela(px), tela(py), image);
            let c = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let i = ((py * w + px) * 4) as usize;
            out[i..i + 4].copy_from_slice(&[c, c, c, 255]);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::texture::{TextureKind, TextureSettings, sample_tiled};

    /// B39.6: cada píxel da prévia presa à tela é a amostra do motor no píxel da tela que ele cobre (recorte a 100 %).
    #[test]
    fn a_previa_presa_a_tela_e_o_recorte_que_o_motor_pinta() {
        let s = TextureSettings {
            kind: TextureKind::Noise,
            size: [3.0, 3.0],
            angle_deg: 30,
            ..TextureSettings::default()
        };
        let (w, h, span) = (70u32, 20u32, 280.0f32);
        let mut out = vec![0u8; (w * h * 4) as usize];
        super::render_tiled_preview(&s, None, &mut out, w, h, span);
        for (py, px) in [(0u32, 0u32), (7, 33), (19, 69)] {
            let tela = |i: u32| ((i as f32 + 0.5) * span / w as f32).floor() as i64;
            let v = sample_tiled(&s, tela(px), tela(py), None);
            let c = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            assert_eq!(out[((py * w + px) * 4) as usize], c, "({px},{py})");
        }
    }
}
