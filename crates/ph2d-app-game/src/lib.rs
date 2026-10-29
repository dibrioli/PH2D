//! A família do JOGO a correr (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`).
//!
//! A shell compõe; a lei vive aqui: [`Sessao`] decide quando o documento se fotografa e quando volta.

mod dicas;
pub mod encaixe;
pub mod formato_do_jogo;
pub mod moldura;
pub mod moldura_tinta;
pub mod object_add;
pub mod personagem;
pub mod sessao;
pub mod smoke;
pub mod smoke_mao;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "game",
    routers: smoke::ROUTERS,
};

pub use sessao::{Passo, Sessao, marca_a_copia};

#[cfg(test)]
#[path = "arte_tests.rs"]
mod arte_tests;
