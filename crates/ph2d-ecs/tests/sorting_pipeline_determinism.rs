//! HR-5 cross-OS determinism gate for the canonical sorting pipeline
//! (Sprite Inspector v2 W3.T3.19, spec
//! [`05_ordering_sorting.md §5.6`](../../../docs/Sprite_projeto/05_ordering_sorting.md)).
//!
//! A fixed 10-sprite, 4-level scene mixing SortingLayer / Z Index ±5 /
//! YSort / SortingGroup / ShowBehindParent must produce a **byte-
//! identical render order on every platform**. The order is captured as
//! a permutation of stable per-sprite labels (not `Entity` bits) and
//! locked to a golden. The only float in the pipeline is the YSort
//! projection, quantized through `libm::roundf`, so the permutation is
//! reproducible across Linux / macOS / Windows.
//!
//! If this golden changes you have altered the sorting semantics —
//! confirm it is intentional, then re-lock (and bump ADR-0073 if the
//! pipeline contract moved).

use bevy_ecs::component::Component;
use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, PresentWorld, ShowBehindParent, SimWorld, SortInput, SortingGroup, SortingLayer,
    Transform, TransformPropagationState, WorklistBuf, YSort, ZAsRelative, ZIndexOverride,
    compute_sort_ranks, propagate_transforms,
};
use ph2d_ecs::{LayerId, SortingLayers};

/// Stable per-sprite label so the locked order does not depend on
/// `Entity` bit allocation.
#[derive(Component, Copy, Clone)]
struct Label(u8);

fn t(x: f32, y: f32) -> Transform {
    Transform::from_translation(Vec2::new(x, y))
}

/// Build the canonical W3 sorting scene. Returns the sim world.
///
/// Layout (labels in brackets):
/// ```text
/// bg     [0]  SortingLayer "Background"(0)
/// world       (no sprite — organizational YSort root, axis = +Y)
/// ├ tree [1]  y=10
/// ├ char [2]  y=5  SortingGroup (multi-piece block root)
/// │ ├ shadow[3]  ShowBehindParent
/// │ ├ body  [4]
/// │ └ hat   [5]  ZIndexOverride(+5) relative
/// └ rock [6]  y=15
/// enemy  [7]  y=8  ZIndexOverride(-5)
/// fx     [8]  SortingLayer "Foreground"(3)
/// ui     [9]  SortingLayer "UI"(4)
/// ```
/// Exercised: named layers, YSort topdown ordering (char<tree<rock by
/// world Y, the libm-quantized float path), a multi-piece SortingGroup
/// block, relative Z (hat front) and absolute-ish Z bucketing (enemy
/// behind), and ShowBehindParent.
fn build_scene() -> SimWorld {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.insert_resource(SortingLayers::canonical());

    w.spawn((t(0.0, 0.0), SortingLayer(LayerId(0)), Label(0)));

    let world = w.spawn((t(0.0, 0.0), YSort::default())).id();
    w.spawn((t(0.0, 10.0), ChildOf(world), Label(1)));

    let char_root = w
        .spawn((t(0.0, 5.0), ChildOf(world), SortingGroup::default(), Label(2)))
        .id();
    w.spawn((t(0.0, 0.0), ChildOf(char_root), ShowBehindParent, Label(3)));
    w.spawn((t(0.0, 0.0), ChildOf(char_root), Label(4)));
    w.spawn((
        t(0.0, 0.0),
        ChildOf(char_root),
        ZIndexOverride(5),
        ZAsRelative(true),
        Label(5),
    ));

    w.spawn((t(0.0, 15.0), ChildOf(world), Label(6)));

    w.spawn((t(0.0, 8.0), ZIndexOverride(-5), Label(7)));

    w.spawn((t(0.0, 0.0), SortingLayer(LayerId(3)), Label(8)));
    w.spawn((t(0.0, 0.0), SortingLayer(LayerId(4)), Label(9)));

    sim
}

/// Drive the real propagate → sort path and return labels in render
/// order.
fn render_order(sim: &mut SimWorld) -> Vec<u8> {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();

    let mut inputs: Vec<SortInput> = Vec::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate_transforms(
            sim_w,
            &mut state,
            present_w,
            &mut worklist,
            |s, _p, e, gt| {
                // Only sprite-labelled entities participate (mirrors the
                // real extract emitting only sprite-bearing entities).
                if s.get::<Label>(e).is_some() {
                    inputs.push(SortInput {
                        entity: e,
                        world_pos: gt.translation(),
                    });
                }
            },
        );
    });

    let ranks = compute_sort_ranks(sim.world(), &inputs);
    let mut by_rank: Vec<(u32, u8)> = ranks
        .iter()
        .map(|&(e, r)| (r, sim.world().get::<Label>(e).unwrap().0))
        .collect();
    by_rank.sort_unstable();
    by_rank.into_iter().map(|(_, l)| l).collect()
}

#[test]
fn canonical_scene_order_is_locked() {
    let mut sim = build_scene();
    let order = render_order(&mut sim);

    // Golden render order (furthest-back → front), by label.
    // Derivation:
    //  - Background layer (0): bg[0] first.
    //  - Default layer (2): enemy[7] (Z=-5) buckets behind everything at
    //    z=0; then YSort orders the z=0 items by world Y — char block
    //    (y=5) < tree[1] (y=10) < rock[6] (y=15). Inside the char block:
    //    shadow[3] draws behind the parent, then char[2], body[4], and
    //    hat[5] last (relative Z +5).
    //  - Foreground (3): fx[8].
    //  - UI (4): ui[9].
    let expected = vec![0u8, 7, 3, 2, 4, 5, 1, 6, 8, 9];
    assert_eq!(
        order, expected,
        "sorting pipeline render order drifted — confirm intent, then re-lock"
    );
}

#[test]
fn order_is_stable_across_repeated_runs() {
    let mut a = build_scene();
    let mut b = build_scene();
    assert_eq!(render_order(&mut a), render_order(&mut b));
}
