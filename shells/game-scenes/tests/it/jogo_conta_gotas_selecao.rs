//! ⭐⭐ **O conta-gotas não muda a selecção** (dono 15/11: «ao selecionar o objeto no canvas ou na hierarquia, a seleção
//! não pode mudar de objeto, deixe o dono do eyedropper»). O rato real na linha pintada da Hierarchy, pela mesma porta
//! da shell (`conta_gotas::pela_hierarquia` à volta do clique, e o objecto da linha em `conta_gotas::escolher`).

use ph2d_app_components::{conta_gotas, projectile_inspector, ray_topo};
use ph2d_component_desc::ComponentSet;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, StableId};
use ph2d_editor_core::IconId;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::projectile_edits::ProjectileFieldEdit;
use ph2d_editor_core::ray_edits::RayFieldEdit;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_hierarchy::{HierarchyPanel, HierarchyState};
use ph2d_physics_ecs::{BodyKind, RaySensor, RigidBody};
use ph2d_ui_testkit::MockPanelHost;
use std::collections::BTreeMap;

const CENA: NodeId = NodeId(0xC0D0);
static RELOGIO: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_000_000_000);

fn linha(name: &str, role: RowRole, entity: Option<Entity>) -> HierarchyEntity {
    HierarchyEntity {
        name: name.into(),
        icon: IconId::Eyedropper,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        hovered: false,
        muted: false,
        locked: false,
        group_locked: false,
        role,
        components: Vec::new(),
        content: Vec::new(),
        has: ComponentSet::default(),
        entity: entity.map(Entity::to_bits),
    }
}

/// A Hierarchy com a Cena e uma linha por objecto (`NodeId` = índice + 1).
fn hierarquia(sim: &SimWorld, objectos: &[Entity]) -> (MockPanelHost, HierarchyState) {
    ph2d_panel_hierarchy::cards::force_cards_on_this_thread(Some(false));
    let mut h = MockPanelHost::with_panel::<HierarchyPanel>();
    let mut linhas = BTreeMap::from([(CENA, linha("Scene", RowRole::Scene, None))]);
    let mut ordem = vec![CENA];
    for (i, &e) in objectos.iter().enumerate() {
        let nome = sim.world().get::<Name>(e).map(|n| n.as_str().to_owned());
        let row = NodeId(i as u64 + 1);
        linhas.insert(
            row,
            linha(&nome.unwrap_or_default(), RowRole::Object, Some(e)),
        );
        ordem.push(row);
    }
    ph2d_panel_hierarchy::sync_from_hierarchy(h.store_mut(), &ordem, linhas);
    let _ = h.drained_actions();
    (h, HierarchyState::default())
}

/// O rato sobre a linha pintada, embrulhado como na shell; devolve o objecto que o conta-gotas recebeu.
fn clica(
    sim: &SimWorld,
    h: &mut MockPanelHost,
    st: &mut HierarchyState,
    row: NodeId,
) -> Option<u64> {
    let rects = h.paint::<HierarchyPanel>(st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    let (_, r) = *rects
        .iter()
        .find(|(n, _)| *n == row)
        .expect("a linha não foi pintada");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ev = |kind, timestamp_ns| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns,
    };
    // Um segundo entre cliques: dois na mesma linha não viram duplo-clique.
    let t = u128::from(RELOGIO.fetch_add(1_000_000_000, std::sync::atomic::Ordering::Relaxed));
    conta_gotas::pela_hierarquia(conta_gotas::armado(sim.world()), || {
        let mut evs = h.dispatch_pointer_event(ev(PointerKind::Down, t));
        evs.extend(h.dispatch_pointer_event(ev(PointerKind::Up, t + 10_000_000)));
        assert!(
            evs.contains(&WidgetEvent::Click(row)),
            "o rato não virou clique: {evs:?}"
        );
        for e in evs {
            h.apply_panel_event::<HierarchyPanel>(st, e);
        }
    })
}

/// Nenhuma acção de selecção saiu do painel (`SelectRow`, `RangeSelect`, `ClearSelection`).
fn nao_seleccionou(h: &mut MockPanelHost, onde: &str) {
    let acoes = h.drained_actions();
    assert!(
        !acoes
            .iter()
            .any(|a| matches!(a, EditorAction::Hierarchy(_) | EditorAction::ClearSelection)),
        "{onde}: a linha mudou a selecção: {acoes:?}"
    );
}

/// ⭐⭐ **Ray «Ignore up to» e Target da bala, pela Hierarchy**: armado, o clique numa linha alimenta o conta-gotas e a
/// selecção fica no dono. A Cena não selecciona nem escreve; a Crate (fora da cadeia do raio, dono 15/11) escreve «Crate»
/// e desarma — a caixa mostra o erro —, o Truck escreve «Truck»; nenhuma muda a selecção.
///
/// **Mutações que sangram:** `pela_hierarquia` não armar (a shell de antes da cura) · o painel sem a guarda da Cena.
#[test]
fn o_conta_gotas_nao_muda_a_selecao() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let corpo = RigidBody {
        kind: BodyKind::Static,
    };
    let truck = w.spawn((Name::new("Truck"), corpo, StableId(31))).id();
    let raio = w
        .spawn((
            Name::new("Ray Sensor"),
            RaySensor::default(),
            StableId(32),
            ChildOf(truck),
        ))
        .id();
    let caixa = w.spawn((Name::new("Crate"), corpo, StableId(33))).id();
    let bala = w.spawn((Name::new("Bullet"), StableId(34))).id();
    let (mut h, mut st) = hierarquia(&sim, &[truck, raio, caixa, bala]);
    let [l_truck, l_caixa] = [NodeId(1), NodeId(3)];
    let mut bus = ActionBus::default();

    // O raio: a Cena não selecciona nem escreve; a Crate e o Truck escrevem e desarmam, sem seleccionar.
    ray_topo::arma(sim.world_mut(), Some(raio.to_bits()));
    assert_eq!(clica(&sim, &mut h, &mut st, CENA), None);
    nao_seleccionou(&mut h, "a Cena com o raio armado");
    let b = clica(&sim, &mut h, &mut st, l_caixa).expect("a linha da Crate vai ao conta-gotas");
    nao_seleccionou(&mut h, "a Crate com o raio armado");
    assert!(conta_gotas::escolher(sim.world_mut(), &mut bus, &[b]));
    assert!(
        !conta_gotas::armado(sim.world()),
        "fora da cadeia: escreve e desarma"
    );
    let crate_ = EditorAction::InspectorComponentEdit {
        entity_bits: raio.to_bits(),
        edit: ComponentEdit::Ray(RayFieldEdit::IgnoreUpTo("Crate".into())),
    };
    assert_eq!(bus.drain().collect::<Vec<_>>(), vec![crate_]);
    ray_topo::arma(sim.world_mut(), Some(raio.to_bits()));
    let b = clica(&sim, &mut h, &mut st, l_truck).expect("a linha do Truck");
    nao_seleccionou(&mut h, "o Truck com o raio armado");
    assert!(conta_gotas::escolher(sim.world_mut(), &mut bus, &[b]));
    let topo = EditorAction::InspectorComponentEdit {
        entity_bits: raio.to_bits(),
        edit: ComponentEdit::Ray(RayFieldEdit::IgnoreUpTo("Truck".into())),
    };
    assert_eq!(bus.drain().collect::<Vec<_>>(), vec![topo]);

    // A bala: a linha da Crate vira o Target.
    sim.world_mut()
        .insert_resource(projectile_inspector::AlvoArmado(Some(bala.to_bits())));
    let b = clica(&sim, &mut h, &mut st, l_caixa).expect("a linha da Crate");
    nao_seleccionou(&mut h, "a Crate com a bala armada");
    assert!(conta_gotas::escolher(sim.world_mut(), &mut bus, &[b]));
    let alvo = EditorAction::InspectorComponentEdit {
        entity_bits: bala.to_bits(),
        edit: ComponentEdit::Projectile(ProjectileFieldEdit::HomingTarget("Crate".into())),
    };
    assert_eq!(bus.drain().collect::<Vec<_>>(), vec![alvo]);

    // Desarmado: a mesma linha volta a seleccionar (controlo positivo da régua).
    assert!(!conta_gotas::armado(sim.world()));
    assert_eq!(clica(&sim, &mut h, &mut st, l_caixa), None);
    assert!(
        h.drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::Hierarchy(_))),
        "desarmado, a linha tem de seleccionar"
    );
}
