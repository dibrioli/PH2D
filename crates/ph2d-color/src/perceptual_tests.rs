//! Os gates da mistura perceptual contra o ORÁCULO corrido (culori 4.0.2, MIT —
//! `tests/fixtures/culori.txt`, gerado por `docs/Motion Nodes/ferramentas/culori_oklab_oracle/`).

use super::*;

const FIXTURA: &str = include_str!("../tests/fixtures/culori.txt");

fn nums(s: &str) -> Vec<f32> {
    s.split([' ', ','])
        .filter(|x| !x.is_empty())
        .map(|x| x.parse().expect("numero"))
        .collect()
}

fn cor(nome: &str) -> [f32; 4] {
    match nome {
        "vermelho" => [1.0, 0.0, 0.0, 1.0],
        "verde" => [0.0, 1.0, 0.0, 1.0],
        "azul" => [0.0, 0.0, 1.0, 1.0],
        "amarelo" => [1.0, 1.0, 0.0, 1.0],
        "ciano" => [0.0, 1.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0, 1.0],
        "branco" => [1.0, 1.0, 1.0, 1.0],
        "preto" => [0.0, 0.0, 0.0, 1.0],
        "laranja" => [1.0, 0.25, 0.02, 1.0],
        "cinza" => [0.18, 0.18, 0.18, 1.0],
        "roxo" => [0.2, 0.02, 0.6, 1.0],
        n => panic!("cor {n}"),
    }
}

fn espaco(e: &str) -> MixSpace {
    match e {
        "lrgb" => MixSpace::Linear,
        "oklab" => MixSpace::Oklab,
        "oklch_curta" => MixSpace::OklchShorter,
        "oklch_longa" => MixSpace::OklchLonger,
        e => panic!("espaco {e}"),
    }
}

/// `(par, espaço, valores)` das linhas com o prefixo dado.
fn linhas(prefixo: &str) -> Vec<(String, String, Vec<f32>)> {
    FIXTURA
        .lines()
        .filter_map(|l| l.strip_prefix(prefixo))
        .map(|l| {
            let (cab, v) = l.split_once(" | ").expect("separador");
            let (par, e) = cab.split_once(' ').expect("par e espaco");
            (par.to_string(), e.to_string(), nums(v))
        })
        .collect()
}

/// Uma ponta cinza tem matiz «sem poder» (CSS Color 4) — nós herdamos a da outra; o culori, numa
/// cor que vem de RGB, guarda o ângulo do resto de arredondamento. Nesses pares a mistura OKLCH
/// difere POR DESENHO e não se compara.
fn tem_cinza(par: &str) -> bool {
    par.split('-')
        .any(|c| matches!(c, "branco" | "preto" | "cinza"))
}

#[test]
fn the_conversions_match_the_oracle() {
    for l in FIXTURA.lines().filter_map(|l| l.strip_prefix("CONV ")) {
        let mut p = l.split(" | ");
        let nome = p.next().expect("nome");
        let lab = nums(p.next().expect("oklab"));
        let lch = nums(p.next().expect("oklch"));
        let c = cor(nome);
        let o = to_oklab([c[0], c[1], c[2]]);
        for k in 0..3 {
            assert!(
                (o[k] - lab[k]).abs() < 2e-5,
                "{nome} oklab[{k}]: {} × {}",
                o[k],
                lab[k]
            );
        }
        let h = oklab_to_oklch(o);
        assert!((h[1] - lch[1]).abs() < 2e-5, "{nome} C");
        if h[1] > 1e-3 {
            let d = (h[2] * 360.0 - lch[2]).rem_euclid(360.0);
            assert!(
                d.min(360.0 - d) < 0.01,
                "{nome} h: {} × {}",
                h[2] * 360.0,
                lch[2]
            );
        }
    }
}

#[test]
fn the_mix_before_the_gamut_matches_the_oracle() {
    let mut n = 0;
    for (par, e, v) in linhas("MIX ") {
        let sp = espaco(&e);
        if sp != MixSpace::Linear && sp != MixSpace::Oklab && tem_cinza(&par) {
            continue;
        }
        let (a, b) = par.split_once('-').expect("par");
        for k in 0..=8_u8 {
            let t = f32::from(k) / 8.0;
            let r = if sp == MixSpace::Linear {
                let m = mix(cor(a), cor(b), t, sp);
                [m[0], m[1], m[2]]
            } else {
                from_oklab(oklch_to_oklab(mix_lch(cor(a), cor(b), t, sp)))
            };
            for c in 0..3 {
                let o = v[usize::from(k) * 3 + c];
                assert!(
                    (r[c] - o).abs() < 5e-5,
                    "{par} {e} t {t} canal {c}: {} × {o}",
                    r[c]
                );
            }
            n += 1;
        }
    }
    assert!(n >= 150, "a fixtura encolheu: {n}");
}

#[test]
fn the_gamut_mapping_matches_the_oracle() {
    let mut pior = 0.0_f32;
    for (par, e, v) in linhas("GAMUT ") {
        let sp = espaco(&e);
        if sp != MixSpace::Oklab && tem_cinza(&par) {
            continue;
        }
        let (a, b) = par.split_once('-').expect("par");
        for k in 0..=8_u8 {
            let t = f32::from(k) / 8.0;
            let r = mix(cor(a), cor(b), t, sp);
            for c in 0..3 {
                let o = v[usize::from(k) * 3 + c];
                pior = pior.max((r[c] - o).abs());
                assert!((0.0..=1.0).contains(&r[c]), "{par} {e} fora da gama: {r:?}");
            }
        }
    }
    eprintln!("gama: pior |nosso − culori| = {pior:e}");
    assert!(
        pior < 2e-3,
        "o mapeamento de gama diverge do oraculo: {pior:e}"
    );
}

#[test]
fn the_linear_space_is_the_old_lerp_bit_for_bit() {
    let (a, b) = ([0.9, 0.1, 0.3, 1.0], [0.05, 0.7, 0.2, 0.5]);
    for k in 0..=64_u8 {
        let t = f32::from(k) / 64.0;
        let m = mix(a, b, t, MixSpace::Linear);
        for c in 0..4 {
            assert_eq!(m[c].to_bits(), (a[c] + (b[c] - a[c]) * t).to_bits());
        }
    }
}

/// O motivo do ciclo, MEDIDO (e com a palavra certa): a meio de azul → amarelo o RGB linear passa
/// pelo CINZA (a croma cai a quase zero) e o OKLCH mantém a cor; a meio de preto → branco o RGB
/// linear já está CLARO demais (a luminosidade percebida não anda por igual) e o OKLab fica a meio.
#[test]
fn the_perceptual_spaces_keep_the_colour_and_the_even_steps() {
    let lch = |c: [f32; 4]| oklab_to_oklch(to_oklab([c[0], c[1], c[2]]));
    let (az, am) = (cor("azul"), cor("amarelo"));
    let c_lin = lch(mix(az, am, 0.5, MixSpace::Linear))[1];
    let c_lch = lch(mix(az, am, 0.5, MixSpace::OklchShorter))[1];
    let (p, b) = (cor("preto"), cor("branco"));
    let l_lin = lch(mix(p, b, 0.5, MixSpace::Linear))[0];
    let l_lab = lch(mix(p, b, 0.5, MixSpace::Oklab))[0];
    eprintln!("azul-amarelo a meio: croma linear {c_lin:.3} · oklch {c_lch:.3}");
    eprintln!("preto-branco a meio: L linear {l_lin:.3} · oklab {l_lab:.3}");
    assert!(c_lin < 0.03, "o linear passa pelo cinza: {c_lin}");
    // Medido: 0,126 (a meio a matiz é verde, e o mapeamento de gama baixa a croma de 0,26).
    assert!(c_lch > 0.1, "o oklch mantem a cor: {c_lch}");
    assert!(l_lin > 0.7, "o linear clareia cedo: {l_lin}");
    assert!((l_lab - 0.5).abs() < 0.01, "o oklab fica a meio: {l_lab}");
}

/// ⭐ O GRADIENTE em OKLab/OKLCH dá a MESMA mistura da porta perceptual (uma lei, um sítio), e o modo
/// sobrevive à ida e volta pelo texto que o documento grava.
#[test]
fn the_ramp_modes_are_the_perceptual_mix_and_survive_the_text() {
    use crate::color_ramp::{ColorRamp, RampColorMode, RampInterp, RampStop};
    let (a, b) = (cor("laranja"), cor("roxo"));
    for (modo, esp) in [
        (RampColorMode::Oklab, MixSpace::Oklab),
        (RampColorMode::Oklch, MixSpace::OklchShorter),
    ] {
        let ramp = ColorRamp::new(
            vec![RampStop::new(0.0, a), RampStop::new(1.0, b)],
            modo,
            RampInterp::Linear,
        );
        for k in 0..=16_u8 {
            let t = f32::from(k) / 16.0;
            let (r, m) = (ramp.eval(t), mix(a, b, t, esp));
            for c in 0..4 {
                assert!((r[c] - m[c]).abs() < 1e-6, "{modo:?} t {t}: {r:?} × {m:?}");
            }
        }
        let texto = crate::serialize_gradient(&ramp);
        let volta = crate::parse_gradient(&texto).expect("o texto relê");
        assert_eq!(
            volta.color_mode, modo,
            "o modo sobrevive ao texto `{texto}`"
        );
    }
}

/// As matrizes por bits são as da [`crate::OklabColor`] (os literais dela), entrada a entrada.
#[test]
fn the_matrices_are_the_oklab_colour_ones() {
    let literais: [[f32; 3]; 3] = [
        [0.412_221_47, 0.536_332_5, 0.051_445_993],
        [0.211_903_5, 0.680_699_5, 0.107_396_96],
        [0.088_302_46, 0.281_718_84, 0.629_978_7],
    ];
    for (m, l) in M_LMS.iter().zip(literais) {
        for (b, v) in m.iter().zip(l) {
            assert_eq!(*b, v.to_bits());
        }
    }
    let lab: [[f32; 3]; 3] = [
        [0.210_454_26, 0.793_617_8, -0.004_072_047],
        [1.977_998_5, -2.428_592_2, 0.450_593_7],
        [0.025_904_037, 0.782_771_77, -0.808_675_77],
    ];
    for (m, l) in M_LAB.iter().zip(lab) {
        for (b, v) in m.iter().zip(l) {
            assert_eq!(*b, v.to_bits());
        }
    }
    let rgb: [[f32; 3]; 3] = [
        [4.076_741_7, -3.307_711_6, 0.230_969_94],
        [-1.268_438, 2.609_757_4, -0.341_319_38],
        [-0.004_196_086_3, -0.703_418_6, 1.707_614_7],
    ];
    for (m, l) in M_RGB.iter().zip(rgb) {
        for (b, v) in m.iter().zip(l) {
            assert_eq!(*b, v.to_bits());
        }
    }
    let de_lab: [[f32; 2]; 3] = [
        [0.396_337_78, 0.215_803_76],
        [-0.105_561_346, -0.063_854_17],
        [-0.089_484_18, -1.291_485_5],
    ];
    for (m, l) in M_LMS_DE_LAB.iter().zip(de_lab) {
        for (b, v) in m.iter().zip(l) {
            assert_eq!(*b, v.to_bits());
        }
    }
}

/// O WGSL leva as MESMAS matrizes, bit a bit (derivado das tabelas, nunca copiado à mão).
#[test]
fn the_wgsl_carries_the_same_matrix_bits() {
    let w = crate::wgsl_perceptual!();
    for b in M_LMS
        .iter()
        .chain(M_LAB.iter())
        .chain(M_RGB.iter())
        .flatten()
        .chain(M_LMS_DE_LAB.iter().flatten())
    {
        assert!(
            w.contains(&format!("0x{b:08x}u")),
            "o WGSL perdeu 0x{b:08x}"
        );
    }
    for f in ["pc_mix", "pc_gamut_map", "pe_cbrt", "pe_atan_turn"] {
        assert!(w.contains(f), "o WGSL perdeu {f}");
    }
}

#[test]
#[ignore = "sonda: imprime o meio de azul -> amarelo em cada espaco (doc 127, o texto do tutorial 11)"]
fn sonda_o_meio_do_azul_ao_amarelo() {
    let (a, b) = (cor("azul"), cor("amarelo"));
    for esp in [
        MixSpace::Linear,
        MixSpace::Oklab,
        MixSpace::OklchShorter,
        MixSpace::OklchLonger,
    ] {
        for t in [0.25_f32, 0.5, 0.75] {
            let m = mix(a, b, t, esp);
            let l = oklab_to_oklch(to_oklab([m[0], m[1], m[2]]));
            eprintln!(
                "{esp:?} t {t}: rgb {:.2} {:.2} {:.2} · L {:.2} C {:.3} h {:.0}°",
                m[0],
                m[1],
                m[2],
                l[0],
                l[1],
                l[2] * 360.0
            );
        }
    }
}
