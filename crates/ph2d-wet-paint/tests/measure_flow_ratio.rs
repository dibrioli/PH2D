//! **O que a grade de FLUXO entrega, medido pela porta do produto.**
//!
//! A F1 previu **1,3×** (e corrigiu o 1,7× que o plano 30 anunciava, que somava
//! os passes SEM a cadência — o erro que o doc 28 §5.40 já tinha documentado).
//! Esta sonda pergunta ao produto se a previsão se confirma, e mede o PASSO
//! inteiro, não um passe: só o passo é comparável com os 62 ms do log do Enio.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_flow_ratio -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::painter::Engine;
use util::drive_stroke;

const SIDE: usize = 4096;
const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;
/// Passos cronometrados por razão (mediana; o 1º é descartado).
const STEPS: usize = 9;
/// Passos por amostra: um CICLO de cadência completo (÷2, ÷3, ÷4, ÷6).
const CYCLE: usize = 12;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// A poça do PRODUTO — três faixas largas e sobrepostas, a escala do log do
/// Enio. (A mesma de `measure_pass_cost::scene_big`.)
fn scene(rf: usize) -> Engine {
    let mut e = Engine::with_flow_ratio(SIDE, SIDE, rf);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let c = SIDE as f64 * 0.5;
    for lane in 0..3 {
        let off = 420.0 * f64::from(lane) - 420.0;
        drive_stroke(
            &mut e,
            c - 1500.0 * DIAG + off,
            c - 1500.0 * DIAG,
            c + 1500.0 * DIAG + off,
            c + 1500.0 * DIAG,
            120.0,
            10,
        );
    }
    e
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_what_the_flow_ratio_buys() {
    println!("\n=== O que a grade de FLUXO entrega (poca do produto, {SIDE}x{SIDE}) ===");
    println!("\n  rf   ms/passo (media do ciclo)   razao   celulas de fluxo");
    let mut base = 0.0;
    for rf in [1usize, 2, 4, 8] {
        let mut e = scene(rf);
        let cells = e.active_grid().flow.cells;
        // ⚠️ **A unidade é o CICLO DE CADÊNCIA, não o passo, e a mediana MENTE
        // aqui.** O `build_flow_field` — o passe que esta wave existe para
        // baratear — roda 1 frame em 4, então a mediana de passos avulsos é
        // sempre um frame SEM ele: medido, ela reportava 1,04× onde o ciclo
        // reporta bem mais. `CYCLE = 12` cobre as quatro cadências do motor
        // (÷2 rebuild, ÷3 project, ÷4 flow, ÷6 drying) de uma vez.
        //
        // ⚠️ E RESTAURA antes de cada ciclo: um 1º corte cronometrava 25 passos
        // seguidos, e a poça SECA enquanto isso — as amostras tardias mediam
        // uma poça menor. Era a fixture medindo o próprio experimento.
        let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
        let frame = e.sim.frame;
        let mut v = Vec::with_capacity(STEPS);
        for _ in 0..STEPS {
            ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
            e.sim.frame = frame;
            let t = Instant::now();
            for _ in 0..CYCLE {
                e.step_simulation();
            }
            v.push(t.elapsed().as_secs_f64() * 1000.0 / CYCLE as f64);
        }
        let p50 = median(&mut v);
        if rf == 1 {
            base = p50;
        }
        println!(
            "  {rf:<4} {p50:20.3} ms   {:5.2}x   {cells:>10}",
            base / p50
        );
    }
    println!(
        "\n  ⚠️ O `Grid Size` que ja shipou compra 9,1x na razao 4 -- 25x mais que\n  \
         isto -- e o preco dele e o pigmento GROSSO, que e a foto do Enio. A\n  \
         entrega desta wave e a BORDA FINA com o fluxo barato; o resto e troco."
    );
}

/// Por PASSE, nas duas razões — onde o custo foi parar.
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_which_pass_pays_for_the_flow_grid() {
    use ph2d_wet_paint::grid::{Grid, restore_grid, snapshot_grid};
    use ph2d_wet_paint::sim::Params;
    use ph2d_wet_paint::solver;
    use ph2d_wet_paint::tuning::Knob;

    println!("\n=== Onde o custo foi parar, passe a passe ===");
    let mut rows: Vec<(usize, Vec<(String, f64)>)> = Vec::new();
    for rf in [1usize, 4] {
        let mut e = scene(rf);
        let p: Params = e.sim.gather_params(&e.tuning);
        let grav = e.sim.gravity(&e.tuning);
        let evap = e.sim.evap_scale * p.k(Knob::Evaporation);
        let rewet = e.sim.rewet_base * p.k(Knob::Rewet);
        let g = e.active_grid_mut();
        let snap = snapshot_grid(g);
        let time_pass = |g: &mut Grid, f: &mut dyn FnMut(&mut Grid)| -> f64 {
            let mut v = Vec::with_capacity(STEPS);
            for _ in 0..STEPS {
                restore_grid(g, &snap);
                let t = Instant::now();
                f(g);
                v.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            median(&mut v)
        };
        let mut out = Vec::new();
        out.push((
            "build_flow_field".to_string(),
            time_pass(g, &mut |g| {
                solver::build_flow_field(g, &p, grav[0], grav[1], false)
            }),
        ));
        out.push((
            "smooth_velocity".to_string(),
            time_pass(g, &mut |g| solver::smooth_velocity(g, &p)),
        ));
        out.push((
            "project".to_string(),
            time_pass(g, &mut |g| solver::project(g, &p)),
        ));
        out.push((
            "advect".to_string(),
            time_pass(g, &mut |g| {
                solver::advect(g, &p, grav[0], grav[1]);
            }),
        ));
        out.push((
            "drying_pass".to_string(),
            time_pass(g, &mut |g| {
                ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, false);
            }),
        ));
        out.push((
            "rebuild_active_region".to_string(),
            time_pass(g, &mut solver::rebuild_active_region),
        ));
        rows.push((rf, out));
    }
    println!("\n  passe                     rf=1        rf=4     razao");
    for k in 0..rows[0].1.len() {
        let (n, a) = (&rows[0].1[k].0, rows[0].1[k].1);
        let b = rows[1].1[k].1;
        println!("  {n:<22} {a:7.3} ms  {b:7.3} ms  {:6.2}x", a / b);
    }
}
