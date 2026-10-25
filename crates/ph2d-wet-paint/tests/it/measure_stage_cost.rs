//! **De que o passo do PRODUTO é feito** — cada estágio pela porta que o produto roda
//! ([`sim_step_stage`]), na poça do log do Enio (3 faixas diagonais a 4096²).
//!
//! O `measure_pass_cost` decompõe as rotas seriais de referência (`drying_pass`, `advect`), que
//! deixaram de ser o produto com o ADR-0147; esta sonda cronometra os estágios Jacobi/paralelos.
//! Dois quadros: `n = 12` (todas as cadências caem: região, secagem, campo de fluxo, pressão) e
//! `n = 13` (o quadro barato: suavização + advecção + bordas). O ciclo de 12 quadros é a taxa real.
//!
//! Rodar:
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke --test it measure_stage_cost -- --ignored --nocapture --test-threads=1
//! ```

use crate::util::drive_stroke;

use std::time::Instant;

use ph2d_wet_paint::grid::{restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::sim_step_stage;

const REPS: usize = 9;
const STAGES: [&str; 7] = [
    "0 rebuild_active_region",
    "1 drying",
    "2 flow_field | smooth",
    "3 advect (+sedimenta)",
    "4 boundaries",
    "5 project",
    "6 cadencia",
];

const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// VERBATIM o `scene_big` do `measure_parallel_rows` — a mesma cena, números comparáveis.
fn scene_big() -> Engine {
    let mut e = Engine::new(4096, 4096);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let c = 4096.0 * 0.5;
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

/// Mínimo e mediana por estágio do quadro `n`, estado (grid E sim) restaurado por amostra.
fn stages_of_frame(e: &mut Engine, n: u64) -> Vec<(f64, f64)> {
    let snap = snapshot_grid(e.active_grid());
    let sim0 = e.sim.clone();
    let mut t = vec![Vec::with_capacity(REPS); STAGES.len()];
    for _ in 0..REPS {
        e.sim = sim0.clone();
        e.sim.frame = n - 1;
        let idx = e.active_layer;
        restore_grid(&mut e.layers[idx].grid, &snap);
        for slot in t.iter_mut() {
            let t0 = Instant::now();
            let done = sim_step_stage(&mut e.sim, &mut e.layers[idx].grid, &e.tuning);
            slot.push(t0.elapsed().as_secs_f64() * 1e3);
            if done.is_some() {
                break;
            }
        }
    }
    t.into_iter()
        .map(|mut v| {
            if v.is_empty() {
                return (0.0, 0.0);
            }
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        })
        .collect()
}

/// A escala da cena da fábrica (`a_largura_e_a_prometida` a raio 55: tela 1000×410, janela ~117 mil
/// células) — abaixo dos pisos do [`ph2d_wet_paint::par`], toda a rota é serial.
fn scene_mid() -> Engine {
    let mut e = Engine::new(1000, 410);
    e.sliders.water = 1.0;
    e.sliders.size = std::env::var("PH2D_SONDA_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.3);
    drive_stroke(&mut e, 100.0, 205.0, 900.0, 205.0, 24.0, 2);
    e
}

#[test]
#[ignore = "sonda de medicao (perfil smoke)"]
fn measure_stage_cost_of_the_product_step() {
    report(scene_big());
}

#[test]
#[ignore = "sonda de medicao (perfil smoke)"]
fn measure_stage_cost_at_the_factory_scale() {
    report(scene_mid());
}

fn report(mut e: Engine) {
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  loadavg inicio: {}", la.trim());
    let g = e.active_grid();
    let (rows, span) = (g.by1 - g.by0 + 1, g.bx1 - g.bx0 + 1);
    println!("  janela: {rows} x {span} = {} celulas", rows * span);
    let heavy = stages_of_frame(&mut e, 12);
    let light = stages_of_frame(&mut e, 13);
    println!(
        "\n  {:<26} {:>10} {:>10}   {:>10} {:>10}",
        "estagio", "n=12 min", "med", "n=13 min", "med"
    );
    let (mut sh, mut sl) = (0.0, 0.0);
    for (i, name) in STAGES.iter().enumerate() {
        let (hm, hd) = heavy[i];
        let (lm, ld) = light[i];
        sh += hm;
        sl += lm;
        println!("  {name:<26} {hm:>10.3} {hd:>10.3}   {lm:>10.3} {ld:>10.3}");
    }
    println!("  {:<26} {sh:>10.3} {:>10}   {sl:>10.3}", "SOMA (min)", "");
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("  loadavg fim: {}", la.trim());
}

/// **O piso da secagem com a lei do Rebelle** (`Lei::Cobre`, a do Wet de fábrica): a célula custa ~10× a do port
/// (o Newton do `assenta`/`levanta`), então o joelho do `MIN_CELLS_DRYING`, medido com a célula barata, não é o
/// dela. Serial × paralela intercaladas, mínimo de `REPS`, estado restaurado; a lei do port ao lado.
#[test]
#[ignore = "sonda de medicao (perfil smoke)"]
fn measure_where_the_rebelle_drying_starts_paying_in_parallel() {
    use ph2d_wet_paint::assenta::Lei;
    use ph2d_wet_paint::drying::drying_pass_jacobi_rows;
    use ph2d_wet_paint::par::Rows;
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  loadavg inicio: {}", la.trim());
    println!(
        "  {:>8} {:>10} | {:>9} {:>9} {:>6} | {:>9} {:>9} {:>6}",
        "lado", "celulas", "Cobre ser", "par", "x", "Porto ser", "par", "x"
    );
    for side in [80usize, 120, 180, 260, 380, 560, 820] {
        let mut linha = Vec::new();
        let mut celulas = 0;
        for lei in [Lei::Cobre, Lei::Porto] {
            let mut e = Engine::new(side, side);
            e.sim.lei = lei;
            e.sim.glaze_filme = 0.35;
            e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
            e.sliders.water = 1.0;
            e.sliders.size = 1.0;
            let f = side as f64;
            drive_stroke(&mut e, f * 0.15, f * 0.35, f * 0.85, f * 0.4, 24.0, 2);
            drive_stroke(&mut e, f * 0.45, f * 0.1, f * 0.55, f * 0.9, 24.0, 40);
            let p = e.sim.gather_params(&e.tuning);
            let g = e.active_grid();
            celulas = ((g.by1 - g.by0 + 1) * (g.bx1 - g.bx0 + 1)) as usize;
            let snap = snapshot_grid(g);
            let mut best = [f64::MAX; 2];
            for _ in 0..REPS {
                for (k, mode) in [Rows::Serial, Rows::Parallel].into_iter().enumerate() {
                    let g = e.active_grid_mut();
                    restore_grid(g, &snap);
                    let t = Instant::now();
                    drying_pass_jacobi_rows(g, &p, 0.001, 0.0001, false, mode);
                    best[k] = best[k].min(t.elapsed().as_secs_f64() * 1e3);
                }
            }
            linha.push(best);
        }
        let [c, o] = [linha[0], linha[1]];
        println!(
            "  {side:>8} {celulas:>10} | {:>9.3} {:>9.3} {:>6.2} | {:>9.3} {:>9.3} {:>6.2}",
            c[0],
            c[1],
            c[0] / c[1],
            o[0],
            o[1],
            o[0] / o[1]
        );
    }
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("  loadavg fim: {}", la.trim());
}
