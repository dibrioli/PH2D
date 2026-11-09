//! Smoke 09/11 (forma neta «não arrasta»): a cadeia inteira fora da shell — as portas do Add (`pendura_forma`)
//! → retrato (`build_hierarchy_snapshot`) → painel em cartões pelo ponteiro real → o pedido `Reparent` com o
//! corpo como pai. A lei que o executa (`ph2d_app_components::hierarchy_reparent`) tem os testes dela: um
//! painel não pode depender de uma família (`architecture_no_dependency_climbs_a_layer`).

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_ecs::scene::{HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot};
use ph2d_ecs::{ChildOf, Entity, RootOrder, SimWorld, Transform};
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn no(e: u64) -> NodeId {
    NodeId(e)
}

/// O retrato no painel; devolve (entidade, profundidade) na ordem das linhas.
fn publica(sim: &mut SimWorld, host: &mut MockPanelHost) -> Vec<(u64, u8)> {
    let (mut estado, mut snap) = (
        HierarchyWalkState::new(sim.world_mut()),
        HierarchySnapshot::new(),
    );
    build_hierarchy_snapshot(sim.world(), &mut estado, &mut Vec::new(), &mut snap);
    let linhas: Vec<(u64, u8)> = snap.entries.iter().map(|e| (e.entity, e.depth)).collect();
    let mapa: BTreeMap<NodeId, HierarchyEntity> = snap
        .entries
        .iter()
        .map(|e| {
            let h = HierarchyEntity {
                name: format!("{:x}", e.entity),
                icon: IconId::Sprite,
                indent: e.depth,
                badge: None,
                swatch: None,
                visible: true,
                selected: false,
                hovered: false,
                muted: false,
                locked: false,
                group_locked: false,
                role: RowRole::Object,
                components: Vec::new(),
                content: Vec::new(),
                has: Default::default(),
                entity: Some(e.entity),
            };
            (no(e.entity), h)
        })
        .collect();
    let ordem: Vec<NodeId> = linhas.iter().map(|&(e, _)| no(e)).collect();
    ph2d_panel_hierarchy::sync_from_hierarchy(host.store_mut(), &ordem, mapa);
    linhas
}

fn ponteiro(
    host: &mut MockPanelHost,
    kind: PointerKind,
    (x, y): (f32, f32),
    t: u128,
) -> Vec<ph2d_editor_core::interaction::WidgetEvent> {
    host.dispatch_pointer_event(PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    })
}

/// A forma NETA (pendurada pelo Add na forma do corpo) arrasta-se para dentro do corpo e lá fica no quadro
/// seguinte. *Mutação: o drop do painel a recusar a linha ⇒ nenhum Reparent, vermelho.*
#[test]
fn a_forma_neta_arrasta_para_dentro_do_corpo_e_la_fica() {
    ph2d_panel_hierarchy::cards::force_cards_on_this_thread(Some(true));
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = w
        .spawn((
            ph2d_physics_ecs::RigidBody::default(),
            Transform::IDENTITY,
            RootOrder(0),
        ))
        .id();
    let s1 = ph2d_physics_ecs::nasce_com_forma(
        w,
        corpo,
        ph2d_physics_ecs::ColliderShape::Ball { radius: 0.5 },
    );
    let raiz = w.spawn((Transform::IDENTITY, RootOrder(1))).id();
    let neta = ph2d_physics_ecs::pendura_forma(w, raiz, Some(s1));
    assert_eq!(
        w.get::<ChildOf>(neta).map(ChildOf::parent),
        Some(s1),
        "controlo: a neta nasce na forma"
    );

    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    let mut st = HierarchyState::default();
    let antes = publica(&mut sim, &mut host);
    assert!(
        antes.contains(&(neta.to_bits(), 2)),
        "a neta na profundidade 2: {antes:?}"
    );
    let mut rects = Vec::new();
    for _ in 0..60 {
        rects = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint::<HierarchyPanel>(&mut st, VIEW),
        );
    }
    let rect = |e: Entity| {
        rects
            .iter()
            .rev()
            .find(|(n, r)| *n == no(e.to_bits()) && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
            .expect("linha pintada")
    };
    let (rn, rc) = (rect(neta), rect(corpo));
    let de = (rn.x + rn.w * 0.5, rn.y + rn.h * 0.5);
    let para = (de.0, rc.y + rc.h * 0.5);
    let _ = ponteiro(&mut host, PointerKind::Down, de, 1_000_000_000);
    let _ = ponteiro(
        &mut host,
        PointerKind::Move,
        (de.0, (de.1 + para.1) / 2.0),
        1_050_000_000,
    );
    let _ = ponteiro(&mut host, PointerKind::Move, para, 1_100_000_000);
    for e in ponteiro(&mut host, PointerKind::Up, para, 1_200_000_000) {
        let _ = host.apply_panel_event::<HierarchyPanel>(&mut st, e);
    }
    let pedidos: Vec<_> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::Hierarchy(HierRequest::Reparent(i)) => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(pedidos.len(), 1, "um Reparent pelo ponteiro");
    let i = &pedidos[0];
    let ent = |n: Option<NodeId>| n.map(|n| Entity::from_bits(n.0));
    assert_eq!(ent(Some(i.dragged)), Some(neta));
    assert_eq!(ent(i.new_parent), Some(corpo), "o pedido larga a neta no corpo");
}
