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

/// Where a dab's height comes from — which part of the dab mask the colour path already built
/// (`silhouette × grain`) sculpts the relief.
///
/// **Two sources, not three.** The design doc listed a third, `Shape` ("the silhouette sample
/// alone"), but for every brush an artist actually builds it is a *silent duplicate* of `Uniform`:
/// with no Shape slot the silhouette IS the falloff, and with an Image Shape the image already
/// replaces the falloff, so "silhouette alone" and "grain neutral" are the same number. Shipping it
/// would have been a knob that does nothing — the exact species of bug the 2026-07-12 sweep spent
/// its length exterminating. Cut before it was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthSource {
    /// **Uniform** (default) — the Grain does *not* bite: the relief is the dab's **body** — the
    /// silhouette pushed through [`body_profile`], a level film with its wall at the paint's edge.
    /// Corel Painter documents its Uniform the same way: *"applies brushstrokes with even depth and
    /// little texture"*. The Grain still textures the *pigment*; it just doesn't carve the *body*.
    #[default]
    Uniform,
    /// **Grain** — the full dab mask (`w × g`): the Grain's striations become bristle marks in the
    /// relief, so the height varies *inside* the dab. This is the real impasto brush (Corel Painter's
    /// bristle depth, ArtRage's loaded brush): the grain's valleys are where the tuft left less paint.
    Grain,
}

impl DepthSource {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 2;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Uniform => 0,
            Self::Grain => 1,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::Uniform`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grain,
            _ => Self::Uniform,
        }
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::Grain => "Grain",
        }
    }
}

/// Which channels a dab writes — colour, height, or both.
///
/// Lets one brush be a pure *sculpting* tool (relief with no pigment: the palette knife that
/// spreads clear medium) or a pure *painting* tool over existing relief.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawTo {
    /// **Color + Depth** (default): the dab paints pigment and lays down thickness — the ordinary
    /// loaded brush.
    #[default]
    ColorAndDepth,
    /// **Color** only: pigment with no thickness. Equivalent to impasto off *for this brush*, but
    /// keeps the impasto settings around so the artist can flip back without re-dialling them.
    Color,
    /// **Depth** only: thickness with no pigment — the canvas RGBA is left byte-identical and only
    /// the height field changes. Sculpt clear medium, or carve (negative depth) into paint that is
    /// already down.
    Depth,
}

impl DrawTo {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::ColorAndDepth => 0,
            Self::Color => 1,
            Self::Depth => 2,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::ColorAndDepth`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Color,
            2 => Self::Depth,
            _ => Self::ColorAndDepth,
        }
    }

    /// Whether a dab with this setting deposits **pigment**. `false` ⇒ the colour path must leave the
    /// canvas RGBA untouched.
    #[must_use]
    pub fn writes_color(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Color)
    }

    /// Whether a dab with this setting deposits **height**. `false` ⇒ the height field is untouched.
    #[must_use]
    pub fn writes_depth(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Depth)
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ColorAndDepth => "Color + Depth",
            Self::Color => "Color",
            Self::Depth => "Depth",
        }
    }
}

/// Coverage below which the paint carries NO body: everything thinner is the STAIN — pigment rubbed
/// into the paper, not paint you could stand a wall on. It is deliberately high: the wall must rise
/// INSIDE the pigmented part of the stroke (Photoshop's bevel runs from the matte's edge *inward*,
/// over solid pixels, for the same reason) — a wall standing on the translucent rim gets its strong
/// lighting multiplied into pixels that are mostly PAPER, which is the white-canvas halo all over
/// again (`impasto_light_shades_the_paint_not_the_paper_showing_through_it` refuses it). // CLAMP-OK
pub const W_TAIL: f32 = 0.35;

/// Coverage at which the paint is a SOLID film: full thickness from here inward. Shared with the
/// light pass (its coverage weighting rides the same [`body_profile`]), so "solid paint" is one
/// concept with one pair of numbers on both sides of the pipeline. // CLAMP-OK
pub const W_SOLID: f32 = 0.75;

/// The **body curve**: how the dab's silhouette becomes the paint's *body*.
///
/// `h = depth × coverage × w` copies the colour's own soft profile into the relief — and for the
/// default brush (hardness 0) that is a dome the full width of the stroke, which reads as a blur, not
/// as paint (measured: shading peak 7.3 levels at 31% of the half-width; 1 level at the visible
/// edge). Nobody in the state of the art does that: Photoshop's bevel is a distance profile from the
/// coverage EDGE (Chisel), Blender's Layer brush caps at a fixed height ("creates the appearance of a
/// flat layer"), Hertzmann (NPAR 2002) gives height its own texture, and Painter documents Uniform as
/// "even depth". See `docs/Painter/17_impasto_deposito_pesquisa2.md` §3.
///
/// So the height gets its own profile: a **plateau** wherever the paint is solid (`w ≥ W_SOLID`),
/// **nothing** over the stain (`w ≤ W_TAIL` — the translucent rim keeps its pigment and stays flat),
/// and the **shoulder** — the wall the light lives on — in between, standing on pigment-backed
/// pixels a little inside the stain's edge, the way a real paint film ends inside its own smear.
/// Because a falloff is monotone in distance, remapping `w` IS a profile-in-distance (the bevel), it
/// commutes with the stroke envelope's `max`, and rule 1 still holds: the height consumes exactly
/// the silhouette the colour consumes. The shoulder's width follows the brush's own softness — a
/// soft brush lays a softer body edge — which is the coupling that should exist; the dome was the
/// one that shouldn't.
#[inline]
#[must_use]
pub fn body_profile(w: f32) -> f32 {
    let t = ((w - W_TAIL) / (W_SOLID - W_TAIL)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How deep the Grain's grooves cut into the body, with [`DepthSource::Grain`].
///
/// The grain must **carve grooves out of a full body**, not scale the body away. The naive
/// `h = depth · w · g` does the latter: a Noise grain's samples average well under half, so the paint
/// came out at ~30% of the Depth the artist asked for — a bristle brush laying a third of the paint
/// it should (measured on Enio's smoke: `max|h| = 0.21` where Uniform gave `0.70`). Here the grain
/// modulates *down* from the full thickness: where the grain is full the paint reaches Depth, where it
/// is empty the groove cuts this deep. That is what a tuft actually leaves behind. // CLAMP-OK
const GRAIN_GROOVE: f32 = 0.65;

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
#[inline]
#[must_use]
pub fn derive_height(spec: &crate::BrushSpec, paint: f32, grain: f32) -> f32 {
    let depth = spec.effective_impasto_depth();
    let m = paint.clamp(0.0, 1.0);
    if depth == 0.0 || m <= 0.0 {
        return 0.0;
    }
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
        a *= 1.0 - GRAIN_GROOVE * (1.0 - grain.clamp(0.0, 1.0));
    }
    depth * a
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
}

impl HeightFields<'_> {
    /// Whether every plane is at least `n` long — a real early-out, not a `debug_assert` that vanishes
    /// from the build the artist runs (the lesson of the 2026-07-12 SIGSEGV).
    fn fits(&self, n: usize) -> bool {
        self.height.len() >= n && self.paint.len() >= n && self.grain.len() >= n
    }
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
            // The **stroke envelope, taken on the PAINT** — the dab that laid the most paint at this
            // pixel owns it. One pass of a loaded brush leaves one thickness (a second pass over the
            // same line does not stack a staircase); separate strokes DO add, at stroke end.
            //
            // Enveloping the paint rather than the height is what makes every knob live: the winner is
            // then chosen by a quantity that no setting can change, so re-deriving the relief at a new
            // Body / Source / Depth cannot silently re-shuffle which dab shaped which pixel.
            let m = (w * coverage).clamp(0.0, 1.0);
            let i = (py as usize) * (width as usize) + px as usize;
            if m <= fields.paint[i] {
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
                let take = b.ground[i] * (m - fields.paint[i]);
                if take != 0.0 {
                    b.plane[i] -= take;
                    b.displaced += take;
                }
            }
            fields.paint[i] = m;
            fields.grain[i] = gq;
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

/// **Plow** one dab: drag the relief (and the paint that carries it) along the stroke — the palette
/// knife.
///
/// It is the SAME lift-and-drag as the colour Smear ([`crate::smear_dab`]): snapshot the source region
/// at `from`, blend it into the footprint at `to`, weighted by the falloff × `strength`. That is not a
/// coincidence and not a shortcut — it is the point. The knife moves paint; the pigment and the body of
/// that paint must move *together*, by the same displacement, under the same mask. Give the relief its
/// own displacement law and the light drifts off the colour: shadows floating beside the ridge they
/// belong to.
///
/// Corel calls the parameter `Plow` ("a brushstroke with a high Plow value... displaces the depth of the
/// existing brushstroke"); ArtRage's palette knife is the same gesture ("swipe paint with a Flat blade
/// to smooth or create impasto effects"). Both are *displacement*, not deposition: this function never
/// adds paint, it only moves what is already there — which is why it lives on the Smear route and needs
/// no Depth of its own.
///
/// `cover` rides along for the same reason the relief does (the light weighs its shading by it). Both
/// are lifted before any write, so a dab never reads back its own output.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn plow_dab_height(
    dst: &mut [f32],
    cover: &mut [u8],
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    from: [f32; 2],
    to: [f32; 2],
    radius: f32,
    strength: f32,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if dst.len() < n || cover.len() < n || width == 0 || height == 0 {
        return None;
    }
    let radius = radius.max(0.5);
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 {
        return None;
    }
    // The integer drag vector — the same convention the colour smear uses, so the two stay in step to
    // the pixel (a half-pixel disagreement here would shear the relief off its own paint).
    let step_x = (to[0].round() as i64) - (from[0].round() as i64);
    let step_y = (to[1].round() as i64) - (from[1].round() as i64);
    if step_x == 0 && step_y == 0 {
        return None;
    }
    let (fw, fh) = (i64::from(width), i64::from(height));
    let min_x = (to[0] - radius).floor().max(0.0) as i64;
    let min_y = (to[1] - radius).floor().max(0.0) as i64;
    let max_x = ((to[0] + radius).ceil() as i64).min(fw);
    let max_y = ((to[1] + radius).ceil() as i64).min(fh);
    if max_x <= min_x || max_y <= min_y {
        return None;
    }
    let (bw, bh) = ((max_x - min_x) as usize, (max_y - min_y) as usize);

    // Lift first: the source footprint overlaps the destination (they are one step apart), so a
    // read-as-you-write loop would drag a pixel through its own smeared output and the knife would leave
    // a comet tail.
    let mut lifted_h = vec![0.0f32; bw * bh];
    let mut lifted_c = vec![0u8; bw * bh];
    for j in 0..bh {
        let sy = min_y + j as i64 - step_y;
        if sy < 0 || sy >= fh {
            continue; // off-canvas source: nothing to drag in (leaves the destination untouched below)
        }
        for i in 0..bw {
            let sx = min_x + i as i64 - step_x;
            if sx < 0 || sx >= fw {
                continue;
            }
            let si = (sy * fw + sx) as usize;
            lifted_h[j * bw + i] = dst[si];
            lifted_c[j * bw + i] = cover[si];
        }
    }

    let inv_r = 1.0 / radius;
    let mut touched = false;
    for j in 0..bh {
        let y = min_y + j as i64;
        let dy = y as f32 + 0.5 - to[1];
        for i in 0..bw {
            let x = min_x + i as i64;
            let dx = x as f32 + 0.5 - to[0];
            let t = (dx * dx + dy * dy).sqrt() * inv_r;
            if t >= 1.0 {
                continue;
            }
            let w = spec.falloff_weight(t) * strength;
            if w <= 0.0 {
                continue;
            }
            let di = (y * fw + x) as usize;
            let (sh, sc) = (lifted_h[j * bw + i], f32::from(lifted_c[j * bw + i]));
            dst[di] += (sh - dst[di]) * w;
            let c = f32::from(cover[di]);
            cover[di] = (c + (sc - c) * w).round().clamp(0.0, 255.0) as u8;
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: min_x as u32,
        y: min_y as u32,
        w: bw as u32,
        h: bh as u32,
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

// **Volume conservation** — the brush shoving the paint it finds out of its way — lives in the sibling
// [`crate::height_push`]: the deposit and the displacement are different physics, and the file-LOC cap
// agrees. Re-exported from the crate root, so callers see one `height` surface.
