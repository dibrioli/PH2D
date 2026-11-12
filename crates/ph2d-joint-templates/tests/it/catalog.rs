//! As leis do catálogo: o que se GRAVA não muda, e nenhuma regra pede o que a montagem não faz.

use ph2d_joint_templates::{ALL, Anchor, JointKind, Link, Side, Spot, by_id, by_name, of_kind};
use ph2d_physics_ecs::PhysicsJoint;

/// **O id de um gabarito é o que o projeto grava** (`JointAssembly::template`): nunca muda de
/// dono. Gabarito novo = linha nova no FIM, com o número seguinte.
#[test]
fn the_template_ids_are_pinned() {
    let pinned: &[(u16, JointKind, &str)] = &[
        (1, JointKind::Wheel, "Car"),
        (2, JointKind::Wheel, "Motorbike"),
        (3, JointKind::Pin, "Door"),
        (4, JointKind::Pin, "Seesaw"),
        (5, JointKind::Pin, "Pendulum"),
        (6, JointKind::Pin, "Flipper"),
        (7, JointKind::Rope, "Chain"),
        (8, JointKind::Rope, "Bridge"),
        (9, JointKind::Rope, "Hanging"),
        (10, JointKind::Slider, "Elevator"),
        (11, JointKind::Slider, "Piston"),
        (12, JointKind::Slider, "Sliding door"),
        (13, JointKind::Spring, "Trampoline"),
        (14, JointKind::Spring, "Soft platform"),
        (15, JointKind::Weld, "Glue"),
        (16, JointKind::Weld, "Breakable"),
        (17, JointKind::Rod, "Rigid pendulum"),
        (18, JointKind::Pulley, "Lift"),
    ];
    assert_eq!(ALL.len(), pinned.len(), "a template without a pinned id");
    for &(id, kind, name) in pinned {
        let t = by_id(id).unwrap_or_else(|| panic!("template {id} ({name}) is gone"));
        assert_eq!((t.kind, t.name), (kind, name), "id {id} changed owner");
        assert_eq!(by_name(kind, name).map(|t| t.id.0), Some(id));
    }
    // `0` é o `Default` do componente: «nenhum gabarito».
    assert!(by_id(0).is_none());
}

#[test]
fn every_family_but_custom_offers_a_template() {
    for kind in JointKind::ALL {
        let n = of_kind(kind).count();
        if kind == JointKind::Custom {
            assert_eq!(n, 0, "Custom is the hand-made joint");
        } else {
            assert!(n > 0, "{kind:?} has no template");
        }
    }
}

/// **Toda regra cabe no que a montagem sabe fazer.**
#[test]
fn every_rule_is_well_formed() {
    for t in ALL {
        let who = |what: &str| format!("{:?} · {}: {what}", t.kind, t.name);
        assert!(!t.joints.is_empty(), "{}", who("no joints"));
        let mut names: Vec<_> = t.roles.iter().map(|r| r.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            t.roles.len(),
            "{}",
            who("two roles share a name")
        );
        for r in t.roles {
            assert!(!r.name.is_empty(), "{}", who("unnamed role"));
            let [hx, hy] = r.guide.half();
            assert!(hx > 0.0 && hy > 0.0, "{}", who("flat guide"));
            if let Some(m) = r.many {
                assert!(m.count >= 2, "{}", who("a row of fewer than two"));
                assert!(
                    m.step != [0.0, 0.0],
                    "{}",
                    who("a row that does not advance")
                );
            }
        }
        let mut used = vec![false; t.roles.len()];
        for j in t.joints {
            let (a, b) = (
                t.role(j.a)
                    .unwrap_or_else(|| panic!("{}", who("rule names no role A"))),
                t.role(j.b)
                    .unwrap_or_else(|| panic!("{}", who("rule names no role B"))),
            );
            used[usize::from(j.a)] = true;
            used[usize::from(j.b)] = true;
            assert!(!j.name.is_empty(), "{}", who("unnamed joint"));
            assert!(
                j.group < 32,
                "{}",
                who("group does not fit the linked bitset")
            );
            // Os números da junta leem-se «B em relação a A»: o cenário só pode ser o lado A.
            assert!(!b.may_be_world, "{}", who("the scenery on side B"));
            match j.link {
                Link::Pair => assert!(
                    a.many.is_none() && b.many.is_none(),
                    "{}",
                    who("a Pair over a row")
                ),
                Link::Chain => assert!(j.a == j.b && a.many.is_some(), "{}", who("bad Chain")),
                Link::ToFirst | Link::ToLast => {
                    assert!(
                        a.many.is_none() && b.many.is_some(),
                        "{}",
                        who("bad row end")
                    );
                }
            }
            // A emenda mede duas silhuetas; o cenário não tem nenhuma.
            if j.anchor == Anchor::Seam {
                assert!(!a.may_be_world, "{}", who("a seam with the scenery"));
            }
            // Um tipo de um ponto só não tem duas pontas, e um de duas não tem um ponto só.
            match j.anchor {
                Anchor::At(_) => {
                    assert!(j.kind.shares_a_point(), "{}", who("At on a two-ended kind"))
                }
                Anchor::Ends(pa, pb) => {
                    assert!(
                        !j.kind.shares_a_point(),
                        "{}",
                        who("Ends on a one-point kind")
                    );
                    assert_eq!(
                        (pa.on, pb.on),
                        (Side::A, Side::B),
                        "{}",
                        who("ends on the wrong sides")
                    );
                }
                Anchor::Seam => {}
            }
            assert_eq!(
                j.axis.is_some(),
                matches!(j.kind, JointKind::Slider | JointKind::Wheel),
                "{}",
                who("the axis belongs to rails and wheels")
            );
        }
        assert!(used.iter().all(|u| *u), "{}", who("a role no joint uses"));
    }
}

/// **Sem diferenças, a junta do gabarito é a do tipo** — e cada diferença liga a caixa dela.
#[test]
fn the_factory_joint_is_the_kind_default_plus_the_tune() {
    let glue = by_name(JointKind::Weld, "Glue").expect("glue");
    assert_eq!(
        glue.joints[0].joint(0.0),
        PhysicsJoint::of_kind(JointKind::Weld)
    );

    let car = by_name(JointKind::Wheel, "Car").expect("car");
    let (rear, front) = (car.joints[0].joint(0.0), car.joints[1].joint(0.0));
    assert!(rear.motor_enabled && rear.limits_enabled);
    assert!(
        rear.motor_speed < 0.0,
        "negative spins towards +x, the front"
    );
    assert!(!front.motor_enabled && front.limits_enabled);
    assert_eq!(car.joints[0].group, car.joints[1].group);
    let base = PhysicsJoint::of_kind(JointKind::Wheel);
    assert_eq!(
        (rear.stiffness, rear.damping),
        (base.stiffness, base.damping)
    );

    let bridge = by_name(JointKind::Rope, "Bridge").expect("bridge");
    assert_eq!(bridge.joints[1].joint(0.02).max_length, 0.02);
    let tramp = by_name(JointKind::Spring, "Trampoline").expect("trampoline");
    let spring = tramp.joints[0].joint(0.5);
    assert!((spring.rest_length - 0.2).abs() < 1e-6, "0.3 m of preload");

    let weak = by_name(JointKind::Weld, "Breakable").expect("breakable");
    assert!(weak.joints[0].joint(0.0).break_enabled);
}

/// O ponto na peça: frações da caixa, nunca metros.
#[test]
fn a_spot_is_a_fraction_of_the_box() {
    assert_eq!(
        Spot::b(-1.0, 0.0),
        Spot {
            on: Side::B,
            u: -1.0,
            v: 0.0
        }
    );
}

/// O nome da peça de fábrica: o gabarito e o papel, sem repetir, e a fila numerada a partir de 1.
#[test]
fn a_factory_part_is_named_after_the_template_and_the_role() {
    let car = by_name(JointKind::Wheel, "Car").expect("car");
    assert_eq!(car.part_name(0, 0), "Car Body");
    assert_eq!(car.part_name(1, 0), "Car Rear Wheel");
    let door = by_name(JointKind::Pin, "Door").expect("door");
    assert_eq!(door.part_name(1, 0), "Door");
    let bridge = by_name(JointKind::Rope, "Bridge").expect("bridge");
    assert_eq!(bridge.part_name(1, 2), "Plank 3");
    assert_eq!(bridge.part_name(2, 0), "Bridge Right Post");
}
