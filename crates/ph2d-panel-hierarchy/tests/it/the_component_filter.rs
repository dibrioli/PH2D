//! ⭐⭐ **O filtro ▾ da Hierarquia por componente** (doc ObjectsAndComponents 01 §7.5 e §13, F6), com
//! CLIQUES REAIS: o ▾ ao lado da busca lista só os componentes presentes, com contagem; escolher um
//! mostra só quem o tem (e os pais de quem o tem); «All» devolve a árvore inteira.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentSet;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::ids::{HIER_FILTER_DD, hier_filter_opt};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const HEROI: NodeId = NodeId(100_001);
const ESPADA: NodeId = NodeId(100_002);
const MOEDA: NodeId = NodeId(100_003);
const GRUPO: NodeId = NodeId(100_004);
const INIMIGO: NodeId = NodeId(100_005);
const ORDEM: [NodeId; 5] = [HEROI, ESPADA, MOEDA, GRUPO, INIMIGO];

fn bit(canonical: &str) -> u8 {
    ComponentSet::authored()
        .find(|(_, d)| d.canonical_name == canonical)
        .map(|(b, _)| b)
        .expect("componente do catálogo")
}

fn linha(nome: &str, indent: u8, tem: &[u8]) -> HierarchyEntity {
    let mut e = ph2d_panel_hierarchy::scene_entry();
    e.name = nome.into();
    e.role = RowRole::Object;
    e.indent = indent;
    e.components = Vec::new();
    e.content = Vec::new();
    for b in tem {
        e.has.insert(*b);
    }
    e
}

fn host() -> (MockPanelHost, HierarchyState) {
    let vida = bit("ph2d::physics::Health");
    let som = bit("ph2d::ecs::AudioSource2D");
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    ph2d_panel_hierarchy::sync_from_hierarchy(
        host.store_mut(),
        &ORDEM,
        BTreeMap::from([
            (HEROI, linha("Herói", 0, &[vida, som])),
            (ESPADA, linha("Espada", 1, &[])),
            (MOEDA, linha("Moeda", 0, &[som])),
            (GRUPO, linha("Grupo", 0, &[])),
            (INIMIGO, linha("Inimigo", 1, &[vida])),
        ]),
    );
    (host, HierarchyState::default())
}

fn pintadas(host: &mut MockPanelHost, state: &mut HierarchyState) -> Vec<NodeId> {
    let painted = host.paint::<HierarchyPanel>(state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    ORDEM
        .into_iter()
        .filter(|id| painted.iter().any(|(i, _)| i == id))
        .collect()
}

/// Abre o ▾ — o clique no chip chega como FOCO e é o próprio dropdown que se abre.
fn abre(host: &mut MockPanelHost, state: &mut HierarchyState) {
    let painted = host.paint::<HierarchyPanel>(state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let (_, r) = painted
        .iter()
        .find(|(i, _)| *i == HIER_FILTER_DD)
        .copied()
        .expect("o ▾ do filtro não foi pintado");
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<HierarchyPanel>(state, ev);
    }
    assert!(
        matches!(
            host.store().get(HIER_FILTER_DD),
            Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
        ),
        "o ▾ não abriu"
    );
}

fn click(host: &mut MockPanelHost, state: &mut HierarchyState, id: NodeId) {
    let painted = host.paint::<HierarchyPanel>(state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let (_, r) = painted
        .iter()
        .find(|(i, _)| *i == id)
        .copied()
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let events = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        events.contains(&WidgetEvent::Click(id)),
        "{id:?}: {events:?}"
    );
    for ev in events {
        host.apply_panel_event::<HierarchyPanel>(state, ev);
    }
}

#[test]
fn escolher_health_mostra_so_quem_tem_vida_e_os_pais_deles() {
    let (mut host, mut state) = host();
    assert_eq!(pintadas(&mut host, &mut state), ORDEM, "sem filtro, todos");
    abre(&mut host, &mut state);
    // A lista: «All», depois as famílias pela ordem da paleta — Lógica (Health) antes de Áudio.
    click(&mut host, &mut state, hier_filter_opt(1));
    assert_eq!(
        state.component_filter,
        Some(bit("ph2d::physics::Health")),
        "a 1.ª linha depois de «All» é a Health"
    );
    assert_eq!(
        pintadas(&mut host, &mut state),
        vec![HEROI, GRUPO, INIMIGO],
        "só quem tem Health — e o Grupo, pai do Inimigo"
    );
    abre(&mut host, &mut state);
    click(&mut host, &mut state, hier_filter_opt(0));
    assert_eq!(state.component_filter, None);
    assert_eq!(pintadas(&mut host, &mut state), ORDEM, "«All» devolve todos");
}

#[test]
fn o_filtro_soma_se_a_busca_por_nome() {
    let (mut host, mut state) = host();
    state.component_filter = Some(bit("ph2d::ecs::AudioSource2D"));
    assert_eq!(pintadas(&mut host, &mut state), vec![HEROI, MOEDA]);
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) = host
        .store_mut()
        .get_mut(ph2d_panel_hierarchy::ids::HIER_SEARCH)
    {
        *text = String::from("moe");
    }
    assert_eq!(
        pintadas(&mut host, &mut state),
        vec![MOEDA],
        "as duas têm de casar"
    );
}

/// A lista diz quantos objectos têm cada componente — e só os presentes.
#[test]
fn a_lista_so_tem_os_presentes_com_a_contagem() {
    let (mut host, mut state) = host();
    abre(&mut host, &mut state);
    let painted = host.paint::<HierarchyPanel>(&mut state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let rows: Vec<usize> = (0..8)
        .filter(|i| painted.iter().any(|(id, _)| *id == hier_filter_opt(*i)))
        .collect();
    assert_eq!(rows, vec![0, 1, 2], "All + Health + Audio Source 2D");
}
