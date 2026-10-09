//! ⭐⭐ **As linhas novas da Hierarquia, CLICADAS** (doc 01 §4–§7): a raiz Cena, os ícones de
//! componente e o conteúdo. Cada teste pinta o painel de verdade, clica no rect pintado pelo
//! dispatcher real e afirma o efeito — o pedido no barramento, o pedido ao Inspector, o painel
//! aberto. *Um ícone que pinta e não responde ao rato é o defeito que estes testes existem para ver.*

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest, SelectModifier};
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, RowRole};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::ids::{HIER_SCENE_ROW, hier_component_icon, hier_content_row};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const HEROI: NodeId = NodeId(100_000);

fn heroi() -> HierarchyEntity {
    HierarchyEntity {
        name: "Herói".into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: RowRole::Object,
        components: vec![ComponentCategory::Physics, ComponentCategory::Audio],
        content: vec![ContentGroup {
            label_key: "hier_rows.content.layers",
            icon: IconId::Layers,
            item_icon: IconId::Layer,
            opens: ph2d_editor_core::hier_rows::ContentOpens::Mode(ObjectMode::Paint),
            items: vec![
                ContentItem {
                    key: 11,
                    name: "Tinta".into(),
                    depth: 0,
                    active: false,
                },
                ContentItem {
                    key: 12,
                    name: "Fundo".into(),
                    depth: 0,
                    active: true,
                },
            ],
        }],
        has: Default::default(),
    }
}

fn host_with_heroi() -> (MockPanelHost, HierarchyState) {
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    ph2d_panel_hierarchy::sync_from_hierarchy(
        host.store_mut(),
        &[HEROI],
        BTreeMap::from([(HEROI, heroi())]),
    );
    (host, HierarchyState::default())
}

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// Pinta, acha o rect de `id` e clica no centro dele; devolve o que o painel respondeu.
fn click(host: &mut MockPanelHost, state: &mut HierarchyState, id: NodeId) -> Vec<EventOutcome> {
    let painted = host.paint::<HierarchyPanel>(state, viewport());
    let (_, r) = painted
        .iter()
        .find(|(i, _)| *i == id)
        .copied()
        .unwrap_or_else(|| panic!("{id:?} não foi pintado — a linha não o mostra"));
    let events = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        events.contains(&WidgetEvent::Click(id)),
        "o rato sobre {id:?} não virou Click dele (registo em falta?): {events:?}"
    );
    events
        .into_iter()
        .map(|ev| host.apply_panel_event::<HierarchyPanel>(state, ev))
        .collect()
}

#[test]
fn o_icone_de_um_componente_selecciona_o_objecto_e_pede_o_cartao() {
    let (mut host, mut state) = host_with_heroi();
    let icon = hier_component_icon(HEROI, ComponentCategory::Audio);
    click(&mut host, &mut state, icon);
    let actions = host.drained_actions();
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::SelectRow {
            row: HEROI,
            modifier: SelectModifier::Replace,
        })),
        "o ícone não seleccionou o dono: {actions:?}"
    );
    let req = host
        .store()
        .inspector_reveal()
        .expect("nenhum pedido ao Inspector");
    assert_eq!(req.category, ComponentCategory::Audio);
    assert_eq!(req.label, "Herói");
    assert!(host.panel_visible("inspector"));
}

#[test]
fn o_icone_do_mundo_fisico_na_cena_abre_o_painel_do_mundo() {
    let (mut host, mut state) = host_with_heroi();
    host.set_panel_visible("physics", false);
    let icon = hier_component_icon(HIER_SCENE_ROW, ComponentCategory::Physics);
    click(&mut host, &mut state, icon);
    assert!(
        host.panel_visible("physics"),
        "o ícone ⬢ da Cena não abriu o painel do mundo físico"
    );
    assert!(host.store().inspector_reveal().is_none());
}

#[test]
fn o_conteudo_nasce_recolhido_e_a_peca_abre_o_modo_com_ela() {
    let (mut host, mut state) = host_with_heroi();
    let head = hier_content_row(HEROI, 0, None);
    let tinta = hier_content_row(HEROI, 0, Some(11));
    let painted = host.paint::<HierarchyPanel>(&mut state, viewport());
    assert!(painted.iter().any(|(i, _)| *i == head));
    assert!(
        !painted.iter().any(|(i, _)| *i == tinta),
        "o conteúdo tem de nascer RECOLHIDO (doc 01 §6)"
    );
    click(&mut host, &mut state, head);
    click(&mut host, &mut state, tinta);
    let actions = host.drained_actions();
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::OpenContent {
            owner: HEROI,
            mode: ObjectMode::Paint,
            key: 11,
        })),
        "a camada clicada não pediu o Paint com ela: {actions:?}"
    );
}

#[test]
fn a_cena_limpa_a_seleccao_e_nao_e_uma_linha_de_objecto() {
    let (mut host, mut state) = host_with_heroi();
    let outcomes = click(&mut host, &mut state, HIER_SCENE_ROW);
    assert!(outcomes.contains(&EventOutcome::Consumed));
    assert!(
        host.drained_actions()
            .contains(&EditorAction::ClearSelection)
    );
    let store = host.store();
    assert!(store.is_hierarchy_row(HEROI));
    assert!(
        !store.is_hierarchy_row(HIER_SCENE_ROW)
            && !store.is_hierarchy_row(hier_content_row(HEROI, 0, None)),
        "a Cena e o conteúdo não se arrastam nem abrem o menu de objecto"
    );
    let out = host
        .apply_panel_event::<HierarchyPanel>(&mut state, WidgetEvent::DoubleClick(HIER_SCENE_ROW));
    assert_eq!(out, EventOutcome::Consumed);
    assert!(state.rename_target_row.is_none(), "a Cena não se renomeia");
}
