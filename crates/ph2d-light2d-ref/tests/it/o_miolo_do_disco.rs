//! ⭐⭐ W5h — **o miolo de um disco é o de POISSON** (a régua sem grade do miolo por difusão): dentro
//! de um disco de raio `R` a solução de Laplace com a luz `g` na borda é o integral do núcleo de
//! Poisson, `u(r, φ) = (1/2π) ∫ g(ψ) (R² − r²) / (R² − 2Rr cos(φ − ψ) + r²) dψ`; e com a penetração `ℓ`
//! (`Δu − u/ℓ² = 0`) e `g` constante, `u = g · I₀(r/ℓ) / I₀(R/ℓ)`.

use std::f64::consts::{PI, TAU};

use ph2d_light2d_ecs::{CenaDeLuz, Oclusor};
use ph2d_light2d_ref::{Gabarito, MIOLO_PADRAO, OpcoesDoMiolo, fluencia};

use crate::comum::disco;

const C: [f64; 2] = [1.5, 0.2];
const R: f64 = 0.5;
const LADOS: usize = 256;
/// Amostras da borda no integral de Poisson.
const AMOSTRAS: usize = 4096;
const RAIOS: [f64; 5] = [0.0, 0.35, 0.65, 0.9, 0.97];
const ANGULOS: usize = 16;

/// O erro máximo sobre o máximo da luz na borda. MEDIDO (W5h, `NOS_NO_MENOR` do miolo): Poisson
/// 5,2e-3 com 160 nós (no anel 0,97, 2,4 nós da borda) · 7,6e-4 com 320; Bessel ≤ 4,4e-4 com 160.
const TETO: f64 = 2e-3;

fn redondo() -> Oclusor {
    Oclusor {
        vertices: (0..LADOS)
            .map(|k| {
                let a = TAU * k as f64 / LADOS as f64;
                [(C[0] + R * a.cos()) as f32, (C[1] + R * a.sin()) as f32]
            })
            .collect(),
        opacidade: 1.0,
        albedo: [0.0; 3],
    }
}

/// A distância do centro à borda do polígono na direção `a`.
fn raio_da_borda(a: f64) -> f64 {
    let lado = TAU / LADOS as f64;
    R * (PI / LADOS as f64).cos() / (a.rem_euclid(lado) - 0.5 * lado).cos()
}

fn ponto(r: f64, a: f64) -> [f64; 2] {
    [C[0] + r * a.cos(), C[1] + r * a.sin()]
}

fn pontos() -> Vec<(f64, f64)> {
    RAIOS
        .iter()
        .flat_map(|&f| (0..ANGULOS).map(move |k| (f * R, TAU * (k as f64 + 0.3) / ANGULOS as f64)))
        .collect()
}

/// `max |u − exato| / max |g|` no canal verde.
fn erro(u: impl Fn([f64; 2]) -> f64, exato: impl Fn(f64, f64) -> f64, g_max: f64) -> f64 {
    let mut pior = (0.0, 0.0, 0.0);
    for (r, a) in pontos() {
        let e = (u(ponto(r, a)) - exato(r, a)).abs() / g_max;
        if e > pior.0 {
            pior = (e, r / R, a);
        }
    }
    eprintln!(
        "  o pior: {:.2e} em r = {:.2} R, φ = {:.2}",
        pior.0, pior.1, pior.2
    );
    pior.0
}

#[test]
fn o_miolo_do_disco_e_o_de_poisson() {
    let cena = CenaDeLuz {
        emissores: vec![
            disco([0.0, 0.0], 0.2, [1.0; 3], f32::INFINITY),
            disco([1.4, 1.5], 0.1, [2.0; 3], f32::INFINITY),
        ],
        oclusores: vec![redondo()],
        ceu: [0.0; 3],
        ..Default::default()
    };
    let g: Vec<(f64, f64)> = (0..AMOSTRAS)
        .map(|k| {
            let a = TAU * k as f64 / AMOSTRAS as f64;
            // 1 µm fora: os vértices são `f32` (a borda real anda ~1e-7 da analítica).
            (a, fluencia(&cena, ponto(raio_da_borda(a) + 1e-6, a))[1])
        })
        .collect();
    let g_max = g.iter().map(|x| x.1).fold(0.0, f64::max);
    let poisson = |r: f64, a: f64| {
        g.iter()
            .map(|&(b, gb)| gb * (R * R - r * r) / (R * R - 2.0 * R * r * (a - b).cos() + r * r))
            .sum::<f64>()
            / AMOSTRAS as f64
    };
    let gab = Gabarito::novo(&cena);
    let e = erro(|p| gab.fluencia(p)[1], poisson, g_max);
    // Controlo: a lei do exterior mais próximo (a suavidade 0) não é a de Poisson.
    let perto = Gabarito::com_miolo(
        &cena,
        OpcoesDoMiolo {
            suavidade: 0.0,
            ..MIOLO_PADRAO
        },
    );
    let e0 = erro(|p| perto.fluencia(p)[1], poisson, g_max);
    eprintln!("miolo × Poisson: erro {e:.2e} (controlo, o exterior mais próximo: {e0:.2e})");
    assert!(e0 > 20.0 * TETO, "o controlo não distingue: {e0:.2e}");
    assert!(
        e <= TETO,
        "o miolo do disco não é o de Poisson: {e:.2e} > {TETO:.0e}"
    );
}

/// `I₀(x)` pela série.
fn i0(x: f64) -> f64 {
    let mut termo = 1.0;
    let mut soma = 1.0;
    for k in 1..200 {
        termo *= (0.5 * x / k as f64).powi(2);
        soma += termo;
        if termo < 1e-17 * soma {
            break;
        }
    }
    soma
}

#[test]
fn a_penetracao_e_a_de_bessel() {
    // Só o céu: logo fora do disco a luz é a de meia volta, π.
    let cena = CenaDeLuz {
        oclusores: vec![redondo()],
        ceu: [1.0; 3],
        ..Default::default()
    };
    for l in [f64::INFINITY, 0.25, 0.1] {
        let gab = Gabarito::com_miolo(
            &cena,
            OpcoesDoMiolo {
                penetracao_m: l,
                ..MIOLO_PADRAO
            },
        );
        let bessel = |r: f64, _| {
            if l.is_finite() {
                PI * i0(r / l) / i0(R / l)
            } else {
                PI
            }
        };
        let e = erro(|p| gab.fluencia(p)[1], bessel, PI);
        eprintln!("penetração {l} m × Bessel: erro {e:.2e}");
        assert!(
            e <= TETO,
            "a penetração {l} m não é a de Bessel: {e:.2e} > {TETO:.0e}"
        );
    }
}
