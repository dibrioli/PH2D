//! A família do JOGO a correr (GameEngineFuncional R0, `docs/GameEngineFuncional/04_R0_N0_plano.md`).
//!
//! A shell compõe; a lei vive aqui: [`Sessao`] decide quando o documento se fotografa e quando volta.

use ph2d_game_dicas as dicas;
pub mod encaixe;
pub mod formato_do_jogo;
pub mod miniatura;
pub mod object_add;
pub mod ordem_smoke;
pub mod personagem;
pub mod sessao;
pub mod smoke;
pub mod smoke_mao;
pub mod smoke_moedas;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "game",
    routers: smoke::ROUTERS,
};

pub use sessao::{Passo, Sessao, marca_a_copia};

#[cfg(test)]
#[path = "arte_tests.rs"]
mod arte_tests;
