//! **ARITMÉTICA OU MEMÓRIA?** — a pergunta que decide a próxima wave.
//!
//! Um passo custa **12,7 ms sobre ~111k células ativas** a 4096² = **114 ns por
//! célula** para ~7 passes = **16 ns por célula-passe**. Aritmética `f64` com
//! desvios custa ~1-2 ns; 16 é a assinatura de **memória**.
//!
//! O experimento separa os dois: a MESMA água (mesmo nº de células ativas, mesmo
//! traço em células) numa tela PEQUENA — onde as linhas são curtas, o stride é
//! curto e tudo cabe no cache — contra a tela GRANDE, onde cada linha está 16 KB
//! da seguinte e o grid inteiro tem ~1 GB.
//!
//! ⚠️ O traço é dado em CÉLULAS, não em fração da tela: o que se quer manter
//! constante é o trabalho, e é a tela que varia.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-wet-paint --release --test measure_density -- --ignored --nocapture
//! ```

mod util;

use std::time::Instant;

use ph2d_wet_paint::painter::Engine;
use util::drive_stroke;

/// O traço, em células — IGUAL em toda tela.
const STROKE_PX: f64 = 1200.0;

fn measure(side: usize, diagonal: bool) -> (f64, u64, f64) {
    let mut e = Engine::new(side, side);
    e.sliders.water = 1.0;
    e.sliders.size = 0.6;
    let c = side as f64 * 0.5;
    let half = STROKE_PX * 0.5;
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
        4,
    );
    let mut ms = Vec::new();
    for _ in 0..9 {
        let t0 = Instant::now();
        e.step_simulation();
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    ms.sort_by(f64::total_cmp);
    let g = e.active_grid();
    let active = g.active.iter().filter(|a| **a != 0).count() as u64;
    let grid_mb = (side * side) as f64
        * (
            // film susp sett vel_x vel_y flow_x flow_y paper (f32)
            8.0 * 4.0
            // susp_rgb sett_rgb ([f32;3])
            + 2.0 * 12.0
            // wet active bloom (u8)
            + 3.0
        )
        / (1024.0 * 1024.0);
    (ms[ms.len() / 2], active, grid_mb)
}

#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_whether_the_step_is_arithmetic_or_memory() {
    println!("\n=== O passo e limitado por ARITMETICA ou por MEMORIA? ===");
    println!(
        "\n{:<10} {:>10} {:>12} {:>12} {:>14} {:>12}",
        "tela", "forma", "passo p50", "ativas", "ns/celula", "grid MB"
    );
    for diagonal in [false, true] {
        let forma = if diagonal { "diagonal" } else { "horizontal" };
        for side in [512usize, 1024, 2048, 4096] {
            let (ms, active, mb) = measure(side, diagonal);
            println!(
                "{:<10} {forma:>10} {ms:>12.3} {active:>12} {:>14.1} {mb:>12.0}",
                format!("{side}x{side}"),
                ms * 1e6 / active.max(1) as f64,
            );
        }
    }
    println!(
        "\n  Leitura: se ns/celula for PLANO na tela, o custo e ARITMETICA (e a saida\n  \
         e o dispositivo). Se CRESCER com a tela sobre a mesma agua, o custo e o\n  \
         LAYOUT — linhas a 16 KB de distancia, TLB e cache — e a saida e os dados.\n"
    );
}
