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

/// O [`frame_rgba`] reduzido por vizinho (o centro de cada célula), com o aspecto, se o lado maior passa de
/// `lado_max`; devolve os bytes e o tamanho `[w, h]`. A miniatura do cartão do Pixel Sprite.
#[must_use]
pub fn frame_rgba_reduzido(sprite: &Sprite, frame: FrameId, lado_max: u32) -> (Vec<u8>, [u32; 2]) {
    let cheio = frame_rgba(sprite, frame);
    let (w, h) = (sprite.width(), sprite.height());
    let maior = w.max(h);
    if maior <= lado_max {
        return (cheio, [w, h]);
    }
    let reduz = |n: u32| (u64::from(n) * u64::from(lado_max) / u64::from(maior)).max(1) as u32;
    let (tw, th) = (reduz(w), reduz(h));
    let centro = |i: u32, de: u32, para: u32| {
        ((u64::from(i) * 2 + 1) * u64::from(de) / (u64::from(para) * 2)) as usize
    };
    let mut out = Vec::with_capacity(tw as usize * th as usize * 4);
    for y in 0..th {
        let linha = centro(y, h, th) * w as usize;
        for x in 0..tw {
            let i = (linha + centro(x, w, tw)) * 4;
            out.extend_from_slice(&cheio[i..i + 4]);
        }
    }
    (out, [tw, th])
}
