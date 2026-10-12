//! W15 — **o preto e branco de uma imagem do quadro**: cada píxel vira a luminância relativa dele
//! (Rec. 709 sobre a luz LINEAR — a `LinearRgba::luminance` do `ph2d-color`, a única pública da casa),
//! de volta a sRGB pela curva do mesmo crate; o alfa fica. É também a lei do `feColorMatrix
//! type="saturate" values="0"` do SVG, que trabalha em `linearRGB` por omissão: o SVG exportado
//! embebe o ficheiro original com esse filtro e diz o mesmo.

use ph2d_color::LinearRgba;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};

/// Os 256 valores lineares de um byte sRGB (a curva do `ph2d-color`, calculada uma vez).
fn decode_table() -> [f32; 256] {
    std::array::from_fn(|i| srgb_to_linear_byte(i as u8))
}

/// O cinzento sRGB de um píxel sRGB (`r, g, b`).
#[must_use]
pub fn gray_of(rgb: [u8; 3]) -> u8 {
    let lin = rgb.map(srgb_to_linear_byte);
    linear_to_srgb_byte(LinearRgba::opaque(lin[0], lin[1], lin[2]).luminance())
}

/// Passa uns píxeis RGBA de 8 bits (alfa reto) a preto e branco, no sítio.
pub fn to_gray(rgba: &mut [u8]) {
    let lut = decode_table();
    // Uma fotografia repete muitas cores seguidas (céu, parede): a última conta-se uma vez.
    let mut last: Option<([u8; 3], u8)> = None;
    for p in rgba.as_chunks_mut::<4>().0 {
        let rgb = [p[0], p[1], p[2]];
        let g = match last {
            Some((c, g)) if c == rgb => g,
            _ => {
                let y = LinearRgba::opaque(
                    lut[usize::from(rgb[0])],
                    lut[usize::from(rgb[1])],
                    lut[usize::from(rgb[2])],
                )
                .luminance();
                let g = linear_to_srgb_byte(y);
                last = Some((rgb, g));
                g
            }
        };
        p[..3].fill(g);
    }
}

#[cfg(test)]
#[path = "gray_tests.rs"]
mod tests;
