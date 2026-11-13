//! **Joint Editor W2 — as costuras novas do Inspector**: a linha Body A/B inteira recebe a queda, o
//! bloco «Joints» do corpo seleciona a junta, e as caixas «Solve as one chain» / «Same as …».
//!
//! ⚠️ O TEXTO pintado não é observável pelo `MockPanelHost` (só ids e rects): os rótulos da Wheel e o
//! título do cartão são gateados pelas fns de rótulo, em `sections/joint_assembly_rows.rs`.

use ph2d_editor_core::action_bus::{EditorAction, SelectModifier};
use ph2d_editor_core::screens::hero::{InspectorBodyJoint, InspectorJointInfo, JointFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids::{
    INSP_JOINT_BODY_ROW_A, INSP_JOINT_BODY_ROW_B, INSP_JOINT_LINKED, INSP_JOINT_ONE_CHAIN,
    INSP_JOINT_PICK_A, INSP_JOINT_PICK_B, insp_body_joint_row,
};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, joint_body_slot_pick, set_current_inspector_body_joints,
    set_current_inspector_joint, set_current_inspector_physics,
};

use crate::seam_joint::{ENTITY, joint, painted_ids};

/// O clique REAL numa caixa (o evento dela é `Toggled`, não `Click`), e as acções que saíram.
fn click_box(info: InspectorJointInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    let r = rect_of(&host.paint::<InspectorPanel>(&mut state, VIEWPORT), id);
    let _ = host.drained_actions();
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    set_current_inspector_joint(None);
    host.drained_actions()
}

const KIND_PIN: u8 = 0;
const KIND_ROPE: u8 = 2;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn rect_of(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Rect {
    rects
        .iter()
        .rev()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} was not painted"))
}

/// ⭐ **Um ponto no NOME de Body A/B (fora do conta-gotas) é o campo da queda**, e o ícone continua a
/// ganhar o clique. **Mutações que sangram:** registar a linha DEPOIS do ícone (o ícone perde o
/// clique) · não registar a linha · tirar `INSP_JOINT_BODY_ROW_*` do `joint_body_slot_pick`.
#[test]
fn the_whole_body_row_takes_the_drop_and_the_icon_keeps_the_click() {
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(joint(KIND_PIN)));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    for (row, pick, slot_b) in [
        (INSP_JOINT_BODY_ROW_A, INSP_JOINT_PICK_A, false),
        (INSP_JOINT_BODY_ROW_B, INSP_JOINT_PICK_B, true),
    ] {
        let (r, p) = (rect_of(&rects, row), rect_of(&rects, pick));
        // O nome: o meio do pedaço da linha à esquerda do ícone.
        let (nx, ny) = ((r.x + p.x) * 0.5, p.y + p.h * 0.5);
        let on_name = host.hit_at(nx, ny);
        assert_eq!(
            on_name,
            Some(row),
            "the name of {row:?} is not the drop field"
        );
        assert_eq!(joint_body_slot_pick(row), Some((ENTITY, slot_b)));
        let on_icon = host.hit_at(p.x + p.w * 0.5, p.y + p.h * 0.5);
        assert_eq!(on_icon, Some(pick), "the eyedropper lost its click");
    }
    // Um clique no nome não arma nada.
    let (r, p) = (
        rect_of(&rects, INSP_JOINT_BODY_ROW_A),
        rect_of(&rects, INSP_JOINT_PICK_A),
    );
    for ev in host.click_at((r.x + p.x) * 0.5, p.y + p.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    assert_eq!(host.drained_actions(), []);
    set_current_inspector_joint(None);
}

fn two_joints() -> Vec<InspectorBodyJoint> {
    vec![
        InspectorBodyJoint {
            joint_bits: 0x0101,
            label: "Rear wheel · Car 1".into(),
        },
        InspectorBodyJoint {
            joint_bits: 0x0202,
            label: "Front wheel · Car 1".into(),
        },
    ]
}

/// ⭐ **O bloco «Joints» do corpo, no clássico**: a 2.ª linha seleciona a 2.ª junta; sem juntas, nada.
/// **Mutações que sangram:** não chamar `paint_body_joints` no `physics_body` · o braço do clique
/// no `event_physics` · não registar as linhas no store (clique morto).
#[test]
fn the_body_lists_its_joints_and_a_row_selects_one() {
    set_current_inspector_physics(Some(crate::seam_physics::with_body()));
    set_current_inspector_body_joints(two_joints());
    let mut host = crate::host_em_metros();
    let mut state = InspectorState::default();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = rect_of(&rects, insp_body_joint_row(0));
    let r = rect_of(&rects, insp_body_joint_row(1));
    let _ = host.drained_actions();
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_body_joints(Vec::new());
    let empty = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_physics(None);
    assert!(
        out.contains(&EditorAction::SelectSprite {
            entity_bits: 0x0202,
            modifier: SelectModifier::Replace,
        }),
        "{out:?}"
    );
    assert!(
        !empty.iter().any(|(n, _)| *n == insp_body_joint_row(0)),
        "an empty list painted a row"
    );
}

/// ⭐ **O mesmo bloco no CARTÃO** (o smoke do dono corre em cartões): dentro da dobra Joint.
#[test]
fn the_body_card_lists_its_joints_and_a_row_selects_one() {
    let (mut host, mut st) =
        crate::o_cartao_da_fisica::cena(crate::seam_physics::with_body(), [1.0, 1.0]);
    set_current_inspector_body_joints(two_joints());
    let r = crate::o_cartao_da_fisica::pinta(&mut host, &mut st);
    let row = crate::o_cartao_da_fisica::achado(&r, insp_body_joint_row(1)).expect("card row 2");
    let out = crate::o_cartao_da_fisica::clica(&mut host, &mut st, row);
    set_current_inspector_body_joints(Vec::new());
    let empty = crate::o_cartao_da_fisica::pinta(&mut host, &mut st);
    crate::o_cartao_da_fisica::sai();
    assert!(
        out.contains(&EditorAction::SelectSprite {
            entity_bits: 0x0202,
            modifier: SelectModifier::Replace,
        }),
        "{out:?}"
    );
    assert!(crate::o_cartao_da_fisica::achado(&empty, insp_body_joint_row(0)).is_none());
}

fn pin_free() -> InspectorJointInfo {
    InspectorJointInfo {
        motor_enabled: false,
        break_enabled: false,
        ..joint(KIND_PIN)
    }
}

/// ⭐ **«Solve as one chain»**: só num Pin sem motor e não quebrável, e o clique pede `OneChain(true)`.
/// **Mutações que sangram:** o braço `INSP_JOINT_ONE_CHAIN` do `event_joint` · `offers_one_chain`
/// a aceitar motor ou outro tipo.
#[test]
fn the_one_chain_box_is_offered_only_on_a_free_pin_and_writes_true() {
    let out = click_box(pin_free(), INSP_JOINT_ONE_CHAIN);
    assert_eq!(
        out,
        [EditorAction::InspectorJointEdit {
            entity_bits: ENTITY,
            edit: JointFieldEdit::OneChain(true),
        }]
    );
    let motor_on = InspectorJointInfo {
        motor_enabled: true,
        ..pin_free()
    };
    let rope = InspectorJointInfo {
        motor_enabled: false,
        break_enabled: false,
        ..joint(KIND_ROPE)
    };
    for (what, info) in [("pin with motor", motor_on), ("rope", rope)] {
        assert!(
            !painted_ids(info).contains(&INSP_JOINT_ONE_CHAIN),
            "the one-chain box painted on a {what}"
        );
    }
}

/// ⭐ **«Same as Rear wheel»** ligada: o clique desliga (`Linked(false)`); sem par, a caixa não existe.
/// **Mutações que sangram:** o braço `INSP_JOINT_LINKED` · escrever o mesmo valor · pintar sem par.
#[test]
fn the_same_as_box_writes_the_other_side_and_hides_without_a_pair() {
    let linked = InspectorJointInfo {
        same_as: "Rear wheel".into(),
        linked: true,
        ..joint(KIND_PIN)
    };
    let out = click_box(linked, INSP_JOINT_LINKED);
    assert_eq!(
        out,
        [EditorAction::InspectorJointEdit {
            entity_bits: ENTITY,
            edit: JointFieldEdit::Linked(false),
        }]
    );
    assert!(!painted_ids(joint(KIND_PIN)).contains(&INSP_JOINT_LINKED));
}
