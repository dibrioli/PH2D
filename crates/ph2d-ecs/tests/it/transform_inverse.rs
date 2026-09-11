//! **`inverse_compose` is `compose` run backwards** — and the two must agree,
//! because a physics solver answers in WORLD space while `Transform` is LOCAL.
//!
//! Writing a world pose straight into a parented entity's `Transform` makes the
//! renderer compose it with the parent *again*: the body simulates in one place
//! and draws in another (`BUGS_physics.md` #2). The fix needs the other
//! direction, and the only specification that matters for it is the round trip.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform, parent_world_transform, parent_world_transform_into};

/// A spread of parents that reaches every branch of `compose`: plain offsets,
/// rotations past ±π, non-uniform scale, negative (mirroring) scale, and skew
/// on both axes.
fn parents() -> Vec<Transform> {
    let mut out = vec![Transform::IDENTITY];
    for &rot in &[0.0f32, 0.3, -1.9, 3.0, -3.1] {
        for &s in &[(1.0f32, 1.0f32), (2.0, 2.0), (0.25, 3.0), (-1.5, 0.75)] {
            for &(kx, ky) in &[(0.0f32, 0.0f32), (0.4, 0.0), (0.0, -0.3), (0.25, 0.2)] {
                out.push(Transform {
                    translation: Vec2::new(-7.25, 4.5),
                    rotation: rot,
                    scale: Vec2::new(s.0, s.1),
                    skew_x: kx,
                    skew_y: ky,
                });
            }
        }
    }
    out
}

fn children() -> Vec<Transform> {
    vec![
        Transform::IDENTITY,
        Transform::from_translation(Vec2::new(3.0, -2.0)),
        Transform {
            translation: Vec2::new(-0.75, 9.5),
            rotation: 1.1,
            scale: Vec2::new(0.5, 2.0),
            skew_x: 0.1,
            skew_y: -0.05,
        },
    ]
}

/// **The round trip is the whole specification.**
///
/// The tolerance is not a guess: the sweep below prints the worst error it
/// actually produced, and the bar sits an order of magnitude above it. A
/// physics pose is in metres, so `1e-4` is a tenth of a millimetre — far under
/// anything the solver's own `normalized_allowed_linear_error` (1.3 mm,
/// measured in W2a) treats as contact.
#[test]
fn inverse_compose_is_the_exact_inverse_of_compose() {
    const TOL: f32 = 1e-4;
    let mut worst = 0.0f32;
    let mut worst_case = String::new();

    for p in parents() {
        for c in children() {
            let world = Transform::compose(p, c);
            let back =
                Transform::inverse_compose(p, world).expect("none of these parents is degenerate");
            let err = [
                (back.translation.x - c.translation.x).abs(),
                (back.translation.y - c.translation.y).abs(),
                (back.rotation - c.rotation).abs(),
                (back.scale.x - c.scale.x).abs(),
                (back.scale.y - c.scale.y).abs(),
                (back.skew_x - c.skew_x).abs(),
                (back.skew_y - c.skew_y).abs(),
            ]
            .into_iter()
            .fold(0.0f32, f32::max);
            if err > worst {
                worst = err;
                worst_case = format!("{p:?} ∘ {c:?}");
            }
        }
    }
    println!("worst round-trip error: {worst:e}  ({worst_case})");
    assert!(
        worst < TOL,
        "inverse_compose lost {worst:e} recovering the child — worst at {worst_case}"
    );
}

/// Under an identity parent there is nothing to undo, so the recovery is not
/// merely close — it is the **same bits**.
///
/// This is the gate that a tolerance cannot hide behind: an implementation that
/// quietly rounded, normalised an angle, or lost a sign would pass the sweep
/// above and fail here.
#[test]
fn an_identity_parent_returns_the_child_unchanged_bit_for_bit() {
    for c in children() {
        let back =
            Transform::inverse_compose(Transform::IDENTITY, c).expect("identity is invertible");
        assert_eq!(back, c, "the identity parent altered the child");
    }
}

/// **A parent that destroys information has no inverse, and saying so is the
/// point.**
///
/// Dividing anyway yields `±inf`/`NaN`, and `compose`'s own `debug_assert`
/// spells out the price: one corrupted angle poisons the whole subtree's
/// `GlobalTransform`, with signaling-vs-quiet NaN patterns that drift
/// cross-host. `None` lets the caller leave the pose alone instead.
///
/// The guard asks whether the RESULT is finite, so a poisoned *input* is
/// refused by the same rule — that case is in the table below on purpose.
#[test]
fn a_parent_that_destroys_information_has_no_inverse() {
    let flat_x = Transform {
        scale: Vec2::new(0.0, 1.0),
        ..Transform::IDENTITY
    };
    let flat_y = Transform {
        scale: Vec2::new(1.0, 0.0),
        ..Transform::IDENTITY
    };
    let poisoned = Transform {
        rotation: f32::NAN,
        ..Transform::IDENTITY
    };

    for (p, what) in [
        (flat_x, "scale.x = 0"),
        (flat_y, "scale.y = 0"),
        (poisoned, "a NaN that arrived in the input"),
    ] {
        assert!(
            Transform::inverse_compose(p, Transform::IDENTITY).is_none(),
            "{what}: must refuse, not hand back something unstorable"
        );
    }

    // …and every non-degenerate parent in the sweep must NOT refuse — an
    // over-eager guard would turn every child body into a no-op, silently.
    for p in parents() {
        assert!(
            Transform::inverse_compose(p, Transform::IDENTITY).is_some(),
            "a healthy parent was refused: {p:?}"
        );
    }
}

/// **An ill-conditioned parent is allowed through, deliberately.**
///
/// `SKEW_LIMIT` is `π/2 − 0.01`, so `tan` reaches ~100 and a shear determinant
/// of nearly zero is inside the legal range — reachable, not a corner. The
/// recovered local coordinates are huge, but `compose` maps them straight back
/// to the right world pose, so the pair stays self-consistent and the object
/// draws where it belongs.
///
/// This gate is the fence: refusing here would need a threshold nobody can
/// justify, and it would break a scene that renders correctly today.
#[test]
fn a_nearly_singular_parent_is_recovered_not_refused() {
    // tan(skew_x) · tan(skew_y) ≈ 1 ⇒ determinant ≈ 0, but not 0.
    let parent = Transform {
        skew_x: libm::atanf(2.0),
        skew_y: libm::atanf(0.5),
        ..Transform::IDENTITY
    };
    let child = Transform::from_translation(Vec2::new(1.25, -0.75));
    let world = Transform::compose(parent, child);

    let back = Transform::inverse_compose(parent, world)
        .expect("ill-conditioned is not degenerate — it must still be recovered");
    let round = Transform::compose(parent, back);
    let err = (round.translation.x - world.translation.x)
        .abs()
        .max((round.translation.y - world.translation.y).abs());
    println!("near-singular determinant: recovered with world-space error {err:e}");
    assert!(
        err < 1e-3,
        "the recovered pose no longer composes back to the world pose ({err:e})"
    );
}

/// The borrowed-buffer walk must answer **exactly** what the plain one answers.
///
/// It exists only so a per-frame caller allocates nothing; the moment it
/// disagrees by an ulp there are two answers to "where is this entity?", which
/// is the bug the whole wave is about.
#[test]
fn the_scratch_walk_is_the_plain_walk() {
    let mut world = bevy_ecs::world::World::new();
    let root = world
        .spawn(Transform {
            translation: Vec2::new(1.5, -2.5),
            rotation: 0.7,
            scale: Vec2::new(1.25, 0.8),
            skew_x: 0.15,
            skew_y: 0.0,
        })
        .id();
    let mid = world
        .spawn((
            Transform {
                translation: Vec2::new(-3.0, 4.0),
                rotation: -1.2,
                scale: Vec2::new(2.0, 2.0),
                skew_x: 0.0,
                skew_y: 0.05,
            },
            ChildOf(root),
        ))
        .id();
    let leaf = world
        .spawn((
            Transform::from_translation(Vec2::new(0.5, 0.25)),
            ChildOf(mid),
        ))
        .id();

    let mut scratch = Vec::new();
    for e in [root, mid, leaf] {
        assert_eq!(
            parent_world_transform_into(&world, e, &mut scratch),
            parent_world_transform(&world, e),
            "the scratch walk drifted from the plain one"
        );
    }
    // Reusing a dirty buffer must not leak the previous chain into this one.
    scratch.push(Transform::from_translation(Vec2::new(999.0, 999.0)));
    assert_eq!(
        parent_world_transform_into(&world, leaf, &mut scratch),
        parent_world_transform(&world, leaf),
        "a dirty scratch buffer leaked into the answer"
    );
}
