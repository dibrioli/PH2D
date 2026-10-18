//! **A cor de cada fio em HSLuv** (W2, prompt 48): o ruído de cor por fio do Rebelle (`bristle_noise`, doc 02
//! §2.4) é em HSLuv — o matiz e a saturação mexem-se sem mudar a luminosidade percebida, e a luminosidade
//! mexe-se em passos iguais aos olhos. A conversão é a de referência do HSLuv (sRGB D65 ↔ CIELUV ↔ LCh, com
//! o croma máximo do gamute para cada L e H), escrita da especificação pública.

const M: [[f64; 3]; 3] = [
    [
        3.240_969_941_904_521,
        -1.537_383_177_570_093,
        -0.498_610_760_293,
    ],
    [
        -0.969_243_636_280_87,
        1.875_967_501_507_72,
        0.041_555_057_407_175,
    ],
    [
        0.055_630_079_696_993,
        -0.203_976_958_888_97,
        1.056_971_514_242_878,
    ],
];
const M_INV: [[f64; 3]; 3] = [
    [
        0.412_390_799_265_95,
        0.357_584_339_383_87,
        0.180_480_788_401_83,
    ],
    [
        0.212_639_005_871_51,
        0.715_168_678_767_75,
        0.072_192_315_360_733,
    ],
    [
        0.019_330_818_715_591,
        0.119_194_779_794_62,
        0.950_532_152_249_66,
    ],
];
const REF_U: f64 = 0.197_830_006_642_83;
const REF_V: f64 = 0.468_319_994_938_79;
const KAPPA: f64 = 903.296_296_296_296_3;
const EPSILON: f64 = 0.008_856_451_679_035_631;

/// `rgb` (sRGB `0..255`) com o ruído `[dh voltas, ds, dl]` (`ds`, `dl` em fracções de 100) somado em HSLuv;
/// a saturação e a luminosidade presas a `[0, 100]`.
#[must_use]
pub fn com_ruido(rgb: [f32; 3], d: [f64; 3]) -> [f32; 3] {
    let [h, s, l] = rgb_para_hsluv(rgb.map(|c| f64::from(c) / 255.0));
    let hsl = [
        (h + 360.0 * d[0]).rem_euclid(360.0),
        (s + 100.0 * d[1]).clamp(0.0, 100.0),
        (l + 100.0 * d[2]).clamp(0.0, 100.0),
    ];
    hsluv_para_rgb(hsl).map(|c| (c.clamp(0.0, 1.0) * 255.0) as f32)
}

/// sRGB `0..1` → HSLuv (`H` graus, `S` e `L` em `0..100`).
#[must_use]
pub fn rgb_para_hsluv(rgb: [f64; 3]) -> [f64; 3] {
    let lin = rgb.map(|c| {
        if c > 0.040_45 {
            ((c + 0.055) / 1.055).powf(2.4)
        } else {
            c / 12.92
        }
    });
    let [x, y, z] = M_INV.map(|r| r[0] * lin[0] + r[1] * lin[1] + r[2] * lin[2]);
    let l = if y <= EPSILON {
        y * KAPPA
    } else {
        116.0 * y.cbrt() - 16.0
    };
    if l < 1e-8 {
        return [0.0, 0.0, 0.0];
    }
    let div = x + 15.0 * y + 3.0 * z;
    let (u, v) = (
        13.0 * l * (4.0 * x / div - REF_U),
        13.0 * l * (9.0 * y / div - REF_V),
    );
    let c = u.hypot(v);
    let h = if c < 1e-8 {
        0.0
    } else {
        v.atan2(u).to_degrees().rem_euclid(360.0)
    };
    if l > 99.999_999_9 {
        return [h, 0.0, 100.0];
    }
    [h, c / croma_maximo(l, h) * 100.0, l]
}

/// HSLuv → sRGB `0..1` (sem prender).
#[must_use]
pub fn hsluv_para_rgb([h, s, l]: [f64; 3]) -> [f64; 3] {
    if l > 99.999_999_9 {
        return [1.0; 3];
    }
    if l < 1e-8 {
        return [0.0; 3];
    }
    let c = croma_maximo(l, h) / 100.0 * s;
    let hr = h.to_radians();
    let (u, v) = (hr.cos() * c, hr.sin() * c);
    let y = if l <= 8.0 {
        l / KAPPA
    } else {
        ((l + 16.0) / 116.0).powi(3)
    };
    let (vu, vv) = (u / (13.0 * l) + REF_U, v / (13.0 * l) + REF_V);
    let x = -(9.0 * y * vu) / ((vu - 4.0) * vv - vu * vv);
    let z = (9.0 * y - 15.0 * vv * y - vv * x) / (3.0 * vv);
    M.map(|r| r[0] * x + r[1] * y + r[2] * z).map(|c| {
        if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    })
}

/// O croma máximo dentro do gamute sRGB para a luminosidade `l` e o matiz `h` (graus).
fn croma_maximo(l: f64, h: f64) -> f64 {
    let hr = h.to_radians();
    let sub1 = (l + 16.0).powi(3) / 1_560_896.0;
    let sub2 = if sub1 > EPSILON { sub1 } else { l / KAPPA };
    let mut min = f64::MAX;
    for [m1, m2, m3] in M {
        for t in [0.0, 1.0] {
            let top1 = (284_517.0 * m1 - 94_839.0 * m3) * sub2;
            let top2 =
                (838_422.0 * m3 + 769_860.0 * m2 + 731_718.0 * m1) * l * sub2 - 769_860.0 * t * l;
            let bottom = (632_260.0 * m3 - 126_452.0 * m2) * sub2 + 126_452.0 * t;
            let (slope, intercept) = (top1 / bottom, top2 / bottom);
            let len = intercept / (hr.sin() - slope * hr.cos());
            if len >= 0.0 {
                min = min.min(len);
            }
        }
    }
    min
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Os valores de referência do HSLuv (o vermelho, o azul, o verde e um cinzento) e a ida-e-volta.
    #[test]
    fn os_valores_de_referencia_e_a_ida_e_volta() {
        for (rgb, hsl) in [
            (
                [1.0, 0.0, 0.0],
                [12.177_050_630_061_776, 100.0, 53.237_115_595_429_33],
            ),
            (
                [0.0, 0.0, 1.0],
                [265.874_320_218_177_9, 100.0, 32.300_872_903_980_02],
            ),
        ] {
            let got = rgb_para_hsluv(rgb);
            for k in 0..3 {
                assert!((got[k] - hsl[k]).abs() < 1e-6, "{rgb:?}: {got:?} × {hsl:?}");
            }
        }
        let cinza = rgb_para_hsluv([0.5; 3]);
        assert!(
            cinza[1] < 1e-6 && (cinza[2] - 53.388_965_822_3).abs() < 1e-4,
            "{cinza:?}"
        );
        for i in 0..200 {
            let c = [i % 7, i % 11, i % 13].map(|v| f64::from(v as u8) / 13.0 + 0.01);
            let back = hsluv_para_rgb(rgb_para_hsluv(c));
            for k in 0..3 {
                assert!((back[k] - c[k]).abs() < 1e-9, "{c:?} → {back:?}");
            }
        }
    }

    /// O ruído zero devolve a cor (ao arredondamento de 1/255); a luminosidade mexe a luminosidade.
    #[test]
    fn o_ruido_mexe_o_que_diz() {
        let azul = [23.0, 101.0, 179.0];
        let igual = com_ruido(azul, [0.0; 3]);
        assert!(
            (0..3).all(|k| (igual[k] - azul[k]).abs() <= 1.0),
            "{igual:?}"
        );
        let claro = rgb_para_hsluv(com_ruido(azul, [0.0, 0.0, 0.1]).map(|c| f64::from(c) / 255.0));
        let base = rgb_para_hsluv(azul.map(|c| f64::from(c) / 255.0));
        assert!(
            (claro[2] - base[2] - 10.0).abs() < 0.6,
            "{claro:?} × {base:?}"
        );
        assert!((claro[0] - base[0]).abs() < 2.0);
    }
}
