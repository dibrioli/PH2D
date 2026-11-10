//! **A TELA «CONTROLS» DO JOGADOR** (plano `docs/InputMap/01_plano.md` §4) — funciona no Play: o jogador abre-a
//! (F1 por omissão, só com «Players can change keys»), escolhe uma acção e carrega na tecla nova; a troca grava-se
//! FORA do projecto, no computador dele, e aplica-se por cima do mapa do jogo.
//!
//! - [`screen`]: o estado e os gestos (↑/↓, Enter ou clique escuta, Esc cancela ou fecha, «Reset to defaults»).
//! - [`layout`] + [`paint`]: o painel, puro (a aba Input Map reusa-o na prévia).
//! - [`file`]: `<base>/players/<game_id>.txt`.
//! - [`host`]: a tecla e o clique que a shell passa à sessão, e o relógio que eles pedem.
//! - [`session`]: uma corrida do Play — o mapa EFECTIVO que os quatro leitores do Play lêem.

pub mod file;
pub mod host;
pub mod layout;
pub mod paint;
pub mod screen;
pub mod session;

pub use file::{home_base, player_file, player_file_of};
pub use layout::Layout;
pub use paint::{paint, paint_preview};
pub use screen::{
    ControlsScreen, Hit, Outcome, Row, SLOTS, readable_name, rebind, rows, rows_marked,
};
pub use session::PlaySession;

/// **A identidade do jogo, no mundo** — recurso como o `StableIdCounter` (campo do ficheiro, fora do undo): o load
/// semeia-o e o projecto que nasce sorteia-o na primeira pergunta. Mora aqui para a aba Input Map achar o ficheiro
/// do jogador ([`player_file`]) sem a shell.
#[derive(bevy_ecs::prelude::Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameId(pub u64);

/// **Um `game_id` novo, nunca `0`** — o acaso do SO (a chave da `RandomState` da std, sem dependência nova)
/// misturado com o relógio.
#[must_use]
pub fn new_game_id() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let agora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(agora.map_or(0, |d| d.as_nanos()));
    h.finish().max(1)
}

/// O `game_id` deste mundo; um projecto novo sorteia-o UMA vez e guarda-o (o load semeia o do ficheiro).
pub fn game_id_of(world: &mut bevy_ecs::world::World) -> u64 {
    world
        .get_resource_or_insert_with(|| GameId(new_game_id()))
        .0
}

#[cfg(test)]
mod play_tests;
#[cfg(test)]
mod r10a_tests;
#[cfg(test)]
mod r11a_tests;
#[cfg(test)]
mod r5c_tests;
#[cfg(test)]
mod r7c_tests;
#[cfg(test)]
mod r9d_tests;
#[cfg(test)]
mod tests;
