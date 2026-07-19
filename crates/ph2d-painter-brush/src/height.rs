//! **Impasto**: the brush's height channel — the paint's own thickness.
//!
//! The engine paints two things per dab, from **one** kernel: colour and, when
//! [`crate::BrushSpec::impasto`] is on, a height `h`. The height is a *second output* of the dab
//! pipeline that already exists — it consumes the same dab list (already mirrored by Symmetry,
//! already replicated by Tiling) and the same [`crate::StampMask`] (silhouette × grain) that the
//! colour consumes. That is what makes Shape / Shape-Tone / Grain / Falloff / Stroke / Jitter /
//! Mirror / Tiling work under impasto **for free** — see `docs/Painter/16_impasto_plano_implementacao.md` §0.
//!
//! `h` is a signed `f32`: positive lifts paint off the canvas, negative carves into it.
//!
//! Not a lighting module — the light pass is the compositor's (`impasto_pass`). This is only the
//! *material*: what the brush deposits.
//!
//! Where a body of paint ENDS — [`W_TAIL`] / [`W_SOLID`] / [`body_profile`] / [`film_coverage`] — lives in
//! the sibling [`crate::height_film`], re-exported here so callers still see one `height` surface.

pub use crate::height_film::{W_SOLID, W_TAIL, body_profile, film_coverage, solid_paint};
pub use crate::height_modes::{DepthSource, DrawTo};

/// How deep the Grain's grooves cut into the body, with [`DepthSource::Grain`].
///
/// The grain must **carve grooves out of a full body**, not scale the body away. The naive
/// `h = depth · w · g` does the latter: a Noise grain's samples average well under half, so the paint
/// came out at ~30% of the Depth the artist asked for — a bristle brush laying a third of the paint
/// it should (measured on Enio's smoke: `max|h| = 0.21` where Uniform gave `0.70`). Here the grain
/// modulates *down* from the full thickness: where the grain is full the paint reaches Depth, where it
/// is empty the groove cuts this deep. That is what a tuft actually leaves behind. // CLAMP-OK
const GRAIN_GROOVE: f32 = 0.65;

/// The grain's weight on a body, as a multiplier — `1` where the grain is full, [`GRAIN_GROOVE`] deep
/// where it is empty. Shared with [`crate::sculpt`] so a grainy spatula bites by the SAME law a grainy
/// brush deposits by; a bare `× g` in either place would not texture the touch, it would quietly remove
/// two thirds of it (a grain's samples average well under half — see [`derive_height`]).
#[inline]
#[must_use]
pub(crate) fn grain_groove(grain: f32) -> f32 {
    1.0 - GRAIN_GROOVE * (1.0 - grain.clamp(0.0, 1.0))
}

/// One dab's inputs to the height kernel — the *same* resolved frames the colour kernel is handed for
/// that dab (its footprint, its Shape basis, its Grain basis). The caller resolves them once and gives
/// both kernels the same ones; that is the whole trick.
#[derive(Clone, Copy)]
pub struct HeightDab<'a> {
    /// Dab centre in canvas pixels. Already wrapped by Tiling and mirrored by Symmetry — the height
    /// kernel consumes the **dab list**, it never re-derives geometry.
    pub center: [f32; 2],
    /// Dab radius in canvas pixels (this dab's, after Jitter Scale).
    pub radius: f32,
    /// Per-dab dynamics (pressure); folded with Flow × Strength exactly as the colour kernel folds it,
    /// so a light touch lays *thinner* paint, not just fainter paint.
    pub coverage: f32,
    /// This dab's flatten/rotate footprint (incl. Jitter Rotate).
    pub footprint: crate::footprint::FootprintDeform,
    /// Centre of the PREVIOUS dab along the path (with this dab's Tiling wrap already applied), or
    /// `None` for the first dab of a stroke. The body is swept along the segment between the two — see
    /// [`accumulate_dab_height`].
    pub prev_center: Option<[f32; 2]>,
    /// The Shape slot's resolved frame + pixels, or `None` when the falloff is the silhouette.
    pub shape: Option<crate::dab::ShapeInput<'a>>,
    /// The Grain's resolved frame — read only when [`DepthSource::Grain`] is selected.
    pub grain: Option<&'a crate::texture::TexDabBasis>,
    /// The Grain's pixels (an `Image` Grain).
    pub grain_image: Option<&'a crate::texture::ImageMask<'a>>,
}

/// How far back the body sweeps, in pixels (0 when there is no previous dab).
#[inline]
pub(crate) fn sweep_len(dab: &HeightDab<'_>) -> f32 {
    sweep_axis(dab).map_or(0.0, |(_, l)| l)
}

/// The dab's **sweep axis**: the unit vector back toward the previous dab and the distance to it.
/// `None` for the first dab of a stroke (nothing to sweep back to) or a degenerate zero-length step.
#[inline]
pub(crate) fn sweep_axis(dab: &HeightDab<'_>) -> Option<([f32; 2], f32)> {
    let prev = dab.prev_center?;
    let v = [prev[0] - dab.center[0], prev[1] - dab.center[1]];
    let len2 = v[0] * v[0] + v[1] * v[1];
    if len2 <= 1e-6 {
        return None;
    }
    let len = len2.sqrt();
    Some(([v[0] / len, v[1] / len], len))
}

/// Offset from the pixel to the nearest point on the segment `[previous centre → this centre]` — the
/// residual the falloff is then evaluated on. With no previous dab this is the plain offset from the
/// centre (byte-identical to a stamped disc).
#[inline]
pub(crate) fn sweep_residual(dx: f32, dy: f32, sweep: Option<([f32; 2], f32)>) -> (f32, f32) {
    match sweep {
        None => (dx, dy),
        Some((u, back)) => {
            // The segment ENDS on the previous dab's centre — a point the stroke certainly painted — so
            // the swept body can never reach past the paint, at any spacing and under any Jitter. That is
            // why the sweep is defined by the real chord and not by a heading and a nominal pitch: the
            // heading is SMOOTHED, so on a curve it cuts across the arc and the far end of the capsule
            // escapes off the convex side (26 pixels of shadow on bare canvas, the first time I tried it).
            let s = (dx * u[0] + dy * u[1]).clamp(0.0, back);
            (dx - s * u[0], dy - s * u[1])
        }
    }
}

/// The relief a pixel carries, from the two things the stroke actually deposited there: how much
/// **paint** (`0..1`, the silhouette × the dab's dynamics) and what the **grain** sampled (`1` = no
/// grain). A pure function of those and the brush's settings.
///
/// ## Why the stroke stores INGREDIENTS and not the height
///
/// Enio, 2026-07-12: *"coloque todos os parâmetros vivos em tempo real para ajustes depois do traço."*
/// Depth was live because it is a pure rescale of a stored height; Body and Depth Source could not be,
/// because they were **baked into** that height, pixel by pixel, and nothing was left to re-derive them
/// from. So the stroke keeps its paint and its grain — the record of what the brush *laid down* — and
/// the relief is computed from them on demand. Every knob in the Body card then edits the last stroke
/// live, and none of them is a special case: the deposit is `derive_height`, and so is the edit.
///
/// The profile runs on the PAINT (`w × dynamics`), not on the bare silhouette: it is the same quantity
/// the light weighs its shading by, so the wall stands exactly where the light says the paint becomes
/// solid — geometry and shading cannot disagree. It also makes a light touch lay thinner, softer-edged
/// paint, which is what a light touch does.
///
/// ## The relief is the thickness of the FILM, not of the raw paint
///
/// [`film_coverage`] cuts the pigment at the body's edge, and the relief has to be cut at the SAME place
/// or the fix just swaps one seam for its mirror: the raw paint still reaches out past the film (the
/// coverage ramps on down to zero), so at Body 0 (`h = depth × paint`) a rim of relief goes on standing
/// over pixels that no longer carry any pigment — and `impasto_light_does_not_shade_paint_that_is_not_there`
/// caught exactly that, a 120-px ring of lit bare paper. So the profile runs on the paint's own film.
///
/// It is not an extra curve bolted on; it is the same statement from the other side. **The paint IS the
/// film, so the relief is the film's thickness.** At Body 0 the relief follows the film exactly — which is
/// the literal reading of Enio's *"a tinta corresponde ao relevo"* — and Body then flattens that film into
/// a slab. What the raw paint keeps doing, unchanged, is being the INGREDIENT: it is what the whole Body
/// card re-derives from, so Depth / Body / Depth Source / Smoothing / Push all stay live.
/// The brush radius at which impasto Depth means exactly what it says: `Depth` loads of paint.
///
/// The deposit's peak height scales with `radius / this`, so the mound's aspect ratio is constant and
/// the relief reads at every brush size (see [`derive_height`]). Ten pixels is the app's default brush,
/// so a freshly-opened brush is unchanged and every other size is relative to it. // CLAMP-OK
pub const IMPASTO_REFERENCE_RADIUS_PX: f32 = 10.0;

#[inline]
#[must_use]
pub fn derive_height(spec: &crate::BrushSpec, paint: f32, grain: f32) -> f32 {
    let depth = spec.effective_impasto_depth();
    let m = paint.clamp(0.0, 1.0);
    if depth == 0.0 || m <= 0.0 {
        return 0.0;
    }
    // **The relief's height scales with the brush's SIZE** (Enio, smoke of 2026-07-14: *"a altura do relevo
    // não está vinculada ao tamanho do pincel … pincéis grandes ficam parecendo que não têm relevo e que são
    // apenas tinta"*).
    //
    // `depth` alone is an absolute thickness in loads, so a 6-px dab and a 60-px dab both peaked at the same
    // height — and a mound that height over a 60-px footprint has a slope of nothing, `n_z ≈ 1`, and the
    // light draws it flat. A bigger brush carries a thicker glob of paint; scaling the peak by the radius
    // keeps the mound's **aspect ratio** (height ÷ width) constant, so the falloff reads at every scale and a
    // big brush is a big dome instead of a puddle.
    //
    // Against a REFERENCE radius, so a dab at the reference is unchanged and the number stays dimensionless.
    // Uses the brush's base radius (`clamped_radius`) — exact for a constant-size stroke, which is the case
    // Enio is comparing; a pressure-tapered stroke scales uniformly by the base rather than per-dab, which is
    // a refinement noted for later, not a visible error.
    let size_scale = spec.clamped_radius() / IMPASTO_REFERENCE_RADIUS_PX;
    // The **Body** dial: at 1 the full body curve (plateau + wall), at 0 the paint's own profile
    // (falloff / Shape / Shape-Tone ramp sculpt the relief — a soft brush lays a perfectly rounded
    // ridge), in between the mesa family. Monotone in `m` at every setting, which is what lets the
    // stroke envelope be taken on the paint alone.
    let body = spec.effective_impasto_body();
    let mut a = m + (body_profile(m) - m) * body;
    if matches!(spec.impasto_source, DepthSource::Grain) {
        // Grooves cut out of a FULL body — never `a *= g`. A grain's samples average well under half,
        // so multiplying by it does not texture the paint, it removes two thirds of it. See
        // [`GRAIN_GROOVE`]. With the body curve the grooves notch a PLATEAU — bristle marks in a level
        // film, the Painter/ArtRage signature — instead of mushing a dome.
        a *= grain_groove(grain);
    }
    depth * a * size_scale
}

/// The grain sample of a pixel that has none: a full, ungrooved body.
pub const NO_GRAIN: u8 = 255;

/// The buffers one stroke writes: the derived relief plus the **ingredients** it was derived from
/// ([`derive_height`]). All canvas-sized (`width × height`); `grain` is 8-bit because it only ever
/// scales a groove (a 1/255 step moves the height by 0.25% of Depth — invisible), while `paint` stays
/// `f32` because it drives the profile, and quantising *that* would stair-step the wall.
pub struct HeightFields<'a> {
    /// The relief itself — always exactly `derive_height(spec, paint[i], grain[i])`.
    pub height: &'a mut [f32],
    /// How much paint the winning dab laid at this pixel (`0..1`).
    pub paint: &'a mut [f32],
    /// That same dab's grain sample ([`NO_GRAIN`] where the brush carries no grain).
    pub grain: &'a mut [u8],
    /// The **solid paint** the stroke laid ([`solid_paint`]) — the film's own alpha, and the coverage the
    /// light weighs its shading by. Its own `max` envelope, NOT the winner-by-paint's value: it is a
    /// different function of the dab (the body curve runs on the silhouette, the dynamics scale it), so
    /// the dab that laid the most paint here is not always the one that laid the most body.
    pub film: &'a mut [u8],
    /// The winning dab's **radius** (raw `radius_px`, the value its `spec` carried) — the third
    /// ingredient. The relief's height scales with the dab's size ([`IMPASTO_REFERENCE_RADIUS_PX`]),
    /// and for the drag-sized methods (Anchored/Drag Dot) and pressure-tapered strokes the dab's
    /// radius is NOT the panel brush's: re-deriving the commit with the base radius flattened an
    /// Anchored ball by `brush/drag` — Enio's live smoke, 2026-07-15, *"ao soltar o mouse, achata
    /// relevo"*. Stored per texel with the same winner as `paint`/`grain`, so every Body-card knob
    /// re-derives the stroke at the size that actually made it — and the Size slider AFTER a stroke
    /// no longer silently re-scales relief that is already on the canvas.
    /// **Accumulate** (`BrushSpec::accumulate`) — a soma do perfil ao longo do caminho, SEM o Depth.
    ///
    /// `None` é o comportamento histórico e o caminho intocado: o relevo é o envelope, e uma 2ª passada
    /// no mesmo traço não empilha. `Some` liga a integral de linha — ver [`pass_normalizer`].
    pub accum: Option<&'a mut [f32]>,
    pub radius: &'a mut [f32],
}

impl HeightFields<'_> {
    /// Whether every plane is at least `n` long — a real early-out, not a `debug_assert` that vanishes
    /// from the build the artist runs (the lesson of the 2026-07-12 SIGSEGV).
    fn fits(&self, n: usize) -> bool {
        self.height.len() >= n
            && self.paint.len() >= n
            && self.grain.len() >= n
            && self.film.len() >= n
            && self.radius.len() >= n
    }
}

/// Quanto UMA passada reta deposita, na unidade em que a integral soma — o divisor que faz
/// **Accumulate ON e OFF coincidirem no caso simples**.
///
/// A integral acumula `perfil · Δs`, então uma passada reta sobre um texel soma
/// `∫ perfil(dist) ds ≈ 2ρ · (perfil médio ao longo da corda)`. Dividir por isso faz essa passada valer
/// o pico que o envelope daria — e é o que torna o toggle honesto: marcá-lo não muda um traço simples,
/// só passa a somar quando o artista **volta por cima**.
///
/// Numérico e amostrado sobre a corda porque o perfil é o produto de duas curvas (o falloff e o **Body**),
/// e a forma fechada mudaria toda vez que o Body mudasse.
#[must_use]
pub fn pass_normalizer(spec: &crate::BrushSpec) -> f32 {
    const N: usize = 33;
    let depth = spec.effective_impasto_depth();
    if depth == 0.0 {
        return 1.0;
    }
    let mut sum = 0.0f32;
    for k in 0..N {
        let t = (k as f32 / (N - 1) as f32) * 2.0 - 1.0;
        let w = spec.falloff.weight(t.abs()).clamp(0.0, 1.0);
        // A parte SEM Depth de `derive_height` — ele é um múltiplo de `depth`, então dividir recupera
        // o resto exatamente. (Grão neutro: o normalizador descreve o pincel, não a textura do lugar.)
        sum += derive_height(spec, w, 1.0) / depth;
    }
    let mean = sum / N as f32;
    (2.0 * spec.clamped_radius().max(0.5) * mean).max(1e-6)
}

/// Deposit one dab's height into the per-stroke envelope `dst` (canvas-sized, `width × height` f32).
///
/// Reads the **same** silhouette and grain the colour kernel reads, through the same
/// [`crate::dab::silhouette_at`] / [`crate::dab::grain_at`]. Returns the touched rect, or `None` if
/// the dab is off-canvas / deposits nothing.
///
/// The deposited height is `depth × coverage × body_profile(w)` (with the Grain notching grooves into
/// that body for [`DepthSource::Grain`]) — thickness follows how much paint the dab lays down, through
/// the paint's OWN profile (plateau + shoulder, [`body_profile`]), which is why every knob that shapes
/// the dab still shapes the relief without the relief inheriting the colour's soft dome.
///
/// ## The body is SWEPT along the path, not stamped as a disc
///
/// The relief must be a property of the brush and the PATH — never of how finely the engine happened to
/// sample that path. A per-dab disc breaks this: the envelope is a `max` of discrete domes, and between
/// two centres the distance to either one grows, so the maximum DIPS. The stroke comes out corrugated,
/// with a ripple whose depth is set by the spacing. Enio proved it with one image: the same brush at
/// spacing 0.1 / 0.05 / 0.01 gave heavy ribs, mild ribs, and a smooth tube.
///
/// So the dab's body is swept back along the segment to the PREVIOUS dab's centre — a capsule, not a
/// disc — and the union of capsules is the stroke's true distance field. Flat at any spacing.
///
/// The segment ends on a centre the stroke certainly painted, so the swept body can never reach past
/// the paint — at any spacing, under any Jitter. (Sweeping a *nominal* pitch along the *smoothed*
/// heading is the obvious cheaper thing, and it is wrong: on a curve that chord cuts across the arc and
/// the far end escapes off the convex side, laying shadow on bare canvas.)
///
/// No new geometry is generated: the previous centre comes from the dab list itself, so Rule 1 holds —
/// the height still consumes exactly the dab list the colour consumes.
///
/// A **Shape image** silhouette is deliberately unaffected: `silhouette_at` samples an Image tip at the
/// pixel, so it stays a STAMP. A stamp brush is supposed to leave stamps.
#[must_use]
pub fn accumulate_dab_height(
    fields: &mut HeightFields<'_>,
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
    mut bite: Option<&mut crate::height_push::PushBite<'_>>,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if !fields.fits(n) || width == 0 || height == 0 {
        return None;
    }
    // A dab with no depth AND no push has nothing to say about the height field. With push, it has:
    // the FOOTPRINT is itself an ingredient (it is what the displacement bites with), so a brush that
    // carries no paint and only shoves must still record where it passed. Bailing on depth alone made
    // the dry brush — the most physical use of Push there is — a silent no-op.
    if spec.effective_impasto_depth() == 0.0 && spec.effective_impasto_push() == 0.0 {
        return None;
    }
    // The same fold the colour kernel applies: pressure × Flow × Strength. A light, thin stroke is
    // both fainter AND thinner — one number drives both.
    let coverage =
        dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    // The bbox has to cover the whole SWEPT body, not just the disc at the centre.
    let reach = radius + sweep_len(dab);
    let x0 = (cx - reach).floor().max(0.0) as i64;
    let y0 = (cy - reach).floor().max(0.0) as i64;
    let x1 = ((cx + reach).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + reach).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    // The grain is sampled whenever the brush HAS one, even under `DepthSource::Uniform`: it is stored
    // as an ingredient, so flipping the source after the stroke re-carves the same grooves this dab
    // would have left. A brush with no grain pays nothing.
    let grain_active = dab.grain.is_some();
    // **Accumulate**: o arco que ESTE dab representa, e o divisor que o converte na unidade do envelope.
    // É `Δs` que tira o espaçamento da conta — dobrar a densidade de dabs dobra a contagem e divide o
    // passo, e a soma converge para a mesma integral (doc 20 §5).
    let step_len = dab
        .prev_center
        .map(|p| ((dab.center[0] - p[0]).powi(2) + (dab.center[1] - p[1]).powi(2)).sqrt())
        .unwrap_or(0.0);
    let accum_w = if fields.accum.is_some() {
        step_len / pass_normalizer(spec)
    } else {
        0.0
    };
    let depth_now = spec.effective_impasto_depth();
    let sweep = sweep_axis(dab);
    let mut touched = false;
    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let (rx, ry) = sweep_residual(dx, dy, sweep);
            let t = dab.footprint.falloff_t(rx * inv_radius, ry * inv_radius);
            let w = crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius);
            if w <= 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            // The **film's** envelope, taken FIRST and on its own: the light's coverage is a different
            // function of the dab than the relief's ingredient is, so it cannot ride the same winner.
            let fq = (crate::height_film::solid_paint(w, coverage) * 255.0 + 0.5) as u8;
            if fq > fields.film[i] {
                fields.film[i] = fq;
                touched = true;
            }
            // The **stroke envelope, taken on the PAINT** — the dab that laid the most paint at this
            // pixel owns it. One pass of a loaded brush leaves one thickness (a second pass over the
            // same line does not stack a staircase); separate strokes DO add, at stroke end.
            //
            // Enveloping the paint rather than the height is what makes every knob live: the winner is
            // then chosen by a quantity that no setting can change, so re-deriving the relief at a new
            // Body / Source / Depth cannot silently re-shuffle which dab shaped which pixel.
            let m = (w * coverage).clamp(0.0, 1.0);
            // ── OFF: o envelope, byte a byte como sempre foi (ordem do Enio 2026-07-18). ─────────
            //
            // O ramo `false` é o CÓDIGO de hoje, não uma re-derivação que por acaso concorda: quem lê
            // vê a identidade em vez de ter de confiar num gate.
            if fields.accum.is_none() && m <= fields.paint[i] {
                continue;
            }
            let g = if let Some(b) = dab.grain {
                crate::dab::grain_at(spec, b, dab.grain_image, px, py, dab.center, radius)
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };
            let gq = if grain_active {
                (g * 255.0 + 0.5) as u8
            } else {
                NO_GRAIN
            };
            // **Volume conservation, riding along** (`crate::height_push`): the ground this dab's advance
            // covers is ground it SHOVES, and it is taken here — inside the walk that already knows `m`,
            // `paint[i]` and the silhouette. Doing it in a kernel of its own meant evaluating
            // `silhouette_at` twice per texel, and that alone put the impasto cost at 5.0 ms/move, over
            // budget, on every stroke. Three operations, folded into a loop that was already running.
            if let Some(b) = bite.as_deref_mut() {
                // The bite takes from the ground AND from the stroke's own accumulated plane — the
                // bow wave the previous dab banked ahead is picked up here and shoved on (see
                // `forward_weight`). `(g + p)` is what actually stands at the texel right now, and
                // `.max(0)` guards float fuzz.
                //
                // **The share is the increment over the REMAINING HEADROOM, not the raw increment** —
                // and that is what makes the trench a fact of the PATH instead of a fact of the dab
                // spacing. With the raw `Δm`, `q = g + p` evolves as `q ← q·(1 − Δm)`, so the total
                // bite is `g·(1 − Π(1 − Δm_k))`: a PRODUCT over the increments, which depends on how
                // many steps the envelope was reached in and on each texel's phase against the dab
                // grid. A soft falloff hides it (its `Δm` are small and even); `Sphere`'s silhouette
                // has a VERTICAL tangent at the rim, so `Δm` jumps hard, the phase term explodes, and
                // the trench floor comes out RIPPLED at exactly the dab period — the coil Enio's smoke
                // caught (2026-07-15). Normalising by `(1 − paint)` telescopes the product exactly:
                // `Π (1 − Δm/(1 − m_{k−1})) = Π (1 − m_k)/(1 − m_{k−1}) = (1 − m_final)`, so the bite
                // lands on `g·m_final` — a pure function of the envelope, at ANY spacing, in ANY
                // order. It is also the honest law: the brush shoves the ground in proportion to how
                // much it ended up covering the texel, and at full coverage it takes all of it and
                // never more (the self-limiting guarantee the raw form gave, now exact).
                let head = 1.0 - fields.paint[i];
                if head > 1e-6 {
                    let share = ((m - fields.paint[i]) / head).clamp(0.0, 1.0);
                    let take = (b.ground[i] + b.plane[i]).max(0.0) * share;
                    if take != 0.0 {
                        b.plane[i] -= take;
                        b.displaced += take;
                    }
                }
            }
            if let Some(acc) = fields.accum.as_deref_mut() {
                // A integral soma a parte SEM Depth do perfil deste dab, pesada pelo arco percorrido.
                // Guardar sem o Depth é o que mantém o slider VIVO depois do traço.
                if depth_now != 0.0 && accum_w > 0.0 {
                    let unit = derive_height(spec, m, f32::from(gq) / 255.0) / depth_now;
                    acc[i] += unit * accum_w;
                }
                // Os INGREDIENTES seguem sendo os do dab mais carregado — a forma vem do dab que mais
                // depositou, a quantidade vem da integral.
                if m > fields.paint[i] {
                    fields.paint[i] = m;
                    fields.grain[i] = gq;
                    fields.radius[i] = spec.radius_px;
                }
                fields.height[i] = depth_now * acc[i];
                touched = true;
                continue;
            }
            fields.paint[i] = m;
            fields.grain[i] = gq;
            fields.radius[i] = spec.radius_px;
            // Derived from the STORED (quantised) grain, so the buffer and the re-derivation always
            // agree to the last bit — a live edit can never make the relief jump.
            fields.height[i] = derive_height(spec, m, f32::from(gq) / 255.0);
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// **Erase** one dab's footprint from the height field: the relief is scrubbed away in proportion to
/// the dab's coverage, exactly where the eraser removes pigment.
///
/// Not optional, and not the same as depositing a negative depth (that would *carve*). Without it the
/// eraser leaves **ghost relief**: the paint is gone but the light still reports a ridge. Reads the
/// same silhouette as the colour path, so an erase with a Shape tip erases that tip's profile.
#[must_use]
pub fn erase_dab_height(
    dst: &mut [f32],
    cover: &mut [u8],
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if dst.len() < n || cover.len() < n || width == 0 || height == 0 {
        return None;
    }
    let coverage =
        dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let reach = radius + sweep_len(dab);
    let x0 = (cx - reach).floor().max(0.0) as i64;
    let y0 = (cy - reach).floor().max(0.0) as i64;
    let x1 = ((cx + reach).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + reach).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let sweep = sweep_axis(dab);
    let mut touched = false;
    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let (rx, ry) = sweep_residual(dx, dy, sweep);
            let t = dab.footprint.falloff_t(rx * inv_radius, ry * inv_radius);
            let w = crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius);
            if w <= 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            if dst[i] == 0.0 && cover[i] == 0 {
                continue;
            }
            let scrub = 1.0 - (w * coverage).clamp(0.0, 1.0);
            dst[i] *= scrub;
            cover[i] = (f32::from(cover[i]) * scrub) as u8; // the paint goes, so does its presence
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

#[cfg(test)]
#[path = "height_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "height_push_tests.rs"]
mod push_tests;

// **Volume conservation** — the brush shoving the paint it finds out of its way — lives in the sibling
// [`crate::height_push`]: the deposit and the displacement are different physics, and the file-LOC cap
// agrees. Re-exported from the crate root, so callers see one `height` surface.
