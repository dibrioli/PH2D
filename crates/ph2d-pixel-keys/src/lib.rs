//! **Os atalhos do Pixel Art como dados** (P12a, `docs/PixelArt/P12_plano.md` §5): a acção, a
//! tecla e o contexto; a tabela de fábrica (a do Aseprite com as excepções do dono); as teclas do
//! app que o Pixel nunca toma; o delta do utilizador em texto; e a busca do Run Command.
//! Mudanças de comportamento e sombras: `docs/PixelArt/P12_teclas.md`.

mod action;
mod chord;
mod factory;
mod keymap;
mod search;
pub mod user_file;

pub use action::{PixelAction, Section, ZoomKey};
pub use chord::{Chord, ChordError, key_name, key_of_name};
pub use factory::{
    ASEPRITE, Context, FACTORY, OWNER_EXCEPTIONS, RESERVED_GLOBAL, SAME_MEANING, reserved,
};
pub use keymap::{Conflict, Keymap, TextReport};
pub use search::{score, search};

#[cfg(test)]
mod tests;
