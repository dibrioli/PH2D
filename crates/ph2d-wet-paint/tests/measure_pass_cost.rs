//! **De que o tick é feito** — a decomposição por PASSE, medida pela porta
//! pública de cada uma (não por um laço que eu reescrevi: um laço próprio
//! ficaria cego à porta, a lição do doc 28 §5.11).
//!
//! O contexto: o `tool-tick` é 82% do frame do produto a 4096², e a medição
//! anterior (`measure_whether_the_sim_pays_for_the_water_or_for_its_bounding_box`,
//! no crate do tool) mostrou que o custo segue a **CAIXA**, não a poça —
//! mesma água, mesmo comprimento de traço, só a forma muda:
//!
//! | forma      | tick p50 | bbox/tela |
//! |------------|----------|-----------|
//! | horizontal | 8,15 ms  |  2,1%     |
//! | diagonal   | 23,53 ms | 18,6%     |
//!
//! Esta sonda pergunta a pergunta seguinte, que decide o desenho da wave:
//! **QUAL passe paga por isso?** Todo passe itera as linhas da bbox e a
//! maioria tem early-out por-célula (`active[i] == 0` → `continue`); se o
//! custo estiver concentrado nos que fazem early-out, a cura é
//! byte-idêntica **por construção** (visitar exatamente as células ativas
//! não muda o que nenhuma delas responde). Se estiver nos que NÃO fazem
//! (`rebuild_active_region`, `drying_pass`), a cura precisa de outra prova.
//!
//! Metodologia: `snapshot_grid` / `restore_grid` (portas públicas do engine)
//! devolvem o MESMO estado inicial para cada amostra, então as amostras de um
//! passe são comparáveis entre si e entre passes. ⚠️ `restore_grid` zera
//! `flow_x/flow_y` de propósito (é a semântica do undo do engine), então o
//! número do `advect` é o de um campo de fluxo recém-zerado — o que interessa
//! aqui é a ORDEM DE GRANDEZA por passe e a razão diagonal÷horizontal, não o
//! milissegundo absoluto de um passe isolado.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_pass_cost -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::grid::{Grid, restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::solver;
use ph2d_wet_paint::tuning::Knob;
use util::drive_stroke;

const SIDE: usize = 4096;
/// Amostras por passe (mediana). O estado é restaurado antes de cada uma.
const REPS: usize = 9;
/// O passo por eixo que mantem o COMPRIMENTO do traco igual ao do horizontal.
const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// **A poça da SESSÃO DO ENIO** — a que mede um passo em ~33 ms, contra os 12,7 da cena diagonal.
///
/// O log do produto (2026-07-29): `agua: sim media 33.43ms x54`. A cena diagonal de 2400 px mede
/// 12,7 ms/passo, então ela **não contém o fenômeno** — a decomposição precisa da escala real, senão
/// otimizo o passe errado. Três traços largos e sobrepostos molham ~3× mais células.
fn scene_big() -> Engine {
    let mut e = Engine::new(SIDE, SIDE);
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

/// Uma cena: traço reto de `len` px na direção dada, com a água já assentada
/// alguns passos (é o estado em que o artista de fato deixa a tela).
fn scene(dx: f64, dy: f64) -> Engine {
    let mut e = Engine::new(SIDE, SIDE);
    e.sliders.water = 1.0;
    e.sliders.size = 0.6;
    let cx = SIDE as f64 * 0.5;
    let cy = SIDE as f64 * 0.5;
    let half = 1600.0;
    drive_stroke(
        &mut e,
        cx - dx * half,
        cy - dy * half,
        cx + dx * half,
        cy + dy * half,
        60.0,
        8,
    );
    e
}

struct Counts {
    bbox_cells: f64,
    active_cells: f64,
    film_cells: f64,
}

fn counts(g: &Grid) -> Counts {
    let mut active = 0u64;
    let mut film = 0u64;
    for (i, a) in g.active.iter().enumerate() {
        if *a != 0 {
            active += 1;
        }
        if g.film[i] > 0.0 || g.susp[i] > 0.0 {
            film += 1;
        }
    }
    let bw = (g.bx1 - g.bx0 + 1).max(0) as f64;
    let bh = (g.by1 - g.by0 + 1).max(0) as f64;
    Counts {
        bbox_cells: bw * bh,
        active_cells: active as f64,
        film_cells: film as f64,
    }
}

/// Cronometra UM passe a partir do estado congelado, `REPS` vezes.
fn time_pass(
    g: &mut Grid,
    snap: &ph2d_wet_paint::grid::GridSnapshot,
    mut f: impl FnMut(&mut Grid),
) -> f64 {
    let mut v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        restore_grid(g, snap);
        let t = Instant::now();
        f(g);
        v.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    median(&mut v)
}

fn decompose(label: &str, dx: f64, dy: f64) -> Vec<(String, f64)> {
    decompose_engine(label, scene(dx, dy))
}

fn decompose_engine(label: &str, mut e: Engine) -> Vec<(String, f64)> {
    let p: Params = e.sim.gather_params(&e.tuning);
    let grav = e.sim.gravity(&e.tuning);
    let evap = e.sim.evap_scale * p.k(Knob::Evaporation);
    let rewet = e.sim.rewet_base * p.k(Knob::Rewet);
    let bypass = e.sim.ext_bypass;
    let diffusion_on = p.k(Knob::ExtDiffusion) > 0.0;
    let g = e.active_grid_mut();
    let c = counts(g);
    let snap = snapshot_grid(g);

    let mut out: Vec<(String, f64)> = Vec::new();
    out.push((
        "rebuild_active_region".into(),
        time_pass(g, &snap, solver::rebuild_active_region),
    ));
    out.push((
        "drying_pass".into(),
        time_pass(g, &snap, |g| {
            ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, bypass);
        }),
    ));
    out.push((
        "build_flow_field".into(),
        time_pass(g, &snap, |g| {
            solver::build_flow_field(g, &p, grav[0], grav[1], bypass);
        }),
    ));
    out.push((
        "smooth_velocity".into(),
        time_pass(g, &snap, |g| solver::smooth_velocity(g, &p)),
    ));
    if diffusion_on {
        out.push((
            "diffusion_pass".into(),
            time_pass(g, &snap, |g| solver::diffusion_pass(g, &p)),
        ));
    }
    out.push((
        "advect".into(),
        time_pass(g, &snap, |g| {
            solver::advect(g, &p, grav[0], grav[1]);
        }),
    ));
    out.push((
        "apply_boundaries".into(),
        time_pass(g, &snap, |g| solver::apply_boundaries(g, false)),
    ));
    out.push((
        "project".into(),
        time_pass(g, &snap, |g| solver::project(g, &p)),
    ));
    restore_grid(g, &snap);

    let tela = (SIDE * SIDE) as f64;
    println!("\n  {label}  ({SIDE}x{SIDE})");
    println!(
        "    bbox {:.0} celulas ({:.1}% da tela) | ativas {:.0} ({:.1}% da bbox) | com agua/pigmento {:.0} ({:.1}% da bbox)",
        c.bbox_cells,
        100.0 * c.bbox_cells / tela,
        c.active_cells,
        100.0 * c.active_cells / c.bbox_cells.max(1.0),
        c.film_cells,
        100.0 * c.film_cells / c.bbox_cells.max(1.0),
    );
    let total: f64 = out.iter().map(|(_, t)| t).sum();
    for (name, t) in &out {
        println!("    {name:<24} {t:7.3} ms   ({:4.1}%)", 100.0 * t / total);
    }
    println!("    {:<24} {total:7.3} ms", "SOMA dos passes");
    out
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_what_a_sim_step_is_made_of() {
    println!("\n=== De que o tick da agua e feito, passe a passe ===");
    let h = decompose("HORIZONTAL (a caixa e fina)", 1.0, 0.0);
    let d = decompose("DIAGONAL (a caixa e a tela)", DIAG, DIAG);

    println!("\n  RAZAO diagonal / horizontal, por passe:");
    let mut worst = ("", 0.0f64, 0.0f64);
    for ((nh, th), (nd, td)) in h.iter().zip(d.iter()) {
        assert_eq!(nh, nd);
        let r = td / th.max(1e-9);
        println!("    {nh:<24} {th:7.3} -> {td:7.3} ms   {r:5.2}x");
        if td - th > worst.2 {
            worst = (nh, r, td - th);
        }
    }
    println!(
        "\n  Quem carrega o fator da CAIXA: {} ({:.2}x, +{:.3} ms)",
        worst.0, worst.1, worst.2
    );
    println!(
        "\n  Leitura: passe com early-out por-celula (`active[i] == 0`) pode ser\n  \
         restringido as celulas ATIVAS de forma byte-identica POR CONSTRUCAO.\n  \
         Passe sem early-out (rebuild/drying) precisa de outra prova."
    );
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_what_a_big_sim_step_is_made_of() {
    println!("\n=== A POCA DO PRODUTO (passo ~33 ms): de que ELA e feita ===");
    let out = decompose_engine("POCA GRANDE (a escala do log do Enio)", scene_big());
    let total: f64 = out.iter().map(|(_, t)| t).sum();
    let mut v: Vec<_> = out.into_iter().collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("\n  Ordenado pelo custo:");
    for (n, t) in &v {
        println!("    {n:<24} {t:7.3} ms   ({:4.1}%)", 100.0 * t / total);
    }
    println!("\n  Os tres maiores somam {:.1}% do passo.", 100.0 * v.iter().take(3).map(|x| x.1).sum::<f64>() / total);
}
