//! Stamp one brush dab into an RGBA8 layer buffer.
//!
//! Behavioural reference (clean-room, no code copied): Blender's **2D image paint** — the soft
//! brush that walks the dab's bounding box, weights each texel by the falloff mask, and blends,
//! plus the curve-mask stage that produces that per-texel weight. Distance is measured from the
//! dab centre to each **pixel centre**
//! (`px + 0.5`), matching the texel-centre convention.

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
    silhouette_at_com(spec, spec.queda_do_dab(), shape, t, px, py, center, radius)
}

/// [`silhouette_at`] com a [`crate::spec::QuedaDoDab`] do dab já lida (o kernel lê-a uma vez por banda).
#[allow(clippy::too_many_arguments)]
#[inline]
#[must_use]
pub(crate) fn silhouette_at_com(
    spec: &BrushSpec,
    queda: crate::spec::QuedaDoDab,
    shape: Option<ShapeInput<'_>>,
    t: f32,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    match shape {
        Some(sh) => spec.compose_shape_silhouette(
            shape_value_at(spec, sh, px, py, center, radius),
            queda.peso(spec, t),
        ),
        None => queda.peso(spec, t),
    }
}

/// The Shape's value at one canvas pixel, through its ramp — what [`silhouette_at`] composes with the
/// falloff. The Wet Paint deposit reads it APART for a procedural Shape (the plan-27 law B: the
/// pattern multiplies after the paper gate, `ph2d_wet_paint::brush::stamp_row_shaped`).
#[inline]
#[must_use]
pub fn shape_value_at(
    spec: &BrushSpec,
    sh: ShapeInput<'_>,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    let raw = crate::texture::sample_shape_silhouette(
        &spec.shape,
        sh.basis,
        px,
        py,
        center,
        radius,
        sh.image,
    );
    crate::texture::remap_shape_value(raw, sh.ramp_lut)
}

/// ⭐ **The dab-CAPSULE's silhouette** at one canvas pixel ([`crate::capsula`], Continuous Spacing): the
/// [`silhouette_at`] of each sub-dab — the dab INTERPOLATED back to the previous one ([`crate::capsula::
/// Capsula::sub_dab`]: centre, radius, footprint rotation, arc length; its coverage scales the pass) —
/// through the film cut, composed by `lei` with the per-pass opacity `c`. A procedural Shape under a
/// zero falloff is skipped (it multiplies); an Image tip never is.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn silhouette_capsula(
    spec: &BrushSpec,
    shape: Option<ShapeInput<'_>>,
    footprint: crate::footprint::FootprintDeform,
    cap: crate::capsula::Capsula,
    lei: crate::capsula::Lei,
    c: f32,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    silhouette_capsula_com(
        spec,
        spec.queda_do_dab(),
        shape,
        footprint,
        cap,
        lei,
        c,
        px,
        py,
        center,
        radius,
        |_, _, _| {},
    )
}

/// [`silhouette_capsula`] that also hands `cada(sub_k, raio_k, x_k)` each sub-dab, front (`k = 0`) to
/// back, with the `x_k` the [`crate::capsula::Junta`] composes — the capsule's COLOUR folds over the same
/// sub-dabs in the same order ([`cor_capsula`], lista 26 A9).
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn silhouette_capsula_com(
    spec: &BrushSpec,
    queda: crate::spec::QuedaDoDab,
    shape: Option<ShapeInput<'_>>,
    footprint: crate::footprint::FootprintDeform,
    cap: crate::capsula::Capsula,
    lei: crate::capsula::Lei,
    c: f32,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
    mut cada: impl FnMut(&crate::capsula::Sub, f32, f32),
) -> f32 {
    let (dx, dy) = (px as f32 + 0.5 - center[0], py as f32 + 0.5 - center[1]);
    let corpo = spec.deposits_height();
    let imagem = spec.shape.kind == crate::texture::TextureKind::Image;
    let flow = spec.shape.flow;
    let mut j = crate::capsula::Junta::new(lei, c);
    for k in 0..cap.n() {
        let s = cap.sub_dab(k);
        let (o, r) = (s.off, radius * s.escala);
        let inv = 1.0 / r;
        let fp = s.rotor.map_or(footprint, |q| footprint.rotated_by(q));
        let queda = queda.peso(spec, fp.falloff_t((dx - o[0]) * inv, (dy - o[1]) * inv));
        let (x, forma) = match shape {
            Some(sh) if queda > 0.0 || imagem => {
                let base;
                let basis = if s.rotor.is_some() || (flow && s.arco != 0.0) {
                    base = sh.basis.sub(s.rotor, s.arco);
                    &base
                } else {
                    sh.basis
                };
                let raw = crate::texture::sample_shape_silhouette(
                    &spec.shape,
                    basis,
                    px,
                    py,
                    [center[0] + o[0], center[1] + o[1]],
                    r,
                    sh.image,
                );
                let sv = crate::texture::remap_shape_value(raw, sh.ramp_lut);
                (spec.compose_shape_silhouette(sv, queda), sv)
            }
            Some(_) => (0.0, 0.0),
            None => (queda, 1.0),
        };
        let x = crate::height::film_coverage(corpo, x) * s.cobertura;
        cada(&s, r, x);
        j.mais(x, queda, forma);
    }
    j.fim()
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
    // O relevo e o Wet mordem com a ponta CHEIA (`w = 1`, L9): o Height e o Linear Height valem por inteiro.
    spec.texture
        .lei
        .factor(1.0, s, spec.grain_depth(), spec.effective_granulation())
        * crate::texture::stencil_gate(&spec.texture, basis, px, py)
}

/// [`grain_at`] com a ponta `w` (`0..1`) da célula: os modos do grão que são função da ponta (Subtract, Darken,
/// Color Burn — L9) mordem a orla do dab do Wet como mordem a do carimbo; com `w = 1` é [`grain_at`] ao bit (o
/// Multiply, o Height e o Linear Height não lêem a ponta).
#[allow(clippy::too_many_arguments)]
pub fn grain_at_na_ponta(
    spec: &BrushSpec,
    basis: &TexDabBasis,
    image: Option<&ImageMask<'_>>,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
    w: f32,
) -> f32 {
    let s = crate::texture::sample(&spec.texture, basis, px, py, center, radius, image);
    spec.texture.lei.factor(
        w.clamp(0.0, 1.0),
        s,
        spec.grain_depth(),
        spec.effective_granulation(),
    ) * crate::texture::stencil_gate(&spec.texture, basis, px, py)
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
    stamp_dab_inner::<u8>(
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
        None,
        None,
        None,
        [1.0, 0.0],
        PARALLEL_MIN_AREA,
        None,
    )
}

/// As [`stamp_dab_textured`], but a baked **Color Ramp** LUT (256 straight-RGBA entries) maps each
/// texel's value to a COLOUR: the brush paints `lut[v]`'s RGB, where `v` is what the ramp's OWNER
/// samples ([`crate::RampIndex`]: the Grain value, or the silhouette — then the Grain still bites the
/// coverage). The ramp **alpha** does what `alpha_mode` selects ([`crate::RampAlphaMode`]): ignored,
/// scaling coverage, or driving the pixel's own alpha (punch transparent).
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
    ramp: crate::RampPaint<'_>,
    // Per-stroke coverage buffer + law so a Color-Ramp stroke honours Accumulate (TextureAlpha
    // uncapped), Enio 2026-06-25.
    mask: Option<&mut [u8]>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
) -> Option<DirtyRect> {
    stamp_dab_ramped_forca(
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
        ramp,
        mask,
        dab_rotation,
        capsula,
        po,
        None,
    )
}

/// [`stamp_dab_ramped`] que também compõe o plano da força das macros (B25.5), como o `stamp_dab`; o Texture Alpha
/// não o escreve (baixa o alfa em vez de pintar por cima).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_ramped_forca(
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
    ramp: crate::RampPaint<'_>,
    mask: Option<&mut [u8]>,
    dab_rotation: [f32; 2],
    capsula: Option<crate::capsula::Capsula>,
    po: Option<crate::texture::PoDoPapel<'_>>,
    forca: Option<&mut [u16]>,
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
        mask,
        forca,
        po,
        dab_rotation,
        PARALLEL_MIN_AREA,
        capsula,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
fn stamp_dab_inner_com<M: crate::Tecto>(
    buf: &mut [u8],
    liso: Option<&mut [u8]>,
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: Option<crate::RampPaint<'_>>,
    // **Accumulate OFF** per-stroke coverage mask (canvas-sized, 1 byte/pixel). `Some` ⇒ cap each
    // pixel's stroke coverage at the dab target (the tool passes it when Accumulate is off and the cap
    // is observable); `None` ⇒ the plain per-dab build-up. `M` = o texel do tecto ([`crate::Tecto`], B29.1).
    mask: Option<&mut [M]>,
    // A força com que o lote pinta cada texel (canvas-sized, `u16`; [`crate::stroke_cover::compoe_forca`]).
    forca: Option<&mut [u16]>,
    // O pó pousado antes do traço ([`crate::texture::PoDoPapel`]): o Height lê o papel com o dente cheio.
    po: Option<crate::texture::PoDoPapel<'_>>,
    // The dab's composed footprint rotor ([`crate::BrushSpec::dab_rotor`], `[1, 0]` = none): the
    // per-dab **Jitter Rotate** spin composed with the **stroke-follow** rotation. It spins the whole
    // footprint — falloff + Shape + View-Grain — together, which is what keeps a following tip and the
    // pattern inside it from disagreeing on a curve.
    dab_rotation: [f32; 2],
    // O piso a partir do qual as linhas do dab se dividem entre os nucleos. Producao passa
    // [`PARALLEL_MIN_AREA`]; o gate de identidade passa `usize::MAX` para forcar a banda UNICA que
    // este kernel rodava antes de 2026-08-04 -- e' isso que torna o oraculo dele **o codigo que
    // shipava**, e nao uma segunda implementacao escrita para o teste.
    min_area: usize,
    // The dab-capsule (Continuous Spacing, [`crate::capsula`]); `None` ⇒ the plain dab, unchanged.
    capsula: Option<crate::capsula::Capsula>,
    // `Some` ⇒ a pegada vai para buffers PRÓPRIOS do tamanho da caixa do dab ([`PegadaLocal`]); `buf`/`liso` não se tocam.
    local: Option<&mut Option<PegadaLocal>>,
) -> (Option<DirtyRect>, Option<DirtyRect>) {
    // Contract guard (sweep 2026-07-12): was `debug_assert!`, i.e. absent from the build the artist runs.
    if local.is_none() && buf.len() < (width as usize) * (height as usize) * 4 {
        return (None, None);
    }
    // Per-dab opacity = the dab's coverage (which already carries the Strength, ONCE —
    // `stroke/dab_build.rs`) × the brush's Flow (per-dab build-up). With `mask` set this is the
    // per-stroke CAP (Accumulate off) — o tecto é a cobertura SEM o Flow e o Flow é a taxa por dab
    // ([`crate::stroke_cover::cover_add`]); without it the dab just builds up (Accumulate on). BUGS #41:
    // multiplying `spec.strength` here as well made the opacity Strength².
    let tecto = coverage.clamp(0.0, 1.0);
    let flow = spec.flow.clamp(0.0, 1.0);
    let coverage = tecto * flow;
    if coverage <= 0.0 || width == 0 || height == 0 {
        return (None, None);
    }
    let radius = spec.clamped_radius();
    let (cx, cy) = (center[0], center[1]);
    // Texture Color ([`crate::ShapeRampLayers`]): a cor lê o VALOR da textura do Shape; um padrão
    // procedural vira cor e deixa de recortar (a silhueta é a queda, como o Grain em cor), uma imagem
    // continua a ser a ponta.
    let shape = shape.filter(|_| spec.shape.is_active());
    let shape_cor = shape.filter(|_| {
        ramp.as_ref()
            .is_some_and(crate::RampPaint::reads_shape_value)
    });
    let forma_extensao = shape;
    let shape = shape
        .filter(|_| shape_cor.is_none() || spec.shape.kind == crate::texture::TextureKind::Image);

    // Screen-space AA of the film silhouette (BUGS #16, impasto half) — hoisted once per dab;
    // `None` = the single-sample `film_of` path, byte-identical (checkbox off / no body / Shape tip).
    // The capsule's sub-dabs already average along the step, and each takes the film cut itself.
    let film_aa = crate::height_film::FilmAa::for_dab(spec, shape.is_some(), radius)
        .filter(|_| capsula.is_none());
    // The AA's outermost fractional ring can live one texel past the geometric rim (a texel whose
    // CENTRE is outside can still be partially covered) — pad the bbox so it is not clipped.
    let aa_pad = crate::height_film::FilmAa::pad_px(&film_aa);
    // The film's PRE-CONVOLVED table (plano 26 §9.6), when this brush and radius admit it: the AA's
    // nine samples then cost nine table reads instead of nine silhouette chains (up to 18 dependent
    // `sqrt`). Memoised per STROKE — asking per dab would cost 9× what it saves. `None` ⇒ the exact
    // path, unchanged. The dab is a plain disc here, so the plan carries no capsule axis.
    let footprint = spec.dab_footprint(dab_rotation);
    let film_lut = film_aa
        .as_ref()
        .and_then(|_| crate::height_film::film_lut_for(spec, radius));
    let lut = film_lut
        .as_ref()
        .map(|l| crate::height_film::FilmLutPlan::new(l, footprint, radius, None));

    // Bounding box of the dab, clamped to the canvas (half-open on the max side).
    //
    // ⭐⭐⭐ **A caixa segue a PEGADA, não o círculo** ([`crate::FootprintDeform::extent`]): um dab
    // achatado é uma lasca dentro do quadrado de lado `2·raio`, e com `flatten = 0,75` esse quadrado
    // tem `4×` os texels que a elipse toca — todos os outros devolvendo cobertura ZERO. ⚠️ **O
    // resultado pintado é o MESMO ao bit** (o que ficava de fora já tinha `falloff_t > 1`), e é isto
    // que faz o custo de um dab deixar de crescer com o quadrado do raio: sobre arte comprimida o
    // raio e o achatamento crescem JUNTOS, logo a área é `R²/|det W|` — os texels que a arte tem.
    // E a ponta de imagem do Shape sai dela quando roda ou tem Size < 1 ([`crate::tip::dab_extent`]).
    let [ext_x, ext_y] = crate::tip::dab_extent(
        footprint.extent(),
        &spec.shape,
        forma_extensao.as_ref().map(|s| s.basis),
    );
    // The capsule's box reaches back to its farthest sub-dab, at its biggest radius.
    let r_max = radius * capsula.map_or(1.0, crate::capsula::Capsula::escala_max);
    let (reach_x, reach_y) = (r_max * ext_x + aa_pad, r_max * ext_y + aa_pad);
    let (lo, hi) = capsula.map_or(([0.0; 2], [0.0; 2]), crate::capsula::Capsula::caixa);
    let x0 = (cx + lo[0] - reach_x).floor().max(0.0) as i64;
    let y0 = (cy + lo[1] - reach_y).floor().max(0.0) as i64;
    let x1 = ((cx + hi[0] + reach_x).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + hi[1] + reach_y).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return (None, None);
    }

    let stride = (width as usize) * 4;
    let tex = tex.filter(|_| spec.texture.is_active());
    let grao =
        tex.and_then(|b| crate::texture::grao_cache::para_o_dab(&spec.texture, b, width, height));
    let dente = spec
        .dente
        .as_ref()
        .and_then(|d| crate::texture::grao_cache::para_o_dente(d, width, height));
    let ctx = DabCtx {
        spec,
        // Texture frame, only when a texture is actually assigned (else the per-pixel sample is
        // skipped entirely so the texture-free path costs nothing).
        tex,
        image: image.copied(),
        // Who indexes it travels with it ([`crate::RampIndex`]): the Grain value, or the silhouette.
        ramp,
        // The Shape supplies the silhouette only when the slot is active (filtered above, so an
        // inactive Shape can never blank the falloff) — and not when Texture Color made it colour.
        shape,
        shape_cor,
        footprint,
        center,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        radius,
        coverage,
        tecto,
        flow,
        preserve_alpha,
        x0,
        x1,
        stride: local.as_ref().map_or(stride, |_| ((x1 - x0) as usize) * 4),
        ox: if local.is_some() { x0 } else { 0 },
        vazio_local: local.is_some(),
        film_aa,
        lut,
        capsula,
        lei: crate::capsula::lei(),
        po,
        grao,
        dente,
    };

    // The per-pixel work is independent, so a LARGE dab (e.g. a big Anchored stamp re-drawn every
    // pointer move) splits across the cores (see `parallel_band_stamp_masked`). The result is
    // bit-identical to serial, so the texture stays fully visible during the drag.
    //
    // ⚠️ **O cap de Accumulate viaja JUNTO desde 2026-08-04.** Ele rodava numa banda só, com o motivo
    // *"lê+escreve a máscara por-traço compartilhada — dabs pequenos e macios de qualquer jeito, onde
    // o cap é observável"*. A premissa era verdadeira e DEIXOU de ser: o cap disparava só em
    // `strength < 1`, e o **AA do filme** do impasto (`film_aa_wanted`) passou a ligá-lo para TODO
    // pincel de impasto — inclusive os maiores do app, que são justamente os que cruzam o piso.
    // Medido: com o dab ABAIXO do piso o cap custa 0,99× (grátis); ACIMA dele, 4,15×.
    let caixa = DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    };
    if let Some(saida) = local {
        let n = (caixa.w as usize) * (caixa.h as usize) * 4;
        let (mut m, mut l) = (vec![0u8; n], liso.map(|_| vec![0u8; n]));
        let (touched, liso_tocou) = liso::parallel_band_stamp_com_liso(
            &mut m,
            l.as_deref_mut(),
            None::<&mut [M]>,
            None,
            0,
            y1 - y0,
            x0,
            x1,
            ctx.stride,
            min_area,
            |dst, lband, mband, fband, band_y0| {
                stamp_band(&ctx, dst, lband, mband, fband, band_y0 + y0)
            },
        );
        *saida = Some(PegadaLocal {
            caixa,
            mascara: m,
            liso: l.unwrap_or_default(),
        });
        return (touched.then_some(caixa), liso_tocou.then_some(caixa));
    }
    let (touched, liso_tocou) = liso::parallel_band_stamp_com_liso(
        buf,
        liso,
        mask,
        forca,
        y0,
        y1,
        x0,
        x1,
        stride,
        min_area,
        |dst, lband, mband, fband, band_y0| stamp_band(&ctx, dst, lband, mband, fband, band_y0),
    );
    (touched.then_some(caixa), liso_tocou.then_some(caixa))
}

/// ⭐ **A pegada de um dab num buffer PRÓPRIO** (Bug #84): a máscara RGBA e a pegada lisa só da caixa do dab, para a
/// rota da mistura calcular as pegadas de um lote em paralelo ENTRE os dabs (a pegada de um dab não depende da tela,
/// só a película). Os mesmos bytes que o kernel pinta na tela vazia, à mesma caixa.
pub struct PegadaLocal {
    pub caixa: DirtyRect,
    pub mascara: Vec<u8>,
    pub liso: Vec<u8>,
}

mod bands;
mod cor_capsula;
mod liso;
mod mascarado;
pub(crate) use bands::parallel_band_stamp_masked;
use bands::{DabCtx, stamp_band};
pub use bands::{PARALLEL_MIN_AREA, band_count};
pub(crate) use bands::{encode, ramp_sample, stamp_rgba};
pub(crate) use bands::{parallel_band_cached, parallel_band_stamp};
use liso::stamp_dab_inner;
pub use liso::{
    pegada_local, pegada_textured_masked_po_com_liso, stamp_dab_textured_masked_po_com_liso,
};
pub use mascarado::{
    stamp_dab_masked_tecto, stamp_dab_textured_masked, stamp_dab_textured_masked_po,
    stamp_dab_textured_masked_po_largo, stamp_dab_textured_masked_with,
};

#[cfg(test)]
mod tests;

// ⚠️ Os comentarios ficam ACIMA do `mod`, nunca ao lado: o rustfmt REORDENA estas linhas e leva o
// comentario trailing junto com a linha errada — a cicatriz que a `line/motion-value` ja' pagou.
// A porta unica da divisao: o piso e' derivado, a contagem sai do TRABALHO.
#[cfg(test)]
mod band_count_tests;
// A mascara do cap atravessa a divisao por linhas sem mudar um byte.
#[cfg(test)]
mod band_mask_tests;
// B29.1: onde a tinta estanca com Flow baixo (sonda).
#[cfg(test)]
mod precisao_tests;
// O A/B do pigmento: a banda com a LUT e sem ela escreve os mesmos bytes.
#[cfg(test)]
mod lut_ab_tests;
