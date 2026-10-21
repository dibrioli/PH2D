//! A ponta de IMAGEM pela ÁREA do píxel ([`crate::GrainLaw::area`], J4 `texturas`).

use super::super::{GrainLaw, ImageMask, TextureKind, TextureMapping, TextureSettings, dab_basis};
use super::sample_shape;
use crate::footprint::FootprintDeform;

/// Um RUÍDO de texels acesos e apagados (um bit de um gerador inteiro) numa imagem de 128²: a média de qualquer
/// área larga é ~½, e um ponto lê 0, 1 ou o que o bilinear lhe der.
fn riscas() -> Vec<u8> {
    let mut e = 0x51A7_u64;
    (0..128 * 128)
        .map(|_| {
            e = e.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            if (e >> 40) & 1 == 1 { 255 } else { 0 }
        })
        .collect()
}

fn ponta(area: bool) -> TextureSettings {
    TextureSettings {
        kind: TextureKind::Image,
        mapping: TextureMapping::ViewPlane,
        lei: GrainLaw { area, ..GrainLaw::NEUTRA },
        ..Default::default()
    }
}

/// Os valores dos píxeis do MIOLO de um dab de raio `r` (os que ficam inteiros dentro da ponta).
fn miolo(area: bool, r: f32) -> Vec<f32> {
    let lum = riscas();
    let img = ImageMask { lum: &lum, width: 128, height: 128 };
    let s = ponta(area);
    let b = dab_basis(&s, &mut 0u64, [256.0, 256.0], FootprintDeform::identity());
    let c = [100.0_f32, 100.0];
    let mut v = Vec::new();
    for py in 0..200 {
        for px in 0..200 {
            let (dx, dy) = (px as f32 + 0.5 - c[0], py as f32 + 0.5 - c[1]);
            if dx.abs() + 0.5 < 0.8 * r && dy.abs() + 0.5 < 0.8 * r {
                v.push(sample_shape(&s, &b, px, py, c, r, Some(&img)));
            }
        }
    }
    v
}

/// ⭐ **Reduzida, a imagem lê-se pela área**: a 5 px (raio 2,5 e 4) cada píxel do miolo é a MÉDIA do ruído
/// (½ ± 0,15) — o carimbo pequeno é uma mancha contínua, não pontos acesos e apagados. Desligada, a amostra
/// no centro do píxel cai nas riscas ao acaso (o pontilhado do Foliage a 5 px).
#[test]
fn a_imagem_reduzida_le_se_pela_area_do_pixel() {
    for r in [2.5_f32, 4.0] {
        let com = miolo(true, r);
        assert!(!com.is_empty());
        for (k, v) in com.iter().enumerate() {
            assert!((v - 0.5).abs() <= 0.15, "raio {r}, píxel {k}: {v} (a média do ruído é ½)");
        }
    }
    // O controlo: sem a lei, o mesmo miolo afasta-se de ½ (a amostra num ponto).
    let sem = miolo(false, 4.0);
    assert!(sem.iter().any(|v| (v - 0.5).abs() > 0.25), "{sem:?}");
}

/// Ampliada (mais píxeis que texels), a lei é a amostra no centro do píxel AO BIT — e desligada nunca muda.
#[test]
fn ampliada_a_area_e_a_amostra_de_sempre() {
    assert_eq!(miolo(true, 200.0), miolo(false, 200.0));
}

/// A forma em unidades do dab numa grelha de píxeis (`passo = 1/raio`, a rota das camadas de cor) lê a MESMA área
/// que a forma em píxeis; sem a lei é a amostra de sempre ao bit.
#[test]
fn a_forma_unitaria_le_a_mesma_area() {
    use super::{sample_shape_silhouette_unit, sample_shape_silhouette_unit_px};
    let lum = riscas();
    let img = ImageMask { lum: &lum, width: 128, height: 128 };
    let (c, r) = ([100.0_f32, 100.0], 4.0_f32);
    for area in [true, false] {
        let s = ponta(area);
        let b = dab_basis(&s, &mut 0u64, [256.0, 256.0], FootprintDeform::identity());
        for py in 97..103 {
            for px in 97..103 {
                let (u, v) = ((px as f32 + 0.5 - c[0]) / r, (py as f32 + 0.5 - c[1]) / r);
                let un = sample_shape_silhouette_unit_px(&s, &b, u, v, 1.0 / r, Some(&img));
                let pix = sample_shape(&s, &b, px, py, c, r, Some(&img));
                assert!((un - pix).abs() < 1e-4, "área {area} ({px}, {py}): {un} × {pix}");
                if !area {
                    assert_eq!(un, sample_shape_silhouette_unit(&s, &b, u, v, Some(&img)));
                }
            }
        }
    }
}
