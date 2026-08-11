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
