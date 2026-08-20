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
//! | **2. posição** | a retícula local + a escala adaptativa | ✅ [`position`] + [`scale`] |
//! | **3. extração** | a malha a partir dos dois campos | ✅ [`mod@extract`] |
//! | 4. consistência | o fluxo de custo mínimo (o passo do QuadriFlow) | ⏳ Q4 |
//! | **5. costura** | o botão `Quad Retopology` e o knob `Detail` | ✅ (shell) |
//!
//! # ⚠️ A faixa é da MALHA, não do slider
//!
//! O que o painel oferece é um `detail` em `0..1`, e quem o converte no lado do
//! quad é a [`scale::edge_for_detail`], **a partir da malha de entrada**. Isto não
//! é conforto de UI: uma retopologia **não pode resolver uma grade mais fina que a
//! malha que ela lê**, e um tamanho absoluto vindo do painel pede exatamente isso
//! metade do tempo. Medido em 2026-08-19 (foto do smoke): abaixo do piso a
//! extração devolve **malha vazia**, e a `1,5×` a aresta de entrada ela devolve um
//! ciclo de **352 lados** com **58 % do volume perdido**.
//!
//! ⚠️ **A1 (`all-quad`) continua a ser a única asserção aberta.** Medido: **97,6 %**
//! na malha que o módulo abre, e **todos** os não-quads que sobram são triângulos
//! **isolados** — o resto é o passo global da Q4, não resíduo barato.

#![forbid(unsafe_code)]

/// **A EXTRAÇÃO — os dois campos viram malha** — ver [`mod@extract`].
pub mod extract;
/// **A HIERARQUIA multirresolução** — ver [`hierarchy`].
pub mod hierarchy;
/// **AS FACES, porte fiel do Instant Meshes** — ver [`im_faces`].
mod im_faces;
/// **O GRAFO, porte fiel do Instant Meshes** — ver [`im_graph`].
mod im_graph;
/// **PESOS COTANGENTE e ÁREAS DUAIS** — ver [`im_weights`].
pub mod im_weights;
/// **O CAMPO DE ORIENTAÇÃO** — ver [`orientation`].
pub mod orientation;
/// **O CAMPO DE POSIÇÃO** — ver [`position`].
pub mod position;
/// **A ESCALA, uniforme ou adaptativa** — ver [`scale`].
pub mod scale;
/// **OS CAMPOS resolvidos de cima para baixo** — ver [`solve`].
pub mod solve;

pub use extract::{Clustering, Quadrangulation, extract, extract_with};
pub use hierarchy::Hierarchy;
pub use orientation::{
    OrientationField, compat_orientation_extrinsic_4, field_from as orientation_from,
    solve_orientation,
};
pub use position::{PositionField, compat_position_extrinsic_4, position_round_4, solve_position};
pub use scale::{
    FLOOR_IN_INPUT_EDGES, MAX_ADAPTIVE_RATIO, MIN_QUADS, ScaleField, edge_for_detail, mean_edge,
    resolvable_edge_range, surface_area,
};
pub use solve::solve_fields;
