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
//! 3. **A stroke leaves ONE thickness.** Within a stroke the deposit is an envelope (`stroke_height`,
//!    by magnitude); separate strokes ADD (committed into the layer at stroke end). Passing the brush
//!    back over its own line does not build a staircase; going over it again tomorrow does.

use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::height::{HeightDab, accumulate_dab_height, erase_dab_height};
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
        let mut field = if erasing {
            match self.heights.remove(&active) {
                Some(f) if f.len() == n => f,
                // Nothing to erase (no relief on this layer) — and a stale, differently-sized field is
                // dropped rather than indexed into (the shape guard the sweep taught us to write).
                _ => return,
            }
        } else {
            let mut f = std::mem::take(&mut self.paint.stroke_height);
            if f.len() != n {
                f = vec![0.0; n]; // lazily sized by the first dab of the stroke; zero cost when unused
            }
            f
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
                erase_dab_height(&mut field, w, h, &spec, &hd)
            } else {
                accumulate_dab_height(&mut field, w, h, &spec, &hd)
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
            self.heights.insert(active, field);
        } else {
            self.paint.stroke_height = field;
        }
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
        self.paint.last_height_center.clear(); // the sweep chain restarts with the stroke
    }

    /// Merge the finished stroke's relief into the active layer, and clear the envelope.
    ///
    /// **Add**, not envelope: within a stroke the brush leaves one thickness, but a *second* stroke
    /// over the same paint genuinely piles more on (and a carving stroke digs further). Called from
    /// `close_stroke`, BEFORE the undo entry is recorded, so the step captures the relief with the
    /// pigment that made it — one Ctrl+Z takes both.
    pub(super) fn commit_stroke_height(&mut self) {
        if self.paint.stroke_height.is_empty() {
            return;
        }
        let mut stroke = std::mem::take(&mut self.paint.stroke_height);
        let Some(active) = self.layers.active() else {
            return;
        };
        // **Smoothing** — thick paint settles. It relaxes the deposit ONCE, at stroke end (it is a
        // property of the paint, not of the light), so the artist can lay a bristly stroke and let it
        // slump like a heavy medium. Runs on the stroke's own envelope, so it softens what THIS stroke
        // laid down without touching the relief that was already on the layer.
        let smoothing = self.paint.brush.effective_impasto_smoothing();
        if smoothing > 0.0 {
            let (w, h) = self.source_size;
            settle(&mut stroke, w, h, smoothing);
        }
        let n = stroke.len();
        let field = self.heights.entry(active).or_default();
        if field.len() != n {
            field.resize(n, 0.0);
        }
        for (dst, add) in field.iter_mut().zip(stroke.iter()) {
            *dst += add;
        }
        // A layer whose relief is entirely flat carries no height at all — drop it so a layer that was
        // never sculpted (or was fully erased) costs nothing downstream and the light pass can skip it.
        if field.iter().all(|&v| v == 0.0) {
            self.heights.remove(&active);
        }
    }

    /// The relief the artist should SEE right now for `id`: the committed layer height plus the
    /// in-progress stroke's envelope. They are separate buffers (the envelope is what stops a stroke
    /// stacking on itself), so anything that displays relief has to add them — this is that one place.
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
