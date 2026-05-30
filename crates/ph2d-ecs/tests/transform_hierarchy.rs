//! Integration tests for hierarchical [`propagate_transforms`].
//!
//! Covers ADR-0025 §"Hierarquia" guarantees: 3-level chain composes
//! correctly across translation+rotation+scale; despawn of a root
//! cascades via `ChildOf`; entities without `Transform` are skipped.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, GlobalTransform, PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState,
    WorklistBuf, propagate_transforms_into_present,
};

/// Run a propagation pass and return all (SimRef, GlobalTransform)
/// pairs from PresentWorld. Used by tests below.
fn extract_pairs(
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
) -> Vec<(bevy_ecs::entity::Entity, GlobalTransform)> {
    present.world_mut().clear_entities();
    ph2d_ecs::extract!(*sim => *present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, state, present_w, worklist);
    });
    let mut q = present.world_mut().query::<(&SimRef, &GlobalTransform)>();
    let mut out: Vec<_> = q.iter(present.world()).map(|(s, g)| (s.0, *g)).collect();
    out.sort_by_key(|(e, _)| e.to_bits());
    out
}

#[test]
fn root_alone_produces_identity_world() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();

    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, e);
    assert_eq!(pairs[0].1.translation(), Vec2::ZERO);
}

#[test]
fn two_level_chain_composes_translation() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let parent = sim
        .world_mut()
        .spawn(Transform::from_translation(Vec2::new(10.0, 0.0)))
        .id();
    let child = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(2.0, 3.0)),
            ChildOf(parent),
        ))
        .id();

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);

    assert_eq!(pairs.len(), 2);
    let parent_g = pairs.iter().find(|(e, _)| *e == parent).unwrap();
    let child_g = pairs.iter().find(|(e, _)| *e == child).unwrap();
    assert_eq!(parent_g.1.translation(), Vec2::new(10.0, 0.0));
    assert_eq!(child_g.1.translation(), Vec2::new(12.0, 3.0));
}

#[test]
fn three_level_chain_with_rotation_and_scale() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    // Root: translate to (5, 5), 90° CCW rotation, 2× scale.
    let root = sim
        .world_mut()
        .spawn(Transform {
            translation: Vec2::new(5.0, 5.0),
            rotation: std::f32::consts::FRAC_PI_2,
            scale: Vec2::new(2.0, 2.0),
            ..Transform::IDENTITY
        })
        .id();
    // Middle: local translate (1, 0) — after parent's rotation+scale
    // becomes (0, 2) added to (5, 5) = (5, 7).
    let mid = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            ChildOf(root),
        ))
        .id();
    // Leaf: local translate (1, 0) — middle's rotation is still 90°
    // and scale still 2× (composed). Two more units forward → adds
    // another (0, 2) to (5, 7) = (5, 9).
    let leaf = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            ChildOf(mid),
        ))
        .id();

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);

    assert_eq!(pairs.len(), 3);
    let root_g = pairs.iter().find(|(e, _)| *e == root).unwrap();
    let mid_g = pairs.iter().find(|(e, _)| *e == mid).unwrap();
    let leaf_g = pairs.iter().find(|(e, _)| *e == leaf).unwrap();
    let eps = 1e-5;
    assert!((root_g.1.translation().x - 5.0).abs() < eps);
    assert!((root_g.1.translation().y - 5.0).abs() < eps);
    assert!((mid_g.1.translation().x - 5.0).abs() < eps);
    assert!((mid_g.1.translation().y - 7.0).abs() < eps);
    assert!((leaf_g.1.translation().x - 5.0).abs() < eps);
    assert!((leaf_g.1.translation().y - 9.0).abs() < eps);
}

#[test]
fn three_level_skew_cascade_shears_leaf(/* ADR-0025-amendment-1 §5 */) {
    // Root skew_x = 0.3, mid skew_x = 0.2 → composed skew_x = 0.5
    // (additive cascade §2.2.1). A leaf translated locally to (0, 1)
    // under a +0.5 skew_x must land at world (tan(0.5), 1): the +Y
    // offset is sheared into +X by tan(skew_x). No rotation/scale, so
    // the only effect observable is the skew shear — the clean smoke.
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let root = sim
        .world_mut()
        .spawn(Transform {
            skew_x: 0.3,
            ..Transform::IDENTITY
        })
        .id();
    let mid = sim
        .world_mut()
        .spawn((
            Transform {
                skew_x: 0.2,
                ..Transform::IDENTITY
            },
            ChildOf(root),
        ))
        .id();
    let leaf = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, 1.0)),
            ChildOf(mid),
        ))
        .id();

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);

    assert_eq!(pairs.len(), 3);
    let leaf_g = pairs.iter().find(|(e, _)| *e == leaf).unwrap();
    let expected_x = libm::tanf(0.5);
    let eps = 1e-5;
    assert!(
        (leaf_g.1.translation().x - expected_x).abs() < eps,
        "leaf world x = {} (expected tan(0.5) = {}); additive skew cascade or shear broken",
        leaf_g.1.translation().x,
        expected_x
    );
    assert!(
        (leaf_g.1.translation().y - 1.0).abs() < eps,
        "leaf world y = {} (expected 1.0); skew must not move the Y axis here",
        leaf_g.1.translation().y
    );
    // root/mid have no translation → stay at the origin.
    let root_g = pairs.iter().find(|(e, _)| *e == root).unwrap();
    let mid_g = pairs.iter().find(|(e, _)| *e == mid).unwrap();
    assert!(root_g.1.translation().abs_diff_eq(Vec2::ZERO, eps));
    assert!(mid_g.1.translation().abs_diff_eq(Vec2::ZERO, eps));
}

#[test]
fn entities_without_transform_are_skipped() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    // Two transformed entities + one bare entity (no Transform).
    let a = sim
        .world_mut()
        .spawn(Transform::from_translation(Vec2::new(1.0, 0.0)))
        .id();
    let b = sim
        .world_mut()
        .spawn(Transform::from_translation(Vec2::new(0.0, 1.0)))
        .id();
    // Spawn a marker component that doesn't include Transform.
    sim.world_mut().spawn_empty();

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);

    assert_eq!(
        pairs.len(),
        2,
        "only the two Transform-bearing entities propagate"
    );
    let ids: Vec<_> = pairs.iter().map(|(e, _)| *e).collect();
    assert!(ids.contains(&a));
    assert!(ids.contains(&b));
}

#[test]
fn despawn_root_cascades_via_child_of() {
    // bevy_ecs 0.18 `ChildOf` relationship makes despawning the root
    // automatically despawn all descendants. propagate_transforms
    // should then emit zero pairs.
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let root = sim.world_mut().spawn(Transform::IDENTITY).id();
    sim.world_mut().spawn((Transform::IDENTITY, ChildOf(root)));
    sim.world_mut().spawn((Transform::IDENTITY, ChildOf(root)));

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    {
        let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);
        assert_eq!(pairs.len(), 3, "root + 2 children present before despawn");
    }

    // Despawn root — should cascade to children.
    sim.world_mut().despawn(root);

    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);
    assert_eq!(
        pairs.len(),
        0,
        "all descendants gone after root despawn (ChildOf cascade)"
    );
}

/// Regression for ADR-0025 M14.4c "imported sprite invisible":
/// `TransformPropagationState` is built once at boot (typical
/// pattern in shell main loops). When the host spawns NEW entities
/// of a NEW archetype (e.g. M14.4c imports add (Transform, Sprite,
/// Name) while the demo's seed had (Transform, Velocity, Sprite,
/// Name)), `propagate_transforms` must still pick them up.
///
/// The fix: `propagate_transforms` calls `update_archetypes` on
/// both `roots` and `chain` query states at the top of each pass.
/// This test would catch a regression where that call was removed.
#[test]
fn propagation_discovers_entities_of_new_archetype_after_init() {
    let mut sim = SimWorld::new();
    // Seed archetype A: (Transform).
    let a = sim.world_mut().spawn(Transform::IDENTITY).id();
    // Build QueryState cache from CURRENT archetypes only.
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();

    // Now spawn a brand-new archetype B: (Transform, Marker) where
    // Marker is a fresh local component the cache has never seen.
    #[derive(bevy_ecs::component::Component)]
    struct Marker;
    let b = sim.world_mut().spawn((Transform::IDENTITY, Marker)).id();

    let mut present = PresentWorld::new();
    ph2d_ecs::extract!(sim => present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, &mut state, present_w, &mut worklist);
    });
    let mut q = present.world_mut().query::<&SimRef>();
    let mut found: Vec<bevy_ecs::entity::Entity> = q.iter(present.world()).map(|s| s.0).collect();
    found.sort_by_key(|e| e.to_bits());
    assert!(
        found.contains(&a),
        "missing original-archetype entity {a:?} (got {found:?})"
    );
    assert!(
        found.contains(&b),
        "missing new-archetype entity {b:?} — update_archetypes regression \
         (got {found:?})"
    );
}

#[test]
fn multiple_roots_are_independent() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let a = sim
        .world_mut()
        .spawn(Transform::from_translation(Vec2::new(10.0, 0.0)))
        .id();
    let b = sim
        .world_mut()
        .spawn(Transform::from_translation(Vec2::new(0.0, 20.0)))
        .id();

    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    let pairs = extract_pairs(&mut sim, &mut present, &mut state, &mut worklist);

    assert_eq!(pairs.len(), 2);
    let a_g = pairs.iter().find(|(e, _)| *e == a).unwrap();
    let b_g = pairs.iter().find(|(e, _)| *e == b).unwrap();
    assert_eq!(a_g.1.translation(), Vec2::new(10.0, 0.0));
    assert_eq!(b_g.1.translation(), Vec2::new(0.0, 20.0));
}
