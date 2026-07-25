//! Stamp one brush dab into an RGBA8 layer buffer.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/mesh/paint_image_2d.cc` (the soft 2D brush that walks the dab's bounding
//! box, weights each texel by the falloff mask, and blends) + `paint_image_2d_curve_mask.cc` (the
//! per-texel falloff weight). Distance is measured from the dab centre to each **pixel centre**
//! (`px + 0.5`), matching the texel-centre convention.

use crate::ramp_alpha::RampAlphaMode;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

/// The **Shape** slot's per-dab inputs (Procreate "Shape"): the resolved tip frame plus the optional
/// silhouette image. `Some` ⇒ the Shape supplies the dab's silhouette — an Image **replaces** the
/// falloff, a procedural kind is **masked by** it (see [`BrushSpec::compose_shape_silhouette`]); `None`
/// ⇒ the falloff is the silhouette (byte-identical to the pre-Shape engine). The caller only passes
/// `Some` when the Shape is genuinely active (see [`BrushSpec::shape_silhouette_active`]), so an Image
/// shape with no pixels never reaches here. Bundled into one param to keep the stamp signatures
/// tractable (each already carries the Grain's `tex` + `image`).
#[derive(Clone, Copy)]
pub struct ShapeInput<'a> {
    /// The dab-relative texture frame for the Shape (rotation/offset), from [`crate::texture::dab_basis`].
    pub basis: &'a TexDabBasis,
    /// The silhouette pixels for [`crate::TextureKind::Image`]; `None` for a procedural Shape kind.
    pub image: Option<&'a ImageMask<'a>>,
    /// The Shape's **value ramp** LUT (256 grayscale entries, baked from `ph2d_color::ValueRamp`) that
    /// remaps the raw silhouette value (B&W tonal curve / invert); `None` ⇒ no remap. See
    /// [`crate::texture::remap_shape_value`].
    pub ramp_lut: Option<&'a [f32]>,
}

/// The dab's **silhouette** at one canvas pixel — the `w` of the kernel's funnel
/// `a = w · g · coverage`.
///
/// An **Image** Shape *replaces* the falloff (a crisp imported tip stays uneroded); a **procedural**
/// Shape is *masked by* it (`falloff × pattern`); **no** Shape ⇒ the bare falloff. `t` is the
/// normalised distance from the dab centre, already deformed by the dab's flatten/rotate footprint.
///
/// **This is the single source of the dab's shape.** The colour kernel ([`stamp_band`]) and the
/// height kernel ([`crate::height`]) both call it, so pigment and relief can never disagree about
/// what a dab looks like — which is precisely what makes Shape, the Shape-Tone ramp, the falloff and
/// the flatten/rotate footprint sculpt the Impasto relief for free, and keeps them sculpting it after
/// someone edits one of them six months from now.
#[inline]
#[must_use]
pub fn silhouette_at(
    spec: &BrushSpec,
    shape: Option<ShapeInput<'_>>,
    t: f32,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    match shape {
        Some(sh) => {
            let raw = crate::texture::sample_shape_silhouette(
                &spec.shape,
                sh.basis,
                px,
                py,
                center,
                radius,
                sh.image,
            );
            let sv = crate::texture::remap_shape_value(raw, sh.ramp_lut);
            spec.compose_shape_silhouette(sv, spec.falloff_weight(t))
        }
        None => spec.falloff_weight(t),
    }
}

/// The dab's **grain** coverage factor at one canvas pixel — the `g` of the funnel. Folds Grain Depth,
/// the watercolor Granulation gate and the Stencil rect, in that order. The colour kernel inlines this
/// same sequence (it needs the raw sample first, to index a Colour Ramp); the height kernel calls it
/// directly. Every step is itself a shared function, so the two cannot drift.
#[inline]
#[must_use]
pub fn grain_at(
    spec: &BrushSpec,
    basis: &TexDabBasis,
    image: Option<&ImageMask<'_>>,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    let s = crate::texture::sample(&spec.texture, basis, px, py, center, radius, image);
    crate::texture::grain_coverage(s, spec.grain_depth(), spec.effective_granulation())
        * crate::texture::stencil_gate(&spec.texture, basis, px, py)
}

/// Axis-aligned region of the buffer touched by a dab, in pixels (half-open: `[x, x+w)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Stamp one dab centred at `center` (image-space pixel coordinates) into `buf`.
///
/// `buf` is straight-alpha RGBA8, row-major, `width * height * 4` bytes, in the layer's native
/// space. `coverage` is the dab's overall opacity in `[0, 1]` — the stroke engine folds pressure
/// dynamics and the per-stroke strength cap into it; the brush's [`BrushSpec::flow`] and falloff
/// are applied here. Returns the touched [`DirtyRect`], or `None` if the dab is fully off-canvas
/// or has zero coverage.
///
/// When `preserve_alpha` is set ("alpha lock" / "preserve transparency"), each pixel's coverage is
/// scaled by the destination's existing alpha, so paint only lands where the layer already has
/// opacity (it recolours the shape without growing it).
///
/// Panics in debug if `buf` is too small. The texture-free fast path (delegates to [`stamp_dab_textured`]).
#[must_use]
pub fn stamp_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    stamp_dab_textured(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        None,
        None,
        None,
    )
}

/// As [`stamp_dab`], but the brush texture ([`BrushSpec::texture`]) modulates each texel's coverage when
/// `tex` is `Some`. `tex` is the per-dab frame from [`crate::texture::dab_basis`] (rotation + random
/// offset); `image` supplies the pixels for [`crate::TextureKind::Image`]. `None`/`None` (or an inactive
/// texture) reproduces [`stamp_dab`] exactly. See [`crate::texture`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        RampAlphaMode::None,
        None,
        [1.0, 0.0],
    )
}

/// As [`stamp_dab_textured`], plus the per-stroke coverage buffer `cover` (canvas-sized, 1 byte/px)
/// and the LAW it accumulates by ([`crate::stroke_cover`]): pigment's Accumulate-OFF cap
/// (`BuildUp` — overlapping / back-and-forth dabs build toward but never past the dab target) or the
/// coverage channel's `Envelope` (the dab profile is the target, kept by `max`, so re-crossing is
/// inert and the feather never hardens). `None` ⇒ exactly [`stamp_dab_textured`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    cover: Option<crate::stroke_cover::StrokeCover<'_>>,
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        RampAlphaMode::None,
        cover,
        dab_rotation,
    )
}

/// As [`stamp_dab_textured`], but a baked **Color Ramp** LUT (256 straight-RGBA entries) maps each
/// texel's texture value to a COLOUR: the brush paints `lut[t]`'s RGB, so the texture's scalar drives
/// the painted colour (not just attenuating the single colour). The ramp **alpha** does what `alpha_mode`
/// selects ([`RampAlphaMode`]): ignored, scaling coverage, or driving the pixel's own alpha (punch
/// transparent). With no texture there's nothing to index, so it falls back to the plain stamp.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_ramped(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: &[[f32; 4]],
    alpha_mode: RampAlphaMode,
    // Per-stroke coverage buffer + law so a Color-Ramp stroke honours Accumulate (TextureAlpha
    // uncapped), Enio 2026-06-25.
    cover: Option<crate::stroke_cover::StrokeCover<'_>>,
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        Some(ramp),
        alpha_mode,
        cover,
        dab_rotation,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
fn stamp_dab_inner(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: Option<&[[f32; 4]]>,
    alpha_mode: RampAlphaMode,
    // The per-stroke coverage buffer (canvas-sized, 1 byte/pixel) + its law. `Some` ⇒ the stroke's
    // coverage is tracked and the dab's alpha is derived from it (`BuildUp` = pigment's cap, threaded
    // when Accumulate is off and the cap is observable; `Envelope` = the coverage channel, the Mask
    // brush); `None` ⇒ the plain per-dab build-up.
    cover: Option<crate::stroke_cover::StrokeCover<'_>>,
    // The dab's composed footprint rotor ([`crate::BrushSpec::dab_rotor`], `[1, 0]` = none): the
    // per-dab **Jitter Rotate** spin composed with the **stroke-follow** rotation. It spins the whole
    // footprint — falloff + Shape + View-Grain — together, which is what keeps a following tip and the
    // pattern inside it from disagreeing on a curve.
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    // Contract guard (sweep 2026-07-12): was `debug_assert!`, i.e. absent from the build the artist runs.
    if buf.len() < (width as usize) * (height as usize) * 4 {
        return None;
    }
    // Per-dab opacity = the stroke's coverage × the brush's Flow (per-dab build-up) × Strength
    // (overall opacity). With `mask` set this is the per-stroke CAP (Accumulate off); without it the
    // dab just builds up (Accumulate on). Both default to 1.0.
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 {
        return None;
    }
    let radius = spec.clamped_radius();
    let (cx, cy) = (center[0], center[1]);

    // Screen-space AA of the film silhouette (BUGS #16, impasto half) — hoisted once per dab;
    // `None` = the single-sample `film_of` path, byte-identical (checkbox off / no body / Shape tip).
    let film_aa = crate::height_film::FilmAa::for_dab(
        spec,
        shape.filter(|_| spec.shape.is_active()).is_some(),
        radius,
    );
    // The AA's outermost fractional ring can live one texel past the geometric rim (a texel whose
    // CENTRE is outside can still be partially covered) — pad the bbox so it is not clipped.
    let aa_pad = crate::height_film::FilmAa::pad_px(&film_aa);

    // Bounding box of the dab, clamped to the canvas (half-open on the max side).
    let x0 = (cx - radius - aa_pad).floor().max(0.0) as i64;
    let y0 = (cy - radius - aa_pad).floor().max(0.0) as i64;
    let x1 = ((cx + radius + aa_pad).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius + aa_pad).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }

    let stride = (width as usize) * 4;
    let ctx = DabCtx {
        spec,
        // Texture frame, only when a texture is actually assigned (else the per-pixel sample is
        // skipped entirely so the texture-free path costs nothing).
        tex: tex.filter(|_| spec.texture.is_active()),
        image: image.copied(),
        // Kept even with no Grain — then the silhouette coverage indexes the ramp (Shape's colour ramp).
        ramp,
        // The Shape supplies the silhouette only when the slot is active (the caller already gates
        // this; the filter is belt-and-braces so an inactive Shape can never blank the falloff).
        shape: shape.filter(|_| spec.shape.is_active()),
        alpha_mode,
        footprint: spec.dab_footprint(dab_rotation),
        center,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        radius,
        coverage,
        preserve_alpha,
        x0,
        x1,
        stride,
        film_aa,
    };

    // The per-pixel work is independent, so a LARGE dab (e.g. a big Anchored stamp re-drawn every
    // pointer move) splits across the cores (see `parallel_band_stamp`). The result is bit-identical
    // to serial, so the texture stays fully visible during the drag. The Accumulate-cap path reads+
    // writes the shared per-stroke mask, so it runs SERIALLY (one band over the dab's rows) — small
    // soft-brush dabs anyway, where the cap is observable.
    let touched = match cover {
        Some(cover) => {
            let region = &mut buf[(y0 as usize) * stride..(y1 as usize) * stride];
            let mrow = width as usize;
            // The band carries the SAME law: the buffer and the law travel together, so a band can
            // never accumulate by one medium's rule into another medium's buffer.
            let band = crate::stroke_cover::StrokeCover {
                buf: &mut cover.buf[(y0 as usize) * mrow..(y1 as usize) * mrow],
                law: cover.law,
            };
            stamp_band(&ctx, region, Some(band), y0)
        }
        None => parallel_band_stamp(buf, y0, y1, x0, x1, stride, |dst, band_y0| {
            stamp_band(&ctx, dst, None, band_y0)
        }),
    };

    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

mod bands;

use bands::{DabCtx, stamp_band};
pub(crate) use bands::{PARALLEL_MIN_AREA, parallel_band_cached, parallel_band_stamp};
pub(crate) use bands::{encode, ramp_sample, stamp_rgba};

#[cfg(test)]
mod tests;
