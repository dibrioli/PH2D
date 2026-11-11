//! B35.10 — o Base evaporation fundiu-se no Dry Speed: a lei usa a constante `2` (a fábrica do knob, ao bit) e o
//! knob deixa de mover a secagem.

use super::*;
use crate::painter::Engine;
use crate::tuning::KNOB_DEFS;

fn seca(base_knob: Option<f64>) -> Vec<u32> {
    let mut e = Engine::new(160, 120);
    e.color = [190.0, 40.0, 30.0];
    if let Some(v) = base_knob {
        e.set_knob(Knob::BaseEvaporation, v);
    }
    e.pointer_down(30.0, 60.0, None);
    let mut x: f64 = 30.0;
    while x < 130.0 {
        x = (x + 4.0).min(130.0);
        e.pointer_frame(x, 60.0);
        if e.sim_should_run() {
            e.step_simulation();
        }
    }
    e.pointer_up();
    for _ in 0..40 {
        e.step_simulation();
    }
    let g = e.active_grid();
    g.film
        .iter()
        .chain(&g.susp)
        .chain(&g.sett)
        .map(|v| v.to_bits())
        .collect()
}

#[test]
fn a_evaporacao_base_fixa_em_2_da_a_mesma_secagem_ao_bit() {
    let def = KNOB_DEFS[Knob::BaseEvaporation as usize].default;
    assert_eq!(
        def.to_bits(),
        EVAPORACAO_BASE.to_bits(),
        "a constante é a fábrica do knob"
    );
    let fabrica = seca(None);
    assert!(
        fabrica == seca(Some(def)),
        "a fábrica explícita muda a secagem"
    );
    assert!(
        fabrica == seca(Some(7.0)),
        "o knob fundido ainda move a secagem"
    );
}
