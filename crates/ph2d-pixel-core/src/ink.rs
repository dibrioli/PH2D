//! **Inks** (a tinta do Aseprite): o que uma célula vale depois de o pincel passar, em função do
//! que ela JÁ valia. Leis da documentação pública do Aseprite (`docs/ink`); sem oráculo que se
//! corra (`docs/PixelArt/handoffs/HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P2.md` §3).
//!
//! Em Indexed a mistura é em RGBA e volta ao índice mais próximo ([`Sprite::value_of_rgba`]); a
//! cor opaca e a transparente escrevem o índice tal qual (não passam pela cor mais próxima).

use crate::sprite::Sprite;

/// A tinta do pincel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ink {
    /// Cor opaca substitui; transparente apaga; semitransparente mistura por cima.
    #[default]
    Simple,
    /// Sempre por cima (source-over); a transparente não faz nada.
    AlphaCompositing,
    /// Substitui o RGBA inteiro, alfa incluído.
    CopyAlphaColor,
    /// Mistura a cor por cima mas guarda o alfa de destino: só pinta onde já há tinta.
    LockAlpha,
    /// Anda um degrau na rampa (as cores escolhidas da paleta): a célula que não está na rampa
    /// fica intacta.
    Shading,
}

impl Ink {
    pub const ALL: [Ink; 5] = [
        Ink::Simple,
        Ink::AlphaCompositing,
        Ink::CopyAlphaColor,
        Ink::LockAlpha,
        Ink::Shading,
    ];
}

/// O valor novo da célula que vale `dest`, com o pincel a pôr `value`. `ramp` (valores no modo do
/// sprite) e `dir` (+1 = para a direita da rampa, −1 = para a esquerda) só servem o Shading.
#[must_use]
pub fn apply(sprite: &Sprite, ink: Ink, value: u32, dest: u32, ramp: &[u32], dir: i8) -> u32 {
    match ink {
        Ink::CopyAlphaColor => value,
        Ink::Shading => shade(ramp, dest, dir),
        Ink::Simple | Ink::AlphaCompositing | Ink::LockAlpha => {
            let src = sprite.rgba_of_value(value);
            let dst = sprite.rgba_of_value(dest);
            match (ink, src[3]) {
                (Ink::Simple, 0 | 255) => value,
                (Ink::AlphaCompositing, 0) | (Ink::LockAlpha, 0) => dest,
                (Ink::AlphaCompositing, 255) => value,
                (Ink::LockAlpha, _) => {
                    if dst[3] == 0 {
                        return dest;
                    }
                    let mut o = mix_rgb(dst, src);
                    o[3] = dst[3];
                    sprite.value_of_rgba(o)
                }
                _ => sprite.value_of_rgba(crate::compose::over(dst, src)),
            }
        }
    }
}

/// O degrau seguinte da rampa (parado nas pontas); fora da rampa, o próprio valor.
#[must_use]
pub fn shade(ramp: &[u32], dest: u32, dir: i8) -> u32 {
    let Some(p) = ramp.iter().position(|&v| v == dest) else {
        return dest;
    };
    let q = (p as i64 + i64::from(dir)).clamp(0, ramp.len() as i64 - 1) as usize;
    ramp[q]
}

/// O RGB de `dst` levado para `src` na proporção do alfa de `src` (`floor(x + 0.5)`).
fn mix_rgb(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let a = u32::from(src[3]);
    let mut o = dst;
    for k in 0..3 {
        o[k] = ((u32::from(src[k]) * a + u32::from(dst[k]) * (255 - a) + 127) / 255) as u8;
    }
    o
}
