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
/// ⚠️ **A RESOLUÇÃO da bissecção — mais fina que a do CSS (`1e-4`), e MEDIDO porquê** (doc 127):
/// a bissecção devolve a croma só até à sua resolução, e um ULP de diferença numa comparação (o
/// dispositivo pode fundir `a·b + c`) muda o lado de uma iteração — com `1e-4` a placa e a CPU
/// divergiam `9,5e-5` no `tint` OKLCH. A `1e-6` a divergência fica na resolução nova.
const RESOLUCAO_CROMA: f32 = 1e-6;

/// As matrizes de Ottosson, as MESMAS da [`OklabColor`], dadas por bits para o WGSL as ler iguais
/// (o gate `the_matrices_are_the_oklab_colour_ones` confere-as contra os literais dela).
pub(crate) const M_LMS: [[u32; 3]; 3] = [
    [0x3ed3_0eb1, 0x3f09_4d16, 0x3d52_b909],
    [0x3e58_fd3b, 0x3f2e_4253, 0x3ddb_f2f0],
    [0x3db4_d7ec, 0x3e90_3d74, 0x3f21_4649],
];
pub(crate) const M_LAB: [[u32; 3]; 3] = [
    [0x3e57_8152, 0x3f4b_2a89, 0xbb85_6ece],
    [0x3ffd_2f0e, 0xc01b_6e0e, 0x3ee6_b438],
    [0x3cd4_34b4, 0x3f48_63bb, 0xbf4f_0560],
];
pub(crate) const M_LMS_DE_LAB: [[u32; 2]; 3] = [
    [0x3eca_ecc9, 0x3e5c_fba9],
    [0xbdd8_308c, 0xbd82_c5fa],
    [0xbdb7_437b, 0xbfa5_4f66],
];
pub(crate) const M_RGB: [[u32; 3]; 3] = [
    [0x4082_74ab, 0xc053_b18c, 0x3e6c_8362],
    [0xbfa2_5c2d, 0x4027_0644, 0xbeae_c16a],
    [0xbb89_7f53, 0xbf34_133e, 0x3fda_931e],
];

fn linha(m: [u32; 3], v: [f32; 3]) -> f32 {
    f32::from_bits(m[0]) * v[0] + f32::from_bits(m[1]) * v[1] + f32::from_bits(m[2]) * v[2]
}

/// RGB linear → OKLab `[L, a, b]` — a raiz cúbica DETERMINISTA ([`ph2d_ease::cbrt`]).
#[must_use]
pub fn to_oklab(c: [f32; 3]) -> [f32; 3] {
    let lms = M_LMS.map(|m| ph2d_ease::cbrt(linha(m, c)));
    M_LAB.map(|m| linha(m, lms))
}

/// OKLab → RGB linear (sem corte).
#[must_use]
pub fn from_oklab(o: [f32; 3]) -> [f32; 3] {
    let lms = M_LMS_DE_LAB.map(|m| {
        let v = o[0] + f32::from_bits(m[0]) * o[1] + f32::from_bits(m[1]) * o[2];
        v * v * v
    });
    M_RGB.map(|m| linha(m, lms))
}

/// OKLab → OKLCH `[L, C, h]`, `h` em VOLTAS (`0..1`), como a matiz do HSV desta crate.
#[must_use]
pub fn oklab_to_oklch(o: [f32; 3]) -> [f32; 3] {
    let c = (o[1] * o[1] + o[2] * o[2]).sqrt();
    [o[0], c, ph2d_ease::fract(ph2d_ease::atan_turn(o[2], o[1]))]
}

/// OKLCH → OKLab.
#[must_use]
pub fn oklch_to_oklab(p: [f32; 3]) -> [f32; 3] {
    [p[0], p[1] * ph2d_ease::cos_turn(p[2]), p[1] * ph2d_ease::sin_turn(p[2])]
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
    while max - min > RESOLUCAO_CROMA {
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
            // ⚠️ O CSS sai aqui quando `JND − E < ε`; nós NÃO (doc 127): essa saída é um degrau
            // que um ULP atravessa num lado e não no outro (placa × CPU `9,5e-5`), e sem ela o laço
            // converge até à resolução — a mesma cor a menos dela.
            min_na_gama = false;
            min = croma;
        } else {
            max = croma;
        }
    }
    cortado
}

/// A matiz misturada, com a «matiz sem poder» de uma cor cinza (CSS Color 4): ela herda a da outra.
/// O sentido é o do [`RampHue`] (a mesma lei do HSV desta crate), e o fim é `x − floor(x)`, que o
/// WGSL escreve igual.
fn matiz(h1: f32, c1: f32, h2: f32, c2: f32, t: f32, hue: RampHue) -> f32 {
    let (h1, h2) = match (c1 < CROMA_SEM_MATIZ, c2 < CROMA_SEM_MATIZ) {
        (true, false) => (h2, h2),
        (false, true) => (h1, h1),
        _ => (h1, h2),
    };
    let mut d = h2 - h1;
    match hue {
        RampHue::Near => {
            if d > 0.5 {
                d -= 1.0;
            } else if d < -0.5 {
                d += 1.0;
            }
        }
        RampHue::Far => {
            if (0.0..0.5).contains(&d) {
                d -= 1.0;
            } else if (-0.5..=0.0).contains(&d) {
                d += 1.0;
            }
        }
        RampHue::Ccw => {
            if d < 0.0 {
                d += 1.0;
            }
        }
        RampHue::Cw => {
            if d > 0.0 {
                d -= 1.0;
            }
        }
    }
    ph2d_ease::fract(h1 + d * t)
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
            mix_lch_hue(a, b, t, hue)
        }
    }
}

/// A mistura em OKLCH com QUALQUER sentido de matiz da crate (curto · longo · horário ·
/// anti-horário) — a do gradiente, antes de voltar à gama.
#[must_use]
pub fn mix_lch_hue(a: [f32; 4], b: [f32; 4], t: f32, hue: RampHue) -> [f32; 3] {
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    {
        {
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
