//! O deslizar de lado do trackpad na aba «Collision Layers» (InputMap onda 3 item 7): sobre a grade é o
//! Shift+roda (uma coluna); fora dela, nada. Pela porta da família (`DocumentSurface`).

use super::a_aba_das_camadas::{Mesa, abre, nomes_b};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_panel_physics::aba::{self, familia::ABA, familia::AbaDasCamadas};

#[test]
fn sobre_a_grade_o_deslizar_de_lado_e_o_shift_roda() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    ph2d_panel_physics::set_visible_layers((1u32 << 27) - 1);
    abre(&mut mesa);
    let g = aba::geometria_da_ultima().expect("a aba pintou");
    let (x, y) = (g.grade.x + g.grade.w * 0.5, g.grade.y + g.grade.h * 0.5);
    let mut f = AbaDasCamadas;
    let pos = || aba::geometria_da_ultima().map(|g| (g.l0, g.c0));

    assert!(f.wheel_sideways(ABA, x, y, -1.0), "de lado: uma coluna");
    assert_eq!(pos(), Some((0, 1)));
    assert!(
        f.wheel(ABA, x, y, 1.0, true),
        "o Shift+roda de igual grandeza volta"
    );
    assert_eq!(pos(), Some((0, 0)));
    assert!(
        !f.wheel_sideways(ABA, x, y, 0.0),
        "o vertical puro não rola de lado"
    );
    assert_eq!(pos(), Some((0, 0)));
    assert!(
        !f.wheel_sideways(ABA, g.grade.x - 5.0, y, -1.0),
        "fora da grade, nada"
    );
    assert_eq!(pos(), Some((0, 0)));
    aba::fecha();
}
