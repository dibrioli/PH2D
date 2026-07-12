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
//!    the layer at stroke end, up to [`H_CEIL`]). Passing the brush back over its own line does not
//!    build a staircase; going over it again tomorrow does.
//! 4. **The stroke stores its INGREDIENTS, not its height.** The relief is always
//!    `derive_height(spec, paint, grain)`, so every knob in the Body card — Depth, Body, Depth Source,
//!    Smoothing — re-derives the LAST stroke live, and none of them is a special case (Enio,
//!    2026-07-12: *"coloque todos os parâmetros vivos em tempo real para ajustes depois do traço"*).

use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::height::{
    HeightDab, HeightFields, accumulate_dab_height, derive_height, erase_dab_height,
    plow_dab_height,
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
        if !erasing && !brush.deposits_height() {
            return; // depth 0 / Draw To = Color ⇒ pigment only, no body
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
            match self.heights.remove(&active) {
                Some(f) if f.len() == n => {
                    let c = self.covers.remove(&active).filter(|c| c.len() == n);
                    erase_buffers = Some((f, c.unwrap_or_else(|| vec![0u8; n])));
                }
                // Nothing to erase (no relief on this layer) — and a stale, differently-sized field is
                // dropped rather than indexed into (the shape guard the sweep taught us to write).
                _ => return,
            }
        }
        let (mut field, mut cover) = erase_buffers.unwrap_or_default();
        let (mut paint, mut grain) = if erasing {
            (Vec::new(), Vec::new())
        } else {
            let mut h = std::mem::take(&mut self.paint.stroke_height);
            let mut p = std::mem::take(&mut self.paint.stroke_paint);
            let mut g = std::mem::take(&mut self.paint.stroke_grain);
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
            field = h;
            (p, g)
        };

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
        // The un-wrapped centre of each original dab (indexed by group), so a wrapped copy can recover
        // its own offset. With Tiling off, `groups` is empty and the entry IS its own original.
        let origin_center = |gi: usize| -> [f32; 2] {
            groups.iter().position(|&g| g as usize == gi).map_or_else(
                || dabs.get(gi).map_or([0.0, 0.0], |d| d.center),
                |first| dabs[first].center,
            )
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
            // The path predecessor, carrying THIS entry's Tiling wrap.
            let gi = groups.get(di).map_or(di, |g| *g as usize);
            let prev_center = if gi >= copies {
                let here = origin_center(gi);
                let there = origin_center(gi - copies);
                let off = [d.center[0] - here[0], d.center[1] - here[1]];
                Some([there[0] + off[0], there[1] + off[1]])
            } else {
                // First sample of this batch: chain to where the stroke was when the last batch ended,
                // per symmetry copy — without this the relief would bead at every pointer event, which is
                // a beading the artist's hardware chose, not their hand.
                self.paint
                    .last_height_center
                    .get(gi)
                    .copied()
                    .flatten()
                    .map(|prev| {
                        let here = origin_center(gi);
                        let off = [d.center[0] - here[0], d.center[1] - here[1]];
                        [prev[0] + off[0], prev[1] + off[1]]
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
            let hit = if erasing {
                erase_dab_height(&mut field, &mut cover, w, h, &spec, &hd)
            } else {
                let mut fields = HeightFields {
                    height: &mut field,
                    paint: &mut paint,
                    grain: &mut grain,
                };
                accumulate_dab_height(&mut fields, w, h, &spec, &hd)
            };
            if let Some(r) = hit {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        // Remember where each Symmetry copy ended, so the NEXT pointer batch sweeps back to it instead
        // of starting a fresh bead.
        if !dabs.is_empty() {
            self.paint.last_height_center.clear();
            self.paint.last_height_center.resize(copies, None);
            let last_group = groups.last().map_or(dabs.len() - 1, |g| *g as usize);
            for c in 0..copies {
                // The last full round of copies in this batch: group indices `last_group - (copies-1) ..= last_group`.
                let gi = last_group.saturating_sub(copies - 1 - c);
                self.paint.last_height_center[c] = Some(origin_center(gi));
            }
        }
        // The RNG copy dies here: `self.paint.tex_rng` is deliberately NOT written back (rule 2).

        if erasing {
            // The eraser scrubs the LAYER directly, so the live stroke's ground is no longer what it
            // says it is — drop it rather than let a later Depth drag resurrect erased paint.
            self.drop_live_relief();
            self.heights.insert(active, field);
            self.covers.insert(active, cover);
            self.sync_relief_flags();
        } else {
            self.paint.stroke_height = field;
            self.paint.stroke_paint = paint;
            self.paint.stroke_grain = grain;
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// **Plow** — the Smear route drags the relief along with the colour (the palette knife).
    ///
    /// Called from the smear route with the SAME dab list and the SAME `from`/`to` chain the colour
    /// uses, so pigment and body move as one thing. The knife *displaces*; it never deposits — so it
    /// needs no Depth, and there is nothing to commit: it edits the layer's relief in place, exactly as
    /// the colour smear edits the layer's pixels in place.
    ///
    /// No-op unless the layer HAS relief. That is what makes this free for everyone else: a painter who
    /// never touched Impasto smears through here and pays a map lookup.
    pub(super) fn plow_dabs(&mut self, dabs: &[Dab], brush: &BrushSpec, strength: f32) {
        if dabs.is_empty() || !brush.impasto_plow_active() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(active) = self.layers.active() else {
            return;
        };
        // Nothing sculpted here ⇒ nothing to plow. (And a stale, differently-sized field is dropped
        // rather than indexed into — the shape guard the 2026-07-12 sweep taught us to write.)
        let Some(mut field) = self.heights.remove(&active).filter(|f| f.len() == n) else {
            return;
        };
        let mut cover = self
            .covers
            .remove(&active)
            .filter(|c| c.len() == n)
            .unwrap_or_else(|| vec![0u8; n]);

        // The smear's own displacement chain — the same `last_smear_pos` the colour route advances, so
        // the two cannot drift apart. (Reading it here rather than keeping a second chain is the whole
        // reason a plowed ridge stays under its own paint.)
        let mut from = self.paint.last_smear_pos;
        let mut touched: Option<Region> = None;
        for d in dabs {
            if let Some(prev) = from {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..*brush
                };
                let amount = strength * d.coverage;
                if let Some(r) = plow_dab_height(
                    &mut field,
                    &mut cover,
                    w,
                    h,
                    &spec,
                    prev,
                    d.center,
                    d.radius_px,
                    amount,
                ) {
                    let rect = Region {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                    };
                    touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
                }
            }
            from = Some(d.center);
        }
        // The plow rewrites the committed relief, so a live stroke's ground (which is the relief BEFORE
        // that stroke) is no longer what it says it is — drop it rather than let a later Depth drag
        // resurrect the un-plowed ridge.
        self.drop_live_relief();
        self.heights.insert(active, field);
        self.covers.insert(active, cover);
        self.sync_relief_flags();
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Drop the in-progress stroke's relief. Called at pen-down, and again before each re-stamp of the
    /// shape editors' live preview (Line / Curve / Ellipse / Polygon / Free Hand re-stamp the WHOLE
    /// shape every pointer move over a restored canvas — without this the envelope would keep the
    /// relief of every shape the artist dragged THROUGH, and a curve would leave a trail of ghosts).
    pub(super) fn reset_stroke_height(&mut self) {
        self.paint.stroke_height.clear();
        self.paint.stroke_paint.clear();
        self.paint.stroke_grain.clear();
        self.paint.last_height_center.clear(); // the sweep chain restarts with the stroke
    }

    /// Merge the finished stroke into the active layer, and hand its INGREDIENTS to the live edit.
    ///
    /// **Add**, not envelope: within a stroke the brush leaves one thickness, but a *second* stroke
    /// over the same paint genuinely piles more on (and a carving stroke digs further). Called from
    /// `close_stroke`, BEFORE the undo entry is recorded, so the step captures the relief with the
    /// pigment that made it — one Ctrl+Z takes both.
    pub(super) fn commit_stroke_height(&mut self) {
        if self.paint.stroke_paint.is_empty() {
            return;
        }
        let paint = std::mem::take(&mut self.paint.stroke_paint);
        let grain = std::mem::take(&mut self.paint.stroke_grain);
        self.paint.stroke_height.clear(); // it is derived; the ingredients are the truth
        let Some(active) = self.layers.active() else {
            return;
        };
        // Coverage merges by MAX, not by sum: two strokes over the same spot do not make the pixel
        // "200% paint". (The HEIGHT does add — more paint IS thicker. The two are different quantities,
        // which is the whole reason the light needs both.)
        {
            let dst = self.covers.entry(active).or_default();
            if dst.len() != paint.len() {
                dst.resize(paint.len(), 0);
            }
            for (d, p) in dst.iter_mut().zip(paint.iter()) {
                *d = (*d).max((p.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
        }
        // Keep the stroke's INGREDIENTS and the layer's relief from BEFORE it. Between them, the whole
        // Body card re-derives this stroke after the fact — the artist lays a stroke and then dials it
        // in while looking at it, exactly like every other property in this panel.
        self.paint.live_relief_base = self.heights.get(&active).cloned().unwrap_or_default();
        self.paint.live_relief_layer = Some(active);
        self.paint.live_paint = paint;
        self.paint.live_grain = grain;
        self.rebuild_live_layer_relief();
    }

    /// Re-derive the live stroke onto the layer at the CURRENT Depth / Body / Depth Source / Smoothing.
    ///
    /// No re-stroking and no repainting: the stroke's paint and grain are stored, and the relief is a
    /// pure function of them ([`derive_height`]) — so a new setting is one pass over the buffer. This
    /// is the single point where the last stroke's relief is made, at commit and at every edit; the
    /// deposit and the edit therefore cannot drift.
    fn rebuild_live_layer_relief(&mut self) {
        let (Some(layer), false) = (
            self.paint.live_relief_layer,
            self.paint.live_paint.is_empty(),
        ) else {
            return;
        };
        let brush = &self.paint.brush;
        let mut field: Vec<f32> = self
            .paint
            .live_paint
            .iter()
            .zip(self.paint.live_grain.iter())
            .map(|(&m, &g)| derive_height(brush, m, f32::from(g) / 255.0))
            .collect();
        let smoothing = brush.effective_impasto_smoothing();
        if smoothing > 0.0 {
            let (w, h) = self.source_size;
            settle(&mut field, w, h, smoothing);
        }
        let base = &self.paint.live_relief_base;
        if base.len() == field.len() {
            for (dst, add) in field.iter_mut().zip(base.iter()) {
                // Strokes ADD — up to the glass ceiling (see [`H_CEIL`]). A lone stroke never reaches
                // it (`|depth| ≤ 1`), so the clamp only ever bites where strokes genuinely pile up.
                *dst = (*dst + add).clamp(-H_CEIL, H_CEIL);
            }
        }
        if field.iter().all(|&v| v == 0.0) {
            self.heights.remove(&layer);
        } else {
            self.heights.insert(layer, field);
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
        // The panel republishes on the layer revision, and NOTHING else bumps it here: a paint stroke
        // is a pixel edit, not a structural one. Without this, sculpting the first ridge on a layer
        // would set the flag and the panel would never hear about it — the Depth row would appear only
        // after some unrelated layer edit happened to bump the revision. (Guarded, so the hot path of a
        // stroke that changes no flag costs nothing.)
        if changed {
            self.invalidate_composite();
        }
    }

    /// A Body-card edit (Depth / Body / Depth Source / Smoothing): re-derive the last stroke in place.
    /// No-op unless that stroke is on the layer the artist is looking at — dialling Depth after
    /// switching layers must not reach back and re-sculpt a stroke on some other one.
    pub(super) fn refresh_live_relief(&mut self) {
        if !self.paint.brush.impasto || self.layers.active() != self.paint.live_relief_layer {
            return;
        }
        self.rebuild_live_layer_relief();
        self.invalidate_composite();
    }

    /// Forget the live stroke — its ground is no longer valid (an erase, an undo, a fresh document).
    pub(crate) fn drop_live_relief(&mut self) {
        self.paint.live_paint = Vec::new();
        self.paint.live_grain = Vec::new();
        self.paint.live_relief_base = Vec::new();
        self.paint.live_relief_layer = None;
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
        let live = (!self.paint.stroke_height.is_empty()
            && self.layers.active() == Some(id)
            && !self.paint.eraser)
            .then_some(&self.paint.stroke_height);
        match (committed, live) {
            (None, None) => None,
            (Some(c), None) => Some(c.clone()),
            (None, Some(l)) => Some(l.clone()),
            (Some(c), Some(l)) if c.len() == l.len() => {
                Some(c.iter().zip(l.iter()).map(|(a, b)| a + b).collect())
            }
            (Some(c), Some(_)) => Some(c.clone()),
        }
    }
}

/// The **glass ceiling** of accumulated paint, in units of a full-Depth stroke: two full loads.
///
/// Strokes ADD across each other (more paint IS thicker), but not forever — Corel Painter documents
/// the same limit for its impasto buffer: *"the accumulated artwork will begin to top out and appear
/// as if the strokes are pressed against glass"*. Unbounded stacking was the other road back to mush:
/// the walls of a 5-stroke pile dwarf every brush-mark on top of it. Symmetric, so carving bottoms
/// out at the same depth. // CLAMP-OK
pub(super) const H_CEIL: f32 = 2.0;

/// Radius, in pixels, of the settling blur at Smoothing = 1. Thick paint slumps a little, not into a
/// puddle — past a few pixels the ridges stop reading as brush-marks. // CLAMP-OK
const SETTLE_MAX_PX: f32 = 4.0;

/// Let a height field **settle** under its own weight: a separable box blur, applied in place.
///
/// Binomial-ish by repetition (two box passes ≈ a triangle kernel), which is what a viscous medium
/// relaxing actually looks like — and it is transcendental-free (HR-5) and O(n) in the radius, unlike
/// a true Gaussian. The blur is signed, so a carved groove softens exactly as a raised ridge does.
fn settle(field: &mut [f32], w: u32, h: u32, amount: f32) {
    let r = (amount.clamp(0.0, 1.0) * SETTLE_MAX_PX).round() as i64;
    if r < 1 || w == 0 || h == 0 || field.len() < (w as usize) * (h as usize) {
        return;
    }
    let (wi, hi) = (w as i64, h as i64);
    let mut tmp = vec![0.0f32; field.len()];
    let inv = 1.0 / (2 * r + 1) as f32;
    // Horizontal pass.
    for y in 0..hi {
        let row = (y * wi) as usize;
        for x in 0..wi {
            let mut sum = 0.0;
            for k in -r..=r {
                let sx = (x + k).clamp(0, wi - 1) as usize;
                sum += field[row + sx];
            }
            tmp[row + x as usize] = sum * inv;
        }
    }
    // Vertical pass.
    for y in 0..hi {
        for x in 0..wi {
            let mut sum = 0.0;
            for k in -r..=r {
                let sy = (y + k).clamp(0, hi - 1) as usize;
                sum += tmp[sy * (w as usize) + x as usize];
            }
            field[(y * wi + x) as usize] = sum * inv;
        }
    }
}
