//! ⭐ **Arrastar sobre os interruptores das linhas pinta-os** (dono 01/11, o Outliner do Blender):
//! premir vira e dá a tinta; cada interruptor atravessado — noutra linha (vertical) ou noutra função
//! da mesma linha (horizontal) — fica com a tinta. Pinta o painel de verdade, arrasta pelo
//! despacho real e afirma os pedidos no barramento.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids::hier_eye_companion;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::ids::{hier_group_companion, hier_lock_companion};
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const A: NodeId = NodeId(200_001);
const B: NodeId = NodeId(200_002);
const C: NodeId = NodeId(200_003);

fn linha(name: &str, visible: bool) -> HierarchyEntity {
    HierarchyEntity {
        name: name.into(),
        icon: IconId::Sprite,
        indent: 0,
        badge: None,
        swatch: None,
        visible,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role: RowRole::Object,
        components: vec![],
        content: vec![],
        has: Default::default(),
        entity: None,
    }
}

/// Três linhas: A e C à vista, B escondida.
fn pinta() -> (MockPanelHost, HierarchyState, Vec<(NodeId, Rect)>) {
    ph2d_panel_hierarchy::cards::force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    ph2d_panel_hierarchy::sync_from_hierarchy(
        host.store_mut(),
        &[A, B, C],
        BTreeMap::from([
            (A, linha("A", true)),
            (B, linha("B", false)),
            (C, linha("C", true)),
        ]),
    );
    let mut state = HierarchyState::default();
    let painted = host.paint::<HierarchyPanel>(&mut state, Rect::new(0.0, 0.0, 1600.0, 900.0));
    (host, state, painted)
}

fn centro(painted: &[(NodeId, Rect)], id: NodeId) -> (f32, f32) {
    let r = painted
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

fn arrasta(from: NodeId, to: NodeId) -> Vec<EditorAction> {
    let (mut host, mut state, painted) = pinta();
    let (x0, y0) = centro(&painted, from);
    let (x1, y1) = centro(&painted, to);
    for ev in host.drag_at(x0, y0, x1, y1) {
        host.apply_panel_event::<HierarchyPanel>(&mut state, ev);
    }
    host.drained_actions()
}

#[test]
fn arrastar_na_coluna_do_olho_pinta_as_outras_linhas_com_o_estado_do_primeiro() {
    let got = arrasta(hier_eye_companion(A), hier_eye_companion(C));
    let vis = |row| EditorAction::Hierarchy(HierRequest::ToggleVisibility { row });
    assert_eq!(
        got.iter()
            .filter(|a| matches!(
                a,
                EditorAction::Hierarchy(HierRequest::ToggleVisibility { .. })
            ))
            .count(),
        2,
        "A vira (esconde) e C fica escondida; B já estava escondida e não vira: {got:?}"
    );
    assert!(got.contains(&vis(A)) && got.contains(&vis(C)) && !got.contains(&vis(B)));
}

#[test]
fn arrastar_na_mesma_linha_pinta_as_outras_funcoes_com_o_mesmo_estado() {
    // Cadeado (aberto) → grupo (aberto) → olho (à vista): a tinta é «ligado».
    let got = arrasta(hier_lock_companion(A), hier_eye_companion(A));
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleLock { row: A })),
        "{got:?}"
    );
    assert!(
        got.contains(&EditorAction::Hierarchy(HierRequest::ToggleGroup {
            row: A
        })),
        "{got:?}"
    );
    assert!(
        !got.contains(&EditorAction::Hierarchy(HierRequest::ToggleVisibility {
            row: A
        })),
        "o olho já estava ligado (à vista) e não vira: {got:?}"
    );
}

/// Um clique sem arrasto continua a ser um clique: vira só aquele interruptor, uma vez.
#[test]
fn premir_e_soltar_no_mesmo_sitio_vira_uma_vez() {
    let got = arrasta(hier_group_companion(B), hier_group_companion(B));
    assert_eq!(
        got,
        vec![EditorAction::Hierarchy(HierRequest::ToggleGroup { row: B })]
    );
}
