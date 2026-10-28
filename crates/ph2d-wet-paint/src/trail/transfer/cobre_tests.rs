//! A lei de UMA célula do [`super::cobre_a_outra`] ([`crate::tuning::Knob::WetCover`], J6 lacuna 7).

use super::cobre_a_outra;
use crate::grid::Grid;
use crate::opacity::alpha_of_mass;

const VERMELHO: [f32; 3] = [191.0, 0.0, 18.0];
const AZUL: [f32; 3] = [23.0, 101.0, 179.0];

fn celula(rgb: [f32; 3], massa: f32) -> (Grid, usize) {
    let mut g = Grid::new(8, 8);
    let i = 4 + 4 * g.s;
    g.susp[i] = massa;
    g.susp_rgb[i] = rgb;
    g.film[i] = 1.0;
    (g, i)
}

/// O azul que pousa no vermelho molhado esconde a parte `q × peso` dele (a massa sai) e devolve `q` para a mistura.
#[test]
fn o_que_a_tinta_nova_esconde_sai_da_celula() {
    let (m, in_a) = (400.0_f32, 0.3);
    let (mut g, i) = celula(VERMELHO, m);
    let q = cobre_a_outra(&mut g, i, 1.0, in_a, AZUL);
    let peso = in_a / (alpha_of_mass(f64::from(m)) * (1.0 - in_a) + in_a);
    assert!(q > 0.9, "o vermelho não é OUTRA tinta para o azul: q {q:.3}");
    let esperado = f64::from(m) * (1.0 - q * peso);
    assert!(
        (f64::from(g.susp[i]) - esperado).abs() < 1e-3 && f64::from(g.susp[i]) < f64::from(m) * 0.8,
        "a massa escondida não saiu: {} (esperado {esperado:.1})",
        g.susp[i]
    );
    // A metade da força esconde menos.
    let (mut h, j) = celula(VERMELHO, m);
    let q2 = cobre_a_outra(&mut h, j, 0.5, in_a, AZUL);
    assert!((q2 - 0.5 * q).abs() < 1e-9 && h.susp[j] > g.susp[i]);
}

/// A MESMA tinta não se cobre: nada sai e a mistura é a de sempre (`q = 0`).
#[test]
fn a_mesma_tinta_nao_se_cobre() {
    let (mut g, i) = celula(AZUL, 400.0);
    assert_eq!(cobre_a_outra(&mut g, i, 1.0, 0.3, AZUL), 0.0);
    assert_eq!(g.susp[i], 400.0);
}
