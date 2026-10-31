//! **Medida B (30/10) — fora do produto.** Um wash 4096×2560 a 100 % de cobertura, pintado pela
//! porta do ponteiro (`pointer_down/frame/up`, como o Wet Paint faz), com a `Engine` montada como
//! `ensure_wet_session` a monta (razão 1, fluxo 1, velatura, `Lei::Cobre`). Depois `SEGUNDOS` de
//! relógio de parede só de água: `step_simulation` em laço livre (o produto pede 40 Hz, `SIM_HZ`;
//! a secagem é adaptativa, 1 em `dry_every` = 6 ou 3). Sem render de óleo.
//!
//! Imprime `READY` antes da água (a amostragem começa aí), a cobertura, passos/s e ms/passo.
//! Doc: `docs/Painter/49_realshader/B_nucleos.md`.
//! ```text
//! bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke \
//!   -p ph2d-wet-paint --example nucleos_wash
//! target/smoke/examples/nucleos_wash [segundos]
//! ```

use std::time::Instant;

use ph2d_wet_paint::painter::Engine;

const W: usize = 4096;
const H: usize = 2560;
const SEGUNDOS: f64 = 30.0;
/// Passo entre as faixas horizontais do wash (px): 100 % de água e 85 % de pigmento com o raio abaixo.
const FAIXA: f64 = 12.0;
/// Raio REAL do dab (px) e pressão (escala §8, ~0..10).
const RAIO: f64 = 60.0;
const PRESSAO: f64 = 6.0;

fn faixa(e: &mut Engine, y: f64) {
    let ppf = 16.0;
    let (x0, x1) = (-64.0, W as f64 + 64.0);
    e.pointer_down(x0, y, None);
    let mut x = x0;
    while x < x1 {
        x = (x + ppf).min(x1);
        // A porta do produto (`pousa_o_dab`): pressão e raio REAIS, um pincel largo de wash.
        e.dispatch_pressure_dab(x, y, PRESSAO, RAIO);
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

fn main() {
    let segundos = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(SEGUNDOS);
    let mut e = Engine::new(W, H);
    e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
    e.sim.glaze_filme = ph2d_wet_paint::colorops::VELATURA_DO_FILME;
    e.sim.pickup_assentado = 0.0;
    e.sim.lei = ph2d_wet_paint::assenta::Lei::Cobre;
    e.sim.esconde = ph2d_wet_paint::colorops::TINTA_ESCONDE;
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    e.wet_canvas_now(); // o botão Wet do produto, antes do wash

    let t0 = Instant::now();
    let mut y = -32.0;
    while y < H as f64 + 32.0 {
        faixa(&mut e, y);
        y += FAIXA;
    }
    let g = e.active_grid();
    let (mut agua, mut pig) = (0usize, 0usize);
    for yy in 1..=g.h {
        let base = yy * g.s;
        for xx in 1..=g.w {
            let i = base + xx;
            agua += usize::from(g.film[i] > 0.0 || g.wet[i] > 0);
            pig += usize::from(g.susp[i] > 0.0);
        }
    }
    let n = (g.w * g.h) as f64;
    println!(
        "wash {W}x{H}: pintado em {:.1} s · água {:.2} % · pigmento {:.2} % · has_fluid {}",
        t0.elapsed().as_secs_f64(),
        100.0 * agua as f64 / n,
        100.0 * pig as f64 / n,
        g.has_fluid
    );
    println!(
        "READY pid={} ms={}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis())
    );

    let t = Instant::now();
    let mut passos = 0u64;
    let mut secos = 0u64;
    let mut ultimo = 0.0;
    while t.elapsed().as_secs_f64() < segundos {
        if e.step_simulation() {
            passos += 1;
        } else {
            secos += 1;
            if secos > 3 {
                break;
            }
        }
        let s = t.elapsed().as_secs_f64();
        if s - ultimo >= 5.0 {
            ultimo = s;
            println!(
                "  t={s:5.1} s passos={passos} dry_every={} frame={}",
                e.sim.dry_every, e.sim.frame
            );
        }
    }
    let dt = t.elapsed().as_secs_f64();
    println!(
        "água: {passos} passos em {dt:.2} s = {:.2} passos/s · {:.1} ms/passo (produto pede {} Hz)",
        passos as f64 / dt,
        1e3 * dt / passos.max(1) as f64,
        ph2d_wet_paint::sim::SIM_HZ
    );
}
