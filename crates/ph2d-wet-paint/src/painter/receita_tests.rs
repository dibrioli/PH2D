//! E4 — os três portões da regra do dono, medidos no DEPÓSITO real (a porta da faixa, a que o pincel usa).

use super::*;
use crate::painter::Engine;

/// Um dab pela porta do produto; devolve (Σ pigmento, Σ água) na janela do rasto.
fn um_dab(pigmento: f64, agua: f64, volume: f64, pressao: f64, b: f64) -> (f64, f64) {
    let mut e = Engine::new(64, 64);
    e.set_knob(Knob::PigmentPerDab, pigmento);
    e.sliders.water = agua;
    e.sliders.pigmento = volume;
    e.sliders.pressure = pressao;
    e.color = [220.0, 45.0, 40.0];
    e.begin_direct_stroke(0, 32.0, 32.0);
    e.direct_segment(0, 4.0);
    e.dispatch_pressure_dab_lane(0, 32.0, 32.0, b, 10.0, None, None, None);
    let (pig, water) = e.lane_trails[0].planes_for_measure();
    let sp: f64 = pig.iter().map(|&v| f64::from(v)).sum();
    let sw: f64 = water.iter().map(|&v| f64::from(v)).sum();
    assert!(sp + sw > 0.0, "o dab não pousou nada");
    (sp, sw)
}

fn volume((p, w): (f64, f64)) -> f64 {
    p + K_W * w
}

fn phi((p, w): (f64, f64)) -> f64 {
    p / volume((p, w))
}

fn espalhamento(v: &[f64]) -> f64 {
    let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    hi / lo - 1.0
}

/// Tolerância: a soma em `f32` de ~300 células.
const TOL: f64 = 1e-4;

#[test]
fn volume_fixo_varrer_pigmento_e_agua_nao_muda_o_volume_depositado() {
    let mut v = Vec::new();
    for pig in [50.0, 300.0, 600.0, 1500.0] {
        for agua in [0.2, 0.6, 1.0] {
            v.push(volume(um_dab(pig, agua, 1.0, PRESSAO_DE_REFERENCIA, 5.0)));
        }
    }
    let e = espalhamento(&v);
    eprintln!("volume: espalhamento relativo {e:.6} sobre {v:?}");
    assert!(e < TOL, "Pigmento/Água mudaram o volume em {e:.4}");
}

#[test]
fn pigmento_e_agua_fixos_varrer_volume_nao_muda_a_concentracao() {
    let d: Vec<_> = [0.25, 0.5, 1.0]
        .iter()
        .map(|&vol| um_dab(300.0, 0.6, vol, PRESSAO_DE_REFERENCIA, 5.0))
        .collect();
    let f: Vec<f64> = d.iter().map(|&x| phi(x)).collect();
    let q: Vec<f64> = d.iter().map(|&x| volume(x)).collect();
    let e = espalhamento(&f);
    eprintln!("φ: espalhamento {e:.6} sobre {f:?}; volume {q:?}");
    assert!(e < TOL, "o Volume mudou a concentração em {e:.4}");
    assert!(
        q[0] < q[1] && q[1] < q[2],
        "o Volume tem de mexer na quantidade: {q:?}"
    );
}

#[test]
fn a_pressao_so_toca_a_quantidade() {
    let mut f = Vec::new();
    let mut q = Vec::new();
    for pressao in [0.1, 0.7, 1.0] {
        for b in [2.0, 5.0, 8.0] {
            let d = um_dab(300.0, 0.6, 1.0, pressao, b);
            f.push(phi(d));
            q.push(volume(d));
        }
    }
    let e = espalhamento(&f);
    eprintln!("φ por pressão: espalhamento {e:.6} sobre {f:?}; volume {q:?}");
    assert!(e < TOL, "a pressão mudou a receita em {e:.4}");
    assert!(q[0] < q[1], "a pressão tem de mexer na quantidade: {q:?}");
}
