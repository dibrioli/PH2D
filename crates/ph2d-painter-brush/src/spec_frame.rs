//! The dab **frame** — [`BrushSpec`]'s stroke-follow rotor and the footprint it orients. Split out of
//! `spec.rs` to keep that file under the workspace LOC cap; it is one responsibility ("which way is this
//! dab pointing?") and, deliberately, one door.

use crate::spec::BrushSpec;

impl BrushSpec {
    /// Whether any slot asks the dab frame to **follow the stroke** — the Shape's Rake or Flow, or the
    /// Grain's Rake. One question, asked once: the tangent orients the dab, not a slot (see
    /// [`Self::dab_rotor`]).
    #[must_use]
    pub fn follows_stroke(&self) -> bool {
        self.shape.rake
            || self.shape.flow
            || self.texture.rake
            || self.pen_tilt.rotation == crate::DabRotation::FollowTrajectory
    }

    /// The rotor that turns the dab frame onto the stroke tangent `dab_dir`, or the identity `[1, 0]`
    /// (bit-exact no-op) when no slot follows. An unset heading (`[0, 0]` — the first dab before the
    /// warm-up settles) also yields the identity, so a stroke never opens at a random angle.
    #[must_use]
    pub fn follow_rotor(&self, dab_dir: [f32; 2]) -> [f32; 2] {
        if self.follows_stroke() {
            crate::texture::normalize_or(dab_dir, [1.0, 0.0])
        } else {
            [1.0, 0.0]
        }
    }

    /// The per-dab rotor applied to the dab **footprint**: the **Jitter Rotate** spin composed with the
    /// **stroke-follow** rotation.
    ///
    /// ⚠️ **This is the one and only place the stroke tangent enters a dab**, and that is the whole
    /// design. Blender carries a single `brush_rotation` and applies it once (to the texture lookup, on a
    /// footprint that is radially symmetric and so has no orientation of its own). We have an elliptical
    /// footprint that Blender does not, so the single rotor is applied to the FRAME every sampler already
    /// reads — falloff, Shape and View-Grain share it, which is what "the deform is brush-wide" has always
    /// meant. Applying the tangent per slot instead is what let a flattened tip and the pattern inside it
    /// disagree on a curve: the flatten stretch sits BETWEEN the two rotations, so they do not commute and
    /// a raked calligraphic nib came out sheared differently at every point of the path.
    #[must_use]
    pub fn dab_rotor(&self, dab: &crate::Dab) -> [f32; 2] {
        crate::heading::rotate(dab.rotation, self.follow_rotor(dab.dir))
    }

    /// ⭐ **A pegada DESTE dab** — a porta de todas as rotas: a do pincel rodada pelo rotor do dab
    /// ([`Self::dab_footprint`]) e, com a caneta deitada ([`crate::Dab::tilt`]), esticada por ela. Sem
    /// caneta deitada é `dab_footprint(dab_rotor(d))` ao bit. O raio do dab já é o eixo maior
    /// ([`Self::tilt_scale`]), então a fracção do menor e o referencial bastam.
    #[must_use]
    pub fn dab_footprint_of(&self, d: &crate::Dab) -> crate::footprint::FootprintDeform {
        Self {
            dab_tilt: d.tilt,
            ..*self
        }
        .dab_footprint(self.dab_rotor(d))
    }

    /// O eixo maior da pegada deitada de `d`, em raios do pincel (`1` sem caneta deitada).
    #[must_use]
    pub fn tilt_scale(&self, d: &crate::Dab) -> f32 {
        match d.tilt {
            Some(t) if t.stretches() => {
                let fp = self.footprint_deform().rotated_by(self.dab_rotor(d));
                t.compose(fp.frame(), fp.minor_fraction()).0
            }
            _ => 1.0,
        }
    }

    /// ⭐ **O pincel DESTE dab** — o raio dele e, com a caneta deitada a alisar, o grão mais raso (W6:
    /// `grain_depth × (1 − smooth)`). Sem caneta é o `BrushSpec { radius_px: d.radius_px, ..*self }` de
    /// sempre, ao bit.
    #[must_use]
    pub fn at_dab(&self, d: &crate::Dab) -> Self {
        let mut s = Self {
            radius_px: d.radius_px,
            dab_tilt: d.tilt,
            ..*self
        };
        if let Some(t) = d.tilt
            && t.smooth > 0.0
        {
            s.grain_depth = self.grain_depth.clamp(0.0, 1.0) * (1.0 - t.smooth);
        }
        // A profundidade do grão por dab (L9): a pressão ou o sorteio deste dab.
        if let Some(t) = d.tilt
            && t.grao != 1.0
        {
            s.grain_depth = s.grain_depth.clamp(0.0, 1.0) * t.grao;
        }
        s
    }

    /// The dab footprint for a composed per-dab `rotor` ([`Self::dab_rotor`]).
    #[must_use]
    ///
    /// ⭐ Com a pegada deitada do dab ([`Self::dab_tilt`], o pincel de [`Self::at_dab`]) ela é esticada pela caneta
    /// e pela forma sorteada da marca — é por aqui que o carimbo a vê; sem ela, ao bit a de sempre.
    pub fn dab_footprint(&self, rotor: [f32; 2]) -> crate::footprint::FootprintDeform {
        let fp = self.footprint_deform().rotated_by(rotor);
        match self.dab_tilt {
            Some(t) if t.stretches() => {
                let (_, frame, minor) = t.compose(fp.frame(), fp.minor_fraction());
                fp.with_frame(frame, minor).espelhada(t.espelho)
            }
            Some(t) => fp.espelhada(t.espelho),
            None => fp,
        }
    }
}
