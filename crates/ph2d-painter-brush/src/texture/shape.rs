//! The **Shape** slot's silhouette sampling (Procreate "Shape"): unlike the Grain (which modulates
//! and tiles), the Shape reads as ONE finite tip — the footprint maps to `[-1, 1]²`, outside it the
//! coverage is `0`, and an image is clamped at its border (never tiled). Split from `texture.rs` to
//! keep that file under the workspace LOC cap.

use super::{
    ImageMask, MAX_TEX_PARAMS, TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureKind,
    TextureMapping, TextureSettings,
};

/// A borrowed **straight RGB** image (3 bytes per texel, row-major), aligned with a sibling [`ImageMask`]
/// (same `width`/`height`). Supplied to the per-layer-colour real-colour path so a captured Shape layer
/// can paint with its OWN per-pixel colour instead of a flat tint. Sampled with the SAME centre-coord
/// clamped bilinear as the silhouette, so colour + coverage line up texel-for-texel.
#[derive(Clone, Copy, Debug)]
pub struct ImageRgb<'a> {
    /// Straight RGB bytes, row-major, at least `width * height * 3` long.
    pub rgb: &'a [u8],
    /// Image width in texels.
    pub width: u32,
    /// Image height in texels.
    pub height: u32,
}

/// The Shape **silhouette composition** rule, shared by the engine ([`crate::BrushSpec::compose_shape_silhouette`])
/// and the panel's live preview ([`render_shape_preview`]): an `Image` kind IS the sampled tip (it
/// REPLACES the falloff, staying crisp); any procedural kind is `falloff × sample` (MASKED by the
/// round envelope). `None` is never composed here (the caller uses the bare falloff).
#[must_use]
pub fn compose_shape_silhouette_kind(kind: TextureKind, sample: f32, falloff: f32) -> f32 {
    if kind == TextureKind::Image {
        sample
    } else {
        falloff * sample
    }
}

/// Remap a raw Shape silhouette value `sv ∈ [0, 1]` through the Shape's **value ramp** LUT (256 entries,
/// grayscale `[0, 1]`), or return it unchanged when no ramp is active. The Shape ramp is **B&W** — it
/// reshapes the silhouette's tonal response (contrast / invert / levels) BEFORE the falloff composition,
/// orthogonally to the Grain's colour ramp (Shape owns the silhouette/tone, Grain owns colour). Applied
/// at every composition site (per-pixel, stamp bake, canvas-cached) + the preview, so they agree.
#[must_use]
pub fn remap_shape_value(sv: f32, lut: Option<&[f32]>) -> f32 {
    match lut {
        Some(l) if !l.is_empty() => {
            let n = l.len();
            let idx = (sv.clamp(0.0, 1.0) * (n as f32 - 1.0) + 0.5) as usize;
            l[idx.min(n - 1)]
        }
        _ => sv,
    }
}

/// Sample the Shape's raw silhouette value at canvas pixel `(px, py)`. An **Image** kind reads the
/// finite clipped tip ([`sample_shape`] — bounded to `[-1, 1]²`, since an image IS a finite tip). A
/// **procedural** kind reads the pattern the GRAIN way ([`crate::texture::sample`]) — mapped across the
/// whole dab, so Size / Offset / Angle transform only the pattern (the falloff is the boundary), exactly
/// like the Grain. The caller multiplies a procedural value by the falloff ([`compose_shape_silhouette_kind`]).
#[must_use]
pub fn sample_shape_silhouette(
    s: &TextureSettings,
    b: &TexDabBasis,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
    image: Option<&ImageMask>,
) -> f32 {
    if s.kind == TextureKind::Image {
        sample_shape(s, b, px, py, center, radius, image)
    } else {
        super::sample(s, b, px, py, center, radius, image)
    }
}

/// Unit-coord (`[-1, 1]`) form of [`sample_shape_silhouette`], for the scale-invariant stamp bake +
/// the panel preview.
#[must_use]
pub fn sample_shape_silhouette_unit(
    s: &TextureSettings,
    b: &TexDabBasis,
    u: f32,
    v: f32,
    image: Option<&ImageMask>,
) -> f32 {
    if s.kind == TextureKind::Image {
        sample_shape_unit(s, b, u, v, image)
    } else {
        super::sample_unit(s, b, u, v, image)
    }
}

/// Render the composed Shape **silhouette** (the Procreate tip) into a `w×h` grayscale buffer (`1`
/// byte/texel = coverage `0..=255`), for the panel's live Shape preview. Uses the SAME composition as
/// the engine — `Image` is the sampled tip, a procedural kind is `falloff × pattern` — so the preview
/// is faithful. `falloff_lut[i]` is the falloff weight at radial `t = i/(N−1)` (`t ∈ [0,1]`), baked by
/// the panel from its live falloff. Static frame (Angle only; Rake needs a live stroke direction).
/// Deterministic (HR-5).
#[allow(clippy::too_many_arguments)]
pub fn render_shape_preview(
    kind: TextureKind,
    angle_deg: u16,
    offset: [f32; 2],
    size: [f32; 2],
    params: [f32; MAX_TEX_PARAMS],
    footprint: crate::footprint::FootprintDeform,
    falloff_lut: &[f32],
    shape_ramp_lut: Option<&[f32]>,
    image: Option<&ImageMask>,
    buf: &mut [u8],
    w: u32,
    h: u32,
) {
    let shape = TextureSettings {
        kind,
        mapping: TextureMapping::ViewPlane,
        angle_deg,
        rake: false,
        // The preview is a static frame (no stroke), so Flow (arc-length along the stroke) is inert here.
        flow: false,
        offset,
        size,
        // Shape silhouette is always ViewPlane — the stencil frame is inert here.
        stencil_offset: [0.0, 0.0],
        stencil_size: [0.5, 0.5],
        stencil_angle_deg: 0,
        params,
    };
    // The dab flatten/rotate deforms the preview exactly as the engine deforms the dab.
    // The preview has no path, so the FLOW frame is `Static` (and `flow` is forced off above anyway).
    let basis = super::shape_basis(
        &shape,
        &mut 0u64,
        [1.0, 1.0],
        footprint,
        super::ShapeFrame::Static,
    );
    let n = falloff_lut.len().max(1);
    let (inv_w, inv_h) = (2.0 / w.max(1) as f32, 2.0 / h.max(1) as f32);
    for j in 0..h {
        let v = (j as f32 + 0.5) * inv_h - 1.0;
        for i in 0..w {
            let u = (i as f32 + 0.5) * inv_w - 1.0;
            let raw = sample_shape_silhouette_unit(&shape, &basis, u, v, image);
            let sv = remap_shape_value(raw, shape_ramp_lut);
            let t = footprint.falloff_t(u, v).min(1.0);
            let f = falloff_lut[((t * (n - 1) as f32) as usize).min(n - 1)];
            let cov = compose_shape_silhouette_kind(kind, sv, f).clamp(0.0, 1.0);
            buf[(j * w + i) as usize] = (cov * 255.0 + 0.5) as u8;
        }
    }
}

/// Sample the **Shape** slot's silhouette alpha at canvas pixel `(px, py)` for a dab centred at
/// `center` with `radius`. Returns the silhouette coverage in `[0, 1]`. Only called when the Shape is
/// active (else the caller uses the falloff).
#[must_use]
pub fn sample_shape(
    s: &TextureSettings,
    b: &TexDabBasis,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
    image: Option<&ImageMask>,
) -> f32 {
    let p = [px as f32 + 0.5, py as f32 + 0.5];
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let r = radius.max(1e-3);
    if s.flow {
        // FLOW: stream the tip ALONG the stroke — the along coordinate (arc-length + the pixel's position
        // in the tangent-oriented footprint) TILES the image (period 2, one copy per `[-1,1]` span) so the
        // tip repeats down the path with a phase continuous across dabs; the across coordinate
        // (perpendicular) CLAMPS to the tip, masking to the stroke width. A pattern-line brush; a
        // non-tileable tip shows a seam (use a tileable image). Same frame as the procedural sibling in
        // [`crate::texture::sample`] — the footprint carries the tangent, the unit is stroke-constant.
        let (arc_len, unit) = b.frame.parts();
        let f = b
            .footprint
            .apply([(p[0] - center[0]) / unit, (p[1] - center[1]) / unit]);
        let rel = [arc_len / unit + f[0], f[1]];
        let along = (rel[0] * b.u[0] + rel[1] * b.u[1]) * sx + s.offset[0];
        let across = (rel[0] * b.v[0] + rel[1] * b.v[1]) * sy + s.offset[1];
        if across.abs() > 1.0 {
            return 0.0; // outside the tip's height → past the stroke edge
        }
        let along_wrapped = along - 2.0 * ((along + 1.0) * 0.5).floor(); // period-2 wrap → [-1, 1)
        return image
            .map(|img| sample_image_clamped(img, along_wrapped, across))
            .unwrap_or(0.0);
    }
    // Dab flatten/rotate deforms the footprint first, so the Shape tip flattens with the falloff.
    let f = b
        .footprint
        .apply([(p[0] - center[0]) / r, (p[1] - center[1]) / r]);
    shape_value(s, b, f, [sx, sy], image)
}

/// As [`sample_shape`] but at the dab-relative unit coord `(u, v) ∈ [-1, 1]` — the scale-invariant
/// form used to bake the cached stamp ([`crate::stamp::render_stamp_mask`]). Only called when the Shape
/// is active.
#[must_use]
pub fn sample_shape_unit(
    s: &TextureSettings,
    b: &TexDabBasis,
    u: f32,
    v: f32,
    image: Option<&ImageMask>,
) -> f32 {
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let f = b.footprint.apply([u, v]);
    shape_value(s, b, f, [sx, sy], image)
}

/// Os quatro cantos da ponta de imagem `|tex| ≤ 1` de [`shape_value`] desfeitos o Offset e o Size —
/// ainda no referencial RODADO da Shape. `None` quando a ponta não é esse quadrado: um Shape
/// procedural multiplica pelo falloff (o círculo manda) e o Flow ladrilha ao longo do traço.
fn image_tip_corners(s: &TextureSettings) -> Option<[[f32; 2]; 4]> {
    if s.kind != TextureKind::Image || s.flow {
        return None;
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let q = |a: f32, b: f32| [(a - s.offset[0]) / sx, (b - s.offset[1]) / sy];
    Some([q(-1.0, -1.0), q(1.0, -1.0), q(-1.0, 1.0), q(1.0, 1.0)])
}

/// ⭐⭐ **A PORTA DO ALCANCE da ponta de imagem** (Bug #52): a meia-caixa, em unidades de raio e nos eixos
/// da TELA, que contém o quadrado `|tex| ≤ 1` de [`shape_value`] levado de volta pelo Offset, pelo Size,
/// pela rotação da Shape (`b.u`/`b.v`) e pela pegada do dab (`b.footprint`, que carrega o Jitter Rotate e
/// o achatamento). `None` = a ponta não é um quadrado ([`image_tip_corners`]); o chamador fica com a
/// caixa da pegada.
///
/// ⚠️ Superconjunto declarado, como o [`crate::FootprintDeform::extent`]: exacto numa pegada plana (os
/// cantos levados à tela), pela curva de nível com margem numa curva. A `0°`, Size `1`, sem Offset dá
/// `[1, 1]` ao bit — a caixa de sempre.
#[must_use]
pub fn image_tip_extent(s: &TextureSettings, b: &TexDabBasis) -> Option<[f32; 2]> {
    let q = image_tip_corners(s)?;
    // `tex = M·rel·size + offset`, com as linhas de `M` = `u`, `v` ⇒ `rel = M⁻¹·q`.
    let det = b.u[0] * b.v[1] - b.u[1] * b.v[0];
    if det.abs() <= 1e-6 || det.is_nan() {
        return None;
    }
    let rel = q.map(|c| {
        [
            (b.v[1] * c[0] - b.u[1] * c[1]) / det,
            (b.u[0] * c[1] - b.v[0] * c[0]) / det,
        ]
    });
    if !b.footprint.is_flat_footprint() {
        let rho = rel
            .iter()
            .map(|r| (r[0] * r[0] + r[1] * r[1]).sqrt())
            .fold(0.0_f32, f32::max);
        return Some(b.footprint.extent_at(rho));
    }
    let mut e = [0.0_f32; 2];
    for r in rel {
        let p = b.footprint.unapply_linear(r);
        e = [e[0].max(p[0].abs()), e[1].max(p[1].abs())];
    }
    Some(e)
}

/// O alcance da ponta de imagem em raios INVARIANTE à rotação (a da Shape e a do dab): o raio do disco
/// que a contém em qualquer ângulo — o que as regiões do tool usam, que não conhecem a rotação sorteada
/// de cada dab. `≥` qualquer componente de [`image_tip_extent`] numa pegada plana (o achatamento só
/// encolhe). `None` como em [`image_tip_corners`].
#[must_use]
pub fn image_tip_radius(s: &TextureSettings) -> Option<f32> {
    image_tip_corners(s).map(|q| {
        q.iter()
            .map(|c| (c[0] * c[0] + c[1] * c[1]).sqrt())
            .fold(0.0_f32, f32::max)
    })
}

/// Shared core of [`sample_shape`] / [`sample_shape_unit`]: rotate the scaled footprint coord `rel`
/// into the shape's frame (basis already carries the rotation), translate by Offset, then read the
/// silhouette. Outside the `[-1, 1]²` tip → `0`. Image kind clamps to the border (one finite copy);
/// an active procedural kind reads its (bounded) value; no Image pixels → `0` (inert, no slab).
fn shape_value(
    s: &TextureSettings,
    b: &TexDabBasis,
    rel: [f32; 2],
    size: [f32; 2],
    image: Option<&ImageMask>,
) -> f32 {
    // Size scales AFTER the rotation (Blender's order) — see the note in `crate::texture::sample`.
    let tex = [
        (rel[0] * b.u[0] + rel[1] * b.u[1]) * size[0] + s.offset[0],
        (rel[0] * b.v[0] + rel[1] * b.v[1]) * size[1] + s.offset[1],
    ];
    if tex[0].abs() > 1.0 || tex[1].abs() > 1.0 {
        return 0.0;
    }
    match s.kind {
        TextureKind::Image => image
            .map(|img| sample_image_clamped(img, tex[0], tex[1]))
            .unwrap_or(0.0),
        TextureKind::None => 1.0, // not reached (the caller uses the falloff when Shape is inactive)
        _ => super::patterns::sample_kind(s.kind, tex, s.params, image).clamp(0.0, 1.0),
    }
}

/// Bilinear sample of `img`'s luminance, **clamped** to the image edges (not tiled), for footprint
/// coords already bounded to `[-1, 1]` by the caller. The Shape reads as ONE finite tip; outside the
/// image the caller returns `0`, and inside we clamp the bilinear taps to the border. Centre-coord
/// bilinear; a malformed buffer reads `0.0` (an inert silhouette, so the dab paints nothing).
/// The **straight-RGB** sibling of [`sample_shape_unit`]: at the dab-relative unit coord `(u, v)`, map
/// through the SAME footprint / Size / rotation / Offset frame as the silhouette, then read the layer's
/// own per-pixel colour from `rgb`. Outside the `[-1, 1]²` tip → `[0, 0, 0]` (the caller weights it by
/// the silhouette alpha, which is also `0` there). Only meaningful for an `Image` Shape (a captured
/// layer); a procedural Shape has no per-pixel colour, so it returns the fallback `[0, 0, 0]`.
/// Deterministic (HR-5: `+ − × ÷` + the shared bilinear).
#[must_use]
pub fn sample_shape_rgb_unit(
    s: &TextureSettings,
    b: &TexDabBasis,
    u: f32,
    v: f32,
    rgb: &ImageRgb,
) -> [f32; 3] {
    if s.kind != TextureKind::Image {
        return [0.0; 3];
    }
    let sx = s.size[0].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let sy = s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
    let rel = b.footprint.apply([u, v]);
    let tex = [
        (rel[0] * b.u[0] + rel[1] * b.u[1]) * sx + s.offset[0],
        (rel[0] * b.v[0] + rel[1] * b.v[1]) * sy + s.offset[1],
    ];
    if tex[0].abs() > 1.0 || tex[1].abs() > 1.0 {
        return [0.0; 3];
    }
    sample_image_clamped_rgb(rgb, tex[0], tex[1])
}

/// Bilinear sample of `img`'s straight RGB, **clamped** to the image edges (centre-coord), for footprint
/// coords already bounded to `[-1, 1]`. The colour twin of [`sample_image_clamped`]; a malformed buffer
/// reads `[0, 0, 0]`.
fn sample_image_clamped_rgb(img: &ImageRgb, u: f32, v: f32) -> [f32; 3] {
    let (w, h) = (img.width.max(1), img.height.max(1));
    if img.rgb.len() < (w as usize) * (h as usize) * 3 {
        return [0.0; 3];
    }
    let x = (u * 0.5 + 0.5) * w as f32 - 0.5;
    let y = (v * 0.5 + 0.5) * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let clamp = |i: i32, n: u32| i.clamp(0, n as i32 - 1) as usize;
    let (xi0, xi1) = (clamp(x0 as i32, w), clamp(x0 as i32 + 1, w));
    let (yi0, yi1) = (clamp(y0 as i32, h), clamp(y0 as i32 + 1, h));
    let at =
        |xi: usize, yi: usize, k: usize| f32::from(img.rgb[(yi * w as usize + xi) * 3 + k]) / 255.0;
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let ch = |k: usize| {
        let top = lerp(at(xi0, yi0, k), at(xi1, yi0, k), tx);
        let bot = lerp(at(xi0, yi1, k), at(xi1, yi1, k), tx);
        lerp(top, bot, ty)
    };
    [ch(0), ch(1), ch(2)]
}

fn sample_image_clamped(img: &ImageMask, u: f32, v: f32) -> f32 {
    let (w, h) = (img.width.max(1), img.height.max(1));
    if img.lum.len() < (w as usize) * (h as usize) {
        return 0.0;
    }
    let x = (u * 0.5 + 0.5) * w as f32 - 0.5;
    let y = (v * 0.5 + 0.5) * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let clamp = |i: i32, n: u32| i.clamp(0, n as i32 - 1) as usize;
    let (xi0, xi1) = (clamp(x0 as i32, w), clamp(x0 as i32 + 1, w));
    let (yi0, yi1) = (clamp(y0 as i32, h), clamp(y0 as i32 + 1, h));
    let at = |xi: usize, yi: usize| f32::from(img.lum[yi * w as usize + xi]) / 255.0;
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let top = lerp(at(xi0, yi0), at(xi1, yi0), tx);
    let bot = lerp(at(xi0, yi1), at(xi1, yi1), tx);
    lerp(top, bot, ty)
}
