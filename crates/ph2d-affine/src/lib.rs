#![forbid(unsafe_code)]
//! **O AFIM 2D em `f64`** — a matriz que leva um ponto de um referencial para outro, e nada mais.
//!
//! # Por que esta crate existe (e por que ela nasce vazia de dependências)
//!
//! O [`Xform`] viveu dentro da `ph2d-vec-scene` desde a ADR-0110, e o corpo dele nunca soube o que
//! é um caminho: são seis `f64` e a álgebra deles. A prova de que ele já era foundational-de-facto
//! é o **censo**: `ph2d-field`, `ph2d-field-ecs`, `ph2d-field-eval`, `ph2d-field-render`,
//! `ph2d-flip` e `ph2d-tool-painter` já o consumiam — **seis crates que não têm nada de vectorial**,
//! e que puxavam a cena inteira para multiplicar duas matrizes.
//!
//! O gatilho foi o esqueleto: a lei da pele serve vector, raster, 3D e Flip, e uma lei que
//! precisasse da `ph2d-vec-scene` para compor `repouso⁻¹ ∘ osso ∘ forma⁻¹` deixaria de ser um
//! módulo no primeiro cliente que não desenha Béziers.
//!
//! ⚠️ **A `ph2d-vec-scene` RE-EXPORTA este tipo**, então nenhum dos ~300 ficheiros que escrevem
//! `ph2d_vec_scene::Xform` muda uma letra. Isto é uma mudança de DONO, não de API.

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

    /// ⭐⭐⭐ **O AFIM QUE LEVA UM TRIÂNGULO A OUTRO** — a lei que faz uma imagem entortar.
    ///
    /// Três pontos determinam um afim do plano, exactamente e sem folga: é o que permite desenhar
    /// uma malha deformada como *N* pedaços da MESMA imagem, cada um com o seu afim, sem
    /// pipeline de triângulos texturados nenhum. ⚠️ O resultado é **afim por partes** — exacto em
    /// cada triângulo e contínuo nas arestas, porque dois triângulos vizinhos concordam nos dois
    /// vértices que partilham e um afim é determinado pelos três.
    ///
    /// `None` quando o triângulo de REPOUSO é degenerado (os três pontos colineares): ali não há
    /// afim nenhum que sirva, e inventar um punha a imagem num sítio que ninguém pediu. ⛔ O
    /// triângulo de DESTINO pode ser degenerado à vontade — uma malha esmagada é uma pose legítima.
    #[must_use]
    pub fn from_triangle(rest: [[f64; 2]; 3], to: [[f64; 2]; 3]) -> Option<Self> {
        // As duas arestas de cada triângulo, a partir do vértice 0.
        let (ux, uy) = (rest[1][0] - rest[0][0], rest[1][1] - rest[0][1]);
        let (vx, vy) = (rest[2][0] - rest[0][0], rest[2][1] - rest[0][1]);
        let det = ux.mul_add(vy, -(uy * vx));
        if !det.is_finite() || det == 0.0 {
            return None;
        }
        let (px, py) = (to[1][0] - to[0][0], to[1][1] - to[0][1]);
        let (qx, qy) = (to[2][0] - to[0][0], to[2][1] - to[0][1]);
        // A parte linear é `P · R⁻¹`, com `R = [u v]` e `P = [p q]` em colunas.
        let a = px.mul_add(vy, -(qx * uy)) / det;
        let c = qx.mul_add(ux, -(px * vx)) / det;
        let b = py.mul_add(vy, -(qy * uy)) / det;
        let d = qy.mul_add(ux, -(py * vx)) / det;
        // E a translação é o que falta para o vértice 0 aterrar onde foi pedido.
        let e = to[0][0] - a.mul_add(rest[0][0], c * rest[0][1]);
        let f = to[0][1] - b.mul_add(rest[0][0], d * rest[0][1]);
        Some(Self([a, b, c, d, e, f]))
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

    /// **O FATOR UNIFORME EQUIVALENTE de uma CANETA**, `√|det|` — a média GEOMÉTRICA.
    ///
    /// ⚠️⚠️ **Não é a [`Self::mean_scale`], e a diferença importa**: aquela é a média
    /// ARITMÉTICA `(sx+sy)/2` e serve os comprimentos do CAMINHO (o raio de quina, o gradiente
    /// radial); esta serve os comprimentos da CANETA, e é a lei que o dono do produto escolheu no
    /// bug #27 (*"quando engrossa, engrossa por igual nos dois eixos"*) — *para escala uniforme é a
    /// própria escala, e é invariante à rotação*. Sob `(3, 1)` elas dão `2,000` e `1,732`.
    #[must_use]
    pub fn uniform_scale(&self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        (a * d - b * c).abs().sqrt()
    }

    /// ⭐⭐⭐ **A PARTE CONFORME de um afim** — a mesma rotação (ou reflexão) com a escala
    /// UNIFORMIZADA por [`Self::uniform_scale`], e a translação intacta.
    ///
    /// ⚠️ **Um afim que já é conforme volta AO BIT** — é isso que faz o caminho comum não mudar um
    /// pixel, e há gate. Só a parte que estica desigualmente é removida.
    ///
    /// É a decomposição polar: a rotação mais próxima é `atan2(b − c, a + d)`, e uma reflexão
    /// (`det < 0`) mede-se sobre a matriz já des-espelhada para não virar uma rotação de meia volta.
    #[must_use]
    pub fn uniform_part(&self) -> Self {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        let k = det.abs().sqrt();
        if k <= f64::MIN_POSITIVE {
            return Self([0.0, 0.0, 0.0, 0.0, e, f]);
        }
        let s = if det < 0.0 { -1.0 } else { 1.0 };
        let (sin, cos) = (b - c * s).atan2(a + d * s).sin_cos();
        Self([k * cos, k * sin, -k * sin * s, k * cos * s, e, f])
    }

    #[must_use]
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    /// ⭐⭐⭐ **O AFIM DE UM TRIÂNGULO LEVA OS TRÊS VÉRTICES EXACTAMENTE ONDE FORAM PEDIDOS.**
    ///
    /// ⚠️ É esta exactidão que faz a malha deformada **não abrir costura**: dois triângulos
    /// vizinhos partilham dois vértices, e cada um leva esses dois ao MESMO sítio — logo a aresta
    /// comum é a mesma recta nos dois. *A continuidade não é uma tolerância; é uma consequência.*
    ///
    /// (Mutação: trocar `c` por `-c` na parte linear ⇒ RED nos vértices 1 e 2.)
    #[test]
    fn a_triangle_affine_lands_its_three_corners_exactly() {
        // Um triângulo de repouso qualquer — ⛔ nem rectângulo nem isósceles, senão uma troca de
        // componente na matriz passaria por simetria.
        let rest = [[3.0, 1.0], [11.0, 2.0], [5.0, 9.0]];
        let alvo = [[-2.0, 4.0], [6.5, 1.0], [1.0, 14.0]];
        let m = Xform::from_triangle(rest, alvo).expect("o repouso nao e' degenerado");
        for i in 0..3 {
            let saiu = m.apply(rest[i]);
            assert!(
                (saiu[0] - alvo[i][0]).abs() < 1e-9 && (saiu[1] - alvo[i][1]).abs() < 1e-9,
                "o vertice {i} saiu em {saiu:?} e foi pedido em {:?}",
                alvo[i]
            );
        }
        // ⭐ E a IDENTIDADE sai da identidade: um triângulo que não se move não move a imagem.
        let ident = Xform::from_triangle(rest, rest).expect("idem");
        assert!(
            ident.is_identity(),
            "parado tinha de dar identidade: {ident:?}"
        );
    }

    /// ⛔ **UM REPOUSO COLINEAR NÃO TEM AFIM, e a resposta é `None`.**
    ///
    /// ⚠️ E o DESTINO degenerado é legal: uma malha esmagada contra uma recta é uma pose que o
    /// artista pode pedir, e recusá-la faria o desenho piscar no meio de um gesto.
    #[test]
    fn a_degenerate_rest_has_no_affine_but_a_degenerate_target_is_a_pose() {
        let colinear = [[0.0, 0.0], [2.0, 2.0], [5.0, 5.0]];
        assert_eq!(Xform::from_triangle(colinear, colinear), None);
        let bom = [[0.0, 0.0], [4.0, 0.0], [0.0, 3.0]];
        let esmagado = [[1.0, 1.0], [5.0, 1.0], [3.0, 1.0]];
        let m = Xform::from_triangle(bom, esmagado).expect("o repouso e' bom");
        assert!((m.det()).abs() < 1e-12, "o destino achatado tem det zero");
        assert!((m.apply(bom[2])[1] - 1.0).abs() < 1e-9);
    }

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
