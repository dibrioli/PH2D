//! **Os Pins `one_chain`: uma corrente resolvida de uma vez** (`world::chains`, Joint Editor W2).
//!
//! A ponte do gabarito Bridge: 8 tábuas 0,5 × 0,1 m (densidade 1 → 0,05 kg), 2 cm de folga, postes
//! fixos, caixa de 10 kg (200:1) caindo de 2 m acima. Números: `docs/JointEditor/02_gabaritos_medidos.md`.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};

type Handle = ph2d_physics::RigidBodyHandle;

fn body(
    w: &mut PhysicsWorld,
    kind: RigidBodyType,
    at: [f32; 3],
    half: [f32; 2],
    density: f32,
) -> Handle {
    w.spawn_body(BodyDesc {
        body_type: kind,
        x: at[0],
        y: at[1],
        rotation: at[2],
        density,
        shape: ShapeDesc::Cuboid {
            half_x: half[0],
            half_y: half[1],
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        shape_pose: [0.0; 3],
        collide: [0; 2],
        shape_off: false,
        offset: [0.0, 0.0],
    })
}

struct Bridge {
    world: PhysicsWorld,
    planks: Vec<Handle>,
    load: Handle,
    joints: Vec<ph2d_physics::ImpulseJointHandle>,
}

/// A ponte, com cada folga presa por `link(meio da folga em A, em B)`.
fn bridge(one_chain: bool, kind: JointKind) -> Bridge {
    let mut w = PhysicsWorld::new();
    body(
        &mut w,
        RigidBodyType::Fixed,
        [0.0, -0.5, 0.0],
        [6.0, 0.5],
        1.0,
    );
    let left = body(
        &mut w,
        RigidBodyType::Fixed,
        [-2.24, 0.6, 0.0],
        [0.15, 0.5],
        1.0,
    );
    let right = body(
        &mut w,
        RigidBodyType::Fixed,
        [2.24, 0.6, 0.0],
        [0.15, 0.5],
        1.0,
    );
    let planks: Vec<Handle> = (0..8)
        .map(|i| {
            let x = -1.82 + 0.52 * i as f32;
            body(
                &mut w,
                RigidBodyType::Dynamic,
                [x, 1.0, 0.0],
                [0.25, 0.05],
                1.0,
            )
        })
        .collect();
    let load = body(
        &mut w,
        RigidBodyType::Dynamic,
        [0.0, 3.0, 0.0],
        [0.25, 0.25],
        40.0,
    );
    let mut pairs = vec![(left, planks[0], [0.16, 0.4], [-0.26, 0.0])];
    for i in 0..7 {
        pairs.push((planks[i], planks[i + 1], [0.26, 0.0], [-0.26, 0.0]));
    }
    pairs.push((planks[7], right, [0.26, 0.0], [-0.16, 0.4]));
    let joints = pairs
        .into_iter()
        .map(|(a, b, la, lb)| {
            let d = JointDesc {
                kind,
                anchor_a: la,
                anchor_b: lb,
                max_length: 0.0,
                one_chain,
                ..JointDesc::default()
            };
            w.spawn_joint(a, b, d).expect("joint")
        })
        .collect();
    Bridge {
        world: w,
        planks,
        load,
        joints,
    }
}

fn y(w: &PhysicsWorld, h: Handle) -> f32 {
    w.body_pose(h).expect("body").translation.y
}

fn run(b: &mut Bridge, ticks: usize) -> f32 {
    let mut low = f32::MAX;
    for _ in 0..ticks {
        b.world.step();
        low = low.min(y(&b.world, b.planks[4]));
    }
    low
}

/// A caixa 200× mais pesada pousa na ponte e ela segura: em repouso a caixa fica a 1,30 m (tábua
/// 1,05 mais meia caixa). Medido: corrente 1,283 (−2 cm), o mais baixo da tábua do meio +0,74; Pins de
/// impulso 1,017 com a caixa escorregada; os mesmos Pins sem `one_chain` são o controlo.
#[test]
fn a_light_bridge_as_one_chain_holds_a_heavy_load() {
    let mut chain = bridge(true, JointKind::Pin);
    let low = run(&mut chain, 600);
    let crate_y = y(&chain.world, chain.load);
    assert!(
        crate_y > 1.25,
        "the crate rests on the bridge, not through it: {crate_y:+.3}"
    );
    assert!(
        low > 0.6,
        "the middle plank never nears the ground: {low:+.3}"
    );
    assert!(
        chain
            .joints
            .iter()
            .all(|&j| chain.world.joint_is_enabled(j) == Some(true))
    );
    // Duas árvores, uma por poste, que se encontram por UM Pin de impulso.
    let reduced = chain
        .joints
        .iter()
        .filter(|&&j| chain.world.joint_is_in_one_chain(j))
        .count();
    assert_eq!(
        reduced,
        chain.joints.len() - 1,
        "every joint but the one that closes the loop"
    );

    let mut control = bridge(false, JointKind::Pin);
    run(&mut control, 600);
    let control_y = y(&control.world, control.load);
    // Medido (dev): corrente −1,7 cm, os mesmos Pins de impulso −17 cm.
    let (sag, control_sag) = (1.30 - crate_y, 1.30 - control_y);
    assert!(
        control_sag > 3.0 * sag,
        "without one chain the same Pins give way: sag {control_sag:.3} m against {sag:.3} m"
    );
}

/// Rebobinar é exato: restaurar o retrato do tique 100 e repetir dá o mesmo hash que nunca ter parado.
#[test]
fn a_chain_rewinds_bit_exact() {
    let mut a = bridge(true, JointKind::Pin);
    run(&mut a, 100);
    let cp = a.world.checkpoint();
    run(&mut a, 80);
    let straight = a.world.deterministic_hash();
    a.world.restore(&cp);
    run(&mut a, 80);
    assert_eq!(a.world.deterministic_hash(), straight);

    let mut b = bridge(true, JointKind::Pin);
    run(&mut b, 180);
    assert_eq!(
        b.world.deterministic_hash(),
        straight,
        "and the same as a fresh run"
    );
}

/// Apagar uma junta do meio refaz as árvores: as duas metades ficam penduradas nos postes.
#[test]
fn removing_a_joint_regrows_the_chain() {
    let mut b = bridge(true, JointKind::Pin);
    run(&mut b, 10);
    b.world.remove_joint(b.joints[4]);
    run(&mut b, 300);
    let reduced = b
        .joints
        .iter()
        .filter(|&&j| b.world.joint_is_in_one_chain(j))
        .count();
    assert_eq!(
        reduced,
        b.joints.len() - 1,
        "both halves are trees now, nothing closes a loop"
    );
    let low = y(&b.world, b.planks[3]);
    assert!(low < 0.6, "the cut half hangs from its post: {low:+.3}");
}

/// Só o Pin inquebrável e sem motor entra; os outros seguem de impulso e quebram como sempre.
#[test]
fn only_an_unbreakable_plain_pin_joins_a_chain() {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        [0.0, 2.0, 0.0],
        [0.1, 0.1],
        1.0,
    );
    let bob = body(
        &mut w,
        RigidBodyType::Dynamic,
        [0.0, 1.0, 0.0],
        [0.1, 0.1],
        1.0,
    );
    let pin = |extra: fn(&mut JointDesc)| {
        let mut d = JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, -0.5],
            anchor_b: [0.0, 0.5],
            one_chain: true,
            ..JointDesc::default()
        };
        extra(&mut d);
        d
    };
    let plain = w.spawn_joint(hook, bob, pin(|_| {})).expect("plain");
    let weak = w
        .spawn_joint(hook, bob, pin(|d| d.break_force = 10.0))
        .expect("weak");
    let rope = w
        .spawn_joint(hook, bob, pin(|d| d.kind = JointKind::Rope))
        .expect("rope");
    w.step();
    assert!(w.joint_is_in_one_chain(plain));
    assert!(
        !w.joint_is_in_one_chain(weak),
        "a break reads the impulse the twin does not have"
    );
    assert!(!w.joint_is_in_one_chain(rope));

    // Reafinar para quebrável tira-o da corrente no passo seguinte.
    assert!(w.retune_joint(plain, &pin(|d| d.break_force = 10.0)));
    w.step();
    assert!(!w.joint_is_in_one_chain(plain));
}

/// A árvore nasce com as coordenadas das poses REAIS: um pêndulo de dois elos tortos, sem gravidade,
/// não salta no primeiro passo.
#[test]
fn a_chain_born_crooked_keeps_its_pose() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        [0.0, 3.0, 0.0],
        [0.05, 0.05],
        1.0,
    );
    let (s, c) = (0.5_f32.sin(), 0.5_f32.cos());
    // Elo 1 girado 0,5 rad, pendurado no gancho pela ponta de cima; elo 2 girado −0,4 rad.
    let p1 = [0.5 * s, 3.0 - 0.5 * c];
    let l1 = body(
        &mut w,
        RigidBodyType::Dynamic,
        [p1[0], p1[1], 0.5],
        [0.05, 0.5],
        1.0,
    );
    let tip = [p1[0] + 0.5 * s, p1[1] - 0.5 * c];
    let (s2, c2) = ((-0.4_f32).sin(), (-0.4_f32).cos());
    let p2 = [tip[0] + 0.5 * s2, tip[1] - 0.5 * c2];
    let l2 = body(
        &mut w,
        RigidBodyType::Dynamic,
        [p2[0], p2[1], -0.4],
        [0.05, 0.5],
        1.0,
    );
    let pin = |la: [f32; 2], lb: [f32; 2]| JointDesc {
        kind: JointKind::Pin,
        anchor_a: la,
        anchor_b: lb,
        one_chain: true,
        ..JointDesc::default()
    };
    let j1 = w
        .spawn_joint(hook, l1, pin([0.0, 0.0], [0.0, 0.5]))
        .expect("j1");
    let j2 = w
        .spawn_joint(l1, l2, pin([0.0, -0.5], [0.0, 0.5]))
        .expect("j2");
    let before = [w.body_pose(l1).expect("l1"), w.body_pose(l2).expect("l2")];
    w.step();
    assert!(w.joint_is_in_one_chain(j1) && w.joint_is_in_one_chain(j2));
    for (h, b) in [l1, l2].into_iter().zip(before) {
        let a = w.body_pose(h).expect("pose");
        let moved = (a.translation - b.translation).length();
        let turned = (a.rotation.angle() - b.rotation.angle()).abs();
        assert!(
            moved < 1e-3 && turned < 1e-3,
            "no snap: moved {moved:.4} m, turned {turned:.4} rad"
        );
    }
}
