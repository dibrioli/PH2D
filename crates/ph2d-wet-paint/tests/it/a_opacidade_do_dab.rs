//! **A opacidade do dab** (prompt 45, doc `45_medicao_o_esmaecer.md`): a pressão do pincel → opacidade pousa
//! DEPOIS do tecto do carimbo — antes dele o miolo de um dab carregado satura e só a borda esmaece.

use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

/// A massa de pigmento que UM dab de `intensity` e `opacity` pousa, com o pó `po`.
fn massa(intensity: f64, opacity: f64, po: f64) -> f64 {
    let mut e = Engine::new(200, 200);
    e.tuning.set(Knob::Drag, 0.0);
    e.tuning.set(Knob::PaperGate, 0.0);
    e.tuning.set(Knob::Dust, po);
    e.sim.semente_do_po = 7;
    let p = e.sim.gather_params(&e.tuning);
    let tex = vec![1.0; e.bristle_texture_for_measure().len()];
    let mut t = Trail::default();
    t.start_stroke(100.0, 100.0, [23.0, 101.0, 179.0], TrailMode::Paint);
    let d = Dab {
        x: 100.0,
        y: 100.0,
        r: 30.0,
        hardness: 9.0,
        intensity,
        water_amount: 0.0,
        dry_gate: 0.0,
        reach: 1.0,
        window_reach: 1.0,
        opacity,
    };
    t.accumulate_paint(e.active_grid_mut(), &p, &tex, &d, false);
    t.planes_for_measure().0.iter().map(|&v| f64::from(v)).sum()
}

/// ⭐ Meia opacidade pousa METADE da tinta — no carimbo contínuo e no pó — num dab de miolo saturado, onde
/// meia INTENSIDADE (a porta de antes do prompt 45) ainda pousa mais de 80 %.
#[test]
fn meia_opacidade_pousa_metade_da_tinta() {
    for po in [0.0, 0.8] {
        let inteira = massa(3.0, 1.0, po);
        let meia = massa(3.0, 0.5, po);
        assert!(inteira > 0.0);
        assert!(
            (meia / inteira - 0.5).abs() < 1e-3,
            "pó {po}: meia opacidade pousa {:.3} da tinta",
            meia / inteira
        );
        assert!(
            massa(1.5, 1.0, po) / inteira > 0.8,
            "pó {po}: o miolo deixou de saturar — a régua não distingue a porta"
        );
    }
}

/// Opacidade nula não pousa pigmento.
#[test]
fn opacidade_nula_nao_pousa_tinta() {
    assert_eq!(massa(3.0, 0.0, 0.0), 0.0);
}
