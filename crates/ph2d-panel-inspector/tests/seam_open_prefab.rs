//! **Varredura de SEAM do botão que ABRE A RECEITA** — o cartão de instância, 2026-09-07.
//!
//! # O defeito que ele fecha
//!
//! Este cartão é a **única** superfície que diz de que receita a cópia nasceu — *Instance of
//! «Car»* — e o nome era **texto**: o app nomeava um sítio e não dava como lá chegar. É o mesmo
//! defeito das três recusas de *«edite no prefab»*, uma superfície adiante.
//!
//! O clique passa pelo `click_at` REAL, e não por um `WidgetEvent` sintético: um evento fabricado
//! pula a checagem de focabilidade do store, e um botão fora do `populate` fica pintado,
//! hit-registrado e **morto sob o ponteiro**, com um teste verde ao lado.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::{InspectorInstanceInfo, InspectorNameInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_instance, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x0BEE_0001;
const ROOT: u64 = 0x0BEE_0002;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// Uma cópia limpa — o cartão mais simples que existe, e o botão tem de estar lá.
fn info() -> InspectorInstanceInfo {
    InspectorInstanceInfo {
        entity_bits: ENTITY,
        master_name: "Car".into(),
        overridden: Vec::new(),
        orphan_rows: Vec::new(),
        root_bits: ROOT,
        is_variant: false,
        apply_levels: Vec::new(),
        apply_levels_beyond: 0,
        removed_rows: Vec::new(),
        added_rows: Vec::new(),
    }
}

/// ⭐⭐⭐ **O botão está PINTADO, REGISTADO, e o clique chega ao barramento com a raiz da cópia.**
///
/// ⚠️ **`root_bits`, e não `entity_bits`:** o vínculo mora na RAIZ da cópia, e é dela que o verbo
/// geral resolve a receita. Mandar a peça clicada faria o verbo resolver na mesma — mas por uma
/// travessia a mais, e o dia em que a peça fosse de uma cópia aninhada as duas respostas
/// divergiriam.
///
/// **Mutação que deve sangrar:** apagar o `hit_index.register` do pintor, o `button(...)` do
/// `populate_instance`, ou a linha do `open_prefab_click` na tabela `SINGLE_ID_CLICKS`.
#[test]
fn the_card_opens_the_prefab_it_names() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Car (1)".into(),
    }));
    set_current_inspector_instance(Some(info()));

    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_INSTANCE_OPEN_PREFAB)
        .map(|(_, r)| *r)
        .expect("o botao de abrir a receita nunca foi pintado nem registado");
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        !events.is_empty(),
        "carregar no botao nao produziu evento — ele esta' morto sob o ponteiro"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let sent = host.drained_actions();

    set_current_inspector_instance(None);
    set_current_inspector_name(None);
    assert!(
        sent.iter().any(|a| matches!(
            a,
            EditorAction::InspectorOpenPrefab { root_bits } if *root_bits == ROOT
        )),
        "o clique nao chegou ao barramento com a raiz da copia: {sent:?}"
    );
}
