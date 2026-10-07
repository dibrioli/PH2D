#![forbid(unsafe_code)]
//! ⭐⭐⭐ **A LEI DA NAVEGAÇÃO** — inimigos que acham o caminho sozinhos (plano
//! [30](../../../docs/Components/30_plano_navegacao.md), pesquisa
//! [29](../../../docs/Components/29_pesquisa_navegacao.md)).
//!
//! Esta folha responde a três perguntas, e mais nenhuma:
//!
//! 1. **Onde se anda?** — [`NavMesh`]: polígonos convexos, a vizinhança, os cantos, as ilhas.
//!    ⚠️ Ela não SABE construir-se a partir de colisores: isso é a `ph2d-navmesh`, que depende desta.
//! 2. **Qual é o caminho mais curto?** — [`Polyanya`]: óptimo em qualquer ângulo (§2.5 do plano).
//! 3. **(W3) O que o agente faz neste tique?** — a condução: o corredor, o recálculo, os estados.
//!
//! # As três cercas
//!
//! - **Zero dependências** e só `+ − × ÷ sqrt` em `f64` (HR-5): um caminho entra no hash que o CI
//!   compara nos três sistemas.
//! - **Nenhum estado escondido entre consultas**: a procura reaproveita buffers e mais nada.
//! - **Toda recusa tem nome** ([`MeshError`], [`NoPath`]) — um caminho vazio calado é a queixa Q4 da
//!   pesquisa, medida no próprio Godot.

pub mod agent;
pub mod blocos;
pub mod cost;
mod cota;
pub mod geom;
mod grelha;
pub mod link;
pub mod mesh;
pub mod plano;
pub mod polyanya;
pub mod refresh;

#[cfg(any(test, feature = "test-support"))]
pub mod oracle;

pub use agent::{AgentConfig, AgentRuntime, Event, Status, Steer};
pub use blocos::{FaixaDeParedes, MalhaPorBlocos, Peca};
pub use geom::V2;
pub use link::{Hop, Link, Query};
pub use mesh::{MeshError, NavMesh, Poly};
pub use plano::{Planeado, Plano};
pub use polyanya::{NoPath, Path, Polyanya, Stats};

#[cfg(test)]
mod tests;
