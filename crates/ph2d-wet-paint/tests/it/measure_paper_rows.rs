//! **Onde a divisão por linhas do PAPEL se paga** — a sonda que dá [`MIN_CELLS_PAPER_BAKE`] e
//! [`MIN_CELLS_PAPER_SEED`] (ADR-0175
//! §3-bis). Para cada lado, o bake do tile e a semente do hospedeiro pelas duas rotas forçadas,
//! ALTERNADAS na mesma ronda, e fica o mínimo de cada uma.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --release --test it \
//!     measure_paper_rows -- --ignored --nocapture
//! ```
//!
//! Imprime o `/proc/loadavg` ao lado: acima de `load ~5` leia só a RAZÃO.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::paper::{PaperKnobs, PaperPreset, bake_paper_rows, generate_paper_tile};
use ph2d_wet_paint::par::{MIN_CELLS_PAPER_BAKE, MIN_CELLS_PAPER_SEED, Rows};
use std::time::Instant;

const AMOSTRAS: usize = 9;

#[test]
#[ignore = "sonda de medicao (relogio); rode com --release --ignored --nocapture"]
fn measure_paper_rows() {
    println!(
        "load {} · pisos actuais: bake {MIN_CELLS_PAPER_BAKE} · seed {MIN_CELLS_PAPER_SEED} células",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let tile = generate_paper_tile(PaperPreset::Cold, 0, PaperKnobs::default());
    // Uma lei de hospedeiro com o custo da ordem da textura procedural do painter (~10 ns).
    let lei = |x: i64, y: i64| -> f64 {
        (0.5 + 0.5 * ((x as f64) * 0.37).sin() * ((y as f64) * 0.23).cos()).clamp(0.0, 1.0)
    };
    println!(
        "{:>5} {:>9} | {:>8} {:>8} {:>6} | {:>8} {:>8} {:>6}",
        "lado", "células", "bake sér", "bake par", "razão", "seed sér", "seed par", "razão"
    );
    for lado in [32usize, 64, 128, 256, 512, 1024, 2048] {
        let mut e = Engine::new(lado, lado);
        let mut m = [f64::INFINITY; 4];
        for _ in 0..AMOSTRAS {
            for (k, mode) in [Rows::Serial, Rows::Parallel].into_iter().enumerate() {
                let t0 = Instant::now();
                bake_paper_rows(e.active_grid_mut(), &tile, mode);
                m[k] = m[k].min(t0.elapsed().as_secs_f64() * 1e3);
                let t1 = Instant::now();
                e.seed_paper_with_rows(&lei, mode);
                m[2 + k] = m[2 + k].min(t1.elapsed().as_secs_f64() * 1e3);
            }
        }
        let cel = (lado + 2) * (lado + 2);
        println!(
            "{lado:>5} {cel:>9} | {:>8.3} {:>8.3} {:>5.2}x | {:>8.3} {:>8.3} {:>5.2}x",
            m[0],
            m[1],
            m[0] / m[1],
            m[2],
            m[3],
            m[2] / m[3]
        );
    }
}
