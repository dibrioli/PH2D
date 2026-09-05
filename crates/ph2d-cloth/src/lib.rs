#![forbid(unsafe_code)]
//! **O TECIDO** — o solver do pincel de Cloth, por *Vertex Block Descent*.
//!
//! # O que é, em uma frase
//!
//! Um passo de Euler implícito resolvido por **descida por blocos de VÉRTICE**:
//! para cada vértice, um Newton `3×3` cujo lado direito acumula a inércia mais o
//! gradiente de todo elemento incidente, varrido em Gauss-Seidel por **cor**.
//!
//! ```text
//! Δxᵢ = Hᵢ⁻¹ · fᵢ
//! Hᵢ = (mᵢ/h²)·I  +  Σ_{j∈Fᵢ} ∂²Eⱼ/∂xᵢ²
//! fᵢ = −(mᵢ/h²)·(xᵢ − yᵢ)  −  Σ_{j∈Fᵢ} ∂Eⱼ/∂xᵢ
//! ```
//!
//! Chen, Liu, Yang & Yuksel — *Vertex Block Descent*, ACM TOG (SIGGRAPH 2024),
//! [arXiv:2403.06321](https://arxiv.org/abs/2403.06321). Portado dos papers e das
//! referências **permissivas** (`AnkaChan/Gaia`, Apache-2.0 · `savant117/avbd-demo`,
//! MIT · `alexrodag/spg`, MIT) — triagem em
//! `docs/3D/cloth/01_pesquisa_o_estado_da_arte.md`.
//!
//! # ⚠️ Por que ISTO e não XPBD, que era o que o plano mandava
//!
//! As duas propriedades que um **pincel** precisa são as duas em que o XPBD é
//! fraco, e são fraquezas documentadas pelos autores do método que as corrigiu:
//! ele diverge do Euler implícito sob **passo grande com iterações limitadas**
//! (que é um evento de ponteiro) e sofre sob **razão de massa alta** (que o pincel
//! FABRICA ao pregar o anel de falloff). O VBD é estável com o orçamento truncado
//! porque **cada energia local é garantidamente reduzida**, e a soma das reduções
//! locais *é* a redução global.
//!
//! # ⚠️ O que este módulo NÃO sabe
//!
//! Nada de malha, pincel, câmera ou escultura. Ele recebe triângulos, posições e
//! quem está pregado. *Um solver que soubesse o que é um dab cresceria para dentro
//! do módulo e deixaria de ser gateável sem GPU.*
//!
//! # ⚠️ Aritmética
//!
//! `f64` no miolo, `f32` no armazenamento de quem chama — a mesma decisão do
//! [`ref_kernels`](../../ph2d-sculpt3d/src/ref_kernels.rs) desta casa. Um traço
//! dá centenas de sub-passos sobre as mesmas posições, e `f32` acumula deriva
//! onde a conta é uma soma de correções pequenas.

mod bending;
mod membrane;
mod topology;
mod vbd;

#[cfg(test)]
mod bending_tests;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod membrane_tests;
#[cfg(test)]
mod topology_tests;
#[cfg(test)]
mod vbd_tests;

pub use bending::Hinge;
pub use topology::ClothTopology;
pub use vbd::{ClothDrive, ClothState, StepConfig, step};

/// Um ponto, ou um vetor. `f64` — ver o cabeçalho.
pub type V3 = [f64; 3];

/// **O PANO** — os cinco números que descrevem o material.
///
/// ⚠️ **Nenhum deles é uma constante de solver.** Todos têm unidade física e
/// significado para quem esculpe, e é isso que cumpre o risco nomeado pelo plano
/// (*«Cloth vira um projeto dentro do projeto»* — um solver que pede afinação por
/// cena). O `substeps` e o `iterations` moram no [`StepConfig`], porque eles são
/// **orçamento**, não material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClothMaterial {
    /// Massa por unidade de **área de repouso**.
    pub density: f64,
    /// Módulo de Young da membrana — quanto o pano resiste a esticar.
    pub young: f64,
    /// Coeficiente de Poisson — quanto esticar num eixo encolhe no outro.
    ///
    /// ⚠️ Preso abaixo de `0,5` na porta: em `0,5` o material é incompressível e
    /// o `λ` de Lamé **diverge**. Um knob que chega a um infinito não é um knob.
    pub poisson: f64,
    /// Rigidez de dobra — quanto o pano resiste a **mudar** a curvatura que já tem.
    pub bending: f64,
    /// Amortecimento de Rayleigh, como fração da rigidez.
    pub damping: f64,
}

impl Default for ClothMaterial {
    fn default() -> Self {
        Self {
            density: 1.0,
            young: 1.0e3,
            poisson: 0.3,
            bending: 1.0e-3,
            damping: 0.02,
        }
    }
}

impl ClothMaterial {
    /// Os parâmetros de **Lamé** (`μ`, `λ`) do material.
    ///
    /// ⚠️ O `poisson` é preso em `0,49` aqui e não no setter: a porta é onde a
    /// lei vale para todo chamador, e um campo público não tem setter.
    #[must_use]
    pub fn lame(&self) -> (f64, f64) {
        let nu = self.poisson.clamp(-0.99, 0.49);
        let e = self.young.max(0.0);
        (
            e / (2.0 * (1.0 + nu)),
            e * nu / ((1.0 + nu) * (1.0 - 2.0 * nu)),
        )
    }
}

/// **O REPOUSO** — tudo o que é medido UMA vez, no pen-down, e nunca mais.
///
/// ⚠️⚠️ **É a peça que torna o pincel barato, e ela existe porque o repouso de um
/// traço é congelado por lei** (a `GripLaw::frozen` que os outros verbos já usam).
/// Área, o inverso da forma de repouso de cada triângulo, o ângulo de repouso de
/// cada dobradiça e a massa de cada vértice **não dependem do gesto** — medi-los
/// por evento seria pagar `O(pegada)` a 200 Hz por uma resposta que não muda.
#[derive(Clone, Debug)]
pub struct ClothRest {
    /// Por triângulo: a forma de repouso e a métrica que torna o repouso um zero
    /// AO BIT (ver [`membrane::TriRest`]).
    pub(crate) tri: Vec<membrane::TriRest>,
    /// Por dobradiça: o ângulo de repouso e o peso do *Discrete Shells*.
    pub(crate) hinge: Vec<bending::HingeRest>,
    /// Por vértice: a massa, somada de um terço de cada triângulo incidente.
    pub(crate) mass: Vec<f64>,
}

impl ClothRest {
    /// Mede o repouso a partir da pose em que o traço começou.
    ///
    /// ⚠️ **Um triângulo degenerado não é um erro** — uma escultura tem deles, e
    /// recusar a pegada inteira por causa de um seria o pincel morrer onde a
    /// malha está feia. Ele recebe área zero, e com área zero ele não contribui
    /// gradiente nenhum: *degenerado vira MUDO, nunca `NaN`*.
    #[must_use]
    pub fn measure(topo: &ClothTopology, x: &[V3], mat: &ClothMaterial) -> Self {
        let mut tri = Vec::with_capacity(topo.tris.len());
        let mut mass = vec![0.0; x.len()];
        for t in &topo.tris {
            let r = membrane::rest_of(x, *t);
            for v in *t {
                mass[v as usize] += mat.density * r.area / 3.0;
            }
            tri.push(r);
        }
        let hinge = topo
            .hinges
            .iter()
            .map(|h| bending::HingeRest {
                angle: bending::dihedral(x, *h),
                weight: bending::weight_of(x, *h),
            })
            .collect();
        Self { tri, hinge, mass }
    }

    /// A massa de cada vértice — o que o gate da razão de massa interroga.
    #[must_use]
    pub fn mass(&self) -> &[f64] {
        &self.mass
    }
}

/// **A ENERGIA da pose** — membrana mais dobra, sem inércia.
///
/// ⚠️ **Ela é o ORÁCULO desta crate, e é por isso que é pública.** Todo gradiente
/// aqui é conferido contra **diferenças finitas dela** (`membrane_tests` ·
/// `bending_tests`): a energia é inequívoca e a derivada é onde um sinal trocado
/// passa despercebido. *Um solver cuja derivada não é gateada contra a própria
/// energia é um solver que descobre o erro no smoke.*
#[must_use]
pub fn energy(topo: &ClothTopology, rest: &ClothRest, mat: &ClothMaterial, x: &[V3]) -> f64 {
    let (mu, lambda) = mat.lame();
    let mut e = 0.0;
    for (i, t) in topo.tris.iter().enumerate() {
        e += membrane::energy(x, *t, &rest.tri[i], mu, lambda);
    }
    for (i, h) in topo.hinges.iter().enumerate() {
        // ⚠️ **Pela MESMA porta que o gradiente** (`bending::delta`) — se a
        // energia medisse o salto cru e o gradiente o dobrado, as duas leis
        // discordariam onde o pano fecha sobre si mesmo.
        let d = bending::delta(x, *h, &rest.hinge[i]);
        e += mat.bending * rest.hinge[i].weight * d * d;
    }
    e
}

// ── vetores, em três linhas cada ───────────────────────────────────────────────

pub(crate) fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(crate) fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub(crate) fn norm(a: V3) -> f64 {
    dot(a, a).sqrt()
}
pub(crate) fn scale(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub(crate) fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
