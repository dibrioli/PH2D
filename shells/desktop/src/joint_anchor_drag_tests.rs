//! **A gesture that poses a number** — the drag math of W-J2 / W-J3.
//!
//! Split out of `joint_anchor_drag.rs` when the parameter grips arrived (LOC).

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};

/// **The nearest candidate wins, not the first one listed.** A shape
/// enumerates its corners in a fixed order, and taking the first match would
/// make the magnet's answer depend on that order rather than on the cursor.
///
/// Mutation-tested: returning on the first candidate within range makes this
/// pick `[1.0, 0.0]` instead of `[0.2, 0.0]`.
#[test]
fn the_nearest_candidate_wins() {
    let cands = [[1.0, 0.0], [0.2, 0.0], [-3.0, 0.0]];
    assert_eq!(nearest_within(&cands, [0.0, 0.0], 2.0), Some([0.2, 0.0]));
}

/// **Out of range is no snap.** The magnet has to let go, or the anchor
/// could never be placed between two candidates.
#[test]
fn nothing_within_range_does_not_snap() {
    let cands = [[5.0, 0.0], [0.0, 5.0]];
    assert_eq!(nearest_within(&cands, [0.0, 0.0], 1.0), None);
    assert!(nearest_within(&[], [0.0, 0.0], 1000.0).is_none());
}

/// **The threshold is inclusive at the boundary**, so a candidate exactly at
/// the radius still catches — a strict comparison would make the magnet's
/// edge depend on float noise.
#[test]
fn a_candidate_exactly_at_the_radius_catches() {
    assert_eq!(
        nearest_within(&[[2.0, 0.0]], [0.0, 0.0], 2.0),
        Some([2.0, 0.0])
    );
}

// ── W-J3: posing a number ────────────────────────────────────────────────────

/// A hinge with limits, at the origin, body B a bar to its right.
fn hinge(min_deg: f32, max_deg: f32) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    for (name, kind, at) in [
        ("Post", BodyKind::Static, [0.0f32, 0.0f32]),
        ("Arm", BodyKind::Dynamic, [1.0, 0.0]),
    ] {
        sim.world_mut().spawn((
            Name::new(name.to_string()),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ));
    }
    let j = sim
        .world_mut()
        .spawn((
            Name::new("Hinge".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Arm"),
                kind: JointKind::Pin,
                limits_enabled: true,
                limit_min: min_deg.to_radians(),
                limit_max: max_deg.to_radians(),
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, j)
}

fn limits_deg(sim: &SimWorld, j: Entity) -> (f32, f32) {
    let c = sim.world().get::<PhysicsJoint>(j).expect("joint");
    (c.limit_min.to_degrees(), c.limit_max.to_degrees())
}

/// **A wall stops at its sibling; it never passes it.**
///
/// `PhysicsJoint::clamped` SWAPS inverted limits — right for a typed pair (a
/// hinge with `min > max` is a weld nobody asked for), wrong for a gesture: the
/// swap hands the artist the OTHER wall mid-drag, and the hand that was widening
/// the arc starts narrowing it with nothing on screen saying why.
///
/// Mutation-tested: dropping the `.min(other)` / `.max(other)` wall lets
/// `clamped` swap, and the two walls come back EXCHANGED — this goes red on both
/// halves.
#[test]
fn a_limit_wall_stops_at_its_sibling_instead_of_swapping() {
    let (mut sim, j) = hinge(-30.0, 45.0);
    // Push the MIN wall far past the max.
    write_limit(
        &mut sim,
        j,
        PointHandleKind::LimitMin,
        90.0_f32.to_radians(),
    );
    let (lo, hi) = limits_deg(&sim, j);
    assert!(
        (lo - 45.0).abs() < 1e-3,
        "the min wall must stop AT the max (45), got {lo:.3}"
    );
    assert!(
        (hi - 45.0).abs() < 1e-3,
        "and the max wall must not have moved, got {hi:.3}"
    );

    // And the mirror: the MAX wall pushed below the min.
    let (mut sim, j) = hinge(-30.0, 45.0);
    write_limit(
        &mut sim,
        j,
        PointHandleKind::LimitMax,
        -80.0_f32.to_radians(),
    );
    let (lo, hi) = limits_deg(&sim, j);
    assert!((lo + 30.0).abs() < 1e-3, "min untouched, got {lo:.3}");
    assert!((hi + 30.0).abs() < 1e-3, "max stops at min, got {hi:.3}");
}

/// **What the drag writes is what the number row reads.**
///
/// The grip and the §12 field are two ways of asking for the same edit, so they
/// go through the same funnel (`joint_with_edit`): degrees at the boundary,
/// radians in the component, the same `clamped()`. A drag that converted its own
/// way would put a wall at 30° on the canvas and 0.52 in the field.
#[test]
fn posing_a_wall_lands_on_the_number_the_row_would_show() {
    let (mut sim, j) = hinge(-90.0, 90.0);
    for want in [-75.0_f32, -10.0, 0.0, 60.0] {
        write_limit(&mut sim, j, PointHandleKind::LimitMin, want.to_radians());
        let (lo, _) = limits_deg(&sim, j);
        assert!(
            (lo - want).abs() < 1e-3,
            "posed {want}°, the component reads {lo:.3}°"
        );
    }
}

/// **A wall dragged across the ±pi cut moves by the small amount the cursor
/// moved**, not a whole turn back.
///
/// The bearing wraps and the stored limit does not, so without the unwrap the
/// wall at 170° dragged 20° further would land at −170° — visually the same
/// place, numerically a 340° jump, and the ARC drawn between the two walls would
/// invert.
///
/// Mutation-tested: `unwrap_near` returning `raw` goes red.
#[test]
fn a_wall_dragged_past_the_cut_does_not_jump_a_whole_turn() {
    let (mut sim, j) = hinge(170.0, 200.0);
    // The cursor's bearing at 190° comes back from `atan2` as −170°.
    write_limit(
        &mut sim,
        j,
        PointHandleKind::LimitMin,
        (-170.0_f32).to_radians(),
    );
    let (lo, _) = limits_deg(&sim, j);
    assert!(
        (lo - 190.0).abs() < 1e-2,
        "the wall must continue to 190°, not jump back to −170°; got {lo:.3}°"
    );
}

/// **The ring names a different field per kind.** One geometry, two meanings —
/// the same reason `JointView.length` is a single field.
#[test]
fn the_length_ring_writes_rest_for_a_spring_and_max_for_a_rope() {
    for (kind, spring) in [(JointKind::Spring, true), (JointKind::Rope, false)] {
        let (mut sim, j) = hinge(0.0, 0.0);
        if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(j) {
            c.kind = kind;
        }
        write_length(&mut sim, j, 2.5);
        let c = *sim.world().get::<PhysicsJoint>(j).expect("joint");
        if spring {
            assert!((c.rest_length - 2.5).abs() < 1e-4, "spring rest length");
            assert!(
                (c.max_length - PhysicsJoint::default().max_length).abs() < 1e-4,
                "a spring drag must not touch the rope's field"
            );
        } else {
            assert!((c.max_length - 2.5).abs() < 1e-4, "rope max length");
            assert!(
                (c.rest_length - PhysicsJoint::default().rest_length).abs() < 1e-4,
                "a rope drag must not touch the spring's field"
            );
        }
    }
}

/// **A Pin has no ring, so a length drag on one writes nothing.** No grip is
/// ever published for it; this pins that the write refuses too, so the two
/// halves cannot drift into a state where a stale drag authors a field the
/// joint does not use.
#[test]
fn a_pin_has_no_length_to_pose() {
    let (mut sim, j) = hinge(-10.0, 10.0);
    let before = *sim.world().get::<PhysicsJoint>(j).expect("joint");
    write_length(&mut sim, j, 3.0);
    assert_eq!(*sim.world().get::<PhysicsJoint>(j).expect("joint"), before);
}

// ── W-J6b: a rail's stroke is a LENGTH, and the grips had it as an angle ──────

/// A **Slider** at the origin whose rail runs along `+X`, with a stroke.
fn rail(min_m: f32, max_m: f32) -> (SimWorld, Entity) {
    let (mut sim, j) = hinge(0.0, 0.0);
    if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(j) {
        c.kind = JointKind::Slider;
        c.limit_min = min_m;
        c.limit_max = max_m;
    }
    (sim, j)
}

/// The pair as the COMPONENT holds it — metres for a rail, so no conversion.
fn limits_m(sim: &SimWorld, j: Entity) -> (f32, f32) {
    let c = sim.world().get::<PhysicsJoint>(j).expect("joint");
    (c.limit_min, c.limit_max)
}

/// **Posing a rail's stroke writes METRES — and the bug it replaces was a
/// runaway, not a wrong number.**
///
/// `write_limit` used to convert to degrees unconditionally, while the shell's
/// `limit_in` takes a Slider's value **verbatim as metres**: dragging to 0.8 m
/// stored `45.8`, which moved the grip 45 m down the rail, which the next frame
/// re-read — Enio's *"loop sem fim"*, ending in an app that stopped answering.
///
/// So the oracle is two claims, and the second is the one that names the defect:
/// the value is the metres that were posed, **and posing the same place twice is
/// a fixed point**. A conversion that is merely wrong would satisfy neither, but
/// a conversion that is wrong *and stable* would satisfy the second alone.
///
/// Mutation: `write_limit` converting with `.to_degrees()` again — RED, 0.8 m
/// stores 45.837 and the second pose stores 2626.
#[test]
fn posing_a_rails_stroke_writes_metres_and_is_a_fixed_point() {
    let (mut sim, j) = rail(-1.0, 1.0);
    for want in [0.2_f32, 0.8, -0.55] {
        // Pose it twice from the SAME place: a unit error compounds, a correct
        // conversion lands on the same number both times.
        write_limit(&mut sim, j, PointHandleKind::LimitMax, want.max(-0.9));
        let (_, first) = limits_m(&sim, j);
        write_limit(&mut sim, j, PointHandleKind::LimitMax, want.max(-0.9));
        let (_, again) = limits_m(&sim, j);
        assert!(
            (first - want.max(-0.9)).abs() < 1e-4,
            "posed {want} m, the component reads {first}"
        );
        assert!(
            (first - again).abs() < 1e-6,
            "posing the same place twice has to be a fixed point: {first} -> {again}"
        );
    }
}

/// **A stroke is never UNWRAPPED**, because a length has no turns.
///
/// The unwrap exists so a hinge's wall dragged across the ±pi cut moves by the
/// small amount the cursor moved. Applied to a rail it would teleport an end by
/// 6.28 m every 3.14 m of travel — the same code doing the right thing for one
/// unit and something absurd for the other.
///
/// Mutation: dropping the `limits_in_metres` branch in `write_limit` — RED, the
/// end lands at −2.28 m instead of 4.0.
#[test]
fn a_rails_stroke_is_not_unwrapped_because_a_length_has_no_turns() {
    let (mut sim, j) = rail(-0.2, 0.2);
    write_limit(&mut sim, j, PointHandleKind::LimitMax, 4.0);
    let (_, hi) = limits_m(&sim, j);
    assert!(
        (hi - 4.0).abs() < 1e-4,
        "a stroke end posed at 4 m has to be 4 m, got {hi}"
    );
}

/// **And a hinge is unaffected** — the control, so the two branches above cannot
/// be satisfied by a `write_limit` that simply stopped converting.
#[test]
fn a_hinges_wall_still_speaks_degrees_at_the_boundary() {
    let (mut sim, j) = hinge(-90.0, 90.0);
    write_limit(&mut sim, j, PointHandleKind::LimitMax, 0.5);
    let c = sim.world().get::<PhysicsJoint>(j).expect("joint");
    assert!(
        (c.limit_max - 0.5).abs() < 1e-4,
        "a hinge posed at 0.5 rad stores 0.5 rad, got {}",
        c.limit_max
    );
}
