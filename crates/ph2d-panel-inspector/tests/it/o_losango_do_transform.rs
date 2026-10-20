//! ⭐ **O losango da linha Position do Inspector** (plano 35 §5) — pintado na coluna do decorator,
//! registado, e um clique REAL nele é um pedido de chave com os dois campos da linha (X e Y), sem
//! mexer no número. A shell publica o mapa (`ph2d_app_components::marcas_inspector`).

use std::collections::BTreeMap;

use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::widget::key_mark::{
    KeyMark, key_mark_id, set_key_marks, take_key_mark_click,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids::{INSP_TRANSFORM_POS_X, INSP_TRANSFORM_POS_Y, INSP_TRANSFORM_ROT};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_transform};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn ev(kind: PointerKind, r: Rect, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// (Mutação: o mapa sem a linha ⇒ sem alvo; o braço do losango fora do `dispatch_down` ⇒ o clique
/// não chega.)
#[test]
fn clicar_no_losango_da_posicao_pede_a_chave_de_x_e_y() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: 0x7E57_0042,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    set_key_marks(BTreeMap::from([
        (INSP_TRANSFORM_POS_X, KeyMark::Chave),
        (INSP_TRANSFORM_POS_Y, KeyMark::Chave),
    ]));
    let _ = take_key_mark_click();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    host.settle_section_folds();
    let r = host
        .painted_rect::<InspectorPanel>(&mut st, VIEWPORT, key_mark_id(INSP_TRANSFORM_POS_X))
        .expect("o losango da posição foi pintado mas não registado");
    assert!(
        host.painted_rect::<InspectorPanel>(&mut st, VIEWPORT, key_mark_id(INSP_TRANSFORM_ROT))
            .is_none(),
        "a rotação não foi declarada pela shell: sem alvo"
    );
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, r, 1_010_000_000));
    let c = take_key_mark_click().expect("o clique no losango não chegou à shell");
    assert_eq!(c.fields, vec![INSP_TRANSFORM_POS_X, INSP_TRANSFORM_POS_Y]);
    set_key_marks(BTreeMap::new());
    set_current_inspector_transform(None);
}
