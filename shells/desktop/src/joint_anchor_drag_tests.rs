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

/// **TODO tipo que PUBLICA um anel de comprimento pode ser ESCRITO pelo arrasto.**
///
/// O gate irmão acima cravava a lista `[(Spring, …), (Rope, …)]` à mão, e foi
/// exatamente assim que o **Rod** shipou com o anel agarrável e a escrita
/// devolvendo em silêncio: o handle era publicado (o `JointView` carrega o
/// comprimento), o grip pegava, o arrasto abria — e `write_length` caía num
/// `_ => return` cujo comentário dizia *"Pin/Weld não têm anel"*, uma
/// **enumeração dos leitores** que era verdade com cinco tipos.
///
/// ⚠️ **A lista vem dos CHIPS que o painel pinta** (`INSP_JOINT_KIND`, via
/// `kind_of`), não de um vetor escrito aqui: é a mesma amarração do gate de
/// round-trip do §12, e é o que faz o sétimo tipo nascer coberto em vez de
/// nascer com o defeito que este gate acabou de pegar.
#[test]
fn every_kind_that_offers_a_length_ring_can_have_it_dragged() {
    for tag in 0..u8::try_from(ph2d_editor::ids::INSP_JOINT_KIND.len()).expect("cabe") {
        let kind = crate::render_loop::inspector_joint::kind_of(tag);
        let Some(field) = kind.length_field() else {
            continue;
        };
        let (mut sim, j) = hinge(0.0, 0.0);
        if let Some(mut c) = sim.world_mut().get_mut::<PhysicsJoint>(j) {
            c.kind = kind;
        }
        let before = *sim.world().get::<PhysicsJoint>(j).expect("joint");
        write_length(&mut sim, j, 2.5);
        let after = *sim.world().get::<PhysicsJoint>(j).expect("joint");

        let (wrote, untouched, other) = match field {
            ph2d_physics_ecs::LengthField::Rest => {
                (after.rest_length, after.max_length, before.max_length)
            }
            ph2d_physics_ecs::LengthField::Max => {
                (after.max_length, after.rest_length, before.rest_length)
            }
        };
        assert!(
            (wrote - 2.5).abs() < 1e-4,
            "o anel de {kind:?} nao escreveu o comprimento: {wrote} em vez de 2.5"
        );
        assert!(
            (untouched - other).abs() < 1e-4,
            "o arrasto de {kind:?} tocou o campo do OUTRO tipo"
        );
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

// ── W-J6c: as duas alças de um trilho estabelecem também a ROTAÇÃO ───────────

/// A rotação autorada do joint (o EIXO do trilho), radianos.
fn rail_angle(sim: &SimWorld, j: Entity) -> f32 {
    sim.world().get::<Transform>(j).expect("transform").rotation
}

/// **Arrastar uma ponta AIMA o trilho** — as duas alças, livres em x e y, dizem
/// entre si tudo que um trilho é: a reta pela âncora e a ponta é o eixo, e a
/// distância até ela é aquele fim de curso.
///
/// O oráculo tem as DUAS metades porque a alça diz duas coisas: o ângulo passa a
/// apontar para onde a ponta foi, e o curso passa a valer a distância. Um gate
/// que só olhasse o ângulo ficaria verde numa alça que gira o trilho e esquece o
/// comprimento — e vice-versa.
///
/// Mutação: `write_rail_end` não escrevendo o `Transform` — RED na metade do
/// ângulo; escrevendo `len` sem o `side` — RED na do Min.
#[test]
fn dragging_a_rail_end_aims_the_rail_and_sets_that_end() {
    // Max arrastado para (0, 2): o eixo vira +Y e o curso máximo vira 2.
    let (mut sim, j) = rail(-1.0, 1.0);
    write_rail_end(
        &mut sim,
        j,
        PointHandleKind::LimitMax,
        [0.0, 0.0],
        [0.0, 2.0],
    );
    let (lo, hi) = limits_m(&sim, j);
    assert!(
        (rail_angle(&sim, j) - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "o eixo tem de apontar para a ponta arrastada, got {} rad",
        rail_angle(&sim, j)
    );
    assert!((hi - 2.0).abs() < 1e-3, "e o curso vira 2 m, got {hi}");
    assert!(
        (lo + 1.0).abs() < 1e-6,
        "a OUTRA ponta mantém o valor dela — ela viaja com o trilho, \
         não é reescrita, got {lo}"
    );

    // Min arrastado para (0, 2): a ponta de trás está ali, então o eixo aponta
    // ao CONTRÁRIO (-Y) e o mínimo vira -2.
    let (mut sim, j) = rail(-1.0, 1.0);
    write_rail_end(
        &mut sim,
        j,
        PointHandleKind::LimitMin,
        [0.0, 0.0],
        [0.0, 2.0],
    );
    let (lo, _) = limits_m(&sim, j);
    assert!(
        (rail_angle(&sim, j) + std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "arrastar a ponta de TRÁS aponta o eixo ao contrário dela, got {} rad",
        rail_angle(&sim, j)
    );
    assert!((lo + 2.0).abs() < 1e-3, "e o mínimo vira -2 m, got {lo}");
}

/// **Uma alça largada NA âncora não envenena o eixo.**
///
/// Um ponto sobre a âncora não tem direção nenhuma dentro dele; entregá-lo ao
/// `atan2` daria um ângulo arbitrário no melhor caso e um `NaN` no pior — e um
/// `NaN` num `Transform` envenena o `GlobalTransform` da subárvore inteira.
///
/// Mutação: tirar o guarda `len > 1e-4` — RED, o eixo salta para 0 rad.
#[test]
fn a_grip_dropped_on_the_anchor_leaves_the_axis_alone() {
    let (mut sim, j) = rail(-1.0, 1.0);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(j) {
        t.rotation = 0.75;
    }
    write_rail_end(
        &mut sim,
        j,
        PointHandleKind::LimitMax,
        [0.0, 0.0],
        [0.0, 0.0],
    );
    assert!(
        (rail_angle(&sim, j) - 0.75).abs() < 1e-6,
        "o eixo tem de ficar onde estava, got {}",
        rail_angle(&sim, j)
    );
    assert!(
        rail_angle(&sim, j).is_finite(),
        "e acima de tudo tem de ser finito"
    );
}

/// **Cada alça de raio mede o raio DELA** (W-Pulley W6).
///
/// ⚠️ **Este gate nasceu de uma mutação que o arch-gate NÃO pegou.** Aquele afirma
/// que o `open_drag` e o apply chamam a porta; o corpo da porta ele não vê, e
/// trocá-lo por `w.radius` deixava os três arch-gates verdes. O defeito seria
/// mudo e caro: agarrar o aro de SAÍDA mediria o deslocamento contra o raio de
/// ENTRADA, então a alça saltaria no instante do clique pela diferença dos dois
/// raios — num tambor 0,5 → 0,125 isso é 0,375 m de salto — e o número escrito
/// sairia errado pela mesma quantia.
///
/// *Um gate que pina a CHAMADA não pina a RESPOSTA.*
#[test]
fn each_radius_handle_measures_its_own_radius() {
    let plain = ph2d_physics_ecs::PulleyWheel {
        radius: 0.5,
        ..Default::default()
    };
    let drum = ph2d_physics_ecs::PulleyWheel {
        radius: 0.5,
        radius_out: 0.125,
        ..Default::default()
    };
    assert!((super::wheel::wheel_radius_of(&drum, PointHandleKind::WheelRim) - 0.5).abs() < 1e-6);
    assert!(
        (super::wheel::wheel_radius_of(&drum, PointHandleKind::WheelRimOut) - 0.125).abs() < 1e-6
    );
    // Numa roldana COMUM o aro de saída não é oferecido; se um chegar aqui, ele
    // mede o raio sobre o qual está desenhado — nunca zero, que faria o agarre
    // nascer com um offset do tamanho do raio inteiro.
    assert!(
        (super::wheel::wheel_radius_of(&plain, PointHandleKind::WheelRimOut) - 0.5).abs() < 1e-6
    );
}
