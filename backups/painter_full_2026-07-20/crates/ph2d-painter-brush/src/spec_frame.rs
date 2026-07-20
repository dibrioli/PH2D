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
        self.shape.rake || self.shape.flow || self.texture.rake
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

    /// The dab footprint for a composed per-dab `rotor` ([`Self::dab_rotor`]).
    #[must_use]
    pub fn dab_footprint(&self, rotor: [f32; 2]) -> crate::footprint::FootprintDeform {
        self.footprint_deform().rotated_by(rotor)
    }
}
