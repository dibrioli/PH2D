//! A lei «Pigment mix» (E10b, F3 §7 R8): um escalar por pincel `p ∈ [0,1]` que interpola a cor da fatia entre a
//! média Oklab pelo volume (`p = 0`, a lei de [`Soma::cor`], IGUAL AO BIT) e a média GEOMÉTRICA pesada pelo volume
//! da reflectância a 10 bandas do libmypaint (WGM, `p = 1`: azul + amarelo = verde).
//!
//! Estado: a [`Soma`] da fatia + o [`Espectro`] lateral `{Σv·log₂R_λ, Σv·δ, Σv·p}` — todas extensivas, logo a
//! mistura é associativa (N passadas = uma) e [`transfere_pigmento`] move a MESMA fracção de tudo. A cor misturada
//! NUNCA volta às bandas (a ida-volta custa ~0,4 ΔE e deriva). `δ` = o resíduo Oklab da ida-volta de cada tinta
//! (`o − o_WGM(R)`), somado pelo volume: `mix(c,c) = c` sem atalho (o `base == tinta` do `espectral.rs` é uma
//! descontinuidade a 1/255). Doc: `docs/Painter/Arquitetura/E10b_pigment_mix.md`.
//!
//! As tabelas são as do `libmypaint` 1.6.1 (`helpers.c`), cópia VERBATIM das de
//! `crates/ph2d-painter-brush/src/espectral.rs` — ISC:
//!
//! > libmypaint - The MyPaint Brush Library. Copyright (C) 2008-2011 Martin Renold and contributors.
//! > Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby
//! > granted, provided that the above copyright notice and this permission notice appear in all copies.
//! > THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE.

use crate::oklab::{Lab, linear_para_oklab};
use crate::soma::{Soma, Tinta};
use crate::transfere::{move_um, transfere};

/// Bandas da reflectância WGM.
pub const N_WGM: usize = 10;

/// O `WGM_EPSILON` do libmypaint: o piso da reflectância (o `log` do canal `0` do seletor).
pub const EPS_WGM: f32 = 0.001;

#[rustfmt::skip]
const T: [[f32; N_WGM]; 3] = [
    [0.026_595_62, 0.049_779_43, 0.022_449_85, -0.218_453_7, -0.256_894_9, 0.445_881_7, 0.772_365_9, 0.194_498_77, 0.014_038_158, 0.007_687_264],
    [-0.032_601_67, -0.061_021_04, -0.052_49, 0.206_659_1, 0.572_496_3, 0.317_837_25, -0.021_216_624, -0.019_387_67, -0.001_521_339, -0.000_835_181_6],
    [0.339_475_47, 0.635_401_4, 0.771_520_8, 0.113_222_64, -0.055_251_114, -0.048_222_58, -0.012_966_666, -0.001_523_814_5, -0.000_094_718_95, -0.000_051_604_595],
];
#[rustfmt::skip]
const SR: [f32; N_WGM] = [0.009_281_363, 0.009_732_627, 0.011_254_253, 0.015_105_579, 0.024_797_924, 0.083_622_59, 0.977_865, 1.0, 0.999_961, 1.0];
#[rustfmt::skip]
const SG: [f32; N_WGM] = [0.002_854_127_4, 0.003_917_59, 0.012_132_152, 0.748_259_2, 1.0, 0.865_695_9, 0.037_477_47, 0.022_816_79, 0.021_747_42, 0.021_384_94];
#[rustfmt::skip]
const SB: [f32; N_WGM] = [0.537_052_15, 0.546_646_4, 0.575_501_8, 0.258_778_83, 0.041_709_924, 0.012_662_639, 0.007_485_593, 0.006_766_901, 0.006_699_765, 0.006_676_22];

/// `log₂ R_λ` de uma cor RGB LINEAR (cortada a `[0,1]` e levada a `[EPS, 1]`, como o libmypaint).
#[must_use]
pub fn log_espectro(rgb: [f32; 3]) -> [f32; N_WGM] {
    let [r, g, b] = rgb.map(|v| v.clamp(0.0, 1.0) * (1.0 - EPS_WGM) + EPS_WGM);
    core::array::from_fn(|i| libm::log2f((SR[i] * r + SG[i] * g) + SB[i] * b))
}

/// A volta: `R_λ = 2^l_λ` → RGB linear pela matriz `T`, sem o piso → Oklab.
#[must_use]
pub fn oklab_de_log(l: &[f32; N_WGM]) -> Lab {
    let mut rgb = [0.0f32; 3];
    for (i, &li) in l.iter().enumerate() {
        let s = libm::exp2f(li);
        for (k, v) in rgb.iter_mut().enumerate() {
            *v += T[k][i] * s;
        }
    }
    linear_para_oklab(rgb.map(|v| ((v - EPS_WGM) / (1.0 - EPS_WGM)).clamp(0.0, 1.0)))
}

/// O lado ESPECTRAL de uma fatia (56 B): somas pesadas pelo MESMO volume `v` da [`Soma`] que acompanha.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Espectro {
    /// `Σ v·log₂ R_λ`.
    pub ln_r: [f32; N_WGM],
    /// `Σ v·δ`, `δ = o − o_WGM(R)`: o resíduo Oklab da ida-volta de cada tinta.
    pub delta: [f32; 3],
    /// `Σ v·p`: o «Pigment mix» do pincel que a pousou.
    pub p: f32,
}

impl Espectro {
    /// `v` µm da tinta `c` pousada com o Pigment mix `p`.
    #[must_use]
    pub fn de(c: Tinta, v: f32, p: f32) -> Self {
        let l = log_espectro(c.linear());
        let ow = oklab_de_log(&l);
        Self {
            ln_r: l.map(|x| v * x),
            delta: [0, 1, 2].map(|k| v * (c.lab[k] - ow[k])),
            p: v * p.clamp(0.0, 1.0),
        }
    }

    /// O lado espectral que NASCE sob uma fatia que já existia (o campo lateral alocado agora): a cor dela com
    /// `p = 0` — a única vez que uma cor misturada sobe às bandas.
    #[must_use]
    pub fn da_soma(s: &Soma) -> Self {
        s.cor()
            .map_or_else(Self::default, |c| Self::de(c, s.v, 0.0))
    }

    /// Cada componente × `k ≥ 0` (acompanha [`Soma::escala`]).
    #[must_use]
    pub fn escala(self, k: f32) -> Self {
        let k = k.max(0.0);
        Self {
            ln_r: self.ln_r.map(|x| x * k),
            delta: self.delta.map(|x| x * k),
            p: self.p * k,
        }
    }

    /// Soma componente a componente, `self` primeiro (acompanha [`Soma::junta`]).
    #[must_use]
    pub fn junta(self, o: Self) -> Self {
        Self {
            ln_r: core::array::from_fn(|i| self.ln_r[i] + o.ln_r[i]),
            delta: core::array::from_fn(|k| self.delta[k] + o.delta[k]),
            p: self.p + o.p,
        }
    }

    /// Move a fracção `f` (a MESMA de [`transfere`]) de `de` para `para`; devolve o que moveu.
    pub fn transfere(de: &mut Self, para: &mut Self, f: f32) -> Self {
        let f = f.clamp(0.0, 1.0);
        Self {
            ln_r: core::array::from_fn(|i| move_um(&mut de.ln_r[i], &mut para.ln_r[i], f)),
            delta: core::array::from_fn(|k| move_um(&mut de.delta[k], &mut para.delta[k], f)),
            p: move_um(&mut de.p, &mut para.p, f),
        }
    }
}

/// [`transfere`] da [`Soma`] e do [`Espectro`] com a mesma fracção: a regra «toda transferência move a mesma
/// fracção» com o lado espectral presente.
pub fn transfere_pigmento(
    de: (&mut Soma, &mut Espectro),
    para: (&mut Soma, &mut Espectro),
    f: f32,
) -> (Soma, Espectro) {
    (
        transfere(de.0, para.0, f),
        Espectro::transfere(de.1, para.1, f),
    )
}

/// O Pigment mix médio da fatia, `Σv·p / Σv ∈ [0,1]`; `0` sem pigmento.
#[must_use]
pub fn p_medio(s: &Soma, e: &Espectro) -> f32 {
    if s.v > 0.0 {
        (e.p / s.v).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// A cor WGM da fatia SEM o resíduo (`o_WGM` cru: a média geométrica pela matriz de volta). Para medir.
#[must_use]
pub fn oklab_wgm_cru(s: &Soma, e: &Espectro) -> Option<Lab> {
    if s.v <= 0.0 {
        return None;
    }
    let v = s.v;
    Some(oklab_de_log(&e.ln_r.map(|x| x / v)))
}

/// ⭐ A cor da fatia: `lerp_Oklab(ō, o_WGM + δ̄, p̄)`. Com `p̄ = 0` é [`Soma::cor`] AO BIT (o ramo nem calcula as
/// bandas). `None` sem pigmento. A transparência é a de [`Soma::cor`].
#[must_use]
pub fn cor(s: &Soma, e: &Espectro) -> Option<Tinta> {
    let base = s.cor()?;
    let p = p_medio(s, e);
    if p <= 0.0 {
        return Some(base);
    }
    let v = s.v;
    let ow = oklab_de_log(&e.ln_r.map(|x| x / v));
    let w: Lab = core::array::from_fn(|k| ow[k] + e.delta[k] / v);
    Some(Tinta {
        lab: core::array::from_fn(|k| base.lab[k] + (w[k] - base.lab[k]) * p),
        ..base
    })
}

#[cfg(test)]
#[path = "pigmento_tests.rs"]
mod tests;
