//! ⭐⭐⭐ **REORDENAR NA HIERARQUIA É DESFAZÍVEL** — a cadeia inteira, medida ponta a ponta.
//!
//! ⚠️ **Estes gates nasceram de um report** (Enio, 2026-09-07: *«reordenei objetos na hierarquia e
//! não funcionou o undo»*) e do que ele mediu: as **quatro** peças da cadeia estão certas
//! isoladamente — o gesto escreve, a fotografia vê, a fotografia com a cache INCREMENTAL vê, e a
//! reposição traz de volta. ⇒ o que falha (se falha) está no QUADRO, e não aqui.
//!
//! ⛔ **É isso que os torna valiosos e não redundantes:** enquanto ninguém os tinha, cada nova
//! suspeita mandava alguém reler as mesmas quatro peças. Agora a pergunta *«a cadeia partiu?»* tem
//! resposta em 200 ms, e um report igual começa onde este acabou.
//!
//! ⚠️ A cadeia tem DUAS metades — as **raízes** (`RootOrder`) e os **irmãos** (`SiblingOrder`) — e
//! elas correm em ramos diferentes do `drain_reparent`. Um gate sobre uma só deixa a outra sem
//! régua, que é como a classe BUGS #15 nasceu.

use crate::undo::ProjectState;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_preview_drive::PreviewDrive;

fn reg2() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

fn capture(
    drive: &PreviewDrive,
    sim: &mut SimWorld,
    reg: &ph2d_ecs::scene::ComponentRegistry,
) -> ProjectState {
    ProjectState::capture(
        drive,
        sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

#[test]
fn the_capture_sees_a_reorder_so_the_step_can_be_born() {
    let reg = reg2();
    let drive = PreviewDrive::default();
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("A"),
            ph2d_ecs::StableId(1),
            ph2d_ecs::RootOrder(0),
        ))
        .id();
    let b = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("B"),
            ph2d_ecs::StableId(2),
            ph2d_ecs::RootOrder(1),
        ))
        .id();
    let antes = capture(&drive, &mut sim, &reg);
    // O gesto: trocar a ordem das duas raízes.
    sim.world_mut().entity_mut(a).insert(ph2d_ecs::RootOrder(1));
    sim.world_mut().entity_mut(b).insert(ph2d_ecs::RootOrder(0));
    let depois = capture(&drive, &mut sim, &reg);
    assert_ne!(
        antes, depois,
        "a captura NAO ve a reordenacao — o passo de undo nunca nasce"
    );
}

#[test]
fn restoring_brings_the_root_order_back() {
    let reg = reg2();
    let drive = PreviewDrive::default();
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("A"),
            ph2d_ecs::StableId(1),
            ph2d_ecs::RootOrder(0),
        ))
        .id();
    let b = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("B"),
            ph2d_ecs::StableId(2),
            ph2d_ecs::RootOrder(1),
        ))
        .id();
    let antes = capture(&drive, &mut sim, &reg);
    sim.world_mut().entity_mut(a).insert(ph2d_ecs::RootOrder(1));
    sim.world_mut().entity_mut(b).insert(ph2d_ecs::RootOrder(0));
    let _ = antes.restore(&mut sim, &reg);
    // Depois do restauro as entidades sao NOVAS — procura-se pelo StableId.
    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::StableId, &ph2d_ecs::RootOrder)>();
    let mut ordens: Vec<(u64, u32)> = q.iter(sim.world()).map(|(s, r)| (s.0, r.0)).collect();
    ordens.sort_unstable();
    assert_eq!(
        ordens,
        vec![(1, 0), (2, 1)],
        "o restauro NAO trouxe a ordem de volta"
    );
}

/// E a metade dos IRMAOS.
#[test]
fn restoring_brings_the_sibling_order_back() {
    let reg = reg2();
    let drive = PreviewDrive::default();
    let mut sim = SimWorld::new();
    let p = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("P"), ph2d_ecs::StableId(9)))
        .id();
    let a = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("A"),
            ph2d_ecs::StableId(1),
            ph2d_ecs::ChildOf(p),
            ph2d_ecs::SiblingOrder(0),
        ))
        .id();
    let b = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("B"),
            ph2d_ecs::StableId(2),
            ph2d_ecs::ChildOf(p),
            ph2d_ecs::SiblingOrder(1),
        ))
        .id();
    let antes = capture(&drive, &mut sim, &reg);
    sim.world_mut()
        .entity_mut(a)
        .insert(ph2d_ecs::SiblingOrder(1));
    sim.world_mut()
        .entity_mut(b)
        .insert(ph2d_ecs::SiblingOrder(0));
    let depois = capture(&drive, &mut sim, &reg);
    assert_ne!(antes, depois, "a captura nao ve a reordenacao de IRMAOS");
    let _ = antes.restore(&mut sim, &reg);
    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::StableId, &ph2d_ecs::SiblingOrder)>();
    let mut ordens: Vec<(u64, u32)> = q.iter(sim.world()).map(|(s, o)| (s.0, o.0)).collect();
    ordens.sort_unstable();
    assert_eq!(
        ordens,
        vec![(1, 0), (2, 1)],
        "o restauro NAO trouxe a ordem dos irmaos de volta"
    );
}

/// SONDA 3: com a cache INCREMENTAL reusada entre quadros — que é como o app captura.
#[test]
fn the_incremental_capture_sees_a_reorder_too() {
    let reg = reg2();
    let drive = PreviewDrive::default();
    let mut cache = ph2d_ecs::scene::incremental::CaptureCache::new();
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("A"),
            ph2d_ecs::StableId(1),
            ph2d_ecs::RootOrder(0),
        ))
        .id();
    let b = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("B"),
            ph2d_ecs::StableId(2),
            ph2d_ecs::RootOrder(1),
        ))
        .id();
    let cap = |sim: &mut SimWorld,
               cache: &mut ph2d_ecs::scene::incremental::CaptureCache,
               base: Option<&ProjectState>| {
        ProjectState::capture(
            &drive,
            sim,
            &ph2d_vec_scene::VecScene::new(),
            &ph2d_flip::FlipDoc::new(),
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &[],
            &reg,
            cache,
            base,
        )
    };
    let antes = cap(&mut sim, &mut cache, None);
    sim.world_mut().entity_mut(a).insert(ph2d_ecs::RootOrder(1));
    sim.world_mut().entity_mut(b).insert(ph2d_ecs::RootOrder(0));
    let depois = cap(&mut sim, &mut cache, Some(&antes));
    assert_ne!(
        antes, depois,
        "com a cache incremental a captura NAO ve a reordenacao — o passo nunca nasce"
    );
}

#[test]
fn the_real_root_drag_changes_the_capture() {
    let reg = crate::init::build_component_registry();
    let drive = PreviewDrive::default();
    let mut sim = SimWorld::new();
    let mut ids = Vec::new();
    for (i, nome) in ["A", "B", "C"].iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                Name::new(*nome),
                ph2d_ecs::StableId(i as u64 + 1),
                ph2d_ecs::RootOrder(i as u32),
            ))
            .id();
        ids.push(e);
    }
    // A ponte nó ↔ entidade, montada como o quadro a monta.
    let mut live = crate::HeroLive {
        bridge: crate::hero_bridge::EntityNodeMap::new(),
        walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        walk_scratch: Vec::new(),
        snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
        z_walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        z_walk_scratch: Vec::new(),
        z_snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
    };
    ph2d_ecs::scene::build_hierarchy_snapshot(
        sim.world(),
        &mut live.walk_state,
        &mut live.walk_scratch,
        &mut live.snapshot,
    );
    let _ = live.bridge.sync_from_snapshot(&live.snapshot);
    let node_of = |live: &crate::HeroLive, e: ph2d_ecs::Entity| {
        live.bridge.node_for(e.to_bits()).expect("no na ponte")
    };
    let antes = ProjectState::capture(
        &drive,
        &mut sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    // O GESTO: arrastar o C para ANTES do A.
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    crate::hero_intents::drain_reparent(
        ph2d_editor_core::screens::hero::HierReparentIntent {
            dragged: node_of(&live, ids[2]),
            new_parent: None,
            before: Some(node_of(&live, ids[0])),
            after: None,
        },
        &live,
        &mut sim,
        &mut toasts,
    );
    let ordem: Vec<(u64, u32)> = {
        let mut q = sim
            .world_mut()
            .query::<(&ph2d_ecs::StableId, &ph2d_ecs::RootOrder)>();
        let mut v: Vec<(u64, u32)> = q.iter(sim.world()).map(|(s, r)| (s.0, r.0)).collect();
        v.sort_unstable();
        v
    };
    assert_eq!(
        ordem,
        vec![(1, 1), (2, 2), (3, 0)],
        "o gesto real nao reordenou (C devia ficar em 0)"
    );
    let depois = ProjectState::capture(
        &drive,
        &mut sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    assert_ne!(antes, depois, "o gesto real NAO muda a captura");
}

/// A outra metade: reordenar IRMÃOS dentro de um grupo.
#[test]
fn the_real_sibling_drag_reorders_and_sticks() {
    let reg = crate::init::build_component_registry();
    let drive = PreviewDrive::default();
    let mut sim = SimWorld::new();
    let pai = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("P"),
            ph2d_ecs::StableId(9),
            ph2d_ecs::RootOrder(0),
        ))
        .id();
    let mut ids = Vec::new();
    for (i, nome) in ["A", "B", "C"].iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                Name::new(*nome),
                ph2d_ecs::StableId(i as u64 + 1),
                ph2d_ecs::ChildOf(pai),
                ph2d_ecs::SiblingOrder(i as u32),
            ))
            .id();
        ids.push(e);
    }
    let mut live = crate::HeroLive {
        bridge: crate::hero_bridge::EntityNodeMap::new(),
        walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        walk_scratch: Vec::new(),
        snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
        z_walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        z_walk_scratch: Vec::new(),
        z_snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
    };
    ph2d_ecs::scene::build_hierarchy_snapshot(
        sim.world(),
        &mut live.walk_state,
        &mut live.walk_scratch,
        &mut live.snapshot,
    );
    let _ = live.bridge.sync_from_snapshot(&live.snapshot);
    let node_of = |live: &crate::HeroLive, e: ph2d_ecs::Entity| {
        live.bridge.node_for(e.to_bits()).expect("no na ponte")
    };
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    crate::hero_intents::drain_reparent(
        ph2d_editor_core::screens::hero::HierReparentIntent {
            dragged: node_of(&live, ids[2]),
            new_parent: None,
            before: Some(node_of(&live, ids[0])),
            after: None,
        },
        &live,
        &mut sim,
        &mut toasts,
    );
    let ordem: Vec<(u64, u32)> = {
        let mut q = sim
            .world_mut()
            .query::<(&ph2d_ecs::StableId, &ph2d_ecs::SiblingOrder)>();
        let mut v: Vec<(u64, u32)> = q.iter(sim.world()).map(|(s, o)| (s.0, o.0)).collect();
        v.sort_unstable();
        v
    };
    assert_eq!(
        ordem,
        vec![(1, 1), (2, 2), (3, 0)],
        "o gesto real nao reordenou os IRMAOS"
    );
    let _ = drive;
    let _ = reg;
}
