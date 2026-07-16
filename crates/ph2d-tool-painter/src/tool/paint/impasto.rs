//! **Impasto** — the height channel: the paint's own thickness (`docs/Painter/16…`).
//!
//! The relief is the *second output* of the dab pipeline that already exists. This module owns the
//! tool-side plumbing of that output; the per-pixel kernel is [`ph2d_painter_brush::height`], and it
//! reads the very same silhouette and grain the colour kernel reads.
//!
//! Three rules hold the design together, and each one is a gate:
//!
//! 1. **The height consumes the dab LIST** the colour consumes — already mirrored by Symmetry,
//!    already replicated by Tiling. It never generates geometry of its own. That is why Mirror and
//!    Tiling (and Stroke, and the shape editors, and Jitter) work under Impasto without a line of
//!    code each — and why they keep working after someone edits them.
//! 2. **The height never draws from the live RNG.** The Grain's per-dab random frame comes off the
//!    persistent `tex_rng` stream; a second pass that drew from it would advance the stream and the
//!    colour would silently get a *different* grain frame than the relief. The pass runs on a COPY of
//!    the stream and throws it away, so it resolves the identical bases and consumes nothing.
//! 3. **A stroke leaves ONE thickness.** Within a stroke the deposit is an envelope — taken on the
//!    PAINT (`stroke_paint`), so the heaviest dab owns the pixel; separate strokes ADD (committed into
//!    the layer at stroke end; how it TOPS OUT is [`super::impasto_ceiling`]). Passing the brush back over its own line does not
//!    build a staircase; going over it again tomorrow does.
//! 4. **The stroke stores its INGREDIENTS, not its height.** The relief is always
//!    `derive_height(spec, paint, grain)`, so every knob in the Body card — Depth, Body, Depth Source,
//!    Smoothing — re-derives the LAST stroke live, and none of them is a special case (Enio,
//!    2026-07-12: *"coloque todos os parâmetros vivos em tempo real para ajustes depois do traço"*).

use super::impasto_settle::{RELIEF_EPS, SETTLE_REACH_PX, for_each_in, owned, settle, union_dirty};
use super::region::grow_region;
use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::height::{
    HeightDab, HeightFields, accumulate_dab_height, derive_height, erase_dab_height,
};
use ph2d_painter_brush::height_push::{
    DEPOSIT_FORWARD_SHARE, PUSH_REACH_MAX_PX, PushBite, bank_dab_push, wave_lobe,
};
use ph2d_painter_brush::{BrushSpec, Dab};

impl PainterTool {
    /// Whether this batch should touch the height field at all: the master switch is on and the brush
    /// is set to write depth. Impasto is **hidden** (§1.2 of the plan) in every mode that does not
    /// deposit fresh paint — Watercolor (which short-circuits far upstream, in `stamp_dabs`), Smear /
    /// Blur / Clone (they move paint that is already down; dragging relief is `Plow`, deferred and
    /// named), Mask (a grayscale channel has no body) and Inpaint (a heal disc that ignores the brush
    /// entirely). Those modes never reach here, except Mask — which routes *through* `stamp_dabs_inner`
    /// — so the mode gate is load-bearing, not decorative.
    fn impasto_batch_active(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::Paint) && self.paint.brush.impasto
    }

    /// Deposit (or, for the Eraser, scrub) this dab batch's HEIGHT.
    ///
    /// Called from the ONE choke point in the route dispatcher, with the list already tiled — so every
    /// route (cached / canvas-cached / per-pixel / ramped / per-layer) contributes relief through the
    /// same path, and a route added tomorrow gets it for free.
    pub(super) fn stamp_dabs_height(&mut self, dabs: &[Dab], brush: &BrushSpec) {
        if dabs.is_empty() || !self.impasto_batch_active() {
            return;
        }
        let erasing = self.paint.eraser;
        if !erasing && !brush.touches_height() {
            return; // no body laid AND none shoved aside ⇒ pigment only
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return;
        }

        // The Eraser scrubs the relief that is ALREADY committed to the layer (there is nothing of its
        // own to build an envelope from); a paint stroke builds its envelope in `stroke_height` and the
        // layer takes it at stroke end. Both write an f32 field of exactly this shape.
        let Some(active) = self.layers.active() else {
            return;
        };
        // The ERASER scrubs the relief already committed to the layer (there is nothing of its own to
        // build an envelope from); a PAINT stroke builds its ingredients in `stroke_paint`/`stroke_grain`
        // and the layer takes them at stroke end.
        let mut erase_buffers = None;
        if erasing {
            match self.heights.remove(&active).map(owned) {
                Some(f) if f.len() == n => {
                    let c = self
                        .covers
                        .remove(&active)
                        .map(owned)
                        .filter(|c| c.len() == n);
                    erase_buffers = Some((f, c.unwrap_or_else(|| vec![0u8; n])));
                }
                // Nothing to erase (no relief on this layer) — and a stale, differently-sized field is
                // dropped rather than indexed into (the shape guard the sweep taught us to write).
                _ => return,
            }
        }
        let (mut field, mut cover) = erase_buffers.unwrap_or_default();
        let (mut paint, mut grain, mut film, mut radius) = if erasing {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else {
            let mut h = std::mem::take(&mut self.paint.relief.stroke_height);
            let mut p = std::mem::take(&mut self.paint.relief.stroke_paint);
            let mut g = std::mem::take(&mut self.paint.relief.stroke_grain);
            let mut f = std::mem::take(&mut self.paint.relief.stroke_film);
            let mut r = std::mem::take(&mut self.paint.relief.stroke_radius);
            // Lazily sized by the first dab of the stroke; zero cost for a document nobody sculpts.
            if h.len() != n {
                h = vec![0.0; n];
            }
            if p.len() != n {
                p = vec![0.0; n];
            }
            if g.len() != n {
                g = vec![0u8; n];
            }
            if f.len() != n {
                f = vec![0u8; n];
            }
            if r.len() != n {
                r = vec![0.0; n];
            }
            field = h;
            (p, g, f, r)
        };
        // The displacement's own plane, and the GROUND it bites into — the layer's relief as the stroke
        // found it. Recorded whenever there IS paint to shove, NOT only when the knob is up: Push has to
        // be dialable AFTER the stroke like every other knob in the Body card, and it cannot be if the
        // ingredient was never written down. A first stroke on bare canvas has no ground, so it pays
        // nothing — the cost falls exactly where the feature is, on paint laid over paint.
        let ground = (!erasing)
            .then(|| self.heights.get(&active).cloned())
            .flatten();
        let mut push_plane = std::mem::take(&mut self.paint.relief.stroke_push);
        if ground.is_some() && push_plane.len() != n {
            push_plane = vec![0.0; n];
        }
        let mut scratch = std::mem::take(&mut self.paint.relief.push_scratch);

        // Resolve each dab's frames EXACTLY as the colour route will — same `d.dir` (Rake), same
        // footprint (Jitter Rotate), same Random draws, same order (Shape before Grain). The RNG is a
        // COPY: this pass must not advance the stream the colour pass is about to read (rule 2).
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        let grain_active = brush.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        // Each dab's body is swept back to the PREVIOUS dab's centre, so the bodies join into the
        // stroke's distance field instead of a string of beads (see `accumulate_dab_height`). Finding
        // that centre is the whole subtlety, and it is why this is not a one-liner:
        //
        //  * Symmetry INTERLEAVES its copies — `push_symmetric` emits `[base, mirror, base, mirror, …]`.
        //    Linking neighbours in the list would draw a capsule from a dab to its own mirror image: a
        //    bar straight across the canvas. The path predecessor is `copies` entries back.
        //  * Tiling REPLICATES the list — `groups[j]` is the index of the ORIGINAL dab entry `j` is a
        //    wrapped copy of. So the predecessor of a wrapped copy is the predecessor of its original,
        //    carrying the SAME wrap offset.
        //
        // Both fall out of data the list already carries. No path is reconstructed.
        let copies = if !brush.symmetry.enabled {
            1
        } else if brush.symmetry.circular {
            brush.symmetry.segments().max(1) as usize
        } else {
            2
        };
        // The bow wave's cargo + last-painted lobe, one per Symmetry copy (each copy has its own
        // travelling tip). Sized here because `copies` is a fact about THIS batch's brush.
        let mut wave = std::mem::take(&mut self.paint.relief.stroke_wave);
        if ground.is_some() {
            wave.resize(copies.max(1), (0.0, None));
        }
        // The un-wrapped centre of each original dab (indexed by group), so a wrapped copy can recover
        // its own offset. With Tiling off, `groups` is empty and the entry IS its own original.
        let origin_center = |gi: usize| -> [f32; 2] {
            groups.iter().position(|&g| g as usize == gi).map_or_else(
                || dabs.get(gi).map_or([0.0, 0.0], |d| d.center),
                |first| dabs[first].center,
            )
        };
        // …and its radius: the capsule law below needs to know how big the PREDECESSOR was.
        let origin_radius = |gi: usize| -> f32 {
            groups.iter().position(|&g| g as usize == gi).map_or_else(
                || dabs.get(gi).map_or(0.0, |d| d.radius_px),
                |first| dabs[first].radius_px,
            )
        };
        // **The capsule law.** A dab's body is swept back to the previous dab's centre so overlapping
        // stamps join into the stroke's true distance field instead of a string of beads — but the
        // sweep's whole premise is that the SEGMENT between the two centres is guaranteed paint. That
        // is only true when the stamps genuinely overlap: each disc contains the other's centre
        // (`dist ≤ min(r, r_prev)`). Airbrush jumps (timer dabs under a fast cursor), per-dab position
        // scatter and Jitter-Scale-shrunken dabs all violate it — and the colour paints BEADS there,
        // while the height swept a TUBE across bare canvas: film + relief with no pigment under it,
        // which the light dutifully shades. That is Enio's live smoke of 2026-07-15, twice — the grey
        // bars "ligando os pontos" beside the paint. Where the premise fails, the dab is a bead,
        // exactly like its pigment.
        let sweepable = |center: [f32; 2], radius: f32, prev: [f32; 2], prev_r: f32| -> bool {
            let (dx, dy) = (center[0] - prev[0], center[1] - prev[1]);
            let lim = radius.min(prev_r).max(0.0);
            dx * dx + dy * dy <= lim * lim
        };

        let mut touched: Option<Region> = None;
        for (di, d) in dabs.iter().enumerate() {
            let tex_rng = dab_rng.enter(&groups, di);
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            let fp = spec.footprint_deform().rotated_by(d.rotation);
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    d.dir,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    [1.0, 0.0],
                    fp,
                )
            });
            let grain_basis = grain_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    d.dir,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    [1.0, 0.0],
                    fp,
                )
            });
            // The path predecessor, carrying THIS entry's Tiling wrap — kept only when the capsule law
            // holds (see `sweepable` above); a non-overlapping predecessor makes this dab a bead.
            let gi = groups.get(di).map_or(di, |g| *g as usize);
            let prev_center = if gi >= copies {
                let here = origin_center(gi);
                let there = origin_center(gi - copies);
                let off = [d.center[0] - here[0], d.center[1] - here[1]];
                let prev = [there[0] + off[0], there[1] + off[1]];
                sweepable(d.center, d.radius_px, prev, origin_radius(gi - copies)).then_some(prev)
            } else {
                // First sample of this batch: chain to where the stroke was when the last batch ended,
                // per symmetry copy — without this the relief would bead at every pointer event, which is
                // a beading the artist's hardware chose, not their hand.
                self.paint
                    .relief
                    .last_height_center
                    .get(gi)
                    .copied()
                    .flatten()
                    .and_then(|(prev, prev_r)| {
                        let here = origin_center(gi);
                        let off = [d.center[0] - here[0], d.center[1] - here[1]];
                        let prev = [prev[0] + off[0], prev[1] + off[1]];
                        sweepable(d.center, d.radius_px, prev, prev_r).then_some(prev)
                    })
            };
            let hd = HeightDab {
                center: d.center,
                radius: d.radius_px,
                coverage: d.coverage,
                footprint: fp,
                prev_center,
                shape: shape_basis
                    .as_ref()
                    .map(|sb| ph2d_painter_brush::ShapeInput {
                        basis: sb,
                        image: shape_image.as_ref(),
                        ramp_lut: shape_ramp_lut,
                    }),
                grain: grain_basis.as_ref(),
                grain_image: grain_image.as_ref(),
            };
            // UN-paint this copy's standing wave lobe FIRST — before the dab's own deposit
            // touches `stroke_paint`, so the `(1 − paint)` weights recompute to the exact numbers
            // that painted it and the subtraction is bit-for-bit (the single-book law).
            let copy_slot = gi % copies.max(1);
            let mut plane_touched: Option<ph2d_painter_brush::dab::DirtyRect> = None;
            if !erasing
                && ground.is_some()
                && let Some((vol, Some(tip))) = wave.get(copy_slot).map(|(v, t)| (*v, *t))
                && vol > 0.0
            {
                let tip_spec = BrushSpec {
                    radius_px: tip.radius,
                    ..*brush
                };
                let tip_hd = ph2d_painter_brush::height::HeightDab {
                    center: tip.center,
                    radius: tip.radius,
                    coverage: 1.0,
                    footprint: tip_spec.footprint_deform().rotated_by(tip.rotation),
                    prev_center: tip.prev_center,
                    shape: None,
                    grain: None,
                    grain_image: None,
                };
                if let Some(r) = wave_lobe(
                    &mut push_plane,
                    &paint,
                    &mut scratch,
                    w,
                    h,
                    &tip_hd,
                    vol,
                    -1.0,
                ) {
                    plane_touched = Some(r);
                }
                wave[copy_slot].1 = None;
            }
            let hit = if erasing {
                erase_dab_height(&mut field, &mut cover, w, h, &spec, &hd)
            } else {
                // The BITE rides inside the deposit's own walk (which already knows the silhouette and the
                // envelope-so-far); the BANK is a separate pass over the RIM, which the deposit never
                // touches. Two halves of one conservation law, each walked exactly once — doing the bite
                // in a kernel of its own meant evaluating the silhouette twice per texel, and that alone
                // put the impasto cost at 5.0 ms/move, past its budget, on every stroke.
                let mut bite = ground.as_ref().map(|g0| PushBite {
                    ground: g0,
                    plane: &mut push_plane,
                    displaced: 0.0,
                });
                let mut fields = HeightFields {
                    height: &mut field,
                    paint: &mut paint,
                    grain: &mut grain,
                    film: &mut film,
                    radius: &mut radius,
                };
                let laid = accumulate_dab_height(&mut fields, w, h, &spec, &hd, bite.as_mut());
                let displaced = bite.map_or(0.0, |b| b.displaced);
                // The bank AIMS by the path: a first dab with no predecessor still has a heading
                // (`d.dir`, warmed up when Push > 0), and a one-pixel synthetic prev turns it into
                // the bank's direction WITHOUT changing the sweep — the W5 Conserve paid for this
                // pattern first. Without it the pen-down dab banks a radial ring, and the stroke's
                // start reads as a stamped cookie-cutter instead of where a blade set off.
                let bank_hd = if hd.prev_center.is_none() && d.dir != [0.0, 0.0] {
                    ph2d_painter_brush::height::HeightDab {
                        prev_center: Some([d.center[0] - d.dir[0], d.center[1] - d.dir[1]]),
                        ..hd
                    }
                } else {
                    ph2d_painter_brush::height::HeightDab { ..hd }
                };
                let banked = ground.as_ref().map(|_| {
                    bank_dab_push(
                        &mut push_plane,
                        &paint,
                        &mut scratch,
                        w,
                        h,
                        &bank_hd,
                        displaced,
                        DEPOSIT_FORWARD_SHARE,
                    )
                });
                let (banked, carried) = match banked {
                    Some((r, c)) => (r, c),
                    None => (None, 0.0),
                };
                // The wave rolls: this dab's carried share joins the copy's cargo, and the whole
                // cargo is painted as a lobe ahead of THIS tip (the old lobe was un-painted above).
                // A directionless dab refuses the lobe and keeps the cargo for the next one that
                // moves; at pen-up whatever lobe stands last simply STAYS — the wave rests at the
                // stroke's frontier, which is the whole point (IMPaSTo's `v_p = −c∇p`).
                let mut waved: Option<ph2d_painter_brush::dab::DirtyRect> = None;
                if ground.is_some() && copies > 0 {
                    let slot = &mut wave[copy_slot];
                    slot.0 += carried;
                    if slot.0 > 0.0
                        && let Some(r) = wave_lobe(
                            &mut push_plane,
                            &paint,
                            &mut scratch,
                            w,
                            h,
                            &bank_hd,
                            slot.0,
                            1.0,
                        )
                    {
                        waved = Some(r);
                        slot.1 = Some(crate::tool::paint::relief_state::WaveTip {
                            center: bank_hd.center,
                            radius: bank_hd.radius,
                            rotation: d.rotation,
                            prev_center: bank_hd.prev_center,
                        });
                    }
                }
                let banked = [banked, waved, plane_touched]
                    .into_iter()
                    .flatten()
                    .reduce(union_dirty);
                // The relief the artist SEES while dragging is `field` (the light's `live_h`). The deposit
                // wrote itself in; the displacement has to be folded in too, or the ridge would appear
                // only at pen-up — which is exactly what Enio's smoke found (2026-07-12).
                let push = spec.effective_impasto_push();
                if let (Some(r), true) = (banked, push > 0.0) {
                    for py in r.y..r.y + r.h {
                        let row = (py as usize) * (w as usize);
                        for px in r.x..r.x + r.w {
                            let i = row + px as usize;
                            let deposit =
                                derive_height(&spec, paint[i], f32::from(grain[i]) / 255.0);
                            field[i] = deposit + push * push_plane[i];
                        }
                    }
                }
                match (laid, banked) {
                    (Some(a), Some(b)) => Some(union_dirty(a, b)),
                    (a, b) => a.or(b),
                }
            };
            if let Some(r) = hit {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
                // …and the union over the WHOLE stroke, which is the window the commit works in. The
                // per-batch `touched` above is the dirty rect for this pointer event; this one outlives
                // it (see `PaintState::stroke_relief_bbox`).
                self.paint.relief.stroke_relief_bbox = Some(
                    self.paint
                        .relief
                        .stroke_relief_bbox
                        .map_or(rect, |acc| union_region(acc, rect)),
                );
            }
        }
        // Remember where each Symmetry copy ended, so the NEXT pointer batch sweeps back to it instead
        // of starting a fresh bead.
        if !dabs.is_empty() {
            self.paint.relief.last_height_center.clear();
            self.paint.relief.last_height_center.resize(copies, None);
            let last_group = groups.last().map_or(dabs.len() - 1, |g| *g as usize);
            for c in 0..copies {
                // The last full round of copies in this batch: group indices `last_group - (copies-1) ..= last_group`.
                let gi = last_group.saturating_sub(copies - 1 - c);
                self.paint.relief.last_height_center[c] =
                    Some((origin_center(gi), origin_radius(gi)));
            }
        }
        // The RNG copy dies here: `self.paint.tex_rng` is deliberately NOT written back (rule 2).
        self.paint.relief.stroke_push = push_plane; // the displacement banked so far, at Push = 1
        self.paint.relief.push_scratch = scratch;
        self.paint.relief.stroke_wave = wave;

        if erasing {
            // The eraser scrubs the LAYER directly, so the live stroke's ground is no longer what it
            // says it is — drop it rather than let a later Depth drag resurrect erased paint.
            self.drop_live_relief();
            self.heights.insert(active, std::sync::Arc::new(field));
            self.covers.insert(active, std::sync::Arc::new(cover));
            self.sync_relief_flags();
        } else {
            self.paint.relief.stroke_height = field;
            self.paint.relief.stroke_paint = paint;
            self.paint.relief.stroke_grain = grain;
            self.paint.relief.stroke_film = film;
            self.paint.relief.stroke_radius = radius;
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Drop the in-progress stroke's relief. Called at pen-down, and again before each re-stamp of the
    /// shape editors' live preview (Line / Curve / Ellipse / Polygon / Free Hand re-stamp the WHOLE
    /// shape every pointer move over a restored canvas — without this the envelope would keep the
    /// relief of every shape the artist dragged THROUGH, and a curve would leave a trail of ghosts).
    pub(crate) fn reset_stroke_height(&mut self) {
        self.paint.relief.stroke_height.clear();
        self.paint.relief.stroke_paint.clear();
        self.paint.relief.stroke_grain.clear();
        self.paint.relief.stroke_film.clear();
        self.paint.relief.stroke_radius.clear();
        self.paint.relief.stroke_push.clear(); // the displacement is per-stroke; a re-stamp starts it over
        self.paint.relief.stroke_wave.clear(); // the wave is a fact about the dab list; it restarts with it
        self.paint.relief.stroke_relief_bbox = None; // the commit's window is per-stroke too
        self.paint.relief.last_height_center.clear(); // the sweep chain restarts with the stroke
    }

    /// Merge the finished stroke into the active layer, and hand its INGREDIENTS to the live edit.
    ///
    /// **Add**, not envelope: within a stroke the brush leaves one thickness, but a *second* stroke
    /// over the same paint genuinely piles more on (and a carving stroke digs further). Called from
    /// `close_stroke`, BEFORE the undo entry is recorded, so the step captures the relief with the
    /// pigment that made it — one Ctrl+Z takes both.
    pub(super) fn commit_stroke_height(&mut self) {
        if self.paint.relief.stroke_paint.is_empty() {
            return;
        }
        let paint = std::mem::take(&mut self.paint.relief.stroke_paint);
        let grain = std::mem::take(&mut self.paint.relief.stroke_grain);
        let film = std::mem::take(&mut self.paint.relief.stroke_film);
        let radius = std::mem::take(&mut self.paint.relief.stroke_radius);
        self.paint.relief.stroke_height.clear(); // it is derived; the ingredients are the truth
        let (Some(active), Some(bbox)) =
            (self.layers.active(), self.paint.relief.stroke_relief_bbox)
        else {
            return;
        };
        // THE WINDOW. The stroke touched `bbox`; the settle reaches `SETTLE_MAX_PX` beyond it, the
        // displacement's rim reaches `PUSH_REACH_PX`, and the relief is exactly ZERO past both — so
        // everything the commit does (derive, settle, PUSH, re-base, diff) belongs inside this rect and
        // nowhere else. Both reaches are CONSTANTS, which is what keeps the window a function of the
        // stroke and not of the brush. Cropping is not an approximation: the blur
        // of a zero field is zero, and the settle CLAMPS at its buffer's edge, which for a window whose
        // border is already zero replicates the same zero the whole-canvas pass would have read. The
        // result is byte-identical, and it is gated as byte-identical.
        let (w, h) = self.source_size;
        let Some(rect) = grow_region(bbox, SETTLE_REACH_PX + PUSH_REACH_MAX_PX as u32, w, h) else {
            return;
        };

        // Coverage merges by MAX, not by sum: two strokes over the same spot do not make the pixel
        // "200% paint". (The HEIGHT does add — more paint IS thicker. The two are different quantities,
        // which is the whole reason the light needs both.) Only inside the window: outside it the
        // stroke's paint is zero, and `max(x, 0)` is `x`.
        {
            let n = (w as usize) * (h as usize);
            let dst = std::sync::Arc::make_mut(self.covers.entry(active).or_default());
            if dst.len() != n {
                dst.resize(n, 0);
            }
            for_each_in(rect, w, |i| {
                dst[i] = dst[i].max(film.get(i).copied().unwrap_or(0));
            });
        }
        // The MATERIAL rides the same stroke — it is what the paint this stroke laid IS
        // (`ph2d_painter_brush::material`). Deposited from the brush, so Symmetry / Tiling / the shape
        // editors sculpt the material exactly as they sculpt the relief, for free: it is the second
        // output of the SAME dab list, which is the rule this whole section is built on.
        //
        // Merged by **`over`**, not by `max` like the coverage — and the difference is not a detail.
        // Coverage is a PRESENCE (two layers of paint over one pixel are not 200% paint, so `max`).
        // Material is an IDENTITY: the paint on top is the paint you see, so new paint replaces the old
        // in proportion to how opaquely it covers it. That is the same `over` operator the pigment
        // itself uses, which is why a translucent glaze of gouache over oil reads as a mix of the two
        // and an opaque one reads as gouache.
        //
        // The un-painted ground starts at NEUTRAL rather than at zero: zero is `roughness = 0`, which is
        // GLOSSY — so a stroke's translucent rim would fade toward a mirror instead of toward bare paper.
        {
            let n = (w as usize) * (h as usize);
            let mat = self.paint.brush.material().to_bytes();
            let neutral = ph2d_painter_brush::material::Material::NEUTRAL.to_bytes();
            let dst = std::sync::Arc::make_mut(self.mats.entry(active).or_default());
            if dst.len() != n {
                dst.resize(n, neutral);
            }
            // The ground the material sits on, and the film it merges with — kept as patches over the
            // window so the four material knobs stay LIVE on the stroke the artist just laid, exactly
            // like Depth and Body are. (`over` does not compose, so a re-bake must start from the base.)
            let mut base = Vec::with_capacity((rect.w as usize) * (rect.h as usize));
            let mut kept_film = Vec::with_capacity((rect.w as usize) * (rect.h as usize));
            for_each_in(rect, w, |i| {
                base.push(dst[i]);
                kept_film.push(film.get(i).copied().unwrap_or(0));
            });
            self.paint.relief.live_mat_base = base;
            self.paint.relief.live_film = kept_film;
            for_each_in(rect, w, |i| {
                let a = u32::from(film.get(i).copied().unwrap_or(0));
                if a == 0 {
                    return; // no paint from this stroke here: the material under it is untouched
                }
                for c in 0..mat.len() {
                    // `over` in 8-bit, rounded: dst = dst·(1−a) + src·a.
                    let old = u32::from(dst[i][c]);
                    let new = u32::from(mat[c]);
                    dst[i][c] = ((old * (255 - a) + new * a + 127) / 255) as u8;
                }
            });
        }
        // Keep the stroke's INGREDIENTS and the layer's relief from BEFORE it. Between them, the whole
        // Body card re-derives this stroke after the fact — the artist lays a stroke and then dials it
        // in while looking at it, exactly like every other property in this panel.
        //
        // The ground is kept as a PATCH: cloning the layer's whole height plane to re-add it was a
        // 64 MB copy per stroke at 4096², for a ground that is only ever read inside the window.
        self.paint.relief.live_relief_had_entry = self.heights.contains_key(&active);
        self.paint.relief.live_relief_base = match self.heights.get(&active) {
            Some(f) if f.len() == (w as usize) * (h as usize) => {
                let mut base = Vec::with_capacity((rect.w as usize) * (rect.h as usize));
                for_each_in(rect, w, |i| base.push(f[i]));
                base
            }
            _ => Vec::new(),
        };
        self.paint.relief.live_relief_rect = Some(rect);
        self.paint.relief.live_relief_layer = Some(active);
        self.paint.relief.live_paint = paint;
        self.paint.relief.live_grain = grain;
        self.paint.relief.live_radius = radius;
        // The displacement at Push = 1 — the third ingredient. The whole displacement is LINEAR in Push,
        // so keeping it lets the knob stay live after the stroke without replaying a single dab.
        self.paint.relief.live_push = std::mem::take(&mut self.paint.relief.stroke_push);
        self.rebuild_live_layer_relief();
    }

    /// Re-derive the live stroke onto the layer at the CURRENT Depth / Body / Depth Source / Smoothing.
    ///
    /// No re-stroking and no repainting: the stroke's paint and grain are stored, and the relief is a
    /// pure function of them ([`derive_height`]) — so a new setting is one pass over the buffer. This
    /// is the single point where the last stroke's relief is made, at commit and at every edit; the
    /// deposit and the edit therefore cannot drift.
    fn rebuild_live_layer_relief(&mut self) {
        let (Some(layer), Some(rect), false) = (
            self.paint.relief.live_relief_layer,
            self.paint.relief.live_relief_rect,
            self.paint.relief.live_paint.is_empty(),
        ) else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.relief.live_paint.len() != n
            || self.paint.relief.live_grain.len() != n
            || self.paint.relief.live_radius.len() != n
        {
            return; // a stale, differently-sized ingredient plane: the shape guard, never an index panic
        }
        let (rw, rh) = (rect.w as usize, rect.h as usize);
        let cells = rw * rh;

        // 1. Derive the stroke's relief — inside the WINDOW only. Outside it the paint is zero, so the
        //    relief is zero, so there is nothing to derive and nothing to write.
        // Derived at the radius that MADE each texel (the third ingredient), not at the panel
        // brush's: a drag-sized Anchored ball is as tall committed as it was live, and dialling the
        // Size slider after the stroke does not re-scale relief already on the canvas.
        let brush = self.paint.brush;
        let mut spec_i = brush;
        let mut field: Vec<f32> = Vec::with_capacity(cells);
        for_each_in(rect, w, |i| {
            let g = f32::from(self.paint.relief.live_grain[i]) / 255.0;
            spec_i.radius_px = self.paint.relief.live_radius[i];
            field.push(derive_height(&spec_i, self.paint.relief.live_paint[i], g));
        });

        // 2. Settle it. The window's border is zero (it was grown by the blur's reach), so the settle's
        //    edge-clamp replicates the same zero a whole-canvas pass would have read from outside — the
        //    crop is byte-identical, not an approximation. This is the 258 ms that used to run on 16 M
        //    texels at 4096² for a stroke that touched a corner of the canvas.
        let smoothing = brush.effective_impasto_smoothing();
        if smoothing > 0.0 {
            settle(&mut field, rect.w, rect.h, smoothing);
        }

        // 3. Add the ground back and write it into the layer — tracking, as we go, the box that actually
        //    MOVED. What the light was showing until this instant is the relief this call replaces (the
        //    raw envelope during the stroke; the previous setting's field on a knob edit), and no PIXEL
        //    changed, so nothing else on this path would mark the canvas dirty: the composite cache would
        //    go on showing the lighting it last drew. That is Enio's *"o primeiro traço aplica o smoothing;
        //    a partir do segundo não"* (2026-07-12) — the first stroke was rescued by accident, because it
        //    flips the layer's `has_relief` and that invalidates the composite.
        let base = std::mem::take(&mut self.paint.relief.live_relief_base);
        let has_base = base.len() == cells;

        // 2b. PUSH — volume conservation. The brush shoved the paint it found out of its way, and that
        //     paint banked up as a ridge just outside the cut (`bank_dab_push`, dab by dab, AS THE BRUSH
        //     MOVED). What is stored is the displacement at `Push = 1` — negative where paint was taken,
        //     positive where it was banked, summing to exactly zero — and the whole thing is LINEAR in
        //     Push, so the knob stays alive after the stroke and re-deriving it costs one multiply.
        //
        //     The GROUND is never mutated: `ground' = ground + push·R₁`. Idempotent by construction, which
        //     is what lets the SHAPE editors re-stamp the whole shape on every pointer move without
        //     carving a canyon.
        let push = brush.effective_impasto_push();
        let has_push = push > 0.0 && self.paint.relief.live_push.len() == n;
        let target = std::sync::Arc::make_mut(
            self.heights
                .entry(layer)
                .or_insert_with(|| std::sync::Arc::new(vec![0.0; n])),
        );
        if target.len() != n {
            target.resize(n, 0.0);
        }
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let mut any_relief = false;
        for ry in 0..rh {
            for rx in 0..rw {
                let c = ry * rw + rx;
                let i = (rect.y as usize + ry) * (w as usize) + rect.x as usize + rx;
                // Strokes ADD — up to the glass ceiling (see [`H_CEIL`]). A lone stroke never reaches it
                // (`|depth| ≤ 1`), so the clamp only ever bites where strokes genuinely pile up.
                let g0 = if has_base { base[c] } else { 0.0 };
                // `R₁` IS the displacement (it sums to zero), so the ground plus a scaled copy of it is
                // the whole story — and it is linear in Push, which is what keeps the knob live.
                let under = if has_push {
                    g0 + push * self.paint.relief.live_push[i]
                } else {
                    g0
                };
                let next = (field[c] + under).clamp(
                    -super::impasto_ceiling::H_MAX,
                    super::impasto_ceiling::H_MAX,
                );
                if next != 0.0 {
                    any_relief = true;
                }
                // A texel that moved by less than a 16-bit tick of the height range cannot change a
                // single output byte; chasing it would repaint the canvas for nothing.
                if (next - target[i]).abs() > RELIEF_EPS {
                    let (px, py) = (rect.x + rx as u32, rect.y + ry as u32);
                    x0 = x0.min(px);
                    y0 = y0.min(py);
                    x1 = x1.max(px);
                    y1 = y1.max(py);
                }
                target[i] = next;
            }
        }
        self.paint.relief.live_relief_base = base; // PRISTINE — the ground is re-read on every knob edit

        // 4. A layer that carried nothing before this stroke and carries nothing now (Depth 0) drops its
        //    entry — but a layer with relief ELSEWHERE keeps it: the window is all this call can speak
        //    for. (`any_relief` is the window's verdict; `live_relief_had_entry` is the rest of the map.)
        if !any_relief && !self.paint.relief.live_relief_had_entry {
            self.heights.remove(&layer);
        }

        if x0 != u32::MAX {
            // Grow by one: the light reads a pixel's NEIGHBOURS (the normal is a central difference), so
            // a texel just outside the changed box is lit by a slope that changed inside it.
            //
            // Stated plainly: **no gate catches this one.** Deleting the grow leaves the suite green, and
            // I could not build a red for it — this path re-derives through a BLUR, so the height change
            // decays to `RELIEF_EPS` at the box's edge and the neighbour's normal shifts by less than an
            // output byte. It is a correctness margin (a hard-edged relief edit arriving here would need
            // it), not an observed fix, and it is written down rather than left to look defended.
            let moved = Region {
                x: x0,
                y: y0,
                w: x1 - x0 + 1,
                h: y1 - y0 + 1,
            };
            if let Some(r) = grow_region(moved, 1, w, h) {
                self.mark_dirty(r);
            }
        }
        self.sync_relief_flags();
    }

    /// Publish onto the layer stack the one fact about relief the PANEL cannot derive: which layers
    /// carry any (`Layer::has_relief`).
    ///
    /// The relief lives here, in the tool, next to the pixels; the panel only ever sees a clone of the
    /// stack. So this is a projection, and the direction matters — the height map is the authority and
    /// the flag is downstream of it, never the reverse. It is what lets the Depth row appear on exactly
    /// the rows it can act on, and it is why a document nobody has sculpted shows no impasto chrome at
    /// all.
    ///
    /// Called wherever `heights` changes (every one of them, which is the invariant the gate pins).
    /// `O(layers)` and allocation-free: it reads a `BTreeMap` key, it does not scan a canvas.
    pub(crate) fn sync_relief_flags(&mut self) {
        let mut changed = false;
        let ids: Vec<crate::tool::RtLayerId> = self.layers.all_ids().collect();
        for id in ids {
            let has = self.heights.contains_key(&id);
            if let Some(l) = self.layers.get_mut(id)
                && l.has_relief != has
            {
                l.has_relief = has;
                changed = true;
            }
        }
        // The panel republishes on the layer revision, and NOTHING else bumps it here: a paint stroke is
        // a pixel edit, not a structural one. Without this, sculpting the first ridge on a layer would
        // set the flag and the panel would never hear about it — the Depth row would appear only after
        // some unrelated layer edit happened to bump the revision.
        //
        // Bump the revision, and ONLY the revision. The obvious `invalidate_composite()` also drops the
        // whole composite cache and every adjustment cut-cache, forcing a full recompose of the canvas:
        // **225 ms at 4096²**, spent on the first stroke of every layer, to publish a boolean. The relief
        // the artist actually just laid is already on screen — `rebuild_live_layer_relief` dirtied
        // exactly the texels that moved. Nothing here needs a single pixel recomposited.
        if changed {
            self.bump_layers_revision();
        }
    }

    /// A Body-card edit (Depth / Body / Depth Source / Smoothing): re-derive the last stroke in place.
    /// No-op unless that stroke is on the layer the artist is looking at — dialling Depth after
    /// switching layers must not reach back and re-sculpt a stroke on some other one.
    pub(super) fn refresh_live_relief(&mut self) {
        if !self.impasto_live_edit() // "Adjust Last Stroke" — finished paint stays finished
            || !self.paint.brush.impasto
            || self.layers.active() != self.paint.relief.live_relief_layer
        {
            return;
        }
        self.rebuild_live_layer_relief();
        self.invalidate_composite();
    }

    /// Forget the live stroke — its ground is no longer valid (an erase, an undo, a fresh document).
    pub(crate) fn drop_live_relief(&mut self) {
        self.paint.relief.live_paint = Vec::new();
        self.paint.relief.live_grain = Vec::new();
        self.paint.relief.live_radius = Vec::new();
        self.paint.relief.live_push = Vec::new();
        self.paint.relief.live_relief_base = Vec::new();
        self.paint.relief.live_film = Vec::new();
        self.paint.relief.live_mat_base = Vec::new();
        self.paint.relief.live_relief_rect = None;
        self.paint.relief.live_relief_had_entry = false;
        self.paint.relief.live_relief_layer = None;
    }

    /// The relief the artist should SEE right now for `id`: the committed layer height plus the
    /// in-progress stroke's envelope. They are separate buffers (the envelope is what stops a stroke
    /// stacking on itself), so anything that reads the relief as a whole has to add them.
    ///
    /// This MATERIALISES the sum. The light pass deliberately does not use it — it samples the layers in
    /// place (`ReliefFields`), because building a canvas-sized buffer every frame cost twice the whole
    /// impasto budget while the pass only ever lights the dirty rect. Kept as the accessor for anything
    /// that genuinely wants the field (the gates do), not as the hot path.
    #[must_use]
    pub fn layer_height_view(&self, id: crate::tool::RtLayerId) -> Option<Vec<f32>> {
        let committed = self.heights.get(&id);
        let live = (!self.paint.relief.stroke_height.is_empty()
            && self.layers.active() == Some(id)
            && !self.paint.eraser)
            .then_some(&self.paint.relief.stroke_height);
        match (committed, live) {
            (None, None) => None,
            (Some(c), None) => Some(c.as_ref().clone()),
            (None, Some(l)) => Some(l.clone()),
            (Some(c), Some(l)) if c.len() == l.len() => {
                Some(c.iter().zip(l.iter()).map(|(a, b)| a + b).collect())
            }
            (Some(c), Some(_)) => Some(c.as_ref().clone()),
        }
    }
}

// The deposit's SETTLE (the box blur that lets paint relax under its own weight), its reach, and the
// sub-visible threshold the dirty-rect diff uses, all live in the sibling `impasto_settle` — the physics
// of the material, apart from the plumbing that schedules it (workspace file-LOC cap).
