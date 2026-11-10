//! **A TELA «CONTROLS» DO JOGADOR** (plano `docs/InputMap/01_plano.md` §4) — funciona no Play: o jogador abre-a
//! (F1 por omissão, só com «Players can change keys»), escolhe uma acção e carrega na tecla nova; a troca grava-se
//! FORA do projecto, no computador dele, e aplica-se por cima do mapa do jogo.
//!
//! - [`screen`]: o estado e os gestos (↑/↓, Enter ou clique escuta, Esc cancela ou fecha, «Reset to defaults»).
//! - [`layout`] + [`paint`]: o painel, puro (a aba Input Map reusa-o na prévia).
//! - [`file`]: `<base>/players/<game_id>.txt`.
//! - [`session`]: uma corrida do Play — o mapa EFECTIVO que os quatro leitores do Play lêem.

pub mod file;
pub mod layout;
pub mod paint;
pub mod screen;
pub mod session;

pub use file::{home_base, player_file};
pub use layout::Layout;
pub use paint::paint;
pub use screen::{
    ControlsScreen, Hit, Outcome, Row, SLOTS, readable_name, rebind, rows, rows_marked,
};
pub use session::PlaySession;

/// **A identidade do jogo, no mundo** — recurso como o `StableIdCounter` (campo do ficheiro, fora do undo): o load
/// semeia-o e o projecto que nasce sorteia-o na primeira pergunta. Mora aqui para a aba Input Map achar o ficheiro
/// do jogador ([`player_file`]) sem a shell.
#[derive(bevy_ecs::prelude::Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameId(pub u64);

#[cfg(test)]
mod play_tests;
#[cfg(test)]
mod r5c_tests;
#[cfg(test)]
mod r7c_tests;
#[cfg(test)]
mod tests;
