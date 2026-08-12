//! **O CENSO DOS SENSORES** (`W-Probes`) — com que frequência cada um é de facto
//! lançado, e o que custa gravar o que eles olharam.
//!
//! ⚠️ **A primeira medição decide o DESENHO**, e é por isso que ela vem antes do
//! overlay: se os três sensores condicionais fossem lançados na maioria dos
//! tiques, desenhar *só o que foi perguntado* bastaria. Se forem raros, o
//! desenho tem de mostrar o **ALCANCE** sempre e usar a cor para dizer o que
//! aconteceu — que é o [`ProbeState::Idle`] da wave.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-physics-ecs --release --test measure_player_probes -- --ignored --nocapture
//! ```

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    ProbeKind, ProbeState, RigidBody,
};

const FLOAT: f32 = 0.9;

/// Um chão comprido, uma parede à direita, um teto baixo no meio.
///
/// ⚠️ **A cena tem de conter os TRÊS fenômenos** — chão para caminhar, parede
/// para o flanco, teto para o agachar — senão o censo mede a ausência da
/// fixture em vez da raridade do sensor.
fn scene(armed: bool) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let mut slab = |name: &str, at: Vec2, half: [f32; 2]| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                ..Collider::default()
            },
            Transform::from_translation(at),
        ));
    };
    slab("Floor", Vec2::new(0.0, -0.5), [40.0, 0.5]);
    slab("Wall", Vec2::new(12.0, 2.0), [0.5, 3.0]);
    slab("Ceiling", Vec2::new(6.0, 2.2), [2.0, 0.5]);

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
                float_height: FLOAT,
                // As três capacidades condicionais, ligadas ou não — é esta a
                // ablação por ENTRADA que separa o custo do recorder.
                wall_slide_speed: if armed { 2.0 } else { 0.0 },
                wall_jump_height: if armed { 2.0 } else { 0.0 },
                corner_reach: if armed { 0.12 } else { 0.0 },
                crouch_height: if armed { 0.5 } else { 0.0 },
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
        ))
        .id();

    (sim, PhysicsBridge::new(), player)
}

/// Anda para a direita, pula de vez em quando, agacha de vez em quando.
fn script(i: u64) -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        jump: i.is_multiple_of(40),
        down: (60..80).contains(&(i % 200)),
        ..PlayerInput::default()
    }
}

/// **QUANTAS VEZES cada sensor é de facto perguntado** — o número que decide se
/// desenhar só o que foi castado bastaria.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_how_often_each_sensor_is_actually_cast() {
    let (mut sim, mut bridge, player) = scene(true);
    let ticks = 600u64;
    let mut seen = [[0usize; 3]; 5]; // [kind][state]

    for i in 1..=ticks {
        bridge.set_player_input(player, script(i));
        bridge.dispatch(&mut sim, true, i);
        for m in bridge.player_probe_marks() {
            let k = match m.kind {
                ProbeKind::Ground => 0,
                ProbeKind::Wall => 1,
                ProbeKind::Corner => 2,
                ProbeKind::Side => 3,
                ProbeKind::Headroom => 4,
            };
            let s = match m.state {
                ProbeState::Idle => 0,
                ProbeState::Clear => 1,
                ProbeState::Hit => 2,
            };
            seen[k][s] += 1;
        }
    }

    println!("\n== censo dos sensores ({ticks} tiques, personagem a andar/pular/agachar) ==");
    println!(
        "{:<10} {:>8} {:>8} {:>8}   perguntado",
        "sensor", "idle", "clear", "hit"
    );
    for (k, name) in ["chao", "parede", "quina", "lado", "agachar"]
        .iter()
        .enumerate()
    {
        let [idle, clear, hit] = seen[k];
        let total = idle + clear + hit;
        let asked = clear + hit;
        let pct = if total == 0 {
            0.0
        } else {
            100.0 * asked as f32 / total as f32
        };
        println!("{name:<10} {idle:>8} {clear:>8} {hit:>8}   {pct:>5.1}%");
    }
    println!(
        "\nLeitura: um sensor com 'perguntado' baixo desenharia NADA em quase todo\n\
         quadro se o overlay mostrasse apenas o que foi castado."
    );
    // (a sonda mede; quem afirma e' o `player_probe_view`.)
}

/// **O QUE CUSTA gravar a leitura** — ablação por ENTRADA (as capacidades),
/// porque é assim que o produto liga e desliga as portas de geometria.
///
/// ⚠️ Com tudo desarmado o recorder grava **uma** marca (a perna) e não chama
/// `body_aabb` nenhuma vez; armado ele grava oito e chama até três. A diferença
/// entre as duas colunas é o preço do canal.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_the_reading_costs() {
    let ticks = 2000u64;
    let row = |label: &str, armed: bool| {
        let (mut sim, mut bridge, player) = scene(armed);
        // Aquece: o primeiro tique constrói o mundo rapier.
        for i in 1..=20 {
            bridge.set_player_input(player, script(i));
            bridge.dispatch(&mut sim, true, i);
        }
        let t = Instant::now();
        for i in 21..=ticks {
            bridge.set_player_input(player, script(i));
            bridge.dispatch(&mut sim, true, i);
        }
        let per = t.elapsed().as_secs_f64() * 1e6 / (ticks - 20) as f64;
        let marks = bridge.player_probe_marks().len();
        println!("{label:<28} {per:>8.2} us/tique   {marks} marcas");
        per
    };

    println!("\n== o preco da leitura ==");
    let off = row("capacidades DESARMADAS", false);
    let on = row("capacidades ARMADAS", true);
    println!(
        "\ndiferenca: {:+.2} us/tique ({:.1}% de um quadro de 60 fps)",
        on - off,
        100.0 * (on - off) / 16_666.0
    );
}

/// **O QUE CUSTA UMA AMOSTRA** — o número que decide o teto dos contadores.
///
/// ⚠️ §0: um teto sem medição é palpite. O doc do `CORNER_SAMPLES` já dizia que
/// o sensor inteiro custa `+0,0002 ms por tique de subida`; aqui a pergunta é a
/// que falta para tornar a CONTAGEM autorável — *quanto custa cada amostra a
/// mais, e onde ela deixa de comprar precisão?*
///
/// Rodar:
/// ```text
/// cargo test -p ph2d-physics-ecs --release --test measure_player_probes -- --ignored --nocapture measure_what_a_sample_costs
/// ```
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_a_sample_costs() {
    use ph2d_physics::PhysicsWorld;

    let mut w = PhysicsWorld::new();
    // Um chao, uma parede e um teto -- para os raios de facto descerem no BVH.
    for (x, y, hx, hy) in [
        (0.0_f32, -0.5_f32, 40.0_f32, 0.5_f32),
        (12.0, 2.0, 0.5, 3.0),
        (6.0, 2.2, 2.0, 0.5),
    ] {
        w.add_static_cuboid(x, y, hx, hy);
    }
    let (body, _) = w.add_dynamic_circle(6.0, 0.9, 0.2, 1.0);
    w.step();

    println!("\n== o preco de uma amostra ==");
    let mut prev: Option<(usize, f64)> = None;
    for n in [1usize, 3, 9, 17, 33, 65, 129, 257] {
        let reps = 200;
        let t = Instant::now();
        let mut seen = 0usize;
        for _ in 0..reps {
            for i in 0..n {
                let f = if n == 1 {
                    0.0
                } else {
                    -1.0 + 2.0 * (i as f32) / ((n - 1) as f32)
                };
                if w.cast_ray([6.0 + f * 0.32, 1.4], [0.0, 1.0], 0.2, Some(body), 0)
                    .is_some()
                {
                    seen += 1;
                }
            }
        }
        let per_cast =
            t.elapsed().as_secs_f64() * 1e9 / f64::from(u32::try_from(reps).unwrap()) / n as f64;
        let per_tick_us = t.elapsed().as_secs_f64() * 1e6 / f64::from(u32::try_from(reps).unwrap());
        // O PASSO do perfil: a resolucao que estas N amostras compram sobre o vao
        // de meia-largura + alcance (0,20 + 0,12 = 0,32 m para cada lado).
        let step_mm = if n > 1 {
            2.0 * 0.32 * 1000.0 / (n - 1) as f64
        } else {
            f64::INFINITY
        };
        println!(
            "  N={n:>4}  {per_cast:6.1} ns/raio  {per_tick_us:7.3} us/perfil  passo {step_mm:7.2} mm  ({seen} acertos)"
        );
        prev = Some((n, per_tick_us));
    }
    let _ = prev;
    println!(
        "\nReferencia: o solver assenta com `normalized_allowed_linear_error` ~= 1,3 mm.\n\
         Abaixo disso o passo do perfil deixa de comprar precisao que a fisica resolva."
    );
}

/// **A PERNA É UM RAIO SÓ — o que isso custa** (pergunta do Enio, 2026-08-11:
/// *"Por que a perna não poderia ter mais de um?"*).
///
/// ⚠️ **É a MESMA pergunta que o `measure_wall_flank` fez ao flanco na W13**, e
/// lá a resposta foi um defeito medido: com um raio só, uma fresta de 0,75 m num
/// corpo de 1,0 m **recusava o pulo de parede por inteiro**. O flanco ganhou três
/// amostras por causa disso; a perna nunca foi medida.
///
/// A cena: um chão com uma FENDA de largura `g`, e o personagem a atravessá-la a
/// caminhar. O corpo mede 0,4 m de largura, então toda fenda abaixo disso é uma
/// que ele deveria **atravessar sem sentir**.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_a_single_ground_ray_costs_over_a_gap() {
    println!("\n== a perna sobre uma FENDA (corpo de 0,40 m de largura) ==");
    println!(
        "{:>7}  {:>8}  {:>5}  {:>9}  {:>10}  {:>9}",
        "fenda", "regime", "raios", "queda max", "% do float", "achou chao"
    );
    // ⚠️ DOIS regimes, e o 1o corte desta sonda so' tinha o rapido -- a
    // `drive = 1.0` atravessa a fenda em poucos tiques e a gravidade mal age, o
    // que deu 0,000 m em TODA fenda e teria fechado a pergunta com um numero que
    // media a velocidade, nao o sensor.
    for (label, drive, park) in [("rapido", 1.0_f32, false), ("PARADO", 0.0, true)] {
        // ⚠️ A coluna `raios` e' a ABLACAO por ENTRADA: 1 e' a perna de antes da
        // `W-Probes2`, 3 e' o default de hoje. Sem o 1 ao lado o numero nao diz
        // se a cura curou -- ele so' diz onde estamos.
        for samples in [1u16, 3, 5] {
        for gap_mm in [0u32, 100, 200, 300, 400, 600] {
            let gap = f64::from(gap_mm) / 1000.0;
            let mut sim = SimWorld::new();
            let mut slab = |name: &str, at: Vec2, half: [f32; 2]| {
                sim.world_mut().spawn((
                    Name::new(name),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: half[0],
                            half_y: half[1],
                        },
                        ..Collider::default()
                    },
                    Transform::from_translation(at),
                ));
            };
            let g = gap as f32;
            slab("Left", Vec2::new(-10.0, -0.5), [10.0, 0.5]);
            slab("Right", Vec2::new(g + 10.0, -0.5), [10.0, 0.5]);

            // Parado: nasce com o CENTRO exatamente sobre o meio da fenda.
            let x0 = if park { g * 0.5 } else { -2.0 };
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
                        float_height: FLOAT,
                        foot_samples: samples,
                        ..PlatformPlayer::default()
                    },
                    Transform::from_translation(Vec2::new(x0, FLOAT)),
                ))
                .id();

            let mut bridge = PhysicsBridge::new();
            let mut lowest = f32::INFINITY;
            let mut ever_lost = false;
            for i in 1..=240u64 {
                bridge.set_player_input(
                    player,
                    PlayerInput {
                        drive,
                        ..PlayerInput::default()
                    },
                );
                bridge.dispatch(&mut sim, true, i);
                let t = sim.world().get::<Transform>(player).expect("transform");
                if t.translation.x > -0.05 && t.translation.x < g + 0.05 {
                    lowest = lowest.min(t.translation.y);
                    ever_lost |= bridge
                        .player_probe_marks()
                        .iter()
                        .any(|m| m.kind == ProbeKind::Ground && m.state == ProbeState::Clear);
                }
            }
            let dip = if lowest.is_finite() { FLOAT - lowest } else { 0.0 };
            println!(
                "{gap:>6.2}m  {label:>8}  {samples:>5}  {dip:>8.3}m  {:>9.1}%  {:>9}",
                100.0 * dip / FLOAT,
                if ever_lost { "PERDEU" } else { "sempre" }
            );
        }
        }
    }
    println!(
        "\n⚠️ O corpo mede 0,40 m: toda fenda ABAIXO disso e' uma que ele deveria\n\
         atravessar sem sentir, porque as bordas ainda o suportam fisicamente."
    );
}
