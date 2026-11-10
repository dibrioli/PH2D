//! Duas ordens do dono (09/11) no cartão Shape da forma de colisão: depois da 1.ª edição de pontos à mão
//! (`custom`), Detail, Alpha cutoff e Ignore specks under só mudam DENTRO de «Edit shape on canvas»; e a forma
//! SOLTA (sem corpo acima) perde «Remove Shape» — apaga-se na Hierarchy.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{
    InspectorOutlineInfo, InspectorPhysicsInfo, PhysicsFieldEdit as E,
};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, set_key_marks};
use ph2d_panel_inspector::ids::*;

use crate::o_cartao_da_fisica::{achado, cena, pinta, sai};
use crate::seam_physics::{as_part, with_body};

fn contorno(editando: bool, custom: bool) -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        has_body: false,
        has_collider: true,
        part_owner: "Torso".into(),
        shape_of: "Torso".into(),
        shape_tag: 3,
        shape_edit_on: editando,
        estica: Vec::new(),
        outline: Some(InspectorOutlineInfo {
            detail_px: 2.0,
            alpha_cutoff: 0.5,
            specks_px: 0.0,
            raised: None,
            traced: true,
            merged: None,
            custom,
        }),
        ..with_body()
    }
}

const TRES: [ph2d_a11y::NodeId; 3] = [
    INSP_PHYS_OUTLINE_DETAIL,
    INSP_PHYS_OUTLINE_ALPHA,
    INSP_PHYS_OUTLINE_SPECKS,
];

fn pediu_o_traco(a: &[EditorAction]) -> bool {
    a.iter().any(|a| {
        matches!(
            a,
            EditorAction::InspectorPhysicsEdit {
                edit: E::Outline(
                    ph2d_editor_core::OutlineEdit::Detail(_)
                        | ph2d_editor_core::OutlineEdit::AlphaCutoff(_)
                        | ph2d_editor_core::OutlineEdit::Specks(_)
                ),
                ..
            }
        )
    })
}

/// **Outline à mão (`custom`) fora de «Edit shape on canvas»: os três TRAVAM** — sem alvo, sem despacho, sem
/// losango clicável, e o bloco dá a dica. Dentro da edição destravam (e o retraço é da cola, um passo de undo).
/// Sem `custom` os três são livres sempre. *Mutações: o pintor sem o `travado` ⇒ o alvo volta, vermelho; o
/// handler sem a trava ⇒ a escrita despacha, vermelho; a marca sem `parado` ⇒ o losango clica, vermelho.*
#[test]
fn o_traco_trava_fora_da_edicao_depois_dos_pontos_a_mao() {
    for custom in [false, true] {
        for editando in [false, true] {
            let travado = custom && !editando;
            let caso = format!("custom={custom} editando={editando}");
            set_key_marks(TRES.iter().map(|&id| (id, KeyMark::Livre)).collect());
            let (mut host, mut st) = cena(contorno(editando, custom), [1.0, 1.0]);
            let r = pinta(&mut host, &mut st);
            set_key_marks(Default::default());
            for id in TRES {
                assert_eq!(
                    achado(&r, id).is_some(),
                    !travado,
                    "{caso}: o alvo de {id:?}"
                );
                assert_eq!(
                    achado(&r, key_mark_id(id)).is_some(),
                    !travado,
                    "{caso}: o losango de {id:?}"
                );
            }
            assert_eq!(
                achado(&r, INSP_PHYS_OUTLINE_LOCKED).is_some(),
                travado,
                "{caso}: o bloco travado com a dica"
            );
            for (id, v) in [
                (INSP_PHYS_OUTLINE_DETAIL, 6.0),
                (INSP_PHYS_OUTLINE_ALPHA, 0.25),
                (INSP_PHYS_OUTLINE_SPECKS, 4.0),
            ] {
                let _ = host.drained_actions();
                host.set_number_value(id, v);
                let _ = host.apply_panel_event::<ph2d_panel_inspector::InspectorPanel>(
                    &mut st,
                    WidgetEvent::ValueChanged(id),
                );
                let a = host.drained_actions();
                assert_eq!(pediu_o_traco(&a), !travado, "{caso} {id:?}: {a:?}");
            }
            sai();
        }
    }
    let dica = ph2d_i18n::tr("card.insp.phys.outline_locked_tip");
    assert_eq!(
        dica,
        "Locked because you moved points by hand. Turn on \u{201c}Edit shape on canvas\u{201d} to change it."
    );
    let (host, _) = cena(contorno(false, true), [1.0, 1.0]);
    assert_eq!(
        host.store().tooltip_for(INSP_PHYS_OUTLINE_LOCKED),
        Some(dica)
    );
    sai();
}

/// **A forma solta não oferece «Remove Shape»** (dono 09/11): nem nos cartões, nem na face de referência, nem
/// no handler (o id vive no store), nem no recibo. A peça com dono guarda o dela. *Mutação: `oferece_remove`
/// sem exigir o dono ⇒ o botão volta à solta, vermelho.*
#[test]
fn a_forma_solta_nao_oferece_remove_shape_e_a_peca_guarda_o_dela() {
    let solta = InspectorPhysicsInfo {
        part_owner: String::new(),
        ..as_part()
    };
    let (mut host, mut st) = cena(solta.clone(), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_PHYS_ADD).is_some(),
        "o cartão da solta pintou"
    );
    assert!(
        achado(&r, INSP_PHYS_REMOVE).is_none(),
        "a solta sem «Remove Shape»"
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
        "o id do store não despacha na solta: {a:?}"
    );
    ph2d_panel_inspector::set_current_inspector_physics(None);
    assert!(
        achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVED_UNDO).is_none(),
        "sem recibo «Shape removed»"
    );
    sai();
    // A face de referência (`PH2D_CARDS=0`).
    let mut host = crate::host_em_metros();
    let mut st = ph2d_panel_inspector::InspectorState::default();
    ph2d_panel_inspector::set_current_inspector_physics(Some(solta));
    let r = host.paint::<ph2d_panel_inspector::InspectorPanel>(
        &mut st,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 320.0, 2400.0),
    );
    assert!(
        achado(&r, INSP_PHYS_ADD).is_some(),
        "a face de referência pintou"
    );
    assert!(achado(&r, INSP_PHYS_REMOVE).is_none(), "nem na referência");
    sai();
    // Controlo: a peça com dono guarda «Remove Shape».
    let (mut host, mut st) = cena(as_part(), [1.0, 1.0]);
    assert!(achado(&pinta(&mut host, &mut st), INSP_PHYS_REMOVE).is_some());
    sai();
}
