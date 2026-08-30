//! ⭐⭐⭐ **O DESEMARANHADOR** — tornar um mapa localmente injectivo **a partir de um estado
//! já dobrado**.
//!
//! ⛔⛔ **A medição que o nomeou** (2026-08-30, peça do artista): o mapa da cadeia de quads
//! dobra `3,12 %` dos triângulos no **ombro** de um espinho contra `0,14 %` no corpo — `23×`
//! — e é isso que a extracção emite como face a `177°` e gravata. Ver
//! `docs/3D/quad-remesh/PLANO_desdobrar_o_mapa.md`.
//!
//! # ⭐ Porque é ESTA formulação, e não uma das outras duas
//!
//! | família | veredito da literatura |
//! |---|---|
//! | *local stiffening* | sem garantia, e falha *«especially for large target edge lengths»* — o nosso alvo de fábrica é grande |
//! | restrições anti-flip lineares | linearização; o espaço pode ficar inviável, e o *branch-and-bound* leva dias |
//! | ⭐ **barreira regularizada** | passa `100 %` de um *benchmark* público de `10 743 + 904` casos, e **funciona a partir de um emaranhado** |
//!
//! ⚠️ **A última linha é a que decide para nós:** não temos partida livre de dobras — temos
//! dobras. Quase toda a família da injectividade exige um estado inicial válido.
//!
//! # A lei, e o truque é uma linha
//!
//! A energia por elemento, pesada pela área de repouso, com `λ` a trocar ângulo por área:
//!
//! ```text
//! f(J) = tr(JᵀJ) / det J        g(J) = (det²J + 1) / det J
//! F(U) = Σ_t ( f(J_t) + λ·g(J_t) ) · área(T_t)
//! ```
//!
//! ⭐ **Todo `det J` num denominador é substituído por**
//!
//! ```text
//! χ(D, ε) = ( D + √(ε² + D²) ) / 2
//! ```
//!
//! `χ` é suave e **estritamente positiva** para qualquer `D` real enquanto `ε > 0`; quando
//! `ε → 0⁺` ela tende a `D` para `D > 0` e a `0⁺` para `D < 0`. ⇒ **a energia é finita e
//! derivável sobre uma malha emaranhada** e só se torna infinita sobre a dobra à medida que
//! `ε` encolhe. *É isto, e só isto, que permite partir de onde estamos.*
//!
//! ⚠️ **Clean-room da literatura pública.** Nenhuma linha traduzida de fonte de alvo restrito.

#![forbid(unsafe_code)]

mod solve;

pub use solve::{Report, Settings, untangle};

/// Um triângulo do domínio, com o referencial de repouso já invertido.
///
/// ⚠️ **O repouso entra INVERTIDO de propósito.** Ele é constante ao longo de toda a
/// optimização, e inverter uma `2×2` por elemento em cada avaliação da energia seria pagar
/// milhares de vezes por um número que não muda. ⭐ [`Element::from_rest`] faz a inversão uma
/// vez, e devolve `None` sobre um triângulo de repouso degenerado — *que é uma entrada
/// inválida, não um caso a tratar com um epsilon.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Element {
    /// Os três cantos, como índices no vector de coordenadas.
    pub verts: [u32; 3],
    /// `[[a,b],[c,d]]` — a inversa da matriz cujas colunas são `p1−p0` e `p2−p0`.
    pub rest_inv: [[f64; 2]; 2],
    /// A área de repouso — o peso do elemento na soma.
    pub area: f64,
}

impl Element {
    /// Constrói o elemento a partir do triângulo de repouso **já achatado em 2D**.
    ///
    /// ⚠️ **O achatamento é do chamador**, e é uma decisão dele: para um triângulo 3D o
    /// referencial natural é o **isométrico** (`p0` na origem, `p1` no eixo `x`), que preserva
    /// os comprimentos das arestas. *Um achatamento que já distorce faz a energia medir a
    /// distorção do achatamento em vez da do mapa.*
    #[must_use]
    pub fn from_rest(verts: [u32; 3], p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> Option<Self> {
        let (a, b) = (p1[0] - p0[0], p2[0] - p0[0]);
        let (c, d) = (p1[1] - p0[1], p2[1] - p0[1]);
        let det = a.mul_add(d, -(b * c));
        if !det.is_finite() || det.abs() < f64::MIN_POSITIVE {
            return None;
        }
        Some(Self {
            verts,
            rest_inv: [[d / det, -b / det], [-c / det, a / det]],
            area: det.abs() / 2.0,
        })
    }

    /// A jacobiana `J = E · S⁻¹`, onde as colunas de `E` são `u1−u0` e `u2−u0`.
    #[must_use]
    fn jacobian(&self, uv: &[[f64; 2]]) -> [[f64; 2]; 2] {
        let (u0, u1, u2) = (
            uv[self.verts[0] as usize],
            uv[self.verts[1] as usize],
            uv[self.verts[2] as usize],
        );
        let e = [
            [u1[0] - u0[0], u2[0] - u0[0]],
            [u1[1] - u0[1], u2[1] - u0[1]],
        ];
        let s = self.rest_inv;
        [
            [
                e[0][0].mul_add(s[0][0], e[0][1] * s[1][0]),
                e[0][0].mul_add(s[0][1], e[0][1] * s[1][1]),
            ],
            [
                e[1][0].mul_add(s[0][0], e[1][1] * s[1][0]),
                e[1][0].mul_add(s[0][1], e[1][1] * s[1][1]),
            ],
        ]
    }
}

/// ⭐ **A REGULARIZAÇÃO** — `χ(D, ε) = (D + √(ε² + D²)) / 2`.
///
/// ⚠️ **Estritamente positiva para todo `D` real enquanto `ε > 0`**, e é isso que torna a
/// energia finita sobre um elemento invertido. Com `ε = 0` ela devolve `D` para `D > 0` e `0`
/// para `D ≤ 0` — o caso limite, e o único em que a energia pode explodir.
#[must_use]
#[inline]
pub fn chi(d: f64, eps: f64) -> f64 {
    (d + eps.mul_add(eps, d * d).sqrt()) / 2.0
}

/// A derivada de [`chi`] em ordem a `D`.
#[must_use]
#[inline]
fn dchi(d: f64, eps: f64) -> f64 {
    let r = eps.mul_add(eps, d * d).sqrt();
    if r <= 0.0 {
        // ⚠️ `D = 0` **e** `ε = 0` — a derivada não existe. Devolver `½` é o limite pela
        // direita, e o laço nunca lá chega porque `ε` é sempre `> 0` durante a descida.
        return 0.5;
    }
    (1.0 + d / r) / 2.0
}

/// **O determinante mínimo sobre os elementos** — a régua que diz se ainda há dobra.
///
/// ⛔ Devolve `f64::INFINITY` sobre uma lista vazia: *«não medido» não pode ler-se como
/// «positivo»*, e um `0.0` ali passaria por dobra.
#[must_use]
pub fn min_det(elements: &[Element], uv: &[[f64; 2]]) -> f64 {
    elements.iter().fold(f64::INFINITY, |acc, e| {
        let j = e.jacobian(uv);
        acc.min(j[0][0].mul_add(j[1][1], -(j[0][1] * j[1][0])))
    })
}

/// Quantos elementos estão invertidos (`det J ≤ 0`).
#[must_use]
pub fn flipped(elements: &[Element], uv: &[[f64; 2]]) -> usize {
    elements
        .iter()
        .filter(|e| {
            let j = e.jacobian(uv);
            j[0][0].mul_add(j[1][1], -(j[0][1] * j[1][0])) <= 0.0
        })
        .count()
}

/// **A energia e o gradiente**, numa passagem só.
///
/// ⚠️ **Uma passagem e não duas**, porque toda a maquinaria por elemento (`J`, `det`, `χ`) é
/// partilhada pelas duas respostas — e uma segunda travessia seria a mesma conta com o dobro do
/// relógio e uma segunda oportunidade de divergir da primeira.
pub(crate) fn energy_and_gradient(
    elements: &[Element],
    uv: &[[f64; 2]],
    eps: f64,
    lambda: f64,
    grad: &mut [[f64; 2]],
) -> f64 {
    for g in grad.iter_mut() {
        *g = [0.0, 0.0];
    }
    let mut total = 0.0;
    for el in elements {
        let j = el.jacobian(uv);
        let det = j[0][0].mul_add(j[1][1], -(j[0][1] * j[1][0]));
        let a = j[0][0].mul_add(j[0][0], j[0][1] * j[0][1])
            + j[1][0].mul_add(j[1][0], j[1][1] * j[1][1]);
        let c = chi(det, eps);
        // ⚠️ `χ > 0` por construção enquanto `ε > 0`; a guarda existe para o caso limite e
        // devolve uma energia enorme em vez de `inf`/`NaN`, que envenenaria a busca linear.
        if !c.is_finite() || c <= 0.0 {
            return f64::MAX / 4.0;
        }
        let f = a / c;
        let g = det.mul_add(det, 1.0) / c;
        total += (f + lambda * g) * el.area;

        // ∂/∂J, pela regra do quociente. `∂det/∂J` é a matriz dos cofactores.
        let dc = dchi(det, eps);
        let ddet = [[j[1][1], -j[1][0]], [-j[0][1], j[0][0]]];
        let kf = -a / (c * c) * dc;
        let kg = (2.0 * det / c) - (det.mul_add(det, 1.0) / (c * c)) * dc;
        let mut dj = [[0.0f64; 2]; 2];
        for r in 0..2 {
            for k in 0..2 {
                dj[r][k] =
                    (2.0 * j[r][k] / c + kf * ddet[r][k] + lambda * kg * ddet[r][k]) * el.area;
            }
        }
        // ∂/∂E = ∂/∂J · Sᵀ
        let s = el.rest_inv;
        let mut de = [[0.0f64; 2]; 2];
        for r in 0..2 {
            for k in 0..2 {
                de[r][k] = dj[r][0].mul_add(s[k][0], dj[r][1] * s[k][1]);
            }
        }
        // As colunas de `E` são `u1−u0` e `u2−u0`.
        let (i0, i1, i2) = (
            el.verts[0] as usize,
            el.verts[1] as usize,
            el.verts[2] as usize,
        );
        for r in 0..2 {
            grad[i1][r] += de[r][0];
            grad[i2][r] += de[r][1];
            grad[i0][r] -= de[r][0] + de[r][1];
        }
    }
    total
}

/// A energia sozinha — o que a busca linear precisa.
pub(crate) fn energy(elements: &[Element], uv: &[[f64; 2]], eps: f64, lambda: f64) -> f64 {
    let mut total = 0.0;
    for el in elements {
        let j = el.jacobian(uv);
        let det = j[0][0].mul_add(j[1][1], -(j[0][1] * j[1][0]));
        let a = j[0][0].mul_add(j[0][0], j[0][1] * j[0][1])
            + j[1][0].mul_add(j[1][0], j[1][1] * j[1][1]);
        let c = chi(det, eps);
        if !c.is_finite() || c <= 0.0 {
            return f64::MAX / 4.0;
        }
        total += (a / c + lambda * (det.mul_add(det, 1.0) / c)) * el.area;
    }
    total
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
