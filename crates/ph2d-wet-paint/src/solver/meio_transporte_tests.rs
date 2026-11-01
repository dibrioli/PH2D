//! O meio anda com o líquido (J9): a célula seca não tem voto, o líquido leva o seu meio, e as rotas dão o mesmo bit.
use super::super::advect_jacobi_rows;
use crate::grid::oleo_celula::OleoCelula;
use crate::painter::Engine;
use crate::par::Rows;
use crate::tuning::Knob;

/// Um traço de água no meio da tela; o campo `oleo` com o meio `molhada` nas células com filme e `seca` nas outras.
/// Corre `passos` da advecção com fluxo para a direita e devolve `(filme antes, filme depois, meio depois)`.
fn corre(molhada: f32, seca: f32, mode: Rows) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut e = Engine::with_flow_ratio(120, 80, 1);
    e.sliders.water = 1.0;
    e.sliders.size = 0.2;
    e.pointer_down(30.0, 40.0, None);
    for k in 1..=6 {
        e.pointer_frame(30.0 + 4.0 * f64::from(k), 40.0);
    }
    e.pointer_up();
    e.tuning.set(Knob::MaxVelocity, 2.0);
    let p = e.sim.gather_params(&e.tuning);
    let g = e.active_grid_mut();
    let antes = g.film.clone();
    g.oleo = (0..g.cells)
        .map(|i| OleoCelula {
            meio: if g.film[i] > 0.0 { molhada } else { seca },
            ..OleoCelula::NEUTRA
        })
        .collect();
    for _ in 0..12 {
        crate::solver::rebuild_active_region(g);
        for v in g.flow_x.iter_mut() {
            *v = 1.5;
        }
        g.flow_y.fill(0.0);
        advect_jacobi_rows(g, &p, 0.0, 0.0, mode);
    }
    let meio = g.oleo.iter().map(|c| c.meio).collect();
    (antes, g.film.clone(), meio)
}

#[test]
fn a_agua_que_chega_ao_oleo_seco_e_agua() {
    let (antes, depois, meio) = corre(0.0, 1.0, Rows::Serial);
    let novas: Vec<usize> = (0..antes.len())
        .filter(|&i| antes[i] <= 0.0 && depois[i] > 0.0)
        .collect();
    assert!(
        novas.len() >= 10,
        "a água não andou: {} células novas",
        novas.len()
    );
    for i in novas {
        assert_eq!(
            meio[i], 0.0,
            "célula {i}: o óleo seco impôs o meio {}",
            meio[i]
        );
    }
}

#[test]
fn o_oleo_molhado_leva_o_seu_meio_ao_papel() {
    let (antes, depois, meio) = corre(1.0, 0.0, Rows::Serial);
    let novas: Vec<usize> = (0..antes.len())
        .filter(|&i| antes[i] <= 0.0 && depois[i] > 0.0)
        .collect();
    assert!(
        novas.len() >= 10,
        "o líquido não andou: {} células novas",
        novas.len()
    );
    for i in novas {
        assert_eq!(meio[i], 1.0, "célula {i}: o óleo chegou como {}", meio[i]);
    }
}

#[test]
fn o_meio_que_anda_e_o_mesmo_bit_nas_rotas() {
    let base = corre(0.3, 1.0, Rows::Serial);
    for mode in [Rows::Parallel, Rows::Medida(0), Rows::Medida(u64::MAX)] {
        let r = corre(0.3, 1.0, mode);
        assert!(
            r.2.iter()
                .zip(&base.2)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "{mode:?}: o meio mudou de bits"
        );
        assert!(
            r.1.iter()
                .zip(&base.1)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
    }
}
