//! **ANDAR** (W3) — os gates de COMPORTAMENTO, com o rapier de verdade.
//!
//! A lei pura tem gates próprios na `ph2d-platformer` (dado um `Δv`, qual
//! aceleração). Estes fazem a outra pergunta, a que só a simulação responde:
//! *o personagem de fato chega à velocidade pedida, para no lugar, sobe a rampa
//! rasa, escorrega na íngreme e cavalga a plataforma?*
//!
//! ⚠️ **`LockRotation` faz parte de toda fixture porque faz parte do DESENHO**
//! (D4 do plano): em 2D trava-se a rotação de um personagem, como Unity e Godot
//! fazem. Sem ele a cápsula tomba e a perna vira um pêndulo — e o que estes
//! gates medem deixa de ser a caminhada.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// ⚠️ **0,9 e não o `0,5` do ponto de partida, e o motivo é GEOMETRIA.**
///
/// O sensor mede na vertical, mas quem encosta na rampa é a cápsula ao longo da
/// normal dela: flutuar de verdade exige
/// `float_height > half_height + radius / cos(max_slope)`
/// ([`ph2d_platformer::RideConfig::min_float_height`], com a tabela). Para a
/// cápsula destes gates (`0,3` / `0,2`) o mínimo já é `0,5` **no plano** — ou
/// seja, o ponto de partida a deixa TANGENTE, e qualquer inclinação a faz
/// penetrar. Com `0,9` ela paira até 70,5°, que cobre as duas rampas daqui.
const FLOAT_HEIGHT: f32 = 0.9;

/// Um chão (estático, opcionalmente inclinado) e um player em cima dele.
///
/// `slope_rad` gira o chão; o player nasce um pouco acima do ponto de contato
/// para a mola o pegar no primeiro tick.
fn scene(slope_rad: f32, start_x: f32) -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
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
        Transform {
            rotation: slope_rad,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    // O topo do chão sob `start_x`, quando ele está inclinado, sobe com o x.
    let top = 0.5 / slope_rad.cos() + start_x * slope_rad.tan();
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
                float_height: FLOAT_HEIGHT,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(start_x, top + FLOAT_HEIGHT)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

/// **O gate da wave.** Ele anda na velocidade autorada, para, e FICA parado.
///
/// ## Os números MEDIDOS (2026-08-03, `speed = 6`, `acceleration = 60`)
///
/// | grandeza | valor |
/// |---|---|
/// | velocidade em regime | ver a asserção (6,00 ± 0,10 m/s) |
/// | distância de frenagem de 6 m/s | ver o `eprintln` |
/// | **deriva residual no 2º meio-segundo** | **< 1 mm** |
///
/// ⚠️ **O oráculo é a DERIVA RESIDUAL, não a distância de frenagem** — e a
/// primeira versão deste gate errava exatamente nisso. Frear 6 m/s tem uma
/// distância; exigir que ela fosse zero seria exigir teletransporte, que é
/// precisamente o que a regra `|Δv| ≤ a·dt` existe para recusar. O que o boost
/// entrega é a outra metade: depois de parar, o personagem **não escorrega mais
/// um milímetro**. Uma força de freio pousaria perto de zero e deixaria resíduo.
#[test]
fn the_player_walks_at_the_authored_speed_and_then_stays_put() {
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    bridge.set_player_input(player, PlayerInput { drive: 1.0 });

    let mut tick = 0_u64;
    // 2 s andando.
    for _ in 0..120 {
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x_a, _) = pose(&sim);
    tick += 1;
    bridge.dispatch(&mut sim, true, tick);
    let (x_b, _) = pose(&sim);
    let speed = (x_b - x_a) * 60.0;

    // Solta: o alvo passa a ser ficar parado (relativo ao chão).
    bridge.set_player_input(player, PlayerInput::default());
    let (x_release, _) = pose(&sim);
    for _ in 0..30 {
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x_half, _) = pose(&sim);
    for _ in 0..30 {
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x_end, _) = pose(&sim);

    let braking = x_half - x_release;
    let residual = (x_end - x_half).abs();
    eprintln!(
        "W3 ANDAR | regime {speed:.4} m/s | frenagem {braking:.4} m | deriva residual {residual:.6} m"
    );

    assert!(
        (speed - 6.0).abs() < 0.1,
        "a velocidade em regime tem de ser a autorada (6 m/s): {speed:.4}"
    );
    assert!(
        braking > 0.0 && braking < 0.4,
        "frear 6 m/s custa uma distancia, e ela tem de ser curta: {braking:.4} m"
    );
    assert!(
        residual < 0.001,
        "depois de parar ele NAO escorrega: derivou {residual:.6} m em meio segundo"
    );
}

/// ⚠️ **Sem entrada, nada anda** — o controle é do jogador, e um player parado
/// com o dedo fora do teclado fica parado.
///
/// Sem este gate, um `drive` que vazasse de outra entidade (ou um default
/// diferente de zero) passaria despercebido: o gate acima só olha para quem foi
/// dirigido.
#[test]
fn without_input_the_player_does_not_walk() {
    let (mut sim, mut bridge, _player) = scene(0.0, 0.0);
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x, _) = pose(&sim);
    assert!(x.abs() < 0.02, "ninguem pediu para andar: x = {x:.4}");
}

/// **Uma rampa RASA se sobe na velocidade autorada** — e o oráculo é a
/// velocidade AO LONGO da rampa, não a subida.
///
/// ⚠️ **A primeira versão deste gate media a ALTURA, e ela não distingue nada:**
/// mesmo com o eixo de caminhada na horizontal o personagem sobe, porque a
/// SUPERFÍCIE o levanta enquanto ele avança e a mola o segue — a trajetória é
/// imposta pela rampa, não pela lei. A mutação (eixo horizontal) passava.
///
/// O que muda entre as duas leis é a VELOCIDADE: com o eixo na tangente, andar
/// a `speed` significa `speed` **ao longo da rampa**; com o eixo horizontal
/// significa `speed` na horizontal, e o percurso sai `speed / cos θ` — 15% mais
/// rápido numa rampa de 30°, e a subida inteira com um número que ninguém
/// autorou.
#[test]
fn a_shallow_ramp_is_climbed_at_the_authored_speed() {
    let slope = 30.0_f32.to_radians();
    let (mut sim, mut bridge, player) = scene(slope, 0.0);
    bridge.set_player_input(player, PlayerInput { drive: 1.0 });
    let (x0, y0) = pose(&sim);
    // 1 s para acelerar, e depois um tick medido em regime.
    for tick in 1..=60 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (xa, ya) = pose(&sim);
    bridge.dispatch(&mut sim, true, 61);
    let (xb, yb) = pose(&sim);
    let along = ((xb - xa).powi(2) + (yb - ya).powi(2)).sqrt() * 60.0;

    for tick in 62..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x1, y1) = pose(&sim);
    let climbed = y1 - y0;
    let run = x1 - x0;
    eprintln!(
        "W3 RAMPA 30deg | avancou {run:.3} m | subiu {climbed:.3} m | \
         velocidade AO LONGO da rampa {along:.3} m/s (horizontal daria {:.3})",
        6.0 / slope.cos()
    );

    assert!(climbed > 1.5, "tem de SUBIR: {climbed:.3} m");
    assert!(
        (along - 6.0).abs() < 0.15,
        "a velocidade autorada e' a do PERCURSO, nao a horizontal: {along:.3} m/s"
    );
    // A trajetória é a da rampa: a razão sobe/anda é a tangente dela.
    let measured = climbed / run;
    assert!(
        (measured - slope.tan()).abs() < 0.05,
        "o caminho tem de ser o da rampa: {measured:.3} vs {:.3}",
        slope.tan()
    );
}

/// ⚠️ **Uma rampa ÍNGREME não se sobe — escorrega.**
///
/// É a metade da `footing` que a W3 acrescentou. Com o limite em 45° e a rampa
/// em 60°, a superfície deixa de ser chão: a perna se cala, a gravidade age
/// inteira, o collider encosta e o personagem desce — mesmo com o dedo no
/// acelerador rampa acima.
#[test]
fn a_ramp_steeper_than_the_limit_is_slipped_down_not_climbed() {
    let slope = 60.0_f32.to_radians();
    let (mut sim, mut bridge, player) = scene(slope, 0.0);
    bridge.set_player_input(player, PlayerInput { drive: 1.0 });
    let (_, y0) = pose(&sim);
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, y1) = pose(&sim);
    let climbed = y1 - y0;
    eprintln!("W3 RAMPA 60deg (limite 45) | variacao de altura {climbed:+.3} m");
    assert!(
        climbed < 0.0,
        "acima do limite o personagem DESCE, nao sobe: {climbed:+.3} m"
    );
}

/// O limite é AUTORADO: a mesma rampa de 60° vira caminhável quando o artista
/// sobe o número.
///
/// ⚠️ É o controle do gate acima — sem ele, *"não subiu"* seria compatível com
/// *"a caminhada não funciona em rampa nenhuma"*.
#[test]
fn raising_the_limit_makes_the_steep_ramp_climbable() {
    let slope = 60.0_f32.to_radians();
    let (mut sim, mut bridge, player) = scene(slope, 0.0);
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.max_slope_deg = 70.0;
    }
    bridge.set_player_input(player, PlayerInput { drive: 1.0 });
    let (_, y0) = pose(&sim);
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, y1) = pose(&sim);
    let climbed = y1 - y0;
    eprintln!("W3 RAMPA 60deg (limite 70) | subiu {climbed:+.3} m");
    assert!(
        climbed > 1.0,
        "com o limite acima da rampa ela e' caminhavel: {climbed:+.3} m"
    );
}

/// ⚠️ **Cavalgar uma plataforma cai de graça** — a lei mede tudo relativo ao
/// chão, então ficar parado sobre um vagão em movimento é ficar parado.
///
/// A plataforma é `Kinematic` dirigida pelo `Transform` — o caminho que a cena
/// e a timeline usam —, e o gate mede que o personagem viaja com ela **sem** que
/// ninguém tenha escrito uma linha de código de plataforma.
#[test]
fn the_player_rides_a_moving_platform() {
    let mut sim = SimWorld::new();
    let platform = sim
        .world_mut()
        .spawn((
            Name::new("Platform"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 3.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
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
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.5 + FLOAT_HEIGHT)),
    ));
    let mut bridge = PhysicsBridge::new();

    // Meio segundo parado para a mola assentar.
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x0, _) = pose(&sim);

    // A plataforma anda 2 m/s para a direita por 1,5 s, dirigida pelo Transform.
    let speed = 2.0_f32;
    for i in 1..=90 {
        let t = i as f32 / 60.0;
        {
            let mut e = sim.world_mut().entity_mut(platform);
            let mut tr = e.get_mut::<Transform>().unwrap();
            tr.translation.x = speed * t;
        }
        bridge.dispatch(&mut sim, true, 30 + i as u64);
    }
    let (x1, _) = pose(&sim);
    let carried = x1 - x0;
    let expected = speed * 1.5;
    eprintln!("W3 PLATAFORMA | levou o player {carried:.3} m (a plataforma andou {expected:.3})");
    assert!(
        (carried - expected).abs() < 0.25,
        "o player tem de viajar COM a plataforma: {carried:.3} vs {expected:.3}"
    );
}

/// ⚠️ **Um corpo sem o componente ignora a entrada** — a wave segue byte-neutra
/// para o resto do módulo, mesmo com alguém escrevendo `drive` nele.
#[test]
fn input_on_a_body_without_the_component_does_nothing() {
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
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let crate_e = sim
        .world_mut()
        .spawn((
            Name::new("Player"), // o nome é o mesmo; o que falta é o COMPONENTE
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(crate_e, PlayerInput { drive: 1.0 });
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x, _) = pose(&sim);
    assert!(
        x.abs() < 0.05,
        "sem o componente, a entrada nao move nada: x = {x:.4}"
    );
}
