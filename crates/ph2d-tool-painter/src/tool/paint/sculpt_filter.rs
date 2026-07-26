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
use super::sculpt::SculptMode;
use crate::tool::PainterTool;
use std::sync::Arc;

/// **What a filter covers** — the two buttons the card offers (Enio, 2026-07-16).
///
/// The engine does not care: both fill `amount` and call the same render. The difference is one factor per
/// texel, which is exactly why this is a scope and not a second feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterScope {
    /// The whole layer — every texel the Selection allows.
    Layer,
    /// Only the **last stroke**, weighted by the paint it laid (`relief.live_paint`).
    ///
    /// The record already exists and is the honest one: it is what the Body card re-derives that stroke
    /// from, so it is *the* answer to "where did the last stroke go" — and it carries the stroke's own
    /// **soft falloff edge**, so the filter feathers out exactly where the paint did. A bbox would have
    /// been the obvious mask and it would have been wrong twice: hard-edged, and rectangular.
    LastStroke,
}

impl SculptMode {
    /// Whether this verb can be applied to a whole layer / a whole stroke — i.e. whether it **reshapes**
    /// the relief, rather than translating it or needing a footprint.
    ///
    /// **Smooth · Sharpen · Inflate**, and nothing else. Two exclusions, two different reasons:
    ///
    /// - The [`SculptFamily::Plane`] verbs (Flatten / Scrape / Fill / Chisel) pull toward a plane
    ///   **least-squares fitted to the dab's footprint** (`plane_sum / amount`). A filter has no footprint,
    ///   so "the whole layer's plane" would have to fit ONE global plane and pull everything to it — a
    ///   different operation with a different meaning (flatten the art to its mean plane), which is a
    ///   **verb to design**, not a flag to flip. The Chisel is refused twice over: its V folds around the
    ///   **stroke's axis**, and a filter has no stroke (W3 paid for the rule that the axis is never a brush
    ///   setting — see [`super::sculpt::SculptState::chisel_dir`]).
    /// - **Layer** is excluded because it is a **dead knob in both scopes** (Enio, 2026-07-16 — I had
    ///   shipped it reasoning that it "falls out for free", which is how dead knobs get shipped). Its
    ///   target is the constant `pre + Depth`, so filtering a whole layer with it adds `k·Depth` to *every*
    ///   texel — a uniform translation of the height field. **The light reads `∇h`**, and a constant has no
    ///   gradient: the button would not move one pixel. Scoped to the last stroke it does vary (by
    ///   `live_paint`), and is then a duplicate of the **Depth slider**, which already re-derives that same
    ///   stroke live from that same plane. Invisible or redundant — never a tool.
    ///
    /// **The one door.** The panel asks it to decide whether to OFFER the buttons; the tool asks it to
    /// decide whether to HONOUR the click. A dimmed button that still dispatches is a lie, and two copies
    /// of "which verbs can be filtered" would drift the moment a ninth verb lands.
    pub(super) fn filters_layer(self) -> bool {
        matches!(
            self,
            SculptMode::Smooth | SculptMode::Sharpen | SculptMode::Inflate
        )
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
    pub fn filter_sculpt_layer(&mut self, scope: FilterScope) -> bool {
        // The refusal is not a silent no-op: the panel never OFFERS the button for these verbs (it asks
        // the same `filters_layer`), and this `false` is pinned by
        // `the_filter_refuses_the_verbs_that_do_not_reshape`. A `debug_assert` here would fire on that
        // very gate; the gate IS the signal.
        if !self.paint.sculpt.mode_enum().filters_layer() {
            return false;
        }
        // The last stroke's envelope, if that is the scope — taken BEFORE the session opens, because
        // opening one is what would make "the last stroke" ambiguous. `None` ⇒ there is no last stroke on
        // this layer to filter, which is a refusal, not an empty run.
        let stroke: Option<Arc<Vec<f32>>> = match scope {
            FilterScope::Layer => None,
            FilterScope::LastStroke => match self.live_stroke_envelope() {
                Some(p) => Some(p),
                None => return false,
            },
        };
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
            // Two factors, both optional, both per-texel: the Selection (the tool's edge) and the scope
            // (the whole layer, or the last stroke's own envelope). `Layer` with nothing selected is the
            // degenerate case where both are 1 — which is why this is one loop and not two features.
            let sel = mask.as_deref().filter(|m| m.len() == n);
            let env = stroke.as_deref().filter(|p| p.len() == n);
            for (i, a) in amount.iter_mut().enumerate() {
                let s = sel.map_or(1.0, |m| f32::from(m[i]) / 255.0);
                let e = env.map_or(1.0, |p| p[i].clamp(0.0, 1.0));
                *a = strength * s * e;
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

    /// The **last stroke's paint envelope**, if there is a last stroke and it belongs to the layer the
    /// filter is about to touch. `None` is the honest refusal for both halves of that sentence.
    ///
    /// The layer check is not paperwork: `live_paint` is the record of a stroke made on
    /// `live_relief_layer`, and using it to weight a filter on a DIFFERENT layer would mask that layer's
    /// relief with the silhouette of a stroke that was never on it — a filter shaped like a ghost.
    ///
    /// **The one door** for "is there a last stroke to filter?": the panel asks it to decide whether to
    /// OFFER the button ([`Self::brush_settings`] publishes the answer), and this asks it to decide whether
    /// to honour the click.
    pub(super) fn live_stroke_envelope(&self) -> Option<Arc<Vec<f32>>> {
        // O PAGAMENTO, construído a partir da RESPOSTA — nunca ao lado dela. Uma segunda cópia dos três
        // testes aqui seria a segunda porta que este próprio doc-comment proíbe.
        self.has_live_stroke_envelope()
            .then(|| Arc::new(self.paint.relief.live_paint.clone()))
    }

    /// **A PERGUNTA, sem o pagamento** — os mesmos três testes, zero bytes copiados.
    ///
    /// ⚠️ **Isto custou 7,6 ms por frame com a mão PARADA, a 4096².** O `can_filter_last_stroke` abaixo é
    /// um **booleano** e chamava o irmão acima, que termina num `live_paint.clone()` — 16,7 M de `f32` =
    /// **67 MB memcpy** para responder *"existe um último traço?"*. Pior: quem pergunta é o
    /// `brush_settings()`, e o frame o chama **DUAS** vezes (o publish do painel e o anel do cursor), então
    /// o preço era cobrado em dobro — medido pelo split por-chamada do `PH2D_PAINT_PERF`, `brushsnap 3,9`
    /// + `ring 3,7`, que somam exatamente o `dispatch p50 = 7,5`.
    ///
    /// A lição é a do ADR-0124 num eixo novo: *quem está a jusante tem de ser informado do que precisa* —
    /// aqui, que precisa de um **sim ou não**, não do campo inteiro.
    #[must_use]
    pub(super) fn has_live_stroke_envelope(&self) -> bool {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        let live = &self.paint.relief;
        // `n == 0` and the `is_some` are not belt-and-braces — without them a tool with no canvas answers
        // YES: `n` is 0, the empty `live_paint` matches it, and `live_relief_layer == layers.active()`
        // because BOTH are `None`. The gate caught exactly that (a virgin tool offering Filter Stroke),
        // which is the fixture law from the other side: **zero does not fail** unless you make it.
        if n == 0 || live.live_paint.len() != n {
            return false;
        }
        live.live_relief_layer.is_some() && live.live_relief_layer == self.layers.active()
    }

    /// Whether the **Filter Stroke** button has anything to act on — the verb reshapes AND there is a last
    /// stroke on this layer. Published to the card so it never paints a button that would refuse.
    #[must_use]
    pub fn can_filter_last_stroke(&self) -> bool {
        self.paint.sculpt.mode_enum().filters_layer() && self.has_live_stroke_envelope()
    }
}
