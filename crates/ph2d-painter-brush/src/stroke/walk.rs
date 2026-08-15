//! **OS CAMINHADORES** — como uma CORDA do caminho vira uma corrida de dabs.
//!
//! Dois, e a diferença entre eles é *o que liga duas amostras do ponteiro*: o [`Stroke::walk_space`]
//! percorre um segmento RETO emitindo um dab a cada `spacing × diâmetro` de arco, e o
//! [`Stroke::walk_smoothed`] achata uma Catmull-Rom entre as amostras em cordas curtas e as encadeia
//! pelo primeiro. Todo o resto do motor de depósito (dash, jitter, taper, atenuação de espaçamento,
//! heading) mora no primeiro, que é a porta por onde um caminho vira tinta.
//!
//! ⚠️ **Módulo FILHO, como os catorze irmãos ao lado** — o corte é de ASSUNTO (*como um caminho vira
//! dabs* contra *o que é um traço*) e não de tamanho, e mantendo-o filho o `use super::*` alcança os
//! campos privados do [`Stroke`] sem alargar visibilidade nenhuma.

use super::*;

impl Stroke {
    /// Space spacing walk from `last_pos` → `target`, emitting a dab every `spacing × diameter` of
    /// arc length and carrying the residual distance across calls (`accum`).
    pub(super) fn walk_space(&mut self, target: StrokePoint, out: &mut Vec<Dab>) {
        let from = self.last_pos;
        let to = target.pos;
        let seg = dist(from, to);
        if seg <= f32::EPSILON {
            self.last_pressure = target.pressure;
            return;
        }
        let base_step = self.spec.dab_spacing_px();
        let dir = [(to[0] - from[0]) / seg, (to[1] - from[1]) / seg];
        // Smoothing length for the heading EMA, from the brush diameter (Krita's Fade is brush-relative).
        let smooth_len = crate::heading::smooth_len(2.0 * self.spec.clamped_radius());
        let overlap = self.method_overlap();
        // Arc-length at `from` (= current cumulative to `last_pos`); each dab stamps `base_arc + traveled`,
        // and the segment advances the accumulator by its full length so the along-stroke coordinate is
        // continuous across walk_space calls (the Catmull-Rom flattener calls this once per short chord).
        let base_arc = self.arc_len;
        let mut traveled = 0.0;
        // Path length of this chord already folded into the heading EMA. The filter is length-weighted
        // (`α = Δs/(Δs + L)`) and its contract is that behaviour is independent of how the path was
        // chopped into steps — so it must be fed the travel that ACTUALLY elapsed, including the travel of
        // chords that emit no dab at all. Feeding it the within-chord remainder instead (the bug until
        // 2026-07-19) ran it with a step up to 16× too small, i.e. an effective smoothing length of ~240 px
        // where 12 was intended, which is where the 52° of Rake lag came from.
        let mut advanced = 0.0;
        // ⚠️ **Uma corda NÃO-FINITA não é percorrível, e falhar cedo aqui é o barato.** `seg` alimenta
        // `f = traveled / seg` e a direção; com `NaN` toda comparação é falsa, o `break` do laço nunca
        // dispara e o percurso escreve para sempre. Recusar é honesto: não há caminho a desenhar.
        if !seg.is_finite() || !to[0].is_finite() || !to[1].is_finite() {
            return;
        }
        loop {
            // ⚠️ **A conferência é ANTES do carimbo, e a 1ª versão era depois.** O flattener chama
            // este percurso uma vez por corda, então um teto conferido no FIM deixa cada chamada
            // seguinte carimbar mais um antes de desistir — medido, 86 dabs de excesso. Perguntada
            // aqui, a corda que chega com o buffer cheio sai sem escrever nada.
            if out.len() >= MAX_DABS_PER_WALK {
                break; // ver [`MAX_DABS_PER_WALK`]: batente de MEMÓRIA, nunca alcançado por um gesto
            }
            // **Jitter Spacing** scales each gap by the carried multiplier (`1.0` = even); a break does
            // not redraw (no wasted draw), and the next-gap draw is gated so an off brush is unchanged.
            // ⚠️ The gap is `spacing × the diameter that will actually be stamped here`, not the nominal
            // one. Spacing is a RATIO — "how much of a dab's width until the next" — so holding the gap
            // fixed while the taper shrinks the dab makes the ratio blow up and the tip comes out as a
            // row of separate DOTS (Enio 2026-08-08, with the picture). Scaling the step by the same
            // width factor the dab gets keeps the overlap constant all the way to the point.
            //
            // The `max(1.0)` floor is the engine's own, not a new number: it already bounded the walk,
            // and it is what stops the loop from stalling as the factor approaches zero at a sharp tip.
            let w = self.taper_width_at(base_arc + traveled);
            let step = (base_step * w * self.spacing_mult).max(1.0);
            // A step that shrank below what is already banked (a taper ramp, or a live spacing edit)
            // must not walk BACKWARDS: emit here, and the reset of `accum` below carries the walk on.
            let to_next = (step - self.accum).max(0.0);
            if traveled + to_next > seg {
                break;
            }
            traveled += to_next;
            let f = traveled / seg;
            let pos = [from[0] + dir[0] * traveled, from[1] + dir[1] * traveled];
            let pressure = lerp(self.last_pressure, target.pressure, f);
            // Fold in exactly the path travelled since the heading was last advanced, so the dab is
            // stamped with the heading as of ITS position and the smoothing length is honoured in arc
            // length, independent of dab density and of the flattener's chord size.
            self.heading =
                crate::heading::advance(self.heading, dir, traveled - advanced, smooth_len);
            advanced = traveled;
            if self.spec.dash_on(self.tot_samples) {
                self.stamp_at(pos, pressure, overlap, base_arc + traveled, out);
            }
            self.tot_samples = self.tot_samples.wrapping_add(1);
            self.accum = 0.0;
            self.spacing_mult = if self.spec.jitter_spacing > 0.0 {
                crate::jitter::spacing_mult(&mut self.rng, self.spec.jitter_spacing)
            } else {
                1.0
            };
        }
        self.accum += seg - traveled;
        // The rest of the chord still happened — fold it in, so a chord that emits NO dab (the common
        // case: the Catmull-Rom flattener chops at ~3 px while the spacing can be tens of px) still steers
        // the heading. Without this the EMA only ever saw the sub-chord slivers around the dabs it did emit.
        self.heading = crate::heading::advance(self.heading, dir, seg - advanced, smooth_len);
        self.arc_len = base_arc + seg; // arc at `to` = arc at `from` + the full chord length
        self.last_pos = to;
        self.last_pressure = target.pressure;
    }

    /// Freehand smoother for the `Space` method — a **Catmull-Rom spline** through the input points.
    /// Each `extend` paints the segment from the last point `a` to the new point `p`, so the stroke
    /// follows the cursor in real time (no held-back tail), and the spline interpolates *through*
    /// every sample with a smooth, continuous tangent, so sparse / coalesced input reads as a clean
    /// curve instead of the connected straight facets the old scheme produced.
    ///
    /// The segment `a → p` is a cubic Hermite with the Catmull-Rom tangents: at `a` the centripetal
    /// tangent `(p − prev_prev)/2` (smooth join with the previous segment), at `p` the causal chord
    /// `p − a`. The first segment after [`Stroke::begin`] has no `prev_prev` (start tangent = chord,
    /// straight). The cubic is flattened into short chords fed through [`Stroke::walk_space`] (which owns
    /// spacing / dash / jitter / attenuation). Collinear input keeps straight strokes straight.
    pub(super) fn walk_smoothed(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        let a = self.last_pos;
        let a_pr = self.last_pressure;
        let b = p.pos;
        let seg = dist(a, b);
        if seg <= f32::EPSILON {
            self.last_pressure = p.pressure;
            return;
        }
        // Catmull-Rom tangents (at `a`: centred on its neighbours `prev_prev` and `b`; the first
        // segment uses the chord. at `b`: the causal chord), scaled by the stabilizer intensity `w`:
        // `w = 0` ⇒ zero tangents ⇒ the Hermite is the straight chord `a→b` (raw, faceted path);
        // `w → 1` ⇒ full curvature between samples. So the one knob ramps from raw to smooth.
        let w = self.spec.stabilizer.clamp(0.0, 1.0);
        let m_a = match self.prev_prev {
            Some(pp) => [(b[0] - pp[0]) * 0.5 * w, (b[1] - pp[1]) * 0.5 * w],
            None => [(b[0] - a[0]) * w, (b[1] - a[1]) * w],
        };
        let m_b = [(b[0] - a[0]) * w, (b[1] - a[1]) * w];
        // Flatten the Hermite `a → b` into short chords (denser than the dab spacing so the curve
        // never facets), each chained through `walk_space`.
        let n = ((seg / 3.0).ceil() as usize).clamp(1, 96);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            self.walk_space(
                StrokePoint {
                    pos: hermite(a, m_a, b, m_b, t),
                    pressure: lerp(a_pr, p.pressure, t),
                },
                out,
            );
        }
        // This segment's start point becomes the next segment's `prev_prev` neighbour.
        self.prev_prev = Some(a);
    }
}
