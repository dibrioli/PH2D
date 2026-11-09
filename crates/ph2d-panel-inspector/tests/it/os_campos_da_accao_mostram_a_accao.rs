//! Smoke do dono 09/11 (cena 121, passo 5): a lista dizia `coin_taken → Destroy · Coin A` e a caixa
//! **On** por baixo estava vazia — a secção Signal Actions não tinha semente, e os campos de texto
//! mostravam o `String::new()` do `populate_action`. A irmã `sync_action_trigger` documenta o mesmo
//! defeito, medido em 17/09.

use ph2d_editor_core::screens::hero::{InspectorActionInfo, InspectorActionRow};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_action};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};

fn linha(on: &str, target: &str, arg: &str) -> InspectorActionRow {
    InspectorActionRow {
        item: Default::default(),
        on: on.into(),
        target: target.into(),
        verb_tag: 0,
        arg: arg.into(),
        uses_arg: true,
        arg_hint: Default::default(),
        arg_options: None,
        uses_target: true,
        target_mode: 0,
        from_tag: 0,
        target_tag: None,
        target_tag_path: String::new(),
    }
}

fn duas_accoes() -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: 0xAC_7104,
        rows: vec![
            linha("coin_taken", "Coin A", "1"),
            linha("door_open", "Door", "2"),
        ],
        verb_labels: vec!["Destroy".into(), "Show".into()],
        selected_count: 1,
    }
}

/// A acção aberta mostra os SEUS valores, e trocar de linha troca-os.
///
/// **Mutação que deve sangrar:** tirar a chamada `sync_action::sync` do `sync_sections`.
#[test]
fn os_campos_da_accao_aberta_mostram_os_valores_dela() {
    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_action(Some(duas_accoes()));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert_eq!(h.store().text(ids::INSP_ACTION_ON), Some("coin_taken"));
    assert_eq!(h.store().text(ids::INSP_ACTION_TARGET), Some("Coin A"));
    assert_eq!(h.store().text(ids::INSP_ACTION_ARG), Some("1"));
    st.action_selected = 1;
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert_eq!(h.store().text(ids::INSP_ACTION_ON), Some("door_open"));
    assert_eq!(h.store().text(ids::INSP_ACTION_TARGET), Some("Door"));
    set_current_inspector_action(None);
}
