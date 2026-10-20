//! **A construção de UM dab** — o estágio do pipeline (cabeçalho de [`super`]) que transforma uma
//! posição do caminho num [`Dab`]: dinâmica de pressão, a atenuação de espaçamento, o jitter por-dab
//! (escala / rotação / cor, depois posição) e o taper.
//!
//! Módulo filho de [`super`], então mantém o acesso privado aos campos de [`Stroke`]; separado por
//! responsabilidade quando o pai cruzou o teto de LOC do workspace — ele estava a **nove linhas** do
//! cap antes desta wave, então o próximo campo o cruzaria de qualquer maneira.

use super::*;

/// O que se sorteia UMA vez por traço, cada um do seu fluxo próprio semeado pela `seed` do traço (o fluxo por
/// dab que os outros sorteios leem fica intacto): a cor por traço e o ângulo do traço (L4).
#[derive(Clone, Copy, Debug)]
pub(super) struct PorTraco {
    /// Randomize Color **per stroke** — `(cor, desvio HSV)`; `None` ⇒ por dab ou desligado.
    cor: Option<([f32; 3], [f32; 3])>,
    /// O rotor do ângulo do traço ([`crate::marca::DabJitter::angle_per_stroke`]); `[1, 0]` = nenhum.
    rotor: [f32; 2],
    /// O fluxo PRÓPRIO dos sorteios da marca (L4): ligar um deles não desloca os de sempre (posição, escala,
    /// rotação, cor), que continuam a ler o `Stroke::rng` ao bit.
    pub(super) rng: u64,
}

impl PorTraco {
    pub(super) fn de(spec: &BrushSpec, seed: u64) -> Self {
        Self {
            cor: Stroke::stroke_colour_draw(spec, seed),
            rotor: crate::marca::rotor_do_traco(&spec.dab_jitter, seed),
            rng: seed ^ 0x6D61_7263_6144_4142,
        }
    }
}

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
    /// on ([`BrushSpec::continuous_spacing`]), and for the deliberate single marks (Drag Dot / Anchored /
    /// Grid Stamp).
    pub(super) fn advance_step(&mut self, thrown: [f32; 2]) {
        let from = self.step_from.replace(thrown);
        self.dab_step = match from {
            Some(f) if self.spec.lays_capsules() => [thrown[0] - f[0], thrown[1] - f[1]],
            _ => [0.0, 0.0],
        };
        self.dab_ante = self.ultimo.filter(|_| self.dab_step != [0.0, 0.0]);
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
        let (scale, mut rotation, mut color, mut color_shift) =
            if self.spec.stroke_method.allows_jitter() {
                crate::jitter::per_dab(&mut self.rng, &self.spec)
            } else {
                (1.0, [1.0, 0.0], self.spec.color, [0.0; 3])
            };
        // Randomize Color «per stroke»: the per-dab draw above still ran (the stream stays the one every
        // other jitter reads), and the stroke's own draw replaces it.
        let jitter = self.spec.stroke_method.allows_jitter();
        if let (Some(c), true) = (self.por_traco.cor, jitter) {
            (color, color_shift) = c;
        }
        // A cor pela MÃO e a secundária (L11): a pressão deste dab, o sorteio do fluxo da marca.
        if self.spec.cor_mao.ativa() {
            color = self.spec.cor_mao.aplica(color, pressure, &mut self.por_traco.rng);
        }
        // O ângulo do traço (L4): o MESMO rotor em todo dab deste traço.
        if jitter && self.por_traco.rotor != [1.0, 0.0] {
            rotation = crate::heading::rotate(rotation, self.por_traco.rotor);
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
        let (disco, alcance) = self.apply_jitter(thrown, radius);
        // O jitter da MARCA (L4), sorteado depois da posição: o desvio ATRAVÉS do traço soma-se ao do disco.
        let marca = jitter
            .then(|| {
                crate::marca::sorteia(
                    &mut self.por_traco.rng,
                    &self.spec.dab_jitter,
                    2.0 * radius,
                    self.heading,
                )
            })
            .flatten();
        let center = marca.map_or(disco, |m| [disco[0] + m.desvio[0], disco[1] + m.desvio[1]]);
        if marca.is_some_and(|m| m.meia_volta) {
            rotation = crate::heading::rotate(rotation, [-1.0, 0.0]);
        }
        let particula = if jitter {
            self.spec.dab_jitter.particula(self.spec.spray_count)
        } else {
            1.0
        };
        let radius = if particula < 1.0 {
            (radius * particula).clamp(0.5, MAX_BRUSH_RADIUS_PX)
        } else {
            radius
        };
        let ante = self.dab_ante.map(|mut a| {
            a.salto = (alcance > 0.0).then_some(crate::capsula::Salto {
                desvio: [disco[0] - thrown[0], disco[1] - thrown[1]],
                alcance,
                por_raio: self.spec.jitter_unit == JitterUnit::Brush,
            });
            a
        });
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
            ante,
            tilt: None,
        };
        if let Some(m) = marca
            && m.cobertura != 1.0
        {
            dab.coverage = (dab.coverage * m.cobertura).clamp(0.0, 1.0);
        }
        // A caneta DEITADA (W6) antes da ponta afilada: o taper escala a ponta que a caneta fez. A forma
        // sorteada da marca (L4: redondeza, espelho) entra na MESMA pegada, antes de a caneta a deitar.
        let forma = marca.filter(|_| self.spec.dab_jitter.muda_a_forma());
        match (self.tilt_do_dab(), forma) {
            (Some(t), f) => {
                let t = f.map_or(t, |m| crate::DabTilt {
                    round: m.redondeza,
                    espelho: m.espelho,
                    ..t
                });
                self.deita(&mut dab, t);
            }
            (None, Some(m)) => self.deita(
                &mut dab,
                crate::DabTilt {
                    round: m.redondeza,
                    espelho: m.espelho,
                    ..crate::DabTilt::NEUTRA
                },
            ),
            (None, None) => {}
        }
        let o = self.opacidade_deitada();
        if o < 1.0 {
            dab.coverage *= o;
        }
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
        self.ultimo = Some(Ante::de(&dab));
        dab
    }

    /// Offset the dab centre by a random vector sampled uniformly inside a disc, sized by the
    /// jitter unit: `Brush` → radius `jitter × diameter`; `View` → radius `2 × jitter_absolute_px`
    /// (Blender `BKE_brush_jitter_pos`). No-op for methods that disable jitter (DragDot/Anchored).
    /// Devolve também o raio do disco (`0` = sem Jitter) — o [`crate::capsula::Salto`] do dab.
    pub(super) fn apply_jitter(&mut self, pos: [f32; 2], radius: f32) -> ([f32; 2], f32) {
        if !self.spec.stroke_method.allows_jitter() {
            return (pos, 0.0);
        }
        let max_offset = self.spec.alcance_do_jitter(radius);
        if max_offset <= 0.0 {
            return (pos, 0.0);
        }
        let (dx, dy) = crate::jitter::disc_sample(&mut self.rng);
        (
            [pos[0] + dx * max_offset, pos[1] + dy * max_offset],
            max_offset,
        )
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
            ante: None,
            tilt: None,
        }
    }
}
