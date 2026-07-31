//! **DE QUE O ADVECT É FEITO** — a decomposição por SUB-PASSE.
//!
//! Filho de [`super`] (`#[path]`) porque os cinco sub-passes são privados: um
//! teste de integração teria de os tornar públicos, e uma superfície pública
//! aberta para medir é uma segunda porta esperando um chamador.
//!
//! ⚠️ **Ablação pela PORTA, nunca por instrumentação.** Cada linha abaixo roda
//! um sub-passe REAL sobre o MESMO estado restaurado — um laço próprio ficaria
//! cego à porta (doc 28 §5.11) e, pior, o LLVM pode provar inútil uma leitura
//! que ninguém consome e medir zero (§5.43).
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release advect_is_made_of -- --ignored --nocapture
//! ```

use std::time::Instant;

use super::*;
use crate::grid::{restore_grid, snapshot_grid};
use crate::painter::Engine;
use crate::tuning::Tuning;

const SIDE: usize = 4096;
const REPS: usize = 7;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// A poça grande — três faixas largas e sobrepostas.
fn scene() -> Engine {
    const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;
    let mut e = Engine::new(SIDE, SIDE);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let c = SIDE as f64 * 0.5;
    for lane in 0..3 {
        let off = 420.0 * f64::from(lane) - 420.0;
        let (x0, y0) = (c - 1500.0 * DIAG + off, c - 1500.0 * DIAG);
        let (x1, y1) = (c + 1500.0 * DIAG + off, c + 1500.0 * DIAG);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        e.pointer_down(x0, y0, None);
        let mut travelled = 0.0;
        let (mut x, mut y) = (x0, y0);
        while travelled < len {
            travelled = (travelled + 120.0).min(len);
            x = x0 + dx / len * travelled;
            y = y0 + dy / len * travelled;
            e.pointer_frame(x, y);
            if e.sim_should_run() {
                e.step_simulation();
            }
        }
        e.pointer_up();
        while e.release_frame(x, y) {}
        for _ in 0..10 {
            e.step_simulation();
        }
    }
    e
}

#[test]
#[ignore = "medicao (wall-clock) — rode com --release --ignored --nocapture"]
fn measure_what_an_advect_is_made_of_by_sub_pass() {
    let mut e = scene();
    let p = e.sim.gather_params(&e.tuning);
    let grav = e.sim.gravity(&Tuning::default());
    let g = e.active_grid_mut();
    // ⚠️ A máscara TEM de estar viva antes do snapshot — o worker roda o
    // rebuild como 1º estágio, e sem ele todo passe gateado em `active` faz
    // early-out em toda célula (a fixture envenenada do doc 28 §5.40).
    crate::solver::rebuild_active_region(g);
    g.scratch.ensure(g.cells);
    let live = g.active.iter().filter(|a| **a != 0).count();
    let win = ((g.by1 - g.by0 + 1).max(0) as usize) * ((g.bx1 - g.bx0 + 1).max(0) as usize);
    let snap = snapshot_grid(g);
    let mode = Rows::Parallel;

    let time = |g: &mut Grid, f: &mut dyn FnMut(&mut Grid)| -> f64 {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            restore_grid(g, &snap);
            g.scratch.ensure(g.cells);
            let t0 = Instant::now();
            f(g);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        median(&mut v)
    };

    let mut out: Vec<(&str, f64)> = Vec::new();
    out.push((
        "momentum_rows",
        time(g, &mut |g| momentum_rows(g, &p, grav[0], grav[1], mode)),
    ));
    out.push((
        "prepare_rows",
        time(g, &mut |g| {
            prepare_rows(g, mode);
        }),
    ));
    // ⚠️ Os três seguintes CONSOMEM o que os anteriores escreveram, então cada
    // amostra tem de refazer a cadeia — cronometrar só o alvo.
    let chained = |g: &mut Grid, upto: u8| -> f64 {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            restore_grid(g, &snap);
            g.scratch.ensure(g.cells);
            momentum_rows(g, &p, grav[0], grav[1], mode);
            prepare_rows(g, mode);
            if upto >= 1 {
                if upto == 1 {
                    let t0 = Instant::now();
                    outflow_rows(g, &p, mode);
                    v.push(t0.elapsed().as_secs_f64() * 1e3);
                    continue;
                }
                outflow_rows(g, &p, mode);
            }
            if upto == 2 {
                let t0 = Instant::now();
                transport_rows(g, &p, mode);
                v.push(t0.elapsed().as_secs_f64() * 1e3);
                continue;
            }
            transport_rows(g, &p, mode);
            let t0 = Instant::now();
            commit_rows(g, mode);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        median(&mut v)
    };
    out.push(("outflow_rows", chained(g, 1)));
    out.push(("transport_rows", chained(g, 2)));
    out.push(("commit_rows", chained(g, 3)));

    let whole = {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            restore_grid(g, &snap);
            // ⚠️ **O `ensure` fica FORA do relógio.** O primeiro corte desta
            // sonda o deixou dentro (ele é a 1ª linha do `advect_jacobi_rows`)
            // e o passe inteiro mediu **99,8 ms contra 31,7 da soma** — 141 MB
            // de rascunho alocados e tocados pela primeira vez, atribuídos ao
            // advect. Foi a linha de reconciliação abaixo que pegou.
            g.scratch.ensure(g.cells);
            let t0 = Instant::now();
            advect_jacobi_rows(g, &p, grav[0], grav[1], mode);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        median(&mut v)
    };
    restore_grid(g, &snap);

    println!("\n  DE QUE UM ADVECT E FEITO ({SIDE}x{SIDE}, mediana de {REPS}, PARALELO)\n");
    // ⚠️ A janela da BBOX não é o que os passes percorrem — eles andam a faixa
    // viva por linha (`span_x_of`). Sem este número não dá para saber se um
    // sub-passe é caro por célula ou por percorrer demais.
    let spanned: usize = (g.by0..=g.by1)
        .map(|y| {
            let (lo, hi) =
                crate::grid::span_x_of(&g.row_lo, &g.row_hi, g.spans_enabled, g.bx0, g.bx1, y);
            (hi - lo + 1).max(0) as usize
        })
        .sum();
    println!(
        "    bbox {win} celulas | FAIXA VIVA {spanned} ({:.1}% da bbox) | ativas {live} ({:.1}% da faixa)",
        100.0 * spanned as f64 / win.max(1) as f64,
        100.0 * live as f64 / spanned.max(1) as f64,
    );
    let sum: f64 = out.iter().map(|(_, t)| t).sum();
    for (name, ms) in &out {
        println!(
            "    {name:<18} {ms:7.3} ms  ({:4.1}%)  {:5.1} ns/celula-da-faixa",
            100.0 * ms / sum,
            ms * 1e6 / spanned.max(1) as f64
        );
    }
    println!(
        "    {:<18} {sum:7.3} ms   (o passe INTEIRO mede {whole:.3})",
        "SOMA"
    );
    println!(
        "\n    Se a SOMA nao bate com o passe inteiro, algum sub-passe esta sendo\n    \
         medido quente (ou frio) contra o que o produto paga."
    );
}
