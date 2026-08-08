//! **O MODO CINEMÁTICO** (W-KinMove) — os gates de comportamento.
//!
//! ⚠️ **A fixture tem de conter o fenômeno, e aqui isso é um NÚMERO:** os dois
//! defeitos que este modo promete zerar (a deriva de rampa e a penetração no
//! impacto) são **zero no default que shipa**, porque o `spring_damping` está no
//! TETO (medido em `measure_kinematic_case.rs`). Um gate que corresse no default
//! passaria nos DOIS modos — *um gate que passa no controle está a medir a coisa
//! errada* (a lição da W-AreaFalloff). Por isso toda comparação aqui corre com o
//! amortecimento **abaixo** do teto, onde o dinâmico de facto sofre.

#[path = "platform_scene.rs"]
mod platform;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerMode,
    RigidBody,
};
use ph2d_platformer::{PlayerInput, RideConfig};

use platform::{FLOAT_HEIGHT, pose};

/// Torna o player da cena cinemático — **os dois campos, por UMA porta**, que é
/// o que o gesto da §14 faz. Escrever só o componente deixaria o corpo dinâmico
/// e o `pose_owner` responderia `Solver` (a falha SEGURA, e não o que se quer
/// medir).
fn make_kinematic(sim: &mut SimWorld, who: ph2d_ecs::Entity) {
    let mut e = sim.world_mut().entity_mut(who);
    e.insert(PlayerMode::Kinematic);
    if let Some(mut rb) = e.get_mut::<RigidBody>() {
        rb.kind = BodyKind::Kinematic;
    }
}

fn set_damping(sim: &mut SimWorld, who: ph2d_ecs::Entity, d: f32) {
    let mut e = sim.world_mut().entity_mut(who);
    if let Some(mut p) = e.get_mut::<PlatformPlayer>() {
        p.spring_damping = d;
    }
}

/// **O DEFEITO 1: a deriva de rampa.** Parado numa rampa por 10 s.
fn ramp_creep(kinematic: bool, damping: f32) -> f32 {
    let (mut sim, mut bridge, who) = platform::scene(30.0_f32.to_radians(), 0.0);
    set_damping(&mut sim, who, damping);
    if kinematic {
        make_kinematic(&mut sim, who);
    }
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = pose(&sim).0;
    for t in 121..=720u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    (pose(&sim).0 - x0).abs()
}

/// **A deriva de rampa é ESTRUTURAL sob Snap, não afinada.**
///
/// ⚠️ O controle é o MESMO amortecimento no modo dinâmico: sem ele o gate não
/// distingue *"o modo curou"* de *"o knob estava no teto"*.
#[test]
fn the_kinematic_player_does_not_creep_up_a_ramp() {
    let d = 0.25 * RideConfig::MAX_DAMPING;
    let dynamic = ramp_creep(false, d);
    let kinematic = ramp_creep(true, d);
    assert!(
        dynamic > 0.02,
        "a fixture tem de CONTER o fenomeno: o dinamico derivou {dynamic:.4} m"
    );
    assert!(
        kinematic < 0.001,
        "sob Snap nao ha mola a integrar: derivou {kinematic:.4} m (dinamico {dynamic:.4})"
    );
}

/// A altura mais BAIXA a que o personagem chega, largado de `drop` acima da
/// altura em que ele descansa neste modo.
///
/// ⚠️ **`drop = 0` é a corrida de CONTROLE que mede o repouso** — escrever o
/// `Transform` a meio do play não teria efeito nenhum (com o relógio a andar a
/// pose é a SAÍDA, não a entrada; só o `settle` de pausa a lê), e a 1ª versão
/// desta fixture fazia exactamente isso: media `0,0000 m` de afundamento e
/// declarava *"a fixture não contém o fenómeno"* sobre um teleporte que nunca
/// aconteceu.
fn lowest_y(kinematic: bool, damping: f32, drop: f32) -> f32 {
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
        ))
        .id();
    if kinematic {
        make_kinematic(&mut sim, who);
    }
    let mut bridge = PhysicsBridge::new();
    let mut worst = f32::INFINITY;
    for t in 1..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        // Os primeiros tiques ainda descrevem a queda; o mínimo é o impacto.
        if t > 30 {
            worst = worst.min(pose(&sim).1);
        }
    }
    worst
}

/// **SONDA — de onde vem a diferença entre os dois modos no impacto.**
///
/// ```text
///    queda      din (m)      kin (m)
///   repouso din 0.9001  kin 0.5566
///      0.5       0.0516       0.0436
///      2.0       0.1488       0.0124
///      5.0       0.2611       0.0465
///     10.0       0.2956       0.0465
/// ```
///
/// O dinâmico **CRESCE com a queda** (é a mola a ser vencida); o cinemático fica
/// preso na PELE do controlador (`predict_ground = offset + 0.05`, o `skinWidth`
/// da Unity), e é isso que o torna estrutural em vez de afinado.
#[test]
#[ignore = "sonda de medição"]
fn probe_dip_against_drop() {
    let d = 0.25 * RideConfig::MAX_DAMPING;
    println!("\n{:>8} {:>12} {:>12}", "queda", "din (m)", "kin (m)");
    let dr = lowest_y(false, d, 0.0);
    let kr = lowest_y(true, d, 0.0);
    println!("  repouso din {dr:.4}  kin {kr:.4}");
    for drop in [0.5f32, 2.0, 5.0, 10.0] {
        println!(
            "{drop:>8.1} {:>12.4} {:>12.4}",
            dr - lowest_y(false, d, drop),
            kr - lowest_y(true, d, drop)
        );
    }
}

/// **O DEFEITO 2: a folga perdida no impacto — e o oráculo é o CRESCIMENTO.**
///
/// ⚠️ **A minha primeira barra estava errada e a medição a corrigiu:** eu pedi
/// *"o cinemático afunda menos de 1 cm"*, e ele afunda **4,7**. Mas os 4,7 cm são
/// a PELE do controlador (`predict_ground = offset + 0.05`), atravessada a
/// velocidade — e ela **não cresce com a queda**: 0,044 · 0,012 · 0,047 · 0,047
/// para 0,5 · 2 · 5 · 10 m. O dinâmico cresce sem parar: 0,052 · 0,149 · 0,261 ·
/// 0,296.
///
/// *Estrutural* não quer dizer *zero*; quer dizer **limitado por uma constante da
/// geometria em vez de pela energia do impacto** — e é exatamente essa a
/// diferença que a wave promete. Uma barra absoluta media a pele; esta mede a
/// propriedade.
#[test]
fn the_kinematic_dip_is_bounded_while_the_dynamic_one_grows_with_the_drop() {
    let d = 0.25 * RideConfig::MAX_DAMPING;
    let dip = |kin: bool, drop: f32| (lowest_y(kin, d, 0.0) - lowest_y(kin, d, drop)).max(0.0);
    let (dyn_low, dyn_high) = (dip(false, 0.5), dip(false, 10.0));
    let (kin_low, kin_high) = (dip(true, 0.5), dip(true, 10.0));
    assert!(
        dyn_high > 3.0 * dyn_low && dyn_high > 0.2,
        "a fixture tem de CONTER o fenomeno: o dinamico foi {dyn_low:.4} -> {dyn_high:.4}"
    );
    assert!(
        kin_high < 2.0 * kin_low.max(0.02) && kin_high < 0.08,
        "sob Snap o mergulho e' a PELE do controlador, nao a energia: \
         {kin_low:.4} -> {kin_high:.4} (dinamico {dyn_low:.4} -> {dyn_high:.4})"
    );
}

/// **A LEI é a mesma nos dois modos** — o MESMO input produz a MESMA intenção;
/// só a perna difere.
///
/// ⚠️ O oráculo é a DISTÂNCIA percorrida em 2 s de caminhada no plano: se o
/// Snap mexesse no `walk`, o personagem andaria a outra velocidade.
#[test]
fn the_law_is_the_same_in_both_modes() {
    let walk = |kinematic: bool| -> f32 {
        let (mut sim, mut bridge, who) = platform::scene(0.0, 0.0);
        if kinematic {
            make_kinematic(&mut sim, who);
        }
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let x0 = pose(&sim).0;
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        pose(&sim).0 - x0
    };
    let (d, k) = (walk(false), walk(true));
    assert!(
        d > 1.0 && k > 1.0,
        "os dois tem de ANDAR: dinamico {d:.3} m, cinematico {k:.3} m"
    );
    // 5% de folga: a arrancada difere porque a perna difere, a CRUZEIRO não.
    assert!(
        (d - k).abs() / d < 0.05,
        "a caminhada e' a MESMA lei: dinamico {d:.3} m vs cinematico {k:.3} m"
    );
}

/// **A plataforma móvel leva o personagem** (K7) — pelo `ground_velocity` que já
/// viajava no `GroundSample`.
#[test]
fn a_kinematic_player_rides_a_moving_platform() {
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
                    half_x: 3.0,
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
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
        ))
        .id();
    make_kinematic(&mut sim, who);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = pose(&sim).0;
    // O vagão anda 2 m/s para a direita, dirigido pela CENA (o `Transform`).
    for t in 61..=180u64 {
        {
            let mut e = sim.world_mut().entity_mut(wagon);
            if let Some(mut tr) = e.get_mut::<Transform>() {
                tr.translation.x += 2.0 / 60.0;
            }
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let travelled = pose(&sim).0 - x0;
    assert!(
        travelled > 3.0,
        "parado sobre o vagao ele tem de VIAJAR com ele: andou {travelled:.3} m de ~4"
    );
}

/// **A velocidade cinemática sobrevive a um SCRUB** (K5) — ela mora no
/// `PlayerState`, que é o valor que o ring de tiques âncora guarda.
#[test]
fn the_kinematic_velocity_survives_a_scrub() {
    let run = |scrub: bool| -> f32 {
        let (mut sim, mut bridge, who) = platform::scene(0.0, 0.0);
        make_kinematic(&mut sim, who);
        // Sobe-o e deixa-o cair, para haver velocidade a preservar.
        {
            let mut e = sim.world_mut().entity_mut(who);
            if let Some(mut t) = e.get_mut::<Transform>() {
                t.translation.y += 4.0;
            }
        }
        for t in 1..=40u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        if scrub {
            // Para trás e de volta — o replay tem de reproduzir a queda.
            bridge.dispatch(&mut sim, true, 10);
            for t in 11..=40u64 {
                bridge.dispatch(&mut sim, true, t);
            }
        }
        pose(&sim).1
    };
    let (plain, scrubbed) = (run(false), run(true));
    assert!(
        (plain - scrubbed).abs() < 1.0e-3,
        "o scrub tem de reproduzir a queda: {plain:.4} vs {scrubbed:.4}"
    );
}

/// **Os dois modos repousam a alturas DIFERENTES, e isso é o desenho** — a
/// consequência honesta de não haver perna.
///
/// ⚠️ Escrito como gate para ninguém o "consertar" fazendo o controlador flutuar:
/// sob Snap a cápsula É a silhueta e ela pousa no chão; sob Spring ela paira
/// `float_height` acima, que é a D1 (a cápsula flutuante) a fazer o seu trabalho.
#[test]
fn the_two_modes_rest_at_different_heights() {
    let rest = |kinematic: bool| -> f32 {
        let (mut sim, mut bridge, who) = platform::scene(0.0, 0.0);
        if kinematic {
            make_kinematic(&mut sim, who);
        }
        for t in 1..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        pose(&sim).1
    };
    let (d, k) = (rest(false), rest(true));
    assert!(
        d > k + 0.25,
        "o dinamico PAIRA e o cinematico POUSA: {d:.3} vs {k:.3}"
    );
    // ⚠️ **O contato é 1,0** (meia-altura 0,5 sobre o topo do chão em 0,5) e o
    // cinemático descansa um pouco acima: a PELE do controlador, que todo
    // controlador tem (o `skinWidth` da Unity). Medido, 1,057.
    let contact = 1.0;
    assert!(
        k >= contact - 0.005 && k < contact + 0.10,
        "a capsula pousa sobre o chao, mais a pele: y = {k:.3}, contato {contact:.3}"
    );
}

/// **SONDA — os números que a cena `=101` afirma.**
///
/// ⚠️ Ela roda ANTES de a mensagem do smoke ser escrita: a política do módulo é
/// que toda cena traz números MEDIDOS, e nesta jornada duas cenas já afirmaram
/// coisas que a medição desmentiu.
#[test]
#[ignore = "sonda de medição"]
fn probe_the_numbers_scene_101_claims() {
    let d = 0.25 * RideConfig::MAX_DAMPING;
    println!("\n=== CENA 101, os numeros ===");
    println!(
        "deriva de rampa (30deg, 10 s):  din {:.4}  kin {:.4}",
        ramp_creep(false, d),
        ramp_creep(true, d)
    );
    println!(
        "repouso no plano:               din {:.3}  kin {:.3}",
        {
            let (mut sim, mut bridge, _) = platform::scene(0.0, 0.0);
            for t in 1..=180u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            pose(&sim).1
        },
        {
            let (mut sim, mut bridge, who) = platform::scene(0.0, 0.0);
            make_kinematic(&mut sim, who);
            for t in 1..=180u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            pose(&sim).1
        }
    );
    let walk = |kin: bool| -> f32 {
        let (mut sim, mut bridge, who) = platform::scene(0.0, 0.0);
        if kin {
            make_kinematic(&mut sim, who);
        }
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let x0 = pose(&sim).0;
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        pose(&sim).0 - x0
    };
    let (wd, wk) = (walk(false), walk(true));
    println!(
        "caminhada em 2 s:               din {wd:.3}  kin {wk:.3}  ({:.1}%)",
        (wd - wk).abs() / wd * 100.0
    );
}
