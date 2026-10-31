//! O piso do espalhar ([`super::espalha::Espalha::piso`]) pela porta do produto (`Engine`): o óleo grosso pousado
//! sobre o óleo grosso SOMA altura e nunca cava a camada de baixo. Doc:
//! `docs/Painter/Arquitetura/RELEVO_oleo_ondas_e_crateras.md` §Mecanismo 2. Medido (31/10, esta cena): com
//! `trail::sonda::SEM_PISO` (a lei de antes) 32 de 340 células do contacto cavam; com o piso, 0.
//! O Drag/Smear do pincel vão a 0: o arrasto é OUTRA lei (move tinta ao longo do traço, `sem_drag` na sonda) e
//! cavaria sozinho, com ou sem piso.

use super::espalha::ESPALHA;
use crate::grid::espessura;
use crate::painter::Engine;
use crate::tuning::Knob;

const L: usize = 96;
const Y1: f64 = 48.0;
/// O traço 2 paralelo, meio sobreposto.
const Y2: f64 = 52.0;

fn traco(e: &mut Engine, y: f64) {
    e.pointer_down(12.0, y, None);
    for i in 1..=12 {
        e.pointer_frame(12.0 + 72.0 * f64::from(i) / 12.0, y);
    }
    e.pointer_up();
    while e.release_frame(84.0, y) {}
}

fn pausa(e: &mut Engine) {
    for _ in 0..120 {
        e.step_simulation();
    }
}

/// Óleo grosso de Real Oil, Water 0, carga alta (a espessura passa a massa do tecto de 3000: só aí o espalhar morde).
fn motor() -> Engine {
    let mut e = Engine::new(L, L);
    e.sim.oleo = true;
    e.set_knob(Knob::PigmentPerDab, 3000.0);
    e.set_knob(Knob::Intensity, 10.0);
    e.set_knob(Knob::Drag, 0.0);
    e.set_knob(Knob::Smear, 0.0);
    e.sliders.pigmento = 1.0;
    e.sliders.water = 0.0;
    e.sliders.size = 1.0;
    e
}

fn alturas(e: &Engine) -> Vec<f64> {
    let g = e.active_grid();
    (0..g.cells).map(|i| espessura::em(g, i)).collect()
}

#[test]
fn o_oleo_grosso_sobre_o_oleo_grosso_soma_massa_e_nao_cava() {
    let mut e = motor();
    traco(&mut e, Y1);
    pausa(&mut e);
    let h1 = alturas(&e);
    traco(&mut e, Y2);
    pausa(&mut e);
    let h2 = alturas(&e);
    let mut so = motor();
    traco(&mut so, Y2);
    pausa(&mut so);
    let h_so = alturas(&so);

    let max1 = h1.iter().copied().fold(0.0, f64::max);
    let max_so = h_so.iter().copied().fold(0.0, f64::max);
    assert!(
        max1 > 3000.0 && max_so > 0.0,
        "o óleo não é grosso: {max1:.0}"
    );
    let contacto: Vec<usize> = (0..h1.len())
        .filter(|&i| h1[i] > 0.05 * max1 && h_so[i] > 0.05 * max_so)
        .collect();
    assert!(contacto.len() > 100, "contacto: {} células", contacto.len());
    let tol = 0.02 * max1;
    let crateras = contacto.iter().filter(|&&i| h2[i] < h1[i] - tol).count();
    assert!(
        crateras == 0,
        "o traço 2 cavou a camada 1: {crateras} de {} células do contacto abaixo da altura de antes − {tol:.0}",
        contacto.len()
    );
    // O volume que a lei promete: `max(t₀, t₀·(1 − ESPALHA) + v)` onde há tinta de baixo, `v` onde não há.
    let prometido: f64 = (0..h1.len())
        .map(|i| {
            if h1[i] > 0.0 && h_so[i] > 0.0 {
                h1[i].max(h1[i] * (1.0 - ESPALHA) + h_so[i])
            } else {
                h1[i] + h_so[i]
            }
        })
        .sum();
    let depois: f64 = h2.iter().sum();
    assert!(
        depois >= 0.97 * prometido,
        "o volume sumiu: Σh depois {depois:.0} < 0,97 × o prometido {prometido:.0} (Σh antes {:.0} + traço 2 {:.0})",
        h1.iter().sum::<f64>(),
        h_so.iter().sum::<f64>()
    );
}
