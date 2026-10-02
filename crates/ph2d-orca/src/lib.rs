#![forbid(unsafe_code)]
//! ⭐⭐⭐ **A LEI DO DESVIO ENTRE AGENTES** — oito perseguidores passam uma porta sem se entalarem
//! (plano [30](../../../docs/Components/30_plano_navegacao.md) §2.7, W5).
//!
//! **ORCA** (*Optimal Reciprocal Collision Avoidance*, van den Berg, Guy, Lin, Manocha — ISRR 2011):
//! cada agente escolhe, de entre as velocidades que não o levam a bater em ninguém durante o horizonte
//! `τ`, a mais perto da que QUERIA. Cada vizinho dá UM semi-plano de velocidades permitidas (metade do
//! desvio é dele, metade do vizinho — o *recíproco*); cada parede dá outro, inteiro. A velocidade é a
//! solução de um programa linear em 2D, e quando o conjunto é vazio (multidão densa) um 3D escolhe a
//! que viola MENOS os vizinhos sem nunca violar as paredes.
//!
//! # As três cercas
//!
//! - **Zero dependências** e só `+ − × ÷ sqrt` em `f64` (HR-5).
//! - **Em sequência, pela ordem das entidades** ([`Crowd::solve_all`]): cada agente vê a velocidade
//!   NOVA dos que resolveram antes. A fotografia comum do artigo prende-se nos empates simétricos
//!   (medido no banco de cenários); a ordem é a mesma nos três sistemas e num replay.
//! - **Vizinhos por ordem total** (distância, depois índice): dois vizinhos à mesma distância entram
//!   sempre pela mesma ordem — a 3D depende da ordem dos semi-planos.
//!
//! # ⭐ A parede é da MALHA (a queixa Q1 da pesquisa)
//!
//! O Godot e o RVO do Unreal desviam **ignorando a área andável** (Godot #60354). Aqui as paredes da
//! malha do raio do agente entram como obstáculos ([`Walls`]) — e como essa malha já está recuada pelo
//! raio do CORPO, o agente é um PONTO contra elas (raio `0`): a mesma folga não se conta duas vezes.

pub mod crowd;
pub mod lines;
pub mod lp;
pub mod v2;
pub mod walls;

pub use crowd::{Agent, Crowd, Params, SIDE_BIAS};
pub use lines::Line;
pub use lp::Regime;
pub use v2::V2;
pub use walls::Walls;

#[cfg(test)]
mod tests;
