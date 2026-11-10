//! **Pixel Art — entrada e saída** (ADR-0196, onda P9 do `docs/PixelArt/02_plano_modulo.md`).
//!
//! `.ase`/`.aseprite` ler e escrever pela spec pública, folha + JSON compatível com o Aseprite,
//! GIF/APNG/PNG/sequência pelos `ph2d-imageio-*`, paletas `.gpl .pal .hex .act`. Na P0 só existe a
//! porta mais pequena: o RGBA de um frame, que é o que todo exportador começa por pedir.

use ph2d_pixel_core::{FrameId, Purpose, Sprite, compose_window_em, instante_do_quadro};

pub mod abrir;
pub mod ase;
pub mod brush_file;
pub mod exporta;
pub mod folha_audio;
pub mod folha_json;
pub mod frases;
pub mod gif_cores;
pub mod godot;
pub mod msk;
pub mod palette_file;
pub mod render;
pub mod saida;
pub mod sheet;
pub mod tiles_saida;

/// O RGBA (alfa reto) do frame `frame`, `width * height * 4` bytes, com os ciclos de cor no
/// instante do quadro ([`instante_do_quadro`]).
#[must_use]
pub fn frame_rgba(sprite: &Sprite, frame: FrameId) -> Vec<u8> {
    let t = instante_do_quadro(sprite, frame);
    compose_window_em(sprite, frame, None, sprite.bounds(), Purpose::View, t)
}
