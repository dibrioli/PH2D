//! **O joint descrito por EIXO** (W-JointCustom) — provado pelo que ele PERMITE
//! e pelo que ele PROÍBE, nunca pela máscara que foi escrita.
//!
//! Todo tipo do kit é um `GenericJoint` com uma configuração fixa, e o `Custom`
//! é o mesmo motor com a configuração autorada. O jeito honesto de gatear isso
//! não é ler `locked_axes` de volta — é montar a configuração de um tipo que já
//! existe e exigir que o corpo se mova **como aquele tipo**: um Custom que trava
//! Y e a rotação É um Slider, e um que trava os três É uma solda.
//!
//! ⚠️ **Cada afirmação de restrição vem com a afirmação do movimento que ela
//! ainda tem de permitir** — a lei deste arquivo desde o W3: *"os corpos estão
//! presos"* é satisfeito igualmente bem por *"nada se moveu"*.

use ph2d_physics::{
    AxisMode, AxisSpec, BodyDesc, CustomAxis, CustomDesc, JointDesc, JointKind, MotorDesc,
    MotorMode, PhysicsWorld, RigidBodyType, ShapeDesc,
};

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
) -> ph2d_physics::RigidBodyHandle {
    world.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 0.25,
            half_y: 0.25,
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
        offset: [0.0, 0.0],
    })
}

fn spec(mode: AxisMode, min: f32, max: f32) -> AxisSpec {
    AxisSpec { mode, min, max }
}

/// Um bloco pendurado de um poste por um Custom com a configuração dada.
/// Devolve `(pose final, deslocamento)` depois de `steps` passos de gravidade.
fn hang(
    axes: [AxisSpec; 3],
    motor: Option<MotorDesc>,
    motor_axis: CustomAxis,
    steps: usize,
) -> ([f32; 3], [f32; 2]) {
    let mut w = PhysicsWorld::new();
    let post = body(&mut w, RigidBodyType::Fixed, 0.0, 5.0);
    let block = body(&mut w, RigidBodyType::Dynamic, 0.0, 5.0);
    let d = JointDesc {
        kind: JointKind::Custom,
        anchor_a: [0.0, 5.0],
        anchor_b: [0.0, 5.0],
        motor,
        custom: CustomDesc { axes, motor_axis },
        ..JointDesc::default()
    };
    let (la, lb) = w
        .world_to_local_anchors(post, block, d.anchor_a, d.anchor_b)
        .expect("bodies");
    w.spawn_joint(
        post,
        block,
        JointDesc {
            anchor_a: la,
            anchor_b: lb,
            ..d
        },
    )
    .expect("joint");
    let start = w.body_pose(block).expect("alive");
    let (sx, sy) = (start.translation.x, start.translation.y);
    for _ in 0..steps {
        w.step();
    }
    let p = w.body_pose(block).expect("alive");
    (
        [p.translation.x, p.translation.y, p.rotation.angle()],
        [p.translation.x - sx, p.translation.y - sy],
    )
}

const FREE: AxisSpec = AxisSpec {
    mode: AxisMode::Free,
    min: -1.0,
    max: 1.0,
};
const LOCKED: AxisSpec = AxisSpec {
    mode: AxisMode::Locked,
    min: -1.0,
    max: 1.0,
};

/// **Travar os três eixos É uma solda** — e deixá-los livres NÃO é.
///
/// O par é o gate: sem o controle, *"o bloco não se moveu"* seria satisfeito por
/// um joint que trava tudo por acidente de construção.
#[test]
fn locking_every_axis_welds_and_freeing_them_does_not() {
    let (_, welded) = hang([LOCKED, LOCKED, LOCKED], None, CustomAxis::X, 120);
    let (_, loose) = hang([FREE, FREE, FREE], None, CustomAxis::X, 120);
    assert!(
        welded[1].abs() < 1e-3,
        "três eixos travados não podem deixar o bloco cair: {welded:?}"
    );
    assert!(
        loose[1] < -0.5,
        "três eixos livres têm de deixar o bloco cair: {loose:?}"
    );
}

/// **Travar Y e a rotação É um trilho horizontal:** o bloco não cai e não gira,
/// e ainda desliza em X.
#[test]
fn locking_y_and_rotation_leaves_a_horizontal_rail() {
    let (pose, moved) = hang([FREE, LOCKED, LOCKED], None, CustomAxis::X, 120);
    assert!(
        moved[1].abs() < 1e-3,
        "Y travado não pode deixar o bloco descer: {moved:?}"
    );
    assert!(
        pose[2].abs() < 1e-3,
        "a rotação travada não pode deixar o bloco girar: {pose:?}"
    );
    // E o eixo LIVRE ainda é livre: um motor nele move o carrinho.
    let (_, driven) = hang(
        [FREE, LOCKED, LOCKED],
        Some(MotorDesc {
            mode: MotorMode::Velocity,
            speed: 1.5,
            target: 0.0,
            max_force: 500.0,
        }),
        CustomAxis::X,
        120,
    );
    assert!(
        driven[0].abs() > 0.5,
        "o eixo X livre tem de deixar o motor mover o carrinho: {driven:?}"
    );
}

/// **Um eixo LIMITADO para onde o batente diz.**
///
/// ⚠️ O controle é o MESMO eixo em `Free`: sem ele, *"parou em 0,5"* seria
/// satisfeito por um eixo que nunca se moveu.
#[test]
fn a_limited_axis_stops_at_its_stop() {
    let drive = Some(MotorDesc {
        mode: MotorMode::Velocity,
        speed: 3.0,
        target: 0.0,
        max_force: 500.0,
    });
    let limited = spec(AxisMode::Limited, -0.5, 0.5);
    let (_, capped) = hang([limited, LOCKED, LOCKED], drive, CustomAxis::X, 180);
    let (_, uncapped) = hang([FREE, LOCKED, LOCKED], drive, CustomAxis::X, 180);
    assert!(
        capped[0] <= 0.55,
        "o batente de 0,5 m tem de segurar o carrinho: {capped:?}"
    );
    assert!(
        uncapped[0] > 1.5,
        "sem batente o mesmo motor tem de levar o carrinho muito além: {uncapped:?}"
    );
}

/// **O eixo do motor é AUTORADO** — o mesmo motor no eixo de rotação GIRA e no
/// eixo X DESLIZA.
///
/// É o gate da decisão de projeto: *"o motor dirige o primeiro eixo não
/// travado"* seria mágica, e este par é o que a torna inexprimível.
#[test]
fn the_motor_drives_the_axis_the_artist_chose() {
    let drive = Some(MotorDesc {
        mode: MotorMode::Velocity,
        speed: 2.5,
        target: 0.0,
        max_force: 500.0,
    });
    // Tudo livre, para que os dois eixos estejam disponíveis e só a ESCOLHA
    // decida — se o motor caísse no "primeiro livre", os dois casos seriam X.
    let (spun, _) = hang([FREE, FREE, FREE], drive, CustomAxis::Rotation, 60);
    let (_, slid) = hang([FREE, FREE, FREE], drive, CustomAxis::X, 60);
    assert!(
        spun[2].abs() > 0.5,
        "o motor no eixo de rotação tem de girar o bloco: {spun:?}"
    );
    assert!(
        slid[0].abs() > 0.5,
        "o mesmo motor no eixo X tem de deslizar o bloco: {slid:?}"
    );
}

/// **Um par de batentes invertido não SOLDA o eixo.**
///
/// O W3 pagou este bug uma vez (limites invertidos soldavam a dobradiça), e a
/// ordem é normalizada na porta única do construtor. O oráculo é o movimento que
/// o eixo tem de continuar permitindo.
#[test]
fn an_inverted_stop_pair_does_not_weld_the_axis() {
    let drive = Some(MotorDesc {
        mode: MotorMode::Velocity,
        speed: 3.0,
        target: 0.0,
        max_force: 500.0,
    });
    let inverted = spec(AxisMode::Limited, 0.5, -0.5);
    let (_, moved) = hang([inverted, LOCKED, LOCKED], drive, CustomAxis::X, 180);
    assert!(
        moved[0] > 0.3,
        "um par invertido é o mesmo intervalo lido ao contrário, não um eixo travado: {moved:?}"
    );
}
