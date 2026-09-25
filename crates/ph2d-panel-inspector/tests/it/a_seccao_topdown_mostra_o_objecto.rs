//! ⭐⭐⭐ **A secção TOP-DOWN PLAYER mostra os números DO OBJECTO** (plano 28, W5).
//!
//! ⛔ **A semente dela não existia desde o TOP-20 #13** e foi achada ao acrescentar-lhe a linha do
//! empurrão (*Push Recovery*): os oito números eram os de PARTIDA do `populate_topdown`. Irmã do
//! [`super::a_seccao_vida_esta_viva`] pelo mesmo motivo — *a semente é a única metade de uma secção
//! cujo sujeito é o WIDGET*, e os gates da lei e do dreno entram abaixo do `WidgetStore`.

use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
    TopDownFieldEdit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_topdown};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 3200.0,
};

/// ⭐ Um mover que **NÃO** é o de fábrica em nenhum número — *um corpus no neutro de um knob não
/// testa esse knob*, e aqui o neutro é exactamente o que o defeito mostrava.
fn mover() -> InspectorTopDownInfo {
    InspectorTopDownInfo {
        entity_bits: 0x00CD_5678,
        speed: 9.5,
        acceleration: 33.0,
        deceleration: 41.5,
        directions: InspectorMoveDirections::default(),
        viewpoint: InspectorViewpoint::default(),
        viewpoint_angle_deg: 40.0,
        facing: InspectorFacing::default(),
        turn_speed_deg: 250.0,
        min_slide_angle_deg: 22.0,
        max_slides: 6,
        default_controls: true,
        knockback_recovery: 12.5,
        body_is_kinematic: true,
        has_body: true,
        conflicts_with_platformer: false,
        clock_playing: true,
        selected_count: 1,
    }
}

/// Os oito números, com o valor da fixtura.
const NUMEROS: [(ph2d_a11y::NodeId, f64); 8] = [
    (ids::INSP_TD_SPEED, 9.5),
    (ids::INSP_TD_ACCEL, 33.0),
    (ids::INSP_TD_DECEL, 41.5),
    (ids::INSP_TD_KNOCKBACK_RECOVERY, 12.5),
    (ids::INSP_TD_VIEW_ANGLE, 40.0),
    (ids::INSP_TD_TURN_SPEED, 250.0),
    (ids::INSP_TD_MIN_SLIDE, 22.0),
    (ids::INSP_TD_MAX_SLIDES, 6.0),
];

/// ⭐⭐⭐ **OS CAMPOS MOSTRAM O QUE O OBJECTO TEM, e não os valores de fábrica.**
///
/// **Mutação que deve sangrar:** tirar a chamada do `sync_topdown` do `sync_sections`.
#[test]
fn os_campos_do_mover_mostram_o_que_o_objecto_tem() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_topdown(Some(mover()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, esperado) in NUMEROS {
        let lido = h
            .store()
            .number_value(id)
            .unwrap_or_else(|| panic!("o campo {id:?} nem sequer esta' registado"));
        assert!(
            (lido - esperado).abs() < 1.0e-5,
            "o campo {id:?} mostra {lido} e o objecto tem {esperado} — o painel mostra os valores \
             de FABRICA do `populate_topdown`"
        );
    }
    set_current_inspector_topdown(None);
}

/// ⭐ **A linha do empurrão é PINTADA e está VIVA sob o dedo** — sem o registo ela pinta e morre.
///
/// **Mutação que deve sangrar:** tirar o `INSP_TD_KNOCKBACK_RECOVERY` do `populate_topdown`.
#[test]
fn a_linha_do_empurrao_esta_viva_sob_o_dedo() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_topdown(Some(mover()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_TD_KNOCKBACK_RECOVERY)
        .map(|(_, r)| *r)
        .expect("a linha do empurrão não foi pintada");
    let _ = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        h.store().focus_id(),
        Some(ids::INSP_TD_KNOCKBACK_RECOVERY),
        "a linha do empurrão está MORTA SOB O DEDO"
    );
    set_current_inspector_topdown(None);
}

/// ⭐ **E o número do empurrão escreve NO CAMPO DELE** — viva sob o dedo não basta: um braço do
/// `event_topdown` que mandasse o valor para a travagem deixava a linha focável e o objecto errado.
///
/// **Mutação que deve sangrar:** o braço `INSP_TD_KNOCKBACK_RECOVERY` pedir `Deceleration`.
#[test]
fn o_numero_do_empurrao_pede_a_edicao_dele() {
    use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
    use ph2d_editor_core::interaction::WidgetEvent;
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_topdown(Some(mover()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_number_value(ids::INSP_TD_KNOCKBACK_RECOVERY, 7.5);
    let _ = h.apply_panel_event::<InspectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(ids::INSP_TD_KNOCKBACK_RECOVERY),
    );
    let edits: Vec<_> = h
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::TopDown(e),
                ..
            } => Some(e),
            _ => None,
        })
        .collect();
    set_current_inspector_topdown(None);
    assert_eq!(edits, vec![TopDownFieldEdit::KnockbackRecovery(7.5)]);
}
