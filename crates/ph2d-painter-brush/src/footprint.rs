//! Brush-dab **flatten + rotate** — the per-pixel footprint deformation shared by the falloff
//! envelope, the Shape silhouette, and the View-mapped Grain, so they flatten + rotate TOGETHER while
//! each slot keeps its own relative Size/Offset/Angle on top (Procreate's Shape panel; Enio
//! 2026-06-26). Transcendental-free (HR-5): the rotation reuses the baked [`crate::texture::rotate_by_degrees`]
//! unit vector, so the result is bit-identical on every platform.

use crate::texture::rotate_by_degrees;

/// Largest **Flatten** value: the minor axis shrinks to `1 - DAB_FLATTEN_MAX` of the major, never to
/// zero (a true line would be degenerate / un-paintable).
pub const DAB_FLATTEN_MAX: f32 = 0.95;

/// Baked flatten + rotate, applied to a footprint-relative unit coord. [`Self::apply`] un-rotates by
/// the dab angle into the ellipse frame, then stretches the minor (post-rotation vertical) axis by
/// `1 / (1 - flatten)`, so a round footprint reads as a rotated ellipse. Identity at flatten 0 / angle 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FootprintDeform {
    cos: f32,
    sin: f32,
    inv_minor: f32,
}

impl FootprintDeform {
    /// The identity (no flatten, no rotation) — byte-identical to the round footprint.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            cos: 1.0,
            sin: 0.0,
            inv_minor: 1.0,
        }
    }

    /// Bake from a `flatten` (`0..1`, clamped to [`DAB_FLATTEN_MAX`]) and a whole-degree dab angle. A
    /// rotation with no flatten still rotates the sampled pattern (the falloff is rotation-invariant).
    #[must_use]
    pub fn new(flatten: f32, angle_deg: u16) -> Self {
        let [cos, sin] = rotate_by_degrees(angle_deg);
        let f = flatten.clamp(0.0, DAB_FLATTEN_MAX);
        Self {
            cos,
            sin,
            inv_minor: 1.0 / (1.0 - f),
        }
    }

    /// Whether this deform changes a footprint at all (any flatten or any rotation) — lets hot paths
    /// skip the transform entirely for a plain round dab.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.inv_minor == 1.0 && self.cos == 1.0 && self.sin == 0.0
    }

    /// Compose an extra rotation `rotor` (a unit vector `[cos, sin]`) onto this deform's frame — the
    /// per-dab **Jitter Rotate** spins the whole footprint (flatten + the falloff/Shape/View-Grain it
    /// drives) by a random angle each dab, on top of the brush's own dab angle. `rotor = [1, 0]` ⇒
    /// unchanged. Transcendental-free (complex multiply of the two rotors); the flatten is preserved.
    #[must_use]
    pub fn rotated_by(self, rotor: [f32; 2]) -> Self {
        Self {
            cos: self.cos * rotor[0] - self.sin * rotor[1],
            sin: self.cos * rotor[1] + self.sin * rotor[0],
            inv_minor: self.inv_minor,
        }
    }

    /// Apply to a footprint unit coord `[u, v]` (pixel offset ÷ radius): rotate by `-angle` into the
    /// ellipse frame, then stretch the minor axis. The falloff reads [`Self::falloff_t`]; the Shape /
    /// Grain samplers feed `apply(f)` into their own Size/rotation/offset.
    #[must_use]
    pub fn apply(self, p: [f32; 2]) -> [f32; 2] {
        if self.is_identity() {
            return p;
        }
        // Rotate by −angle (the transpose of the `(cos, sin)` basis), then stretch the minor axis.
        let ru = p[0] * self.cos + p[1] * self.sin;
        let rv = -p[0] * self.sin + p[1] * self.cos;
        [ru, rv * self.inv_minor]
    }

    /// A fração do eixo MAIOR que o eixo MENOR mede (`1` = redondo, `1 − Flatten` achatado).
    ///
    /// É o número de que a admissibilidade da LUT do filme é feita ([`crate::height_film::FilmLut`]):
    /// o erro da expansão escala com a **CURVATURA** da silhueta, e a curvatura é governada pelo menor
    /// raio local — que num bico achatado é `raio × minor`, não `raio`. Medido: uma elipse de
    /// `minor = 0,45` erra **6×** a redonda no mesmo raio, e `1/0,45² = 4,9`.
    #[must_use]
    pub fn minor_fraction(self) -> f32 {
        1.0 / self.inv_minor
    }

    /// The deformed radial distance `length(apply([u, v]))` — the falloff index for an elliptical dab.
    /// At identity this is the plain `sqrt(u² + v²)`. Rotation alone preserves it (a circle is
    /// rotation-invariant); only the flatten makes it elliptical.
    #[must_use]
    pub fn falloff_t(self, u: f32, v: f32) -> f32 {
        let d = self.apply([u, v]);
        (d[0] * d[0] + d[1] * d[1]).sqrt()
    }
}
