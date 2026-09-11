//! **Sonda: como se constrói uma BARRA RÍGIDA no rapier 0.28?**
//!
//! Roda com
//! `cargo test -p ph2d-physics --test measure_rod -- --ignored --nocapture`.
//!
//! ⛔ **O desenho óbvio está MEDIDO e MORTO:** uma rope com `set_limits(LinX,
//! [d, d])` não segura. `solver/joint_constraint/joint_constraint_builder.rs`
//! traz, dentro de `limit_linear_coupled`, o comentário
//! `// FIXME: handle min limit too.` — ele lê só `limits[1]` e sai com
//! `impulse_bounds = [0, INFINITY]`, ou seja **unilateral**. O mínimo de um
//! limite linear acoplado não está implementado.
//!
//! Sobram duas construções, e esta sonda mede as duas na cena que separa uma
//! barra de uma corda — o **pêndulo INVERTIDO**, onde o vínculo tem de
//! EMPURRAR — e na que separa uma barra de uma mola: uma **carga pendurada**,
//! onde ele tem de não esticar.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyType, ShapeDesc,
};

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn body(
    w: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    radius: f32,
) -> ph2d_physics::RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius },
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

fn pose(w: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> [f32; 2] {
    let p = w.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y]
}

/// Uma cena de dois corpos ligados por `desc`, com o corpo B começando `dy`
/// acima (positivo) ou abaixo (negativo) do gancho. Devolve `(min, max)` da
/// distância ao longo de 3 s.
fn run(desc: JointDesc, dy: f32, bob_radius: f32) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    let hook = body(&mut w, RigidBodyType::Fixed, 0.0, 0.0, 0.05);
    let bob = body(&mut w, RigidBodyType::Dynamic, 0.0, dy, bob_radius);
    let (la, lb) = w
        .world_to_local_anchors(hook, bob, [0.0, 0.0], [0.0, dy])
        .expect("bodies alive");
    w.spawn_joint(
        hook,
        bob,
        JointDesc {
            anchor_a: la,
            anchor_b: lb,
            ..desc
        },
    )
    .expect("joint built");
    let d0 = dist(pose(&w, hook), pose(&w, bob));
    let (mut min, mut max) = (d0, d0);
    for _ in 0..180 {
        w.step();
        let d = dist(pose(&w, hook), pose(&w, bob));
        min = min.min(d);
        max = max.max(d);
    }
    (min, max)
}

#[test]
#[ignore = "sonda de medição"]
fn measure_how_to_build_a_rigid_rod() {
    println!("\n== A rope com [d, d] (o desenho do plano) ==");
    let (min, max) = run(
        JointDesc {
            kind: JointKind::Rod,
            max_length: 2.0,
            ..Default::default()
        },
        2.0,
        0.1,
    );
    println!("  pendulo INVERTIDO: min {min:.4}  max {max:.4}   (alvo 2.0000)");

    println!("\n== Candidata A: MOTOR de posição no eixo acoplado (mola rígida) ==");
    println!("  stiffness | invertido min/max | pendurado min/max");
    for k in [1.0e3_f32, 1.0e4, 1.0e5, 1.0e6] {
        let desc = JointDesc {
            kind: JointKind::Spring,
            rest_length: 2.0,
            stiffness: k,
            damping: (2.0 * k.sqrt()) * 0.5,
            ..Default::default()
        };
        let (imin, imax) = run(desc, 2.0, 0.1);
        // A carga pendurada é PESADA (r = 0.5 => ~0.785 kg contra 0.031),
        // porque é a massa que estica uma mola.
        let (hmin, hmax) = run(desc, -2.0, 0.5);
        println!("  {k:>9.0} | {imin:.4} / {imax:.4} | {hmin:.4} / {hmax:.4}");
    }

    println!(
        "\n== Candidata B: limite [0, d] (rígido na TRAÇÃO) + motor (empurra na COMPRESSÃO) =="
    );
    println!("  stiffness | invertido min/max | pendurado min/max");
    for k in [1.0e3_f32, 1.0e4, 1.0e5, 1.0e6] {
        let desc = JointDesc {
            kind: JointKind::Rope,
            max_length: 2.0,
            motor: Some(MotorDesc {
                mode: MotorMode::Position,
                speed: 0.0,
                target: 2.0,
                max_force: f32::INFINITY,
            }),
            stiffness: k,
            ..Default::default()
        };
        let (imin, imax) = run(desc, 2.0, 0.1);
        let (hmin, hmax) = run(desc, -2.0, 0.5);
        println!("  {k:>9.0} | {imin:.4} / {imax:.4} | {hmin:.4} / {hmax:.4}");
    }

    // A rigidez se ESCOLHE por medicao: quanto ela estica sob massa crescente, e
    // se o rabo assentado ondula (um motor rigido demais para o numero de
    // sub-passos vibra, e vibrar e pior que esticar 1 mm).
    println!("\n== Escolhendo a rigidez: massa x esticamento x ondulacao do rabo ==");
    println!("  stiffness |   raio |    massa |  estica |  ripple");
    for k in [1.0e4_f32, 1.0e5, 1.0e6, 1.0e7] {
        for r in [0.1_f32, 0.5, 1.0, 2.0] {
            let mut w = PhysicsWorld::new();
            let hook = body(&mut w, RigidBodyType::Fixed, 0.0, 0.0, 0.05);
            let bob = body(&mut w, RigidBodyType::Dynamic, 0.0, -2.0, r);
            let (la, lb) = w
                .world_to_local_anchors(hook, bob, [0.0, 0.0], [0.0, -2.0])
                .expect("bodies alive");
            w.spawn_joint(
                hook,
                bob,
                JointDesc {
                    kind: JointKind::Spring,
                    rest_length: 2.0,
                    stiffness: k,
                    damping: 2.0 * k.sqrt() * 0.5,
                    anchor_a: la,
                    anchor_b: lb,
                    ..Default::default()
                },
            )
            .expect("joint built");
            let mass = std::f32::consts::PI * r * r;
            // 2 s para assentar, depois 1 s medindo o rabo.
            for _ in 0..120 {
                w.step();
            }
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for _ in 0..60 {
                w.step();
                let d = dist(pose(&w, hook), pose(&w, bob));
                lo = lo.min(d);
                hi = hi.max(d);
            }
            let stretch = (lo + hi) * 0.5 - 2.0;
            println!(
                "  {k:>9.0} | {r:>6.2} | {mass:>8.3} | {:>7.4} | {:>7.4}",
                stretch,
                hi - lo
            );
        }
    }

    // O DAMPING e load-bearing? A mutacao `ROD_DAMPING = 0` sobreviveu aos 4
    // gates, e isso ou e buraco de gate ou e um fato sobre o solver. Mede-se a
    // propriedade que a mudanca E: oscilacao.
    println!("\n== O damping do rod: ele importa? (stiffness 1e6, carga 0,785 kg) ==");
    println!("    damping | pico do transiente | ripple do rabo");
    for d in [0.0_f32, 200.0, 2000.0, 20000.0] {
        let mut w = PhysicsWorld::new();
        let hook = body(&mut w, RigidBodyType::Fixed, 0.0, 0.0, 0.05);
        let load = body(&mut w, RigidBodyType::Dynamic, 0.0, -2.0, 0.5);
        let (la, lb) = w
            .world_to_local_anchors(hook, load, [0.0, 0.0], [0.0, -2.0])
            .expect("bodies alive");
        w.spawn_joint(
            hook,
            load,
            JointDesc {
                kind: JointKind::Spring,
                rest_length: 2.0,
                stiffness: 1.0e6,
                damping: d,
                anchor_a: la,
                anchor_b: lb,
                ..Default::default()
            },
        )
        .expect("joint built");
        let mut peak = 0.0_f32;
        for _ in 0..120 {
            w.step();
            peak = peak.max((dist(pose(&w, hook), pose(&w, load)) - 2.0).abs());
        }
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..60 {
            w.step();
            let x = dist(pose(&w, hook), pose(&w, load));
            lo = lo.min(x);
            hi = hi.max(x);
        }
        println!("  {d:>9.0} | {peak:>18.6} | {:>14.6}", hi - lo);
    }

    println!("\n== Controle: a CORDA de hoje (o que um rod NÃO pode ser) ==");
    let (min, max) = run(
        JointDesc {
            kind: JointKind::Rope,
            max_length: 2.0,
            ..Default::default()
        },
        2.0,
        0.1,
    );
    println!("  pendulo INVERTIDO: min {min:.4}  max {max:.4}");
}
