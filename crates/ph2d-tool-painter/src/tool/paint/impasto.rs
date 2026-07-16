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

use super::impasto_settle::{owned, union_dirty};
use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::height::{
    HeightDab, HeightFields, accumulate_dab_height, derive_height, erase_dab_height,
};
use ph2d_painter_brush::height_push::{DEPOSIT_FORWARD_SHARE, PushBite, bank_dab_push, wave_lobe};
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
}

// The stroke's COMMIT + the live Body-card re-derivation moved to the sibling `impasto_live`
// (workspace file-LOC cap): same `impl PainterTool`, split by responsibility — deposit here,
// re-derive there.

// The deposit's SETTLE (the box blur that lets paint relax under its own weight), its reach, and the
// sub-visible threshold the dirty-rect diff uses, all live in the sibling `impasto_settle` — the physics
// of the material, apart from the plumbing that schedules it (workspace file-LOC cap).
