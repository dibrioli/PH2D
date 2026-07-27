//! **Impasto — the live half**: the stroke's COMMIT (ingredients → the layer) and the Body card's
//! live re-derivation. Split from the sibling [`super::impasto`] (the deposit half) for the
//! workspace file-LOC cap — same `impl PainterTool`, one responsibility per file. The rules that
//! hold the design together are documented on the deposit half's module header.

use super::Region;
use super::impasto_settle::{RELIEF_EPS, SETTLE_REACH_PX, for_each_in, settle};
use super::region::grow_region;
use crate::tool::PainterTool;
use ph2d_painter_brush::height::derive_height;
use ph2d_painter_brush::height_push::PUSH_REACH_MAX_PX;

impl PainterTool {
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
            let dst = super::plane_fork::fork_par(
                self.covers.entry(active).or_default(),
                &self.undo_window,
            );
            if dst.len() != n {
                dst.resize(n, 0);
            }
            for_each_in(rect, w, |i| {
                dst[i] = dst[i].max(film.get(i).copied().unwrap_or(0));
            });
        }
        // A janela deste acesso é EXATAMENTE o `rect` do commit (o bloco acima só escreve por
        // `for_each_in(rect, …)`), então o histórico não precisa varrer o plano para redescobri-la.
        self.declare_wrote(Some(rect));
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
            let dst = super::plane_fork::fork_par(
                self.mats.entry(active).or_default(),
                &self.undo_window,
            );
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
        self.declare_wrote(Some(rect));
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
        let target = super::plane_fork::fork_par(
            self.heights
                .entry(layer)
                .or_insert_with(|| std::sync::Arc::new(vec![0.0; n])),
            &self.undo_window,
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
        // ⚠️ O laço acima escreve `target[i]` em TODO o `rect`, e não só onde o relevo de fato mudou — é
        // por isso que a declaração do undo é o `rect` e não o `moved` que o `mark_dirty` abaixo usa. As
        // duas perguntas são diferentes (*onde a imagem mudou* × *onde bytes foram escritos*), e confundi-las
        // foi o que reprovou o atalho do doc 28 §5.17.
        self.declare_wrote(Some(rect));

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
