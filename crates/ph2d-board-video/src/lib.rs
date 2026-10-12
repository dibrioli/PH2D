//! W18 ([ADR-0185]) — **o vídeo do quadro, o lado que fala com o MUNDO**: a miniatura pela rede
//! ([`thumb::fetch`]) e o tocador oficial no navegador da família Chrome que a pessoa já tem ([`Player`]),
//! conduzido pelo CDP num tubo e projectado no canvas. Sem navegador ([`PlayerState::NoBrowser`]): a
//! miniatura e «Abrir no navegador». Nada aqui corre no fio da janela.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

mod browser;
mod player;
pub mod thumb;

pub use browser::find as find_browser;
pub use player::{Button, Frame, Key, Mouse, Player, PlayerState};

/// Os JPEG (a miniatura, os quadros do tocador) → RGBA.
#[must_use]
pub fn decode_jpeg(bytes: &[u8]) -> Option<Frame> {
    player::decode_jpeg(bytes)
}

#[cfg(test)]
mod tests;
