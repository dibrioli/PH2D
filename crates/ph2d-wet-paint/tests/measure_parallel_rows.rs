//! **Quanto o thread-pool compra, e a partir de que tamanho** — a medição que
//! fixa o `PAR_MIN_CELLS` do [`ph2d_wet_paint::par`] (ADR-0145).
//!
//! Duas perguntas, e a segunda é a que decide a constante:
//!
//! 1. **por passe, na poça GRANDE:** quanto a rota paralela corta;
//! 2. **o JOELHO:** a partir de que janela ela deixa de ser prejuízo. Abaixo de
//!    algum tamanho a caminhada paralela manda milhares de tarefas de nada ao
//!    pool e fica MAIS LENTA que o laço direto — e um limiar escolhido em vez de
//!    medido é o palpite que o §0 dos inegociáveis proíbe.
//!
//! ⚠️ Os dois lados são cronometrados pela MESMA porta (`*_rows`), sobre estados
//! construídos identicamente — não por um laço que esta sonda reescreveu, que
//! ficaria cego à porta (a lição do doc 28 §5.11).
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_parallel_rows -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::grid::{restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::solver;
use util::drive_stroke;

/// Amostras por célula da tabela (mediana). O estado é RESTAURADO antes de cada
/// uma — a mesma metodologia do `measure_pass_cost`.
///
/// ⚠️ **Sem o restore este probe MENTIA para baixo:** repetir o
/// `rebuild_active_region` sobre o mesmo grid APERTA a bbox e a faixa viva, então
/// as amostras 2..15 mediam uma janela menor que a que o produto vê na 1ª
/// chamada (`project` serial saía 2,35 ms em vez de 11,98). Um passe que muda a
/// janela que ele próprio varre não é repetível sem restore.
const REPS: usize = 9;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Uma poça de `side`² com traços cruzados — a forma que dá faixa viva larga.
fn puddle(side: usize) -> Engine {
    let mut e = Engine::new(side, side);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    let f = side as f64;
    drive_stroke(&mut e, f * 0.15, f * 0.35, f * 0.85, f * 0.4, 24.0, 2);
    drive_stroke(&mut e, f * 0.45, f * 0.1, f * 0.55, f * 0.9, 24.0, 2);
    e
}

const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// **A poça do LOG DO ENIO** — VERBATIM o `scene_big` do `measure_pass_cost`:
/// três faixas largas e sobrepostas na DIAGONAL a 4096², que é a escala em que o
/// produto media ~33 ms/passo. É nela que o número vale, e ela é a mesma nas duas
/// sondas de propósito (duas cenas "grandes" diferentes dariam dois números que
/// ninguém consegue comparar).
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

fn window(e: &Engine) -> (usize, usize) {
    let g = e.active_grid();
    (
        (g.by1 - g.by0 + 1).max(0) as usize,
        (g.bx1 - g.bx0 + 1).max(0) as usize,
    )
}

fn time_pass(e: &mut Engine, p: &Params, mode: Rows, which: usize) -> f64 {
    let snap = snapshot_grid(e.active_grid());
    let mut s = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let g = e.active_grid_mut();
        restore_grid(g, &snap);
        let t = Instant::now();
        match which {
            0 => solver::project_rows(g, p, mode),
            1 => solver::smooth_velocity_rows(g, p, mode),
            _ => solver::rebuild_active_region_rows(g, mode),
        }
        s.push(t.elapsed().as_secs_f64() * 1e3);
    }
    median(&mut s)
}

const NAMES: [&str; 3] = ["project", "smooth_velocity", "rebuild_active_region"];

/// ⚠️ **O A/B tem de ser no MESMO processo, sobre a MESMA poça.** Comparar duas
/// corridas do `measure_pass_cost` (uma antes, uma depois do commit) mediu o
/// `advect` — que esta wave **não toca** — oscilando 12,1 -> 7,8 ms, ou seja 36%
/// de deriva de máquina. Uma soma cross-run atribuiria isso ao ganho.
#[test]
#[ignore = "sonda de medicao (release)"]
fn measure_what_the_thread_pool_buys_on_the_product_scene() {
    println!("\n  POR PASSE, a poca do LOG DO ENIO (3 faixas @ 4096x4096, mediana de {REPS})\n");
    let (rows, span) = window(&scene_big());
    let mut total = (0.0, 0.0);
    for (which, nome) in NAMES.into_iter().enumerate() {
        let mut e = scene_big();
        let p = e.sim.gather_params(&e.tuning);
        let ser = time_pass(&mut e, &p, Rows::Serial, which);
        let par = time_pass(&mut e, &p, Rows::Parallel, which);
        total.0 += ser;
        total.1 += par;
        println!(
            "    {nome:<24} serial {ser:7.3} ms   paralelo {par:7.3} ms   {:5.2}x",
            ser / par.max(1e-9)
        );
    }
    println!(
        "    {:<24} serial {:7.3} ms   paralelo {:7.3} ms   {:5.2}x  <- o que o passo deixa de pagar",
        "OS TRES",
        total.0,
        total.1,
        total.0 / total.1.max(1e-9)
    );
    println!(
        "\n    janela: {rows} linhas x {span} = {} celulas",
        rows * span
    );
}

#[test]
#[ignore = "sonda de medicao (release)"]
fn measure_what_the_thread_pool_buys_per_pass() {
    println!("\n  POR PASSE, poca de 4096x4096 (mediana de {REPS})\n");
    let mut total = (0.0, 0.0);
    for (which, nome) in NAMES.into_iter().enumerate() {
        let mut e = puddle(4096);
        let p = e.sim.gather_params(&e.tuning);
        let ser = time_pass(&mut e, &p, Rows::Serial, which);
        let par = time_pass(&mut e, &p, Rows::Parallel, which);
        total.0 += ser;
        total.1 += par;
        println!(
            "    {nome:<24} serial {ser:7.3} ms   paralelo {par:7.3} ms   {:5.2}x",
            ser / par.max(1e-9)
        );
    }
    println!(
        "    {:<24} serial {:7.3} ms   paralelo {:7.3} ms   {:5.2}x  <- os tres somados",
        "TOTAL",
        total.0,
        total.1,
        total.0 / total.1.max(1e-9)
    );
    let (rows, span) = window(&puddle(4096));
    println!(
        "\n    janela da poca: {rows} linhas x {span} = {} celulas",
        rows * span
    );
}

/// **O NÚMERO QUE VIRA Hz** — um passo inteiro pela porta do produto.
///
/// A taxa visual da água É a taxa de passos (o composite roda quando um passo
/// completa), então `1000 / ms_por_passo` é a resposta ao *"a água está lenta"*.
///
/// ⚠️ O "antes" NÃO sai de aritmética sobre os passes: rode esta sonda, ponha os
/// três pisos do `par.rs` em `usize::MAX` (= toda rota serial), rode de novo, e
/// devolva. É a MESMA fixture e o MESMO binário, com uma linha de diferença.
#[test]
#[ignore = "sonda de medicao (release)"]
fn measure_what_a_whole_step_costs_through_the_product_door() {
    const STEPS: usize = 13;
    let mut e = scene_big();
    let (rows, span) = window(&e);
    // ⚠️ O estado é RESTAURADO antes de cada passo. Sem isso a poça seca e a
    // janela encolhe ao longo das amostras: a 1ª versão desta sonda mediu 5,6
    // ms/passo (177 Hz) numa cena em que o produto paga ~52 ms — ela não continha
    // o fenômeno. O produto paga o passo CARO repetidamente porque o traço está
    // vivo e alimenta água; restaurar reproduz isso.
    let snap = snapshot_grid(e.active_grid());
    let mut s = Vec::with_capacity(STEPS);
    for _ in 0..STEPS {
        restore_grid(e.active_grid_mut(), &snap);
        let t = Instant::now();
        e.step_simulation();
        s.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let first = s[0];
    let mut rest = s[1..].to_vec();
    let med = median(&mut rest);
    let worst = rest.iter().copied().fold(0.0f64, f64::max);
    println!("\n  UM PASSO INTEIRO, poca do log do Enio (3 faixas @ 4096x4096)\n");
    println!(
        "    janela inicial: {rows} x {span} = {} celulas",
        rows * span
    );
    println!("    1o passo  {first:7.3} ms   (first-touch, descartado)");
    println!(
        "    mediana   {med:7.3} ms   -> {:5.1} Hz",
        1000.0 / med.max(1e-9)
    );
    println!(
        "    pior      {worst:7.3} ms   -> {:5.1} Hz",
        1000.0 / worst.max(1e-9)
    );
}

#[test]
#[ignore = "sonda de medicao (release)"]
fn measure_where_the_thread_pool_starts_paying() {
    println!("\n  O JOELHO: razao serial/paralelo por tamanho de janela\n");
    println!(
        "    {:>6}  {:>12}  {:>28}",
        "tela", "celulas", "razao ser/par por passe"
    );
    for side in [256usize, 384, 512, 768, 1024, 1536, 2048, 3072, 4096] {
        let (rows, span) = window(&puddle(side));
        let cells = rows * span;
        let mut r = [0.0f64; 3];
        for (which, slot) in r.iter_mut().enumerate() {
            let mut e = puddle(side);
            let p = e.sim.gather_params(&e.tuning);
            let ser = time_pass(&mut e, &p, Rows::Serial, which);
            let par = time_pass(&mut e, &p, Rows::Parallel, which);
            *slot = ser / par.max(1e-9);
        }
        // A rota que o PRODUTO escolheria, com o piso de cada passe.
        let pj = Rows::pick(rows, span, ph2d_wet_paint::par::MIN_CELLS_JACOBI);
        let pg = Rows::pick(rows, span, ph2d_wet_paint::par::MIN_CELLS_GATHER);
        let pr = Rows::pick(rows, span, ph2d_wet_paint::par::MIN_CELLS_REBUILD);
        let tag = |r: Rows| if r == Rows::Parallel { "par" } else { "ser" };
        println!(
            "    {side:>6}  {cells:>12}  proj {:5.2}x [{}]  smooth {:5.2}x [{}]  rebuild {:5.2}x [{}]",
            r[0],
            tag(pj),
            r[1],
            tag(pg),
            r[2],
            tag(pr)
        );
    }
    println!(
        "\n    Leitura: razao < 1,00 = o pool custa mais que o trabalho, e o piso\n    \
         daquele passe tem de ficar ACIMA da ultima linha em que isso acontece.\n    \
         O `[ser]`/`[par]` e a rota que o PRODUTO toma ali -- todo `[par]` tem\n    \
         de estar numa linha com razao > 1,00."
    );
}
