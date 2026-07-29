//! **O QUE UMA SESSÃO DE VERDADE CUSTA** — a fixture que as anteriores não tinham.
//!
//! Toda medição do tick até aqui rodou ~10 passos **logo depois** do traço
//! (`measure_pass_cost`, `measure_how_the_tick_scales_with_the_wet_area`). A
//! sessão do artista roda **centenas**: a água escorre sob gravidade, espalha,
//! seca — e o conjunto ATIVO é função do tempo, não do traço.
//!
//! **O que ela achou (2026-07-29):** o custo NÃO cresce — ele CAI a zero. O filme
//! total vai de ~470 a 0 em ~40 passos (**1 segundo de sim**), e daí em diante o
//! passo custa 0,00 ms porque não há água. O pico está nos ~30 primeiros passos
//! (4-6 ms cada, ~90k células ativas), e é ele que o orçamento de tempo governa.
//!
//! ⚠️ **E uma armadilha de fixture que ela expôs:** a sim **não roda com o pincel
//! encostado** (`sim_should_run` = `!stroke_down`), e `drive_stroke` termina com a
//! cauda de release — então `sim_after: 0` deixa `sim.frame = 0`, o primeiro passo
//! cai num frame ÍMPAR e `rebuild_active_region` (que só roda em pares) nunca
//! rodou: `active` lê **zero** sobre uma poça cheia. Amostrar em múltiplos de 4 é
//! o que torna a tabela legível.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_long_session -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use util::drive_stroke;

const SIDE: usize = 4096;

struct Shot {
    step: usize,
    ms: f64,
    active: u64,
    span: u64,
    bbox: u64,
    film: f64,
}

fn counts(g: &Grid) -> (u64, u64, u64) {
    let active = g.active.iter().filter(|a| **a != 0).count() as u64;
    let mut span = 0u64;
    for y in g.by0..=g.by1 {
        let (lo, hi) = g.span_x(y);
        if hi >= lo {
            span += (hi - lo + 1) as u64;
        }
    }
    let bbox = ((g.bx1 - g.bx0 + 1).max(0) as u64) * ((g.by1 - g.by0 + 1).max(0) as u64);
    (active, span, bbox)
}

fn session(label: &str, diagonal: bool, steps: usize) {
    let mut e = Engine::new(SIDE, SIDE);
    e.sliders.water = 1.0;
    e.sliders.size = 0.6;
    let c = SIDE as f64 * 0.5;
    let half = 1200.0;
    let (dx, dy) = if diagonal {
        (
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        )
    } else {
        (1.0, 0.0)
    };
    drive_stroke(
        &mut e,
        c - dx * half,
        c - dy * half,
        c + dx * half,
        c + dy * half,
        60.0,
        0,
    );

    let mut shots: Vec<Shot> = Vec::new();
    for step in 1..=steps {
        let t0 = Instant::now();
        e.step_simulation();
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        // Amostra em múltiplos de 4 (o passo que roda `build_flow_field`), denso
        // no começo e esparso depois: o regime que interessa é o que DURA.
        let sample = step.is_multiple_of(4) && (step <= 40 || step.is_multiple_of(40));
        if sample {
            let g = e.active_grid();
            let (active, span, bbox) = counts(g);
            let film: f64 = g.film.iter().map(|v| f64::from(*v)).sum();
            shots.push(Shot {
                step,
                ms,
                active,
                span,
                bbox,
                film,
            });
        }
    }

    let tela = (SIDE * SIDE) as f64;
    println!(
        "\n  {label}  ({SIDE}x{SIDE}, {steps} passos = {:.1}s de sim)",
        steps as f64 / 40.0
    );
    println!(
        "    {:>6} {:>9} {:>11} {:>11} {:>9} {:>9} {:>12}",
        "passo", "ms", "ativas", "faixa", "bbox", "faixa/ativ", "film total"
    );
    for s in &shots {
        println!(
            "    {:>6} {:>9.2} {:>11} {:>11} {:>8.1}% {:>8.1}x {:>12.0}",
            s.step,
            s.ms,
            s.active,
            s.span,
            100.0 * s.bbox as f64 / tela,
            s.span as f64 / (s.active.max(1)) as f64,
            s.film,
        );
    }
}

#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_what_a_long_session_costs() {
    println!("\n=== O custo do passo AO LONGO de uma sessao ===");
    session("HORIZONTAL", false, 400);
    session("DIAGONAL", true, 400);
    println!();
}
