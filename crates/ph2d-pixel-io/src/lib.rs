//! **Pixel Art — entrada e saída** (ADR-0192, onda P9 do `docs/PixelArt/02_plano_modulo.md`).
//!
//! `.ase`/`.aseprite` ler e escrever pela spec pública, folha + JSON compatível com o Aseprite,
//! GIF/APNG/PNG/sequência pelos `ph2d-imageio-*`, paletas `.gpl .pal .hex .act`. Na P0 só existe a
//! porta mais pequena: o RGBA de um frame, que é o que todo exportador começa por pedir.

use ph2d_pixel_core::{FrameId, Sprite, compose_rgba};

pub mod msk;
pub mod palette_file;

/// O RGBA (alfa reto) do frame `frame`, `width * height * 4` bytes.
#[must_use]
pub fn frame_rgba(sprite: &Sprite, frame: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(sprite, frame, None, &mut out);
    out
}
