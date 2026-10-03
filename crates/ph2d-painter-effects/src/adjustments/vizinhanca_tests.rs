//! ADR-0177, P4: o espaço do núcleo de cada efeito de vizinhança. O Krita a 8 bits borra em tons
//! de ecrã pré-multiplicados — o Gaussian dele de raio 9 é o nosso ao byte —, o GIMP em luz
//! (`docs/Painter/ferramentas/oraculo_camadas_gimp/corre_vizinhanca.sh`; o gate do oráculo vive no
//! compositor, `oraculo_vizinhanca_tests`). Aqui cada efeito tem um valor EXACTO definido no espaço
//! dele: um gate relacional («o degrau fica suave») sobrevive a uma mutação de espaço.

use super::compute::{linear_to_srgb_f32, srgb_to_linear_f32};
use super::spatial::{
    AdjustWindow, apply_bloom, apply_chromatic_aberration, apply_gaussian, apply_motion_blur,
    apply_sharpen, gaussian_weights, motion_weights,
};
use super::*;

const PRETO: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BRANCO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
/// Branco a `140/255` — a cobertura translúcida separa «pré-multiplicado em luz» de «em ecrã».
const BRANCO_140: [f32; 4] = [1.0, 1.0, 1.0, 140.0 / 255.0];

/// Uma linha de `w` píxeis (`h = 1`: o passo vertical é a identidade): `esq` antes de `corte`.
fn degrau(w: usize, corte: usize, esq: [f32; 4], dir: [f32; 4]) -> Vec<[f32; 4]> {
    (0..w).map(|x| if x < corte { esq } else { dir }).collect()
}

fn linha(w: usize) -> AdjustWindow {
    AdjustWindow::full(w as u32, 1)
}

/// O núcleo simétrico `pesos[0..=half]` sobre a linha, em tons de ecrã PRÉ-MULTIPLICADOS, borda
/// por clamp — o valor que o tipo promete.
fn nucleo_em_ecra(px: &[[f32; 4]], pesos: &[f32], half: u32) -> Vec<[f32; 4]> {
    let w = px.len() as i32;
    let half = half as i32;
    (0..w)
        .map(|x| {
            let mut c = [0.0f32; 4];
            for k in -half..=half {
                let s = px[(x + k).clamp(0, w - 1) as usize];
                let p = pesos[k.unsigned_abs() as usize];
                for ch in 0..3 {
                    c[ch] += s[ch] * s[3] * p;
                }
                c[3] += s[3] * p;
            }
            [c[0] / c[3], c[1] / c[3], c[2] / c[3], c[3]]
        })
        .collect()
}

fn igual(nome: &str, obtido: &[[f32; 4]], esperado: &[[f32; 4]]) {
    for (x, (o, e)) in obtido.iter().zip(esperado).enumerate() {
        for ch in 0..4 {
            assert!(
                (o[ch] - e[ch]).abs() < 2e-5,
                "{nome}: x={x} canal {ch}: {} ≠ {} (o esperado é o núcleo em tons de ecrã)",
                o[ch],
                e[ch]
            );
        }
    }
}

#[test]
fn the_gaussian_blurs_in_premultiplied_display_tones() {
    let (pesos, half) = gaussian_weights(4.5);
    for (nome, px) in [
        ("preto|branco", degrau(32, 16, PRETO, BRANCO)),
        ("branco a 140|preto", degrau(32, 16, BRANCO_140, PRETO)),
    ] {
        let mut acc = px.clone();
        apply_gaussian(
            &GaussianBlurParams { radius: 4.5 },
            &mut acc,
            linha(px.len()),
        );
        igual(nome, &acc, &nucleo_em_ecra(&px, &pesos, half));
    }
}

#[test]
fn the_motion_blur_is_a_box_of_display_tones() {
    // distância 9 ⇒ 11 tomas: o último preto vê 5 brancos ⇒ 5/11 em tons de ecrã (em luz seria 0,72).
    let px = degrau(32, 16, PRETO, BRANCO);
    let mut acc = px.clone();
    apply_motion_blur(
        &MotionBlurParams {
            distance: 9.0,
            angle: 0.0,
        },
        &mut acc,
        linha(px.len()),
    );
    let (pesos, half) = motion_weights(9.0);
    assert_eq!(half, 5);
    igual("motion", &acc, &nucleo_em_ecra(&px, &pesos, half));
    assert!((acc[15][0] - 5.0 / 11.0).abs() < 2e-5, "{}", acc[15][0]);
}

#[test]
fn the_sharpen_is_an_unsharp_mask_of_display_tones() {
    // Cinzentos sem corte: `b + amount·(b − desfoque(b))` cabe em [0, 1].
    let (esc, cla) = ([0.25, 0.25, 0.25, 1.0], [0.75, 0.75, 0.75, 1.0]);
    let px = degrau(32, 16, esc, cla);
    let mut acc = px.clone();
    let p = SharpenParams {
        amount: 0.5,
        radius: 3.0,
        mask_edges: false,
    };
    apply_sharpen(&p, &mut acc, linha(px.len()));
    let (pesos, half) = gaussian_weights(3.0);
    let borrado = nucleo_em_ecra(&px, &pesos, half);
    let esperado: Vec<[f32; 4]> = px
        .iter()
        .zip(&borrado)
        .map(|(b, d)| {
            let v = (b[0] + p.amount * (b[0] - d[0])).clamp(0.0, 1.0);
            [v, v, v, 1.0]
        })
        .collect();
    igual("sharpen", &acc, &esperado);
}

#[test]
fn the_chromatic_aberration_gathers_premultiplied_display_tones() {
    // w = 9 ⇒ centro 4, canto 4; `red_shift = 4` ⇒ o vermelho do píxel 5 vem do píxel 6.
    let mut px = vec![PRETO; 9];
    px[6] = BRANCO_140;
    let mut acc = px.clone();
    apply_chromatic_aberration(
        &ChromaticAberrationParams {
            red_shift: 4.0,
            green_shift: 0.0,
            blue_shift: 0.0,
            falloff_center: 0.0,
        },
        &mut acc,
        linha(9),
    );
    // O vermelho pré-multiplicado do 6 (1 · 140/255) sobre a cobertura do 5 (opaco): 140/255 em
    // tons de ecrã (em luz seria 0,77).
    assert!(
        (acc[5][0] - 140.0 / 255.0).abs() < 2e-5,
        "vermelho {} ≠ 140/255",
        acc[5][0]
    );
    assert_eq!([acc[5][1], acc[5][2], acc[5][3]], [0.0, 0.0, 1.0]);
}

#[test]
fn the_bloom_stays_a_glow_of_light() {
    // O Bloom é óptico (luz somada): um cinzento escuro opaco (abaixo do limiar) ao lado do branco
    // sai `encode(decode(0,2) + Σ pesos dos brancos)`. O cinzento e não o preto: 0 e 1 são pontos
    // fixos da curva sRGB, e uma porta que esquecesse a luz passaria sobre eles.
    const CINZA: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
    let px = degrau(32, 16, CINZA, BRANCO);
    let mut acc = px.clone();
    apply_bloom(
        &BloomParams {
            threshold: 0.5,
            intensity: 1.0,
            radius: 3.0,
            falloff: 1e-3,
        },
        &mut acc,
        linha(px.len()),
    );
    let (pesos, half) = gaussian_weights(3.0);
    for x in 16 - half as usize..16 {
        let brilho: f32 = (16..=x + half as usize).map(|t| pesos[t - x]).sum::<f32>();
        let esperado = linear_to_srgb_f32(srgb_to_linear_f32(0.2) + brilho);
        assert!(
            (acc[x][0] - esperado).abs() < 2e-5,
            "x={x}: {} ≠ encode({brilho}) = {esperado}",
            acc[x][0]
        );
    }
}
