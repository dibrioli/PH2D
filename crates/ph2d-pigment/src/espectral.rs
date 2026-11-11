//! ⭐ **A mistura ESPECTRAL do pigmento** — a lei do MyPaint 2 (`libmypaint` 1.6.1, `helpers.c`
//! `rgb_to_spectral`/`spectral_to_rgb`): cada cor sobe a uma reflectância de 10 bandas, as cores misturam-se pela
//! MÉDIA GEOMÉTRICA PESADA banda a banda, e a mistura desce ao RGB pela matriz `T`. Vermelho + azul dá o
//! violeta-cinza (`64,74,121`), não o oliva/quase preto do K–M de 3 canais (B32, decisão 7a do dono). Movida de
//! `ph2d-painter-brush` (B32.1) para que o Digital e o Wet leiam a MESMA porta.
//!
//! As tabelas são as do `libmypaint` — ISC, que permite copiar com o aviso:
//!
//! > libmypaint - The MyPaint Brush Library. Copyright (C) 2008-2011 Martin Renold and contributors.
//! > Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby
//! > granted, provided that the above copyright notice and this permission notice appear in all copies.
//! > THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE.
//!
//! A potência é por `log2`/`exp2` polinomiais (só `+ × ÷` e bits: o mesmo resultado em toda máquina, HR-5).

/// O `WGM_EPSILON` do libmypaint: a reflectância nunca é zero (o `log` da média geométrica).
const EPS: f32 = 0.001;

#[rustfmt::skip]
const T: [[f32; 10]; 3] = [
    [0.026_595_62, 0.049_779_43, 0.022_449_85, -0.218_453_7, -0.256_894_9, 0.445_881_7, 0.772_365_9, 0.194_498_77, 0.014_038_158, 0.007_687_264],
    [-0.032_601_67, -0.061_021_04, -0.052_49, 0.206_659_1, 0.572_496_3, 0.317_837_25, -0.021_216_624, -0.019_387_67, -0.001_521_339, -0.000_835_181_6],
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

/// Do `log2` do espectro de volta ao RGB `[0, 1]`.
#[inline]
fn rgb_do_log(l: &[f32; 10]) -> [f32; 3] {
    let mut rgb = [0.0f32; 3];
    for (i, li) in l.iter().enumerate() {
        let s = exp2(*li);
        for (k, v) in rgb.iter_mut().enumerate() {
            *v += T[k][i] * s;
        }
    }
    rgb.map(|v| ((v - EPS) / (1.0 - EPS)).clamp(0.0, 1.0))
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
    rgb_do_log(&std::array::from_fn(|i| lb[i] + fac * (lt[i] - lb[i])))
}

/// A cor de canais `0..255` no invólucro canal a canal de `a` e `b` (o que o K–M também garante: a volta nunca
/// deriva para fora das entradas, e uma aguada parada não se move a cada passe).
#[inline]
fn no_involucro(v: f32, a: f64, b: f64) -> f64 {
    (f64::from(v) * 255.0).clamp(a.min(b), a.max(b))
}

/// [`mistura`] na escala do motor de fluido (`0..255` em `f64`): `w = 0` → `d`, `w = 1` → `s`, ao bit.
#[must_use]
pub fn mistura_255(d: [f64; 3], s: [f64; 3], w: f64) -> [f64; 3] {
    if w <= 0.0 || d == s {
        return d;
    }
    if w >= 1.0 {
        return s;
    }
    let u = |c: [f64; 3]| c.map(|v| (v / 255.0) as f32);
    let m = mistura(u(d), u(s), w as f32);
    [0, 1, 2].map(|k| no_involucro(m[k], d[k], s[k]))
}

/// A média espectral pesada de até 4 cores `0..255` (`colors` = `[r0,g0,b0, r1,…]`, `inv_total` = `1/Σw`): a irmã
/// de `km_weighted_mean_color` do motor de fluido. Cantos pesados iguais devolvem a própria cor, ao bit; o
/// resultado fica no invólucro dos cantos pesados.
pub fn media_pesada_255(
    colors: &[f64; 12],
    weights: &[f64; 4],
    count: usize,
    inv_total: f64,
    out: &mut [f64; 3],
) {
    let pesados = (0..count).filter(|&k| weights[k] > 0.0);
    let mut primeiro: Option<[f64; 3]> = None;
    let mut iguais = true;
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for k in pesados {
        let c = [colors[k * 3], colors[k * 3 + 1], colors[k * 3 + 2]];
        match primeiro {
            None => primeiro = Some(c),
            Some(p) => iguais &= p == c,
        }
        for ch in 0..3 {
            lo[ch] = lo[ch].min(c[ch]);
            hi[ch] = hi[ch].max(c[ch]);
        }
    }
    let Some(p) = primeiro else {
        *out = [0.0; 3];
        return;
    };
    if iguais {
        *out = p;
        return;
    }
    let mut l = [0.0f32; 10];
    for k in 0..count {
        if weights[k] <= 0.0 {
            continue;
        }
        let w = (weights[k] * inv_total) as f32;
        let c = [0, 1, 2].map(|ch| (colors[k * 3 + ch] / 255.0) as f32);
        let lk = log_espectro(c);
        for i in 0..10 {
            l[i] += w * lk[i];
        }
    }
    let m = rgb_do_log(&l);
    for ch in 0..3 {
        out[ch] = (f64::from(m[ch]) * 255.0).clamp(lo[ch], hi[ch]);
    }
}

/// `log2(x)` para `x > 0` normal: a mantissa em `[√½, √2)` e a série de `atanh` até `t⁹` (erro < 1e-7).
#[inline]
#[must_use]
pub fn log2(x: f32) -> f32 {
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

/// `2^y` para `y ∈ [−126, 4]`: `2^n · e^(f·ln2)` com `f ∈ [−½, ½]` e Taylor até ao 7.º termo.
#[inline]
#[must_use]
pub fn exp2(y: f32) -> f32 {
    let y = y.clamp(-126.0, 4.0);
    let n = (y + 0.5).floor();
    let x = (y - n) * std::f32::consts::LN_2;
    let p = 1.0
        + x * (1.0
            + x * (0.5
                + x * (1.0 / 6.0
                    + x * (1.0 / 24.0 + x * (1.0 / 120.0 + x * (1.0 / 720.0 + x / 5040.0))))));
    p * f32::from_bits(((n as i32 + 127) as u32) << 23)
}

#[cfg(test)]
#[path = "espectral_tests.rs"]
mod tests;
