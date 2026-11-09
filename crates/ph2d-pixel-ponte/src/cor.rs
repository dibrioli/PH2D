//! **A cor e a paleta** do chrome para o núcleo: o modo, o valor de uma amostra, a rampa do
//! Shading e a matriz e as pontas do Gradient (saiu de `ph2d-app-pixel::{paleta, degrade}`, P14).

use std::collections::BTreeSet;

use ph2d_editor_core::pixel_panel::{NewSpriteMode, PixelGradientDither};
use ph2d_pixel_core::dither::Dither;
use ph2d_pixel_core::{ColorMode, Sprite};

/// O modo do documento no vocabulário do chrome.
#[must_use]
pub fn modo(m: ColorMode) -> NewSpriteMode {
    match m {
        ColorMode::Rgba => NewSpriteMode::Rgba,
        ColorMode::Gray => NewSpriteMode::Gray,
        ColorMode::Indexed => NewSpriteMode::Indexed,
    }
}

/// O valor (no modo do sprite) da amostra `i` da paleta: Indexed = o índice, RGBA = os bytes,
/// cinza = a cor levada a cinza.
#[must_use]
pub fn valor_da_amostra(s: &Sprite, i: usize) -> Option<u32> {
    let c = *s.palette().get(i)?;
    Some(match s.mode() {
        ColorMode::Indexed => u32::try_from(i).ok()?,
        ColorMode::Rgba => u32::from_le_bytes(c),
        ColorMode::Gray => s.value_of_rgba(c),
    })
}

/// A rampa do Shading: as amostras seleccionadas em ordem de paleta, como valores do sprite.
#[must_use]
pub fn rampa(s: &Sprite, sel: &BTreeSet<usize>) -> Vec<u32> {
    sel.iter().filter_map(|&i| valor_da_amostra(s, i)).collect()
}

/// A matriz do Gradient no vocabulário do núcleo.
#[must_use]
pub fn dither(d: PixelGradientDither) -> Dither {
    match d {
        PixelGradientDither::None => Dither::None,
        PixelGradientDither::Bayer2 => Dither::Bayer2,
        PixelGradientDither::Bayer4 => Dither::Bayer4,
        PixelGradientDither::Bayer8 => Dither::Bayer8,
    }
}

/// A entrada da paleta da cor `c`: a igual, senão a mais próxima em RGBA (no Indexed a frente e o
/// fundo já são entradas da paleta; no RGB uma cor fora dela anda até à vizinha — a rampa nunca
/// inventa uma cor que a paleta não tem).
#[must_use]
pub fn entrada(palette: &[[u8; 4]], c: [u8; 4]) -> usize {
    let d = |p: &[u8; 4]| -> u32 {
        (0..4)
            .map(|k| (i32::from(p[k]) - i32::from(c[k])).unsigned_abs().pow(2))
            .sum()
    };
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| d(p))
        .map_or(0, |(i, _)| i)
}
