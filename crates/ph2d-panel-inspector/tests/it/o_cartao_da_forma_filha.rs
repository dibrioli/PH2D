//! GEF02 B4 (opção B, ADR-0199): o corpo com a forma na filha só tem o Body; a filha «Collision Shape»
//! tem Shape e Collision Layers e diz de quem é; o corpo antigo oferece «Move shape to a child».

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit as E};
use ph2d_panel_inspector::ids::*;

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::with_body;

fn da_filha() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        has_body: false,
        has_collider: true,
        part_owner: "Torso".into(),
        shape_of: "Torso".into(),
        ..with_body()
    }
}

#[test]
fn b4_o_corpo_com_a_forma_na_filha_so_tem_o_cartao_body() {
    let info = InspectorPhysicsInfo {
        shape_on_child: true,
        ..with_body()
    };
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION).is_some());
    for (id, nome) in [
        (CARD_PHYS_SHAPE, "Shape"),
        (CARD_PHYS_LAYERS, "Collision Layers"),
        (INSP_PHYS_EDIT_SHAPE, "Edit shape on canvas"),
        (INSP_PHYS_MOVE_TO_CHILD, "Move shape to a child"),
    ] {
        assert!(achado(&r, id).is_none(), "o corpo não pinta {nome}");
    }
    sai();
}

#[test]
fn b4_a_filha_tem_shape_e_camadas_e_diz_de_quem_e() {
    let (mut host, mut st) = cena(da_filha(), [1.0, 1.0]);
    let (r, avisos) = ph2d_panel_inspector::censo_dos_avisos::medindo(|| pinta(&mut host, &mut st));
    assert!(achado(&r, CARD_PHYS_SHAPE).is_some(), "o cartão Shape");
    assert!(
        achado(&r, CARD_PHYS_LAYERS).is_some(),
        "o cartão Collision Layers"
    );
    assert!(
        achado(&r, INSP_PHYS_EDIT_SHAPE).is_some(),
        "as alças vivem na forma"
    );
    assert!(
        achado(&r, INSP_PHYS_ADD).is_none(),
        "sem «Make Independent Body»"
    );
    assert!(achado(&r, INSP_PHYS_MOVE_TO_CHILD).is_none());
    let frase = ph2d_i18n::tr_with("card.insp.phys.shape_of", &[("owner", &"Torso")]);
    assert_eq!(frase, "Shape of: Torso");
    assert!(
        avisos.iter().any(|a| a.texto == frase),
        "sem «Shape of»: {avisos:?}"
    );
    sai();
}

#[test]
fn b4_o_corpo_antigo_oferece_mover_a_forma_para_a_filha() {
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, CARD_PHYS_SHAPE).is_some(),
        "o Shape de hoje fica"
    );
    let b = achado(&r, INSP_PHYS_MOVE_TO_CHILD).expect("o botão");
    let a = clica(&mut host, &mut st, b);
    assert!(
        a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::MoveShapeToChild,
                ..
            }
        )),
        "{a:?}"
    );
    sai();
    let (mut host, mut st) = cena(with_body(), [1.0, 1.0]);
    let b = achado(&pinta(&mut host, &mut st), INSP_PHYS_MOVE_TO_CHILD).expect("botão");
    ph2d_panel_inspector::set_current_inspector_physics(Some(InspectorPhysicsInfo {
        shape_on_child: true,
        ..with_body()
    }));
    let a = clica(&mut host, &mut st, b);
    assert!(
        !a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::MoveShapeToChild,
                ..
            }
        )),
        "já na filha, o id não despacha: {a:?}"
    );
    sai();
}
