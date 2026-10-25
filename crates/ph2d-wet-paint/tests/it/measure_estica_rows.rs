//! **Onde a divisão por linhas do ESTICÃO se paga** — a sonda do piso [`MIN_CELLS_ESTICA`] (`trail/molhado.rs`).
//!
//! O traço de `molhado_rows` (azul sobre o vermelho molhado) por raio, pelas duas rotas forçadas, ALTERNADAS na
//! mesma ronda, o MÍNIMO de cada. O esticão é a diferença entre o transfer com `WetOpen` e sem ele, na MESMA rota (o
//! bico também segue a rota forçada; a diferença tira-o). `WetOpenPush = 0`: o levantar não corre.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke --test it \
//!     measure_estica_rows -- --ignored --nocapture --test-threads=1
//! ```

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::PISO_ALVOS_ESTICA;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

const W: usize = 600;
const H: usize = 400;
const AMOSTRAS: usize = 9;

/// ms do transfer (soma das janelas) de um traço de raio `r`, e as células da janela.
fn traco(abre: f64, r: f64) -> (f64, usize) {
    let mut e = Engine::new(W, H);
    {
        let g = e.active_grid_mut();
        for i in 0..g.susp.len() {
            g.film[i] = 0.5;
            g.susp[i] = 20.0 + (i % 97) as f32;
            g.susp_rgb[i] = [191.0, (i % 7) as f32, 18.0];
        }
    }
    e.tuning.set(Knob::WetOpen, abre);
    e.tuning.set(Knob::WetOpenPush, 0.0);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(150.0, 200.0, [23.0, 101.0, 179.0], TrailMode::Paint);
    t.on_segment(r * 0.2, r * 0.2);
    let g = e.active_grid_mut();
    let mut ms = 0.0;
    let passo = (r * 0.2).max(1.0);
    for k in 0..24 {
        let d = Dab {
            x: 150.0 + passo * f64::from(k),
            y: 200.0,
            r,
            hardness: 0.5,
            intensity: 4.0,
            water_amount: 0.5,
            dry_gate: 0.2,
            reach: 1.0,
            window_reach: 1.0,
            opacity: 1.0,
        };
        if t.accumulate_paint(g, &p, &tex, &d, false).window_full {
            let t0 = Instant::now();
            let _ = t.transfer_paint(g, &p);
            ms += t0.elapsed().as_secs_f64() * 1e3;
        }
    }
    let lado = (2.0 * r * 1.6 + 3.0) as usize;
    (ms, lado * lado)
}

const PISOS: [usize; 6] = [usize::MAX, 0, 1 << 11, 1 << 13, 1 << 15, 1 << 17];

#[test]
#[ignore = "sonda: cronometra; correr sozinha"]
fn measure_estica_rows() {
    let original = PISO_ALVOS_ESTICA.load(Relaxed);
    eprintln!(
        "loadavg {} · piso actual {original}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!("raio  ~células | ms do esticão por piso de destinos: {PISOS:?}");
    for r in [3.0, 5.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 61.0] {
        let mut m = [f64::MAX; PISOS.len()];
        let mut base = f64::MAX;
        let mut celulas = 0;
        for _ in 0..AMOSTRAS {
            for (j, &piso) in PISOS.iter().enumerate() {
                PISO_ALVOS_ESTICA.store(piso, Relaxed);
                let (ms, c) = traco(0.6, r);
                m[j] = m[j].min(ms);
                celulas = c;
            }
            base = base.min(traco(0.0, r).0);
        }
        let linha: Vec<String> = m
            .iter()
            .map(|x| format!("{:7.3}", (x - base).max(0.0)))
            .collect();
        eprintln!("{r:4} {celulas:9} | {}", linha.join(" "));
    }
    PISO_ALVOS_ESTICA.store(original, Relaxed);
}
