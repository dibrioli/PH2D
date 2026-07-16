//! **The whole-layer filter** — the Sculpt's verbs applied to the entire layer at once, with no stroke
//! (Blender's *Mesh Filter*; plan §8, the second half of W5).
//!
//! Sibling of [`super::sculpt_session`] (the per-stroke session) because it is the OTHER driver of the
//! same engine: a session freezes `pre` and lets the dab walk fill `amount`; this freezes `pre` and fills
//! `amount` **itself**, uniformly. Then both call the SAME [`super::sculpt_blur::render_sculpt`]. That is
//! the whole feature — no second kernel, no geometry of its own (plan §10.1: *"um passe com geometria
//! própria é como nasce 'Tiling não funciona no Sculpt' daqui a seis meses"*).
//!
//! It also inherits every knob for free, which is why it needs **no knob of its own**: the verb chips pick
//! the verb, the verb's own knob (Radius / Depth / Smoothness) shapes it, and the brush's **Strength** is
//! how hard it lands — `amount` IS strength, because `k = amount.clamp(0, 1)` is the render's *"how far
//! along the travel we go"*. A filter with a private strength slider would be a second model of a number
//! the card already shows.

use super::Region;
use super::sculpt::{SculptFamily, SculptMode};
use crate::tool::PainterTool;
use std::sync::Arc;

impl SculptMode {
    /// Whether this verb can be applied to a **whole layer** — i.e. whether its target is a function of
    /// `pre` alone, or of the brush's FOOTPRINT.
    ///
    /// The [`SculptFamily::Plane`] verbs (Flatten / Scrape / Fill / Chisel) are refused, and the reason is
    /// not squeamishness: their target is a plane **least-squares fitted to the dab's footprint**
    /// (`plane_sum / amount`). A filter has no footprint, so "the whole layer" would have to fit ONE global
    /// plane and pull everything to it — a different operation with a different meaning (flatten the art to
    /// its mean plane), which is a **verb to design**, not a flag to flip. The Chisel is refused twice over:
    /// its V is folded around the **stroke's axis**, and a filter has no stroke (W3 paid for the rule that
    /// the axis is never a brush setting — see [`super::sculpt::SculptState::chisel_dir`]).
    ///
    /// What is left is exactly the list plan §8 named for W5: **Smooth · Sharpen · Inflate** (+ Layer, which
    /// falls out for free: its target is the constant `pre + Depth`).
    ///
    /// **The one door.** The panel asks it to decide whether to OFFER the button; the tool asks it to decide
    /// whether to HONOUR the click. A dimmed button that still dispatches is a lie, and two copies of "which
    /// verbs can be filtered" would drift the moment a ninth verb lands.
    pub(super) fn filters_layer(self) -> bool {
        !matches!(self.family(), SculptFamily::Plane)
    }
}

impl PainterTool {
    /// Apply the selected Sculpt verb to the **whole layer** at the brush's Strength, honouring the
    /// Selection. One undo step. Returns `false` when there was nothing to do — and every `false` is a
    /// fact, never a shrug: no layer, no relief on it, or a verb that has no whole-layer meaning
    /// ([`SculptMode::filters_layer`]).
    ///
    /// ## Why this is ten lines and not a kernel
    ///
    /// `render_sculpt` renders `h = pre + k·Δ(verb)` over a rect, reading `amount` for `k`. A stroke fills
    /// `amount` by walking dabs; there is nothing else a stroke contributes that a filter needs. So the
    /// filter fills `amount` with the Strength **directly** and renders the whole canvas. Every property the
    /// stroke path bought comes along: the memo (Smooth/Sharpen), the ball + matter advection (Inflate), the
    /// `pre`-is-frozen idempotence, the four-plane restore, the ceiling living in the light.
    ///
    /// The **Selection attenuates `amount`**, exactly as it attenuates a dab (`stamp_dabs_sculpt` hands the
    /// kernel the mask so it lands per-dab). Here there is one "dab" covering everything and it is written
    /// once, so multiplying in is not the double-scaling hazard that comment warns about — it is the same
    /// law, applied once.
    pub fn filter_sculpt_layer(&mut self) -> bool {
        // The refusal is not a silent no-op: the panel never OFFERS the button for these verbs (it asks
        // the same `filters_layer`), and this `false` is pinned by
        // `the_filter_refuses_the_verbs_whose_target_is_a_footprint`. A `debug_assert` here would fire on
        // that very gate; the gate IS the signal.
        if !self.paint.sculpt.mode_enum().filters_layer() {
            return false;
        }
        let before = self.snapshot_model();
        // A live gesture would have its OWN frozen `pre`; filtering mid-stroke would render the gesture and
        // the filter from one `amount` and commit both under one undo step that says "filter".
        self.end_sculpt_session();
        if !self.ensure_sculpt_session() {
            return false; // no layer, or no relief on it: nothing to filter (§5, the honest false)
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        // Strength is the filter's amount: `k = amount.clamp(0, 1)` is how far along the travel the render
        // goes, and Strength is the card's word for exactly that. Flow is deliberately NOT folded in — Flow
        // meters paint per dab along a stroke, and there is no stroke here.
        let strength = self.paint.brush.strength.clamp(0.0, 1.0);
        let mask: Option<Arc<Vec<u8>>> = self
            .selection_restricts_paint()
            .then(|| Arc::clone(&self.paint.selection_mask));
        {
            let amount = Arc::make_mut(&mut self.paint.sculpt.amount);
            if amount.len() != n {
                return false;
            }
            match mask.as_deref().filter(|m| m.len() == n) {
                Some(m) => {
                    for (a, &s) in amount.iter_mut().zip(m.iter()) {
                        *a = strength * (f32::from(s) / 255.0);
                    }
                }
                None => amount.fill(strength),
            }
        }
        let rect = Region { x: 0, y: 0, w, h };
        // `render_sculpt` names the texels it moved to `mark_dirty` itself — the screen is its business,
        // and adding an `invalidate_composite()` here is the exact mistake its own comment warns about.
        self.render_sculpt(rect);
        self.paint.sculpt.bbox = Some(rect);
        self.end_sculpt_session();
        self.commit_structural_edit(before);
        true
    }
}
