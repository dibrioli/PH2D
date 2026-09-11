//! **Reafinar um joint vivo descreve o MESMO joint que respawná-lo** — a porta
//! única `build_joint`, provada pelo comportamento em vez de pela leitura.
//!
//! `PhysicsWorld::retune_joint` existe porque um edit de PARÂMETRO não é
//! estrutural: sobrescrever o `data` do `ImpulseJoint` não mexe nas arenas, e é
//! isso que deixa o ring de checkpoints sobreviver a um número keyframado. O
//! risco que ele traz é o de sempre — **duas portas construindo o mesmo joint**
//! —, e uma divergência ali só apareceria num scrub, que é onde ninguém lê um
//! número.
//!
//! ⚠️ **Os gates comparam antes do primeiro `step`, de propósito.** Um retune
//! PRESERVA os impulsos acumulados do joint (é o mesmo `ImpulseJoint`) e um
//! respawn os zera, então os dois só são idênticos enquanto não há nada
//! acumulado. Essa diferença é uma FEATURE do retune — é o warm-start que o
//! solver perderia a cada quadro de slider arrastado —, e afirmá-la aqui é o
//! que impede alguém de "consertar" o gate afrouxando a comparação.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyType, ShapeDesc,
};

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
) -> ph2d_physics::RigidBodyHandle {
    world.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
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
        offset: [0.0, 0.0],
    })
}

/// Uma dobradiça com servo: gancho estático em `(0, 6)`, prancha de 1 m à
/// direita, pino na ponta esquerda dela. O motor de POSIÇÃO puxa a prancha para
/// o alvo, então o alvo é um número que se VÊ.
fn servo(target: f32) -> JointDesc {
    JointDesc {
        kind: JointKind::Pin,
        anchor_a: [0.0, 5.0],
        anchor_b: [0.0, 5.0],
        motor: Some(MotorDesc {
            mode: MotorMode::Position,
            speed: 0.0,
            target,
            max_force: 500.0,
        }),
        ..JointDesc::default()
    }
}

/// O que uma montagem devolve: o mundo, a prancha, o joint e o par de âncoras
/// LOCAIS que ele foi construído com (o retune reusa exactamente esse par, que é
/// o que a ponte faz com o `j.rest`).
struct Rig {
    world: PhysicsWorld,
    plank: ph2d_physics::RigidBodyHandle,
    joint: ph2d_physics::ImpulseJointHandle,
    locals: ([f32; 2], [f32; 2]),
}

/// Um mundo com a dobradiça montada e o servo apontando para `target`.
fn rig(target: f32) -> Rig {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let plank = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.5,
        5.0,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    let d = servo(target);
    let (la, lb) = w
        .world_to_local_anchors(hook, plank, d.anchor_a, d.anchor_b)
        .expect("bodies alive");
    let h = w
        .spawn_joint(
            hook,
            plank,
            JointDesc {
                anchor_a: la,
                anchor_b: lb,
                ..d
            },
        )
        .expect("joint");
    Rig {
        world: w,
        plank,
        joint: h,
        locals: (la, lb),
    }
}

fn run(w: &mut PhysicsWorld, plank: ph2d_physics::RigidBodyHandle, steps: usize) -> [f32; 3] {
    for _ in 0..steps {
        w.step();
    }
    let p = w.body_pose(plank).expect("alive");
    [p.translation.x, p.translation.y, p.rotation.angle()]
}

/// **Um joint reafinado para D é o mesmo joint que um spawnado com D.**
///
/// Mutação que sangra: dar ao `retune_joint` um construtor próprio (por exemplo,
/// escrever só o motor e deixar o resto do `data` como estava) — a prancha
/// deixaria de perseguir o alvo novo e as duas trajetórias divergiriam já no
/// primeiro décimo de segundo.
#[test]
fn a_retuned_joint_is_the_joint_a_respawn_would_have_built() {
    // A: nasce apontando para 0 e é reafinado para −0,8 rad antes do 1º passo.
    let mut a_rig = rig(0.0);
    let d = servo(-0.8);
    assert!(a_rig.world.retune_joint(
        a_rig.joint,
        &JointDesc {
            anchor_a: a_rig.locals.0,
            anchor_b: a_rig.locals.1,
            ..d
        }
    ));
    let a = run(&mut a_rig.world, a_rig.plank, 90);

    // B: nasce já apontando para −0,8.
    let mut b_rig = rig(-0.8);
    let b = run(&mut b_rig.world, b_rig.plank, 90);

    assert_eq!(
        a, b,
        "retune e respawn descrevem o mesmo joint: {a:?} contra {b:?}"
    );
    // CONTROLE: o alvo de fato move a prancha — sem isto os dois lados poderiam
    // concordar por nada ter acontecido.
    let mut c_rig = rig(0.0);
    let c = run(&mut c_rig.world, c_rig.plank, 90);
    assert!(
        (a[2] - c[2]).abs() > 0.3,
        "o alvo do servo tem de mover a prancha: {a:?} contra o controle {c:?}"
    );
}

/// **Um handle morto devolve `false`** — o chamador respawna em vez de achar que
/// escreveu.
#[test]
fn retuning_a_dead_handle_reports_that_it_did_not_land() {
    let mut r = rig(0.0);
    r.world.remove_joint(r.joint);
    assert!(!r.world.retune_joint(r.joint, &servo(-0.8)));
}
