//! **G1** — disco emissor de raio `a` e radiância `L`, ponto a distância `r` no vazio:
//! fluência = `2·L·asin(a/r)` (forma fechada do ângulo subtendido).

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_ref::fluencia;

use crate::comum::{cena, disco, dist, f64x2};

/// O piso de uma varredura angular em `f64`: cada ângulo-evento em `[0, 2π)` vale até meio ulp de
/// `2π` (`4·ε` ≈ 8,9e-16 rad). Num disco minúsculo isso domina o erro RELATIVO (medido: 1,7e-13 a
/// `δ = 1e-3`), por isso a tolerância é ABSOLUTA em ângulo — 4 ulp de `2π` — mais o relativo do
/// `asin`/`hypot` da forma fechada. Os números medidos estão no handoff da W1.
const TOL_G1_RAD: f64 = 4.0 * 4.0 * f64::EPSILON;
const TOL_G1_REL: f64 = 1e-14;

fn dentro_da_tolerancia(obtido: f64, esperado: f64, l: f64) -> bool {
    (obtido - esperado).abs() <= TOL_G1_RAD * l + TOL_G1_REL * esperado.abs()
}

const L: [f32; 3] = [1.0, 0.37, 2.5];

#[test]
fn g1_a_fluencia_do_disco_e_2_l_asin_a_sobre_r() {
    let centro = [0.37_f32, -1.21];
    let mut pior_rad = 0.0_f64;
    let mut n = 0;
    for (i, &a) in [1e-3_f32, 0.05, 0.3, 1.0, 7.5].iter().enumerate() {
        for (j, &k) in [1.0001_f64, 1.01, 1.5, 3.0, 10.0, 1e3].iter().enumerate() {
            let phi = 0.7 + 1.3 * (i * 6 + j) as f64;
            let r = f64::from(a) * k;
            let c = f64x2(centro);
            let p = [c[0] + r * phi.cos(), c[1] + r * phi.sin()];
            let r = dist(p, c);
            let f = fluencia(&cena(vec![disco(centro, a, L, f32::INFINITY)], vec![]), p);
            for ch in 0..3 {
                let l = f64::from(L[ch]);
                let esperado = 2.0 * l * (f64::from(a) / r).asin();
                pior_rad = pior_rad.max((f[ch] - esperado).abs() / l);
                assert!(
                    dentro_da_tolerancia(f[ch], esperado, l),
                    "a={a} r={r} canal {ch}: gabarito {} × fechada {esperado} (erro {:e} rad)",
                    f[ch],
                    (f[ch] - esperado).abs() / l
                );
            }
            n += 1;
        }
    }
    eprintln!(
        "G1: {n} pontos, erro angular máximo medido = {pior_rad:e} rad = {:.2} ulp de 2π",
        pior_rad / (4.0 * f64::EPSILON)
    );
}

#[test]
fn g1_dentro_do_disco_a_fluencia_e_2_pi_l() {
    let f = fluencia(
        &cena(vec![disco([2.0, 3.0], 0.5, L, f32::INFINITY)], vec![]),
        [2.1, 3.2],
    );
    for ch in 0..3 {
        let esperado = std::f64::consts::TAU * f64::from(L[ch]);
        assert!(
            dentro_da_tolerancia(f[ch], esperado, f64::from(L[ch])),
            "{f:?}"
        );
    }
}

/// O `Ponto` é o limite `raio → 0` do disco com a MESMA intensidade (`L = I/(2a)`): `F = I/r`, e o
/// disco aproxima-se dele como `I/r·(1 + (a/r)²/6 + …)`.
#[test]
fn g1_o_ponto_e_o_limite_do_disco() {
    let i = [3.0_f32, 1.0, 0.5];
    let p = [1.7, -0.4];
    let c = [-0.6_f32, 0.9];
    let r = dist(p, f64x2(c));
    let ponto = CenaDeLuz {
        emissores: vec![Emissor::Ponto {
            centro: c,
            intensidade: i,
            alcance_m: f32::INFINITY,
        }],
        ..CenaDeLuz::default()
    };
    let f = fluencia(&ponto, p);
    for ch in 0..3 {
        let esperado = f64::from(i[ch]) / r;
        assert!(((f[ch] - esperado) / esperado).abs() <= TOL_G1_REL, "{f:?}");
    }
    for a in [0.1_f32, 0.01, 0.001] {
        let l = i.map(|x| x / (2.0 * a));
        let fd = fluencia(&cena(vec![disco(c, a, l, f32::INFINITY)], vec![]), p);
        let x = f64::from(a) / r;
        // A intensidade que o disco REALMENTE tem depois de `l` passar por `f32`.
        let i_efetiva = f64::from(l[0]) * 2.0 * f64::from(a);
        let previsto = i_efetiva / r * (1.0 + x * x / 6.0);
        // O resto da série (3x⁴/40) mais o piso angular da varredura sobre a largura 2δ ≈ 2x.
        assert!(
            ((fd[0] - previsto) / previsto).abs() <= x.powi(4) + TOL_G1_RAD / (2.0 * x),
            "a={a}: {} × {previsto}",
            fd[0]
        );
    }
}

#[test]
fn g1_o_ceu_aberto_da_2_pi_l() {
    let ceu = [0.25_f32, 0.5, 1.0];
    let f = fluencia(
        &CenaDeLuz {
            ceu,
            ..CenaDeLuz::default()
        },
        [0.3, 0.3],
    );
    for ch in 0..3 {
        let esperado = std::f64::consts::TAU * f64::from(ceu[ch]);
        assert!(
            dentro_da_tolerancia(f[ch], esperado, f64::from(ceu[ch])),
            "{f:?}"
        );
    }
}
