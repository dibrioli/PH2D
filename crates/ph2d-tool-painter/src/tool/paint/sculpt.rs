//! **Sculpt** — reshaping the relief the paint already laid (`docs/Painter/18_plano_sculpt_relevo.md`).
//!
//! Wave 1: **Smooth** (pull the relief toward its own local average) and **Sharpen** (the same kernel run
//! backwards). One engine, one sign.
//!
//! ## The two things that make this module unlike Deform
//!
//! **It has no geometry of its own.** Deform is mode-exclusive with its own dab lifecycle; Sculpt hangs
//! off the ONE choke point the colour already goes through (`stamp_dabs_inner`), so Symmetry, Tiling, the
//! shape editors, pressure, Jitter, falloff, Shape and Grain reach it without a line of code each — and
//! keep reaching it when someone changes them (§10.1). A pass with geometry of its own is how
//! *"Tiling doesn't work in Sculpt"* gets born six months from now. The consequence the reader should
//! expect: **the brush's own Size / Strength / Spacing / Falloff / Shape / Grain ARE the sculpt's
//! controls**, and the Sculpt card only adds what the brush cannot already say — which verb, and over
//! what scale.
//!
//! **It is applied once per stroke, not once per dab.** The dab does not blur `h` in place; it
//! accumulates an intensity ([`ph2d_painter_brush::sculpt::accumulate_dab_sculpt`]) and the relief is
//! re-rendered from a FROZEN copy taken at the start of the stroke. See that module for why the obvious
//! version is wrong twice (it is not idempotent under the shape editors' re-stamp, and composing N dab
//! blurs turns the smoothing scale into a function of the SPACING slider). The kernel + the memo that
//! makes it affordable live in the sibling [`super::sculpt_blur`].

use super::Region;
use crate::tool::PainterTool;
use std::sync::Arc;

/// The number of sub-modes — the segmented control's option count; [`SculptMode::from_u8`] clamps to it.
/// Waves 2-3 append (Scrape / Fill / Flatten / Clay / Layer / Inflate …); the order never shifts.
pub(crate) const SCULPT_MODE_COUNT: u8 = 2;

/// The kernel's radius at Radius = 1, in pixels.
///
/// Small on purpose, and the cap is a design statement rather than a performance excuse: smoothing at a
/// LARGE scale is not a bigger blur, it is **Flatten** — a least-squares plane fit over the footprint
/// (§7), which is a different kernel and lands in Wave 2. A 100-px "smooth" would be a slow, mushy,
/// worse Flatten. Sixteen pixels is the scale brush-marks and grain actually live at. // CLAMP-OK
pub(super) const SCULPT_RADIUS_MAX_PX: f32 = 16.0;

/// Which verb the sculpt brush performs. Discriminants are the segmented control's indices.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum SculptMode {
    /// Pull the relief toward its own local average — `lerp(pre, blur(pre), k)`.
    Smooth,
    /// Push it away from that average — `pre + k·(pre − blur(pre))`, an unsharp mask on a height field.
    /// Bounded by construction (`k ≤ 1` ⇒ at most twice the local detail), so it cannot ring away.
    Sharpen,
}

impl SculptMode {
    pub(super) fn from_u8(v: u8) -> Self {
        match v {
            1 => SculptMode::Sharpen,
            _ => SculptMode::Smooth,
        }
    }
}

/// Sculpt settings + the per-stroke session (`docs/Painter/18…` §4).
///
/// The session is born at the first sculpt dab and **dies the moment that gesture is committed**: at
/// pen-up for the freehand methods, at **Apply** for the shape editors (which deliberately keep the stroke
/// open until then), and on Cancel, on a mode switch, and on a document rebind. On undo it is not dropped
/// but RESTORED, in lock-step with the relief ([`crate::undo::SculptSnap`]).
///
/// ## The session does NOT outlive the stroke — and that is a correction, not an omission
///
/// It used to. The session was *parked* at pen-up so that Radius and Smooth↔Sharpen would re-render the
/// stroke the artist had just made, riding the Body card's **"Adjust Last Stroke"** checkbox. Enio's smoke
/// killed it in one sentence: reaching for **Sharpen** — to sharpen somewhere *else* — silently converted
/// the Smooth he had just made into its opposite.
///
/// The mistake was reasoning by analogy from the deposit without asking whether the analogy holds:
///
/// * The **deposit is a substance**. Depth / Body / Source are properties *of the paint that stroke laid*,
///   so "let me keep tuning them" is a coherent offer, and the checkbox makes it.
/// * A **sculpt stroke is an operation**. It leaves nothing behind that has properties — only the relief,
///   as it now is. There is no "the smoothing" sitting there to re-parameterise. Operations are undone and
///   re-done; they are not re-dialled. (Photoshop's Blur tool has no radius-after-the-fact either, and for
///   this reason.)
/// * And the **Mode is not a parameter at all** — it is *which tool*. A verb that rewrites history when you
///   select it is not an adjustment, it is a destruction the artist did not ask for.
///
/// Once the verb cannot be retro-active, a retro-active Radius is *worse* than either rule applied
/// uniformly: two knobs side by side on one card, one rewriting the past and one not, and nothing on
/// screen to say which is which.
///
/// **What stays live is what is still being authored.** A shape editor's stroke is a PREVIEW — it has an
/// Apply button precisely because it is not committed — so while a shape is open the knobs do re-render it
/// ([`Self::refresh_live_sculpt`] gates on `open`). That is not an exception to the rule; it is the rule:
/// *the session lives exactly as long as the gesture is uncommitted.*
pub(crate) struct SculptState {
    /// Sub-mode (`0` Smooth · `1` Sharpen).
    pub(crate) mode: u8,
    /// Radius slider track (`0..1`; mapped to px by [`Self::radius_px`]).
    pub(crate) radius_norm: f32,

    // ── Session (per stroke; see the struct docs) ────────────────────────────────────────────────
    /// The layer's relief as this stroke found it — the frozen source every re-render reads.
    ///
    /// An `Arc` clone of the layer's own plane, so opening a session allocates **nothing**: the first
    /// write to `heights` copies-on-write and leaves this pointing at the original. A stroke that
    /// changes nothing (Strength 0) therefore never forks it, and costs literally zero bytes.
    pub(crate) pre: Arc<Vec<f32>>,
    /// Accumulated per-texel intensity (canvas-sized). Dabs SUM into it; the render clamps the total to 1.
    ///
    /// `Arc`-shared so the undo snapshot captures it with a refcount bump (the first write after a capture
    /// copies-on-write) — the same trick `deform.disp` uses, and for the same reason: every shape-editor
    /// gesture takes a snapshot, and a canvas-sized `Vec` cloned per gesture is a 64 MB copy at 4096².
    pub(crate) amount: Arc<Vec<f32>>,
    /// `blur(pre, radius)`, memoised tile by tile (see [`super::sculpt_blur`]). Pure derived data: it dies
    /// with the session and is rebuilt lazily, tile by tile, on demand.
    pub(crate) blurred: Vec<f32>,
    /// One flag per tile of [`Self::blurred`] — whether that tile has been computed this session.
    pub(crate) blur_done: Vec<bool>,
    /// The radius [`Self::blurred`] holds. A change invalidates every tile.
    pub(crate) blur_radius: u32,
    /// Which layer the session belongs to — and, because the session dies at commit, **this is also what
    /// "a gesture is in flight" means**. `None` ⇒ no session; every guard in this module reads it as both.
    ///
    /// There used to be a separate `open: bool` beside it. Once the session stopped outliving the stroke
    /// the two became coextensive, and a redundant flag is one that eventually disagrees with the field it
    /// shadows — so the target IS the guard.
    pub(crate) layer: Option<crate::tool::RtLayerId>,
    /// The union of this stroke's dab footprints — the window the re-render owns, and the window a
    /// shape editor's re-stamp restores.
    pub(crate) bbox: Option<Region>,
}

impl Default for SculptState {
    fn default() -> Self {
        Self {
            mode: 0,          // Smooth — the one Enio asked for first, and the most valuable of the list
            radius_norm: 0.5, // 8 px: brush-mark scale
            pre: Arc::new(Vec::new()),
            amount: Arc::new(Vec::new()),
            blurred: Vec::new(),
            blur_done: Vec::new(),
            blur_radius: 0,
            layer: None,
            bbox: None,
        }
    }
}

impl SculptState {
    /// The kernel radius in px (`1..=SCULPT_RADIUS_MAX_PX`). Never 0 — a zero-radius blur is a no-op, and
    /// a control whose bottom end silently does nothing is a control that lies.
    pub(super) fn radius_px(&self) -> u32 {
        let t = self.radius_norm.clamp(0.0, 1.0);
        (1.0 + t * (SCULPT_RADIUS_MAX_PX - 1.0)).round() as u32
    }
}

impl PainterTool {
    // ── UI-edit setters (the single clamp source; `handle_panel_event` forwards raw panel values here) ──

    /// Select the sub-mode from the segmented index (out of range clamps to the last mode).
    pub fn set_sculpt_mode(&mut self, m: u8) {
        self.paint.sculpt.mode = m.min(SCULPT_MODE_COUNT - 1);
        self.refresh_live_sculpt();
    }

    /// Set the kernel Radius from the slider's `0..1` track.
    pub fn set_sculpt_radius(&mut self, t: f32) {
        self.paint.sculpt.radius_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Whether the active operation is **Sculpt** — the panel shows the Sculpt card (and hides the colour
    /// controls, which a tool that lays no pigment has no use for).
    #[must_use]
    pub fn is_sculpt_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Sculpt)
    }

    /// The kernel radius in px — what the Radius chip shows the artist, and what the kernel actually uses.
    /// One function, so the number on screen cannot drift from the number in the blur.
    #[must_use]
    pub fn sculpt_radius_px(&self) -> u32 {
        self.paint.sculpt.radius_px()
    }

    /// Route a Sculpt-panel event to its setter. Segmented mode = `Click` on an option id (its array
    /// position is the mode); sliders = `SetValue`. Returns `true` iff consumed. Mirrors
    /// `route_deform_event`; hung off the `handle_panel_event` chain.
    pub(crate) fn route_sculpt_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if core_ids::PAINTER_SCULPT_MODE_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SCULPT_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_sculpt_mode(idx);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_RADIUS_SLIDER => {
                self.set_sculpt_radius(*v as f32);
                true
            }
            _ => false,
        }
    }

    // ── Session lifecycle ───────────────────────────────────────────────────────────────────────

    /// Open a session if none is live: freeze the layer's relief as `pre` and allocate the accumulator.
    ///
    /// Returns `false` when there is nothing to sculpt — **no layer, or a layer with no relief on it**.
    /// That is not an error path, it is the tool being honest (§5): a sculpt brush creates no paint, so a
    /// document nobody has sculpted pays not one byte for the fact that the mode exists.
    pub(super) fn ensure_sculpt_session(&mut self) -> bool {
        if self.paint.sculpt.layer.is_some() {
            return true; // a gesture is already in flight — keep ITS frozen source
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
        self.paint.sculpt.amount = Arc::new(vec![0.0; n]);
        self.paint.sculpt.blurred = vec![0.0; n];
        self.paint.sculpt.blur_done = vec![false; super::sculpt_blur::tile_count(w, h)];
        self.paint.sculpt.blur_radius = self.paint.sculpt.radius_px();
        self.paint.sculpt.layer = Some(active);
        self.paint.sculpt.bbox = None;
        true
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
            if let Some(r) = ph2d_painter_brush::sculpt::accumulate_dab_sculpt(
                &mut amount,
                mask.as_ref().map(|m| m.as_slice()),
                w,
                h,
                &spec,
                &hd,
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| super::union_region(acc, rect)));
            }
        }
        *Arc::make_mut(&mut self.paint.sculpt.amount) = amount;
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
    pub(crate) fn sculpt_for_snapshot(&self) -> crate::undo::SculptSnap {
        crate::undo::SculptSnap {
            pre: Arc::clone(&self.paint.sculpt.pre),
            amount: Arc::clone(&self.paint.sculpt.amount),
            layer: self.paint.sculpt.layer,
            bbox: self.paint.sculpt.bbox,
        }
    }

    /// Reinstate the sculpt session from an undo model, in lock-step with the relief it describes.
    ///
    /// The blur memo is NOT restored — it is pure derived data, and rebuilding it lazily is what keeps the
    /// snapshot two refcount bumps instead of a third canvas-sized copy.
    pub(crate) fn restore_sculpt(&mut self, s: crate::undo::SculptSnap) {
        self.paint.sculpt.pre = s.pre;
        self.paint.sculpt.amount = s.amount;
        self.paint.sculpt.layer = s.layer;
        self.paint.sculpt.bbox = s.bbox;
        self.paint.sculpt.blurred = Vec::new();
        self.paint.sculpt.blur_done = Vec::new();
        self.paint.sculpt.blur_radius = 0;
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
        self.paint.sculpt.blurred = Vec::new();
        self.paint.sculpt.blur_done = Vec::new();
        self.paint.sculpt.layer = None;
        self.paint.sculpt.bbox = None;
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
        let pre = Arc::clone(&self.paint.sculpt.pre);
        if let Some(target) = self.heights.get_mut(&layer)
            && target.len() == pre.len()
        {
            let dst = Arc::make_mut(target);
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = pre[i]);
        }
        let amount = Arc::make_mut(&mut self.paint.sculpt.amount);
        super::impasto_settle::for_each_in(rect, w, |i| {
            if let Some(a) = amount.get_mut(i) {
                *a = 0.0; // a shape guard, not a formality: a rebind can leave a session sized for the
            } // OLD canvas while `rect` is measured against the new one
        });
        self.paint.sculpt.bbox = None;
        if let Some(r) = super::region::grow_region(rect, 1, w, self.source_size.1) {
            self.mark_dirty(r);
        }
    }
}
