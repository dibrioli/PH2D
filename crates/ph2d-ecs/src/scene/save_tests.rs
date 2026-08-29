//! **Os gates do snapshot v2** — irmão de [`super::save`] pelo teto de 700 LOC da workspace.
//!
//! ⚠️ **O corte é o precedente da própria pasta** (`save_v1.rs` + `save_v1_tests.rs`), e foi
//! forçado em 2026-08-25: o `Arc` por linha da F2 (ADR-0164) levou o `save.rs` a **704** contra um
//! teto de 700. O ficheiro-mãe responde *que forma tem um snapshot e como o mundo entra e sai
//! dele*; isto responde *o que tem de continuar verdadeiro sobre ele*.

use super::*;
use crate::SimWorld;
use crate::scene::register_ecs_components;
use crate::{Name, Transform};
use ph2d_core::Vec2;

/// ⭐ **A propriedade que o `canonicalize` comprava, provada sem ele** (ADR-0164 F1).
///
/// A lei: *dois estados logicamente iguais dão o MESMO snapshot* — e o caso duro é o
/// **restore**, que despawna tudo e re-spawna com `Entity` novos. Enquanto a ordem das
/// linhas vinha do `to_bits()` (id de ALOCAÇÃO), esse respawn mudava os bytes, e o diff
/// do undo registava um passo espúrio a cada quadro com input — o Ctrl+Z parecia *"não
/// fazer nada"* (Enio, 2026-07-09). O shell curava-o reordenando por CONTEÚDO a cada
/// captura (18,7 ms a 10 k entidades).
///
/// Na v2 a ordem é o `StableId`, que **sobrevive ao respawn por construção**. Este gate é
/// o que prova que a cura não se perdeu com a função que a implementava.
#[test]
fn the_snapshot_survives_a_respawn_byte_for_byte() {
    let (mut sim, reg) = populated_world();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();

    let mut before = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut before)
        .expect("captura");

    // O restore do undo: despawna tudo e re-spawna — `Entity` novos, `to_bits` novos.
    // ⚠️ `Without<IsResource>`: no bevy_ecs 0.19 os recursos são entidades, e despawná-las
    // **entra em pânico** (`Entity despawned: 0v0 is invalid`). O undo de verdade
    // (`shells/desktop/src/undo.rs`) nunca teve este defeito — ele filtra por `With<Transform>`,
    // que é positivo e por isso nunca as alcançou. Este teste imitava o undo pela FORMA e não
    // pela GARANTIA, e foi só aqui que a diferença apareceu.
    let editable: Vec<Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<Entity, bevy_ecs::query::Without<bevy_ecs::resource::IsResource>>();
        q.iter(sim.world()).collect()
    };
    for e in editable {
        let _ = sim.world_mut().despawn(e);
    }
    snapshot_to_world(sim.world_mut(), &before, &reg).expect("restore");

    let mut after = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut after)
        .expect("re-captura");

    assert_eq!(
        before.state_hash(),
        after.state_hash(),
        "capturar -> restaurar -> capturar tem de dar o MESMO hash. Se falhar, a ordem \
         das linhas voltou a depender de algo que o respawn muda, e cada quadro com \
         input volta a registar um passo de undo espurio.",
    );
    assert_eq!(before, after, "e byte a byte, nao so o hash");
}

/// **O `parent` é um ID, e é isso que faz a linha de um objeto não mudar quando OUTRO
/// nasce** — a propriedade sobre a qual a captura incremental da F2 é construída.
///
/// Com o `parent` em índice, inserir uma entidade empurrava o índice de todas as linhas
/// seguintes: os bytes delas mudavam, e um diff por linha veria o mundo inteiro sujo por
/// causa de um objeto novo.
#[test]
fn adding_an_entity_does_not_change_the_other_rows() {
    let (mut sim, reg) = populated_world();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();

    let mut before = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut before)
        .expect("captura");

    // Uma raiz nova, sem relacao nenhuma com as que ja' existiam.
    sim.world_mut()
        .spawn((Transform::IDENTITY, Name::new("Newcomer")));

    let mut after = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut after)
        .expect("re-captura");

    for old in &before.entities {
        let same = after
            .entities
            .iter()
            .find(|r| r.id == old.id)
            .expect("a linha antiga continua la");
        assert_eq!(
            same, old,
            "a linha de {:?} mudou por causa de um objeto NOVO — o `parent` voltou a ser \
             um indice, e a captura incremental da F2 veria o mundo inteiro sujo.",
            old.id,
        );
    }
}

fn populated_world() -> (SimWorld, ComponentRegistry) {
    let mut sim = SimWorld::new();
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    // Build a 3-level hierarchy with names.
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(10.0, 20.0)),
            Name::new("Root"),
        ))
        .id();
    let mid = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            Name::new("Mid"),
            ChildOf(root),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.5, 0.5)),
        Name::new("Leaf"),
        ChildOf(mid),
    ));
    (sim, reg)
}

#[test]
fn snapshot_captures_all_entities() {
    let (mut sim, reg) = populated_world();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
    assert_eq!(snap.entities.len(), 3);
}

/// ⚠️ **A §5 e a §12 nasceram registadas e SEM prova de que sobrevivem ao disco** (achado da
/// auditoria de fecho do 9-slice, 2026-08-22).
///
/// Estar no `ComponentRegistry` faz o componente ser **escrito**; nada disso prova que ele
/// volta igual. E a forma do `SliceNine` mudou três vezes num dia — cada uma dessas mudanças
/// atravessou este caminho sem um teste a olhar.
///
/// ⚠️ **O fixture usa valores NÃO-DEFAULT em cada campo**, e é isso que o torna uma prova: um
/// `SliceNine::INERT` gravado e relido daria igual mesmo que o restore devolvesse o default,
/// e a mesma lei vale para a âncora (nome, pose, bounds e centro todos diferentes de zero).
#[test]
fn the_sprite_authoring_components_survive_the_disk() {
    use crate::{
        NamedAnchor, NamedAnchorList, SliceDrawMode, SliceNine, SliceTileMode, TileRegionMode,
    };

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);

    let slice = SliceNine {
        draw_mode: SliceDrawMode::Sliced,
        borders: [3.0, 5.0, 7.0, 11.0],
        size: [1.5, 2.5],
        tile_modes: [
            TileRegionMode::Stretch,
            TileRegionMode::Repeat,
            TileRegionMode::Blank,
            TileRegionMode::Mirror,
            TileRegionMode::Repeat,
            TileRegionMode::Stretch,
            TileRegionMode::Mirror,
            TileRegionMode::Blank,
        ],
        centre_tile_mode: TileRegionMode::Mirror,
        tile_mode: SliceTileMode::Whole,
        fill_center: false,
    };
    let mut anchors = NamedAnchorList::default();
    let mut a = NamedAnchor::socket("muzzle");
    a.transform.translation = ph2d_core::Vec2::new(0.25, -0.75);
    a.set_bounds(Some([1.0, 2.0, 3.0, 4.0]));
    a.set_center(Some([0.5, 0.5, 0.5, 0.5]));
    anchors.insert(a).expect("cabe");

    let mut sim_a = SimWorld::new();
    sim_a
        .world_mut()
        .spawn((Transform::default(), slice, anchors.clone()));
    let mut state = TransformPropagationState::new(sim_a.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(
        sim_a.world_mut(),
        &mut state,
        &mut worklist,
        &reg,
        &mut snap,
    )
    .unwrap();

    let mut sim_b = SimWorld::new();
    let back = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
    let e = *back.first().expect("uma entidade");
    assert_eq!(
        sim_b.world_mut().get::<SliceNine>(e).copied(),
        Some(slice),
        "o 9-slice nao voltou igual do disco"
    );
    assert_eq!(
        sim_b.world_mut().get::<NamedAnchorList>(e),
        Some(&anchors),
        "a lista de ancoras nao voltou igual do disco"
    );
}

/// **A montagem tem de sobreviver ao disco COM o pai** (ADR-0072 §2.6).
///
/// ⚠️ Guardar o componente e perder a hierarquia — ou o contrário — deixa o vínculo
/// pendurado sem que nada avise: a espada reabre no sítio certo, parada, e só se descobre
/// quando o braço se mexe. Por isso este gate reabre a árvore e **volta a perguntar o estado
/// da montagem**, em vez de comparar dois blobs.
#[test]
fn a_mount_survives_the_disk_together_with_the_parent_that_gives_it_meaning() {
    use crate::{AnchorMount, MountState, NamedAnchor, NamedAnchorList, mount_state_of};

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);

    let mut anchors = NamedAnchorList::default();
    let mut a = NamedAnchor::socket("hand_r");
    a.transform.translation = ph2d_core::Vec2::new(0.5, 1.25);
    anchors.insert(a).expect("cabe");

    let mut sim_a = SimWorld::new();
    let host = sim_a
        .world_mut()
        .spawn((Transform::default(), anchors, crate::Name::new("hero")))
        .id();
    sim_a.world_mut().spawn((
        Transform::default(),
        crate::ChildOf(host),
        AnchorMount::new("hand_r"),
        crate::Name::new("sword"),
    ));

    let mut state = TransformPropagationState::new(sim_a.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(
        sim_a.world_mut(),
        &mut state,
        &mut worklist,
        &reg,
        &mut snap,
    )
    .unwrap();

    let mut sim_b = SimWorld::new();
    let back = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
    let sword = back
        .iter()
        .copied()
        .find(|&e| {
            sim_b
                .world_mut()
                .get::<crate::Name>(e)
                .is_some_and(|n| n.as_str() == "sword")
        })
        .expect("a espada voltou");
    assert!(
        matches!(mount_state_of(sim_b.world(), sword), MountState::Mounted(_)),
        "a montagem nao resolveu depois de reabrir — o componente ou o pai perdeu-se"
    );
    assert_eq!(
        sim_b.world_mut().get::<AnchorMount>(sword),
        Some(&AnchorMount::new("hand_r"))
    );
}

#[test]
fn snapshot_restore_round_trip_preserves_names_and_hierarchy() {
    let (mut sim_a, reg) = populated_world();
    let mut state = TransformPropagationState::new(sim_a.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(
        sim_a.world_mut(),
        &mut state,
        &mut worklist,
        &reg,
        &mut snap,
    )
    .unwrap();

    let mut sim_b = SimWorld::new();
    let entities = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
    assert_eq!(entities.len(), 3);

    // Verify names + hierarchy in the restored world.
    let names: Vec<String> = entities
        .iter()
        .map(|e| {
            sim_b
                .world_mut()
                .get::<Name>(*e)
                .unwrap()
                .as_str()
                .to_owned()
        })
        .collect();
    assert!(names.contains(&"Root".to_string()));
    assert!(names.contains(&"Mid".to_string()));
    assert!(names.contains(&"Leaf".to_string()));

    // The Leaf entity (last in visit order) should have a parent.
    let parents: Vec<Option<Entity>> = entities
        .iter()
        .map(|e| sim_b.world_mut().get::<ChildOf>(*e).map(|c| c.0))
        .collect();
    // Exactly one root (Root) → exactly one None.
    let root_count = parents.iter().filter(|p| p.is_none()).count();
    assert_eq!(root_count, 1);
}

#[test]
fn state_hash_is_deterministic_across_round_trip() {
    let (mut sim_a, reg) = populated_world();
    let mut state = TransformPropagationState::new(sim_a.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap_a = WorldSnapshot::new();
    world_to_snapshot(
        sim_a.world_mut(),
        &mut state,
        &mut worklist,
        &reg,
        &mut snap_a,
    )
    .unwrap();
    let hash_a = snap_a.state_hash();

    // Restore to a fresh world, snapshot again — hashes match.
    let mut sim_b = SimWorld::new();
    snapshot_to_world(sim_b.world_mut(), &snap_a, &reg).unwrap();
    let mut state_b = TransformPropagationState::new(sim_b.world_mut());
    let mut snap_b = WorldSnapshot::new();
    world_to_snapshot(
        sim_b.world_mut(),
        &mut state_b,
        &mut worklist,
        &reg,
        &mut snap_b,
    )
    .unwrap();
    let hash_b = snap_b.state_hash();
    assert_eq!(
        hash_a, hash_b,
        "snapshot round-trip produced a different state hash"
    );
}

#[test]
fn snapshot_postcard_round_trips() {
    let (mut sim, reg) = populated_world();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
    let bytes = postcard::to_allocvec(&snap).unwrap();
    let decoded: WorldSnapshot = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, snap);
}
