//! **ANDAR** (W3) — os gates de COMPORTAMENTO, com o rapier de verdade.
//!
//! A lei pura tem gates próprios na `ph2d-platformer` (dado um `Δv`, qual
//! aceleração). Estes fazem a outra pergunta, a que só a simulação responde:
//! *o personagem de fato chega à velocidade pedida, para no lugar, sobe a rampa
//! rasa, escorrega na íngreme e cavalga a plataforma?*

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};
use scene_fixture::{FLOAT_HEIGHT, pose, scene};

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
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );

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
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
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
///
/// ⚠️ **A BARRA foi RE-MEDIDA na W9, e o motivo é que ela estava verde sobre o
/// bug** (report do Enio, 2026-08-04). O oráculo era `climbed < 0.0` e o número
/// que ele aceitava era **−0,047 m** — ou seja, *"o personagem fica GRUDADO"*
/// passava por *"escorrega"*. Com a lei do `no_uphill` a mesma cena mede
/// **−49,9 m**, então a barra passa a ser uma DESCIDA de verdade: mais de uma
/// altura de corpo em 3 s.
#[test]
fn a_ramp_steeper_than_the_limit_is_slipped_down_not_climbed() {
    let slope = 60.0_f32.to_radians();
    let (mut sim, mut bridge, player) = scene(slope, 0.0);
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let (_, y0) = pose(&sim);
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, y1) = pose(&sim);
    let climbed = y1 - y0;
    eprintln!("W3 RAMPA 60deg (limite 45) | variacao de altura {climbed:+.3} m");
    assert!(
        climbed < -1.0,
        "acima do limite o personagem ESCORREGA, e escorregar e' descer -- \
         ficar grudado tambem satisfaz `< 0` e nao e' o produto: {climbed:+.3} m"
    );
}

/// **O limite de rampa é o ângulo que o personagem DE FATO sobe** (W9).
///
/// # O report, e o número que ele valia
///
/// Enio (2026-08-04): *"Max Slope na UI aparece 45, mas o player sobe até
/// aproximadamente 60 graus."* Medido antes de tocar em código, com o limite em
/// 45: **44° subia +12,29 m e 46° subia +4,38 m** — o produto honrava um teto
/// efetivo de ~52°, não o autorado.
///
/// A fixture é o par que **cerca** o limite: um grau abaixo e um acima. O gate
/// anterior media 60°, que já ficava *depois* do teto acidental — a razão exata
/// de o defeito ter atravessado a wave inteira com a suíte verde.
///
/// **Mutação que deve sangrar:** tirar o `no_uphill` do `player_motor`.
#[test]
fn the_slope_limit_is_the_angle_the_player_actually_climbs() {
    let climb = |deg: f32| {
        let (mut sim, mut bridge, player) = scene(deg.to_radians(), 0.0);
        for tick in 1..=30 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let (_, y0) = pose(&sim);
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        for tick in 31..=210 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let (_, y1) = pose(&sim);
        y1 - y0
    };
    // O limite autorado é o do ponto de partida: 45°.
    let below = climb(44.0);
    let above = climb(46.0);
    eprintln!("W9 LIMITE 45deg | rampa 44deg {below:+.3} m | rampa 46deg {above:+.3} m");
    assert!(
        below > 1.0,
        "um grau ABAIXO do limite tem de ser escalavel: {below:+.3} m"
    );
    assert!(
        above < -1.0,
        "um grau ACIMA do limite tem de escorregar -- era ele que subia +4,38 m: \
         {above:+.3} m"
    );
}

/// ⚠️ **O teto de subida NÃO é função de outro knob** (W9).
///
/// A ablação por ENTRADA que diagnosticou o defeito vira o gate que o pina: a
/// MESMA rampa (50°, acima do limite de 45) com três acelerações aéreas
/// diferentes tem de dar o MESMO veredito. Antes da lei ela dava três respostas
/// — `air = 0` escorregava 28,9 m, `air = 5` ficava parado, `air = 20` **subia
/// 4,0 m** —, e é essa dependência que fazia de *Max Slope* um número que o
/// produto não honrava ([[feedback_ergonomics_verdict_is_a_design_bug]]).
///
/// **Mutação que deve sangrar:** tirar o `no_uphill` do `player_motor`.
#[test]
fn the_climbing_ceiling_does_not_move_with_the_air_acceleration() {
    let climb = |air: f32| {
        let (mut sim, mut bridge, player) = scene(50.0_f32.to_radians(), 0.0);
        {
            let mut e = sim.world_mut().entity_mut(player);
            let mut p = e.get_mut::<PlatformPlayer>().unwrap();
            p.air_acceleration = air;
        }
        for tick in 1..=30 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let (_, y0) = pose(&sim);
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        for tick in 31..=210 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let (_, y1) = pose(&sim);
        y1 - y0
    };
    let slow = climb(0.0);
    let mid = climb(5.0);
    let fast = climb(20.0);
    eprintln!(
        "W9 RAMPA 50deg (limite 45) | air=0 {slow:+.3} | air=5 {mid:+.3} | air=20 {fast:+.3}"
    );
    for (label, d) in [("0", slow), ("5", mid), ("20", fast)] {
        assert!(
            d < -1.0,
            "com `air_acceleration = {label}` a rampa acima do limite tem de \
             escorregar do mesmo jeito: {d:+.3} m"
        );
    }
    // E não só o veredito: a DESCIDA é praticamente a mesma, porque o que o
    // controle aéreo ainda governa (o empurrão morro ABAIXO) é de segunda ordem.
    let spread = (slow - fast).abs();
    assert!(
        spread < 1.0,
        "o teto de subida nao pode ser funcao da aceleracao aerea: as tres \
         descidas diferem em {spread:.3} m"
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
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
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
    bridge.set_player_input(
        crate_e,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for tick in 1..=180 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (x, _) = pose(&sim);
    assert!(
        x.abs() < 0.05,
        "sem o componente, a entrada nao move nada: x = {x:.4}"
    );
}
