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

    /// ⭐⭐⭐ **O CONTORNO do dab** — o ponto da fronteira da pegada no parâmetro `t ∈ [0, 1)`, em
    /// unidades de RAIO.
    ///
    /// ⛔⛔ **Ele existe para o anel do cursor não reconstruir a elipse por fora** (item 1 da fila do
    /// esqueleto, 2026-09-14). O anel desenhava `(cos θ, m·sin θ)` rodado pelo rotor vivo — a mesma
    /// conta, escrita noutra crate — e por isso continuava a mostrar a forma de REPOUSO quando a
    /// pegada passou a carregar a deformação da arte. *Uma lei escrita em dois sítios ainda não é
    /// uma lei.*
    ///
    /// ⭐ **E a amarra é demonstrável, não prometida:** por construção `falloff_t` deste ponto é
    /// exactamente `1` — a fronteira da pegada É a curva de nível que o amostrador usa. Há gate.
    #[must_use]
    pub fn outline_at(self, t: f32) -> [f32; 2] {
        let (sen, cos) = (t * std::f32::consts::TAU).sin_cos();
        let menor = sen * self.minor_fraction();
        [
            cos * self.cos - menor * self.sin,
            cos * self.sin + menor * self.cos,
        ]
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

#[cfg(test)]
mod outline_tests {
    use super::*;

    /// ⭐⭐⭐ **O CONTORNO É A CURVA DE NÍVEL DO AMOSTRADOR** — `falloff_t` da fronteira é `1`.
    ///
    /// ⛔ É esta a amarra que impede o anel do cursor de desenhar uma elipse que a tinta não pinta:
    /// o anel percorre [`FootprintDeform::outline_at`] e o motor lê [`FootprintDeform::falloff_t`],
    /// e as duas só podem discordar se esta asserção cair.
    ///
    /// ⚠️ O corpus varre achatamento **e** ângulo: com `flatten = 0` a elipse é um círculo e
    /// qualquer contorno passa — *um corpus redondo não mede a forma*.
    #[test]
    fn the_outline_is_the_sampler_level_set() {
        let mut casos = 0;
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 90, 233] {
                let fp = FootprintDeform::new(flatten, angle);
                for k in 0..64 {
                    let p = fp.outline_at(k as f32 / 64.0);
                    let t = fp.falloff_t(p[0], p[1]);
                    casos += 1;
                    assert!(
                        (t - 1.0).abs() < 1e-5,
                        "com flatten {flatten} e ângulo {angle}°, o contorno em {k}/64 tem \
                         falloff {t} — ele não é a fronteira que o amostrador usa"
                    );
                }
            }
        }
        assert_eq!(casos, 4 * 4 * 64, "o corpus mudou de tamanho");
    }

    /// ⭐ **E a rotação do dab MOVE o contorno** — a metade anti-vácuo: sem ela a asserção de cima
    /// passa sobre um contorno que ignorasse o ângulo (num círculo tudo é fronteira).
    #[test]
    fn the_outline_turns_with_the_dab() {
        let reto = FootprintDeform::new(0.5, 0);
        let torto = FootprintDeform::new(0.5, 90);
        let a = reto.outline_at(0.0);
        let b = torto.outline_at(0.0);
        let d = (a[0] - b[0]).hypot(a[1] - b[1]);
        assert!(
            d > 0.5,
            "o contorno não roda com o ângulo do dab (desvio {d}): ele está a ignorar a orientação"
        );
    }
}
