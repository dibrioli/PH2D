//! **A SONDA DO REPOUSO** — o que o personagem faz quando ninguém toca nele.
//!
//! Dois relatos do Enio no smoke da W10, e os dois são sobre o MESMO gesto (não
//! fazer nada): *"de tempos em tempos enquanto está parado o player dá pulinhos
//! involuntários"* e *"nas rampas, se parado, a depender do Float Height ele pode
//! subir a rampa sozinho bem devagar"*.
//!
//! ⚠️ Esta sonda **não conserta nada** — ela mede, pelas portas do produto
//! (`dispatch`), para que a atribuição venha de um número e não de uma hipótese.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_idle -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use scene_fixture::pose;

/// Uma cena de UMA rampa + o player, com a altura de flutuação escolhida.
fn rig(slope_deg: f32, float_height: f32) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let slope = slope_deg.to_radians();
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            // ⚠️ **200 e não 40, e a fixture ANTERIOR mentia:** a 6 m/s o
            // personagem cruza 48 m em oito segundos, **cai da beirada** de um
            // chão de 40, e a sonda de subida media a queda junto — ela reportou
            // `1,181 ×` a velocidade autorada no PLANO, onde o certo é 1,000.
            shape: ColliderShape::Cuboid {
                half_x: 200.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform {
            rotation: slope,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let top = 0.5 / slope.cos();
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, top + float_height)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
}

/// Roda `ticks` tiques SEM entrada nenhuma e devolve a trajetória.
fn idle(slope_deg: f32, float_height: f32, ticks: u64) -> Vec<(f32, f32)> {
    let (mut sim, mut bridge, _player) = rig(slope_deg, float_height);
    let mut out = Vec::with_capacity(ticks as usize);
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, true, t);
        out.push(pose(&sim));
    }
    out
}

#[test]
#[ignore = "sonda"]
fn measure_what_an_idle_player_does() {
    println!("\n=== O REPOUSO: o personagem parado, sem entrada nenhuma ===");
    println!("(capsula half_height 0.3 / radius 0.2; o minimo geometrico no plano e' 0.5)\n");

    // ⚠️ O mínimo geométrico no PLANO é `half_height + radius` = 0,5. Abaixo
    // dele a cápsula ENCOSTA, e quem responde deixa de ser a mola.
    println!("--- plano, VARRENDO abaixo do minimo geometrico (0,5) ---");
    println!(
        "{:>7}  {:>10}  {:>10}  {:>12}",
        "float", "deriva x", "amplitude", "saltos>2mm"
    );
    for &fh in &[0.25_f32, 0.35, 0.45, 0.49, 0.50, 0.55] {
        let path = idle(0.0, fh, 600);
        let tail = &path[120..];
        let ymin = tail.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let ymax = tail.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        let hops = tail
            .windows(2)
            .filter(|w| (w[1].1 - w[0].1).abs() > 2.0e-3)
            .count();
        println!(
            "{fh:>7.2}  {:>10.4}  {:>10.5}  {hops:>12}",
            path.last().unwrap().0 - path[0].0,
            ymax - ymin
        );
    }
    println!();

    for &slope in &[0.0_f32, 10.0, 20.0, 30.0] {
        println!("--- rampa {slope:.0}° ---");
        println!(
            "{:>7}  {:>10}  {:>10}  {:>10}  {:>12}",
            "float", "deriva x", "amplitude", "y final", "subida/s"
        );
        for &fh in &[0.5_f32, 0.6, 0.7, 0.9, 1.2] {
            let path = idle(slope, fh, 600);
            let x0 = path[0].0;
            let (xe, ye) = *path.last().unwrap();
            // A amplitude do repouso: pico a pico do y depois de assentar.
            let tail = &path[120..];
            let ymin = tail.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
            let ymax = tail.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
            // A subida ao longo da rampa, em metros por segundo de relógio.
            let along = (xe - x0) / slope.to_radians().cos();
            println!(
                "{fh:>7.2}  {:>10.4}  {:>10.5}  {:>10.4}  {:>12.5}",
                xe - x0,
                ymax - ymin,
                ye,
                along / 10.0
            );
        }
        println!();
    }
}

/// **A pergunta que decide tudo: a oscilacao MORRE ou e' um ciclo-limite?**
///
/// Um transiente que assenta é higiene; um ciclo que se sustenta é um motor, e um
/// motor acoplado a uma rampa é a subida que o Enio viu.
#[test]
#[ignore = "sonda"]
fn measure_whether_the_idle_wobble_dies_or_sustains() {
    println!("\n=== O CICLO: a amplitude e a deriva por JANELA de 2 s ===");
    for &slope in &[10.0_f32, 20.0, 30.0] {
        let path = idle(slope, 0.9, 1800); // 30 s
        println!("--- rampa {slope:.0}°, float 0.9 ---");
        println!("{:>10}  {:>12}  {:>12}", "janela", "amplitude", "deriva x");
        for w in 0..15 {
            let a = w * 120;
            let b = a + 120;
            let seg = &path[a..b];
            let ymin = seg.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
            let ymax = seg.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
            println!(
                "{:>4}-{:>4}s  {:>12.5}  {:>12.5}",
                a / 60,
                b / 60,
                ymax - ymin,
                seg.last().unwrap().0 - seg[0].0
            );
        }
        println!();
    }
}

/// **O INTERIOR do ciclo** — 40 tiques da fase assentada, tique a tique.
#[test]
#[ignore = "sonda"]
fn measure_the_inside_of_the_cycle() {
    let (mut sim, mut bridge, _player) = rig(30.0, 0.9);
    for t in 1..=600 {
        bridge.dispatch(&mut sim, true, t);
    }
    println!("\n=== O INTERIOR DO CICLO (rampa 30°, float 0.9, assentado) ===");
    println!(
        "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>10}",
        "tique", "x", "y", "vx", "vy", "perp"
    );
    // A folga PERPENDICULAR ao chao: o que a capsula de fato tem por baixo.
    let n = [
        -(30.0_f32.to_radians()).sin(),
        (30.0_f32.to_radians()).cos(),
    ];
    let (mut px, mut py) = pose(&sim);
    for t in 601..=640 {
        bridge.dispatch(&mut sim, true, t);
        let (x, y) = pose(&sim);
        let (vx, vy) = ((x - px) * 60.0, (y - py) * 60.0);
        (px, py) = (x, y);
        // Distancia do centro ao plano do topo do chao (que passa pela origem
        // deslocada de 0.5 ao longo da normal).
        let perp = x * n[0] + y * n[1] - 0.5;
        println!("{t:>5}  {x:>10.5}  {y:>10.5}  {vx:>10.5}  {vy:>10.5}  {perp:>10.5}");
    }
}

/// **A ATRIBUIÇÃO — por ablação pelas ENTRADAS**, nunca por instrumentação.
///
/// Cada linha desliga UM termo pelo knob que o artista tem na mão, e a que mudar
/// a deriva nomeia o culpado.
#[test]
#[ignore = "sonda"]
fn measure_which_term_carries_the_player_uphill() {
    let base = PlatformPlayer {
        float_height: 0.9,
        ..PlatformPlayer::default()
    };
    let variants: [(&str, PlatformPlayer); 7] = [
        ("controle", base),
        (
            "sem caminhada (accel 0)",
            PlatformPlayer {
                acceleration: 0.0,
                ..base
            },
        ),
        (
            "sem amortecimento",
            PlatformPlayer {
                spring_damping: 0.0,
                ..base
            },
        ),
        (
            "amortecimento cheio",
            PlatformPlayer {
                spring_damping: 1.0,
                ..base
            },
        ),
        (
            "sem reacao (support 0)",
            PlatformPlayer {
                reaction_support: 0.0,
                ..base
            },
        ),
        (
            "rampa RECUSADA (slope 20)",
            PlatformPlayer {
                max_slope_deg: 20.0,
                ..base
            },
        ),
        (
            "mola mole (k 40)",
            PlatformPlayer {
                spring_strength: 40.0,
                ..base
            },
        ),
    ];
    // ⚠️ **A RIGIDEZ paga o erro de repouso.** O erro vale `d · g · dt / 2 / k`,
    // então subir `k` o encolhe — e o erro de repouso é o que rouba PESO do
    // personagem (o `react` devolve `spring.accel = g + k·offset`, e um `offset`
    // negativo devolve menos que o peso).
    println!("\n=== A RIGIDEZ: erro de repouso, peso transmitido e deriva (d = 1,0) ===");
    println!(
        "{:>8}  {:>12}  {:>12}  {:>12}",
        "k", "erro (mm)", "peso", "deriva 30°"
    );
    for &k in &[400.0_f32, 800.0, 1600.0, 3200.0, 6400.0] {
        // O erro de repouso, no plano.
        let (mut sim, mut bridge, player) = rig(0.0, 0.9);
        sim.world_mut()
            .get_mut::<PlatformPlayer>(player)
            .unwrap()
            .spring_strength = k;
        for t in 1..=900 {
            bridge.dispatch(&mut sim, true, t);
        }
        let err = (pose(&sim).1 - 0.5) - 0.9;
        // A deriva, na rampa.
        let (mut s2, mut b2, p2) = rig(30.0, 0.9);
        s2.world_mut()
            .get_mut::<PlatformPlayer>(p2)
            .unwrap()
            .spring_strength = k;
        for t in 1..=120 {
            b2.dispatch(&mut s2, true, t);
        }
        let x0 = pose(&s2).0;
        for t in 121..=720 {
            b2.dispatch(&mut s2, true, t);
        }
        println!(
            "{k:>8.0}  {:>12.3}  {:>11.1}%  {:>12.5}",
            err * 1000.0,
            (9.81 + k * err) / 9.81 * 100.0,
            pose(&s2).0 - x0
        );
    }

    // ⚠️ **O PREÇO do amortecimento cheio:** se `1.0` zera a deriva, ele tem de
    // pousar bem — senão a cura é um personagem que bate no chão como pedra.
    println!("\n=== O POUSO: cair de 3 m e assentar (plano) ===");
    println!(
        "{:>10}  {:>12}  {:>12}  {:>12}",
        "damping", "quique (mm)", "tiques ate' assentar", "folga final"
    );
    for &d in &[0.25_f32, 0.5, 0.75, 1.0] {
        let (mut sim, mut bridge, player) = rig(0.0, 0.9);
        {
            let mut p = sim.world_mut().get_mut::<PlatformPlayer>(player).unwrap();
            p.spring_damping = d;
        }
        sim.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .y = 4.4;
        let mut settled = 0;
        let mut over = 0.0_f32;
        let mut prev = f32::NAN;
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
            let gap = pose(&sim).1 - 0.5;
            // O quique: quanto a folga passa da pedida DEPOIS de ter chegado.
            if gap < 0.9 {
                over = over.max(0.9 - gap);
            }
            if (gap - prev).abs() < 1.0e-5 && settled == 0 && t > 30 {
                settled = t;
            }
            prev = gap;
        }
        println!(
            "{d:>10.2}  {:>12.2}  {settled:>20}  {:>12.6}",
            over * 1000.0,
            pose(&sim).1 - 0.5
        );
    }

    // ⚠️ **A pergunta decisiva:** a deriva sobrevive num mundo SEM GRAVIDADE?
    // Sem gravidade a perna não tem o que cancelar, então o impulso de
    // cancelamento — aplicado no topo do tique para uma força que age ATRAVÉS
    // dele — deixa de existir. Se a deriva morre aqui, é ele.
    println!("\n=== SEM GRAVIDADE: a deriva sobrevive? (30°, 10 s) ===");
    for &g in &[-9.81_f32, -4.0, -1.0, 0.0] {
        let (mut sim, mut bridge, _p) = rig(30.0, 0.9);
        bridge.set_gravity(0.0, g);
        let x0 = pose(&sim).0;
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!(
            "  gravidade {g:>6.2}  ->  deriva x {:>9.4}",
            pose(&sim).0 - x0
        );
    }

    println!("\n=== O ERRO DE REPOUSO: a folga que a perna de fato segura (PLANO) ===");
    println!(
        "{:>10}  {:>12}  {:>12}  {:>12}",
        "damping", "folga", "pedida", "erro (mm)"
    );
    for &d in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let (mut sim, mut bridge, player) = rig(0.0, 0.9);
        sim.world_mut()
            .get_mut::<PlatformPlayer>(player)
            .unwrap()
            .spring_damping = d;
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        // O topo do chao plano esta' em y = 0.5; a folga e' o que o raio mede.
        let held = pose(&sim).1 - 0.5;
        println!(
            "{d:>10.2}  {held:>12.6}  {:>12.6}  {:>12.3}",
            0.9,
            (held - 0.9) * 1000.0
        );
    }

    println!("\n=== O AMORTECIMENTO decide? (30°, 10 s) ===");
    println!("{:>10}  {:>10}", "damping", "deriva x");
    for &d in &[0.0_f32, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let (mut sim, mut bridge, player) = rig(30.0, 0.9);
        sim.world_mut()
            .get_mut::<PlatformPlayer>(player)
            .unwrap()
            .spring_damping = d;
        let x0 = pose(&sim).0;
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        println!("{d:>10.2}  {:>10.4}", pose(&sim).0 - x0);
    }

    println!("\n=== ATRIBUICAO: quem carrega o personagem rampa acima (30°, 10 s) ===");
    println!("{:>28}  {:>10}  {:>10}", "variante", "deriva x", "y final");
    for (name, cfg) in variants {
        let slope = 30.0_f32.to_radians();
        let (mut sim, mut bridge, player) = rig(30.0, cfg.float_height);
        *sim.world_mut().get_mut::<PlatformPlayer>(player).unwrap() = cfg;
        let _ = slope;
        let x0 = pose(&sim).0;
        for t in 1..=600 {
            bridge.dispatch(&mut sim, true, t);
        }
        let (x, y) = pose(&sim);
        println!("{name:>28}  {:>10.4}  {y:>10.4}", x - x0);
    }
}

/// **O PULINHO — onde ele mora.** Quatro chãos, o mesmo gesto: nenhum.
///
/// O plano estático saiu perfeitamente parado na primeira sonda, então o
/// candidato é o chão que se MOVE: o vagão da cena 81 é KINEMATIC dirigido pela
/// timeline, e a velocidade dele que o sensor lê vem do solver.
#[test]
#[ignore = "sonda"]
fn measure_where_the_idle_hop_lives() {
    println!("\n=== O PULINHO: a folga PERPENDICULAR ao chao, tique a tique ===");

    // (a) chão estático plano — o controle.
    report("plano estatico", &plat_ride(Platform::Static, 600));
    // (b) plataforma DINÂMICA viajando a 3 m/s (o vagão da cena 90).
    report("vagao dinamico 3 m/s", &plat_ride(Platform::Dynamic, 600));
    // (c) plataforma KINEMATIC dirigida por pose (o vagão da cena 81).
    report("vagao kinematic", &plat_ride(Platform::Kinematic, 600));
}

enum Platform {
    Static,
    Dynamic,
    Kinematic,
}

/// Uma plataforma horizontal + o player em cima; devolve a folga vertical
/// (`y_player − y_topo_da_plataforma`) por tique.
fn plat_ride(kind: Platform, ticks: u64) -> Vec<f32> {
    use ph2d_physics_ecs::{GravityScale, InitialVelocity, MassOverride};
    let mut sim = SimWorld::new();
    let top = 0.25_f32;
    let mut plat = sim.world_mut().spawn((
        Name::new("Plat"),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: top,
            },
            ..Collider::default()
        },
    ));
    match kind {
        Platform::Static => {
            plat.insert(RigidBody {
                kind: BodyKind::Static,
            });
        }
        Platform::Dynamic => {
            plat.insert((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                LockRotation,
                GravityScale(0.0),
                MassOverride(1000.0),
                InitialVelocity {
                    linvel: [3.0, 0.0],
                    angvel: 0.0,
                },
            ));
        }
        Platform::Kinematic => {
            plat.insert(RigidBody {
                kind: BodyKind::Kinematic,
            });
        }
    }
    let plat = plat.id();
    sim.world_mut().spawn((
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: 0.9,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, top + 0.9)),
    ));
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::with_capacity(ticks as usize);
    for t in 1..=ticks {
        if matches!(kind, Platform::Kinematic) {
            // O que a timeline faria: escrever a pose do tique.
            let x = 3.0 * (t as f32) / 60.0;
            sim.world_mut()
                .get_mut::<Transform>(plat)
                .unwrap()
                .translation
                .x = x;
        }
        bridge.dispatch(&mut sim, true, t);
        let py = pose(&sim).1;
        let plat_y = sim.world().get::<Transform>(plat).unwrap().translation.y;
        out.push(py - (plat_y + top));
    }
    out
}

fn report(name: &str, gap: &[f32]) {
    let tail = &gap[180..];
    let lo = tail.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = tail.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let hops = tail
        .windows(2)
        .filter(|w| (w[1] - w[0]).abs() > 2.0e-3)
        .count();
    println!(
        "{name:>22}:  folga [{lo:.5}, {hi:.5}]  amplitude {:.5}  saltos>2mm/tique {hops}",
        hi - lo
    );
}

/// **O relógio REAL não deve um tique por quadro** — a última diferença entre
/// esta fixture e o app: a 60 fps com jitter, um dispatch deve 0, 1 ou 2 tiques.
#[test]
#[ignore = "sonda"]
fn measure_an_idle_player_under_a_jittery_clock() {
    println!("\n=== O RELOGIO IRREGULAR (plano, float 0.9) ===");
    // A cadência de um quadro real: às vezes nenhum tique, às vezes dois.
    let owed = [1_u64, 1, 2, 0, 1, 1, 0, 2, 1, 1, 1, 2, 0, 1];
    let (mut sim, mut bridge, _p) = rig(0.0, 0.9);
    let mut target = 0_u64;
    let mut ys = Vec::new();
    for i in 0..1200 {
        target += owed[i % owed.len()];
        bridge.dispatch(&mut sim, true, target);
        ys.push(pose(&sim).1);
    }
    let tail = &ys[300..];
    let lo = tail.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = tail.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let hops = tail
        .windows(2)
        .filter(|w| (w[1] - w[0]).abs() > 2.0e-3)
        .count();
    println!(
        "y em [{lo:.6}, {hi:.6}]  amplitude {:.6}  saltos>2mm {hops}",
        hi - lo
    );
}

/// **A SEGUNDA consequência do mesmo eixo: subir a rampa é FREADO.**
///
/// O amortecedor da perna remove `d · (v · up)` por tique. Andando ao longo da
/// rampa `v · up = v · sen θ`, então quanto mais íngreme a rampa, mais o próprio
/// amortecedor come a subida — um freio que ninguém autorou.
#[test]
#[ignore = "sonda"]
fn measure_how_fast_the_player_climbs() {
    println!("\n=== SUBIR: a velocidade ao longo do chao, com o dedo no acelerador ===");
    println!(
        "{:>7}  {:>12}  {:>12}  {:>10}",
        "rampa", "v ao longo", "alvo", "fracao"
    );
    for &slope in &[0.0_f32, 10.0, 20.0, 30.0, 40.0] {
        let (mut sim, mut bridge, player) = rig(slope, 0.9);
        // Subir a rampa que sobe para a DIREITA e' `drive = +1`.
        for t in 1..=300 {
            bridge.dispatch(&mut sim, true, t);
            bridge.set_player_input(
                player,
                ph2d_platformer::PlayerInput {
                    drive: 1.0,
                    jump: false,
                    down: false,
                },
            );
        }
        let a = pose(&sim);
        for t in 301..=480 {
            bridge.dispatch(&mut sim, true, t);
        }
        let b = pose(&sim);
        let along = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() / 3.0;
        let target = PlatformPlayer::default().speed;
        println!(
            "{slope:>6.0}°  {along:>12.4}  {target:>12.4}  {:>10.3}",
            along / target
        );
    }
}

/// O "pulinho": o personagem parado chega a **sair do chão**?
#[test]
#[ignore = "sonda"]
fn measure_whether_an_idle_player_leaves_the_ground() {
    println!("\n=== O PULINHO: a trajetoria de y, tique a tique (plano, float 0.9) ===");
    let path = idle(0.0, 0.9, 400);
    let ymin = path[120..]
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min);
    let ymax = path[120..]
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max);
    println!(
        "assentado: y em [{ymin:.6}, {ymax:.6}]  amplitude {:.6}",
        ymax - ymin
    );
    println!("\n  tique        y          dy");
    let mut prev = path[0].1;
    for (i, &(_, y)) in path.iter().enumerate() {
        let dy = y - prev;
        prev = y;
        // Imprime só o que se destaca: os saltos maiores que meio milimetro.
        if i > 120 && dy.abs() > 5.0e-4 {
            println!("{i:>7}  {y:>10.6}  {dy:>10.6}");
        }
    }
    println!("(nenhuma linha acima = nenhum salto maior que 0,5 mm por tique)");
}
