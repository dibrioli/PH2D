//! The **Impasto relief's per-stroke state** — split out of the `PaintState` god-struct for the
//! workspace LOC cap, and because it is one coherent thing: the planes a stroke deposits its body into,
//! the ingredients the Body card re-derives that body FROM, and the window all of them are indexed
//! against. Everything here is born at pen-down and dies at the next one; nothing survives the document.

use super::Region;

/// See the module docs. Fields are `pub(super)` so the whole `paint` module tree reaches them exactly as
/// it did when they lived on `PaintState` directly.
#[derive(Default)]
pub(super) struct ReliefState {
    /// **Impasto** per-stroke relief (f32, `w*h`) — the deposit ([`ph2d_painter_brush::height::
    /// derive_height`] of the ingredient planes below) plus the displacement the brush banked. Merged
    /// into the layer at `close_stroke`. Empty ⇒ no impasto this stroke (zero cost).
    pub(super) stroke_height: Vec<f32>,
    /// **Impasto** per-stroke PAINT envelope (f32, `0..1`, by `max` — the heaviest dab owns the pixel,
    /// so one pass leaves one thickness). Both the stroke's coverage (what the light weighs its shading
    /// by; merged into the layer's `covers` at stroke end) and the first ingredient of the relief.
    pub(super) stroke_paint: Vec<f32>,
    /// **Impasto** per-stroke DISPLACEMENT at `Push = 1` (f32, `w*h`; `ph2d_painter_brush::height_push`)
    /// — negative where the brush took paint, positive where it banked it, summing to zero. Linear in
    /// Push, so it is an INGREDIENT: the ridge appears under the moving brush *and* the knob stays live.
    pub(super) stroke_push: Vec<f32>,
    /// **Impasto** per-stroke GRAIN (1 byte/px, `255` = none): the grain sample of the dab that won each
    /// pixel. The second ingredient — what lets `Depth Source` be flipped AFTER the stroke and re-carve
    /// the very grooves that dab would have left.
    pub(super) stroke_grain: Vec<u8>,
    /// The **solid paint** this stroke laid (`ph2d_painter_brush::height::solid_paint`) — the film's own
    /// alpha, and the coverage the LIGHT weighs its shading by. Separate from [`Self::stroke_paint`] (the
    /// relief's ingredient, the raw silhouette × dynamics) because the two are different functions of the
    /// dab: the body curve runs on the silhouette and the dynamics scale it, which is what keeps the light
    /// alive under a light touch. Merged into the layer's `covers` at stroke end.
    pub(super) stroke_film: Vec<u8>,
    /// **Impasto**: where each Symmetry copy of the stroke was when the LAST pointer batch ended, so the
    /// first dab of the next batch can sweep back to it. Without it the relief would bead at every
    /// pointer event — a beading chosen by the artist's mouse polling rate, not by their hand. One slot
    /// per copy; cleared on pen-down. Mirrors `last_smear_pos` (which is the same idea, for the smear
    /// chain), but per-copy, because Symmetry paints several strokes at once.
    pub(super) last_height_center: Vec<Option<[f32; 2]>>,
    /// **Impasto live-edit** — the LAST stroke's ingredients, so the whole Body card re-derives that
    /// stroke after the fact instead of only affecting the next one (Enio 2026-07-12). Storing the
    /// HEIGHT instead bakes Body and Depth Source into it, leaving nothing to re-derive them from — the
    /// first cut's bug. Empty ⇒ nothing live.
    pub(super) live_paint: Vec<f32>,
    /// That stroke's grain plane — the second ingredient (see [`Self::stroke_grain`]).
    pub(super) live_grain: Vec<u8>,
    pub(super) live_push: Vec<f32>, // that stroke's displacement at `Push = 1` — see [`Self::stroke_push`]
    pub(super) push_scratch: Vec<f32>, // per-dab rim weights, reused (so `bank_dab_push` allocates nothing)
    /// The active layer's committed relief BEFORE that stroke — the ground the re-derived stroke is
    /// added back onto. **A PATCH over [`Self::live_relief_rect`], not the canvas**: outside that rect
    /// the stroke contributes nothing, so the layer's relief there is its own and never re-derived.
    /// Empty ⇒ the layer had none (the common case: a first stroke).
    pub(super) live_relief_base: Vec<f32>,
    /// Which layer the live stroke belongs to. `None` ⇒ nothing live.
    pub(super) live_relief_layer: Option<crate::tool::RtLayerId>,
    /// The union of this stroke's dab footprints, in canvas texels — accumulated as the relief is
    /// deposited, cleared with the stroke.
    ///
    /// The commit used to re-derive, settle, diff and re-base over the **whole canvas** for a stroke that
    /// touched a corner of it — a one-SECOND freeze at every pen-up at 4096² (§11). This is the window
    /// that makes that work `O(stroke)`.
    pub(super) stroke_relief_bbox: Option<Region>,
    /// [`Self::stroke_relief_bbox`] grown by the settle's reach and clipped to the canvas — the window
    /// the live re-derive owns. Every buffer above that is "per-stroke" is indexed against THIS.
    pub(super) live_relief_rect: Option<Region>,
    /// Whether the layer already carried a height entry before the live stroke — so a re-derive that
    /// zeroes the stroke out (Depth 0) knows whether the entry is now empty or merely untouched here.
    pub(super) live_relief_had_entry: bool,
}
