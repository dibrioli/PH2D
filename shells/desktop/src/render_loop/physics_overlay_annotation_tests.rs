//! **A terceira metade do overlay: as ANOTAÇÕES desenhadas por cima.**
//!
//! `physics_overlay_tests` prova a geometria pura ("este círculo é redondo, em pixels de
//! tela, nesta câmera"); `physics_overlay_scene_tests` prova o CONTORNO sobre um mundo
//! real (que cor, onde, de que tamanho). Estas provam o que é desenhado EM CIMA dele: a
//! seta de uma zona de força, o frame em que ela aponta, o glifo de giro de uma zona de
//! torque — as marcas que descrevem uma propriedade que o contorno não consegue mostrar.
//!
//! Separado dos irmãos pelo cap de 600 LOC do shell (a wave do frame, W-AreaFrame, levou
//! o arquivo da cena a 615). Os helpers ficam no arquivo da geometria — um `camera()`, um
//! `window()`, um `points()` — para que as três metades não comecem a discordar sobre o
//! que é um pixel.

use super::outlines;
use super::tests::{camera, points, window};
use crate::render_loop::physics_overlay_annotations::{
    EFFECTOR_RGBA, FALLOFF_RGBA, FALLOFF_RING, TORQUE_RGBA,
};
use ph2d_physics_ecs::{BodyKind, ColliderShape};

/// **A force zone draws an arrow showing which way it blows — and keeps drawing it
/// while the clock runs** (W-Area).
///
/// The zone's outline is already magenta because it is a sensor, and a sensor that
/// merely notices things looks exactly the same as one that pushes them. The arrow is
/// the whole difference, so it is the thing the gate asserts.
///
/// ⚠️ `show_velocity` is passed **false** throughout — that is the "the clock is
/// running" state, where the launch arrow is deliberately hidden because a body's
/// authored velocity stops being true once it moves. A force is a property of the
/// AREA and never stops being true, so it must survive that flag.
#[test]
fn a_force_zone_draws_its_push_even_while_the_clock_runs() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{AreaEffector, Collider, RigidBody};

    let zone = |force: [f32; 2]| {
        let mut sim = ph2d_ecs::SimWorld::new();
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            AreaEffector { force },
        ));
        sim
    };

    // A pushing zone: outline + arrow, and the arrow is its own colour.
    let mut pushing = zone([20.0, 0.0]);
    let drawn = outlines(true, false, &mut pushing, &[], &camera(), window());
    assert_eq!(
        drawn.len(),
        2,
        "a force zone should draw its outline AND an arrow, got {} paths",
        drawn.len()
    );
    assert_eq!(
        drawn[1].1, EFFECTOR_RGBA,
        "the push arrow must not wear a colour that already means something else"
    );

    // It points the way it blows: the shaft's first two points run +X on screen.
    let pts = points(&drawn[1].0);
    assert!(
        pts[1].0 > pts[0].0 && (pts[1].1 - pts[0].1).abs() < 1e-6,
        "an arrow for a +X force should point +X on screen, got {pts:?}"
    );

    // A zone that pushes nothing draws no arrow — the outline alone.
    let mut idle = zone([0.0, 0.0]);
    assert_eq!(
        outlines(true, false, &mut idle, &[], &camera(), window()).len(),
        1,
        "a zero force must draw no arrow — an arrow of no length is a dot nobody can read"
    );

    // And the whole thing obeys the toggle, like every other piece of this chrome.
    assert!(outlines(false, false, &mut pushing, &[], &camera(), window()).is_empty());
}

/// **The arrow turns with the zone** (W-AreaFrame), and stops turning when the artist
/// pins the push to world axes.
///
/// The overlay resolves the direction through the SAME `zone_force_world_at` the solver's
/// substep asks. A second copy of the rule here would draw a wind that does not blow —
/// and the arrow is the only place a person ever reads this direction, so nothing else
/// would catch it: a screenshot is not something a gate reads.
#[test]
fn the_push_arrow_turns_with_the_zone_unless_it_is_pinned_to_world_axes() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{AreaEffector, AreaForceWorldAxes, Collider, RigidBody};

    // A zone turned a quarter turn, carrying a force along its OWN +X.
    let zone = |world_axes: bool| {
        let mut sim = ph2d_ecs::SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 1.0,
                        half_y: 1.0,
                    },
                    is_sensor: true,
                    ..Collider::default()
                },
                Transform {
                    translation: Vec2::new(0.0, 0.0),
                    rotation: std::f32::consts::FRAC_PI_2,
                    scale: Vec2::new(1.0, 1.0),
                    skew_x: 0.0,
                    skew_y: 0.0,
                },
                AreaEffector { force: [20.0, 0.0] },
            ))
            .id();
        if world_axes {
            sim.world_mut().entity_mut(e).insert(AreaForceWorldAxes);
        }
        sim
    };

    // Turned: the shaft must leave the origin along the SCREEN's vertical, not its
    // horizontal. (Screen Y grows downward, so a world +Y push draws upward — the sign
    // is not the claim here; running along Y rather than X is.)
    let mut turned = zone(false);
    let drawn = outlines(true, false, &mut turned, &[], &camera(), window());
    let pts = points(&drawn[1].0);
    assert!(
        (pts[1].0 - pts[0].0).abs() < 1e-4 && (pts[1].1 - pts[0].1).abs() > 1.0,
        "a zone turned a quarter turn must draw its arrow along the screen's vertical, \
         got {pts:?} — the overlay is not resolving the force through the zone's frame"
    );

    // Pinned to world axes: the same zone, the same pose, and the arrow is back on +X.
    let mut pinned = zone(true);
    let drawn = outlines(true, false, &mut pinned, &[], &camera(), window());
    let pts = points(&drawn[1].0);
    assert!(
        pts[1].0 > pts[0].0 && (pts[1].1 - pts[0].1).abs() < 1e-4,
        "pinned to world axes the arrow must stay on +X whatever the pose, got {pts:?}"
    );
}

/// **A torque zone draws a spin glyph showing which way it turns — and keeps drawing it
/// while the clock runs** (W-AreaTorque).
///
/// The rotational sibling of the force-arrow gate. A pure whirlpool carries no force
/// arrow, so without this glyph a spin zone would be an invisible property — a magenta
/// box no different from a sensor that merely notices things. The SIGN is the direction,
/// so the two handednesses must draw DIFFERENT shapes: a glyph that ignored the sign
/// would tell the artist a clockwise zone spins counter-clockwise.
#[test]
fn a_torque_zone_draws_its_spin_even_while_the_clock_runs() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{AreaTorque, Collider, RigidBody};

    let zone = |torque: f32| {
        let mut sim = ph2d_ecs::SimWorld::new();
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
            AreaTorque(torque),
        ));
        sim
    };

    // A spinning zone: outline + glyph, and the glyph is its own colour.
    let mut ccw = zone(8.0);
    let drawn = outlines(true, false, &mut ccw, &[], &camera(), window());
    assert_eq!(
        drawn.len(),
        2,
        "a torque zone should draw its outline AND a spin glyph, got {} paths",
        drawn.len()
    );
    assert_eq!(
        drawn[1].1, TORQUE_RGBA,
        "the spin glyph must not wear a colour that already means something else"
    );

    // The sign is the direction: the +torque glyph and the -torque glyph must be
    // DIFFERENT shapes (opposite sweeps), not the same arc drawn twice. Mirror the sign
    // and the arc points move.
    let mut cw = zone(-8.0);
    let cw_drawn = outlines(true, false, &mut cw, &[], &camera(), window());
    let (a, b) = (points(&drawn[1].0), points(&cw_drawn[1].0));
    assert!(
        a.iter()
            .zip(b.iter())
            .any(|(p, q)| (p.0 - q.0).abs() > 1.0 || (p.1 - q.1).abs() > 1.0),
        "a +torque and a -torque glyph must differ (the sign is the spin direction), but \
         they drew the same shape"
    );

    // A zone that spins nothing draws no glyph — the outline alone.
    let mut idle = zone(0.0);
    assert_eq!(
        outlines(true, false, &mut idle, &[], &camera(), window()).len(),
        1,
        "a zero torque must draw no glyph — a spin of no strength is not a whirlpool"
    );

    // And it obeys the toggle, like every other piece of this chrome.
    assert!(outlines(false, false, &mut ccw, &[], &camera(), window()).is_empty());
}

/// **Uma zona com falloff desenha o anel de meio caminho — e só quando há o que atenuar**
/// (W-AreaFalloff).
///
/// O falloff era o único número do modelo de área sem marca nenhuma na tela: a seta
/// continua do mesmo tamanho (ela desenha a força AUTORADA, que é a do centro), então uma
/// rajada e um bloco de vento uniforme ficavam idênticos até alguém rodar a simulação e
/// reparar que os corpos se movem diferente.
///
/// ⚠️ O anel é a curva de nível EXATA porque a régua é invariante sob escala: `t = 0.5` é
/// a silhueta encolhida à metade. O gate mede isso — a caixa do anel tem de ser metade da
/// do contorno —, e não apenas "há mais um path".
#[test]
fn a_zone_with_falloff_draws_the_half_way_ring_and_only_when_it_pushes() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{AreaEffector, AreaFalloff, Collider, RigidBody};

    // `force` ligado ou não, `falloff` ligado ou não — as quatro combinações.
    let zone = |force: bool, falloff: f32| {
        let mut sim = ph2d_ecs::SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 2.0,
                        half_y: 1.0,
                    },
                    is_sensor: true,
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        if force {
            sim.world_mut()
                .entity_mut(e)
                .insert(AreaEffector { force: [5.0, 0.0] });
        }
        if falloff > 0.0 {
            sim.world_mut().entity_mut(e).insert(AreaFalloff(falloff));
        }
        sim
    };
    let span = |path: &ph2d_vector::BezPath| {
        let pts = points(path);
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for (x, _) in &pts {
            lo = lo.min(*x);
            hi = hi.max(*x);
        }
        hi - lo
    };

    // Sem falloff: contorno + seta, e nada mais.
    let mut plain = zone(true, 0.0);
    let drawn = outlines(true, false, &mut plain, &[], &camera(), window());
    assert_eq!(
        drawn.len(),
        2,
        "uma zona sem falloff desenha contorno + seta e nada mais, saíram {} paths",
        drawn.len()
    );

    // Com falloff: mais um path, na cor do falloff, com METADE da largura do contorno.
    let mut fading = zone(true, 1.0);
    let drawn = outlines(true, false, &mut fading, &[], &camera(), window());
    assert_eq!(
        drawn.len(),
        3,
        "uma zona que desvanece tem de desenhar o anel de meio caminho, saíram {} paths",
        drawn.len()
    );
    let ring = drawn
        .iter()
        .find(|(_, rgba)| *rgba == FALLOFF_RGBA)
        .expect("o anel do falloff tem de ter cor própria (o laranja apagado da força)");
    let outline = &drawn[0].0;
    let ratio = span(&ring.0) / span(outline);
    assert!(
        (ratio - f64::from(FALLOFF_RING)).abs() < 0.02,
        "o anel tem de ser a silhueta encolhida a {FALLOFF_RING} — mediu {ratio} da \
         largura do contorno, então não é a curva de nível que a régua descreve"
    );

    // ⚠️ E NÃO é desenhado quando não há o que atenuar: um falloff sobre uma zona que não
    // empurra nem gira descreveria o desvanecimento de nada.
    let mut inert = zone(false, 1.0);
    let drawn = outlines(true, false, &mut inert, &[], &camera(), window());
    assert!(
        drawn.iter().all(|(_, rgba)| *rgba != FALLOFF_RGBA),
        "o anel apareceu numa zona que não empurra nada"
    );
}
