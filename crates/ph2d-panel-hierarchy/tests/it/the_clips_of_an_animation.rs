//! ⭐⭐ **Os clips da Animação na Hierarquia** (doc ObjectsAndComponents 01 §6), com CLIQUES REAIS:
//! o grupo *Clips (n)* sob o objecto e sob a Cena, o clique num clip abre a Timeline nele, e o 🎞
//! de um objecto com clips leva à Timeline (o cartão da Animação É o painel).

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest, SelectModifier};
use ph2d_editor_core::hier_rows::{ContentGroup, ContentItem, ContentOpens, RowRole};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::PANEL_TIMELINE;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::ids::{HIER_SCENE_ROW, hier_component_icon, hier_content_row};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const HEROI: NodeId = NodeId(100_000);

fn clips(names: &[(u64, &str)]) -> ContentGroup {
    ContentGroup {
        label_key: "hier_rows.content.clips",
        icon: IconId::Film,
        item_icon: IconId::Layer,
        opens: ContentOpens::Panel(PANEL_TIMELINE),
        items: names
            .iter()
            .map(|(key, name)| ContentItem {
                key: *key,
                name: (*name).into(),
                depth: 0,
                active: false,
            })
            .collect(),
    }
}

fn host() -> (MockPanelHost, HierarchyState) {
    let mut heroi = ph2d_panel_hierarchy::scene_entry();
    heroi.name = "Herói".into();
    heroi.role = RowRole::Object;
    heroi.components = vec![ComponentCategory::Animation];
    heroi.content = vec![clips(&[(1, "andar"), (2, "pular")])];
    let mut cena = ph2d_panel_hierarchy::scene_entry();
    cena.content = vec![clips(&[(0, "Main")])];
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    host.set_panel_visible(PANEL_TIMELINE, false);
    ph2d_panel_hierarchy::sync_from_hierarchy(
        host.store_mut(),
        &[HEROI],
        BTreeMap::<NodeId, HierarchyEntity>::from([(HEROI, heroi), (HIER_SCENE_ROW, cena)]),
    );
    (host, HierarchyState::default())
}

fn click(host: &mut MockPanelHost, state: &mut HierarchyState, id: NodeId) {
    let painted = host.paint::<HierarchyPanel>(state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let (_, r) = painted
        .iter()
        .find(|(i, _)| *i == id)
        .copied()
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let events = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(events.contains(&WidgetEvent::Click(id)), "{id:?}: {events:?}");
    for ev in events {
        host.apply_panel_event::<HierarchyPanel>(state, ev);
    }
}

#[test]
fn um_clip_do_heroi_abre_a_timeline_nele_e_selecciona_o_heroi() {
    let (mut host, mut state) = host();
    click(&mut host, &mut state, hier_content_row(HEROI, 0, None));
    click(&mut host, &mut state, hier_content_row(HEROI, 0, Some(2)));
    let actions = host.drained_actions();
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::SelectRow {
            row: HEROI,
            modifier: SelectModifier::Replace,
        })),
        "o dono não ficou seleccionado: {actions:?}"
    );
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::OpenContentInPanel {
            owner: HEROI,
            panel: PANEL_TIMELINE,
            key: 2,
        })),
        "o clip «pular» não foi pedido à Timeline: {actions:?}"
    );
    assert!(host.panel_visible(PANEL_TIMELINE), "a Timeline não abriu");
}

#[test]
fn a_cena_mostra_a_animacao_dela_e_o_clip_nao_mexe_na_seleccao() {
    let (mut host, mut state) = host();
    let head = hier_content_row(HIER_SCENE_ROW, 0, None);
    click(&mut host, &mut state, head);
    click(&mut host, &mut state, hier_content_row(HIER_SCENE_ROW, 0, Some(0)));
    let actions = host.drained_actions();
    assert!(actions.contains(&EditorAction::Hierarchy(HierRequest::OpenContentInPanel {
        owner: HIER_SCENE_ROW,
        panel: PANEL_TIMELINE,
        key: 0,
    })));
    assert!(
        !actions.iter().any(|a| matches!(a, EditorAction::Hierarchy(HierRequest::SelectRow { .. }))),
        "a Cena não é linha de objecto: {actions:?}"
    );
}

#[test]
fn o_icone_da_animacao_de_um_objecto_leva_a_timeline() {
    let (mut host, mut state) = host();
    click(&mut host, &mut state, hier_component_icon(HEROI, ComponentCategory::Animation));
    assert!(host.panel_visible(PANEL_TIMELINE), "o 🎞 do Herói não abriu a Timeline");
    assert!(
        host.store().inspector_reveal().is_none(),
        "o cartão da Animação é a Timeline, não um cartão do Inspector"
    );
}
