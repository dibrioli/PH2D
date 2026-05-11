//! Full ADR-0025 M14 pipeline integration test.
//!
//! Exercises every layer end-to-end:
//!
//! 1. JSON5 source → cooker → postcard bytes (M14.3a).
//! 2. AssetDb::insert_prefab_bytes (M14.3a).
//! 3. spawn_prefab installs Transform+Name+Sprite in SimWorld (M14.3b).
//! 4. propagate_transforms walks the hierarchy and emits
//!    GlobalTransform + SimRef in PresentWorld (M14.1).
//! 5. build_hierarchy_snapshot exposes the editor-facing view (M14.3c).
//! 6. world_to_snapshot captures full state (M14.3d).
//! 7. snapshot_to_world restores into a fresh SimWorld (M14.3d).
//! 8. Restored world's snapshot hash equals the original (HR-5).
//!
//! If this test passes on Linux + macOS + Windows the M14 milestone
//! is end-to-end correct.

use ph2d_asset::AssetDb;
use ph2d_asset_cooker::cook_prefab_json5;
use ph2d_ecs::scene::{
    ComponentRegistry, HierarchySnapshot, HierarchyWalkState, WorldSnapshot,
    build_hierarchy_snapshot, register_ecs_components, snapshot_to_world, spawn_prefab,
    world_to_snapshot,
};
use ph2d_ecs::{
    Name, PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState, WorklistBuf,
    propagate_transforms_into_present,
};
use ph2d_render::register_render_components;

const PREFAB_JSON5: &str = include_str!(
    "../../../tests/fixtures/prefab/simple_sprite.json5"
);

fn build_registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    reg
}

#[test]
fn full_pipeline_cook_spawn_propagate_snapshot_restore() {
    // 1. Cook the JSON5 prefab.
    let cooked = cook_prefab_json5(PREFAB_JSON5).expect("cook prefab");
    assert!(!cooked.is_empty());

    // 2. Insert into AssetDb.
    let db = AssetDb::new();
    let prefab_id = db.insert_prefab_bytes(&cooked).expect("insert prefab");

    // 3. Spawn into SimWorld.
    let registry = build_registry();
    let mut sim = SimWorld::new();
    let prefab_asset = db.get(&prefab_id).unwrap();
    let prefab_doc = match &*prefab_asset {
        ph2d_asset::Asset::Prefab(p) => std::sync::Arc::clone(p),
        _ => panic!("expected prefab variant"),
    };
    let root = spawn_prefab(sim.world_mut(), &prefab_doc, &db, &registry).expect("spawn");

    // Verify components present.
    assert!(sim.world_mut().get::<Transform>(root).is_some());
    let name = sim.world_mut().get::<Name>(root).expect("name component");
    assert_eq!(name.as_str(), "Hero");
    assert!(sim.world_mut().get::<ph2d_render::Sprite>(root).is_some());

    // 4. Propagate transforms — verify the renderer-side view.
    let mut prop_state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut present = PresentWorld::new();
    ph2d_ecs::extract!(sim => present, |sim_w, present_w| {
        present_w.clear_entities();
        propagate_transforms_into_present(sim_w, &mut prop_state, present_w, &mut worklist);
    });
    let mut q = present
        .world_mut()
        .query::<(&SimRef, &ph2d_ecs::GlobalTransform)>();
    let pairs: Vec<_> = q.iter(present.world()).collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0.0, root);

    // 5. Build hierarchy snapshot — editor would consume this.
    let mut walk_state = HierarchyWalkState::new(sim.world_mut());
    let mut scratch = Vec::new();
    let mut hier = HierarchySnapshot::new();
    build_hierarchy_snapshot(sim.world(), &mut walk_state, &mut scratch, &mut hier);
    assert_eq!(hier.len(), 1);
    assert_eq!(hier.entries[0].name.as_deref(), Some("Hero"));

    // 6 + 7 + 8. Capture full snapshot, restore into fresh world,
    // verify hash equality (HR-5).
    let mut snap_a = WorldSnapshot::new();
    world_to_snapshot(
        sim.world(),
        &mut prop_state,
        &mut worklist,
        &registry,
        &mut snap_a,
    )
    .unwrap();
    let hash_a = snap_a.state_hash();

    let mut sim_b = SimWorld::new();
    snapshot_to_world(sim_b.world_mut(), &snap_a, &registry).unwrap();
    let mut prop_state_b = TransformPropagationState::new(sim_b.world_mut());
    let mut snap_b = WorldSnapshot::new();
    world_to_snapshot(
        sim_b.world(),
        &mut prop_state_b,
        &mut worklist,
        &registry,
        &mut snap_b,
    )
    .unwrap();
    let hash_b = snap_b.state_hash();
    assert_eq!(
        hash_a, hash_b,
        "end-to-end roundtrip diverged — HR-5 broken somewhere in the pipeline"
    );
}
