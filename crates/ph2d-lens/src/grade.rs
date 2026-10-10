//! **O GRADE** — exposição · balanço de branco · contraste · saturação · LUT, nesta ordem, em luz
//! LINEAR (a LUT sobre os valores codificados, que é o domínio dela).
//!
//! Cada lei vem de uma referência publicada, nunca de uma constante inventada:
//! - exposição: `c × 2^stops`;
//! - balanço de branco: adaptação de von Kries no espaço LMS do CAT02 (CIECAM02), da luz de
//!   referência (o locus de Planck a 6500 K) para a luz pedida (o locus pela aproximação de Kim et
//!   al. 2002, mais o desvio `Duv` perpendicular a ele no CIE 1960 `uv`); matrizes sRGB↔XYZ da
//!   IEC 61966-2-1. A matriz é calculada aqui, uma vez por quadro, e viaja ao WGSL como uniform;
//! - contraste: potência à volta do cinzento médio `0,18` (o contraste do ACEScc, em linear);
//! - saturação: o croma do OKLab escalado (a mesma conta, nas duas línguas, do ciclo 18).

use crate::cubo::Cubo;
use ph2d_color::perceptual::{from_oklab, to_oklab};
use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_unit};

/// Os params do artista.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    /// Em stops (`+1` = o dobro da luz).
    pub exposure: f32,
    /// `-1..1`: `+` é a luz mais QUENTE (`6500 K × 2^(-1,3 t)`).
    pub temperatura: f32,
    /// `-1..1`: `+` é magenta, `-` é verde (`Duv = -0,02 t`).
    pub matiz: f32,
    /// `-1..1`: o expoente à volta de `0,18` é `2^contraste`.
    pub contraste: f32,
    /// `-1..1`: `-1` é cinzento, `+1` é o dobro do croma.
    pub saturacao: f32,
    /// `0..1`: quanto da LUT entra (sem LUT carregada não faz nada).
    pub lut_mix: f32,
}

impl Grade {
    #[must_use]
    pub fn neutra(&self, tem_lut: bool) -> bool {
        self.exposure == 0.0
            && self.temperatura == 0.0
            && self.matiz == 0.0
            && self.contraste == 0.0
            && self.saturacao == 0.0
            && (!tem_lut || self.lut_mix == 0.0)
    }
}

/// O cinzento médio do contraste.
pub const PIVO: f32 = 0.18;
/// A temperatura de referência (a neutra).
pub const KELVIN_REF: f64 = 6500.0;
/// O alcance do locus aproximado de Kim et al.
pub const KELVIN_MIN: f64 = 1667.0;
pub const KELVIN_MAX: f64 = 25000.0;
/// `Duv` por unidade de `matiz`.
pub const DUV_POR_MATIZ: f64 = -0.02;

type M3 = [[f64; 3]; 3];

const RGB_PARA_XYZ: M3 = [
    [0.412_456_4, 0.357_576_1, 0.180_437_5],
    [0.212_672_9, 0.715_152_2, 0.072_175_0],
    [0.019_333_9, 0.119_192_0, 0.950_304_1],
];
const XYZ_PARA_RGB: M3 = [
    [3.240_454_2, -1.537_138_5, -0.498_531_4],
    [-0.969_266_0, 1.876_010_8, 0.041_556_0],
    [0.055_643_4, -0.204_025_9, 1.057_225_2],
];
const CAT02: M3 = [
    [0.7328, 0.4296, -0.1624],
    [-0.7036, 1.6975, 0.0061],
    [0.0030, 0.0136, 0.9834],
];
const CAT02_INV: M3 = [
    [1.096_124, -0.278_869, 0.182_745],
    [0.454_369, 0.473_533, 0.072_098],
    [-0.009_628, -0.005_698, 1.015_326],
];

fn mul(a: &M3, b: &M3) -> M3 {
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn vec(m: &M3, v: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|i| (0..3).map(|k| m[i][k] * v[k]).sum())
}

/// O locus de Planck em `xy` (Kim et al. 2002).
#[must_use]
pub fn planck_xy(kelvin: f64) -> [f64; 2] {
    let t = kelvin.clamp(KELVIN_MIN, KELVIN_MAX);
    let (t1, t2, t3) = (1e3 / t, 1e6 / (t * t), 1e9 / (t * t * t));
    let x = if t <= 4000.0 {
        -0.266_123_9 * t3 - 0.234_358_9 * t2 + 0.877_695_6 * t1 + 0.179_910
    } else {
        -3.025_846_9 * t3 + 2.107_037_9 * t2 + 0.222_634_7 * t1 + 0.240_390
    };
    let y = if t <= 2222.0 {
        -1.106_381_4 * x * x * x - 1.348_110_20 * x * x + 2.185_558_32 * x - 0.202_196_83
    } else if t <= 4000.0 {
        -0.954_947_6 * x * x * x - 1.374_185_93 * x * x + 2.091_370_15 * x - 0.167_488_67
    } else {
        3.081_758_0 * x * x * x - 5.873_386_70 * x * x + 3.751_129_97 * x - 0.370_014_83
    };
    [x, y]
}

fn xy_para_uv([x, y]: [f64; 2]) -> [f64; 2] {
    let d = -2.0 * x + 12.0 * y + 3.0;
    [4.0 * x / d, 6.0 * y / d]
}

fn uv_para_xy([u, v]: [f64; 2]) -> [f64; 2] {
    let d = 2.0 * u - 8.0 * v + 4.0;
    [3.0 * u / d, 2.0 * v / d]
}

/// A cromaticidade da luz `kelvin` desviada `duv` do locus (positivo = para o verde).
#[must_use]
pub fn branco_uv(kelvin: f64, duv: f64) -> [f64; 2] {
    let uv = xy_para_uv(planck_xy(kelvin));
    let a = xy_para_uv(planck_xy(kelvin / 1.01));
    let b = xy_para_uv(planck_xy(kelvin * 1.01));
    let (tu, tv) = (b[0] - a[0], b[1] - a[1]);
    let n = tu.hypot(tv);
    let (mut nu, mut nv) = (-tv / n, tu / n);
    if nv < 0.0 {
        (nu, nv) = (-nu, -nv);
    }
    [uv[0] + duv * nu, uv[1] + duv * nv]
}

fn lms_do_branco(uv: [f64; 2]) -> [f64; 3] {
    let [x, y] = uv_para_xy(uv);
    vec(&CAT02, [x / y, 1.0, (1.0 - x - y) / y])
}

/// O kelvin que `temperatura` pede.
#[must_use]
pub fn kelvin(temperatura: f32) -> f64 {
    (KELVIN_REF * (-1.3 * f64::from(temperatura)).exp2()).clamp(KELVIN_MIN, KELVIN_MAX)
}

/// **A matriz do balanço de branco** (RGB linear → RGB linear). Neutra = a identidade EXACTA.
#[must_use]
pub fn matriz_balanco(temperatura: f32, matiz: f32) -> [[f32; 3]; 3] {
    if temperatura == 0.0 && matiz == 0.0 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let refe = lms_do_branco(branco_uv(KELVIN_REF, 0.0));
    let alvo = lms_do_branco(branco_uv(
        kelvin(temperatura),
        DUV_POR_MATIZ * f64::from(matiz),
    ));
    let d: M3 = [
        [alvo[0] / refe[0], 0.0, 0.0],
        [0.0, alvo[1] / refe[1], 0.0],
        [0.0, 0.0, alvo[2] / refe[2]],
    ];
    let m = mul(
        &XYZ_PARA_RGB,
        &mul(&CAT02_INV, &mul(&d, &mul(&CAT02, &RGB_PARA_XYZ))),
    );
    m.map(|l| l.map(|v| v as f32))
}

/// O grade de uma cor LINEAR.
#[must_use]
pub fn aplica(c: [f32; 3], g: &Grade, balanco: &[[f32; 3]; 3], lut: Option<&Cubo>) -> [f32; 3] {
    let e = g.exposure.exp2();
    let c = c.map(|v| v * e);
    let c = [0, 1, 2].map(|i| balanco[i][0] * c[0] + balanco[i][1] * c[1] + balanco[i][2] * c[2]);
    let k = g.contraste.exp2();
    let c = c.map(|v| {
        if v > 0.0 {
            PIVO * (v / PIVO).powf(k)
        } else {
            0.0
        }
    });
    let mut ok = to_oklab(c);
    ok[1] *= 1.0 + g.saturacao;
    ok[2] *= 1.0 + g.saturacao;
    let c = from_oklab(ok).map(|v| v.max(0.0));
    match lut {
        Some(l) if g.lut_mix > 0.0 => {
            let cod = c.map(|v| linear_to_srgb_unit(v.min(1.0)));
            let olhado = l.tetraedrica(cod).map(srgb_to_linear_unit);
            [0, 1, 2].map(|i| c[i] + (olhado[i] - c[i]) * g.lut_mix.min(1.0))
        }
        _ => c,
    }
}
