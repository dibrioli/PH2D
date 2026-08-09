//! **SONDA — o que o modo CINEMÁTICO existe para curar, medido HOJE.**
//!
//! ⚠️ **Esta sonda existe porque o plano 07 envelheceu antes de ser executado.**
//! Ele nomeia dois defeitos do corpo dinâmico (a deriva de rampa e a penetração
//! no impacto) e cita `0,164 m` para o primeiro. A `W-Landing` levou o
//! amortecimento ao TETO no mesmo dia, e o teto sempre zerou a deriva — então o
//! número do plano descreve um produto que não existe mais.
//!
//! **A regra é a do `CLAUDE.md` §0 aplicada a mim mesmo:** *quem move o número
//! que justificava uma wave tem de reconferir a nota antes de a construir.*
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_kinematic_case -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod platform;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::RideConfig;

use platform::{FLOAT_HEIGHT, pose};

/// Quanto o personagem PARADO viaja numa rampa de `deg` graus em `secs`, com o
/// amortecimento em `damping`.
fn idle_travel(deg: f32, secs: u64, damping: f32) -> f32 {
    let slope = deg.to_radians();
    let (mut sim, mut bridge, who) = platform::scene(slope, 0.0);
    {
        let mut e = sim.world_mut().entity_mut(who);
        let mut p = e.get_mut::<PlatformPlayer>().expect("player");
        p.spring_damping = damping;
    }
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let (x0, _) = pose(&sim);
    for t in 121..=(120 + secs * 60) {
        bridge.dispatch(&mut sim, true, t);
    }
    (pose(&sim).0 - x0).abs()
}

/// **A DERIVA DE RAMPA, hoje** — e a tabela diz se o plano ainda descreve o
/// produto.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_ramp_creep_today() {
    let ceiling = RideConfig::MAX_DAMPING;
    println!("\n=== DERIVA DE RAMPA (parado 10 s, corpo DINAMICO) ===");
    println!("{:<34} {:>10}", "amortecimento", "viajou (m)");
    for (label, d) in [
        (
            "o DEFAULT que shipa",
            RideConfig::STARTING_POINT.spring_damping,
        ),
        ("o teto", ceiling),
        ("meio curso", 0.5 * ceiling),
        ("um quarto", 0.25 * ceiling),
    ] {
        println!("{label:<34} {:>10.4}", idle_travel(30.0, 10, d));
    }
    println!(
        "\n  (o plano 07 cita 0,164 m para o default; o default de hoje e' {:.2} = o teto)",
        RideConfig::STARTING_POINT.spring_damping
    );

    // ⚠️ **`0,0000` em 10 s pode ser uma deriva LENTA que a janela não conteve**,
    // e é isso que decide se o item do Enio (*"o player sobe sozinho bem
    // devagar"*) está morto ou só escondido. Duas varreduras: o TEMPO, para uma
    // deriva lenta aparecer, e a INCLINAÇÃO, porque a força que escorrega é
    // função dela — um zero só a 30° não diz nada sobre 45°.
    let d = RideConfig::STARTING_POINT.spring_damping;
    println!("\n=== O DEFAULT AO LONGO DO TEMPO (rampa de 30 graus) ===");
    println!("{:<34} {:>10}", "parado por", "viajou (m)");
    for secs in [10u64, 30, 60, 120] {
        println!(
            "{:<34} {:>10.4}",
            format!("{secs} s"),
            idle_travel(30.0, secs, d)
        );
    }
    println!("\n=== O DEFAULT AO LONGO DA INCLINACAO (parado 60 s) ===");
    println!("{:<34} {:>10}", "rampa", "viajou (m)");
    for deg in [10.0f32, 20.0, 30.0, 40.0, 44.0] {
        println!(
            "{:<34} {:>10.4}",
            format!("{deg:.0} graus"),
            idle_travel(deg, 60, d)
        );
    }
}

/// **A PENETRAÇÃO no impacto, hoje** — o segundo defeito que o plano nomeia.
///
/// ⚠️ O oráculo é a distância do PÉ ao topo do chão: um personagem que paira a
/// `float_height` tem de manter essa folga, e o que a queda lhe tira é o número
/// que o modo cinemático promete zerar.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_impact_penetration_today() {
    println!("\n=== PENETRACAO NO IMPACTO, NOS DOIS MODOS (§7.3) ===");
    println!(
        "{:>10} {:>9} {:>12} {:>14} {:>14}",
        "queda (m)", "damping", "modo", "pior folga", "perdido (mm)"
    );
    // ⚠️ **A coluna do amortecimento é o CONTROLE, e sem ela a tabela mente por
    // vácuo:** no teto o boost mata a velocidade relativa inteira em UM tique, e
    // o personagem é apanhado no instante em que o raio o vê. Uma tabela só do
    // default diria *"não há penetração"* quando o que ela mede é *"o knob está
    // no teto"* — e é essa distinção que decide se o modo cinemático compra algo.
    let ceiling = RideConfig::MAX_DAMPING;
    // ⚠️ **A régua é o repouso DAQUELE modo, e a primeira versão errou nisso.**
    // Ela media `FLOAT_HEIGHT − pior`, e o `FLOAT_HEIGHT` é onde a cápsula
    // FLUTUANTE descansa — o cinemático não flutua, ele assenta a ~0,51, então
    // a tabela lhe cobrava **387 mm de penetração** por estar exactamente onde
    // devia estar. Uma régua tomada do outro modo mede a diferença entre os
    // modos e chama-lhe defeito.
    let rest = |kin: bool| lowest_gap(0.0, ceiling, kin);
    let (rest_dyn, rest_kin) = (rest(false), rest(true));
    println!("  (repouso: dinamico {rest_dyn:.4} · cinematico {rest_kin:.4})");
    for (drop, damping, kinematic) in [0.5_f32, 2.0, 5.0, 10.0]
        .into_iter()
        .flat_map(|d| [(d, ceiling), (d, 0.25 * ceiling)])
        .flat_map(|(d, k)| [(d, k, false), (d, k, true)])
    {
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
                spring_damping: damping,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT_HEIGHT + drop)),
        ));
        if kinematic {
            // Os DOIS campos, por UMA porta — escrever só o componente deixaria
            // o corpo dinâmico e o `pose_owner` responderia `Solver`.
            let who = {
                let mut q = sim
                    .world()
                    .try_query::<(ph2d_ecs::Entity, &Name)>()
                    .unwrap();
                let mut found = None;
                for (e, n) in q.iter(sim.world()) {
                    if n.as_str() == "Player" {
                        found = Some(e);
                    }
                }
                found.expect("player")
            };
            let mut e = sim.world_mut().entity_mut(who);
            e.insert(ph2d_physics_ecs::PlayerMode::Kinematic);
            if let Some(mut rb) = e.get_mut::<RigidBody>() {
                rb.kind = BodyKind::Kinematic;
            }
        }
        let tag = if kinematic { "CINEMATICO" } else { "dinamico" };
        let mut bridge = PhysicsBridge::new();
        let mut worst = f32::INFINITY;
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
            worst = worst.min(pose(&sim).1);
        }
        let base = if kinematic { rest_kin } else { rest_dyn };
        println!(
            "{drop:>10.1} {damping:>9.2} {tag:>12} {worst:>14.4} {:>14.1}",
            (base - worst) * 1000.0
        );
    }
}

/// A altura MAIS BAIXA a que o personagem chega, largado de `drop` acima da
/// altura de flutuação. Com `drop = 0` ela é o REPOUSO daquele modo — a régua
/// que a penetração usa.
fn lowest_gap(drop: f32, damping: f32, kinematic: bool) -> f32 {
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
                spring_damping: damping,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT_HEIGHT + drop)),
        ))
        .id();
    if kinematic {
        sim.world_mut()
            .entity_mut(who)
            .insert(ph2d_physics_ecs::PlayerMode::Kinematic);
    }
    let mut bridge = PhysicsBridge::new();
    let mut worst = f32::INFINITY;
    for t in 1..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        worst = worst.min(pose(&sim).1);
    }
    worst
}

fn raft_y(sim: &SimWorld, raft: ph2d_ecs::Entity) -> f32 {
    sim.world()
        .get::<Transform>(raft)
        .expect("raft")
        .translation
        .y
}

/// **O QUE A JANGADA SENTE NOS DOIS MODOS** — a medição que decide a
/// `W-KinWeight` (plano 07 §6).
///
/// A K6 diz que a 3ª lei sobrevive ao modo: a `reaction` toma o suporte como
/// ARGUMENTO e o chão vem do `footing`, então nada nela depende de o corpo ser
/// dinâmico. A ponte já a monta **fora** do ramo de modo. A pergunta que falta
/// é a que o plano nomeia: **a massa é AUTORADA**, e um corpo cinemático não
/// tem massa que o rapier calcule.
///
/// ⚠️ **A jangada leva `GravityScale(0)`** — sem isso ela cai por conta própria
/// e separar *"afundou porque o personagem pesa"* de *"afundou porque tudo
/// cai"* viraria a subtração de dois números grandes. Com peso próprio zero,
/// **todo milímetro é do personagem**, que é o oráculo desta wave.
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_raft_feels_in_both_modes() {
    println!("\n=== A JANGADA SOB O PERSONAGEM (120 tiques, parado no centro) ===");
    println!(
        "{:<22} {:>12} {:>16} {:>14}",
        "modo", "massa", "acel (m/s2)", "% de m.g"
    );
    // ⚠️ **A ablação é pela porta do ARTISTA** (`MassOverride`, da W-Mass), não
    // por um getter de debug: se o vão entre os modos for a MASSA, autorar a
    // mesma nos dois fecha-o; se sobreviver, o vão é o SUPORTE, e uma massa
    // autorada seria a cura errada.
    for (kinematic, forced) in [
        (false, None),
        (true, None),
        (false, Some(1.0_f32)),
        (true, Some(1.0_f32)),
        // ⚠️ **5 kg está FORA do regime físico desta fixture, e a linha fica
        // para dizer isso.** A jangada não tem peso próprio, então `5·9,81/3 =
        // 16,35 m/s²` — ela foge para baixo MAIS RÁPIDO que a gravidade, e um
        // personagem não consegue continuar em cima de um chão nessas
        // condições: o cinemático separa-se dela (70%), que é o correto, e é o
        // DINÂMICO que segue a empurrar (1367%) porque a mola dele está
        // ancorada na própria inércia. Quem quiser subir este número tem de
        // subir a massa da jangada junto.
        (false, Some(5.0_f32)),
        (true, Some(5.0_f32)),
    ] {
        let mut sim = SimWorld::new();
        let raft = sim
            .world_mut()
            .spawn((
                Name::new("Raft"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::GravityScale(0.0),
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
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
                Transform::from_translation(Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
            ))
            .id();
        if kinematic {
            sim.world_mut()
                .entity_mut(player)
                .insert(ph2d_physics_ecs::PlayerMode::Kinematic);
        }
        if let Some(m) = forced {
            sim.world_mut()
                .entity_mut(player)
                .insert(ph2d_physics_ecs::MassOverride(m));
        }
        let mut bridge = PhysicsBridge::new();
        // ⚠️ **AQUECER antes de medir, e é a lição do berço da cena 101 outra
        // vez:** os dois modos repousam a alturas DIFERENTES (sob Snap a perna é
        // o próprio corpo, não o `float_height`), então largar os dois na mesma
        // altura dá ao cinemático 0,4 m de QUEDA antes do primeiro contato — e a
        // rampa `[27, 57, 70, 77]` que isso produz lê-se como *"ele pesa menos"*.
        // Sessenta tiques de assentamento, e cada modo começa de onde ele
        // realmente fica.
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        // ⚠️ **O oráculo é a ACELERAÇÃO, não o deslocamento** — depois do
        // aquecimento a jangada já tem velocidade (nada a segura: o peso próprio
        // dela é zero de propósito), e `½at²` pressupõe partir do repouso. A
        // segunda diferença sobre três amostras igualmente espaçadas mata o
        // termo `v₀·t` por construção, e é isso que torna a comparação honesta
        // entre um modo que assenta rápido e outro que assenta devagar.
        let mut y = [0.0f32; 3];
        y[0] = raft_y(&sim, raft);
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
            if t == 120 {
                y[1] = raft_y(&sim, raft);
            }
        }
        y[2] = raft_y(&sim, raft);
        let secs = 1.0f32;
        let accel = (y[2] - 2.0 * y[1] + y[0]) / (secs * secs);
        // `a = (A_player / A_raft)·g` — a densidade CANCELA, então a expectativa
        // não depende dela; o sinal é para BAIXO.
        let area_p = 0.6 * 0.4 + core::f32::consts::PI * 0.2 * 0.2;
        let want = -(area_p / 3.0) * 9.81;
        println!(
            "{:<22} {:>12} {:>16.4} {:>14.1}",
            if kinematic { "CINEMATICO" } else { "dinamico" },
            match forced {
                Some(m) => format!("{m:.2} autorada"),
                None => "auto".to_string(),
            },
            accel,
            accel / want * 100.0
        );
    }
}
