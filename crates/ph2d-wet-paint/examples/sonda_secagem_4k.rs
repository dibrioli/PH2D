//! **B13.12 cura 1 (12/11) — fora do produto.** Uma secagem a 3840×2160 com 6 faixas de cores diferentes
//! (`Lei::Cobre`, a montagem de `nucleos_wash.rs`), cronometrada no pool global em rotas intercaladas no MESMO
//! processo, cada uma a partir do mesmo instantâneo: `memo` (o produto, `MEMO_ASSENTA = 1`) · `sem memo` (0) ·
//! `chão` (3: o `assenta` devolve `sc` sem conta — o que a secagem custa sem a transferência; resultado errado) ·
//! `cópia` (o `restore_grid`, a banda). Imprime o mínimo de `RODADAS` e os acertos do memo (`MEMO_ASSENTA = 2`).
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke \
//!   -p ph2d-wet-paint --example sonda_secagem_4k
//! target/smoke/examples/sonda_secagem_4k [rodadas]
//! ```

use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use ph2d_wet_paint::assenta::{MEMO_ASSENTA, MEMO_BITS, MEMO_CONTA, MEMO_POTENCIA};
use ph2d_wet_paint::grid::{restore_grid, snapshot_grid};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
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
        .unwrap_or(5);
    // Passos da sim depois de pintar: a água corre e mistura as cores (o estado do Wet Wash no app).
    let passos: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut e = Engine::new(W, H);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.wet_canvas_now();
    let mut y = -32.0;
    while y < H as f64 + 32.0 {
        let k = ((y.max(0.0) as usize) * CORES.len() / H).min(CORES.len() - 1);
        e.color = CORES[k];
        faixa(&mut e, y);
        y += 12.0;
    }
    for _ in 0..passos {
        e.step_simulation();
    }
    let p = e.sim.gather_params(&e.tuning);
    let (ev, rw, by) = (
        e.sim.evap_scale * p.k(Knob::Evaporation),
        e.sim.rewet_base * p.k(Knob::Rewet),
        e.sim.ext_bypass,
    );
    let g = e.active_grid_mut();
    let snap = snapshot_grid(g);
    let celulas = g.live_span_cells().max(1) as f64;
    println!(
        "secagem {W}x{H} · 6 cores · faixa {celulas} células · ecra {:?} · {} fios · load {}",
        p.ecra,
        rayon::current_num_threads(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    // `PH2D_SONDA_SO=<MEMO_ASSENTA>`: só essa rota, em laço (para o `perf`).
    if let Some(m) = std::env::var("PH2D_SONDA_SO")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        MEMO_ASSENTA.store(m, Relaxed);
        if let Some(v) = std::env::var("PH2D_SONDA_POT")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            MEMO_POTENCIA.store(v, Relaxed);
        }
        for _ in 0..rodadas {
            restore_grid(g, &snap);
            ph2d_wet_paint::drying::drying_pass_jacobi_rows(g, &p, ev, rw, by, Rows::Parallel);
        }
        return;
    }
    MEMO_ASSENTA.store(2, Relaxed);
    restore_grid(g, &snap);
    ph2d_wet_paint::drying::drying_pass_jacobi_rows(g, &p, ev, rw, by, Rows::Parallel);
    let k = |i: usize| MEMO_CONTA[i].load(Relaxed);
    println!("memo: acertos {} · falhas {}", k(0), k(1));
    // (nome, MEMO_ASSENTA, MEMO_BITS, MEMO_POTENCIA); 255 = a cópia.
    let rotas: [(&str, u8, u8, u8); 6] = [
        ("memo", 1, 15, 1),
        ("memo pow", 1, 15, 0),
        ("sem memo", 0, 15, 1),
        ("sem m pow", 0, 15, 0),
        ("chão", 3, 15, 1),
        ("cópia", 255, 15, 1),
    ];
    let mut melhor = [f64::MAX; 6];
    let mut hash = [0u64; 6];
    for r in 0..rodadas {
        for k in 0..rotas.len() {
            let j = (k + r) % rotas.len();
            restore_grid(g, &snap);
            let t = Instant::now();
            if rotas[j].1 == 255 {
                restore_grid(g, &snap);
            } else {
                MEMO_ASSENTA.store(rotas[j].1, Relaxed);
                MEMO_BITS.store(rotas[j].2, Relaxed);
                MEMO_POTENCIA.store(rotas[j].3, Relaxed);
                ph2d_wet_paint::drying::drying_pass_jacobi_rows(g, &p, ev, rw, by, Rows::Parallel);
            }
            melhor[j] = melhor[j].min(t.elapsed().as_secs_f64() * 1e3);
            hash[j] = fnv(g);
        }
    }
    MEMO_ASSENTA.store(1, Relaxed);
    MEMO_BITS.store(15, Relaxed);
    MEMO_POTENCIA.store(1, Relaxed);
    for (((nome, ..), ms), h) in rotas.iter().zip(melhor).zip(hash) {
        println!(
            "  {nome:>9}: mín {ms:7.2} ms · {:5.2} ns/célula · hash {h:016x}",
            ms * 1e6 / celulas
        );
    }
}

fn fnv(g: &ph2d_wet_paint::grid::Grid) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for a in [&g.film, &g.susp, &g.sett] {
        for v in a.iter() {
            h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
        }
    }
    for a in [&g.susp_rgb, &g.sett_rgb] {
        for v in a.iter().flatten() {
            h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}
