//! ⭐⭐⭐⭐ **COMO UM ATRIBUTO NASCE NO MEIO DE UMA ARESTA** — a lei que o refinamento usa para dar
//! valores a um vértice que ele inventa.
//!
//! # O defeito que a trouxe (smoke do dono, 2026-09-16, com foto e cinco setas)
//!
//! > *«Smooth parece ter resultado discretamente inferior, gerando micro irregularidades»*
//!
//! O refinamento adaptativo seguia o campo **fielmente** — e o campo era o defeito. Os pesos de
//! pele são guardados **nos vértices** da malha do bind e eram interpolados em **linha recta**
//! dentro de cada triângulo (P1). Um peso P1 tem gradiente constante por triângulo e **salta** em
//! cada aresta, logo a pele deformada tem um VINCO em cada aresta da malha e um ARCO no meio de
//! cada triângulo — e os dois viram para lados opostos. Medido no lado de cima da arte da cena
//! (`PH2D_VEC_BONE_PAINT_SMOKE`), a rotação da tangente que vai para um lado e volta:
//!
//! | o que se desenha | vai-e-volta | maior vinco |
//! |---|---:|---:|
//! | `Fast` (as cordas entre os vértices do bind) | `26,6°` | `3,57°` |
//! | o campo P1, amostrado denso | **`177,4°`** | `4,58°` |
//! | `Smooth` a seguir o campo P1, zoom `8×` | **`47,0°`** | `3,10°` |
//! | o campo com **esta** lei, amostrado denso | `39,7°` | **`0,23°`** |
//!
//! ⛔⛔ **O `Fast` era mais liso POR ACIDENTE:** as cordas dele saltam os meandros do P1. Os
//! `26,6°` que sobram nele não são ruído — são a curva em S real do esqueleto (as pontas de uma
//! pele misturada curvam ao contrário), e a tabela vértice a vértice mostra-os a variar suavemente.
//!
//! # ⭐ A lei: Hermite cúbico sobre gradientes RECUPERADOS
//!
//! Cada vértice passa a levar, além do valor, o **gradiente** dele — a média dos gradientes dos
//! triângulos vizinhos, pesada pela área ([`recover_gradients`]). No meio de uma aresta `a → b` o
//! valor é o da cúbica de Hermite que passa pelas duas pontas com essas derivadas:
//!
//! ```text
//! w(½)  = (w_a + w_b)/2 + (d_a − d_b)/8                  d = ∇w · (x_b − x_a)
//! ∇w(½) = (∇w_a + ∇w_b)/2 + [1,5·(w_b − w_a) − 0,75·(d_a + d_b)] · (x_b − x_a)/|x_b − x_a|²
//! ```
//!
//! A segunda linha é o que deixa a lei **descer níveis**: o vértice novo nasce com o gradiente que
//! a cúbica tem ali (a componente ao longo da aresta) e a média das pontas na transversal.
//!
//! ⭐⭐ **Três propriedades saem de graça, e cada uma tem gate:**
//! - **exacta num campo LINEAR** (`d_a = d_b = w_b − w_a` ⇒ a correcção é zero e o gradiente fica);
//! - **exacta num campo QUADRÁTICO** ao longo da aresta (é o que uma cúbica de Hermite é);
//! - ⭐ **a partição da unidade é EXACTA sem renormalizar** — a lei é LINEAR nos valores e nos
//!   gradientes, os pesos somam `1` e os gradientes recuperados somam `0` (medido: `|Σ − 1| ≤ 4e-16`).
//!   ⛔ Por isso **não há corte em zero**: o Hermite passa ligeiramente abaixo (`−8,6e-5` no pior
//!   ponto), a pele mistura por soma afim e aguenta-o, e um `max(0, ·)` poria de volta exactamente os
//!   vincos que esta lei existe para tirar.

use crate::Mesh2d;

/// ⭐ **Como os atributos por vértice são lidos pelo refinamento.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttrLaw {
    /// Cada componente interpolada em linha recta — a lei P1 de sempre.
    #[default]
    Linear,
    /// ⭐ Os primeiros `values` componentes são VALORES e os `2 × values` seguintes são os
    /// gradientes deles (`[gx, gy]` por valor) — o formato que o [`hermite_attrs`] produz. O meio
    /// de uma aresta nasce da cúbica de Hermite (ver o cabeçalho do módulo).
    Hermite {
        /// Quantos valores há por vértice (o `stride` é `3 ×` isto).
        values: usize,
    },
}

/// ⭐⭐ **O GRADIENTE de cada valor em cada vértice** — a média dos gradientes dos triângulos
/// vizinhos, pesada pela área. Devolve `[gx, gy]` achatado: `out[(v · stride + k) · 2 + eixo]`.
///
/// ⚠️ **Num campo LINEAR ele é exacto em todo vértice**, interior ou de bordo — cada triângulo tem o
/// mesmo gradiente e a média de iguais é ele. É essa propriedade que faz a lei de Hermite não mexer
/// num campo que o P1 já representava bem.
#[must_use]
pub fn recover_gradients(mesh: &Mesh2d, values: &[f64], stride: usize) -> Vec<f64> {
    let n = mesh.rest.len();
    let mut soma = vec![0.0_f64; n * stride * 2];
    let mut area = vec![0.0_f64; n];
    if stride == 0 {
        return soma;
    }
    let val = |v: usize, k: usize| values.get(v * stride + k).copied().unwrap_or(0.0);
    for t in &mesh.tris {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (Some(&pa), Some(&pb), Some(&pc)) =
            (mesh.rest.get(a), mesh.rest.get(b), mesh.rest.get(c))
        else {
            continue;
        };
        let (e1, e2) = (
            [pb[0] - pa[0], pb[1] - pa[1]],
            [pc[0] - pa[0], pc[1] - pa[1]],
        );
        let det = e1[0] * e2[1] - e1[1] * e2[0];
        if det == 0.0 || det.is_nan() {
            continue;
        }
        let ar = det.abs() / 2.0;
        for k in 0..stride {
            let (d1, d2) = (val(b, k) - val(a, k), val(c, k) - val(a, k));
            let g = [
                (d1 * e2[1] - d2 * e1[1]) / det,
                (d2 * e1[0] - d1 * e2[0]) / det,
            ];
            for v in [a, b, c] {
                soma[(v * stride + k) * 2] += g[0] * ar;
                soma[(v * stride + k) * 2 + 1] += g[1] * ar;
            }
        }
        for v in [a, b, c] {
            area[v] += ar;
        }
    }
    for (v, &ar) in area.iter().enumerate() {
        if ar > 0.0 {
            for x in &mut soma[v * stride * 2..(v + 1) * stride * 2] {
                *x /= ar;
            }
        }
    }
    soma
}

/// ⭐ **Os atributos no formato do [`AttrLaw::Hermite`]** — valores e, a seguir, os gradientes
/// recuperados deles, vértice a vértice.
#[must_use]
pub fn hermite_attrs(mesh: &Mesh2d, values: &[f64], stride: usize) -> Vec<f64> {
    let g = recover_gradients(mesh, values, stride);
    let mut out = Vec::with_capacity(mesh.rest.len() * stride * 3);
    for v in 0..mesh.rest.len() {
        for k in 0..stride {
            out.push(values.get(v * stride + k).copied().unwrap_or(0.0));
        }
        out.extend_from_slice(&g[v * stride * 2..(v + 1) * stride * 2]);
    }
    out
}

/// ⭐⭐⭐ **OS ATRIBUTOS DO MEIO DE UMA ARESTA** — a porta ÚNICA que as duas leis de refinamento e
/// a régua do desvio chamam, para as três medirem o mesmo campo.
///
/// `ra`/`rb` são as posições de repouso das pontas, `a`/`b` os atributos delas, e `out` recebe os
/// do meio (o mesmo comprimento).
pub(crate) fn midpoint(
    law: AttrLaw,
    ra: [f64; 2],
    rb: [f64; 2],
    a: &[f64],
    b: &[f64],
    out: &mut [f64],
) {
    let at = |s: &[f64], i: usize| s.get(i).copied().unwrap_or(0.0);
    match law {
        AttrLaw::Linear => {
            for (c, o) in out.iter_mut().enumerate() {
                *o = f64::midpoint(at(a, c), at(b, c));
            }
        }
        AttrLaw::Hermite { values } => {
            let dx = [rb[0] - ra[0], rb[1] - ra[1]];
            let l2 = dx[0] * dx[0] + dx[1] * dx[1];
            for k in 0..values {
                let (wa, wb) = (at(a, k), at(b, k));
                let (gi, gj) = (values + 2 * k, values + 2 * k + 1);
                let (ga, gb) = ([at(a, gi), at(a, gj)], [at(b, gi), at(b, gj)]);
                let (da, db) = (ga[0] * dx[0] + ga[1] * dx[1], gb[0] * dx[0] + gb[1] * dx[1]);
                if let Some(o) = out.get_mut(k) {
                    *o = f64::midpoint(wa, wb) + (da - db) / 8.0;
                }
                // A componente AO LONGO da aresta vem da cúbica; a transversal fica a média.
                let corr = if l2 > 0.0 {
                    (1.5 * (wb - wa) - 0.75 * (da + db)) / l2
                } else {
                    0.0
                };
                if let Some(o) = out.get_mut(gi) {
                    *o = f64::midpoint(ga[0], gb[0]) + corr * dx[0];
                }
                if let Some(o) = out.get_mut(gj) {
                    *o = f64::midpoint(ga[1], gb[1]) + corr * dx[1];
                }
            }
        }
    }
}
