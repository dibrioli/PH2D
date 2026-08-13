//! **A construção de UM dab** — o estágio do pipeline (cabeçalho de [`super`]) que transforma uma
//! posição do caminho num [`Dab`]: dinâmica de pressão, a atenuação de espaçamento, o jitter por-dab
//! (escala / rotação / cor, depois posição) e o taper.
//!
//! Módulo filho de [`super`], então mantém o acesso privado aos campos de [`Stroke`]; separado por
//! responsabilidade quando o pai cruzou o teto de LOC do workspace — ele estava a **nove linhas** do
//! cap antes desta wave, então o próximo campo o cruzaria de qualquer maneira.

use super::*;

impl Stroke {
    /// Build a dab at `pos`/`pressure`, applying pressure dynamics, the space-attenuation
    /// `overlap` multiplier, and the per-dab jitter (scale / rotate / colour, then position).
    pub(super) fn dab_at(&mut self, pos: [f32; 2], pressure: f32, overlap: f32, arc: f32) -> Dab {
        // Per-dab Scale / Rotate / Randomize-Color in a FIXED draw order, gated like position jitter
        // (Drag Dot / Anchored opt out). Drawn BEFORE the position jitter so it keeps its old slot
        // when these are off — an all-off brush draws nothing and matches the no-jitter baseline.
        let (scale, rotation, color) = if self.spec.stroke_method.allows_jitter() {
            crate::jitter::per_dab(&mut self.rng, &self.spec)
        } else {
            (1.0, [1.0, 0.0], self.spec.color)
        };
        let radius = (self.spec.clamped_radius() * self.dynamics.radius_scale(pressure) * scale)
            .clamp(0.5, MAX_BRUSH_RADIUS_PX);
        let coverage =
            (self.spec.strength * self.dynamics.coverage_scale(pressure) * overlap).clamp(0.0, 1.0);
        // Jitter reads the UN-tapered radius on purpose: the scatter is a property of the brush, not of
        // where along the stroke this dab happens to fall.
        //
        // O ARREMESSO entra antes do jitter — e a ordem NÃO é load-bearing; o porquê (medido) está
        // no doc do [`Stroke::throw`].
        let center = self.apply_jitter(self.throw(pos), radius);
        let mut dab = Dab {
            center,
            radius_px: radius,
            coverage,
            color,
            rotation,
            // The length-weighted EMA of the PATH tangent, advanced by this dab's own travel just above.
            // Read from the path, never from the jittered `center`, so position jitter cannot wobble it.
            dir: self.heading,
            arc_len: arc,
            stroke_radius_px: self.spec.clamped_radius(),
        };
        // The TAPER, applied here and ONLY here, to every dab this engine emits — the instant it is
        // emitted, at the position the pointer put it. ⚠️ Nothing is ever held back: a taper is a
        // property of a MARK, and an engine that withholds dabs to learn their distance-to-the-end has
        // traded the artist's latency for its own convenience (Enio 2026-08-08: *"o traço não pode ter
        // nenhum delay e nenhum stabilize"*). What the far end costs, [`mod@self::ends`] answers by
        // GEOMETRY instead: a path that knows its length is tapered exactly, and one that does not is
        // not tapered at its far end at all.
        let w = self.taper_width_at(arc);
        if w < 1.0 {
            crate::taper::scale_dab(&mut dab, &self.spec.taper, w);
        }
        dab
    }

    /// Offset the dab centre by a random vector sampled uniformly inside a disc, sized by the
    /// jitter unit: `Brush` → radius `jitter × diameter`; `View` → radius `2 × jitter_absolute_px`
    /// (Blender `BKE_brush_jitter_pos`). No-op for methods that disable jitter (DragDot/Anchored).
    pub(super) fn apply_jitter(&mut self, pos: [f32; 2], radius: f32) -> [f32; 2] {
        if !self.spec.stroke_method.allows_jitter() {
            return pos;
        }
        let max_offset = match self.spec.jitter_unit {
            JitterUnit::Brush => self.spec.jitter.clamp(0.0, 1.0) * (2.0 * radius),
            JitterUnit::View => 2.0 * self.spec.jitter_absolute_px.max(0.0),
        };
        if max_offset <= 0.0 {
            return pos;
        }
        let (dx, dy) = crate::jitter::disc_sample(&mut self.rng);
        [pos[0] + dx * max_offset, pos[1] + dy * max_offset]
    }
}
