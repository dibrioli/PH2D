//! The **multi-layer coloured stamp** — a Shape made of several z-ordered luminance layers, each
//! painted in its OWN colour, composited top-over-bottom into one cached RGBA stamp.
//!
//! This is the per-layer-colour mode of the Shape slot ([`docs/Painter/`]): when the Shape image has
//! several layers and the artist turns on "Per-Layer Color", every layer gets a colour (a custom pick,
//! or the brush's base colour) and the higher layer paints ABOVE the lower one — a multi-colour, 3-D-
//! looking tip. Like the grayscale [`crate::stamp`] it is **scale-invariant**: the N-layer composite is
//! baked ONCE into a unit-square [`ColorStampMask`] (premultiplied RGBA) and bilinearly scale-blitted
//! per dab, so the per-dab cost is one blit — the layer compositing never runs per pixel per dab.
//!
//! Premultiplied storage is deliberate: bilinear sampling of straight colour across an alpha edge bleeds
//! the rim colour inward (dark fringing); premultiplied sampling then un-premultiplying at blit time is
//! the correct, fringe-free path. Deterministic (HR-5): only `+ − × ÷` + the shared samplers.

use crate::blend::BrushBlend;
use crate::dab::{DirtyRect, encode, parallel_band_stamp};
use crate::spec::BrushSpec;
use crate::texture::ImageMask;

/// A baked multi-layer coloured stamp: per-texel **premultiplied** RGBA over the dab's `[-1, 1]²` square,
/// as `u8`. Scale-invariant — one stamp serves every dab size via [`blit_color_stamp`].
#[derive(Clone)]
pub struct ColorStampMask {
    data: Vec<u8>,
    size: u32,
    /// O domínio assado em raios ([`crate::tip::stamp_domain`]); quem amostra percorre `radius · reach`.
    reach: f32,
}

impl ColorStampMask {
    /// The stamp's resolution (texels per side).
    #[must_use]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// O domínio assado em raios (`≥ 1`) — o raio de quem amostra é `radius · reach`.
    #[must_use]
    pub fn reach(&self) -> f32 {
        self.reach
    }

    /// The premultiplied-RGBA texels (`size·size·4`), for the panel's coloured Shape preview.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// The Grain's per-channel multiplier for a sampled Grain value `g`. WITHOUT a ramp: the scalar value
/// (Grain-Depth-blended toward `1`) on all four channels. WITH the Grain Colour Ramp: `ramp[g]` (each
/// channel Depth-blended toward neutral `1`) so the Grain colour tints the premultiplied composite; the
/// alpha channel scales coverage either way. Deterministic (HR-5: float-only + the shared LUT sampler).
/// ⭐ **A lei do Bug #51:** uma camada que pinta a sua PRÓPRIA cor de textura pinta `textura × tinta`,
/// e o Randomize Color desloca esse produto pelo `shift` sorteado do carimbo. Com a tinta branca e sem
/// desvio é a textura ao byte. Os três núcleos (o carimbo assado, o acumulado por carimbo e o lote
/// dinâmico) passam por aqui — uma lei, um sítio.
#[inline]
pub(crate) fn texture_times_tint(texel: [f32; 3], tint: [f32; 3], shift: [f32; 3]) -> [f32; 3] {
    crate::jitter::apply_color_shift(
        [texel[0] * tint[0], texel[1] * tint[1], texel[2] * tint[2]],
        shift,
    )
}

fn grain_modulation(
    g: f32,
    depth: f32,
    granulation: f32,
    ramp: Option<&[[f32; 4]]>,
    lei: &crate::texture::GrainLaw,
) -> [f32; 4] {
    // Watercolor valley gate — scales the whole premultiplied modulation (coverage + tint) so pigment
    // is rejected in the paper's low areas. `1` when granulation is 0 → byte-identical.
    let vg = crate::texture::granulation_gate(g, granulation);
    match ramp {
        Some(lut) => {
            let c = crate::dab::ramp_sample(lut, g);
            [
                (1.0 + (c[0] - 1.0) * depth) * vg,
                (1.0 + (c[1] - 1.0) * depth) * vg,
                (1.0 + (c[2] - 1.0) * depth) * vg,
                (1.0 + (c[3] - 1.0) * depth) * vg,
            ]
        }
        None => {
            // A Shape de camadas de cor: o grão morde a ponta CHEIA (`w = 1`, L9) — o mesmo em cada camada.
            let m = lei.factor(1.0, g, depth, granulation);
            [m, m, m, m]
        }
    }
}

/// Render the unit multi-layer coloured stamp at `size`×`size`. `layers` are the Shape silhouette layers
/// in **bottom-to-top** z-order; `layer_colors` is each layer's already-resolved straight RGB (a custom
/// pick, or the brush base colour for an un-coloured layer) — same length as `layers`. Each layer's image
/// REPLACES the falloff (a crisp finite tip, like a single-image Shape), and the layers composite with
/// "over" so a higher layer paints above a lower one. The **Grain** (`spec.texture` + `grain_image`) bites
/// the whole composite's coverage, exactly like the grayscale stamp. With `grain_ramp` (the Grain Colour
/// Ramp LUT, 256 straight-sRGBA entries) the Grain value indexes a COLOUR that multiplies the composite
/// (Grain-Depth-blended toward neutral white) — so the Grain ramp tints the multi-colour tip instead of
/// only darkening it. Deterministic (HR-5).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn render_color_stamp_mask(
    spec: &BrushSpec,
    layers: &[ImageMask],
    layer_colors: &[[f32; 3]],
    layer_rgb: &[Option<crate::texture::ImageRgb>],
    layer_opacity: &[f32],
    grain_image: Option<&ImageMask>,
    grain_ramp: Option<&[[f32; 4]]>,
    size: u32,
) -> ColorStampMask {
    let size = size.max(1);
    let mut data = vec![0u8; (size as usize) * (size as usize) * 4];
    let grain_active = spec.texture.is_active();
    let depth = spec.grain_depth();
    let footprint = spec.footprint_deform();
    // The Grain + Shape View bases (no Rake/Random/Jitter ⇒ the rng / canvas / extra-rot args are unused).
    let grain_basis = crate::texture::dab_basis(&spec.texture, &mut 0u64, [1.0, 1.0], footprint);
    let shape_basis = crate::texture::shape_basis(
        &spec.shape,
        &mut 0u64,
        [1.0, 1.0],
        footprint,
        crate::texture::ShapeFrame::Static,
    );
    let reach = crate::tip::stamp_domain(&spec.shape, Some(&shape_basis));
    let inv = 2.0 / size as f32;
    for j in 0..size {
        let v = ((j as f32 + 0.5) * inv - 1.0) * reach;
        let row = (j as usize) * (size as usize) * 4;
        for i in 0..size {
            let u = ((i as f32 + 0.5) * inv - 1.0) * reach;
            // Composite the layers bottom→top — each higher layer "over" the accumulated lower ones.
            let mut acc = [0.0f32; 4]; // premultiplied rgba
            for (li, layer) in layers.iter().enumerate() {
                let a = crate::texture::sample_shape_silhouette_unit(
                    &spec.shape,
                    &shape_basis,
                    u,
                    v,
                    Some(layer),
                ) * layer_opacity.get(li).copied().unwrap_or(1.0);
                if a <= 0.0 {
                    continue;
                }
                // Texture Color: the layer paints its OWN per-pixel RGB (sampled at the same coord as the
                // silhouette) × its resolved tint; else the flat resolved colour. Layers still composite "over" (z-order) —
                // the per-layer blend modes apply at the on-canvas recomposite, not in this preview bake.
                let flat = layer_colors.get(li).copied().unwrap_or([0.0, 0.0, 0.0]);
                let c = match layer_rgb.get(li).and_then(|o| o.as_ref()) {
                    Some(img) => texture_times_tint(
                        crate::texture::sample_shape_rgb_unit(&spec.shape, &shape_basis, u, v, img),
                        flat,
                        [0.0; 3],
                    ),
                    None => flat,
                };
                let inv_a = 1.0 - a;
                acc[0] = c[0] * a + acc[0] * inv_a;
                acc[1] = c[1] * a + acc[1] * inv_a;
                acc[2] = c[2] * a + acc[2] * inv_a;
                acc[3] = a + acc[3] * inv_a;
            }
            // Grain bites the whole stamp's coverage (premultiplied → scale all four channels). With a
            // Grain Colour Ramp, the Grain value indexes a COLOUR (depth-blended toward neutral white)
            // that multiplies the premultiplied composite — tinting the tip by the Grain pattern.
            if acc[3] > 0.0 && grain_active {
                let g = crate::texture::sample_unit(&spec.texture, &grain_basis, u, v, grain_image);
                let m = grain_modulation(
                    g,
                    depth,
                    spec.effective_granulation(),
                    grain_ramp,
                    &spec.texture.lei,
                );
                acc[0] *= m[0];
                acc[1] *= m[1];
                acc[2] *= m[2];
                acc[3] *= m[3];
            }
            let o = row + (i as usize) * 4;
            data[o] = encode(acc[0]);
            data[o + 1] = encode(acc[1]);
            data[o + 2] = encode(acc[2]);
            data[o + 3] = encode(acc[3]);
        }
    }
    ColorStampMask { data, size, reach }
}

/// Render the unit **ramp-coloured** stamp at `size`×`size` — the cached fast path for a **Grain +
/// Colour Ramp** brush, and for the Shape's **Texture Color** with or without a Grain. The ramp's OWNER
/// indexes it ([`crate::RampIndex`]: the Grain value, or the Shape's layers — then the Grain, if any,
/// bites the coverage), exactly like the per-pixel kernel; the silhouette
/// (the Shape image, else the falloff) is the coverage, and in [`crate::RampAlphaMode::Strength`] the
/// ramp's alpha scales it (`TextureAlpha` is never baked — the route keeps it per-pixel). Scale-invariant (View Grain + a static silhouette), so the whole
/// grain×ramp colour bakes ONCE and scale-blits, instead of resampling the Grain + ramp per pixel per dab
/// (the slow path for the Line/Curve/Ellipse/Polygon fills). `shape_tone_lut` remaps the silhouette tone
/// (Shape ramp B&W on). Premultiplied RGBA. Deterministic (HR-5).
#[must_use]
pub fn render_ramp_color_stamp(
    spec: &BrushSpec,
    grain_image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_tone_lut: Option<&[f32]>,
    ramp: crate::RampPaint<'_>,
    size: u32,
) -> ColorStampMask {
    let size = size.max(1);
    let mut data = vec![0u8; (size as usize) * (size as usize) * 4];
    let footprint = spec.footprint_deform();
    let grain_basis = crate::texture::dab_basis(&spec.texture, &mut 0u64, [1.0, 1.0], footprint);
    let shape_active = spec.shape_silhouette_active(shape_image.is_some());
    let shape_basis = shape_active.then(|| {
        crate::texture::shape_basis(
            &spec.shape,
            &mut 0u64,
            [1.0, 1.0],
            footprint,
            crate::texture::ShapeFrame::Static,
        )
    });
    let reach = crate::tip::stamp_domain(&spec.shape, shape_basis.as_ref());
    // Texture Color: a cor lê o valor do Shape e um padrão procedural deixa de recortar (a regra do
    // núcleo por pixel, `dab.rs`); o Grain só entra quando há um.
    let le_forma = ramp.reads_shape_value();
    let recorta = !le_forma || spec.shape.kind == crate::texture::TextureKind::Image;
    let grain_active = spec.texture.is_active();
    let inv = 2.0 / size as f32;
    for j in 0..size {
        let v = ((j as f32 + 0.5) * inv - 1.0) * reach;
        let row = (j as usize) * (size as usize) * 4;
        for i in 0..size {
            let u = ((i as f32 + 0.5) * inv - 1.0) * reach;
            // Silhouette coverage — the Shape image REPLACES the falloff (else the bare falloff), exactly
            // like `render_stamp_mask`, so the cached colour matches the per-pixel path's coverage.
            let t = footprint.falloff_t(u, v);
            let (w, forma) = match &shape_basis {
                Some(sb) => {
                    let raw = crate::texture::sample_shape_silhouette_unit(
                        &spec.shape,
                        sb,
                        u,
                        v,
                        shape_image,
                    );
                    let sv = crate::texture::remap_shape_value(raw, shape_tone_lut);
                    let queda = spec.falloff_weight(t);
                    let w = if recorta {
                        spec.compose_shape_silhouette(sv, queda)
                    } else {
                        queda
                    };
                    (w, le_forma.then_some(sv))
                }
                None => (spec.falloff_weight(t), None),
            };
            // The FILM, on the silhouette (see `render_stamp_mask`) — the per-layer-colour tip is a
            // pigment path like any other, so the rule binds it too.
            let w = crate::height::film_coverage(spec.deposits_height(), w);
            if w <= 0.0 {
                continue; // leaves the texel transparent (premul [0,0,0,0])
            }
            // The ramp's OWNER indexes it, and the Grain bites unless it is the owner — the same door
            // as the per-pixel kernel ([`crate::ramp_alpha::ramp_texel`]).
            let s = grain_active.then(|| {
                crate::texture::sample_unit(&spec.texture, &grain_basis, u, v, grain_image)
            });
            let tinta = crate::ramp_alpha::ramp_texel(
                Some(&ramp),
                w,
                forma,
                s,
                spec.grain_depth(),
                spec.effective_granulation(),
                &spec.texture.lei,
            );
            let c = tinta.color.unwrap_or(spec.color);
            let a = w * tinta.g;
            if a <= 0.0 {
                continue;
            }
            let o = row + (i as usize) * 4;
            data[o] = encode(c[0] * a);
            data[o + 1] = encode(c[1] * a);
            data[o + 2] = encode(c[2] * a);
            data[o + 3] = encode(a);
        }
    }
    ColorStampMask { data, size, reach }
}

/// Read-only per-dab context for the coloured-stamp blit, shared across the row bands.
struct ColorBlitCtx<'a> {
    stamp: &'a ColorStampMask,
    blend: BrushBlend,
    /// O modo sobre o vazio (L7).
    vazio: bool,
    cx: f32,
    cy: f32,
    inv_radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    shift: [f32; 3],
    x0: i64,
    x1: i64,
    stride: usize,
}

/// Composite a cached [`ColorStampMask`] onto `buf` at `center` with `radius` (bilinear scale-blit). The
/// per-texel colour comes from the stamp itself; `spec` supplies the blend + Flow × Strength (its
/// `color` is unused — the layers already carry colour). Mirrors [`crate::blit_stamp`]. Returns the
/// touched [`DirtyRect`], or `None` if fully off-canvas / zero-coverage. `shift` = o Randomize Color
/// deste carimbo ([`crate::Dab::color_shift`]) sobre a cor de cada texel, fora do carimbo assado (que é
/// partilhado pelo traço); zero ⇒ a cor assada ao bit.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blit_color_stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamp: &ColorStampMask,
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    shift: [f32; 3],
) -> Option<DirtyRect> {
    debug_assert!(
        buf.len() >= (width as usize) * (height as usize) * 4,
        "buffer too small"
    );
    let coverage = coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    // O carimbo cobre `[-reach, reach]²` raios ([`crate::tip::stamp_domain`]).
    let radius = radius * stamp.reach;
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let stride = (width as usize) * 4;
    let ctx = ColorBlitCtx {
        stamp,
        blend: spec.blend,
        vazio: spec.blend_sobre_vazio,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        coverage,
        preserve_alpha,
        shift,
        x0,
        x1,
        stride,
    };
    let touched = parallel_band_stamp(buf, y0, y1, x0, x1, stride, |dst, band_y0| {
        blit_color_band(&ctx, dst, band_y0)
    });
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

/// Blit the cached coloured stamp into the full-width row band `dst` (first row `band_y0`).
fn blit_color_band(ctx: &ColorBlitCtx, dst: &mut [u8], band_y0: i64) -> bool {
    let mut touched = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let v = ((py as f32 + 0.5) - ctx.cy) * ctx.inv_radius;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            let u = ((px as f32 + 0.5) - ctx.cx) * ctx.inv_radius;
            let (rgb, cov) = sample_color_mask(ctx.stamp, u, v);
            if cov <= 0.0 {
                continue;
            }
            let i = row + (px as usize) * 4;
            let prev = [
                f32::from(dst[i]) / 255.0,
                f32::from(dst[i + 1]) / 255.0,
                f32::from(dst[i + 2]) / 255.0,
                f32::from(dst[i + 3]) / 255.0,
            ];
            let a = if ctx.preserve_alpha {
                cov * ctx.coverage * prev[3]
            } else {
                cov * ctx.coverage
            };
            if a <= 0.0 {
                continue;
            }
            let rgb = crate::jitter::apply_color_shift(rgb, ctx.shift);
            let out = crate::blend_vazio::blend_over_modo(ctx.blend, prev, rgb, a, ctx.vazio);
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}

/// Bilinear (straight RGB, coverage) from the premultiplied `stamp` at the dab-relative unit coord
/// `(u, v) ∈ [-1, 1]`, clamp-to-edge INSIDE a ponta. Samples premultiplied (fringe-free across the
/// alpha edge) then un-premultiplies the colour. Returns `([0; 3], 0.0)` where the composite is
/// transparent — **e fora do quadrado unitário, onde não há dab**.
///
/// ⚠️ **É esta a rota do report do Enio de 2026-08-09** (*"o carimbo invadindo o grid à direita e
/// abaixo … a borda da imagem cortada à esquerda e acima"*): com **Use Texture Colors** ligado o
/// roteador toma o carimbo COLORIDO, e ele estendia a borda opaca do stamp para fora da ponta, até
/// onde o retângulo assimétrico do blit fosse (`floor(c−r)` de um lado, `ceil(c+r) + 1` do outro).
/// MEDIDO pela porta do artista: numa célula de **40** px o carimbo pintava **41** colunas, **64 → 65**
/// e **256 → 257** — sempre UMA coluna a mais, sempre à direita e abaixo. As duas metades da frase dele
/// são as duas pontas do mesmo defeito: a coluna extra é a borda DUPLICADA, então a margem da imagem
/// engorda à direita e embaixo e lê-se como cortada à esquerda e acima.
///
/// A lei vem da porta única [`crate::tip::axis_taps`] — ver o porquê de ela existir lá.
fn sample_color_mask(stamp: &ColorStampMask, u: f32, v: f32) -> ([f32; 3], f32) {
    let (Some((x0, x1, tx)), Some((y0, y1, ty))) = (
        crate::tip::axis_taps(u, stamp.size),
        crate::tip::axis_taps(v, stamp.size),
    ) else {
        return ([0.0; 3], 0.0);
    };
    let sz = stamp.size as usize;
    let ch = |xi: usize, yi: usize, k: usize| f32::from(stamp.data[(yi * sz + xi) * 4 + k]) / 255.0;
    let lerp2 = |k: usize| {
        let top = ch(x0, y0, k) + (ch(x1, y0, k) - ch(x0, y0, k)) * tx;
        let bot = ch(x0, y1, k) + (ch(x1, y1, k) - ch(x0, y1, k)) * tx;
        top + (bot - top) * ty
    };
    let pa = lerp2(3);
    if pa <= 0.0 {
        return ([0.0; 3], 0.0);
    }
    let inv = 1.0 / pa;
    ([lerp2(0) * inv, lerp2(1) * inv, lerp2(2) * inv], pa)
}

mod accumulate;
mod accumulate_batch;
pub use accumulate::{
    accumulate_color_stamp_coverage, accumulate_color_stamps_fused, accumulate_shape_layer_rgba,
};
pub use accumulate_batch::{
    DynDab, FusedDab, accumulate_color_stamps_fused_batch, accumulate_color_stamps_rgba_batch,
    accumulate_shape_layers_rgba_batch,
};

#[cfg(test)]
mod tests;
