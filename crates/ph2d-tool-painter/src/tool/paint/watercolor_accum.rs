//! Watercolor **stroke buffers** — the per-stroke coverage mask + deposited-colour accumulation the
//! optical composite ([`super::watercolor_render`]) reconstructs the wash from, plus their incremental
//! dirty-rect tracking (wet_edges `stampCoverage`/`stampColor`/`expand`). Split from
//! `watercolor_render.rs` for the workspace LOC cap.

use super::*;

/// The soft-disc weight at normalised radius `dn ∈ [0, 1]`: `1.0` at the centre, `0.92` at `0.62`,
/// `0` at the rim (the two-segment linear gradient of `stampCoverage` / `stampColor`). Shared with
/// [`super::watercolor_mixer`], whose disc pickup uses the same weighting as the deposit.
pub(super) fn feather(dn: f32) -> f32 {
    if dn <= 0.62 {
        1.0 - dn / 0.62 * 0.08 // 1.00 → 0.92 across the plateau
    } else {
        0.92 * (1.0 - (dn - 0.62) / 0.38) // 0.92 → 0 across the rim
    }
}

/// Per-pixel PAINT-GATE weight for the watercolor splats: the product of the two canvas gates the
/// normal stamp route enforces by snapshot/restore ([`PainterTool::restore_deselected_region`] /
/// [`PainterTool::restore_protected_region`]) — the **selection** coverage (1 = inside, 0 = outside;
/// a feathered edge scales) × the **protection** scratch keep (`mask_value`: 1 = unprotected, 0 =
/// frozen). The watercolor path can't use snapshot/restore (its canvas write happens later, in
/// `apply_watercolor`, over a frozen base), so instead the wash never FORMS on gated-out texels:
/// the coverage / colour / soak splats scale by this weight, the optics read "no paint here"
/// (`cw = 0`) and the composite leaves the frozen base verbatim — the same end state the restore
/// path produces. A THIRD gate (`alock`, doc 13 #8) is the **alpha-lock**: the frozen-base RGBA, whose
/// α scales the deposit so the wash forms only into existing paint (0 where transparent) — same family,
/// same machinery. Callers pass `None` for an inactive gate (all `None` = ungated fast path,
/// byte-identical to before the gates existed).
#[inline]
pub(super) fn splat_keep(
    sel: Option<&[u8]>,
    prot: Option<&[u8]>,
    alock: Option<&[u8]>,
    gidx: usize,
) -> f32 {
    let mut keep = 1.0f32;
    if let Some(m) = sel {
        keep *= m.get(gidx).map_or(1.0, |&v| f32::from(v) / 255.0);
    }
    if let Some(s) = prot
        && (gidx * 4 + 2) < s.len()
    {
        keep *= crate::compositor::mask_value(s, gidx);
    }
    // Alpha-lock (§2.10): scale by the frozen base's existing α (RGBA, α at `*4 + 3`) — the wash forms
    // proportional to the paint already there, so transparent texels (α = 0) never take a deposit.
    if let Some(a) = alock
        && (gidx * 4 + 3) < a.len()
    {
        keep *= f32::from(a[gidx * 4 + 3]) / 255.0;
    }
    keep
}

/// A cloned paint-gate buffer handle (`None` = that gate is off) — see [`PainterTool::wet_splat_gates`].
pub(super) type SplatGate = Option<Arc<Vec<u8>>>;

/// Per-batch context for the **manual** (Shape-driven) watercolor stamp — Shape "Automatic" OFF
/// (doc 13 #1, Enio 2026-07-07): the user's Falloff (incl. the [`ph2d_painter_brush::Falloff::Watercolor`]
/// preset = the built-in feather, bit-identical) + hardness + the Shape slot silhouette drive the
/// coverage disc instead of the fixed `feather()`. `None` = Automatic (the historical stamp, untouched).
///
/// Everything downstream of the coverage (rim, bleed, thinning, granulation, rewet) is
/// silhouette-agnostic, so the wet dynamics follow whatever this stamps.
pub(super) struct WetShapeStamp {
    /// The captured spec (falloff/hardness/shape read here, never from live state mid-batch).
    spec: ph2d_painter_brush::BrushSpec,
    /// Whether the Shape slot actively shapes the silhouette (image present / procedural kind).
    shape_active: bool,
    /// Whether the Shape slot is an IMAGE tip (the [`Self::img_norm`] scaling applies).
    is_image: bool,
    /// Tip-luminance normaliser (`1/max`, from [`PainterTool`]'s per-stroke scan): the coverage is
    /// wetness geometry and must saturate in the tip core — see `PaintState::wet_shape_norm`.
    img_norm: f32,
}

/// A textured tip's WETNESS ramp: a normalised tip sample at/below `TIP_WET_LO` is a true hole
/// (dry — outside the tip / zero bristle contact); by `TIP_WET_HI` the texel is FULLY wet. In real
/// watercolor the water fills the tip's outer silhouette while the texture modulates the PIGMENT —
/// so mid-tone texture must not hole the wash (that starved the optics: pale centre + dead rim);
/// it becomes tip DENSITY instead (`stroke_density` → the composite's fill term).
const TIP_WET_LO: f32 = 0.03;
const TIP_WET_HI: f32 = 0.20;
/// MIX-1: the pigment-reserve splat's taper shell — the outer fraction of the dab radius over which
/// the splatted value ramps to 0 (`dn ∈ [1−RAMP, 1]`). Keeps the per-pixel depletion map free of
/// binary 255/0 steps at disc boundaries, which the composite's warped nearest-sample rendered as
/// hard pixelated seams where a wash crosses old paint (Enio smoke 2026-07-08). Matches the scale
/// of the coverage feather's own taper; interior pixels take the full value from nearer dabs.
const DEPL_RIM_RAMP: f32 = 0.15;

impl WetShapeStamp {
    /// The dab-local stamp sample at normalised distance `dn`: `(wet, density)`.
    /// - Shape inactive ⇒ `(falloff, 1)` — the plain manual disc.
    /// - IMAGE tip ⇒ wet = the WETNESS envelope (normalised sample through the [`TIP_WET_LO`] ramp —
    ///   the image still REPLACES the falloff, mirroring `compose_shape_silhouette`'s Image rule, so
    ///   the tip's outline shapes the wash); density = the normalised sample (the tip's texture).
    /// - PROCEDURAL tip ⇒ wet = the falloff envelope alone (the pattern must not hole the water);
    ///   density = the pattern value. (Shape Tone ramp deliberately not applied — queued, doc 13 #7.)
    #[inline]
    fn sample(
        &self,
        dn: f32,
        basis: Option<&ph2d_painter_brush::texture::TexDabBasis>,
        img: Option<&ph2d_painter_brush::texture::ImageMask>,
        at: [usize; 2],
        center: [f32; 2],
        radius: f32,
    ) -> (f32, f32) {
        let falloff = self.spec.falloff_weight(dn);
        match basis {
            Some(b) if self.shape_active => {
                let raw = ph2d_painter_brush::texture::sample_shape_silhouette(
                    &self.spec.shape,
                    b,
                    at[0] as i64,
                    at[1] as i64,
                    center,
                    radius,
                    img,
                );
                if self.is_image {
                    // Normalise by the tip's max luminance (per-stroke scan): coverage is wetness
                    // geometry that must SATURATE in the wash core; the RELATIVE texture survives
                    // as density.
                    let n = (raw * self.img_norm).min(1.0);
                    (
                        super::watercolor_field::smoothstep(TIP_WET_LO, TIP_WET_HI, n),
                        n,
                    )
                } else {
                    (falloff, raw.clamp(0.0, 1.0))
                }
            }
            _ => (falloff, 1.0),
        }
    }

    /// The per-dab stamp frame: the flatten/rotate footprint (Shape gizmo `dab_flatten` +
    /// `dab_angle_deg`, spun by Jitter Rotate) + the Shape basis when the slot is active. The
    /// footprint deforms the ENVELOPE distance too ([`Self::env_t`]) — Flatten/Rotate act on the
    /// watercolor stamp like they do on the plain dab.
    fn dab_frame(
        &self,
        d: &Dab,
        rng: &mut u64,
        canvas: [f32; 2],
    ) -> (
        ph2d_painter_brush::footprint::FootprintDeform,
        Option<ph2d_painter_brush::texture::TexDabBasis>,
    ) {
        let fp = self.spec.footprint_deform().rotated_by(d.rotation);
        let basis = self.shape_active.then(|| {
            ph2d_painter_brush::texture::dab_basis(
                &self.spec.shape,
                d.dir,
                rng,
                canvas,
                [1.0, 0.0],
                fp,
            )
        });
        (fp, basis)
    }

    /// The envelope distance for the falloff: the plain `dn` when the footprint is identity (the
    /// bit-exact continuity path — `falloff_t`'s own rounding differs), else the deformed distance
    /// (Flatten squeezes the minor axis; Rotate/Jitter orient it).
    #[inline]
    fn env_t(
        fp: ph2d_painter_brush::footprint::FootprintDeform,
        dn: f32,
        dx: f32,
        dy: f32,
        inv_r: f32,
    ) -> f32 {
        if fp.is_identity() {
            dn
        } else {
            fp.falloff_t(dx * inv_r, dy * inv_r)
        }
    }
}

impl PainterTool {
    /// The two active paint-gate buffers for the watercolor splats (`None` each when its gate is off):
    /// the selection mask + the protection scratch, as cloned `Arc` handles so the splat loops can hold
    /// them alongside their `&mut` buffer borrows. See [`splat_keep`].
    pub(super) fn wet_splat_gates(&self) -> (SplatGate, SplatGate, SplatGate) {
        let sel = self
            .selection_restricts_paint()
            .then(|| Arc::clone(&self.paint.selection_mask));
        let prot = self
            .mask_protection_active()
            .then(|| Arc::clone(&self.paint.mask_scratch_rgba));
        // Alpha-lock (§2.10, doc 13 #8): gate by the FROZEN base the composite reads — the session base
        // during a wet session, else the per-stroke base — so the α reference matches the composite's.
        let alock = self
            .active_alpha_locked()
            .then(|| {
                self.paint
                    .wet_session_base
                    .clone()
                    .or_else(|| self.paint.watercolor_base.clone())
            })
            .flatten();
        (sel, prot, alock)
    }

    /// The manual Shape-driven stamp context when **Automatic is OFF** (doc 13 #1); `None` =
    /// Automatic (the built-in feather stamp — the historical, byte-identical path).
    pub(super) fn wet_shape_stamp(&self) -> Option<WetShapeStamp> {
        let spec = self.paint.brush;
        if spec.watercolor_shape_auto {
            return None;
        }
        let shape_active = spec.shape_silhouette_active(self.paint.shape_image.is_some());
        let is_image = spec.shape.kind == ph2d_painter_brush::TextureKind::Image;
        Some(WetShapeStamp {
            spec,
            shape_active,
            is_image,
            img_norm: self.paint.wet_shape_norm,
        })
    }
    /// Zero the per-stroke coverage (retain capacity). Shape-preview methods (Drag Dot / Anchored / Line)
    /// rebuild coverage each frame from the current batch, so the moving preview leaves no trail.
    ///
    /// The cleared shape's footprint is folded into the frame dirty rect before the cumulative one is
    /// dropped: the moving preview's OLD position must be recomposited (base restored where coverage is
    /// now zero), which a dirty rect of only the NEW dabs would miss (the moving-overlay union).
    pub(super) fn clear_wet_coverage(&mut self) {
        if let Some(c) = self.paint.wet_cum_dirty.take() {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, c),
                None => c,
            });
        }
        self.paint.stroke_coverage.iter_mut().for_each(|c| *c = 0);
        self.paint.stroke_density.iter_mut().for_each(|c| *c = 0);
    }

    /// Zero the per-stroke deposited-colour buffer (retain capacity); twin of [`Self::clear_wet_coverage`].
    pub(super) fn clear_wet_color(&mut self) {
        self.paint.stroke_color.iter_mut().for_each(|c| *c = 0);
    }

    /// Splat each dab's soft disc into the per-stroke coverage (max-blend = the wet "one pass" union,
    /// independent of colour, like wet_edges `stampCoverage`). Coverage is geometric but scaled by the
    /// dab's own opacity, so a faint wash pools a fainter rim.
    pub(super) fn accumulate_wet_coverage(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_coverage.len() != fw * fh {
            self.paint.stroke_coverage = vec![0u8; fw * fh];
        }
        // Track the batch footprint incrementally (wet_edges `expand`): the per-frame dirty rect bounds
        // the live recomposite; the cumulative one gives the pen-up bake its bbox without a canvas scan.
        if let Some(r) = self.dab_batch_region(dabs) {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, r),
                None => r,
            });
            self.paint.wet_cum_dirty = Some(match self.paint.wet_cum_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
            self.paint.wet_stroke_dirty = Some(match self.paint.wet_stroke_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
        }
        // The soak disc follows the newest dab — where the tick heartbeat pours water dwell.
        if let Some(d) = dabs.last() {
            self.paint.wet_soak_pos = Some((d.center, d.radius_px.max(0.0)));
        }
        // Dilution (Wet Mix, `docs/Painter/07` §4): more water lays down LESS coverage → a thinner,
        // paler wash. `flow = 1 − dilution`; default `0` → `flow = 1` (byte-identical).
        let flow = (1.0 - self.paint.brush.wet_dilution).clamp(0.0, 1.0);
        // EDGE-2 (backrun): the WATER the stroke CARRIES pours into the session soak per dab —
        // pure water (Dilution 1 → flow 0, zero pigment coverage) still lands a wet pool the
        // composite rewets/blooms against (the canonical gesture was unreachable: `peak = 0` ⇒
        // the whole path skipped). The dwell pour (`grow_wet_soak`) is TIME-based; the carried
        // water is not. `dilution = 0` ⇒ inert (byte-identical).
        let water = self.paint.brush.wet_dilution.clamp(0.0, 1.0);
        if water > 0.0 && self.paint.stroke_water.len() != fw * fh {
            self.paint.stroke_water = vec![0u8; fw * fh];
        }
        // Selection + protection + alpha-lock gates ([`splat_keep`]): the wash never forms on
        // gated-out texels (nor beyond the layer's existing alpha when locked).
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        // Shape "Automatic" OFF ⇒ the Shape section drives the stamp ([`WetShapeStamp`]); `None`
        // (default) = the built-in feather (byte-identical historical path). The colour splat REPLAYS
        // the same rng stream (it re-reads `tex_rng` and is the one that advances it), so both passes
        // draw identical per-dab Random bases — coverage and colour always agree pixel-wise.
        let stamp = self.wet_shape_stamp();
        // Wrapped Tiling copies are the SAME dab across the seam → one random frame each (`tiling::DabRng`).
        let groups = self.paint.dab_groups.clone();
        // NOT written back here — the colour pass replays the identical stream and advances it.
        let mut rng = super::tiling::DabRng::new(self.paint.tex_rng);
        // Tip-density buffer: sized only when a textured tip is stamping (see `stroke_density` docs);
        // stays empty otherwise so the composite's fast path (density ≡ 1) is untouched. Sized BEFORE
        // the shape-image borrow below (disjoint field borrows carry the loop).
        let textured_tip = stamp.as_ref().is_some_and(|s| s.shape_active);
        if textured_tip && self.paint.stroke_density.len() != fw * fh {
            let mut dens = vec![0u8; fw * fh];
            // Sessão molhada: washs ANTERIORES (tip liso ⇒ densidade plena) não podem ler 0 do
            // mapa recém-criado — backfill 255 sob a cobertura existente (gêmeo do depl abaixo).
            for (d, &c) in dens.iter_mut().zip(self.paint.stroke_coverage.iter()) {
                if c > 0 {
                    *d = 255;
                }
            }
            self.paint.stroke_density = dens;
        }
        let shape_img_owned = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let canvas = [fw as f32, fh as f32];
        // MIX-1: per-dab Charge-depletion factors (None = mixer off — byte-identical). The reserve
        // goes into the per-pixel PIGMENT map, NEVER into the coverage: a sub-saturated coverage
        // leaves `inner` short of 1.0 across the interior and the edge term floods the centre (the
        // flat opaque slab, Enio smoke 2026-07-08). Water footprint intact; pigment fades.
        let depletion = self.wet_mix_depletion(dabs);
        if depletion.is_some() && self.paint.stroke_deplete.len() != fw * fh {
            self.paint.stroke_deplete = vec![0u8; fw * fh];
            // EDGE-1 wet session: the union buffers may already hold PRIOR strokes of this wet
            // session (painted mixer-off ⇒ full reserve, no map). Backfill their footprint with
            // 255, or the union re-bake would multiply their density by 0 — the pools vanished.
            for (d, &c) in self
                .paint
                .stroke_deplete
                .iter_mut()
                .zip(self.paint.stroke_coverage.iter())
            {
                if c > 0 {
                    *d = 255;
                }
            }
        }
        let map_live = depletion.is_some() || !self.paint.stroke_deplete.is_empty();
        // EDGE-1 per-stroke style: the OWNER map — recency, the LAST stroke to touch a pixel
        // styles it (matching the colour buffer's source-over). Sized with the session's first
        // watercolor stroke; `0` = unowned (composite falls back to the current brush).
        let own_v = (!self.paint.wet_styles.table.is_empty())
            .then(|| self.paint.wet_styles.current_owner());
        if own_v.is_some() && self.paint.wet_styles.owner.len() != fw * fh {
            self.paint.wet_styles.owner = vec![0u8; fw * fh];
        }
        let cov = &mut self.paint.stroke_coverage;
        let dens_buf = &mut self.paint.stroke_density;
        let depl_buf = &mut self.paint.stroke_deplete;
        let own_buf = &mut self.paint.wet_styles.owner;
        let water_buf = &mut self.paint.stroke_water;
        for (di, d) in dabs.iter().enumerate() {
            // Frame draw BEFORE any skip: the colour pass replays this stream per emitted dab, and
            // its skip conditions differ (no Dilution `flow` there) — drawing after a skip would
            // desynchronise the two passes' Random draws.
            let rng = rng.enter(&groups, di);
            let frame = stamp.as_ref().map(|s| s.dab_frame(d, &mut *rng, canvas));
            let r = d.radius_px;
            // The dab's pigment reserve (fresh + carry), max-blended over its whole footprint —
            // re-inking a faded trail restores it (a fresh head dab wins over a depleted tail's).
            // Splatted with a RAMP over the dab's outer rim shell (below): a binary 255/0 step at
            // the disc boundary, nearest-sampled at the composite's WARPED coords, read as hard
            // pixelated seams where the wash crosses old paint (Enio smoke 2026-07-08); the ramp
            // matches the coverage's own taper, and stroke-interior pixels take the full value
            // from a nearer dab via the max-blend. While the session map is LIVE, a mixer-OFF
            // stroke still splats FULL reserve (its paint must not read the 0-initialised map).
            let depl_v =
                map_live.then(|| depletion.as_ref().map_or(1.0, |v| v[di].clamp(0.0, 1.0)));
            let peak = (d.coverage * flow).clamp(0.0, 1.0);
            // Water-only dabs (peak 0, Dilution high) still walk the disc to pour the soak.
            if r <= 0.0 || (peak <= 0.0 && water <= 0.0) {
                continue;
            }
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let idx = base + x;
                    let keep = if gated {
                        splat_keep(
                            sel.as_deref().map(Vec::as_slice),
                            prot.as_deref().map(Vec::as_slice),
                            alock.as_deref().map(Vec::as_slice),
                            idx,
                        )
                    } else {
                        1.0
                    };
                    if keep <= 0.0 {
                        continue;
                    }
                    let (wgt, dens) = match (&stamp, &frame) {
                        (Some(s), Some((fp, basis))) => s.sample(
                            WetShapeStamp::env_t(*fp, dn, dx, dy, inv_r),
                            basis.as_ref(),
                            shape_img_owned.as_ref(),
                            [x, y],
                            d.center,
                            r,
                        ),
                        _ => (feather(dn), 1.0),
                    };
                    let v = (peak * keep * wgt * 255.0) as u8;
                    if v > cov[idx] {
                        cov[idx] = v;
                    }
                    // Tip density (textured tip only): max-blend, matching the coverage's "one pass"
                    // union — the composite multiplies it into the interior fill.
                    if textured_tip && wgt > 0.0 {
                        let dv = (dens * 255.0) as u8;
                        if dv > dens_buf[idx] {
                            dens_buf[idx] = dv;
                        }
                    }
                    // Carried water (EDGE-2): max-blend into the session WATER pool — the
                    // composite's backrun ring + lift read it. Feathered by the disc + gates.
                    if water > 0.0 && wgt > 0.0 {
                        let s = (water * wgt * keep * 255.0) as u8;
                        if s > water_buf[idx] {
                            water_buf[idx] = s;
                        }
                        // Px VIRGEM ganha o dono da água (o anel resolve o estilo do traço
                        // d'água PRA SEMPRE, nunca o brush vivo — estabilidade de re-render);
                        // px com pigmento mantém o dono (a água não rouba o estilo de baixo).
                        if let Some(o) = own_v
                            && own_buf[idx] == 0
                        {
                            own_buf[idx] = o;
                        }
                    }
                    // Pigment reserve (MIX-1): max-blend wherever the dab touches, tapered over the
                    // outer rim shell so the map never steps 255→0 at a disc boundary (see above).
                    if let Some(dv) = depl_v
                        && peak > 0.0
                        && wgt > 0.0
                    {
                        let ramp = ((1.0 - dn) / DEPL_RIM_RAMP).min(1.0);
                        let dvb = (dv * ramp * 255.0) as u8;
                        if dvb > depl_buf[idx] {
                            depl_buf[idx] = dvb;
                        }
                    }
                    // Style owner (recency overwrite — see the map's sizing above). A water-only
                    // dab deposits no pigment and must not steal the style of the wash beneath.
                    if let Some(o) = own_v
                        && peak > 0.0
                        && wgt > 0.0
                    {
                        own_buf[idx] = o;
                    }
                }
            }
        }
    }

    /// Splat each dab's colour into the per-stroke colour buffer, **source-over** (recent dab wins on
    /// overlap, carrying its colour — wet_edges `stampColor`), straight-alpha RGBA. Same soft-disc
    /// feather as the coverage, so the deposited colour and the silhouette agree.
    pub(super) fn accumulate_wet_color(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_color.len() != fw * fh * 4 {
            self.paint.stroke_color = vec![0u8; fw * fh * 4];
        }
        // Wet Mix (`docs/Painter/07` §4): the deposited colour is the mixer's per-dab blend of the
        // brush colour with the surface it picked up (Charge/Pull). Off (default `wet_charge = 1`) ⇒
        // each dab's own colour (byte-identical). Computed BEFORE borrowing `stroke_color`.
        let mixed = self.wet_mix_dab_colors(dabs);
        // Selection + protection + alpha-lock gates — twin of the coverage splat (see [`splat_keep`]).
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        // Shape "Automatic" OFF: REPLAY the coverage pass's rng stream from the same seed (identical
        // per-dab Random bases ⇒ colour and coverage agree pixel-wise), then ADVANCE the stroke
        // stream here — net one advance per batch, like every other stamp route.
        let stamp = self.wet_shape_stamp();
        let groups = self.paint.dab_groups.clone(); // same group map ⇒ the two passes stay in lock-step
        let mut rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let shape_img_owned = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let canvas = [fw as f32, fh as f32];
        let buf = &mut self.paint.stroke_color;
        for (di, (d, (dcol, prio, depl))) in dabs.iter().zip(&mixed).enumerate() {
            // Frame draw BEFORE any skip — mirror of the coverage pass (stream sync; see there).
            let rng = rng.enter(&groups, di);
            let frame = stamp.as_ref().map(|s| s.dab_frame(d, &mut *rng, canvas));
            let r = d.radius_px;
            let peak = d.coverage.clamp(0.0, 1.0);
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            // Deposit PRIORITY (Wet Mix): a high-pickup dab writes at full alpha; a low-pickup one
            // (leaving a pool over bare ground) barely writes, so it can't overwrite the stronger
            // picked-up colour — the pool's exit edge stays as coloured as its entry (Enio 2026-07-07).
            // `prio == 1` when the mixer is off ⇒ byte-identical source-over.
            let prio = prio.clamp(0.0, 1.0);
            let depl = depl.clamp(0.0, 1.0);
            let col = [
                (dcol[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ];
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let keep = if gated {
                        splat_keep(
                            sel.as_deref().map(Vec::as_slice),
                            prot.as_deref().map(Vec::as_slice),
                            alock.as_deref().map(Vec::as_slice),
                            base + x,
                        )
                    } else {
                        1.0
                    };
                    // source alpha = silhouette weight × deposit priority × the paint gates —
                    // the SAME weight as the coverage splat (manual Shape stamp or the feather),
                    // so the deposited colour and the silhouette always agree.
                    let wgt = match (&stamp, &frame) {
                        (Some(st), Some((fp, basis))) => {
                            st.sample(
                                WetShapeStamp::env_t(*fp, dn, dx, dy, inv_r),
                                basis.as_ref(),
                                shape_img_owned.as_ref(),
                                [x, y],
                                d.center,
                                r,
                            )
                            .0
                        }
                        _ => feather(dn),
                    };
                    let a = peak * wgt * prio * depl * keep;
                    if a <= 0.0 {
                        continue;
                    }
                    let idx = (base + x) * 4;
                    let da = f32::from(buf[idx + 3]) / 255.0;
                    let na = a + da * (1.0 - a); // straight-alpha over
                    if na <= 0.0 {
                        continue;
                    }
                    for c in 0..3 {
                        let dc = f32::from(buf[idx + c]) / 255.0;
                        let sc = f32::from(col[c]) / 255.0;
                        let out = (sc * a + dc * da * (1.0 - a)) / na;
                        buf[idx + c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    }
                    buf[idx + 3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                }
            }
        }
        // Manual stamp: advance the stroke's texture-rng stream (once per batch — the coverage pass
        // replayed the same seed without writing back). Automatic ⇒ `rng` untouched, write-back is a
        // no-op (the historical stream stays byte-identical).
        self.paint.tex_rng = rng.finish();
    }
}
