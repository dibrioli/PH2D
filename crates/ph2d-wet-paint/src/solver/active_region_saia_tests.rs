//! A saia por tiras de colunas × a serial, ao bit (B13.12 cura 4).

use super::*;
use crate::painter::Engine;

fn faixa(e: &mut Engine, y0: f64, x0: f64, x1: f64) {
    let mut y = y0;
    while y < y0 + 36.0 {
        e.pointer_down(x0, y, None);
        let mut x = x0;
        while x < x1 {
            x = (x + 8.0).min(x1);
            e.dispatch_pressure_dab(x, y, 4.0, 40.0);
        }
        e.pointer_up();
        while e.release_frame(x, y) {}
        y += 9.0;
    }
}

fn foto(g: &Grid) -> (Vec<u8>, [i32; 4], bool, Vec<i32>, Vec<i32>, Vec<u64>) {
    (
        g.active.clone(),
        [g.bx0, g.bx1, g.by0, g.by1],
        g.has_fluid,
        g.row_lo.clone(),
        g.row_hi.clone(),
        g.row_tiles.clone(),
    )
}

#[test]
fn a_saia_por_colunas_e_a_serial_ao_bit() {
    // 3 tiras de 256 colunas e faixas que ficam em pé e descem (frentes isoladas com saia).
    let mut e = Engine::new(700, 260);
    e.sliders.water = 1.0;
    for (k, y) in [20.0, 70.0, 150.0].into_iter().enumerate() {
        e.color = [40.0 + 60.0 * k as f64, 90.0, 200.0 - 50.0 * k as f64];
        faixa(&mut e, y, 30.0 + 100.0 * k as f64, 660.0 - 40.0 * k as f64);
    }
    let pools: Vec<rayon::ThreadPool> = [1, 2, 7]
        .into_iter()
        .map(|n| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .unwrap()
        })
        .collect();
    let mut disparos = 0;
    for passo in 0..40 {
        let g0 = e.active_grid().clone();
        let mut a = g0.clone();
        reconstroi(&mut a, Rows::Serial, false);
        let fa = foto(&a);
        disparos += a.active.iter().filter(|&&v| v == 2).count();
        for (p, pool) in pools.iter().enumerate() {
            for mode in [Rows::Serial, Rows::Parallel] {
                let mut b = g0.clone();
                pool.install(|| reconstroi(&mut b, mode, true));
                assert!(
                    foto(&b) == fa,
                    "passo {passo}, pool {p}, {mode:?}: a saia mudou"
                );
            }
        }
        e.step_simulation();
    }
    assert!(
        disparos > 1000,
        "a cena não exercita a saia: {disparos} células 2"
    );
}
