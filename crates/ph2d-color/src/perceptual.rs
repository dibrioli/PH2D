//! ⭐ **MISTURAR CORES COMO O OLHO VÊ** (Motion Nodes, ciclo 18 — doc 127).
//!
//! A mistura em RGB linear de dois saturados passa por um cinza «lamacento» no meio (vermelho →
//! azul dá um roxo escuro); em OKLab a luminosidade percebida anda em linha recta, e em OKLCH a
//! matiz roda pelo círculo (o caminho curto ou o longo, [`RampHue`]). É a mistura do CSS Color 4
//! (`color-mix(in oklab …)`, `in oklch shorter|longer hue`).
//!
//! ⚠️ **A mistura em OKLCH sai da gama do ecrã** (vermelho → azul pelo caminho longo passa por um
//! verde que o sRGB não tem): o resultado volta à gama pelo algoritmo do CSS Color 4
//! ([`gamut_map`] — reduz a CROMA em OKLCH por bissecção até o corte ficar a menos de uma JND),
//! nunca por um corte por canal que mudaria a matiz.
//!
//! Conferido contra o culori 4.0.2 (MIT), corrido sobre as nossas cores
//! (`tests/fixtures/culori.txt`, `docs/Motion Nodes/ferramentas/culori_oklab_oracle/`).
//!
//! Entradas e saídas em RGBA **linear, alfa direito** (a coluna `tint` da casa); o alfa mistura-se
//! em linha recta.

use crate::color_ramp::RampHue;
use crate::{LinearRgba, OklabColor};

/// O espaço de uma mistura de cor — a escada é wire format (um param guarda o NÚMERO).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MixSpace {
    /// RGB linear — a mistura de sempre, ao bit.
    #[default]
    Linear,
    /// OKLab — luminosidade percebida em linha recta.
    Oklab,
    /// OKLCH, a matiz pelo caminho CURTO.
    OklchShorter,
    /// OKLCH, a matiz pelo caminho LONGO.
    OklchLonger,
}

impl MixSpace {
    /// Da escada `0..=3` (fora dela: `Linear`).
    #[must_use]
    pub fn from_index(i: i32) -> Self {
        match i {
            1 => Self::Oklab,
            2 => Self::OklchShorter,
            3 => Self::OklchLonger,
            _ => Self::Linear,
        }
    }
}

/// A croma abaixo da qual a matiz é «sem poder» (CSS Color 4 §12.4): uma cor quase cinza não puxa
/// a matiz da outra para um ângulo arbitrário.
pub const CROMA_SEM_MATIZ: f32 = 2e-4;
/// A JND do mapeamento de gama do CSS Color 4 (distância euclidiana em OKLab).
const JND: f32 = 0.02;
/// A precisão da bissecção da croma.
const EPS_CROMA: f32 = 1e-4;

/// RGB linear → OKLab `[L, a, b]`.
#[must_use]
pub fn to_oklab(c: [f32; 3]) -> [f32; 3] {
    let o = OklabColor::from_linear(LinearRgba::new(c[0], c[1], c[2], 1.0));
    [o.l, o.a, o.b]
}

/// OKLab → RGB linear (sem corte).
#[must_use]
pub fn from_oklab(o: [f32; 3]) -> [f32; 3] {
    let l = OklabColor::new(o[0], o[1], o[2], 1.0).to_linear();
    [l.r(), l.g(), l.b()]
}

/// OKLab → OKLCH `[L, C, h]`, `h` em VOLTAS (`0..1`), como a matiz do HSV desta crate.
#[must_use]
pub fn oklab_to_oklch(o: [f32; 3]) -> [f32; 3] {
    let c = o[1].hypot(o[2]);
    let h = (o[2].atan2(o[1]) / std::f32::consts::TAU).rem_euclid(1.0);
    [o[0], c, h]
}

/// OKLCH → OKLab.
#[must_use]
pub fn oklch_to_oklab(p: [f32; 3]) -> [f32; 3] {
    let (s, c) = (p[2] * std::f32::consts::TAU).sin_cos();
    [p[0], p[1] * c, p[1] * s]
}

fn na_gama(c: [f32; 3]) -> bool {
    c.iter().all(|v| (0.0..=1.0).contains(v))
}

fn corta(c: [f32; 3]) -> [f32; 3] {
    c.map(|v| v.clamp(0.0, 1.0))
}

fn distancia(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// ⭐ **O mapeamento de gama do CSS Color 4** (§13.2, «binary search in OKLCH»): uma cor OKLCH
/// fora do sRGB volta a ele baixando a CROMA (a luminosidade e a matiz ficam), até o corte por canal
/// estar a menos de uma JND. Devolve RGB linear em `[0, 1]`.
#[must_use]
pub fn gamut_map(lch: [f32; 3]) -> [f32; 3] {
    let rgb = from_oklab(oklch_to_oklab(lch));
    if na_gama(rgb) {
        return rgb;
    }
    if lch[0] >= 1.0 {
        return [1.0; 3];
    }
    if lch[0] <= 0.0 {
        return [0.0; 3];
    }
    let mut corrente = lch;
    let cortado = corta(rgb);
    if distancia(to_oklab(cortado), oklch_to_oklab(corrente)) < JND {
        return cortado;
    }
    let (mut min, mut max) = (0.0_f32, lch[1]);
    let mut min_na_gama = true;
    let mut cortado = cortado;
    while max - min > EPS_CROMA {
        let croma = (min + max) * 0.5;
        corrente[1] = croma;
        let rgb = from_oklab(oklch_to_oklab(corrente));
        if min_na_gama && na_gama(rgb) {
            min = croma;
            continue;
        }
        cortado = corta(rgb);
        let e = distancia(to_oklab(cortado), oklch_to_oklab(corrente));
        if e < JND {
            if JND - e < EPS_CROMA {
                return cortado;
            }
            min_na_gama = false;
            min = croma;
        } else {
            max = croma;
        }
    }
    cortado
}

/// A matiz misturada, com a «matiz sem poder» de uma cor cinza (CSS Color 4): ela herda a da outra.
fn matiz(h1: f32, c1: f32, h2: f32, c2: f32, t: f32, hue: RampHue) -> f32 {
    let (h1, h2) = match (c1 < CROMA_SEM_MATIZ, c2 < CROMA_SEM_MATIZ) {
        (true, false) => (h2, h2),
        (false, true) => (h1, h1),
        _ => (h1, h2),
    };
    crate::color_ramp::lerp_hue_pub(h1, h2, t, hue)
}

/// A mistura no espaço, ANTES de voltar à gama: OKLCH `[L, C, h]` (o `Linear` não passa aqui).
#[must_use]
pub fn mix_lch(a: [f32; 4], b: [f32; 4], t: f32, space: MixSpace) -> [f32; 3] {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    match space {
        MixSpace::Linear | MixSpace::Oklab => {
            let (p, q) = (to_oklab([a[0], a[1], a[2]]), to_oklab([b[0], b[1], b[2]]));
            oklab_to_oklch([lerp(p[0], q[0]), lerp(p[1], q[1]), lerp(p[2], q[2])])
        }
        MixSpace::OklchShorter | MixSpace::OklchLonger => {
            let hue = if space == MixSpace::OklchShorter {
                RampHue::Near
            } else {
                RampHue::Far
            };
            let p = oklab_to_oklch(to_oklab([a[0], a[1], a[2]]));
            let q = oklab_to_oklch(to_oklab([b[0], b[1], b[2]]));
            [
                lerp(p[0], q[0]),
                lerp(p[1], q[1]),
                matiz(p[2], p[1], q[2], q[1], t, hue),
            ]
        }
    }
}

/// ⭐ **A mistura de duas cores RGBA lineares** no espaço `space`, em `t ∈ [0, 1]` — de volta à
/// gama do ecrã pelo [`gamut_map`] (o `Linear` é a mistura de sempre, ao bit, sem mapeamento).
#[must_use]
pub fn mix(a: [f32; 4], b: [f32; 4], t: f32, space: MixSpace) -> [f32; 4] {
    let alfa = a[3] + (b[3] - a[3]) * t;
    if space == MixSpace::Linear {
        let lerp = |x: f32, y: f32| x + (y - x) * t;
        return [lerp(a[0], b[0]), lerp(a[1], b[1]), lerp(a[2], b[2]), alfa];
    }
    let rgb = gamut_map(mix_lch(a, b, t, space));
    [rgb[0], rgb[1], rgb[2], alfa]
}

#[cfg(test)]
#[path = "perceptual_tests.rs"]
mod tests;
