//! B13.2: o `outflow` do advect Jacobi e o `div`/`prs` do `project` zeram só os troços dos tiles; o
//! oráculo é o zero do intervalo inteiro ([`super::ZERO_DO_INTERVALO`]), com o rascunho envenenado.
use super::ZERO_DO_INTERVALO;
use crate::painter::Engine;
use std::sync::atomic::Ordering::SeqCst;

struct ZeroAntigo;
impl ZeroAntigo {
    fn new() -> Self {
        ZERO_DO_INTERVALO.fetch_add(1, SeqCst);
        ZeroAntigo
    }
}
impl Drop for ZeroAntigo {
    fn drop(&mut self) {
        ZERO_DO_INTERVALO.fetch_sub(1, SeqCst);
    }
}

fn traco(e: &mut Engine, x: f64, y0: f64, y1: f64) {
    e.pointer_down(x, y0, None);
    let mut y = y0;
    while y < y1 {
        y = (y + 4.0).min(y1);
        e.pointer_frame(x + (y - y0) * 0.2, y);
        if e.sim_should_run() {
            e.step_simulation();
        }
    }
    e.pointer_up();
    while e.release_frame(x, y) {}
}

/// Duas manchas nas MESMAS linhas, longe (4 tiles secos entre elas), água a escorrer de lado; o digest de
/// todo campo persistente depois de cada um dos passos seguintes.
fn sessao(rf: usize, jacobi: bool, antigo: bool) -> Vec<u32> {
    let _g = antigo.then(ZeroAntigo::new);
    crate::grid::scratch::veneno::LIGADO.set(true);
    let mut e = Engine::with_flow_ratio(700, 160, rf);
    e.sim.order_invariant = jacobi;
    e.sliders.water = 1.0;
    e.sim.gravity_override = Some([1.0, 0.3]);
    traco(&mut e, 40.0, 30.0, 130.0);
    traco(&mut e, 620.0, 30.0, 130.0);
    let mut d = Vec::new();
    for _ in 0..24 {
        e.step_simulation();
        let g = e.active_grid();
        assert!(g.tiles_enabled && g.spans_enabled);
        for a in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
            d.extend(a.iter().map(|v| v.to_bits()));
        }
        d.extend(g.susp_rgb.iter().flatten().map(|v| v.to_bits()));
    }
    crate::grid::scratch::veneno::LIGADO.set(false);
    let g = e.active_grid();
    let longe = (0..g.h).any(|y| {
        g.film[y * g.s + 600..y * g.s + 690]
            .iter()
            .any(|&f| f > 0.0)
    });
    assert!(longe, "a 2.ª mancha secou: a cena não exercita o 2.º troço");
    d
}

#[test]
fn o_zero_pelos_troços_da_o_zero_do_intervalo_ao_bit() {
    for rf in [1, 2] {
        for jacobi in [false, true] {
            let novo = sessao(rf, jacobi, false);
            let antigo = sessao(rf, jacobi, true);
            let dif = novo.iter().zip(&antigo).filter(|(a, b)| a != b).count();
            assert_eq!(dif, 0, "rf {rf} · jacobi {jacobi}: {dif} palavras diferem");
        }
    }
}
