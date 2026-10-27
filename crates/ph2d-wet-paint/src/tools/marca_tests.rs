//! O Smear leva e dilui a marca do nivelado ([`super`], K9).

use crate::painter::{Engine, Tool};

const W: usize = 96;
const H: usize = 96;

/// Duas faixas de tinta (y = 40 e y = 58), a de cima MARCADA a 1 se `marca`; o Smear desce por x = 48 (y 30 → 76).
fn esfregado(marca: bool) -> Engine {
    let mut e = Engine::new(W, H);
    e.color = [220.0, 45.0, 40.0];
    for y in [40.0, 58.0] {
        e.begin_direct_stroke(0, 20.0, y);
        let mut x = 20.0;
        while x < 76.0 {
            x += 4.0;
            e.direct_segment(0, 4.0);
            e.dispatch_pressure_dab_lane(0, x, y, 5.0, 10.0, None, None, None);
        }
        e.end_direct_stroke();
    }
    if marca {
        let g = e.active_grid_mut();
        let s = g.s;
        crate::grid::nivelado::garante(g);
        for i in 0..g.cells {
            if i / s < 50 && g.susp[i] + g.sett[i] > 0.0 {
                g.nivelado[i] = 1.0;
            }
        }
    }
    let mut prev = None;
    let mut y = 30.0;
    while y <= 76.0 {
        e.dispatch_pressure_dab_tool(Tool::Smear, 48.0, y, 6.0, 12.0, prev);
        prev = Some([48.0, y]);
        y += 2.0;
    }
    e
}

/// A marca média PESADA PELA MASSA no caminho do Smear entre as linhas `y0..=y1` (x 44..52).
fn media(e: &Engine, y0: usize, y1: usize) -> f64 {
    let g = e.active_grid();
    let (mut a, mut w) = (0.0f64, 0.0f64);
    for y in y0..=y1 {
        for x in 44..=52 {
            let i = x + y * g.s;
            let m = f64::from(g.susp[i] + g.sett[i]);
            a += f64::from(crate::grid::nivelado::em(g, i)) * m;
            w += m;
        }
    }
    a / w
}

/// ⭐ A faixa de baixo (sem marca) recebe a marca que o Smear trouxe da de cima, DILUÍDA pela sua própria massa
/// (`0 < marca < 1`); fora do caminho fica a zero; sem marca na sessão o campo nem se aloca.
#[test]
fn o_smear_leva_e_dilui_a_marca_do_nivelado() {
    let e = esfregado(true);
    // O vão entre as faixas (quase sem massa própria) toma a marca que chega: LEVA.
    let vao = media(&e, 50, 52);
    assert!(vao > 0.9, "vão: {vao}");
    // A faixa de baixo (a sua massa sem marca) recebe-a DILUÍDA.
    let b = media(&e, 55, 63);
    assert!(b > 1e-4 && b < 0.1, "faixa de baixo no caminho: {b}");
    let g = e.active_grid();
    let longe = crate::grid::nivelado::em(g, 24 + 58 * g.s);
    assert_eq!(longe, 0.0, "fora do caminho a marca mexeu");
    let sem = esfregado(false);
    assert!(
        sem.active_grid().nivelado.is_empty(),
        "sem marca na sessão o Smear alocou-a"
    );
}
