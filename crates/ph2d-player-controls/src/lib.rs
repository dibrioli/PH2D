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
pub use screen::{ControlsScreen, Hit, Outcome, Row, SLOTS, readable_name, rebind, rows};
pub use session::PlaySession;

#[cfg(test)]
mod tests;
