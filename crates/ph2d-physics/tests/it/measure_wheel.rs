//! **Sonda: o que uma RODA faz, e com que suspensão ela nasce?**
//!
//! Roda com
//! `cargo test -p ph2d-physics --test measure_wheel -- --ignored --nocapture`.
//!
//! Quatro perguntas, nesta ordem, porque cada uma só faz sentido se a anterior
//! respondeu sim:
//!
//! 1. **A construção segura?** — a roda trava de lado (`LIN_Y`), sobe e desce na
//!    suspensão (`LinX`) e gira livre (`AngX`). Se qualquer uma falhar, o
//!    desenho está morto como o `[d, d]` do Rod esteve.
//! 2. **O limite de curso vale dos DOIS lados?** — é justamente o que o rod não
//!    conseguiu, e aqui tem de valer, porque nada está acoplado.
//! 3. **Com que rigidez ela nasce?** — a varredura que escolhe a constante.
//! 4. **Com que amortecimento?** — o quique, medido, não escolhido.
//!
//! ⚠️ **A cena é um CARRO NO CHÃO, e a primeira versão desta sonda não era.**
//! Sem chão, chassi e roda estão os DOIS em queda livre: eles caem *juntos*, a
//! distância entre eles não muda e a suspensão mede **zero afundamento em toda
//! rigidez** — uma tabela inteira de zeros, verde e vazia. Uma suspensão só
//! comprime quando a roda está APOIADA e o peso do chassi desce sobre ela.
//!
//! ⚠️ **E o desalinhamento lateral é medido no frame do CHASSI**, não no mundo:
//! o eixo travado é a perpendicular à suspensão *no corpo A*, então um chassi
//! que gira leva o eixo livre junto — na primeira versão isso apareceu como
//! 0,475 m de "violação" de uma restrição que estava sendo honrada.
//!
//! ⚠️ Tudo pela porta do PRODUTO (`spawn_joint` com um `JointDesc`), nunca por
//! uma segunda construção escrita aqui: uma tabela medida sobre um joint que
//! ninguém shipa é uma tabela sobre outra coisa.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyHandle,
    RigidBodyType, ShapeDesc,
};

/// Meia-altura do chassi, metros — e a régua de tudo o que a sonda imprime.
const CHASSIS_HALF_Y: f32 = 0.2;
/// Meia-largura do chassi.
const CHASSIS_HALF_X: f32 = 1.0;
/// Raio da roda, metros.
const WHEEL_R: f32 = 0.3;
/// Altura de marcha AUTORADA: o quanto o cubo fica abaixo do centro do chassi
/// quando o artista monta o carro. A suspensão parte daqui e AFUNDA.
const RIDE: f32 = 0.5;
/// Altura do chão.
const GROUND_Y: f32 = 0.0;

#[allow(clippy::too_many_arguments)]
fn body(
    w: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    density: f32,
    linvel: [f32; 2],
) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density,
        shape,
        restitution: 0.0,
        friction: 1.0,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel,
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

fn pose(w: &PhysicsWorld, h: RigidBodyHandle) -> [f32; 3] {
    let p = w.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y, p.rotation.angle()]
}

/// Um carro: chão + chassi + duas rodas, cada uma por um joint Wheel de
/// suspensão vertical. Devolve `(mundo, chassi, [roda esquerda, direita])`.
///
/// A roda nasce APOIADA no chão (`GROUND_Y + WHEEL_R`), e o chassi um `RIDE`
/// acima dela: é a pose que o artista monta, e é dela que a suspensão afunda.
fn car(
    stiffness: f32,
    damping: f32,
    limits: Option<[f32; 2]>,
    motor: Option<MotorDesc>,
    density: f32,
    push_x: f32,
) -> (PhysicsWorld, RigidBodyHandle, [RigidBodyHandle; 2]) {
    let mut w = PhysicsWorld::new();
    body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        GROUND_Y - 0.5,
        ShapeDesc::Cuboid {
            half_x: 200.0,
            half_y: 0.5,
        },
        1.0,
        [0.0, 0.0],
    );
    let hub_y = GROUND_Y + WHEEL_R;
    let chassis = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        hub_y + RIDE,
        ShapeDesc::Cuboid {
            half_x: CHASSIS_HALF_X,
            half_y: CHASSIS_HALF_Y,
        },
        density,
        [push_x, 0.0],
    );
    let mut wheels = [chassis; 2];
    for (i, hub_x) in [-0.7_f32, 0.7].into_iter().enumerate() {
        let wheel = body(
            &mut w,
            RigidBodyType::Dynamic,
            hub_x,
            hub_y,
            ShapeDesc::Ball { radius: WHEEL_R },
            1.0,
            [push_x, 0.0],
        );
        // O cubo É a âncora — os dois corpos compartilham o ponto, como um Pin.
        let (la, lb) = w
            .world_to_local_anchors(chassis, wheel, [hub_x, hub_y], [hub_x, hub_y])
            .expect("bodies alive");
        w.spawn_joint(
            chassis,
            wheel,
            JointDesc {
                kind: JointKind::Wheel,
                anchor_a: la,
                anchor_b: lb,
                // A suspensão é VERTICAL: o eixo livre aponta para cima.
                axis_a: [0.0, 1.0],
                axis_b: [0.0, 1.0],
                stiffness,
                damping,
                limits,
                motor,
                ..Default::default()
            },
        )
        .expect("wheel joint");
        wheels[i] = wheel;
    }
    (w, chassis, wheels)
}

/// A altura de marcha VIVA: o quanto o cubo está abaixo do centro do chassi,
/// **medida ao longo do eixo da suspensão** (o local +Y do chassi).
fn ride(w: &PhysicsWorld, chassis: RigidBodyHandle, wheel: RigidBodyHandle) -> f32 {
    -local_offset(w, chassis, wheel)[1]
}

/// O cubo visto do frame do CHASSI. `[0]` é o eixo TRAVADO (a perpendicular),
/// `[1]` é o eixo LIVRE (a suspensão).
fn local_offset(w: &PhysicsWorld, chassis: RigidBodyHandle, wheel: RigidBodyHandle) -> [f32; 2] {
    let c = pose(w, chassis);
    let p = pose(w, wheel);
    let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
    let (s, co) = (-c[2]).sin_cos();
    [dx * co - dy * s, dx * s + dy * co]
}

/// Corre `ticks` e devolve `(altura final, MENOR altura vista, MAIOR)`.
fn settle(
    w: &mut PhysicsWorld,
    chassis: RigidBodyHandle,
    wheel: RigidBodyHandle,
    ticks: usize,
) -> (f32, f32, f32) {
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for _ in 0..ticks {
        w.step();
        let r = ride(w, chassis, wheel);
        lo = lo.min(r);
        hi = hi.max(r);
    }
    (ride(w, chassis, wheel), lo, hi)
}

#[test]
#[ignore = "sonda de medição: imprime as tabelas da roda"]
fn probe_the_wheel_holds_slides_and_spins() {
    println!("\n== 1. A CONSTRUÇÃO ==");

    // (a) DESLIZA: o carro pousa e a suspensão cede sob o peso do chassi.
    let (mut w, c, wh) = car(200.0, 20.0, None, None, 1.0, 0.0);
    let (end, lo, _hi) = settle(&mut w, c, wh[0], 240);
    println!(
        "  suspensao : autorada {RIDE:.3} m -> assenta em {end:.4} (minimo {lo:.4}) => afunda {:.4} m",
        RIDE - end
    );

    // (b) TRAVA DE LADO: o carro parte lançado; a roda tem de vir junto. Medido
    // no frame do chassi, que é onde o eixo travado mora.
    let (mut w, c, wh) = car(200.0, 20.0, None, None, 1.0, 3.0);
    let mut worst = 0.0_f32;
    for _ in 0..180 {
        w.step();
        worst = worst.max(local_offset(&w, c, wh[0])[0].abs() - 0.7);
    }
    println!(
        "  lateral   : o cubo desvia no maximo {worst:.6} m do lugar dele no chassi \
         (carro andou ate x={:.2})",
        pose(&w, c)[0]
    );

    // (c) GIRA: com motor no eixo angular a roda tem de rodar E o carro andar.
    let (mut w, c, wh) = car(
        200.0,
        20.0,
        None,
        Some(MotorDesc {
            mode: MotorMode::Velocity,
            speed: -8.0,
            target: 0.0,
            max_force: 50.0,
        }),
        1.0,
        0.0,
    );
    for _ in 0..180 {
        w.step();
    }
    let snap = w.body_snapshots();
    let spin = snap
        .iter()
        .find(|s| s.handle_index == wh[0].into_raw_parts().0)
        .map(|s| s.angvel)
        .unwrap_or(0.0);
    println!(
        "  giro      : roda a {spin:.4} rad/s (pedido -8.0) e o carro andou ate x={:.4}",
        pose(&w, c)[0]
    );

    println!("\n== 2. O CURSO tem os DOIS batentes? ==");
    // Um curso apertado: se o batente valer, a suspensão para de afundar nele.
    // ⚠️ O sinal do curso segue o EIXO (para cima é +), então comprimir é
    // NEGATIVO: o batente que morde é o `min`, que é o que o rod não tinha.
    for lim in [[-0.20_f32, 0.20], [-0.05, 0.05], [-0.01, 0.01]] {
        let (mut w, c, wh) = car(30.0, 5.0, Some(lim), None, 4.0, 0.0);
        let (end, lo, hi) = settle(&mut w, c, wh[0], 240);
        println!(
            "  limite [{:+.2}, {:+.2}] : assenta {end:.4} (faixa {lo:.4}..{hi:.4}), afunda {:.4}",
            lim[0],
            lim[1],
            RIDE - end
        );
    }
    let (mut w, c, wh) = car(30.0, 5.0, None, None, 4.0, 0.0);
    let (end, _lo, _hi) = settle(&mut w, c, wh[0], 240);
    println!(
        "  sem limite            : assenta {end:.4}, afunda {:.4}",
        RIDE - end
    );

    println!("\n== 3. RIGIDEZ: quanto a suspensao afunda sob o carro ==");
    println!("  (chassi 2.0x0.4 m: densidade 1 => 0.8 kg, densidade 4 => 3.2 kg; 2 rodas)");
    for k in [30.0_f32, 60.0, 100.0, 200.0, 400.0, 800.0, 1600.0] {
        let mut line = format!("  k={k:>6.0} :");
        for (label, density) in [("leve", 1.0_f32), ("pesado", 4.0)] {
            let (mut w, c, wh) = car(k, 20.0, None, None, density, 0.0);
            let (end, lo, _hi) = settle(&mut w, c, wh[0], 300);
            line.push_str(&format!(
                "  {label} afunda {:.4} (pico {:.4})",
                RIDE - end,
                RIDE - lo
            ));
        }
        println!("{line}");
    }

    println!("\n== 5. UMA RODA REPORTA TORQUE? (a linha de ruptura) ==");
    // A pergunta que decide se a row "Break Torque" existe para este tipo: um
    // eixo angular MOTORIZADO reporta reação (é o que um Pin com servo faz), um
    // eixo LIVRE não (é o que uma Rope e uma Spring fazem). Uma roda tem o eixo
    // livre quando o motor está desligado e motorizado quando está — então a
    // resposta tem de ser medida nos dois estados, e é o segundo que manda.
    for (label, motor) in [
        ("sem motor", None),
        (
            "com motor",
            Some(MotorDesc {
                mode: MotorMode::Velocity,
                speed: -8.0,
                target: 0.0,
                max_force: 50.0,
            }),
        ),
    ] {
        let mut w = PhysicsWorld::new();
        body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            GROUND_Y - 0.5,
            ShapeDesc::Cuboid {
                half_x: 200.0,
                half_y: 0.5,
            },
            1.0,
            [0.0, 0.0],
        );
        let hub_y = GROUND_Y + WHEEL_R;
        let chassis = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            hub_y + RIDE,
            ShapeDesc::Cuboid {
                half_x: CHASSIS_HALF_X,
                half_y: CHASSIS_HALF_Y,
            },
            4.0,
            [0.0, 0.0],
        );
        let wheel = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            hub_y,
            ShapeDesc::Ball { radius: WHEEL_R },
            1.0,
            [0.0, 0.0],
        );
        let (la, lb) = w
            .world_to_local_anchors(chassis, wheel, [0.0, hub_y], [0.0, hub_y])
            .expect("bodies alive");
        let h = w
            .spawn_joint(
                chassis,
                wheel,
                JointDesc {
                    kind: JointKind::Wheel,
                    anchor_a: la,
                    anchor_b: lb,
                    axis_a: [0.0, 1.0],
                    axis_b: [0.0, 1.0],
                    stiffness: 400.0,
                    damping: 20.0,
                    motor,
                    ..Default::default()
                },
            )
            .expect("wheel joint");
        for _ in 0..120 {
            w.step();
        }
        let load = w.joint_load(h).expect("joint vivo");
        println!(
            "  {label} : forca {:.4} N, torque {:.4} N.m",
            load.force, load.torque
        );
    }

    println!("\n== 4. AMORTECIMENTO: o carro pousa, e depois? ==");
    println!("  ultrapassagem = pico/repouso (1.00 = nao quica) · assenta = tick em que");
    println!("  a altura para de mudar mais de 1 mm por 10 ticks (300 = nunca assentou)");
    for k in [200.0_f32, 400.0, 800.0] {
        for d in [0.0_f32, 5.0, 10.0, 20.0, 40.0, 80.0] {
            let (mut w, c, wh) = car(k, d, None, None, 1.0, 0.0);
            let mut track = Vec::new();
            for _ in 0..300 {
                w.step();
                track.push(RIDE - ride(&w, c, wh[0]));
            }
            let rest = *track.last().expect("300 ticks");
            let peak = track.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let quiet = (0..track.len().saturating_sub(10))
                .rev()
                .take_while(|&i| (track[i] - track[i + 10]).abs() < 0.001)
                .last()
                .unwrap_or(track.len());
            println!(
                "  k={k:>4.0} d={d:>5.1} : repouso {rest:.4}, pico {peak:.4} \
                 => ultrapassagem {:.2}x, assenta no tick {quiet}",
                if rest.abs() > 1e-6 {
                    peak / rest
                } else {
                    f32::NAN
                }
            );
        }
    }
}
