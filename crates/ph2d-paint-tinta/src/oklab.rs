//! sRGB ↔ RGB linear ↔ Oklab, em `f32` com a ordem das somas FIXA.
//!
//! Fonte das matrizes: Björn Ottosson, «A perceptual color space for image processing» (2020),
//! <https://bottosson.github.io/posts/oklab/> — domínio público / MIT. As inversas são as PUBLICADAS
//! (não invertidas aqui), as mesmas do oráculo `lei_unica.py`. EOTF sRGB: IEC 61966-2-1.
//!
//! Cada produto escalar é `(c0·x + c1·y) + c2·z`, sem `mul_add` (o rustc não contrai em FMA): o
//! backend GPU porta estas linhas na mesma ordem e com a contracção proibida (ADR-0197 §4.3).

/// Um ponto Oklab `[L, a, b]`.
pub type Lab = [f32; 3];

#[inline]
fn ponto(c: [f32; 3], x: [f32; 3]) -> f32 {
    (c[0] * x[0] + c[1] * x[1]) + c[2] * x[2]
}

/// EOTF sRGB (não linear `[0,1]` → linear), por `libm::powf` para o mesmo bit em todo SO.
#[must_use]
pub fn srgb_para_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.040_45 {
        c / 12.92
    } else {
        libm::powf((c + 0.055) / 1.055, 2.4)
    }
}

/// OETF sRGB (linear → não linear), com `clamp` a `[0,1]`.
#[must_use]
pub fn linear_para_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * libm::powf(c, 1.0 / 2.4) - 0.055
    }
}

/// RGB linear → Oklab (`M1`, raiz cúbica, `M2`).
#[must_use]
pub fn linear_para_oklab(rgb: [f32; 3]) -> Lab {
    let lms = [
        ponto([0.412_221_470_8, 0.536_332_536_3, 0.051_445_992_9], rgb),
        ponto([0.211_903_498_2, 0.680_699_545_1, 0.107_396_956_6], rgb),
        ponto([0.088_302_461_9, 0.281_718_837_6, 0.629_978_700_5], rgb),
    ];
    let r = lms.map(libm::cbrtf);
    [
        ponto([0.210_454_255_3, 0.793_617_785_0, -0.004_072_046_8], r),
        ponto([1.977_998_495_1, -2.428_592_205_0, 0.450_593_709_9], r),
        ponto([0.025_904_037_1, 0.782_771_766_2, -0.808_675_766_0], r),
    ]
}

/// Oklab → RGB linear (inversas publicadas; o cubo é `x·x·x`, sem `powf`). Pode sair de `[0,1]`
/// (cor fora da gama): quem a usa como reflectância faz o `clamp`.
#[must_use]
pub fn oklab_para_linear(lab: Lab) -> [f32; 3] {
    let r = [
        ponto([1.0, 0.396_337_777_4, 0.215_803_757_3], lab),
        ponto([1.0, -0.105_561_345_8, -0.063_854_172_8], lab),
        ponto([1.0, -0.089_484_177_5, -1.291_485_548_0], lab),
    ];
    let lms = r.map(|x| x * x * x);
    [
        ponto([4.076_741_662_1, -3.307_711_591_3, 0.230_969_929_2], lms),
        ponto([-1.268_438_004_6, 2.609_757_401_1, -0.341_319_396_5], lms),
        ponto([-0.004_196_086_3, -0.703_418_614_7, 1.707_614_701_0], lms),
    ]
}

/// sRGB não linear → Oklab.
#[must_use]
pub fn srgb_para_oklab(srgb: [f32; 3]) -> Lab {
    linear_para_oklab(srgb.map(srgb_para_linear))
}

/// Oklab → sRGB não linear (`clamp` por canal).
#[must_use]
pub fn oklab_para_srgb(lab: Lab) -> [f32; 3] {
    oklab_para_linear(lab).map(linear_para_srgb)
}
