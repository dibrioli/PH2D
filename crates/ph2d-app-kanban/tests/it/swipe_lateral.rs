//! O deslizar de lado do trackpad (InputMap onda 3 item 7) = o Shift+roda onde ele rola de lado (o quadro,
//! a Tabela); vertical = a roda; diagonal = cada eixo uma vez. Pela porta do hero, contra um gémeo.

use crate::{AREA, Mesa, centro};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
use ph2d_kanban_views::table::Column;

/// O gesto `d` num, e no gémeo as rodas `(dy, shift)` que ele tem de valer.
fn passo(g: &mut Mesa, r: &mut Mesa, p: (f32, f32), d: [f32; 2], rodas: &[(f32, bool)]) {
    let _ = scroll(&mut g.hero, p.0, p.1, BoardScroll::Pan(d));
    g.quadro();
    for &(dy, shift) in rodas {
        r.hero.store.set_shift_held(shift);
        let _ = r.roda(p.0, p.1, dy);
    }
    r.hero.store.set_shift_held(false);
}

const PASSOS: [([f32; 2], &[(f32, bool)]); 3] = [
    ([-48.0, 0.0], &[(-48.0, true)]),
    ([0.0, -48.0], &[(-48.0, false)]),
    ([-32.0, -16.0], &[(-32.0, true), (-16.0, false)]),
];

#[test]
fn no_quadro_o_deslizar_de_lado_e_o_shift_roda() {
    let ((mut g, id), (mut r, _)) = (Mesa::cena1(), Mesa::cena1());
    let v = |m: &mut Mesa| m.docs().probe_view(id.0).expect("vista");
    for (i, (d, rodas)) in PASSOS.into_iter().enumerate() {
        passo(&mut g, &mut r, centro(AREA), d, rodas);
        assert_eq!(v(&mut g), v(&mut r), "passo {i}");
        assert!(i > 0 || v(&mut g).scroll_x > 0.0, "rolou de lado");
    }
}

#[test]
fn na_tabela_o_deslizar_de_lado_e_o_shift_roda() {
    let ((mut g, id), (mut r, _)) = (Mesa::cena1(), Mesa::cena1());
    crate::j1_vistas::vista(&mut g, id, 1);
    crate::j1_vistas::vista(&mut r, id, 1);
    let cel = |m: &mut Mesa| m.docs().probe_table_cell(id.0, 0, Column::Due);
    let p = centro(cel(&mut g).expect("pintou").1);
    for (i, (d, rodas)) in PASSOS.into_iter().enumerate() {
        passo(&mut g, &mut r, p, d, rodas);
        assert_eq!(cel(&mut g), cel(&mut r), "passo {i}");
    }
}
