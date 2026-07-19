//! The Sculpt **session** — when it is born, what a dab batch writes into it, how it is snapshotted,
//! restored, cancelled and re-stamped, and what it says it displaced.
//!
//! Split from [`super::sculpt`] (the model: the verbs, the knobs, the routing) for the workspace file-LOC
//! cap, and along a seam that was already there: that module answers *what the artist chose*, this one
//! answers *what the stroke is doing about it*. The kernel that turns the two into relief lives in the
//! third sibling, [`super::sculpt_blur`].

use super::Region;
use super::sculpt::SculptFamily;
use crate::tool::PainterTool;
use std::sync::Arc;

/// The stroke's direction AT a dab, from the path itself rather than from the smoothed tangent.
///
/// Dab centres are exact samples of the path, so a backward difference between two of them trails by half
/// a dab spacing — about a pixel. [`ph2d_painter_brush::Dab::dir`] is a *smoothed* heading (the Rake
/// texture rides it and wants the smoothing), and it trails by roughly the brush RADIUS, which is why the
/// chisel's groove drifted further off the harder the artist leaned on brush size.
///
/// Falls back to the smoothed heading when there is no previous centre (the stroke's first dab) or when
/// the dab has not moved (a tap, a held brush) — in both cases the difference carries no direction and
/// inventing one would make the mark depend on float noise.
fn path_axis(prev: Option<[f32; 2]>, center: [f32; 2], smoothed: [f32; 2]) -> [f32; 2] {
    let Some(p) = prev else { return smoothed };
    let (dx, dy) = (center[0] - p[0], center[1] - p[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-4 {
        return smoothed;
    }
    [dx / len, dy / len]
}

impl PainterTool {
    // ── Session lifecycle ───────────────────────────────────────────────────────────────────────

    /// Open a session if none is live: freeze the layer's relief as `pre` and allocate the accumulator.
    ///
    /// Returns `false` when there is nothing to sculpt — **no layer, or a layer with no relief on it**.
    /// That is not an error path, it is the tool being honest (§5): a sculpt brush creates no paint, so a
    /// document nobody has sculpted pays not one byte for the fact that the mode exists.
    pub(super) fn ensure_sculpt_session(&mut self) -> bool {
        if self.paint.sculpt.layer.is_some() {
            self.ensure_family_target(); // a family switch mid-shape needs the OTHER target sized
            return true; // …but the gesture keeps ITS frozen source
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(active) = self.layers.active() else {
            return false;
        };
        // The layer's own plane, by `Arc` — zero allocation, and it stays frozen because the first write
        // to `heights` copies it out from under us.
        let Some(pre) = self.heights.get(&active).filter(|f| f.len() == n).cloned() else {
            return false; // no relief here: nothing to smooth, nothing to sharpen
        };
        self.paint.sculpt.pre = pre;
        // The MATTER, frozen beside the relief — three more `Arc` clones, still zero allocation. Only
        // Inflate reads them, but they are taken for every verb: the artist can switch the verb mid-shape,
        // and a session that froze `pre` at dab 1 and the paint at dab 40 would re-render the gesture
        // against a canvas that never existed. Freeze the whole state, once, at the start. (§10.4 — *ao
        // adicionar um plano, adicione-o ao snapshot no mesmo commit* — is the scar this obeys.)
        self.paint.sculpt.pre_cover = self.covers.get(&active).cloned().unwrap_or_default();
        self.paint.sculpt.pre_mats = self.mats.get(&active).cloned().unwrap_or_default();
        self.paint.sculpt.pre_rgba = Arc::clone(&self.canvas_rgba);
        self.paint.sculpt.amount = Arc::new(vec![0.0; n]);
        self.paint.sculpt.layer = Some(active);
        self.paint.sculpt.bbox = None;
        self.ensure_family_target();
        true
    }

    /// Put the gesture's window back to the state the stroke found it in — **all four planes**.
    ///
    /// THE one restore. Both callers (the knob re-render in [`PainterTool::refresh_live_sculpt`] and the
    /// shape editors' re-stamp in [`Self::restamp_reset_sculpt`]) go through here, because the render only
    /// ever WRITES texels the brush touched: a knob that shrinks the effect would otherwise leave the
    /// stronger version standing wherever the new one writes nothing.
    ///
    /// It restores the matter as well as the relief, and unconditionally — not only when the verb moves
    /// matter. Switching Inflate → Smooth mid-shape has to give the paint back; a restore that asked the
    /// CURRENT verb what to undo would ask the wrong verb.
    pub(super) fn restore_sculpt_window(&mut self, layer: crate::tool::RtLayerId, rect: Region) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let pre = Arc::clone(&self.paint.sculpt.pre);
        if pre.len() != n {
            return; // a session that no longer describes this canvas is not a session
        }
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::plane_fork::fork_par(entry);
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = pre[i]);
        }
        let cover = Arc::clone(&self.paint.sculpt.pre_cover);
        if cover.len() == n
            && let Some(entry) = self.covers.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::plane_fork::fork_par(entry);
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = cover[i]);
        }
        let mats = Arc::clone(&self.paint.sculpt.pre_mats);
        if mats.len() == n
            && let Some(entry) = self.mats.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::plane_fork::fork_par(entry);
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = mats[i]);
        }
        let rgba = Arc::clone(&self.paint.sculpt.pre_rgba);
        if rgba.len() == n * 4 && self.canvas_rgba.len() == n * 4 {
            let dst = super::plane_fork::fork_par(&mut self.canvas_rgba);
            super::impasto_settle::for_each_in(rect, w, |i| {
                dst[i * 4..i * 4 + 4].copy_from_slice(&rgba[i * 4..i * 4 + 4]);
            });
        }
    }

    /// Size the per-texel target the ACTIVE family needs, and only that one.
    ///
    /// The families are mutually exclusive by construction (a verb belongs to exactly one), so the session
    /// carries ONE 4 B/px target — which is what keeps eight verbs at the same **12 B/px** the two of Wave 1
    /// cost. The **Height** family carries none at all: its target is a function of `pre` and a knob,
    /// evaluated per texel in the render, so Layer and Inflate run at **8 B/px**.
    ///
    /// Idempotent: a correctly-sized buffer is left alone, so this is free on the hot path.
    fn ensure_family_target(&mut self) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        match self.paint.sculpt.mode_enum().family() {
            SculptFamily::Memo => {
                if self.paint.sculpt.memo.len() != n {
                    self.paint.sculpt.memo = vec![0.0; n];
                    self.paint.sculpt.memo_done = vec![false; super::sculpt_blur::tile_count(w, h)];
                    self.paint.sculpt.memo_key = self.paint.sculpt.memo_key();
                }
            }
            SculptFamily::Plane => {
                if self.paint.sculpt.plane_sum.len() != n {
                    self.paint.sculpt.plane_sum = Arc::new(vec![0.0; n]);
                }
            }
            SculptFamily::Height => {} // no target buffer exists to size
        }
    }

    /// Deposit one dab batch's sculpt intensity and re-render the relief it reshapes.
    ///
    /// Called from the ONE choke point in the route dispatcher, with the list already mirrored (Symmetry)
    /// and already replicated (Tiling) — the same list the colour would have consumed, which is the whole
    /// design (§10.1).
    pub(super) fn stamp_dabs_sculpt(
        &mut self,
        dabs: &[ph2d_painter_brush::Dab],
        brush: &ph2d_painter_brush::BrushSpec,
    ) {
        if dabs.is_empty() || !self.ensure_sculpt_session() {
            return;
        }
        let (w, h) = self.source_size;
        // The Selection, handed to the kernel so it attenuates each dab AS IT LANDS. Multiplying the
        // accumulated `amount` afterwards is the obvious place and it is wrong: `amount` carries every
        // earlier batch, so a Feathered edge would be scaled once per pointer event (see
        // `ph2d_painter_brush::sculpt`). Nothing selected ⇒ `None` ⇒ the spatula reaches everywhere.
        let mask: Option<Arc<Vec<u8>>> = self
            .selection_restricts_paint()
            .then(|| Arc::clone(&self.paint.selection_mask));
        // Resolve each dab's frames exactly as the colour route would — same Shape basis, same Grain
        // frame, same order. The RNG is a COPY: this pass must not advance the stream (the deposit's rule
        // 2, and it holds here for the same reason — a sculpt stroke can carry a grain).
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        let grain_active = brush.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);

        let mut amount = std::mem::take(Arc::make_mut(&mut self.paint.sculpt.amount));
        // The plane family's second output + the frozen surface it fits to, lifted out of `self` for the
        // loop. Empty in the Smooth family, and the kernel is never handed them there — so a Smooth stroke
        // is byte-for-byte the one Wave 1 shipped, and its gates keep meaning what they meant.
        let plane_family = self.paint.sculpt.mode_enum().family() == SculptFamily::Plane;
        let mut plane_sum = std::mem::take(Arc::make_mut(&mut self.paint.sculpt.plane_sum));
        let mut scratch = std::mem::take(&mut self.paint.sculpt.fit_scratch);
        // The stroke's own direction, carried across batches — the Chisel's V axis
        // (see `SculptState::last_dab_center`).
        let mut prev_center = self.paint.sculpt.last_dab_center;
        let pre = Arc::clone(&self.paint.sculpt.pre);
        // **Rake OFF: the knife enters at the angle the stroke started in, and holds it.** Lock on the first
        // dab of the gesture that HAS a heading — not on the pen-down dab, whose heading is `[0, 0]` until
        // the hand has travelled (the warm-up holds those dabs precisely so this is never `[0, 0]`, but the
        // `find` says so out loud rather than trusting it).
        //
        // Once set it never moves again this gesture, which is the whole feature: drag a curve and the crease
        // keeps pointing the way you set off. It is cleared in `restamp_reset_sculpt`, so a shape editor
        // re-aims when the artist re-draws the shape.
        if !self.paint.sculpt.rake && self.paint.sculpt.locked_dir.is_none() {
            self.paint.sculpt.locked_dir = dabs.iter().map(|d| d.dir).find(|d| *d != [0.0, 0.0]);
        }
        // The Chisel's tilt — `tan(angle)`, and exactly `0` in every other verb, so the `+ tilt·|lateral|`
        // term vanishes and Flatten / Scrape / Fill stay byte-for-byte the Wave 2 kernel. Computed ONCE, here.
        let chisel_tilt = self.paint.sculpt.chisel_tilt();
        let mut touched: Option<Region> = None;
        for (di, d) in dabs.iter().enumerate() {
            let tex_rng = dab_rng.enter(&groups, di);
            let spec = ph2d_painter_brush::BrushSpec {
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
            let hd = ph2d_painter_brush::height::HeightDab {
                center: d.center,
                radius: d.radius_px,
                coverage: d.coverage,
                footprint: fp,
                // No sweep: a sculpt dab marks where it IS. (The deposit sweeps back to the previous dab
                // so the bodies join into one ridge instead of a string of beads; an intensity field has
                // no such seam to hide — it already sums, so consecutive dabs blend by construction.)
                prev_center: None,
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
            let hit = if plane_family {
                // The plane is fitted to `pre`, the FROZEN surface — never to the live one. Fitting to the
                // live relief would let each dab flatten what the previous dab already flattened, and the
                // result would depend on the spacing and on how fast the artist moved
                // (`a_faster_mouse_does_not_sculpt_deeper`, and it now runs for every verb).
                ph2d_painter_brush::sculpt::accumulate_dab_plane(
                    ph2d_painter_brush::sculpt::PlaneOut {
                        amount: &mut amount,
                        plane_sum: &mut plane_sum,
                        pre: &pre,
                        scratch: &mut scratch,
                    },
                    // The V's axis — the stroke's direction while **Rake** is on (so the groove curves
                    // with the hand), or the heading it entered at while it is off. One function answers
                    // it, so the checkbox and the kernel cannot disagree about what "the direction of the
                    // stroke" means.
                    //
                    // ⚠️ **From the dab CENTRES, not from `Dab::dir`.** `dir` is the *smoothed* tangent the
                    // Rake texture rides, and a smoothed tangent TRAILS: measured on a quarter circle, the
                    // groove came out **9° behind at radius 8 and up to 52° behind at radius 60** — the
                    // lag grows with the brush, because the smoothing window does. Half a right angle off
                    // does not read as "the knife lags"; it reads as *"Rake não consegue rotacionar o
                    // brush"* (Enio, 2026-07-18), which is how it was reported.
                    //
                    // The dab centres are exact samples of the path, so the difference between two of them
                    // trails by half a dab SPACING (~1 px) instead of ~1 radius. `prev_center` is already
                    // maintained on the state (`last_dab_center`), so it survives batch boundaries and the
                    // axis is continuous across pointer events — which a per-batch estimate would not be.
                    //
                    // `Dab::dir` remains the fallback, and it is the RIGHT one: on the first dab of a
                    // stroke there is no previous centre, and the heading warm-up exists precisely so that
                    // dab already carries a settled direction.
                    ph2d_painter_brush::sculpt::Chisel {
                        tilt: chisel_tilt,
                        dir: self.chisel_dir(path_axis(prev_center, d.center, d.dir)),
                    },
                    mask.as_ref().map(|m| m.as_slice()),
                    w,
                    h,
                    &spec,
                    &hd,
                )
            } else {
                ph2d_painter_brush::sculpt::accumulate_dab_sculpt(
                    &mut amount,
                    mask.as_ref().map(|m| m.as_slice()),
                    w,
                    h,
                    &spec,
                    &hd,
                )
            };
            if let Some(r) = hit {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| super::union_region(acc, rect)));
            }
            prev_center = Some(d.center);
        }
        *Arc::make_mut(&mut self.paint.sculpt.amount) = amount;
        *Arc::make_mut(&mut self.paint.sculpt.plane_sum) = plane_sum;
        self.paint.sculpt.fit_scratch = scratch; // hand the buffer back so the next batch reuses it
        self.paint.sculpt.last_dab_center = prev_center;
        if let Some(rect) = touched {
            self.paint.sculpt.bbox = Some(
                self.paint
                    .sculpt
                    .bbox
                    .map_or(rect, |acc| super::union_region(acc, rect)),
            );
            self.render_sculpt(rect);
        }
        // The RNG copy dies here: `self.paint.tex_rng` is deliberately NOT written back.
    }

    /// Snapshot the sculpt session for the undo model (the buffers are `tool::paint`-private, so the
    /// general `snapshot_model` reaches them through here — mirror of `deform_for_snapshot`).
    ///
    /// `Arc`s, so this is two refcount bumps and a `Copy`. See [`crate::undo::SculptSnap`] for *why* the
    /// session has to ride the snapshot at all — the short version is that the sculpt writes the relief
    /// LIVE, so a snapshot without the session restores a carved plane and calls it pristine.
    /// `plane_sum` rides it too, and it MUST: unlike the blur memo it is derived from the DAB LIST, not
    /// from `pre`, so the restore cannot rebuild it. Doc 18 §10.4 states this as a rule with a scar behind
    /// it — *"ao adicionar um plano, adicione-o ao snapshot no mesmo commit"* — because `mats` was left out
    /// when the material landed and the hole hid until paint went over paint.
    pub(crate) fn sculpt_for_snapshot(&self) -> crate::undo::SculptSnap {
        crate::undo::SculptSnap {
            pre: Arc::clone(&self.paint.sculpt.pre),
            amount: Arc::clone(&self.paint.sculpt.amount),
            plane_sum: Arc::clone(&self.paint.sculpt.plane_sum),
            // The frozen MATTER rides too, and it must: Inflate re-renders the paint from it, so a restore
            // that put back the relief and not the source of the pixels would re-derive the colour from a
            // canvas the gesture has already written on — compounding, every undo. §10.4.
            pre_cover: Arc::clone(&self.paint.sculpt.pre_cover),
            pre_mats: Arc::clone(&self.paint.sculpt.pre_mats),
            pre_rgba: Arc::clone(&self.paint.sculpt.pre_rgba),
            layer: self.paint.sculpt.layer,
            bbox: self.paint.sculpt.bbox,
        }
    }

    /// Reinstate the sculpt session from an undo model, in lock-step with the relief it describes.
    ///
    /// The blur memo is NOT restored — it is pure derived data (a function of `pre` alone), and rebuilding
    /// it lazily is what keeps the snapshot three refcount bumps instead of another canvas-sized copy.
    /// `plane_sum` is NOT derived data and IS restored: nothing but the dabs can reproduce it.
    pub(crate) fn restore_sculpt(&mut self, s: crate::undo::SculptSnap) {
        self.paint.sculpt.pre = s.pre;
        self.paint.sculpt.amount = s.amount;
        self.paint.sculpt.plane_sum = s.plane_sum;
        self.paint.sculpt.pre_cover = s.pre_cover;
        self.paint.sculpt.pre_mats = s.pre_mats;
        self.paint.sculpt.pre_rgba = s.pre_rgba;
        self.paint.sculpt.layer = s.layer;
        self.paint.sculpt.bbox = s.bbox;
        self.paint.sculpt.memo = Vec::new();
        self.paint.sculpt.memo_done = Vec::new();
        self.paint.sculpt.memo_key = super::sculpt::MemoKey::None;
    }

    /// **The one exit.** Drop the session WITHOUT restoring anything — the relief in `heights` is kept.
    ///
    /// This is what a *committed* gesture does: pen-up for the freehand methods, **Apply** for the shape
    /// editors. The carving IS the layer's relief now, so there is nothing to put back and nothing left to
    /// re-render — the Sculpt card's knobs arm the NEXT stroke, they do not reach back and rewrite the last
    /// one (see the [`SculptState`] docs for why the deposit's "Adjust Last Stroke" does not transfer here).
    ///
    /// It is also the abandon path — mode switch, document rebind, deactivate — where the relief is going
    /// away with the document anyway. When the carving must be UN-done rather than kept, that is
    /// [`Self::cancel_sculpt_session`].
    pub(crate) fn end_sculpt_session(&mut self) {
        self.paint.sculpt.pre = Arc::new(Vec::new());
        self.paint.sculpt.amount = Arc::new(Vec::new());
        self.paint.sculpt.plane_sum = Arc::new(Vec::new());
        self.paint.sculpt.last_dab_center = None;
        self.paint.sculpt.fit_scratch = Vec::new();
        self.paint.sculpt.memo = Vec::new();
        self.paint.sculpt.memo_done = Vec::new();
        self.paint.sculpt.pre_cover = Arc::new(Vec::new());
        self.paint.sculpt.pre_mats = Arc::new(Vec::new());
        self.paint.sculpt.pre_rgba = Arc::new(Vec::new());
        self.paint.sculpt.locked_dir = None; // the knife's angle belongs to the gesture, and it is over
        self.paint.sculpt.layer = None;
        self.paint.sculpt.bbox = None;
    }

    /// The **volume this gesture has displaced**, in paint-loads × texels: `Σ (h − pre)` over the window it
    /// touched. Negative when the spatula removed material (Scrape), positive when it added (Fill).
    ///
    /// ## Why it is here now, when nothing uses it yet
    ///
    /// Doc 18 §6 leaves a bifurcation deliberately open — *where does the scraped paint go?* Blender simply
    /// **deletes** matter: Scrape removes it, Draw creates it from nowhere. **Paint does not do that**, and
    /// we already own a conservative displacement engine (`height_push` sums to exactly zero). So the
    /// eventual answer is a *bow wave*: the volume the spatula takes off piles up at its edge.
    ///
    /// The scope decision was: ship the **non-conservative** Scrape (Blender's), *but have the kernel
    /// compute the displaced volume and throw it away explicitly* — so conservation would be a **flag**, not a
    /// rewrite. This is that computation, and gating it now (`the_scrape_reports_the_volume_it_removed`) is
    /// what makes it a fact rather than a promise: the number is checked against the relief that actually
    /// moved, today, while the code that moved it is still under my hand.
    ///
    /// Computed on demand rather than accumulated: `h` is a pure function of `(pre, amount, plane_sum)` and
    /// the render re-derives the whole window from `pre` every time, so a running total would double-count
    /// on the very first shape-editor re-stamp. O(window), and nothing on the hot path calls it.
    #[must_use]
    pub fn sculpt_displaced_volume(&self) -> f64 {
        let (Some(layer), Some(rect)) = (self.paint.sculpt.layer, self.paint.sculpt.bbox) else {
            return 0.0;
        };
        let (w, _) = self.source_size;
        let pre = &self.paint.sculpt.pre;
        let Some(h) = self.heights.get(&layer).filter(|h| h.len() == pre.len()) else {
            return 0.0;
        };
        let mut sum = 0.0f64;
        super::impasto_settle::for_each_in(rect, w, |i| {
            if let (Some(a), Some(b)) = (h.get(i), pre.get(i)) {
                sum += f64::from(*a - *b);
            }
        });
        sum
    }

    /// **Cancel** (Esc / Delete on an open shape): put the relief back the way the gesture found it, THEN
    /// drop the session.
    ///
    /// `cancel_open_shape` peels the pixels back to pristine, and its own comment states the invariant this
    /// closes: *"the pixels are peeled back to pristine — so the relief has to go with them"*. The deposit
    /// obeys it (`reset_stroke_height` + `drop_live_relief`); the sculpt could not, because it does not
    /// stage its relief in an envelope — it rewrites the layer's plane LIVE. So a cancelled Curve left its
    /// carving behind: the shape gone, the smoothing permanent, and **no undo entry**, because the shape was
    /// never committed. Third door of the same house as the Apply and the rebind (`sculpt_tests::session`).
    pub(super) fn cancel_sculpt_session(&mut self) {
        self.restamp_reset_sculpt(); // heights[bbox] = pre — the same restore a re-stamp does
        self.end_sculpt_session();
    }

    /// A shape editor is about to re-stamp the WHOLE stroke: put the relief back the way this stroke found
    /// it and zero what it had accumulated.
    ///
    /// Without this, a Curve would carve deeper on every pointer move while the artist merely LOOKED at
    /// it — and every frame of the drag would leave its ghost in the relief. It is the same restore the
    /// pixels get (`stamp_drag_preview`), one channel over. `pre` and the blur memo SURVIVE: the restore
    /// makes `heights` byte-identical to `pre` again, so the frozen source is still the truth and the
    /// memo of its blur is still valid.
    pub(super) fn restamp_reset_sculpt(&mut self) {
        let (Some(layer), Some(rect)) = (self.paint.sculpt.layer, self.paint.sculpt.bbox) else {
            return;
        };
        let (w, _) = self.source_size;
        // Inflate fattens beyond the touched texels — restore the widest the Blob can reach so a shape-editor
        // re-stamp leaves no ghost ring (mirror of `refresh_live_sculpt`). The `amount` zeroing below stays on
        // `rect`: coverage only ever accrues where the brush actually touched.
        let restore_rect = if self.paint.sculpt.mode_enum() == super::sculpt::SculptMode::Inflate {
            let (rw, rh) = self.source_size;
            let pad = 2
                * (super::sculpt::SCULPT_DEPTH_MAX * super::impasto_light::DEPTH_UNIT_PX).ceil()
                    as u32
                + super::sculpt::INFLATE_SMOOTH_MAX_PX as u32;
            super::region::grow_region(rect, pad, rw, rh).unwrap_or(rect)
        } else {
            rect
        };
        self.restore_sculpt_window(layer, restore_rect);
        // …and the knife's locked angle with it. The lock is a fact about the dab list, exactly like
        // `plane_sum` — and a shape editor rebuilds that list from scratch every frame. Keep it and dragging
        // a Line's endpoint round would re-cut the shape with the angle of a shape the artist has since
        // redrawn: the knife would remember a stroke that no longer exists.
        self.paint.sculpt.locked_dir = None;
        let amount = Arc::make_mut(&mut self.paint.sculpt.amount);
        super::impasto_settle::for_each_in(rect, w, |i| {
            if let Some(a) = amount.get_mut(i) {
                *a = 0.0; // a shape guard, not a formality: a rebind can leave a session sized for the
            } // OLD canvas while `rect` is measured against the new one
        });
        // …and the plane target with it. It is the DENOMINATOR's partner: leaving `plane_sum` behind while
        // zeroing `amount` would make the next re-stamp's `plane_sum / amount` a ratio of this frame's
        // planes over the last frame's weights — a target the artist never asked for anywhere the two
        // stamps disagree, which is every texel along the edge of a shape being dragged. Empty in the
        // Smooth family, where this is a no-op.
        let plane_sum = Arc::make_mut(&mut self.paint.sculpt.plane_sum);
        super::impasto_settle::for_each_in(rect, w, |i| {
            if let Some(p) = plane_sum.get_mut(i) {
                *p = 0.0;
            }
        });
        self.paint.sculpt.last_dab_center = None;
        self.paint.sculpt.bbox = None;
        if let Some(r) = super::region::grow_region(rect, 1, w, self.source_size.1) {
            self.mark_dirty(r);
        }
    }
}
