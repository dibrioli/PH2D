//! **A régua do overlay do Gap Closure** (doc `06 §8` — os helpers ao vivo).
//!
//! O overlay recomputa `preview_closures()` por frame em modo Fill, então o custo dele
//! tem de ser MEDIDO antes de decidir se precisa de cache (§0 do CLAUDE.md: medir antes
//! de limitar — e antes de complicar). A colisão do Gap Closure é O(raios × paredes):
//! raios = 2 pontas por traço aberto + quinas apertadas; paredes = todos os segmentos.
//!
//! Rodar: `cargo test -p ph2d-flip-fill --release --test measure_closures -- --ignored --nocapture`
//!
//! Medido (2026-07-25, workstation, `--release`):
//!   típico   (60 traços × 24 pts,  1,4k pts):   **5,0 ms**/chamada
//!   pesado   (300 traços × 40 pts, 12k pts):    **339 ms**/chamada
//!   extremo  (800 traços × 60 pts, 48k pts):    **5,3 s**/chamada
//!
//! **Veredito: recompute por frame está REFUTADO** — e o síncrono por tique de scroll
//! também (339 ms por tique num quadro pesado seria o slider travando a UI). O custo é
//! O(raios × paredes) + O(raios²), e "raio" inclui as quinas apertadas — um desenho
//! hachurado multiplica os raios. O overlay dos helpers roda num **worker fora da thread
//! de UI** (`flip_gap_live.rs`, o MESMO padrão do ajuste ao vivo do Colorize), com no
//! máximo um em voo e o pedido mais recente coalescido. Baratear o kernel (o BVH que o
//! GP usa na colisão) é wave própria do engine, nomeada no handoff — não contrabando.

use ph2d_core::Vec2;
use ph2d_flip_fill::preview_closures;
use std::time::Instant;

/// Um "desenho" sintético: `n` traços ABERTOS de `pts` pontos, serpenteando numa grade
/// (pontas espalhadas ⇒ raios de verdade; nada de casos degenerados).
fn scene(n: usize, pts: usize) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    let mut out = Vec::with_capacity(n);
    for s in 0..n {
        let base_x = (s % 20) as f32 * 10.0;
        let base_y = (s / 20) as f32 * 10.0;
        let mut p = Vec::with_capacity(pts);
        for i in 0..pts {
            let t = i as f32 / (pts - 1) as f32;
            // Zigue-zague: segmentos curtos, algumas viradas apertadas (quinas).
            let x = base_x + t * 8.0;
            let y = base_y + if i % 2 == 0 { 0.0 } else { 1.5 } + t * 3.0;
            p.push(Vec2::new(x, y));
        }
        out.push((p, vec![0.13; pts], false));
    }
    out
}

fn measure(label: &str, strokes: &[(Vec<Vec2>, Vec<f32>, bool)], reach: f32) {
    // Warm-up + medição por média (a chamada é sub-ms no caso típico).
    let reps = 20u32;
    let mut total = 0u128;
    let mut n_closures = 0usize;
    for _ in 0..reps {
        let t0 = Instant::now();
        let cls = preview_closures(strokes, reach);
        total += t0.elapsed().as_micros();
        n_closures = cls.len();
    }
    let pts: usize = strokes.iter().map(|(p, _, _)| p.len()).sum();
    println!(
        "[measure_closures] {label}: {} traços / {pts} pts -> {n_closures} fechamentos, {:.2} ms/chamada",
        strokes.len(),
        total as f64 / f64::from(reps) / 1000.0
    );
}

#[test]
#[ignore = "régua de perf: rodar com --release --ignored --nocapture"]
fn measure_the_per_frame_cost_of_the_gap_helpers() {
    measure("típico ", &scene(60, 24), 2.0);
    measure("pesado ", &scene(300, 40), 2.0);
    measure("extremo", &scene(800, 60), 2.0);
}
