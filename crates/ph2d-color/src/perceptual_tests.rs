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
    par.split('-').any(|c| matches!(c, "branco" | "preto" | "cinza"))
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
            assert!((o[k] - lab[k]).abs() < 2e-5, "{nome} oklab[{k}]: {} × {}", o[k], lab[k]);
        }
        let h = oklab_to_oklch(o);
        assert!((h[1] - lch[1]).abs() < 2e-5, "{nome} C");
        if h[1] > 1e-3 {
            let d = (h[2] * 360.0 - lch[2]).rem_euclid(360.0);
            assert!(d.min(360.0 - d) < 0.01, "{nome} h: {} × {}", h[2] * 360.0, lch[2]);
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
                assert!((r[c] - o).abs() < 5e-5, "{par} {e} t {t} canal {c}: {} × {o}", r[c]);
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
    assert!(pior < 2e-3, "o mapeamento de gama diverge do oraculo: {pior:e}");
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
