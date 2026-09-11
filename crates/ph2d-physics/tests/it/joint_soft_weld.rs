//! **A solda que CEDE** (W-SoftWeld) — o vão entre segurar um ângulo
//! *absolutamente* e deixá-lo *inteiramente livre*.
//!
//! O oráculo de toda cena aqui é uma **viga em balanço**: um braço preso pela
//! ponta a uma parede estática, sob gravidade. É a cena mais curta em que as três
//! coisas que podem dar errado são distinguíveis — não ceder (é rígido), ceder e
//! não voltar (é uma dobradiça), e **se soltar** (a variante que a medição
//! reprovou).

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc,
};

const ARM_HALF: [f32; 2] = [0.5, 0.1];

fn arm_body(w: &mut PhysicsWorld, x: f32, y: f32, shape: ShapeDesc) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
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

/// O que a viga fez em 6 s.
struct Cantilever {
    /// Quanto ela pendeu, graus (positivo = para baixo).
    droop: f32,
    /// Quanto a ponta SOLDADA se afastou do gancho, metros. Numa solda isto é 0
    /// — é a metade que a variante *tudo mole* reprovou (0,92 m medidos).
    separation: f32,
    /// Pico-a-pico do ângulo no último terço. Perto de zero = assentou.
    swing: f32,
}

fn cantilever(soft: bool, stiffness: f32, damping: f32) -> Cantilever {
    let mut w = PhysicsWorld::new();
    let (wall, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.3);
    let arm = arm_body(
        &mut w,
        ARM_HALF[0],
        0.0,
        ShapeDesc::Cuboid {
            half_x: ARM_HALF[0],
            half_y: ARM_HALF[1],
        },
    );
    let (la, lb) = w
        .world_to_local_anchors(wall, arm, [0.0, 0.0], [0.0, 0.0])
        .expect("bodies alive");
    w.spawn_joint(
        wall,
        arm,
        JointDesc {
            kind: JointKind::Weld,
            soft,
            stiffness,
            damping,
            anchor_a: la,
            anchor_b: lb,
            ..Default::default()
        },
    )
    .expect("joint built");

    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..360 {
        w.step();
        if i >= 240 {
            let d = -w
                .body_pose(arm)
                .expect("alive")
                .rotation
                .angle()
                .to_degrees();
            lo = lo.min(d);
            hi = hi.max(d);
        }
    }
    let p = w.body_pose(arm).expect("alive");
    let (s, c) = (p.rotation.angle().sin(), p.rotation.angle().cos());
    let ax = p.translation.x - c * ARM_HALF[0];
    let ay = p.translation.y - s * ARM_HALF[0];
    Cantilever {
        droop: -p.rotation.angle().to_degrees(),
        separation: ax.hypot(ay),
        swing: hi - lo,
    }
}

/// **A entrega da wave, e o CONTROLE está na mesma função.**
///
/// A solda rígida de hoje não cede um grau; a mole verga sob o próprio peso e
/// para lá. Sem o controle o gate não distinguiria *"a solda mole funciona"* de
/// *"toda solda sempre pendeu"*.
#[test]
fn a_soft_weld_bends_and_a_rigid_one_does_not() {
    let rigid = cantilever(false, 30.0, 0.5);
    let soft = cantilever(true, 30.0, 0.5);

    assert!(
        rigid.droop.abs() < 0.05,
        "o controle se moveu: a solda RÍGIDA pendeu {:.3}°",
        rigid.droop
    );
    assert!(
        soft.droop > 2.0,
        "a solda mole não cedeu: {:.3}° (medido 5,3° com os defaults)",
        soft.droop
    );
}

/// **E as duas peças continuam UMA.** É a metade que separa vergar de soltar, e
/// a que decidiu o desenho: com os três eixos moles (nada travado, três motores
/// de posição) o braço derivou **0,92 m** para longe da parede — as peças vêm
/// APART, que se lê como a solda falhando.
#[test]
fn a_soft_weld_does_not_come_apart() {
    let soft = cantilever(true, 30.0, 0.5);
    assert!(
        soft.separation < 1e-3,
        "a solda mole se abriu {:.4} m — os eixos LINEARES têm de ficar travados",
        soft.separation
    );
}

/// **Ela PARA.** Uma solda que oscila para sempre é uma mola, não uma solda — e
/// é isto que o ganho angular compra: com ele em 1 a mesma cena balança 77°
/// pico-a-pico sem nunca assentar.
#[test]
fn a_soft_weld_settles() {
    let soft = cantilever(true, 30.0, 0.5);
    assert!(
        soft.swing < 1.0,
        "a solda mole ainda balançava {:.3}° pico-a-pico no fim",
        soft.swing
    );
}

/// **A dureza é o knob, e ele responde na direção certa** — mais rígido, menos
/// cede —, e a faixa inteira do artista assenta.
#[test]
fn the_stiffness_knob_runs_from_rubber_to_rigid_and_every_step_settles() {
    let mut last = f32::MAX;
    for k in [1.0, 10.0, 100.0, 1000.0] {
        let c = cantilever(true, k, 0.5);
        assert!(
            c.droop < last,
            "stiffness {k} pendeu {:.3}°, não menos que os {last:.3}° do valor anterior",
            c.droop
        );
        assert!(
            c.swing < 1.0,
            "stiffness {k} não assentou: {:.3}° pico-a-pico",
            c.swing
        );
        last = c.droop;
    }
}
