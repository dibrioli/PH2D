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
            let hd = HeightDab {
                center: d.center,
                radius: d.radius_px,
                coverage: d.coverage,
                footprint: fp,
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
        let stroke = std::mem::take(&mut self.paint.stroke_height);
        let Some(active) = self.layers.active() else {
            return;
        };
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
