//! **A construção de UM dab** — o estágio do pipeline (cabeçalho de [`super`]) que transforma uma
//! posição do caminho num [`Dab`]: dinâmica de pressão, a atenuação de espaçamento, o jitter por-dab
//! (escala / rotação / cor, depois posição) e o taper.
//!
//! Módulo filho de [`super`], então mantém o acesso privado aos campos de [`Stroke`]; separado por
//! responsabilidade quando o pai cruzou o teto de LOC do workspace — ele estava a **nove linhas** do
//! cap antes desta wave, então o próximo campo o cruzaria de qualquer maneira.

use super::*;

impl Stroke {
    /// Randomize Color **per stroke**: the `(colour, HSV offset)` drawn ONCE at pen-down, from a stream
    /// of its own seeded by the stroke `seed` — so the per-dab stream every other jitter reads is
    /// untouched, and a fill method that rebuilds its stroke every Move with the same seed keeps its
    /// colour for the whole gesture.
    pub(crate) fn stroke_colour_draw(spec: &BrushSpec, seed: u64) -> Option<([f32; 3], [f32; 3])> {
        (spec.color_jitter_scope == crate::jitter::ColorJitterScope::PerStroke
            && spec.has_colour_jitter_amount())
        .then(|| crate::jitter::colour_draw(&mut (seed ^ 0x636F_6C6F_7572_5354), spec))
    }

    /// Build a dab at `pos`/`pressure`, applying pressure dynamics, the space-attenuation
    /// `overlap` multiplier, and the per-dab jitter (scale / rotate / colour, then position).
    ///
    /// ⚠️ **Ela ARREMESSA e só então constrói**, e as duas metades estão separadas porque o
    /// arremesso é um fato do CAMINHO (acontece uma vez por ponto) e a construção é um fato da
    /// MARCA (acontece uma vez por cópia do spray) — ver [`mod@super::spray`].
    pub(super) fn dab_at(&mut self, pos: [f32; 2], pressure: f32, overlap: f32, arc: f32) -> Dab {
        let thrown = self.throw(pos, arc);
        self.advance_step(thrown);
        self.dab_from(thrown, pressure, overlap, arc)
    }

    /// The path step to `thrown` from the previous path point ([`Dab::step`]) — once per path point,
    /// so the spray copies and Rough passes of that point share it. Zero unless **Continuous Spacing** is
    /// armed ([`BrushSpec::drags_bristles`]), and for the deliberate single marks (Drag Dot / Anchored /
    /// Grid Stamp).
    pub(super) fn advance_step(&mut self, thrown: [f32; 2]) {
        let from = self.step_from.replace(thrown);
        self.dab_step = match from {
            Some(f) if self.spec.drags_bristles() && self.spec.stroke_method.allows_jitter() => {
                [thrown[0] - f[0], thrown[1] - f[1]]
            }
            _ => [0.0, 0.0],
        };
    }

    /// Constrói UMA marca na posição onde a tinta já caiu (pós-arremesso) — a metade de
    /// [`Self::dab_at`] que o spray repete.
    ///
    /// ⚠️ **Cada chamada sorteia o SEU jitter** (escala → rotação → cor → posição, na ordem fixa de
    /// sempre), e é exatamente isso que faz de `n` cópias uma nuvem em vez de `n` carimbos
    /// empilhados. Um pincel sem jitter nenhum devolve `n` marcas idênticas — degenerado, honesto,
    /// e o slider ao lado é a cura.
    pub(super) fn dab_from(
        &mut self,
        thrown: [f32; 2],
        pressure: f32,
        overlap: f32,
        arc: f32,
    ) -> Dab {
        // Per-dab Scale / Rotate / Randomize-Color in a FIXED draw order, gated like position jitter
        // (Drag Dot / Anchored opt out). Drawn BEFORE the position jitter so it keeps its old slot
        // when these are off — an all-off brush draws nothing and matches the no-jitter baseline.
        let (scale, rotation, mut color, mut color_shift) =
            if self.spec.stroke_method.allows_jitter() {
                crate::jitter::per_dab(&mut self.rng, &self.spec)
            } else {
                (1.0, [1.0, 0.0], self.spec.color, [0.0; 3])
            };
        // Randomize Color «per stroke»: the per-dab draw above still ran (the stream stays the one every
        // other jitter reads), and the stroke's own draw replaces it.
        if let (Some(c), true) = (self.stroke_colour, self.spec.stroke_method.allows_jitter()) {
            (color, color_shift) = c;
        }
        let radius = (self.spec.clamped_radius() * self.dynamics.radius_scale(pressure) * scale)
            .clamp(0.5, MAX_BRUSH_RADIUS_PX);
        let coverage =
            (self.spec.strength * self.dynamics.coverage_scale(pressure) * overlap).clamp(0.0, 1.0);
        // Jitter reads the UN-tapered radius on purpose: the scatter is a property of the brush, not of
        // where along the stroke this dab happens to fall.
        //
        // O ARREMESSO entra antes do jitter — e a ordem NÃO é load-bearing; o porquê (medido) está
        // no doc do [`Stroke::throw`], que o chamador já rodou.
        let center = self.apply_jitter(thrown, radius);
        let mut dab = Dab {
            center,
            radius_px: radius,
            coverage,
            color,
            color_shift,
            rotation,
            // The length-weighted EMA of the PATH tangent, advanced by this dab's own travel just above.
            // Read from the path, never from the jittered `center`, so position jitter cannot wobble it.
            dir: self.heading,
            arc_len: arc,
            stroke_radius_px: self.spec.clamped_radius(),
            step: self.dab_step,
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

    /// Build the Anchored stamp's single dab at `center` with the drag-defined `radius_px`. No
    /// jitter (Blender disables it for Anchored, `paint_stroke_use_jitter`), full pressure
    /// (coverage = strength); the falloff / hardness / blend / colour come from the spec at stamp
    /// time, like every dab — so the anchored stamp wears the brush profile, just sized by the drag.
    pub(super) fn anchored_dab(&self, center: [f32; 2], radius_px: f32, dir: [f32; 2]) -> Dab {
        Dab {
            center,
            radius_px: radius_px.clamp(0.0, MAX_BRUSH_RADIUS_PX),
            coverage: self.spec.strength.clamp(0.0, 1.0),
            // Anchored opts out of all per-dab jitter (like position jitter): a deliberate stamp.
            color: self.spec.color,
            color_shift: [0.0; 3],
            rotation: [1.0, 0.0],
            // The Rake heading is the drag direction (anchor → cursor), set by the caller.
            dir,
            // Anchored is a single pinned stamp — there is no travel, so the Flow along-coordinate is 0.
            arc_len: 0.0,
            stroke_radius_px: self.spec.clamped_radius(),
            step: [0.0, 0.0],
        }
    }
}
