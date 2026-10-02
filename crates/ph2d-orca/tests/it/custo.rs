//! ⏳ **O CUSTO do desvio a 10 / 100 / 1000 agentes** (plano 30 §4, a medição que abre a W5).
//!
//! Sonda, não gate (relógio): `cargo test -p ph2d-orca --release --test it custo -- --ignored
//! --nocapture`, com o `loadavg` ao lado — acima de `load ~5` o número não vale nada (§5.0).
//!
//! A cena é a do PIOR caso honesto: todos na mesma sala, numa grelha apertada (`3 r` entre centros),
//! cada um a querer ir para o lado oposto — todos se vêem e quase todos estão apertados.

use std::time::Instant;

use ph2d_orca::{Agent, Crowd, Params, Regime};

fn multidao(n: usize) -> Vec<Agent> {
    let lado = (n as f64).sqrt().ceil() as usize;
    let r = 0.3;
    let passo = 3.0 * r;
    let meio = lado as f64 * passo * 0.5;
    (0..n)
        .map(|k| {
            let (i, j) = (k % lado, k / lado);
            let pos = [i as f64 * passo - meio, j as f64 * passo - meio];
            let l = (pos[0] * pos[0] + pos[1] * pos[1]).sqrt().max(1e-9);
            let pref = [-pos[0] / l * 2.0, -pos[1] / l * 2.0];
            Agent {
                pos,
                vel: pref,
                pref,
                radius: r,
                max_speed: 2.0,
                avoids: true,
                ignores: None,
            }
        })
        .collect()
}

#[test]
#[ignore = "sonda de relógio — corre em --release, à mão"]
fn custo_por_tique() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg: {}", load.trim());
    eprintln!("| agentes | ms por tique (mín de 5) | µs por agente | vizinhos médios | apertados |");
    for n in [10, 100, 1000] {
        let mut melhor = f64::INFINITY;
        let mut viz = 0.0;
        let mut apertados = 0;
        for _ in 0..5 {
            let mut c = Crowd::new(multidao(n), Params::PRODUCT);
            let t = Instant::now();
            let v = c.solve_all(|_| None, 1.0 / 60.0);
            let ms = t.elapsed().as_secs_f64() * 1e3;
            std::hint::black_box(v);
            melhor = melhor.min(ms);
            let c = Crowd::new(multidao(n), Params::PRODUCT);
            let mut nb = Vec::new();
            let mut soma = 0;
            apertados = 0;
            for i in 0..n {
                c.neighbors(i, &mut nb);
                soma += nb.len();
                apertados += usize::from(c.solve(i, None, 1.0 / 60.0).1 == Regime::Dense);
            }
            viz = soma as f64 / n as f64;
        }
        eprintln!(
            "| {n} | {melhor:.3} | {:.2} | {viz:.1} | {apertados} |",
            melhor * 1e3 / n as f64
        );
    }
}

#[test]
#[ignore = "sonda de relógio — corre em --release, à mão"]
fn o_preco_de_um_tecto_de_vizinhos() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg: {}", load.trim());
    eprintln!("| n | tecto | ms por tique | max |Δv| (m/s) | agentes com Δv > 1e-9 |");
    for n in [100, 1000] {
        let mut todos = Crowd::new(multidao(n), Params::PRODUCT);
        let base = todos.solve_all(|_| None, 1.0 / 60.0);
        for tecto in [6, 10, 16, 24, 40] {
            let p = Params { max_neighbors: Some(tecto), ..Params::PRODUCT };
            let mut melhor = f64::INFINITY;
            let mut v = Vec::new();
            for _ in 0..5 {
                let mut c = Crowd::new(multidao(n), p);
                let t = Instant::now();
                v = c.solve_all(|_| None, 1.0 / 60.0);
                melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
            }
            let (mut m, mut k) = (0.0_f64, 0);
            for (a, b) in v.iter().zip(&base) {
                let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
                m = m.max(d);
                k += usize::from(d > 1e-9);
            }
            eprintln!("| {n} | {tecto} | {melhor:.3} | {m:.4} | {k} |");
        }
    }
}

#[test]
#[ignore = "sonda de relógio — corre em --release, à mão"]
fn onde_vai_o_tempo() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg: {}", load.trim());
    for n in [100, 1000] {
        let c = Crowd::new(multidao(n), Params::PRODUCT);
        let mut nb = Vec::new();
        let mut melhor = f64::INFINITY;
        for _ in 0..5 {
            let t = Instant::now();
            for i in 0..n {
                c.neighbors(i, &mut nb);
                std::hint::black_box(&nb);
            }
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!("n={n}: só a vizinhança {melhor:.3} ms");
    }
}
