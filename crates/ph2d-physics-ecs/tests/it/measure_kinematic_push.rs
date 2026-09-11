//! **SONDA — o que um personagem CINEMÁTICO faz ao que está no caminho dele.**
//!
//! A W-KinMove deu ao modo Snap a metade VERTICAL da 3ª lei (o chão sente o
//! peso, pela `reaction`, e a W-KinWeight mediu que ela já atravessa o modo).
//! Falta a metade HORIZONTAL: um corpo cinemático tem massa infinita para o
//! solver, então o `move_shape` desliza contra um caixote solto **sem lhe
//! transmitir nada** — o personagem passa a ser, de lado, o fantasma que a
//! `react.rs` descreve.
//!
//! ⚠️ **O CONTROLE é o modo dinâmico**, e ele não é opcional: sem uma coluna que
//! empurra, um zero na coluna cinemática não distingue *"o modo não empurra"* de
//! *"a cena não tem o que empurrar"*.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_kinematic_push -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerMode,
    RigidBody,
};
use ph2d_platformer::PlayerInput;

const FLOAT_HEIGHT: f32 = 0.9;

/// Um chão plano, um CAIXOTE solto à direita e o personagem à esquerda dele.
fn scene(crate_density: f32, kinematic: bool) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    sim.world_mut().spawn((
        Name::new("Crate"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            density: crate_density,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(1.5, 0.3)),
    ));
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: if kinematic {
                    BodyKind::Kinematic
                } else {
                    BodyKind::Dynamic
                },
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
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT_HEIGHT)),
        ))
        .id();
    if kinematic {
        sim.world_mut()
            .entity_mut(player)
            .insert(PlayerMode::Kinematic);
    }
    (sim, PhysicsBridge::new(), player)
}

fn x_of(sim: &SimWorld, name: &str) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == name {
            found = Some(t.translation.x);
        }
    }
    found.expect("a cena tem de conter o corpo")
}

/// Anda para a direita por `secs` e devolve `(quanto o caixote andou, quanto o
/// personagem andou)`.
fn walk_into_crate(crate_density: f32, kinematic: bool, secs: u64) -> (f32, f32) {
    let (mut sim, mut bridge, who) = scene(crate_density, kinematic);
    // Assenta antes de medir: o primeiro tique de uma cena tem o BVH vazio.
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let c0 = x_of(&sim, "Crate");
    let p0 = x_of(&sim, "Player");
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 61..=(60 + secs * 60) {
        bridge.dispatch(&mut sim, true, t);
    }
    (x_of(&sim, "Crate") - c0, x_of(&sim, "Player") - p0)
}

/// **O CAIXOTE NO CAMINHO** — a coluna dinâmica é o controle.
#[test]
#[ignore = "sonda de medição"]
fn measure_what_a_walking_player_does_to_a_loose_crate() {
    println!("\n=== ANDAR CONTRA UM CAIXOTE (3 s, densidade 1) ===");
    println!(
        "{:<24} {:>14} {:>16}",
        "modo", "caixote (m)", "personagem (m)"
    );
    for (label, kin) in [("DINAMICO (controle)", false), ("CINEMATICO", true)] {
        let (c, p) = walk_into_crate(1.0, kin, 3);
        println!("{label:<24} {c:>14.4} {p:>16.4}");
    }

    println!("\n=== A MASSA MANDA? (3 s, por densidade do caixote) ===");
    println!(
        "{:<16} {:>14} {:>14}",
        "densidade", "dinamico (m)", "cinematico (m)"
    );
    for d in [0.25f32, 1.0, 4.0, 16.0] {
        let (cd, _) = walk_into_crate(d, false, 3);
        let (ck, _) = walk_into_crate(d, true, 3);
        println!("{:<16} {cd:>14.4} {ck:>14.4}", format!("{d:.2}"));
    }
}

/// **O CHÃO É EMPURRADO DUAS VEZES?** — a pergunta que decide se este canal
/// precisa de um filtro.
///
/// ⚠️ A 3ª lei vertical (K6) já entrega o PESO ao corpo em que o personagem
/// está de pé. Se o empurrão lateral também entregasse a componente vertical do
/// bloqueio, a jangada afundaria **por dois canais** — a doença que este módulo
/// mais encontra: duas respostas para um fato. A jangada é o oráculo, e a
/// comparação é `push 0` contra `push 1`: elas têm de dar o MESMO número.
///
/// ⚠️ **A fixture é a do `measure_kinematic_case`, e as duas linhas que a fazem
/// funcionar não são opcionais:** a jangada leva `GravityScale(0)` (senão o que
/// se mede é a queda livre dela, e a 1ª versão desta sonda mediu exactamente
/// isso — `−22 m` em 2 s) e há **60 tiques de aquecimento** antes de medir, com
/// a aceleração vindo de uma SEGUNDA DIFERENÇA (depois do aquecimento a jangada
/// já tem velocidade, e `½at²` pressupõe partir do repouso).
#[test]
#[ignore = "sonda de medição"]
fn measure_whether_the_floor_is_pushed_twice() {
    fn raft_accel(kinematic: bool, push: f32, walk: f32) -> f32 {
        let mut sim = SimWorld::new();
        let raft = sim
            .world_mut()
            .spawn((
                Name::new("Raft"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    // ⚠️ **LARGA de propósito:** o personagem anda a ~5,9 m/s e
                    // uma jangada de 6 m acaba em meio segundo — o resto da
                    // janela mediria uma jangada à deriva, e a segunda diferença
                    // daria `0,0000`, que se lê como *"o empurrão não afunda"*
                    // quando o que houve foi ninguém em cima.
                    shape: ColliderShape::Cuboid {
                        half_x: 30.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::GravityScale(0.0),
                // ⚠️ **Sem travar a rotação o `y` do CENTRO deixa de ser
                // oráculo:** um empurrão lateral acima do centro INCLINA a
                // jangada, e o centro de um corpo que tomba desce por um motivo
                // que não é peso. A pergunta desta sonda é vertical, então a
                // fixture tem de tirar a inclinação do caminho.
                LockRotation,
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        let who = sim
            .world_mut()
            .spawn((
                Name::new("Player"),
                RigidBody {
                    kind: if kinematic {
                        BodyKind::Kinematic
                    } else {
                        BodyKind::Dynamic
                    },
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
                    reaction_push: push,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
            ))
            .id();
        if kinematic {
            sim.world_mut()
                .entity_mut(who)
                .insert(PlayerMode::Kinematic);
        }
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: walk,
                ..PlayerInput::default()
            },
        );
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let y0 = sim.world().get::<Transform>(raft).unwrap().translation.y;
        let mut y1 = 0.0;
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
            if t == 120 {
                y1 = sim.world().get::<Transform>(raft).unwrap().translation.y;
            }
        }
        let y2 = sim.world().get::<Transform>(raft).unwrap().translation.y;
        y2 - 2.0 * y1 + y0
    }

    // `a = (A_player / A_raft)·g` — a densidade cancela; o sinal é para BAIXO.
    let area_p = 0.6 * 0.4 + core::f32::consts::PI * 0.2 * 0.2;
    let want = -(area_p / (60.0 * 0.5)) * 9.81;
    println!("\n=== A JANGADA AFUNDA DUAS VEZES? (acel, 2 s) ===");
    println!("{:<40} {:>14} {:>12}", "cena", "acel (m/s2)", "% de m.g");
    for (label, kin, push, walk) in [
        ("DINAMICO (controle)", false, 1.0f32, 0.0f32),
        ("CINEMATICO parado, push 0", true, 0.0, 0.0),
        ("CINEMATICO parado, push 1", true, 1.0, 0.0),
        ("CINEMATICO andando, push 0", true, 0.0, 1.0),
        ("CINEMATICO andando, push 1", true, 1.0, 1.0),
    ] {
        let a = raft_accel(kin, push, walk);
        println!("{label:<40} {a:>14.4} {:>12.1}", a / want * 100.0);
    }
}

/// **A AMPLITUDE EM REGIME** — o kill-criterion desta wave.
///
/// ⚠️ Um personagem que empurra por IMPULSO a 60 Hz pode ressoar com o slide
/// (empurra, o caixote foge, o slide o segue, empurra outra vez). O número que
/// decide não é *"o caixote andou"*, é **quanto a distância entre os dois
/// oscila** depois de o contato assentar.
///
/// ⚠️ **As DUAS cenas, e a segunda é a dura:** com o caixote solto o empurrão
/// tem para onde ir; com ele **encostado numa parede** a massa efetiva do que
/// se empurra é infinita e o personagem continua a bater nele todo tique — que
/// é onde uma realimentação apareceria.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_steady_state_amplitude_against_a_resting_crate() {
    for blocked in [false, true] {
        println!(
            "\n=== FOLGA EM REGIME ({}) ===",
            if blocked {
                "caixote ENCOSTADO numa parede"
            } else {
                "caixote solto"
            }
        );
        for (label, kin) in [("DINAMICO (controle)", false), ("CINEMATICO", true)] {
            let (mut sim, mut bridge, who) = scene(1.0, kin);
            if blocked {
                sim.world_mut().spawn((
                    Name::new("Wall"),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.25,
                            half_y: 2.0,
                        },
                        ..Collider::default()
                    },
                    Transform::from_translation(Vec2::new(2.05, 1.0)),
                ));
            }
            for t in 1..=60u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            bridge.set_player_input(
                who,
                PlayerInput {
                    drive: 1.0,
                    ..PlayerInput::default()
                },
            );
            // Um segundo para encostar e assentar; depois mede a FOLGA tique a
            // tique.
            for t in 61..=180u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            let mut lo = f32::INFINITY;
            let mut hi = f32::NEG_INFINITY;
            for t in 181..=300u64 {
                bridge.dispatch(&mut sim, true, t);
                let gap = x_of(&sim, "Crate") - x_of(&sim, "Player");
                lo = lo.min(gap);
                hi = hi.max(gap);
            }
            println!(
                "{label:<24} folga {lo:.4}..{hi:.4}  amplitude {:.4} m",
                hi - lo
            );
        }
    }
}
