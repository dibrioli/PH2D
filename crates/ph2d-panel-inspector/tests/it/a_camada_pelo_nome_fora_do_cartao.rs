//! ⭐ **A camada pelo NOME fora do cartão** (dono 07/11, fecho da GEF02). A face clássica (`PH2D_CARDS=0`) e o Ray
//! Sensor diziam só o número da camada.
//!
//! Mutações: o chip da face clássica sem o contorno, ou com a cor de outra camada; a lista sem as camadas além da 7
//! (os chips «0»–«7» de antes); o nome do Ray noutra camada ou por cima do campo de número.

use crate::seam_physics::{click_real, expect, with_body};
use ph2d_editor_core::screens::hero::PhysicsFieldEdit;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::camadas_fisica::set_current;
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_physics,
    set_current_inspector_ray,
};

fn mundo() {
    let mut nomes: [String; 32] = Default::default();
    for (i, n) in [(0, "Ground"), (1, "Player"), (2, "Props"), (20, "Lava")] {
        nomes[i] = n.into();
    }
    set_current(nomes, [u32::MAX; 32], 0b111 | 1 << 20);
}

fn fim() {
    set_current(Default::default(), [u32::MAX; 32], 0xFF);
    set_cards_on(false);
}

#[test]
fn a_face_classica_escolhe_a_camada_pelo_nome_em_todas_as_visiveis() {
    set_cards_on(false);
    mundo();
    let mut info = with_body();
    info.layer = 2;
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(info.clone()));
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 2400.0));
    set_current_inspector_physics(None);
    let chip = rects
        .iter()
        .find(|(n, _)| *n == INSP_PHYS_LAYER_PICK)
        .map(|(_, r)| *r)
        .expect("a face clássica pinta o seletor Layer");
    assert!(
        !rects.iter().any(|(n, _)| INSP_PHYS_LAYER.contains(n)),
        "fechada, nenhum chip de número"
    );
    let c = ph2d_editor_core::paint::layer_outline_of(INSP_PHYS_LAYER_PICK).expect("contorno");
    assert_eq!(
        (c.camada, c.rect),
        (2, chip),
        "o chip veste a cor da camada do objecto"
    );
    // A camada 20 (além dos oito chips de antes) escolhe-se pelo clique real, um passo.
    expect(
        &click_real(info, INSP_PHYS_LAYER[20]),
        PhysicsFieldEdit::Layer(20),
        "a camada 20 pela lista da face clássica",
    );
    fim();
}

#[test]
fn o_ray_sensor_diz_o_nome_da_camada_ao_lado_do_numero() {
    mundo();
    let mut olho = crate::a_seccao_ray_sensor_esta_viva::olho();
    olho.layer = 20;
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_ray(Some(olho));
    let tela = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let rects = host.paint::<InspectorPanel>(&mut state, tela);
    set_current_inspector_ray(None);
    let campo = rects
        .iter()
        .find(|(n, _)| *n == INSP_RAY_LAYER)
        .map(|(_, r)| *r)
        .expect("o campo de número continua");
    let nome =
        ph2d_editor_core::paint::layer_outline_of(INSP_RAY_LAYER).expect("o nome no contorno");
    assert_eq!(nome.camada, 20, "o nome é o da camada do raio");
    assert!(
        nome.rect.x >= campo.x + campo.w,
        "o nome fica à direita do campo, sem o tapar: {:?} × {campo:?}",
        nome.rect
    );
    assert!(
        (nome.rect.y - campo.y).abs() < 0.5 && (nome.rect.h - campo.h).abs() < 0.5,
        "à altura da caixa do número: {:?} × {campo:?}",
        nome.rect
    );
    fim();
}
