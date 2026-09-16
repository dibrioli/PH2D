//! ⭐⭐⭐⭐ **COMO OS PESOS GUARDADOS ATRAVESSAM O REFINAMENTO DO `Smooth`** — a porta única que o
//! quadro vivo, os fantasmas do onion e os gates chamam.
//!
//! # O defeito que a trouxe (smoke do dono, 2026-09-16, foto com cinco setas)
//!
//! > *«Smooth parece ter resultado discretamente inferior, gerando micro irregularidades»*
//!
//! ⛔⛔ **A lei de refinamento estava certa, e era esse o problema:** ela segue o campo de
//! deformação com fidelidade, e o campo tinha um vinco em cada aresta da malha do bind — os pesos
//! eram lidos em linha recta dentro de cada triângulo (P1). O `Fast` saltava os vincos por
//! desenhar só as cordas; o `Smooth` desenhava-os. Mecanismo e tabela:
//! [`ph2d_poly2d::AttrLaw`] (o cabeçalho de `attr_law.rs`).
//!
//! ⭐ **A cura mora na LEI dos atributos, não na lei de refinamento nem na pele:** os pesos
//! viajam com o gradiente recuperado de cada vértice e nascem no meio de uma aresta pela cúbica de
//! Hermite. *Trocar o refinador escondia a causa; trocar a pele mudava a arte no `Fast` também.*
//!
//! # ⚠️ Duas portas de bissecção, independentes
//!
//! - `PH2D_SKIN_REFINE=uniforme` — a lei de refinamento de antes ([`crate::skin_budget`]);
//! - `PH2D_SKIN_WEIGHTS=linear` — a lei dos pesos de antes (esta folha).
//!
//! *Um caminho antigo sem porta não se mede contra o novo* — e com as duas é possível perguntar
//! qual das mudanças de 2026-09-16 mexeu num report.

use std::borrow::Cow;

use ph2d_poly2d::{AttrLaw, DeformAttrs, Mesh2d, RefineOptions, RefineReport};

/// ⭐ **Os pesos são lidos pela lei de antes (P1)?** — `PH2D_SKIN_WEIGHTS=linear`, lido uma vez.
fn pesos_lineares() -> bool {
    static LINEAR: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LINEAR.get_or_init(|| {
        std::env::var("PH2D_SKIN_WEIGHTS").is_ok_and(|v| v.trim().eq_ignore_ascii_case("linear"))
    })
}

/// ⭐⭐ **A LEI com que os pesos guardados atravessam o refinamento.**
///
/// ⚠️ **Hermite só na lei ADAPTATIVA** — a uniforme é a porta de bissecção do caminho de antes e
/// lê os atributos em linha recta por declaração ([`ph2d_poly2d::refine_posed_with`]); dar-lhe o
/// formato de Hermite só lhe acrescentaria gradientes que ela não lê.
#[must_use]
pub fn weight_law(ossos: usize, adaptativo: bool) -> AttrLaw {
    if ossos == 0 || !adaptativo || pesos_lineares() {
        AttrLaw::Linear
    } else {
        AttrLaw::Hermite { values: ossos }
    }
}

/// ⭐ **Os pesos no formato da LEI** — emprestados quando ela é linear, com os gradientes
/// recuperados atrás quando é Hermite.
#[must_use]
pub fn weight_attrs<'a>(mesh: &Mesh2d, pesos: &'a [f64], law: AttrLaw) -> Cow<'a, [f64]> {
    match law {
        AttrLaw::Linear => Cow::Borrowed(pesos),
        AttrLaw::Hermite { values } => Cow::Owned(ph2d_poly2d::hermite_attrs(mesh, pesos, values)),
    }
}

/// O que o refinamento de uma imagem presa devolve.
pub struct SkinnedRefine {
    /// A malha refinada, em pixels da imagem.
    pub mesh: Mesh2d,
    /// As posições deformadas de cada vértice dela.
    pub posed: Vec<[f64; 2]>,
    /// Os atributos de cada vértice, **no formato da [`Self::law`]** (os pesos vêm primeiro).
    pub attrs: Vec<f64>,
    /// A lei com que os atributos foram lidos — ver [`weight_law`].
    pub law: AttrLaw,
    /// O que o refinamento fez.
    pub report: RefineReport,
}

#[cfg(test)]
thread_local! {
    /// ⏱️ **Quantas vezes o refinamento correu nesta thread** — o instrumento dos gates de CUSTO
    /// (um relógio aqui seria uma flake; a pergunta *«a lei correu?»* é uma contagem).
    pub(crate) static REFINAMENTOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// ⭐⭐⭐ **A MALHA DE UMA IMAGEM PRESA, REFINADA PELO `Smooth`.**
///
/// `pesos[v · ossos + j]` é a tabela guardada no bind (vazia ⇒ `ossos == 0` e a lei derivada), e
/// `campo(ponto, pesos)` deforma um ponto de repouso — ⚠️ ele recebe **só os pesos**, nunca os
/// gradientes que viajam atrás deles.
pub fn refine_skinned(
    mesh: &Mesh2d,
    pesos: &[f64],
    ossos: usize,
    campo: &mut DeformAttrs<'_>,
    opts: RefineOptions,
) -> SkinnedRefine {
    #[cfg(test)]
    REFINAMENTOS.with(|c| c.set(c.get() + 1));
    let law = weight_law(ossos, opts.adaptativo);
    let attrs = weight_attrs(mesh, pesos, law);
    let stride = attrs.len().checked_div(mesh.rest.len()).unwrap_or(0);
    let mut so_pesos = |q: [f64; 2], w: &[f64]| campo(q, w.get(..ossos).unwrap_or(w));
    let (mesh, posed, attrs, report) =
        ph2d_poly2d::refine_posed_with(mesh, &attrs, stride, law, &mut so_pesos, opts);
    SkinnedRefine {
        mesh,
        posed,
        attrs,
        law,
        report,
    }
}

/// ⭐⭐ **Quanto uma malha posada erra o campo que o `Smooth` SEGUE** — o maior desvio nos meios das
/// arestas, com os pesos lidos pela `law` de [`weight_law`].
///
/// ⚠️ `attrs` vem no formato da `law` ([`weight_attrs`], ou o [`SkinnedRefine::attrs`]).
pub fn skinned_deviation(
    mesh: &Mesh2d,
    posed: &[[f64; 2]],
    attrs: &[f64],
    law: AttrLaw,
    campo: &mut DeformAttrs<'_>,
) -> f64 {
    let ossos = match law {
        AttrLaw::Linear => usize::MAX,
        AttrLaw::Hermite { values } => values,
    };
    let stride = attrs.len().checked_div(mesh.rest.len()).unwrap_or(0);
    let mut so_pesos = |q: [f64; 2], w: &[f64]| campo(q, w.get(..ossos).unwrap_or(w));
    ph2d_poly2d::deviation_with(mesh, posed, attrs, stride, law, &mut so_pesos)
}
