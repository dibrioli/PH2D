//! O afim 2D que leva a geometria **local** de um path para o **mundo**.
//!
//! Desde o ADR-0110 cada path é uma entidade, e desde o ADR-0111 essa entidade tem
//! um `Transform` como qualquer sprite. A geometria guardada em [`crate::VecPath`]
//! passa a ser **local**; quem a coloca no mundo é este afim, publicado pela shell
//! uma vez por frame (`parent_world_transform ∘ Transform`).
//!
//! Identidade ⇒ local **é** mundo, que é o estado de todo path recém-criado e o de
//! todos os testes puros. Nada regride quando ninguém transforma nada.
//!
//! O documento não conhece ECS: a shell traduz `Transform` (e a cadeia de pais) num
//! [`Xform`] e entrega o mapa pronto. Aqui só há álgebra.

use crate::VecPathId;
use std::collections::BTreeMap;

/// Afim 2D em coeficientes column-major `[a, b, c, d, e, f]` — o mesmo layout que
/// `kurbo::Affine::new` e `GlobalTransform::affine()` consomem:
///
/// ```text
/// x' = a·x + c·y + e
/// y' = b·x + d·y + f
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Xform(pub [f64; 6]);

impl Default for Xform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xform {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// Leva um PONTO (a translação conta).
    #[must_use]
    pub fn apply(&self, p: [f64; 2]) -> [f64; 2] {
        let [a, b, c, d, e, f] = self.0;
        [a * p[0] + c * p[1] + e, b * p[0] + d * p[1] + f]
    }

    /// Leva um VETOR (um delta de arrasto): só a parte linear, sem translação.
    /// Transladar um delta o transformaria em ponto — o erro clássico.
    #[must_use]
    pub fn apply_vec(&self, v: [f64; 2]) -> [f64; 2] {
        let [a, b, c, d, _, _] = self.0;
        [a * v[0] + c * v[1], b * v[0] + d * v[1]]
    }

    /// `self ∘ other` — aplica `other` primeiro, depois `self`.
    #[must_use]
    pub fn then(&self, outer: &Self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let [a2, b2, c2, d2, e2, f2] = outer.0;
        Self([
            a2 * a + c2 * b,
            b2 * a + d2 * b,
            a2 * c + c2 * d,
            b2 * c + d2 * d,
            a2 * e + c2 * f + e2,
            b2 * e + d2 * f + f2,
        ])
    }

    /// Determinante da parte linear. Zero ⇒ o afim colapsa (escala 0 num eixo) e
    /// **não** tem inverso.
    #[must_use]
    pub fn det(&self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        a * d - b * c
    }

    /// `None` quando o afim é singular — uma forma escalada a zero não tem
    /// world→local, e o chamador deve simplesmente ignorá-la (não é agarrável).
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.det();
        if det.abs() < f64::EPSILON {
            return None;
        }
        let [a, b, c, d, e, f] = self.0;
        let inv = 1.0 / det;
        Some(Self([
            d * inv,
            -b * inv,
            -c * inv,
            a * inv,
            (c * f - d * e) * inv,
            (b * e - a * f) * inv,
        ]))
    }

    /// Fator de escala médio dos dois eixos — o que um raio (o de um gradiente
    /// radial, o de um hit-test em pixels) sofre. Não é exato sob escala
    /// não-uniforme; é a mesma aproximação que `scale_path` já fazia.
    #[must_use]
    pub fn mean_scale(&self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        let sx = (a * a + b * b).sqrt();
        let sy = (c * c + d * d).sqrt();
        (sx + sy) * 0.5
    }

    #[must_use]
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

/// O afim de cada path, publicado pela shell. Ausente ⇒ identidade.
pub type VecXforms = BTreeMap<VecPathId, Xform>;

/// O afim de `id`, ou identidade. É o acesso canônico: um path recém-criado ainda
/// não está no mapa, e isso tem de significar "local = mundo", não um panic.
#[must_use]
pub fn xform_of(xforms: &VecXforms, id: VecPathId) -> Xform {
    xforms.get(&id).copied().unwrap_or(Xform::IDENTITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
    }

    #[test]
    fn identity_leaves_a_point_where_it_is_and_has_itself_as_inverse() {
        let p = [3.0, -7.0];
        assert_eq!(Xform::IDENTITY.apply(p), p);
        assert_eq!(Xform::IDENTITY.inverse().unwrap(), Xform::IDENTITY);
        assert!(Xform::IDENTITY.is_identity());
        assert_eq!(xform_of(&VecXforms::new(), 1), Xform::IDENTITY);
    }

    /// `then` compõe na ordem em que se lê: "escala, DEPOIS translada".
    #[test]
    fn then_applies_self_first_and_the_outer_transform_second() {
        let scale = Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
        let translate = Xform([1.0, 0.0, 0.0, 1.0, 10.0, 0.0]);
        // escala e depois translada: (1,1) -> (2,2) -> (12,2)
        assert!(approx(
            scale.then(&translate).apply([1.0, 1.0]),
            [12.0, 2.0]
        ));
        // translada e depois escala: (1,1) -> (11,1) -> (22,2)
        assert!(approx(
            translate.then(&scale).apply([1.0, 1.0]),
            [22.0, 2.0]
        ));
    }

    #[test]
    fn inverse_round_trips_a_point_through_a_rotated_skewed_affine() {
        let x = Xform([0.6, 0.8, -0.5, 1.1, 4.0, -2.5]);
        let inv = x.inverse().unwrap();
        for p in [[0.0, 0.0], [1.0, 0.0], [-3.5, 7.25]] {
            assert!(approx(inv.apply(x.apply(p)), p));
        }
        assert!((x.then(&inv).0[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_collapsed_affine_has_no_inverse_and_a_uniform_one_reports_its_scale() {
        assert_eq!(Xform([0.0, 0.0, 0.0, 0.0, 1.0, 1.0]).inverse(), None);
        assert_eq!(Xform([3.0, 0.0, 0.0, 3.0, 0.0, 0.0]).mean_scale(), 3.0);
        // Rotação pura não muda a escala.
        let (s, c) = (0.6, 0.8);
        assert!((Xform([c, s, -s, c, 0.0, 0.0]).mean_scale() - 1.0).abs() < 1e-12);
    }
}
