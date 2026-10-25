//! A família do JOGO a correr (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`).
//!
//! A shell compõe; a lei vive aqui: [`Sessao`] decide quando o documento se fotografa e quando volta.

pub mod sessao;
pub mod smoke;

pub use sessao::{Passo, Sessao, marca_a_copia};
