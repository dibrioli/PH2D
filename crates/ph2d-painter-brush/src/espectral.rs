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

/// O `WGM_EPSILON` do libmypaint: a reflectância nunca é zero (o `log` da média geométrica).
const EPS: f32 = 0.001;

#[rustfmt::skip]
const T: [[f32; 10]; 3] = [
    [0.026_595_62, 0.049_779_43, 0.022_449_85, -0.218_453_7, -0.256_894_9, 0.445_881_7, 0.772_365_9, 0.194_498_77, 0.014_038_158, 0.007_687_264],
    [-0.032_601_67, -0.061_021_04, -0.052_49, 0.206_659_1, 0.572_496_3, 0.317_837_25, -0.021_216_624, -0.019_387_669, -0.001_521_339, -0.000_835_181_6],
    [0.339_475_47, 0.635_401_4, 0.771_520_8, 0.113_222_64, -0.055_251_114, -0.048_222_58, -0.012_966_666, -0.001_523_814_5, -0.000_094_718_95, -0.000_051_604_595],
];
#[rustfmt::skip]
const SR: [f32; 10] = [0.009_281_363, 0.009_732_627, 0.011_254_253, 0.015_105_579, 0.024_797_924, 0.083_622_59, 0.977_865, 1.0, 0.999_961, 1.0];
#[rustfmt::skip]
const SG: [f32; 10] = [0.002_854_127_4, 0.003_917_59, 0.012_132_152, 0.748_259_2, 1.0, 0.865_695_9, 0.037_477_47, 0.022_816_79, 0.021_747_42, 0.021_384_94];
#[rustfmt::skip]
const SB: [f32; 10] = [0.537_052_15, 0.546_646_4, 0.575_501_8, 0.258_778_83, 0.041_709_924, 0.012_662_639, 0.007_485_593, 0.006_766_901, 0.006_699_765, 0.006_676_22];

/// O `log2` da reflectância de 10 bandas de uma cor RGB `[0, 1]`.
#[inline]
fn log_espectro(c: [f32; 3]) -> [f32; 10] {
    let [r, g, b] = c.map(|v| v.clamp(0.0, 1.0) * (1.0 - EPS) + EPS);
    std::array::from_fn(|i| log2(SR[i] * r + SG[i] * g + SB[i] * b))
}

/// ⭐ **A média geométrica pesada** de duas cores: `fac` da `tinta`, `1 − fac` da `base` (RGB `[0, 1]`).
#[must_use]
pub fn mistura(base: [f32; 3], tinta: [f32; 3], fac: f32) -> [f32; 3] {
    let fac = fac.clamp(0.0, 1.0);
    if base == tinta || fac >= 1.0 {
        return tinta;
    }
    if fac <= 0.0 {
        return base;
    }
    let (lb, lt) = (log_espectro(base), log_espectro(tinta));
    let mut rgb = [0.0f32; 3];
    for i in 0..10 {
        let s = exp2(lb[i] + fac * (lt[i] - lb[i]));
        for (k, v) in rgb.iter_mut().enumerate() {
            *v += T[k][i] * s;
        }
    }
    rgb.map(|v| ((v - EPS) / (1.0 - EPS)).clamp(0.0, 1.0))
}

/// ⭐ **A porta do pigmento espectral** — a irmã de [`crate::blend::blend_over_pigment`] (o K–M): o mesmo `plain`,
/// a mesma regra do papel (o quase-branco opaco não é pigmento, [`PAPEL_DO_PIGMENTO`]), e a mistura pela média
/// geométrica com o peso do source-over (`a / (a + (1 − a)·αb)`) — sobre o vazio (`αb = 0`) é a tinta, o `plain`.
#[must_use]
#[inline]
pub fn blend_over_espectral(mode: BrushBlend, dst: [f32; 4], color: [f32; 3], a: f32, mix: f32) -> [f32; 4] {
    let plain = blend_over(mode, dst, color, a);
    let ab = dst[3].clamp(0.0, 1.0);
    let tinta = ((1.0 - dst[0].min(dst[1]).min(dst[2]).clamp(0.0, 1.0)) / (1.0 - PAPEL_DO_PIGMENTO)).min(1.0);
    let m = mix.clamp(0.0, 1.0) * tinta;
    let a = a.clamp(0.0, 1.0);
    if m <= 0.0 || ab <= 0.0 || a <= 0.0 || !mode.lays_pigment() {
        return plain;
    }
    let fac = a / (a + (1.0 - a) * ab);
    let sub = mistura([dst[0], dst[1], dst[2]], color, fac);
    let l = |p: f32, s: f32| p + (s - p) * m;
    [l(plain[0], sub[0]), l(plain[1], sub[1]), l(plain[2], sub[2]), plain[3]]
}

/// `log2(x)` para `x > 0` normal: a mantissa em `[√½, √2)` e a série de `atanh` até `t⁹` (erro < 1e-7).
#[inline]
fn log2(x: f32) -> f32 {
    let bits = x.to_bits();
    let mut e = ((bits >> 23) & 0xff) as i32 - 127;
    let mut m = f32::from_bits((bits & 0x007f_ffff) | 0x3f80_0000);
    if m > std::f32::consts::SQRT_2 {
        m *= 0.5;
        e += 1;
    }
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let s = t * (1.0 + t2 * (1.0 / 3.0 + t2 * (1.0 / 5.0 + t2 * (1.0 / 7.0 + t2 * (1.0 / 9.0)))));
    e as f32 + s * (2.0 * std::f32::consts::LOG2_E)
}

/// `2^y` para `y ∈ [−126, 4]` (as reflectâncias passam pouco de 1): `2^n · e^(f·ln2)` com `f ∈ [−½, ½]` e Taylor
/// até ao 7.º termo.
#[inline]
fn exp2(y: f32) -> f32 {
    let y = y.clamp(-126.0, 4.0);
    let n = (y + 0.5).floor();
    let x = (y - n) * std::f32::consts::LN_2;
    let p = 1.0
        + x * (1.0 + x * (0.5 + x * (1.0 / 6.0 + x * (1.0 / 24.0 + x * (1.0 / 120.0 + x * (1.0 / 720.0 + x / 5040.0))))));
    p * f32::from_bits(((n as i32 + 127) as u32) << 23)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AZUL: [f32; 3] = [23.0 / 255.0, 101.0 / 255.0, 179.0 / 255.0];
    const VERMELHO: [f32; 3] = [191.0 / 255.0, 0.0, 18.0 / 255.0];

    #[test]
    fn as_potencias_rapidas_casam_as_do_std() {
        for &x in &[1e-5f32, 0.003, 0.1, 0.5, 0.7071, 0.99, 1.0] {
            assert!((log2(x) - x.log2()).abs() < 2e-6, "log2({x})");
        }
        for &y in &[-16.0f32, -9.3, -1.5, -0.49, -0.01, 0.0, 0.09, 1.3] {
            assert!((exp2(y) / y.exp2() - 1.0).abs() < 2e-6, "exp2({y})");
        }
    }

    /// ⭐ **O azul sobre o vermelho na camada dá o violeta-cinza do MyPaint** (`deevad/ballpen`: `64,74,121`): o modelo
    /// de 10 bandas do libmypaint numa camada transparente (o vermelho a α 0,686, o azul a 0,687 por cima) — o verde
    /// cai abaixo do 94 que toda mistura de 3 canais dá.
    #[test]
    fn o_azul_sobre_o_vermelho_da_o_violeta_cinza_do_mypaint() {
        let (ab, a) = (0.686f32, 0.687f32);
        let out = blend_over_espectral(BrushBlend::Mix, [VERMELHO[0], VERMELHO[1], VERMELHO[2], ab], AZUL, a, 1.0);
        let sobre_papel = [0, 1, 2].map(|k| ((out[k] * out[3] + 1.0 - out[3]) * 255.0).round());
        let mypaint = [64.0, 74.0, 121.0];
        for k in 0..3 {
            assert!((sobre_papel[k] - mypaint[k]).abs() <= 6.0, "{sobre_papel:?} × MyPaint {mypaint:?}");
        }
        assert!(sobre_papel[1] < 80.0, "o verde fica no dos 3 canais: {sobre_papel:?}");
    }

    /// A mesma tinta sobre ela própria é ela própria (o néon que se soma não escurece) e sobre o vazio é o `plain`.
    #[test]
    fn a_mesma_tinta_e_o_vazio_nao_mudam() {
        let azul = [AZUL[0], AZUL[1], AZUL[2], 0.3];
        let (e, o) = (blend_over_espectral(BrushBlend::Mix, azul, AZUL, 0.45, 1.0), blend_over(BrushBlend::Mix, azul, AZUL, 0.45));
        assert!((0..4).all(|k| (e[k] - o[k]).abs() < 1e-6), "{e:?} × {o:?}");
        let vazio = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(blend_over_espectral(BrushBlend::Mix, vazio, AZUL, 0.45, 1.0), blend_over(BrushBlend::Mix, vazio, AZUL, 0.45));
        let papel = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(blend_over_espectral(BrushBlend::Mix, papel, AZUL, 0.45, 1.0), blend_over(BrushBlend::Mix, papel, AZUL, 0.45));
    }
}
