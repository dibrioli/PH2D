//! ⭐ **A conta das capas de imagem do Kanban** (veio da `ph2d-app-kanban`, `front/cover.rs`): a
//! redução por metades da miniatura e o véu do título que lê o texto a 4,5:1 (WCAG) sobre a linha
//! mais clara da imagem real.

/// Metade em cada lado (média de cada quadrado 2×2; a coluna/linha ímpar final sai).
#[must_use]
pub fn halve(w: u32, h: u32, px: &[u8]) -> (u32, u32, Vec<u8>) {
    let (nw, nh) = (w / 2, h / 2);
    let mut out = Vec::with_capacity(nw as usize * nh as usize * 4);
    let at =
        |x: u32, y: u32, c: usize| u32::from(px[(y as usize * w as usize + x as usize) * 4 + c]);
    for y in 0..nh {
        for x in 0..nw {
            for c in 0..4 {
                let s = at(2 * x, 2 * y, c)
                    + at(2 * x + 1, 2 * y, c)
                    + at(2 * x, 2 * y + 1, c)
                    + at(2 * x + 1, 2 * y + 1, c);
                #[allow(clippy::cast_possible_truncation)]
                out.push(((s + 2) / 4) as u8);
            }
        }
    }
    (nw, nh, out)
}

fn lin(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A luminância relativa (WCAG) de um píxel; o transparente conta como BRANCO (o pior fundo).
#[must_use]
pub fn pixel_luminance(p: &[u8; 4]) -> f32 {
    let l = 0.2126 * lin(p[0]) + 0.7152 * lin(p[1]) + 0.0722 * lin(p[2]);
    let a = f32::from(p[3]) / 255.0;
    l * a + (1.0 - a)
}

/// O contraste WCAG entre duas luminâncias.
#[must_use]
pub fn contrast(a: f32, b: f32) -> f32 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// O contraste que o texto tem de ter sobre a imagem (WCAG AA, texto normal).
pub const MIN_CONTRAST: f32 = 4.5;

/// ⭐ O alfa do véu (cor de luminância `veil`) para o texto de luminância `ink` se ler a
/// [`MIN_CONTRAST`] sobre a linha mais clara `brightest` da faixa do título — composto em luz
/// LINEAR (o Vello compõe em sRGB, que dá MAIS escuro: a conta é do lado seguro). Nunca menos que
/// `floor` (o véu da maquete).
#[must_use]
pub fn veil_alpha(brightest: f32, ink: f32, veil: f32, floor: f32) -> f32 {
    let goal = (ink + 0.05) / MIN_CONTRAST - 0.05;
    let need = if brightest <= goal || brightest <= veil {
        0.0
    } else {
        (brightest - goal) / (brightest - veil)
    };
    need.clamp(floor, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halving_averages_each_two_by_two() {
        let px = [
            0, 0, 0, 255, 255, 255, 255, 255, //
            255, 255, 255, 255, 0, 0, 0, 255,
        ];
        let (w, h, out) = halve(2, 2, &px);
        assert_eq!((w, h), (1, 1));
        assert_eq!(out, vec![128, 128, 128, 255]);
    }

    #[test]
    fn the_veil_brings_white_text_to_four_and_a_half_on_a_white_image() {
        let a = veil_alpha(1.0, 0.8, 0.0, 0.3);
        let under = 1.0 * (1.0 - a);
        assert!((contrast(0.8, under) - MIN_CONTRAST).abs() < 1e-3, "{a}");
        assert!((veil_alpha(0.0, 0.8, 0.0, 0.3) - 0.3).abs() < f32::EPSILON);
    }
}
