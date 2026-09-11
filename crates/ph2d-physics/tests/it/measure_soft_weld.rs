//! **Sonda: uma solda que CEDE — em que eixos, e com que ganho?**
//!
//! Roda com
//! `cargo test -p ph2d-physics --test measure_soft_weld -- --ignored --nocapture`.
//!
//! O vão que ela mede é o espelho exato do que a [`JointKind::Rod`] preencheu.
//! Este conjunto sabe segurar um ÂNGULO de dois jeitos e só dois: **absoluto**
//! (Weld, Slider) ou **livre** (Spring, Rope, Rod, o giro do Wheel). Não há nada
//! no meio — uma placa que balança e volta, um poste que verga sob o vento, um
//! pescoço que resiste mas cede, nenhum é exprimível.
//!
//! ⛔ **E o desenho óbvio está morto antes de começar** (plano 02 §8, medido em
//! 2026-07-27): pôr mola nos eixos de um `FixedJoint` não faz nada —
//! `contact_constraints_set.rs:48` faz `motor_axes.bits() & !locked_axes`, e um
//! motor num eixo TRAVADO é mascarado. Sobra construir a solda mole com OUTRO
//! vínculo, e é aí que a sonda tem duas perguntas:
//!
//! 1. **Que eixos ficam moles?** *Tudo* (três molas) faz as duas peças se
//!    SEPARAREM sob carga, o que se lê como a solda falhando, não vergando.
//!    *Só o angular* mantém as âncoras coincidentes e verga — mas é preciso
//!    medir o quanto cada um dá antes de escolher.
//! 2. **Um knob só serve os dois eixos?** A rigidez linear é N/m e a angular é
//!    N·m/rad: não são a mesma grandeza, e usar o mesmo número cru nos dois é
//!    dimensionalmente torto. O quanto isso custa é o que a varredura diz.

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};

const ARM_HALF: [f32; 2] = [0.5, 0.1];

fn body(
    w: &mut PhysicsWorld,
    body_type: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
) -> ph2d_physics::RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type,
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

/// O que uma viga em balanço faz: quanto ela CAIU de ângulo (graus, positivo =
/// pendeu) e quanto a ponta soldada se AFASTOU do gancho (metros — numa solda de
/// verdade isto é zero).
///
/// ⚠️ **`swing` é o que separa assentado de oscilando**, e sem ele a varredura
/// MENTE: a 1ª corrida leu a rigidez 100 como pior que a 30 e o damping 5 como
/// pior que o 0,5, o que é impossível — os dois números eram instantes de uma
/// viga ainda balançando.
struct Droop {
    degrees: f32,
    separation: f32,
    /// Pico-a-pico do ângulo no ÚLTIMO terço da corrida. Perto de zero = parou.
    swing: f32,
}

/// Um braço de 1,0 × 0,2 m preso pela ponta esquerda a uma parede estática, sob
/// gravidade. A cena mais curta que separa *vergar* de *soltar*.
fn cantilever(soft: bool, stiffness: f32, damping: f32, extra_mass: Option<f32>) -> Droop {
    let mut w = PhysicsWorld::new();
    let wall = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Cuboid {
            half_x: 0.1,
            half_y: 0.3,
        },
    );
    let arm = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: ARM_HALF[0],
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: ARM_HALF[0],
            half_y: ARM_HALF[1],
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
        mass_override: extra_mass,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });
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
    // 6 s: tempo de sobra para uma viga mole assentar, e o último terço mede se
    // ela de fato assentou.
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..360 {
        w.step();
        if i >= 240 {
            let d = -w
                .body_pose(arm)
                .expect("arm alive")
                .rotation
                .angle()
                .to_degrees();
            lo = lo.min(d);
            hi = hi.max(d);
        }
    }
    let p = w.body_pose(arm).expect("arm alive");
    // A âncora do braço é a ponta esquerda dele, em local `(-0.5, 0)`; onde ela
    // FOI parar em mundo é o que diz se a solda se abriu.
    let (s, c) = (p.rotation.angle().sin(), p.rotation.angle().cos());
    let ax = p.translation.x + c * (-ARM_HALF[0]);
    let ay = p.translation.y + s * (-ARM_HALF[0]);
    Droop {
        degrees: -p.rotation.angle().to_degrees(),
        separation: (ax * ax + ay * ay).sqrt(),
        swing: hi - lo,
    }
}

#[test]
#[ignore = "sonda de medição"]
fn measure_what_a_soft_weld_gives() {
    println!("\n=== CONTROLE: a solda RÍGIDA de hoje ===");
    let rigid = cantilever(false, 30.0, 0.5, None);
    println!(
        "  rigid            droop {:>8.3}°   separation {:>8.4} m   swing {:>7.3}°",
        rigid.degrees, rigid.separation, rigid.swing
    );

    println!("\n=== VARREDURA de stiffness (ganho angular = a const de hoje) ===");
    println!("  stiffness |    droop    | separation |  swing");
    for k in [10.0, 30.0, 100.0, 300.0, 1_000.0, 3_000.0, 10_000.0] {
        let d = cantilever(true, k, JointDesc::DEFAULT_DAMPING, None);
        println!(
            "  {k:>9.0} | {:>8.3}°   | {:>9.4} m | {:>7.3}°",
            d.degrees, d.separation, d.swing
        );
    }

    println!("\n=== A MESMA solda sob CARGAS diferentes (stiffness default) ===");
    println!("  massa do braço |   droop    | separation |  swing");
    for m in [0.05, 0.2, 1.0, 5.0] {
        let d = cantilever(true, JointDesc::DEFAULT_STIFFNESS, 0.5, Some(m));
        println!(
            "  {m:>13.2} kg | {:>8.3}°  | {:>9.4} m | {:>7.3}°",
            d.degrees, d.separation, d.swing
        );
    }

    println!("\n=== O PAR (rigidez angular x damping angular) — droop / swing ===");
    print!("  k \\ d   ");
    for d in [1.0, 3.0, 10.0, 30.0, 100.0] {
        print!("|{d:>16.0}   ");
    }
    println!();
    for k in [100.0, 300.0, 1_000.0, 3_000.0, 10_000.0] {
        print!("  {k:>7.0} ");
        for damp in [1.0, 3.0, 10.0, 30.0, 100.0] {
            let r = cantilever(true, k, damp, None);
            print!("| {:>6.2}° s{:>6.2}° ", r.degrees, r.swing);
        }
        println!();
    }

    println!("\n=== O GANHO: os defaults do artista (k=30, d=0.5) vezes G ===");
    println!("  G     | k_ang  d_ang |   droop    |  swing");
    for g in [1.0, 5.0, 10.0, 20.0, 30.0, 50.0, 100.0, 200.0] {
        let k = JointDesc::DEFAULT_STIFFNESS * g;
        let d = JointDesc::DEFAULT_DAMPING * g;
        let r = cantilever(true, k, d, None);
        println!(
            "  {g:>5.0} | {k:>6.0} {d:>6.1} | {:>8.3}°   | {:>7.3}°",
            r.degrees, r.swing
        );
    }

    println!("\n=== Com G = 20, a FAIXA do artista (damping default 0.5) ===");
    println!("  Stiffness |   droop    |  swing");
    for k in [1.0, 3.0, 10.0, 30.0, 100.0, 300.0, 1_000.0] {
        let r = cantilever(true, k * 20.0, JointDesc::DEFAULT_DAMPING * 20.0, None);
        println!("  {k:>9.0} | {:>8.3}°   | {:>7.3}°", r.degrees, r.swing);
    }

    println!("\n=== O BREAK TORQUE alcanca uma solda MOLE? ===");
    println!("  (um Weld RIGIDO le 0.0000: rapier nao publica reacao de eixo TRAVADO)");
    for soft in [false, true] {
        let mut w = PhysicsWorld::new();
        let (wall, _) = w.add_static_cuboid(0.0, 0.0, 0.1, 0.3);
        let arm = w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: ARM_HALF[0],
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: ARM_HALF[0],
                half_y: ARM_HALF[1],
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
        });
        let (la, lb) = w
            .world_to_local_anchors(wall, arm, [0.0, 0.0], [0.0, 0.0])
            .expect("bodies alive");
        let h = w
            .spawn_joint(
                wall,
                arm,
                JointDesc {
                    kind: JointKind::Weld,
                    soft,
                    stiffness: JointDesc::DEFAULT_STIFFNESS,
                    damping: JointDesc::DEFAULT_DAMPING,
                    anchor_a: la,
                    anchor_b: lb,
                    ..Default::default()
                },
            )
            .expect("joint built");
        for _ in 0..360 {
            w.step();
        }
        let load = w.joint_load(h);
        println!(
            "  soft={soft:<5} force {:>9.4} N   torque {:>9.4} N.m",
            load.map_or(f32::NAN, |l| l.force),
            load.map_or(f32::NAN, |l| l.torque)
        );
    }

    println!();
}
