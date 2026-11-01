//! Sonda da E10g (`docs/Painter/Arquitetura/E10g_oleo_diluido.md`): o óleo diluído escorre? Água × Óleo por Dilution,
//! com a tela inclinada e plana, nas duas leis; a viscosidade e o endurecimento pelas réguas de [`ph2d_wet_paint::meio`].
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke --test e10g_oleo_diluido -- --ignored --nocapture`

use ph2d_wet_paint::meio::{PAPEL_ENDURECE, PAPEL_ETA};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::sim::LeiDaTinta;
use ph2d_wet_paint::tuning::Knob;
use std::sync::atomic::Ordering;

const W: usize = 200;
const H: usize = 200;
const Y0: f64 = 50.0;

fn motor(lei: LeiDaTinta, d: f64, m: f64, inclinada: bool) -> Engine {
    let mut e = Engine::new(W, H);
    e.sim.set_tinta(lei);
    e.set_knob(Knob::Dilution, d);
    e.set_knob(Knob::Medium, m);
    e.sliders.size = ((30.0 * 0.5 - 2.0) / 33.0f64).clamp(0.0, 1.0);
    e.sliders.pressure = 1.0;
    e.sim.tilt_on = inclinada;
    e.sim.tilt_dir_x = 0.0;
    e.sim.tilt_dir_y = 1.0;
    e
}

struct Leitura {
    /// Fracção da massa abaixo da faixa do traço (`y > Y0 + 20`).
    escorreu: f64,
    /// O centro de massa em y menos `Y0`.
    desce: f64,
    /// A água que resta.
    agua: f64,
}

fn le(e: &Engine) -> Leitura {
    let g = e.active_grid();
    let (mut tot, mut baixo, mut my, mut agua) = (0.0, 0.0, 0.0, 0.0);
    for y in 1..g.h.min(H) {
        for x in 1..g.w.min(W) {
            let i = x + y * g.s;
            let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
            tot += m;
            my += m * y as f64;
            agua += f64::from(g.film[i]);
            if y as f64 > Y0 + 20.0 {
                baixo += m;
            }
        }
    }
    Leitura {
        escorreu: baixo / tot.max(1e-9),
        desce: my / tot.max(1e-9) - Y0,
        agua,
    }
}

fn corre(lei: LeiDaTinta, d: f64, m: f64, inclinada: bool) -> (Leitura, Leitura, u32) {
    let mut e = motor(lei, d, m, inclinada);
    e.pointer_down(30.0, Y0, None);
    for i in 1..=40 {
        e.pointer_frame(30.0 + 140.0 * f64::from(i) / 40.0, Y0);
    }
    e.pointer_up();
    while e.release_frame(170.0, Y0) {}
    let mut cedo = None;
    let mut seco = 0;
    for n in 1..=6000u32 {
        e.step_simulation();
        if n == 300 {
            cedo = Some(le(&e));
        }
        if le(&e).agua <= 1e-6 {
            seco = n;
            break;
        }
    }
    (cedo.unwrap_or_else(|| le(&e)), le(&e), seco)
}

#[test]
#[ignore = "sonda E10g"]
fn e10g_o_oleo_diluido_escorre() {
    let etas: Vec<f32> = std::env::var("PH2D_E10G_ETA")
        .map(|s| s.split(',').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_else(|_| vec![0.0]);
    let ends: Vec<f32> = std::env::var("PH2D_E10G_ENDURECE")
        .map(|s| s.split(',').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_else(|_| vec![0.0]);
    println!(
        "lei | inclinada | eta | endurece | meio | Dilution | 300: escorreu desce água | fim: escorreu desce | seco no passo"
    );
    for lei in [LeiDaTinta::Velha, LeiDaTinta::Pigmento] {
        for inclinada in [true, false] {
            for &eta in &etas {
                for &en in &ends {
                    PAPEL_ETA.store(eta.to_bits(), Ordering::Relaxed);
                    PAPEL_ENDURECE.store(en.to_bits(), Ordering::Relaxed);
                    for m in [0.0, 1.0] {
                        for d in [0.0, 0.3, 0.6, 0.9] {
                            let (a, b, seco) = corre(lei, d, m, inclinada);
                            println!(
                                "{lei:?} | {inclinada} | {eta} | {en} | {m} | {d} | {:.3} {:.1} {:.1} | {:.3} {:.1} | {}",
                                a.escorreu,
                                a.desce,
                                a.agua,
                                b.escorreu,
                                b.desce,
                                if seco == 0 {
                                    "não secou em 6000".into()
                                } else {
                                    seco.to_string()
                                }
                            );
                        }
                    }
                }
            }
        }
    }
    PAPEL_ETA.store(0, Ordering::Relaxed);
    PAPEL_ENDURECE.store(0, Ordering::Relaxed);
}
