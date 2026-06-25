//! The **Shape** slot's silhouette sampling (Procreate "Shape"): unlike the Grain (which modulates
//! and tiles), the Shape reads as ONE finite tip — the footprint maps to `[-1, 1]²`, outside it the
//! coverage is `0`, and an image is clamped at its border (never tiled). Split from `texture.rs` to
//! keep that file under the workspace LOC cap.

use super::{
    ImageMask, MAX_TEX_PARAMS, TEX_SIZE_MAX, TEX_SIZE_MIN, TexDabBasis, TextureKind,
    TextureMapping, TextureSettings,
};

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

/// Render the composed Shape **silhouette** (the Procreate tip) into a `w×h` grayscale buffer (`1`
/// byte/texel = coverage `0..=255`), for the panel's live Shape preview. Uses the SAME composition as
/// the engine — `Image` is the sampled tip, a procedural kind is `falloff × pattern` — so the preview
/// is faithful. `falloff_lut[i]` is the falloff weight at radial `t = i/(N−1)` (`t ∈ [0,1]`), baked by
/// the panel from its live falloff. Static frame (Angle only; Rake / Random need a stroke direction).
/// Deterministic (HR-5).
#[allow(clippy::too_many_arguments)]
pub fn render_shape_preview(
    kind: TextureKind,
    angle_deg: u16,
    offset: [f32; 2],
    size: [f32; 2],
    params: [f32; MAX_TEX_PARAMS],
    falloff_lut: &[f32],
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
        random_angle: false,
        offset,
        size,
        params,
    };
    let basis = super::dab_basis(&shape, [0.0, 0.0], &mut 0u64, [1.0, 1.0], [1.0, 0.0]);
    let n = falloff_lut.len().max(1);
    let (inv_w, inv_h) = (2.0 / w.max(1) as f32, 2.0 / h.max(1) as f32);
    for j in 0..h {
        let v = (j as f32 + 0.5) * inv_h - 1.0;
        for i in 0..w {
            let u = (i as f32 + 0.5) * inv_w - 1.0;
            let sv = sample_shape_unit(&shape, &basis, u, v, image);
            let t = (u * u + v * v).sqrt().min(1.0);
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
    let rel = [(p[0] - center[0]) * sx / r, (p[1] - center[1]) * sy / r];
    shape_value(s, b, rel, image)
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
    shape_value(s, b, [u * sx, v * sy], image)
}

/// Shared core of [`sample_shape`] / [`sample_shape_unit`]: rotate the scaled footprint coord `rel`
/// into the shape's frame (basis already carries the rotation), translate by Offset, then read the
/// silhouette. Outside the `[-1, 1]²` tip → `0`. Image kind clamps to the border (one finite copy);
/// an active procedural kind reads its (bounded) value; no Image pixels → `0` (inert, no slab).
fn shape_value(
    s: &TextureSettings,
    b: &TexDabBasis,
    rel: [f32; 2],
    image: Option<&ImageMask>,
) -> f32 {
    let tex = [
        rel[0] * b.u[0] + rel[1] * b.u[1] + s.offset[0],
        rel[0] * b.v[0] + rel[1] * b.v[1] + s.offset[1],
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
