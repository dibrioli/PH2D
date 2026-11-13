//! n.º 6 (12/11): sem Real Oil em célula nenhuma a fotografia do chão não corre (o campo do óleo existe só pela
//! solubilidade — o Wet Wash a 4K pagava 33 MB e ~5 ms por traço para um chão todo zero). O mesmo chão, ao bit, com
//! e sem o salto; com Real Oil na grade o salto não corre (o controlo: o chão sai não vazio).

use super::{CHAO_SEM_REAL_SALTA, fotografa};
use crate::painter::Engine;
use std::sync::atomic::Ordering::Relaxed;

fn traco(e: &mut Engine, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
    e.pointer_down(x0, y0, None);
    for i in 1..=8 {
        let t = f64::from(i) / 8.0;
        e.pointer_frame(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
    }
    e.pointer_up();
    while e.release_frame(x1, y1) {}
}

/// O chão fotografado com o salto ligado e desligado, sobre a mesma grade.
fn os_dois(e: &mut Engine) -> (Vec<f32>, Vec<f32>) {
    CHAO_SEM_REAL_SALTA.store(true, Relaxed);
    fotografa(e.active_grid_mut(), false);
    let novo = e.active_grid().chao.clone();
    CHAO_SEM_REAL_SALTA.store(false, Relaxed);
    fotografa(e.active_grid_mut(), false);
    let velho = e.active_grid().chao.clone();
    CHAO_SEM_REAL_SALTA.store(true, Relaxed);
    (novo, velho)
}

fn pintada(oleo: bool) -> Engine {
    let mut e = Engine::new(256, 128);
    e.sim.oleo = oleo;
    e.set_knob(crate::tuning::Knob::Solubility, 0.6);
    e.set_knob(crate::tuning::Knob::PigmentPerDab, 3000.0);
    e.sliders.size = 9.0;
    traco(&mut e, (40.0, 40.0), (200.0, 90.0));
    for _ in 0..240 {
        e.step_simulation();
    }
    e
}

#[test]
fn sem_real_oil_o_chao_nao_se_fotografa_e_sai_o_mesmo() {
    let mut agua = pintada(false);
    let g = agua.active_grid();
    assert!(
        !g.oleo.is_empty() && !g.oleo_real,
        "a fixture: o campo do óleo existe só pela solubilidade (vazio {}, real {})",
        g.oleo.is_empty(),
        g.oleo_real
    );
    assert!(
        g.sett.iter().any(|&s| s > 0.0),
        "a fixture: há tinta assente"
    );
    let (novo, velho) = os_dois(&mut agua);
    assert!(novo.is_empty(), "sem Real Oil o chão fica vazio");
    assert_eq!(novo, velho, "o chão de antes também era vazio");

    let mut oleo = pintada(true);
    assert!(oleo.active_grid().oleo_real, "a fixture: Real Oil na grade");
    let (novo, velho) = os_dois(&mut oleo);
    assert!(
        !novo.is_empty(),
        "CONTROLO: com Real Oil o chão fotografa-se (o salto não corre)"
    );
    assert!(
        novo.iter()
            .zip(&velho)
            .all(|(a, b)| a.to_bits() == b.to_bits())
            && novo.len() == velho.len(),
        "o mesmo chão, ao bit"
    );
}
