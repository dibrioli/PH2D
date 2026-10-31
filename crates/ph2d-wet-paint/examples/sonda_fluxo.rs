//! **Sonda dos cortes do fluxo (31/10) — fora do produto.** O wash 4096×2560 do `nucleos_wash`, e depois
//! blocos de `BLOCO` passos completos (`step_simulation`, um ciclo de secagem por bloco) com cada variante
//! ligada por override EM EXECUÇÃO, intercaladas em ordem rodada; resumo = o MÍNIMO de ms/passo por bloco.
//! Variantes cumulativas: `flow::COMMIT_ROTA` 1 (cópia, a rotina antiga) × 2 (troca de planos). Doc:
//! `docs/Painter/49_realshader/FLUXO_cortes.md`.
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke \
//!   -p ph2d-wet-paint --example sonda_fluxo
//! target/smoke/examples/sonda_fluxo [rondas]
//! ```

use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use ph2d_wet_paint::flow::COMMIT_ROTA;
use ph2d_wet_paint::painter::Engine;

const W: usize = 4096;
const H: usize = 2560;
const FAIXA: f64 = 12.0;
const RAIO: f64 = 60.0;
const PRESSAO: f64 = 6.0;
const BLOCO: usize = 6;

fn faixa(e: &mut Engine, y: f64) {
    let (x0, x1) = (-64.0, W as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + 16.0).min(x1);
        e.dispatch_pressure_dab(x, y, PRESSAO, RAIO);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

struct Variante {
    nome: &'static str,
    liga: fn(),
}

fn main() {
    let rondas = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);
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
        faixa(&mut e, y);
        y += FAIXA;
    }
    let vs = [
        Variante {
            nome: "commit cópia (antigo)",
            liga: || {
                COMMIT_ROTA.store(1, Relaxed);
            },
        },
        Variante {
            nome: "commit troca (3d5e69a39c)",
            liga: || {
                COMMIT_ROTA.store(2, Relaxed);
            },
        },
    ];
    let mut min = vec![f64::INFINITY; vs.len()];
    for r in 0..rondas {
        let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
        let mut linha = format!("ronda {r} load {}:", load.split(' ').next().unwrap_or("?"));
        for k in 0..vs.len() {
            let j = (k + r) % vs.len();
            (vs[j].liga)();
            let t = Instant::now();
            for _ in 0..BLOCO {
                e.step_simulation();
            }
            let ms = 1e3 * t.elapsed().as_secs_f64() / BLOCO as f64;
            min[j] = min[j].min(ms);
            linha += &format!(" {}={ms:.1}", vs[j].nome);
        }
        println!("{linha}");
    }
    for (v, m) in vs.iter().zip(&min) {
        println!("MÍNIMO {:<28} {m:.1} ms/passo", v.nome);
    }
    for k in 1..min.len() {
        println!(
            "Δ {} − {} = {:.1} ms",
            vs[k - 1].nome,
            vs[k].nome,
            min[k - 1] - min[k]
        );
    }
}
