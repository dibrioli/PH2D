//! **Onde a divisão por linhas do DAB e do BICO se paga** — a sonda que dá os pisos
//! [`MIN_CELLS_DEPOSIT`] e [`MIN_CELLS_TIP`] (ADR-0175).
//!
//! Para cada raio, o MESMO dab é pousado pelas duas rotas forçadas, ALTERNADAS dentro da mesma
//! ronda (uma máquina que muda de carga a meio muda as duas colunas por igual), e fica o MÍNIMO de
//! cada uma. O estado é o de um traço real: dabs consecutivos sobre a mesma zona molhada, que é o
//! que o produto faz (dois dabs seguidos sobrepõem-se ~95 %) — por isso a janela quente é a
//! representativa aqui, ao contrário dos passes do solver (`measure_parallel_rows`).
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --release --test it \
//!     measure_deposit_rows -- --ignored --nocapture
//! ```
//!
//! Imprime o `/proc/loadavg` ao lado: acima de `load ~5` leia só a RAZÃO.

use ph2d_wet_paint::brush::BrushShape;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::{MIN_CELLS_DEPOSIT, MIN_CELLS_TIP, Rows};
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;
use std::time::Instant;

const AMOSTRAS: usize = 15;

fn dab(x: f64, y: f64, r: f64) -> Dab {
    Dab {
        x,
        y,
        r,
        hardness: 0.5,
        intensity: 40.0,
        water_amount: 0.5,
        dry_gate: 0.2,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// `(ms série, ms paralelo)` do depósito e do transfer para um dab de raio `r`.
fn mede(r: f64) -> ((f64, f64), (f64, f64)) {
    let lado = (4.0 * r) as usize + 200;
    let mut e = Engine::new(lado, lado);
    e.tuning.set(Knob::Pickup, 0.2);
    e.tuning.set(Knob::TipClean, 0.02);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let c = lado as f64 / 2.0;
    let sil = move |x: i32, y: i32| -> f64 {
        let (dx, dy) = (f64::from(x) - c, f64::from(y) - c);
        let d = (dx * dx + dy * dy).sqrt() / r;
        if d >= 1.0 { 0.0 } else { 1.0 - d * d }
    };
    let g = e.active_grid_mut();
    let mut t = Trail::default();
    t.start_stroke(c, c, [40.0, 90.0, 200.0], TrailMode::Paint);
    t.on_segment(1.0, 1.0);
    let mut melhor = [f64::INFINITY; 4];
    for _ in 0..AMOSTRAS {
        for (k, mode) in [Rows::Serial, Rows::Parallel].into_iter().enumerate() {
            let t0 = Instant::now();
            let _ =
                t.accumulate_paint_shaped_rows(g, &p, &tex, &dab(c, c, r), false, &sil, None, mode);
            melhor[k] = melhor[k].min(t0.elapsed().as_secs_f64() * 1e3);
            let t1 = Instant::now();
            let _ = t.transfer_paint_rows(g, &p, mode);
            melhor[2 + k] = melhor[2 + k].min(t1.elapsed().as_secs_f64() * 1e3);
        }
    }
    ((melhor[0], melhor[1]), (melhor[2], melhor[3]))
}

#[test]
#[ignore = "sonda de medicao (relogio); rode com --release --ignored --nocapture"]
fn measure_deposit_rows() {
    println!(
        "load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!(
        "pisos actuais: depósito {MIN_CELLS_DEPOSIT} · bico {MIN_CELLS_TIP} células\n\
         {:>5} {:>9} | {:>9} {:>9} {:>6} | {:>9} {:>9} {:>6}",
        "raio", "células", "dep sér", "dep par", "razão", "bico sér", "bico par", "razão"
    );
    for r in [8.0, 16.0, 24.0, 40.0, 64.0, 100.0, 160.0, 250.0, 400.0] {
        let ((ds, dp), (ts, tp)) = mede(r);
        let lado = 2 * (r as usize) + 1;
        println!(
            "{r:>5} {:>9} | {ds:>9.3} {dp:>9.3} {:>5.2}x | {ts:>9.3} {tp:>9.3} {:>5.2}x",
            lado * lado,
            ds / dp,
            ts / tp
        );
    }
}
