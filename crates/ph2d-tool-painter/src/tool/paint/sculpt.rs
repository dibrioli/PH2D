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
/// Wave 3 appends (Clay / Layer / Draw Sharp / Inflate …); the order never shifts.
pub(crate) const SCULPT_MODE_COUNT: u8 = 5;

/// The kernel's radius at Radius = 1, in pixels.
///
/// Small on purpose, and the cap is a design statement rather than a performance excuse: smoothing at a
/// LARGE scale is not a bigger blur, it is **Flatten** — a least-squares plane fit over the footprint
/// (§7), which is a different kernel and lands in Wave 2. A 100-px "smooth" would be a slow, mushy,
/// worse Flatten. Sixteen pixels is the scale brush-marks and grain actually live at. // CLAMP-OK
pub(super) const SCULPT_RADIUS_MAX_PX: f32 = 16.0;

/// How far the plane Offset can travel, in **paint-loads** — the same unit `h` itself is in (`H_CEIL = 2.0`
/// is two full loads of paint).
///
/// One load either way: at `−1` the Scrape's plane sits a full stroke's thickness UNDER the surface it was
/// fitted to, which is a spatula gouging rather than skimming; at `+1` the Fill mounds a full load above it.
/// Wider than that and the plane leaves the paint entirely — Scrape would find nothing above it to remove,
/// and the control's whole outer half would silently do nothing. // CLAMP-OK
pub(super) const PLANE_OFFSET_MAX: f32 = 1.0;

/// Which verb the sculpt brush performs. Discriminants are the segmented control's indices.
///
/// **All five are one kernel** — `h = pre + k·Δ`, where `Δ` is the distance to a per-texel *target* and the
/// verb decides two things: where the target comes from, and which SIGN of `Δ` is allowed through.
///
/// | verb | target | Δ |
/// |---|---|---|
/// | Smooth | `blur(pre)` | both ways |
/// | Sharpen | `pre + (pre − blur(pre))` | both ways |
/// | Flatten | the fitted **plane** | both ways |
/// | Scrape | the fitted plane | **down only** (`min(Δ, 0)`) |
/// | Fill | the fitted plane | **up only** (`max(Δ, 0)`) |
///
/// Scrape and Fill are therefore not two more engines: they are Flatten with one half of the number thrown
/// away, and they cost one `min` each.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum SculptMode {
    /// Pull the relief toward its own local average — `lerp(pre, blur(pre), k)`.
    Smooth,
    /// Push it away from that average — `pre + k·(pre − blur(pre))`, an unsharp mask on a height field.
    /// Bounded by construction (`k ≤ 1` ⇒ at most twice the local detail), so it cannot ring away.
    Sharpen,
    /// Pull the relief toward the **tilted plane** fitted to the footprint (§7). Tilted, not horizontal:
    /// a horizontal fit cuts a crater into a hillside instead of flattening it.
    Flatten,
    /// The same plane, **down only** — the spatula taking the high ground off and leaving the valleys.
    Scrape,
    /// The same plane, **up only** — paint pushed into the valleys, the ridges left standing.
    Fill,
}

/// Which *engine* a verb runs on — and therefore which per-texel target buffer the session must carry.
///
/// It is not cosmetic bookkeeping: the two targets are derived from different things, and only one of them
/// can be rebuilt from what a parked session holds.
///
/// * [`Smooth`](SculptFamily::Smooth) → `blur(pre)`: a function of the frozen relief ALONE, so it is a pure
///   memo — throw it away and it rebuilds itself, tile by tile, from `pre`.
/// * [`Plane`](SculptFamily::Plane) → `Σ w·plane(i)`: a function of the **dab list**, which no longer
///   exists once the batch has been consumed. It cannot be rebuilt without re-stamping the stroke.
///
/// That asymmetry is why [`PainterTool::set_sculpt_mode`] re-stamps an open shape when the family changes,
/// and why it does not need to when it does not.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum SculptFamily {
    Smooth,
    Plane,
}

impl SculptMode {
    pub(super) fn from_u8(v: u8) -> Self {
        match v {
            1 => SculptMode::Sharpen,
            2 => SculptMode::Flatten,
            3 => SculptMode::Scrape,
            4 => SculptMode::Fill,
            _ => SculptMode::Smooth,
        }
    }

    pub(super) fn family(self) -> SculptFamily {
        match self {
            SculptMode::Smooth | SculptMode::Sharpen => SculptFamily::Smooth,
            SculptMode::Flatten | SculptMode::Scrape | SculptMode::Fill => SculptFamily::Plane,
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
    /// Plane Offset track (`0..1`; mapped to `∓PLANE_OFFSET_MAX` paint-loads by [`Self::plane_offset`]).
    pub(crate) offset_norm: f32,

    /// `Σ w·plane_d(i)` over the stroke's dabs — the **plane family's** per-texel target, un-normalised.
    ///
    /// The render divides by `amount[i]` (the same `Σ w`) to get the coverage-weighted mean of every plane
    /// that touched the texel. Storing the mean directly would be wrong the moment a second dab arrives;
    /// storing the planes would need the dab list, which the shape editors rebuild from scratch every frame.
    ///
    /// `Arc` for the same reason `amount` is: the undo snapshot must be a refcount bump, not a 64 MB copy.
    /// Empty in the Smooth family — the two targets are mutually exclusive, which is what holds the session
    /// at **12 B/px** with five verbs instead of the sixteen a naive union would cost.
    pub(crate) plane_sum: Arc<Vec<f32>>,
    /// Per-dab `(index, weight)` scratch for the plane fit — caller-owned so a hot stroke allocates nothing
    /// (the silhouette is the expensive sample and it must be taken exactly once; see
    /// [`ph2d_painter_brush::sculpt::accumulate_dab_plane`]). Never read across dabs; never snapshotted.
    pub(crate) fit_scratch: Vec<(u32, f32)>,

    /// `blur(pre, radius)`, memoised tile by tile (see [`super::sculpt_blur`]). Pure derived data: it dies
    /// with the session and is rebuilt lazily, tile by tile, on demand. Empty in the Plane family.
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
            offset_norm: 0.5, // dead centre = offset 0: the plane sits ON the surface it was fitted to
            pre: Arc::new(Vec::new()),
            amount: Arc::new(Vec::new()),
            plane_sum: Arc::new(Vec::new()),
            fit_scratch: Vec::new(),
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

    /// The plane's Offset in **paint-loads** (`−PLANE_OFFSET_MAX ..= +PLANE_OFFSET_MAX`), from the slider's
    /// `0..1` track. Dead centre is exactly `0.0` — the plane sits on the surface it was fitted to.
    pub(super) fn plane_offset(&self) -> f32 {
        (self.offset_norm.clamp(0.0, 1.0) - 0.5) * 2.0 * PLANE_OFFSET_MAX
    }

    pub(super) fn mode_enum(&self) -> SculptMode {
        SculptMode::from_u8(self.mode)
    }
}

impl PainterTool {
    // ── UI-edit setters (the single clamp source; `handle_panel_event` forwards raw panel values here) ──

    /// Select the sub-mode from the segmented index (out of range clamps to the last mode).
    ///
    /// **A change of FAMILY re-stamps an open shape** rather than merely re-rendering it, and that is not
    /// belt-and-braces: the plane family's per-texel target (`plane_sum`) is a function of the DAB LIST,
    /// which the render no longer has. Switching Smooth → Scrape and only re-rendering would divide by an
    /// all-zero `plane_sum` and pull the whole footprint to height 0 — a flattening to the floor, dressed
    /// as a scrape. Re-stamping rebuilds the target from the dabs, and it is precisely what the house
    /// already does when Size / Spacing / Falloff change (`refill_if_appearance_changed`).
    ///
    /// A change WITHIN a family (Smooth ↔ Sharpen, Scrape ↔ Fill) reuses the target it already has, so the
    /// cheap re-render is the correct one.
    pub fn set_sculpt_mode(&mut self, m: u8) {
        let before = self.paint.sculpt.mode_enum().family();
        self.paint.sculpt.mode = m.min(SCULPT_MODE_COUNT - 1);
        if self.paint.sculpt.mode_enum().family() == before {
            self.refresh_live_sculpt();
        } else {
            self.drop_stale_family_target();
            self.refill_open_shape(); // rebuild the new family's target from the dabs (no-op if none open)
            self.refresh_live_sculpt();
        }
    }

    /// Set the kernel Radius from the slider's `0..1` track (Smooth family).
    pub fn set_sculpt_radius(&mut self, t: f32) {
        self.paint.sculpt.radius_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Set the plane Offset from the slider's `0..1` track (Plane family).
    ///
    /// No re-stamp: the Offset is a **rigid shift** of the plane, and `Σ w·(plane + off) = plane_sum +
    /// off·amount`, so the render applies it with one add. That is the whole reason it is not folded into
    /// `plane_sum` at accumulation time — it keeps the slider live on an open shape for free.
    pub fn set_sculpt_offset(&mut self, t: f32) {
        self.paint.sculpt.offset_norm = t.clamp(0.0, 1.0);
        self.refresh_live_sculpt();
    }

    /// Whether the active operation is **Sculpt** — the panel shows the Sculpt card (and hides the colour
    /// controls, which a tool that lays no pigment has no use for).
    #[must_use]
    pub fn is_sculpt_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Sculpt)
    }

    /// Whether the active verb is a **plane** one (Flatten / Scrape / Fill) — the panel shows the Offset row
    /// for these and the Radius row for the others. One or the other, never both: a knob that does nothing
    /// to the active verb is a knob that lies.
    #[must_use]
    pub fn is_sculpt_plane_mode(&self) -> bool {
        self.paint.sculpt.mode_enum().family() == SculptFamily::Plane
    }

    /// The kernel radius in px — what the Radius chip shows the artist, and what the kernel actually uses.
    /// One function, so the number on screen cannot drift from the number in the blur.
    #[must_use]
    pub fn sculpt_radius_px(&self) -> u32 {
        self.paint.sculpt.radius_px()
    }

    /// The plane Offset in paint-loads — what the Offset chip shows, and what the kernel adds. One function,
    /// same reason.
    #[must_use]
    pub fn sculpt_plane_offset(&self) -> f32 {
        self.paint.sculpt.plane_offset()
    }

    /// Free the target the OTHER family owns, on a family switch.
    ///
    /// Not an optimisation — it is what keeps the promise the cost gate makes. The two targets are the same
    /// size, so a session that had carried both would cost 16 B/px instead of 12, and the moment that is
    /// true "the session costs 12 B/px" becomes a sentence in a comment rather than a fact about the
    /// program. Both are rebuildable: `blurred` from `pre` (lazily, tile by tile) and `plane_sum` from the
    /// re-stamp that is about to run anyway.
    fn drop_stale_family_target(&mut self) {
        match self.paint.sculpt.mode_enum().family() {
            SculptFamily::Smooth => {
                self.paint.sculpt.plane_sum = Arc::new(Vec::new());
            }
            SculptFamily::Plane => {
                self.paint.sculpt.blurred = Vec::new();
                self.paint.sculpt.blur_done = Vec::new();
                self.paint.sculpt.blur_radius = 0;
            }
        }
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
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SCULPT_OFFSET_SLIDER => {
                self.set_sculpt_offset(*v as f32);
                true
            }
            _ => false,
        }
    }
}
