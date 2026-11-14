//! **B13.7 — a velocidade da difusão Jacobi (n.º 10, fora do produto).** A cena da `sonda_secagem_4k` (4K, 6 faixas de
//! cor, `Lei::Cobre`) com `Diffusion` no tecto (0,25), cronometrada no MESMO processo, rotas intercaladas a partir do
//! mesmo instantâneo: `gauss-seidel` (a rota de antes, `DIFUSAO_GS = 1`) · `jacobi série` · `jacobi paralelo` (o
//! produto) · `cópia` (o `restore_grid`, a banda). Mínimo de `RODADAS`.
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke \
//!   -p ph2d-wet-paint --example sonda_difusao_4k
//! target/smoke/examples/sonda_difusao_4k [rodadas]
//! ```

use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use ph2d_wet_paint::grid::{Grid, restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::solver::{DIFUSAO_GS, diffusion_pass, diffusion_pass_jacobi_rows};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 3840;
const H: usize = 2160;
const CORES: [[f64; 3]; 6] = [
    [40.0, 70.0, 200.0],
    [200.0, 40.0, 40.0],
    [230.0, 200.0, 30.0],
    [30.0, 150.0, 60.0],
    [120.0, 40.0, 160.0],
    [90.0, 60.0, 40.0],
];

fn faixa(e: &mut Engine, y: f64) {
    let (x0, x1) = (-64.0, W as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 16.0).min(x1);
        e.dispatch_pressure_dab(x, y, 6.0, 60.0);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn main() {
    let rodadas: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9);
    let mut e = Engine::new(W, H);
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.tuning.set(Knob::ExtDiffusion, 0.25);
    e.wet_canvas_now();
    let mut y = -32.0;
    while y < H as f64 + 32.0 {
        let k = ((y.max(0.0) as usize) * CORES.len() / H).min(CORES.len() - 1);
        e.color = CORES[k];
        faixa(&mut e, y);
        y += 12.0;
    }
    let p = e.sim.gather_params(&e.tuning);
    let g = e.active_grid_mut();
    ph2d_wet_paint::solver::rebuild_active_region(g);
    let vivas = (0..g.cells)
        .filter(|&i| g.active[i] != 0 && g.film[i] > 0.1)
        .count();
    println!("donas vivas {vivas}");
    let snap = snapshot_grid(g);
    let celulas = g.live_span_cells().max(1) as f64;
    println!(
        "difusão {W}x{H} · 6 cores · faixa {celulas} células · {} fios · load {}",
        rayon::current_num_threads(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let rotas = ["gauss-seidel", "jacobi série", "jacobi paralelo", "cópia"];
    let mut melhor = [f64::MAX; 4];
    let mut hash = [0u64; 4];
    for r in 0..rodadas {
        for k in 0..rotas.len() {
            let j = (k + r) % rotas.len();
            restore_grid(g, &snap);
            let t = Instant::now();
            match j {
                0 => {
                    DIFUSAO_GS.store(1, Relaxed);
                    diffusion_pass(g, &p);
                    DIFUSAO_GS.store(0, Relaxed);
                }
                1 => diffusion_pass_jacobi_rows(g, &p, Rows::Serial),
                2 => diffusion_pass(g, &p),
                _ => {
                    restore_grid(g, &snap);
                }
            }
            melhor[j] = melhor[j].min(t.elapsed().as_secs_f64() * 1e3);
            hash[j] = fnv(g);
        }
    }
    for ((nome, ms), h) in rotas.iter().zip(melhor).zip(hash) {
        println!(
            "  {nome:>15}: mín {ms:7.2} ms · {:5.2} ns/célula · hash {h:016x}",
            ms * 1e6 / celulas
        );
    }
}

fn fnv(g: &Grid) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for a in [&g.susp] {
        for v in a.iter() {
            h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
        }
    }
    for v in g.susp_rgb.iter().flatten() {
        h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
    }
    h
}
