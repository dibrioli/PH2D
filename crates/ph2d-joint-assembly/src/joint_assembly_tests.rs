//! **O gabarito monta uma máquina que FUNCIONA** — o oráculo é a simulação, não os componentes.
//!
//! As peças de cada cena saem das formas-guia do próprio gabarito ([`parts_of`]): é a mesma
//! geometria que a aba desenha e que «Make the parts» cria, então um número de fábrica que só
//! servisse a outras peças reprova aqui.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, TransformPropagationState, WorklistBuf};
use ph2d_joint_templates::{Guide, JointKind, Link, Template, by_name};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InitialVelocity, JointAssembly, PhysicsBridge, PhysicsJoint,
    RigidBody,
};

use super::{Assembled, PlaceError, assemble, joints_of, new_assembly, place_part};

/// Um papel e a peça dele (`None` = o cenário).
type Parts = Vec<(u16, Option<Entity>)>;

const TICKS_5S: u64 = 300;

fn body(sim: &mut SimWorld, name: &str, guide: Guide, at: [f32; 2], kind: BodyKind) -> Entity {
    let shape = match guide {
        Guide::Box { half } => ColliderShape::Cuboid {
            half_x: half[0],
            half_y: half[1],
        },
        Guide::Disc { radius } => ColliderShape::Ball { radius },
    };
    let w = sim.world_mut();
    let e = w
        .spawn((
            Name::new(name.to_owned()),
            RigidBody { kind },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id();
    // ADR-0199: a forma é a filha «Collision Shape», como o editor a cria.
    let f = ph2d_physics_ecs::nasce_com_forma(w, e, shape);
    if let Some(mut c) = w.get_mut::<Collider>(f) {
        // O pneu agarra (o atrito das rodas do smoke 57).
        c.friction = 1.0;
    }
    e
}

fn ground(sim: &mut SimWorld) {
    let floor = Guide::Box { half: [40.0, 0.5] };
    body(sim, "Ground", floor, [0.0, -0.5], BodyKind::Static);
}

/// As peças de fábrica do gabarito, no lugar que o desenho lhes dá. `scenery_is_world` deixa os
/// papéis que o cenário pode fazer SEM peça.
fn parts_of(sim: &mut SimWorld, t: &Template, scenery_is_world: bool) -> Parts {
    let mut out = Parts::new();
    for (i, role) in t.roles.iter().enumerate() {
        let i = u16::try_from(i).expect("few roles");
        if role.may_be_world && scenery_is_world {
            out.push((i, None));
            continue;
        }
        let (count, step) = role.many.map_or((1, [0.0, 0.0]), |m| (m.count, m.step));
        for n in 0..count {
            let at = [
                role.at[0] + step[0] * f32::from(n),
                role.at[1] + step[1] * f32::from(n),
            ];
            let kind = if role.fixed {
                BodyKind::Static
            } else {
                BodyKind::Dynamic
            };
            let name = if count > 1 {
                format!("{} {}", role.name, n + 1)
            } else {
                role.name.to_owned()
            };
            out.push((i, Some(body(sim, &name, role.guide, at, kind))));
        }
    }
    out
}

fn built(kind: JointKind, name: &str, scenery_is_world: bool) -> (SimWorld, Parts, Assembled) {
    let t = by_name(kind, name).expect("the template exists");
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let parts = parts_of(&mut sim, t, scenery_is_world);
    let made = assemble(&mut sim, t, &parts).expect("the factory parts assemble");
    (sim, parts, made)
}

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge, from: u64, ticks: u64) {
    for t in from + 1..=from + ticks {
        bridge.dispatch(sim, false, t);
    }
}

fn pose(sim: &SimWorld, e: Entity) -> Transform {
    ph2d_ecs::world_transform(sim.world(), e).expect("the part exists")
}

fn part(parts: &Parts, role: u16) -> Entity {
    parts
        .iter()
        .find(|(r, _)| *r == role)
        .and_then(|(_, e)| *e)
        .expect("the role has a part")
}

/// Quantas juntas o gabarito pede para estas peças.
fn joints_wanted(t: &Template, parts: &Parts) -> usize {
    let count = |role: u16| parts.iter().filter(|(r, _)| *r == role).count();
    t.joints
        .iter()
        .map(|r| match r.link {
            Link::Chain => count(r.a).saturating_sub(1),
            _ => usize::from(count(r.a) > 0 && count(r.b) > 0),
        })
        .sum()
}

/// **Todo gabarito monta as próprias peças e a máquina fica inteira** — com as peças de cenário
/// feitas de objetos e com elas por conta do mundo.
#[test]
fn every_template_assembles_its_factory_parts_and_holds() {
    for t in ph2d_joint_templates::ALL {
        for scenery_is_world in [false, true] {
            let label = format!("{:?} · {} (world: {scenery_is_world})", t.kind, t.name);
            let mut sim = SimWorld::new();
            ground(&mut sim);
            let parts = parts_of(&mut sim, t, scenery_is_world);
            let made = assemble(&mut sim, t, &parts).unwrap_or_else(|e| panic!("{label}: {e:?}"));
            assert_eq!(
                made.joints.len(),
                joints_wanted(t, &parts),
                "{label}: joints made"
            );
            let asm_id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
                .expect("the assembly has an identity")
                .0;
            for (e, j) in joints_of(sim.world_mut(), asm_id) {
                let rule = t.rule(j.role).expect("the joint names a rule");
                assert_eq!(j.kind, rule.kind, "{label}: kind of {}", rule.name);
                assert!(made.joints.contains(&e), "{label}: a joint nobody reported");
            }
            let mut bridge = PhysicsBridge::new();
            run(&mut sim, &mut bridge, 0, TICKS_5S);
            for &(_, e) in &parts {
                let Some(e) = e else { continue };
                let p = pose(&sim, e);
                assert!(
                    p.translation.x.is_finite() && p.translation.y.is_finite(),
                    "{label}: a part flew to NaN"
                );
                // Nada atravessa o chão nem sai da cena: a máquina continua montada.
                assert!(
                    p.translation.y > -0.2 && p.translation.length() < 30.0,
                    "{label}: a part ended at {:?}",
                    p.translation
                );
            }
        }
    }
}

/// **O carro de três peças anda para a frente, apoiado na suspensão.**
#[test]
fn the_car_drives_forward_on_its_suspension() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    assert_eq!(made.joints.len(), 2);
    let chassis = part(&parts, 0);
    let start = pose(&sim, chassis).translation;
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 0, TICKS_5S);
    let end = pose(&sim, chassis).translation;
    assert!(
        end.x - start.x > 3.0,
        "the car moved {:.2} m in 5 s",
        end.x - start.x
    );
    // Nem sentado no batente (o curso a subir é 0,10 m) nem a saltar.
    assert!(
        (end.y - start.y).abs() < 0.10,
        "the body rides {:.3} m off its drawing height",
        end.y - start.y
    );
    assert!(
        pose(&sim, chassis).rotation.abs() < 0.2,
        "the car tipped over"
    );
}

/// **Só a roda de trás puxa; as duas têm o curso do gabarito e dizem o papel delas.**
#[test]
fn the_car_joints_carry_their_role_and_factory_numbers() {
    let (mut sim, _, made) = built(JointKind::Wheel, "Car", false);
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
        .expect("id")
        .0;
    let joints = joints_of(sim.world_mut(), id);
    let (rear, front) = (joints[0].1, joints[1].1);
    assert_eq!((rear.role, front.role), (0, 1));
    assert!(rear.motor_enabled && !front.motor_enabled);
    for j in [rear, front] {
        assert_eq!(j.kind, JointKind::Wheel);
        assert!(j.limits_enabled);
        assert_eq!([j.limit_min, j.limit_max], [-0.15, 0.10]);
        assert!(j.anchored, "the anchor is the wheel's centre, not a seed");
        assert_eq!(j.local_b, [0.0, 0.0]);
    }
    let name = sim.world().get::<Name>(made.assembly).expect("named");
    assert_eq!(name.as_str(), "Car 1");
}

/// **A porta gira e para no limite** — empurrada com força, nunca passa do quarto de volta.
#[test]
fn the_door_swings_and_stops_at_its_limit() {
    for scenery_is_world in [false, true] {
        let (mut sim, parts, _) = built(JointKind::Pin, "Door", scenery_is_world);
        let door = part(&parts, 1);
        sim.world_mut().entity_mut(door).insert(InitialVelocity {
            linvel: [0.0, 0.0],
            angvel: 12.0,
        });
        let mut bridge = PhysicsBridge::new();
        let mut widest = 0.0f32;
        for t in 0..TICKS_5S {
            run(&mut sim, &mut bridge, t, 1);
            widest = widest.max(pose(&sim, door).rotation.abs());
        }
        let limit = std::f32::consts::FRAC_PI_2;
        assert!(
            widest > 1.0,
            "the door barely moved ({widest:.2} rad, world: {scenery_is_world})"
        );
        assert!(
            widest < limit + 0.1,
            "the door swung {widest:.2} rad past its {limit:.2} limit (world: {scenery_is_world})"
        );
    }
}

/// **A ponte de N tábuas aguenta** — sozinha e com uma caixa de 10 kg no meio.
#[test]
fn the_bridge_holds_itself_and_a_load() {
    for scenery_is_world in [false, true] {
        let (mut sim, parts, made) = built(JointKind::Rope, "Bridge", scenery_is_world);
        assert_eq!(made.joints.len(), 9, "8 planks ⇒ 7 links + the two ends");
        let crate_half = 0.25;
        let load = sim
            .world_mut()
            .spawn((
                Name::new("Crate"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Transform::from_translation(Vec2::new(0.0, 1.5)),
            ))
            .id();
        let w = sim.world_mut();
        let shape = ph2d_physics_ecs::nasce_com_forma(
            w,
            load,
            ColliderShape::Cuboid {
                half_x: crate_half,
                half_y: crate_half,
            },
        );
        // 0,25 m² × 40 = 10 kg.
        if let Some(mut c) = w.get_mut::<Collider>(shape) {
            c.density = 40.0;
        }
        let mut bridge = PhysicsBridge::new();
        run(&mut sim, &mut bridge, 0, TICKS_5S);
        let lowest = parts
            .iter()
            .filter(|(r, _)| *r == 1)
            .filter_map(|(_, e)| *e)
            .map(|e| pose(&sim, e).translation.y)
            .fold(f32::INFINITY, f32::min);
        // As pontas estão a 1,0 m. Resolvida como uma corrente (W2), a flecha com 10 kg em repouso fica
        // em centímetros: medido 0,99 m na tábua do meio (doc 02 §«A ponte sob carga»).
        assert!(
            lowest > 0.9,
            "the deck sagged to y = {lowest:.2} (world: {scenery_is_world})"
        );
        let crate_y = pose(&sim, load).translation.y;
        assert!(
            crate_y > lowest,
            "the crate fell through the deck (y = {crate_y:.2}, world: {scenery_is_world})"
        );
    }
}

/// **A ponta direita da ponte segue a ÚLTIMA tábua**, entre a ordem que for: o poste antes das
/// tábuas dá as mesmas juntas que o poste depois.
#[test]
fn the_far_end_follows_the_last_plank_whatever_the_order() {
    let t = by_name(JointKind::Rope, "Bridge").expect("bridge");
    let pairs = |post_first: bool| {
        let mut sim = SimWorld::new();
        let mut parts = parts_of(&mut sim, t, false);
        if post_first {
            let post = parts.pop().expect("the right post is the last role");
            parts.insert(0, post);
        }
        let made = assemble(&mut sim, t, &parts).expect("assembles");
        let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
            .expect("id")
            .0;
        let name_of = |sim: &SimWorld, id: u64| {
            ph2d_ecs::entity_with_stable_id(sim.world(), ph2d_ecs::StableId(id))
                .and_then(|e| sim.world().get::<Name>(e))
                .map(|n| n.as_str().to_owned())
        };
        let joints = joints_of(sim.world_mut(), id);
        assert_eq!(
            joints.len(),
            made.joints.len(),
            "no stale joint left behind"
        );
        let mut out: Vec<_> = joints
            .iter()
            .map(|(_, j)| (j.role, name_of(&sim, j.body_a), name_of(&sim, j.body_b)))
            .collect();
        out.sort();
        out
    };
    let after = pairs(false);
    assert_eq!(after.len(), 9);
    assert_eq!(pairs(true), after);
    assert!(
        after.contains(&(2, Some("Right post".to_owned()), Some("Plank 8".to_owned()))),
        "{after:?}"
    );
}

/// **O elevador sobe um andar e para** — com o trilho feito de objeto e com ele no cenário.
#[test]
fn the_elevator_rises_one_storey_and_stops() {
    for scenery_is_world in [false, true] {
        let (mut sim, parts, _) = built(JointKind::Slider, "Elevator", scenery_is_world);
        let platform = part(&parts, 1);
        let start = pose(&sim, platform).translation;
        let mut bridge = PhysicsBridge::new();
        run(&mut sim, &mut bridge, 0, TICKS_5S);
        let end = pose(&sim, platform).translation;
        assert!(
            (end.y - start.y - 3.0).abs() < 0.1,
            "the platform rose {:.2} m (world: {scenery_is_world})",
            end.y - start.y
        );
        assert!((end.x - start.x).abs() < 0.05, "the platform left its rail");
    }
}

/// **Salvar e abrir devolve a mesma montagem**, com as juntas ainda a apontar para ela.
#[test]
fn an_assembly_round_trips_through_the_snapshot() {
    let reg = crate::test_registry();
    let (mut sim, _, made) = built(JointKind::Wheel, "Car", false);
    let before = sim
        .world()
        .get::<JointAssembly>(made.assembly)
        .expect("assembly")
        .clone();
    assert_eq!(before.parts.len(), 3);
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
        .expect("id")
        .0;
    let joints_before: Vec<PhysicsJoint> = joints_of(sim.world_mut(), id)
        .into_iter()
        .map(|(_, j)| j)
        .collect();

    let mut snap = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut snap)
        .expect("snapshot");
    let mut back = SimWorld::new();
    snapshot_to_world(back.world_mut(), &snap, &reg).expect("load");

    let mut q = back.world_mut().query::<(&Name, &JointAssembly)>();
    let found: Vec<_> = q
        .iter(back.world())
        .map(|(n, a)| (n.as_str().to_owned(), a.clone()))
        .collect();
    assert_eq!(found, vec![("Car 1".to_owned(), before)]);
    let joints_after: Vec<PhysicsJoint> = joints_of(back.world_mut(), id)
        .into_iter()
        .map(|(_, j)| j)
        .collect();
    assert_eq!(joints_after, joints_before);
}

/// **Cada recusa tem nome, e uma montagem recusada não deixa nada para trás.**
#[test]
fn a_refused_part_is_named_and_leaves_nothing_behind() {
    let car = by_name(JointKind::Wheel, "Car").expect("car");
    let mut sim = SimWorld::new();
    let parts = parts_of(&mut sim, car, false);
    let (chassis, wheel) = (part(&parts, 0), part(&parts, 1));
    let asm = new_assembly(&mut sim, car);
    assert_eq!(place_part(&mut sim, asm, 0, Some(chassis)), Ok(vec![]));
    assert_eq!(
        place_part(&mut sim, asm, 0, Some(wheel)),
        Err(PlaceError::RoleTaken)
    );
    assert_eq!(
        place_part(&mut sim, asm, 1, Some(chassis)),
        Err(PlaceError::AlreadyPlaced)
    );
    assert_eq!(
        place_part(&mut sim, asm, 1, None),
        Err(PlaceError::WorldNotAllowed)
    );
    assert_eq!(
        place_part(&mut sim, asm, 9, Some(wheel)),
        Err(PlaceError::UnknownRole)
    );
    assert_eq!(
        place_part(&mut sim, chassis, 1, Some(wheel)),
        Err(PlaceError::NotAnAssembly)
    );
    // A roda entra e a junta dela nasce nesse gesto.
    let made = place_part(&mut sim, asm, 1, Some(wheel)).expect("the wheel fits");
    assert_eq!(made.len(), 1);

    let count = |sim: &mut SimWorld| sim.world_mut().query::<Entity>().iter(sim.world()).count();
    let n = count(&mut sim);
    let refused = assemble(&mut sim, car, &[(0, Some(chassis)), (1, Some(chassis))]);
    assert_eq!(refused, Err(PlaceError::AlreadyPlaced));
    assert_eq!(
        count(&mut sim),
        n,
        "the refused assembly left objects behind"
    );
}

/// **A SONDA dos números de fábrica** — a tabela de `docs/JointEditor/02_gabaritos_medidos.md`.
/// Instrumento, não gate: `cargo-test-narrow.sh ph2d-app-physics probe_factory -- --ignored --nocapture`.
#[test]
#[ignore = "instrumento: imprime a tabela do doc 02"]
fn probe_factory_numbers() {
    for t in ph2d_joint_templates::ALL {
        for scenery_is_world in [false, true] {
            let mut sim = SimWorld::new();
            ground(&mut sim);
            let parts = parts_of(&mut sim, t, scenery_is_world);
            assemble(&mut sim, t, &parts).expect("assembles");
            let moving: Vec<(Entity, Transform)> = parts
                .iter()
                .filter_map(|(_, e)| *e)
                .filter(|&e| {
                    sim.world()
                        .get::<RigidBody>(e)
                        .is_some_and(|b| b.kind == BodyKind::Dynamic)
                })
                .map(|e| (e, pose(&sim, e)))
                .collect();
            let mut bridge = PhysicsBridge::new();
            run(&mut sim, &mut bridge, 0, TICKS_5S);
            for (e, p0) in moving {
                let p = pose(&sim, e);
                let name = sim.world().get::<Name>(e).map(|n| n.as_str().to_owned());
                println!(
                    "{:<8} {:<15} world={:<5} {:<14} dx={:+.3} dy={:+.3} rot={:+.3}",
                    format!("{:?}", t.kind),
                    t.name,
                    scenery_is_world,
                    name.unwrap_or_default(),
                    p.translation.x - p0.translation.x,
                    p.translation.y - p0.translation.y,
                    p.rotation - p0.rotation,
                );
            }
        }
    }
}

fn snap(sim: &mut SimWorld) -> WorldSnapshot {
    let reg = crate::test_registry();
    let mut s = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut worklist, &reg, &mut s).expect("snapshot");
    s
}

/// **Um gesto, um passo de desfazer** — a montagem inteira (o objeto «Car 1» e as duas juntas)
/// nasce numa chamada só, então o retrato de antes dela (o que o Ctrl+Z restaura) não tem NADA
/// dela, e as peças voltam intactas.
#[test]
fn restoring_the_snapshot_before_the_gesture_removes_the_whole_assembly() {
    let car = by_name(JointKind::Wheel, "Car").expect("car");
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let parts = parts_of(&mut sim, car, false);
    let before = snap(&mut sim);
    assemble(&mut sim, car, &parts).expect("assembles");
    let after = snap(&mut sim);

    let reg = crate::test_registry();
    let count = |s: &WorldSnapshot| {
        let mut w = SimWorld::new();
        snapshot_to_world(w.world_mut(), s, &reg).expect("load");
        let joints = w
            .world_mut()
            .query::<&PhysicsJoint>()
            .iter(w.world())
            .count();
        let asms = w
            .world_mut()
            .query::<&JointAssembly>()
            .iter(w.world())
            .count();
        let bodies = w.world_mut().query::<&RigidBody>().iter(w.world()).count();
        (asms, joints, bodies)
    };
    assert_eq!(
        count(&before),
        (0, 0, 4),
        "ground + body + two wheels, nothing else"
    );
    assert_eq!(
        count(&after),
        (1, 2, 4),
        "the gesture added one assembly and two joints"
    );
}

/// ⭐ **O cartão Joint de uma roda do Car** (Joint Editor W2): o título pelo papel, o «Same as» das
/// irmãs, a afinação que se espalha no grupo ligado (e o Drive que não), e o bloco «Joints» do chassi.
/// **Mutações que sangram:** `edit_is_shared` a devolver `false` · `MotorEnabled` partilhado ·
/// `set_linked` a não limpar o bit · `body_joints` sem o lado A ou fora de ordem.
#[test]
fn the_car_wheel_card_titles_links_and_lists_its_joints() {
    use crate::joint_assembly_card::{body_joints, card_context, set_linked, with_linked};
    use ph2d_editor_core::JointFieldEdit as E;
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly)
        .expect("id")
        .0;
    let joints = joints_of(sim.world_mut(), id);
    let ((rear_e, _), (front_e, front)) = (joints[0], joints[1]);
    let ctx = card_context(sim.world(), &front);
    assert_eq!(
        (ctx.title.as_str(), ctx.same_as.as_str()),
        ("Front wheel · Car 1", "Rear wheel")
    );
    let _ = set_linked(sim.world_mut(), front_e, true);
    let (rear_b, front_b) = (rear_e.to_bits(), front_e.to_bits());
    let spread = with_linked(&mut sim, &[(front_b, E::Stiffness(9.0))]);
    assert_eq!(
        spread,
        [(front_b, E::Stiffness(9.0)), (rear_b, E::Stiffness(9.0))]
    );
    let drive = with_linked(&mut sim, &[(front_b, E::MotorEnabled(true))]);
    assert_eq!(
        drive,
        [(front_b, E::MotorEnabled(true))],
        "the Drive spread"
    );
    assert!(set_linked(sim.world_mut(), front_e, false));
    let alone = with_linked(&mut sim, &[(front_b, E::Stiffness(9.0))]);
    assert_eq!(
        alone,
        [(front_b, E::Stiffness(9.0))],
        "an unlinked group spread"
    );
    let chassis = part(&parts, 0);
    let listed = body_joints(sim.world_mut(), Some(chassis.to_bits()));
    let labels: Vec<_> = listed
        .iter()
        .map(|j| (j.joint_bits, j.label.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            (rear_b, "Rear wheel · Car 1"),
            (front_b, "Front wheel · Car 1")
        ]
    );
    assert_eq!(body_joints(sim.world_mut(), None), []);
}

fn parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    sim.world().get::<ph2d_ecs::ChildOf>(e).map(ph2d_ecs::ChildOf::parent)
}

fn by_sid(sim: &SimWorld, id: u64) -> Entity {
    ph2d_ecs::entity_with_stable_id(sim.world(), ph2d_ecs::StableId(id)).expect("by id")
}

/// ⭐ **A montagem é a PASTA do conjunto** (dono 15/11): as peças moram dentro de «Car 1», cada
/// junta dentro do corpo A dela, e nada saiu do lugar ao entrar na pasta.
/// **Mutações que sangram:** `place_part` sem o `adopt` · `joint_home` a escolher o corpo B ·
/// `adopt_keeping_world` sem o `inverse_compose`.
#[test]
fn the_assembly_is_the_folder_of_its_parts_and_each_joint_lives_in_body_a() {
    let car = by_name(JointKind::Wheel, "Car").expect("car");
    let mut sim = SimWorld::new();
    ground(&mut sim);
    let parts = parts_of(&mut sim, car, false);
    let at: Vec<Transform> = parts.iter().map(|(_, e)| pose(&sim, e.expect("part"))).collect();
    let made = assemble(&mut sim, car, &parts).expect("assembles");
    for ((_, e), was) in parts.iter().zip(&at) {
        let e = e.expect("part");
        assert_eq!(parent(&sim, e), Some(made.assembly), "the part is inside the assembly");
        let now = pose(&sim, e);
        assert!(
            (now.translation - was.translation).length() < 1e-5,
            "entering the folder moved nothing: {was:?} → {now:?}"
        );
    }
    assert_eq!(made.joints.len(), 2);
    for &j in &made.joints {
        let pj = *sim.world().get::<PhysicsJoint>(j).expect("joint");
        assert_eq!(parent(&sim, j), Some(by_sid(&sim, pj.body_a)), "the joint lives in body A");
    }
}

/// **Apagar a montagem apaga o carro inteiro** (dono 15/11) — a pasta leva as peças, e as juntas
/// vão dentro delas.
#[test]
fn deleting_the_assembly_deletes_the_whole_machine() {
    let (mut sim, _parts, made) = built(JointKind::Wheel, "Car", false);
    sim.world_mut().despawn(made.assembly);
    let w = sim.world_mut();
    assert_eq!(w.query::<&PhysicsJoint>().iter(w).count(), 0, "no joint left");
    assert_eq!(w.query::<&RigidBody>().iter(w).count(), 1, "only the ground is left");
}

/// **Tirar uma peça da pasta não desfaz as juntas** (dono 15/11): a junta vai junto com o corpo A,
/// e a montagem continua a contá-la.
#[test]
fn moving_a_part_out_of_the_folder_keeps_its_joints() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    assert!(ph2d_physics_ecs::adopt_keeping_world(sim.world_mut(), chassis, None));
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly).expect("id").0;
    let left = joints_of(sim.world_mut(), id);
    assert_eq!(left.len(), 2, "both wheels still hang from the chassis");
    for (j, _) in left {
        assert_eq!(parent(&sim, j), Some(chassis));
    }
}

/// **Mover a pasta leva o conjunto inteiro, e as juntas seguem as peças** — o pivô de cada junta
/// continua no ponto do corpo A, lido pela porta de MUNDO.
#[test]
fn moving_the_folder_moves_the_machine_and_its_pivots() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    let before: Vec<[f32; 2]> = made
        .joints
        .iter()
        .map(|&j| {
            let t = ph2d_physics_ecs::joint_world_pose(sim.world(), j).expect("pose");
            [t.translation.x, t.translation.y]
        })
        .collect();
    let was = pose(&sim, chassis).translation;
    sim.world_mut()
        .get_mut::<Transform>(made.assembly)
        .expect("folder")
        .translation += Vec2::new(5.0, 2.0);
    let moved = pose(&sim, chassis).translation - was;
    assert!((moved - Vec2::new(5.0, 2.0)).length() < 1e-5, "the chassis moved with the folder");
    for (&j, b) in made.joints.iter().zip(&before) {
        let t = ph2d_physics_ecs::joint_world_pose(sim.world(), j).expect("pose");
        assert!((t.translation.x - b[0] - 5.0).abs() < 1e-5);
        assert!((t.translation.y - b[1] - 2.0).abs() < 1e-5);
    }
}

/// **Toda junta de todo gabarito mora na casa certa**: dentro do corpo A, ou — num pino de
/// cenário, cujo prego não pode andar com a peça — dentro da pasta da montagem.
#[test]
fn every_joint_of_every_template_lives_in_its_home() {
    let mut pins = 0;
    for t in ph2d_joint_templates::ALL {
        for scenery_is_world in [false, true] {
            let label = format!("{} (world: {scenery_is_world})", t.name);
            let mut sim = SimWorld::new();
            let parts = parts_of(&mut sim, t, scenery_is_world);
            let made = assemble(&mut sim, t, &parts).unwrap_or_else(|e| panic!("{label}: {e:?}"));
            let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly).expect("id").0;
            for (j, pj) in joints_of(sim.world_mut(), id) {
                let pin = pj.body_b == 0
                    || sim.world().get::<ph2d_physics_ecs::JointWorldAnchor>(j).is_some();
                let home = if pin { made.assembly } else { by_sid(&sim, pj.body_a) };
                pins += usize::from(pin);
                assert_eq!(parent(&sim, j), Some(home), "{label}: home of a joint");
            }
        }
    }
    assert!(pins > 0, "the catalogue has scenery pins to test");
}

/// **Desmontar devolve as peças à raiz, no mesmo lugar** — a aba «Remove» não apaga o que o
/// artista desenhou.
#[test]
fn dissolving_puts_the_parts_back_at_the_root_where_they_were() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let at: Vec<Transform> = parts.iter().map(|(_, e)| pose(&sim, e.expect("p"))).collect();
    super::dissolve(&mut sim, made.assembly);
    for ((_, e), was) in parts.iter().zip(&at) {
        let e = e.expect("p");
        assert_eq!(parent(&sim, e), None, "back at the root");
        assert!((pose(&sim, e).translation - was.translation).length() < 1e-5);
    }
    let w = sim.world_mut();
    assert_eq!(w.query::<&PhysicsJoint>().iter(w).count(), 0);
}

/// **Duplicar só o chassi não engorda a montagem** — as juntas filhas vão na cópia, presas às rodas
/// de antes (a lei da cópia: o que ficou de fora continua o mesmo alvo), mas como juntas AVULSAS:
/// «Car 1» continua com as duas dela.
#[test]
fn duplicating_only_the_chassis_leaves_the_assembly_with_its_own_joints() {
    let (mut sim, parts, made) = built(JointKind::Wheel, "Car", false);
    let chassis = part(&parts, 0);
    let reg = crate::test_registry();
    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &reg, chassis, Some(made.assembly))
        .expect("copy");
    ph2d_physics_ecs::remap_joint_refs(sim.world_mut(), &copy.copies(), &copy.stable_ids);
    let id = ph2d_ecs::stable_id_of(sim.world(), made.assembly).expect("id").0;
    assert_eq!(joints_of(sim.world_mut(), id).len(), 2, "the assembly keeps its two");
    let new_id = ph2d_ecs::stable_id_of(sim.world(), copy.root).expect("id").0;
    let w = sim.world_mut();
    let copied: Vec<PhysicsJoint> =
        w.query::<&PhysicsJoint>().iter(w).filter(|j| j.body_a == new_id).copied().collect();
    assert_eq!(copied.len(), 2, "the copy carries its child joints");
    assert!(copied.iter().all(|j| j.assembly == 0), "as loose joints");
}
