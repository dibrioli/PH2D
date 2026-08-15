//! **LARGAR UMA PLATAFORMA EM MOVIMENTO** — a sonda do §3.J da auditoria 09, e
//! o 1.º passo da wave `W-Leave` (plano 10 §6).
//!
//! O Godot tem três políticas explícitas ao largar uma plataforma móvel
//! (`platform_on_leave`): `ADD_VELOCITY`, `ADD_UPWARD_VELOCITY` — que existe
//! porque uma plataforma a **descer** te atiraria para baixo — e `DO_NOTHING`.
//! Nós não temos nenhuma escrita; temos o `lift_momentum` (W10), que resolve o
//! problema **vizinho**: o referencial em que o controle aéreo mede.
//!
//! ⚠️ **A pergunta não é *"está partido?"*, é *"o que é que os três modos
//! fazem?"***, e há duas razões concretas para desconfiar de que não concordam:
//!
//! 1. **O gate da W10 (`platform_lift.rs`) só conhece o modo dinâmico** — o
//!    vagão dele é um corpo com massa enorme e o personagem não tem
//!    `PlayerMode`. A tabela que escolheu `lift_momentum = 1,5` (no doc do
//!    `JumpConfig`) foi medida nesse único modo.
//! 2. **O eixo VERTICAL nunca foi medido de todo**, e ele é o que o desenho
//!    separa: a lei da caminhada age ao longo da TANGENTE, então o que uma
//!    plataforma horizontal entrega vive na velocidade que o personagem
//!    *possui*; o que um elevador entrega passa pelo `ground_carry`, que é
//!    deslocamento **deste** tique e não velocidade guardada. Se a divergência
//!    existir, é aqui.
//!
//! # O oráculo, e porque não é a velocidade
//!
//! Mede-se o **deslocamento em MUNDO numa janela FIXA depois do pulo** — a
//! mesma escolha do gate da W10 (*"a velocidade oscila com o passo do solver, e
//! o que o jogador vê é o quanto ele avançou"*) — e o CONTROLE está DENTRO da
//! tabela: a mesma corrida com a plataforma **parada**. A diferença entre as
//! duas linhas **é** o que a plataforma entregou; sem ela, um número sozinho não
//! distingue *"levou a velocidade"* de *"pulou"*.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --test measure_platform_leave
//! --release -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod platform;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformLift, PlatformPlayer,
    PlayerInput, PlayerMode, RigidBody,
};

use platform::FLOAT_HEIGHT;

/// A velocidade da plataforma — **o número que o plano nomeia** (§6: *"largar
/// uma plataforma a 4 m/s nos dois modos"*), e o mesmo do vagão da W10.
const PLATFORM_SPEED: f32 = 4.0;

/// ⚠️ **A plataforma é LONGA de propósito** — a lição da sonda irmã
/// (`measure_kinematic_carry`), onde uma meia-largura de 3,0 deixava o
/// personagem **sair de cima dela** e a coluna passava a medir um deslize sem
/// chão. Aqui o risco é o mesmo em voo: 30 m cobrem qualquer alcance de pulo
/// que estes números produzem.
const WAGON_HALF: f32 = 30.0;

/// Quanto tempo ele anda na plataforma antes de largar. Um segundo é folgado: a
/// tração da caminhada assenta numa fração disto, e o que sobra é margem.
const RIDE_TICKS: u64 = 60;

/// **A janela do voo, FIXA** — meio segundo. Fixa porque as linhas têm de ser
/// comparáveis: medir *"até aterrar"* daria a cada linha uma duração própria, e
/// então um deslocamento maior poderia significar apenas mais tempo no ar.
const FLIGHT_TICKS: u64 = 30;

/// Os três modos, com o nome que os smokes usam.
///
/// ⚠️ **O modo é um PAR, não um componente:** a porta é o `pose_owner`, e ela
/// pergunta ao KIND do corpo que a ponte de facto construiu antes de olhar para
/// o `PlayerMode` — inserir só o componente sobre um corpo `Dynamic` mediria o
/// MESMO caminho três vezes.
const MODES: [(Option<PlayerMode>, &str); 3] = [
    (None, "Spring"),
    (Some(PlayerMode::Kinematic), "Snap"),
    (Some(PlayerMode::Pure), "Pure"),
];

/// As quatro plataformas — e a primeira é o **CONTROLE**.
///
/// ⚠️ **O elevador a DESCER está aqui porque é o caso que dá nome à política do
/// Godot**: herdar uma velocidade para baixo ao largar é o que faz um pulo
/// parecer que não saiu do lugar.
const RIGS: [(&str, [f32; 2]); 4] = [
    ("parada (CONTROLE)", [0.0, 0.0]),
    ("vagao ->", [1.0, 0.0]),
    ("elevador SOBE", [0.0, 1.0]),
    ("elevador DESCE", [0.0, -1.0]),
];

/// Uma plataforma cinemática dirigida pela CENA e um personagem em cima dela.
///
/// ⚠️ **Não há chão nenhum além dela**, e é deliberado: um chão por baixo daria
/// ao elevador que desce um fundo onde aterrar, e a janela mediria o pouso em
/// vez do que ele levou consigo.
fn scene(
    mode: Option<PlayerMode>,
    lift: PlatformLift,
) -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    let wagon = sim
        .world_mut()
        .spawn((
            Name::new("Wagon"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: WAGON_HALF,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let who = sim
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
                float_height: FLOAT_HEIGHT,
                platform_lift: lift,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
        ))
        .id();
    if let Some(m) = mode {
        let mut e = sim.world_mut().entity_mut(who);
        e.insert(m);
        if let Some(mut rb) = e.get_mut::<RigidBody>() {
            rb.kind = BodyKind::Kinematic;
        }
    }
    (sim, PhysicsBridge::new(), wagon, who)
}

fn nudge(sim: &mut SimWorld, wagon: Entity, d: [f32; 2]) {
    let mut e = sim.world_mut().entity_mut(wagon);
    if let Some(mut tr) = e.get_mut::<Transform>() {
        tr.translation.x += d[0];
        tr.translation.y += d[1];
    }
}

fn pose_of(sim: &SimWorld, who: Entity) -> (f32, f32) {
    sim.world()
        .get::<Transform>(who)
        .map(|t| (t.translation.x, t.translation.y))
        .expect("o personagem tem de existir")
}

/// Uma corrida completa: assenta, anda com a plataforma, **PULA**, e voa.
///
/// Devolve `(dx, dy, pico)` na janela do voo, medidos a partir do instante do
/// pulo.
///
/// ⚠️ **O botão fica PREMIDO durante o voo inteiro**, o que dá o pulo cheio e
/// determinístico. É seguro porque `air_jumps` nasce em **0** (`JumpConfig`):
/// com a capacidade ligada, segurar o botão dispararia um segundo pulo e a
/// coluna passaria a medir isso.
fn leave(mode: Option<PlayerMode>, axis: [f32; 2], lift: PlatformLift) -> (f32, f32, f32) {
    let (mut sim, mut bridge, wagon, who) = scene(mode, lift);
    // Assenta com a plataforma parada: um corpo a cair mede a queda.
    for t in 1..=60u64 {
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }
    let step = [
        axis[0] * PLATFORM_SPEED / 60.0,
        axis[1] * PLATFORM_SPEED / 60.0,
    ];
    for t in 61..=(60 + RIDE_TICKS) {
        nudge(&mut sim, wagon, step);
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }

    let p0 = pose_of(&sim, who);
    let mut peak = 0.0f32;
    for t in (61 + RIDE_TICKS)..=(60 + RIDE_TICKS + FLIGHT_TICKS) {
        // A plataforma **continua a andar**, que é o que ela faz num jogo.
        nudge(&mut sim, wagon, step);
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        peak = peak.max(pose_of(&sim, who).1 - p0.1);
    }
    let p1 = pose_of(&sim, who);
    (p1.0 - p0.0, p1.1 - p0.1, peak)
}

/// **O CONTROLE POSITIVO: ele SAI mesmo do chão?**
///
/// ⚠️ Sem isto, uma tabela de deslocamentos pequenos leria como *"a plataforma
/// não entrega nada"* quando a causa seria o pulo nunca ter acontecido — e o
/// Pure é o modo em que essa suspeita é concreta, porque o mundo físico lhe é
/// cenário. A coluna é o pico com a plataforma PARADA: um zero aqui torna todas
/// as outras leituras daquele modo leituras sobre nada.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_that_the_jump_happens_at_all() {
    println!("\n== o pulo acontece? (plataforma PARADA) ==");
    println!(
        "  modo       pico em {:.2} s (m)",
        FLIGHT_TICKS as f32 / 60.0
    );
    for (mode, tag) in MODES {
        let (_, _, peak) = leave(mode, [0.0, 0.0], PlatformLift::Full);
        println!("  {tag:<9}  {peak:>8.4}");
    }
}

/// **O QUE A POLÍTICA COMPRA** — as três variantes contra as três plataformas.
///
/// ⚠️ **A coluna que decide é o PICO**, e o oráculo dela é a linha do CONTROLE
/// (plataforma parada): a política existe para que a altura autorada seja
/// entregue em MUNDO, então *"funciona"* significa **o pico do elevador que
/// desce igualar o pico com a plataforma parada** — não um número maior.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_the_policy_buys() {
    println!(
        "\n=== A POLITICA (pico em {:.2} s, m) ===",
        FLIGHT_TICKS as f32 / 60.0
    );
    for (mode, tag) in MODES {
        println!("\n  modo {tag}");
        println!("    plataforma           Full     UpOnly    Nothing");
        for (name, axis) in RIGS {
            let cols: Vec<f32> = [
                PlatformLift::Full,
                PlatformLift::UpOnly,
                PlatformLift::Nothing,
            ]
            .into_iter()
            .map(|l| leave(mode, axis, l).2)
            .collect();
            println!(
                "    {name:<18}  {:>7.4}   {:>7.4}   {:>7.4}",
                cols[0], cols[1], cols[2]
            );
        }
    }
    println!(
        "\nLEITURA: a linha 'parada' e' o CONTROLE e tem de ser IDENTICA nas tres\n\
         colunas (chao estatico ⇒ as tres politicas sao a mesma, ao bit).\n\
         'elevador DESCE' sob UpOnly/Nothing tem de igualar esse controle.\n\
         'elevador SOBE' separa UpOnly de Nothing: uma mantem o impulso, a\n\
         outra o descarta.\n"
    );
}

/// **A sonda da wave: o que cada modo leva ao largar a plataforma.**
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_what_leaving_a_platform_delivers() {
    let window = FLIGHT_TICKS as f32 / 60.0;
    let travelled = PLATFORM_SPEED * window;
    println!("\n=== LARGAR UMA PLATAFORMA A {PLATFORM_SPEED:.1} m/s ===");
    println!(
        "janela de voo {window:.2} s; a plataforma anda {travelled:.3} m nela\n\
         a coluna 'entregue' e' a linha MENOS o CONTROLE do mesmo modo,\n\
         e a fracao e' o quanto disso a plataforma de facto passou\n"
    );
    println!("  plataforma          modo       dx(m)     dy(m)    pico(m)   entregue(m)  fracao");
    for (mode, tag) in MODES {
        let (bx, by, _) = leave(mode, [0.0, 0.0], PlatformLift::Full);
        for (name, axis) in RIGS {
            let (dx, dy, peak) = leave(mode, axis, PlatformLift::Full);
            // O eixo em que a plataforma se move é o eixo em que se pergunta.
            let delivered = (dx - bx) * axis[0] + (dy - by) * axis[1];
            let frac = if axis[0].abs() + axis[1].abs() > 0.0 {
                format!("{:>6.0}%", 100.0 * delivered / travelled)
            } else {
                "    --".to_string()
            };
            println!(
                "  {name:<18}  {tag:<9}  {dx:>8.4}  {dy:>8.4}  {peak:>8.4}   {delivered:>9.4}  {frac}"
            );
        }
        println!();
    }
    println!(
        "LEITURA: fracao ~100% e' levar a plataforma consigo; ~0% e' larga-la\n\
         a' porta. O elevador que DESCE tem fracao NEGATIVA se ele herdar a\n\
         descida -- e' o caso que da' nome ao `ADD_UPWARD_VELOCITY` do Godot.\n"
    );
}
