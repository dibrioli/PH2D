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

/// Dono 09/11: a forma é um OBJECTO filho — apagá-la na Hierarchy (ou Delete no canvas) é o mesmo passo de
/// undo —, por isso o cartão dela não oferece «Remove Shape»: nem nos cartões, nem na face de referência, nem no
/// handler (o id vive no store a sessão inteira), nem no recibo. O corpo guarda «Remove Physics Body».
#[test]
fn b4_a_filha_nao_oferece_remove_shape_e_o_corpo_guarda_o_remove() {
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::PanelHostInternal;
    let (mut host, mut st) = cena(da_filha(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(achado(&r, CARD_PHYS_SHAPE).is_some(), "o cartão pintou");
    assert!(
        achado(&r, INSP_PHYS_REMOVE).is_none(),
        "a filha não pinta «Remove Shape»"
    );
    let _ = host.drained_actions();
    let _ = host.apply_panel_event::<ph2d_panel_inspector::InspectorPanel>(
        &mut st,
        WidgetEvent::Click(INSP_PHYS_REMOVE),
    );
    let a = host.drained_actions();
    assert!(
        !a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::Remove,
                ..
            }
        )),
        "o id que ficou no store não despacha na filha: {a:?}"
    );
    ph2d_panel_inspector::set_current_inspector_physics(None);
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none(),
        "nada foi removido: sem recibo «Shape removed»"
    );
    sai();
    // A face de referência (`PH2D_CARDS=0`) diz o mesmo.
    let mut host = crate::host_em_metros();
    let mut st = ph2d_panel_inspector::InspectorState::default();
    ph2d_panel_inspector::set_current_inspector_physics(Some(da_filha()));
    let r = host.paint::<ph2d_panel_inspector::InspectorPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 320.0, 2400.0),
    );
    assert!(
        r.iter().any(|(n, _)| *n == INSP_PHYS_DENSITY),
        "a face de referência pintou a forma"
    );
    assert!(
        achado(&r, INSP_PHYS_REMOVE).is_none(),
        "a face de referência não pinta «Remove Shape» na filha"
    );
    sai();
    // O corpo com a forma na filha: «Remove Physics Body» fica e despacha.
    let corpo = InspectorPhysicsInfo {
        shape_on_child: true,
        ..with_body()
    };
    let (mut host, mut st) = cena(corpo, [1.0, 1.0]);
    let b = achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVE).expect("«Remove Physics Body»");
    let a = clica(&mut host, &mut st, b);
    assert!(
        a.iter().any(|a| matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::Remove,
                ..
            }
        )),
        "{a:?}"
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

/// (W4 a, c) **A forma «Outline» pinta as três linhas do traço, e o aviso só quando o tecto subiu o
/// Detail.** *Mutação: `aviso` sem o guarda do `raised` ⇒ a frase aparece sem subir, vermelho.*
#[test]
fn w4_o_outline_pinta_as_linhas_e_o_aviso_so_quando_subiu() {
    use ph2d_editor_core::screens::hero::InspectorOutlineInfo;
    let frase = ph2d_i18n::tr_with(
        "card.insp.phys.outline_raised",
        &[("x", &"3"), ("n", &256u32)],
    );
    for raised in [None, Some((3.0, 256))] {
        let info = InspectorPhysicsInfo {
            shape_tag: 3,
            outline: Some(InspectorOutlineInfo {
                detail_px: 2.0,
                alpha_cutoff: 0.5,
                specks_px: 0.0,
                raised,
            }),
            ..da_filha()
        };
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let (r, avisos) =
            ph2d_panel_inspector::censo_dos_avisos::medindo(|| pinta(&mut host, &mut st));
        for id in [
            INSP_PHYS_OUTLINE_DETAIL,
            INSP_PHYS_OUTLINE_ALPHA,
            INSP_PHYS_OUTLINE_SPECKS,
        ] {
            assert!(achado(&r, id).is_some(), "a linha {id:?}");
        }
        assert!(achado(&r, INSP_PHYS_HALF_X).is_none(), "sem Width");
        let tem = avisos.iter().any(|a| a.texto == frase);
        assert_eq!(tem, raised.is_some(), "{frase:?} em {avisos:?}");
        sai();
    }
}
