//! ⭐ **A mistura ESPECTRAL do pigmento** (J4 efeitos 2.ª ronda; `docs/Painter/Pinceis/exames/efeitos.md`): a lei do
//! MyPaint 2 (`libmypaint` 1.6.1, `helpers.c` `rgb_to_spectral`/`spectral_to_rgb` e `brushmodes.c`
//! `draw_dab_pixels_BlendMode_Normal_Paint`) — cada cor sobe a uma reflectância de 10 bandas (a soma das três
//! primárias espectrais pesadas pelo RGB), as duas misturam-se pela MÉDIA GEOMÉTRICA PESADA banda a banda, e a
//! mistura desce ao RGB pela matriz `T`. O peso da tinta é o do source-over em alfa recto,
//! `a / (a + (1 − a)·αb)`: numa camada TRANSPARENTE (onde o artista pinta) o azul ralo é azul a α baixo, não um
//! azul-claro — e o azul sobre o vermelho dá o violeta-cinza do MyPaint (`64,74,121`) e não o azul do K–M
//! (`48,94,137`, o verde preso nos 94 de toda mistura de 3 canais).
//!
//! As tabelas são as do `libmypaint` — ISC, que permite copiar com o aviso:
//!
//! > libmypaint - The MyPaint Brush Library. Copyright (C) 2008-2011 Martin Renold and contributors.
//! > Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby
//! > granted, provided that the above copyright notice and this permission notice appear in all copies.
//! > THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE.
//!
//! A potência é por `log2`/`exp2` polinomiais (só `+ × ÷` e bits: o mesmo resultado em toda máquina, HR-5; o
//! MyPaint usa o `fastpow` dele, aproximado do mesmo modo).

use crate::blend::{BrushBlend, PAPEL_DO_PIGMENTO, blend_over};
/// A lei mora em `ph2d-pigment` (B32.1): o Wet lê a MESMA porta.
pub use ph2d_pigment::espectral::{exp2, log2, mistura};

/// ⭐ **A porta do pigmento espectral** — a irmã de [`crate::blend::blend_over_pigment`] (o K–M): o mesmo `plain`,
/// a mesma regra do papel (o quase-branco opaco não é pigmento, [`PAPEL_DO_PIGMENTO`]), e a mistura pela média
/// geométrica com o peso do source-over (`a / (a + (1 − a)·αb)`) — sobre o vazio (`αb = 0`) é a tinta, o `plain`.
#[must_use]
#[inline]
pub fn blend_over_espectral(
    mode: BrushBlend,
    dst: [f32; 4],
    color: [f32; 3],
    a: f32,
    mix: f32,
) -> [f32; 4] {
    let plain = blend_over(mode, dst, color, a);
    let ab = dst[3].clamp(0.0, 1.0);
    let tinta = ((1.0 - dst[0].min(dst[1]).min(dst[2]).clamp(0.0, 1.0))
        / (1.0 - PAPEL_DO_PIGMENTO))
        .min(1.0);
    let m = mix.clamp(0.0, 1.0) * tinta;
    let a = a.clamp(0.0, 1.0);
    if m <= 0.0 || ab <= 0.0 || a <= 0.0 || !mode.lays_pigment() {
        return plain;
    }
    let fac = a / (a + (1.0 - a) * ab);
    let sub = mistura([dst[0], dst[1], dst[2]], color, fac);
    let l = |p: f32, s: f32| p + (s - p) * m;
    [
        l(plain[0], sub[0]),
        l(plain[1], sub[1]),
        l(plain[2], sub[2]),
        plain[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const AZUL: [f32; 3] = [23.0 / 255.0, 101.0 / 255.0, 179.0 / 255.0];
    const VERMELHO: [f32; 3] = [191.0 / 255.0, 0.0, 18.0 / 255.0];

    /// ⭐ **O azul sobre o vermelho na camada dá o violeta-cinza do MyPaint** (`deevad/ballpen`: `64,74,121`): o modelo
    /// de 10 bandas do libmypaint numa camada transparente (o vermelho a α 0,686, o azul a 0,687 por cima) — o verde
    /// cai abaixo do 94 que toda mistura de 3 canais dá.
    #[test]
    fn o_azul_sobre_o_vermelho_da_o_violeta_cinza_do_mypaint() {
        let (ab, a) = (0.686f32, 0.687f32);
        let out = blend_over_espectral(
            BrushBlend::Mix,
            [VERMELHO[0], VERMELHO[1], VERMELHO[2], ab],
            AZUL,
            a,
            1.0,
        );
        let sobre_papel = [0, 1, 2].map(|k| ((out[k] * out[3] + 1.0 - out[3]) * 255.0).round());
        let mypaint = [64.0, 74.0, 121.0];
        for k in 0..3 {
            assert!(
                (sobre_papel[k] - mypaint[k]).abs() <= 6.0,
                "{sobre_papel:?} × MyPaint {mypaint:?}"
            );
        }
        assert!(
            sobre_papel[1] < 80.0,
            "o verde fica no dos 3 canais: {sobre_papel:?}"
        );
    }

    /// A mesma tinta sobre ela própria é ela própria (o néon que se soma não escurece) e sobre o vazio é o `plain`.
    #[test]
    fn a_mesma_tinta_e_o_vazio_nao_mudam() {
        let azul = [AZUL[0], AZUL[1], AZUL[2], 0.3];
        let (e, o) = (
            blend_over_espectral(BrushBlend::Mix, azul, AZUL, 0.45, 1.0),
            blend_over(BrushBlend::Mix, azul, AZUL, 0.45),
        );
        assert!((0..4).all(|k| (e[k] - o[k]).abs() < 1e-6), "{e:?} × {o:?}");
        let vazio = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(
            blend_over_espectral(BrushBlend::Mix, vazio, AZUL, 0.45, 1.0),
            blend_over(BrushBlend::Mix, vazio, AZUL, 0.45)
        );
        let papel = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(
            blend_over_espectral(BrushBlend::Mix, papel, AZUL, 0.45, 1.0),
            blend_over(BrushBlend::Mix, papel, AZUL, 0.45)
        );
    }
}
