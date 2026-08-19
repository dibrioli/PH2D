//! **QUAD REMESH POR CAMPO CRUZADO** — a retopologia do módulo de escultura 3D.
//!
//! Porte **nativo** do *QuadriFlow* (Huang, Zhou, Nießner, Shewchuk, Guibas,
//! SGP 2018), que é o *Instant Field-Aligned Meshes* (Jakob, Tarini, Panozzo,
//! Sorkine-Hornung, SIGGRAPH Asia 2015) mais um passo global de consistência.
//! Racional, alternativas rejeitadas e o conjunto de aceitação congelado:
//! **ADR-0160**.
//!
//! ⚠️ **As duas referências são permissivas** (BSD-3-Clause / MIT), então — ao
//! contrário do Blender, que é GPL e de quem só se pode descrever comportamento
//! — aqui a **citação de fonte é permitida**, e os doc-comments a usam.
//!
//! # Por que esta crate existe se a [`ph2d_sdf`](../ph2d_sdf/index.html) já devolve quads
//!
//! Porque as duas respondem a perguntas diferentes, e o ADR-0160 §1 as mede lado
//! a lado. O `surface_nets` re-amostra um **campo de voxels**: os quads dele se
//! alinham aos eixos da grade, uma feição diagonal sai em escada, e uma alça mais
//! fina que o voxel desaparece. É o que se quer de uma **arrumação destrutiva**
//! (depois de uma booleana, depois de o barro se auto-intersectar).
//!
//! O campo cruzado faz o contrário: a grade se alinha às **direções principais da
//! forma**, a topologia da entrada é **preservada**, e a densidade **segue a
//! curvatura**. É o que se quer de uma **retopologia** — a malha que se
//! subdivide, se anima e se edita.
//!
//! # O pipeline, e onde esta wave está
//!
//! | passo | o que é | estado |
//! |---|---|---|
//! | **1. orientação** | um campo 4-RoSy por vértice, suavizado | ✅ [`orientation`] |
//! | 2. posição | a retícula local + a escala adaptativa | ⏳ Q2 |
//! | 3. extração | a malha a partir dos dois campos | ⏳ Q3 |
//! | 4. consistência | o fluxo de custo mínimo (o passo do QuadriFlow) | ⏳ Q4 |
//!
//! ⚠️ **Nada aqui é alcançável pelo produto ainda** — a costura no shell é a Q5.
//! Uma crate que o app não chama é uma feature morta, e o ADR-0160 §5 nomeia a
//! onda que a acorda. Isto é uma dívida DECLARADA, não um esquecimento.

#![forbid(unsafe_code)]

/// **O CAMPO DE ORIENTAÇÃO** — ver [`orientation`].
pub mod orientation;

pub use orientation::{OrientationField, compat_orientation_extrinsic_4, solve_orientation};
