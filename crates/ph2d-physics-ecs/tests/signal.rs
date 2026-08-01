//! **A física GRITA um nome** (W-Signal) — o publicador que faltava.
//!
//! Quatro canais de leitura existem desde o W7 e **nenhum fazia nada acontecer**.
//! ⚠️ E a decisão que a nota chamava de *"cross-line, do Enio"* já estava tomada
//! e escrita no `render_loop`: *"gameplay é um dos consumidores diferidos do
//! MESMO outbox"*. Esta wave é o publicador, não um barramento novo.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, PhysicsBridge, RigidBody,
    SignalOnHit,
};

fn cuboid(hx: f32, hy: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        ..Collider::default()
    }
}

fn ground(sim: &mut SimWorld, signal: Option<&str>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(10.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    if let Some(s) = signal {
        e.insert(SignalOnHit(s.to_string()));
    }
    e.id()
}

fn ball(sim: &mut SimWorld, y: f32, signal: Option<&str>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Ball"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        cuboid(0.3, 0.3),
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
    if let Some(s) = signal {
        e.insert(SignalOnHit(s.to_string()));
    }
    e.id()
}

/// Roda `ticks` e devolve TODO sinal emitido no caminho.
fn run(sim: &mut SimWorld, ticks: u64) -> Vec<(String, Entity, Entity)> {
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 0..=ticks {
        bridge.dispatch(sim, true, t);
        for s in bridge.signal_events(sim) {
            out.push((s.name, s.source, s.other));
        }
    }
    out
}

/// **A afirmação da wave:** um corpo marcado que é atingido GRITA o nome dele.
///
/// Mutação (o publicador ignorar o `SignalOnHit`) ⇒ nada é emitido, e os quatro
/// canais de leitura voltam a não fazer nada acontecer.
#[test]
fn a_marked_body_that_is_hit_shouts_its_name() {
    let mut sim = SimWorld::new();
    let g = ground(&mut sim, Some("landed"));
    let b = ball(&mut sim, 2.0, None);
    let fired = run(&mut sim, 120);
    assert!(
        fired
            .iter()
            .any(|(n, s, o)| n == "landed" && *s == g && *o == b),
        "o chão marcado não gritou: {fired:?}"
    );
}

/// **O CONTROLE, e ele é a metade que importa:** sem componente, silêncio total.
///
/// Uma cena que já existe não pode passar a emitir nada — a ausência do
/// componente É o default de todo projeto salvo.
#[test]
fn an_unmarked_scene_is_silent() {
    let mut sim = SimWorld::new();
    ground(&mut sim, None);
    ball(&mut sim, 2.0, None);
    assert!(
        run(&mut sim, 120).is_empty(),
        "uma cena sem sinal emitiu algo"
    );
}

/// **Um nome em BRANCO não é um sinal** — a regra emprestada palavra por palavra
/// do `set_marker_signal` da timeline: *um sinal sem nome não é um contrato que
/// alguém possa casar*.
#[test]
fn a_blank_name_is_not_a_signal() {
    let mut sim = SimWorld::new();
    ground(&mut sim, Some("   "));
    ball(&mut sim, 2.0, None);
    assert!(
        run(&mut sim, 120).is_empty(),
        "um nome em branco virou sinal"
    );
}

/// **Os DOIS lados podem gritar**, cada um com o `other` certo — um contato é
/// simétrico, e escolher um lado exigiria uma regra que ninguém autorou.
#[test]
fn both_ends_of_a_contact_can_shout_each_with_its_own_other() {
    let mut sim = SimWorld::new();
    let g = ground(&mut sim, Some("floor"));
    let b = ball(&mut sim, 2.0, Some("ball"));
    let fired = run(&mut sim, 120);
    assert!(
        fired
            .iter()
            .any(|(n, s, o)| n == "floor" && *s == g && *o == b),
        "faltou o lado do chão: {fired:?}"
    );
    assert!(
        fired
            .iter()
            .any(|(n, s, o)| n == "ball" && *s == b && *o == g),
        "faltou o lado da bola: {fired:?}"
    );
}

/// **Um SENSOR grita quando algo ENTRA nele** — o caso canônico de gameplay (a
/// porta), e o que um canal só de contato deixaria em silêncio: um sensor é
/// atravessado, então ele nunca gera contato.
///
/// Mutação (não diferenciar o conjunto permanente de triggers) ⇒ a porta nunca
/// dispara.
#[test]
fn a_sensor_shouts_when_something_enters_it() {
    let mut sim = SimWorld::new();
    let zone = sim
        .world_mut()
        .spawn((
            Name::new("Door"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                ..cuboid(1.0, 1.0)
            },
            SignalOnHit("door".to_string()),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let b = ball(&mut sim, 3.0, None);
    let fired = run(&mut sim, 120);
    assert!(
        fired
            .iter()
            .any(|(n, s, o)| n == "door" && *s == zone && *o == b),
        "a porta não disparou: {fired:?}"
    );
}

/// **Uma ENTRADA, não uma por quadro** — um sensor com algo parado dentro dele
/// grita UMA vez, não sessenta vezes por segundo.
///
/// É a diferença entre uma transição e o conjunto permanente, e é o que torna o
/// canal consumível: um som que toca por quadro não é um som.
#[test]
fn something_resting_inside_a_sensor_shouts_once_not_every_frame() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Door"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(2.0, 2.0)
        },
        SignalOnHit("door".to_string()),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // Gravidade zero: a bola nasce dentro e FICA, então o conjunto permanente
    // nunca muda depois do primeiro quadro.
    sim.world_mut().spawn((
        Name::new("Ball"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        cuboid(0.3, 0.3),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let n = run(&mut sim, 120)
        .iter()
        .filter(|(name, _, _)| name == "door")
        .count();
    assert_eq!(n, 1, "a porta gritou {n} vezes com uma bola parada dentro");
}

/// **Um scrub NÃO é uma tempestade de chegadas.**
///
/// O conjunto permanente é recomputado do zero, então um diff ingênuo faz de toda
/// descontinuidade do relógio uma enxurrada de eventos. A disciplina é a MESMA do
/// canal de contato: `rewind` re-baseliza em silêncio.
///
/// Mutação (não derrubar `triggers_continuous` no rewind) ⇒ o scrub grita.
#[test]
fn a_scrub_backwards_is_not_a_storm_of_arrivals() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Door"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(2.0, 2.0)
        },
        SignalOnHit("door".to_string()),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Ball"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        cuboid(0.3, 0.3),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    for t in 0..=60 {
        bridge.dispatch(&mut sim, true, t);
        let _ = bridge.signal_events(&sim);
    }
    // Volta o relógio: uma descontinuidade, não uma chegada.
    let mut fired = 0;
    for t in (0..=30).rev() {
        bridge.dispatch(&mut sim, true, t);
        fired += bridge.signal_events(&sim).len();
    }
    assert_eq!(fired, 0, "o scrub emitiu {fired} sinal(is)");
}

/// **SONDA:** a que velocidade um corpo atravessa um sensor dentro de UM tick — o
/// número que torna honesta a escolha de diferenciar os triggers por DISPATCH e
/// não por tick.
///
/// `cargo test -p ph2d-physics-ecs --release measure_sensor_blind_speed --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn measure_sensor_blind_speed() {
    println!("\n=== a que velocidade um sensor de 1,0 m fica CEGO? ===");
    for v in [30.0_f32, 60.0, 120.0, 240.0, 280.0, 320.0, 480.0, 960.0] {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Door"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                ..cuboid(0.5, 0.5)
            },
            SignalOnHit("door".to_string()),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ));
        sim.world_mut().spawn((
            Name::new("Bullet"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            GravityScale(0.0),
            InitialVelocity {
                linvel: [v, 0.0],
                angvel: 0.0,
            },
            cuboid(0.05, 0.05),
            Transform::from_translation(Vec2::new(-3.0, 0.0)),
        ));
        let n = run(&mut sim, 60)
            .iter()
            .filter(|(name, _, _)| name == "door")
            .count();
        println!("   {v:>5.0} m/s -> {n} sinal(is)");
    }
    println!(
        "   (a aritmetica previa cegueira a 2h/dt = 60 m/s; a fase LARGA trabalha com
    AABBs preditas, entao o par entra no grafo pelo MOVIMENTO -- cinco vezes
    a margem prevista. Acima disso a resposta e o CCD, que ja existe.)"
    );
}

/// **Algo que ATRAVESSA um sensor grita UMA vez, e o canal se cala depois.**
///
/// ⚠️ **Este gate nasceu de um defeito MEU que a sonda achou.** O
/// `rebuild_triggers` tem um early-out quando nada sobrepõe — construir os dois
/// mapas numa cena sem triggers seria desperdício —, e a primeira versão do diff
/// ficava DEPOIS dele. Consequência: assim que o grafo esvaziava, nada mais
/// limpava `trigger_events`, e o publicador re-emitia a última entrada **para
/// sempre**: medido, uma bala a 60 m/s disparava a porta **58 vezes em 60 ticks**.
///
/// *Um early-out que pula uma limpeza não é um atalho, é um vazamento.*
///
/// O irmão `something_resting_inside_a_sensor...` **não pega isto**: lá a bola
/// nunca sai, então o grafo nunca esvazia e o early-out nunca roda. O que separa
/// os dois é a SAÍDA.
#[test]
fn something_passing_through_a_sensor_shouts_once_and_then_the_channel_goes_quiet() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Door"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(0.5, 0.5)
        },
        SignalOnHit("door".to_string()),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Bullet"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        GravityScale(0.0),
        InitialVelocity {
            linvel: [60.0, 0.0],
            angvel: 0.0,
        },
        cuboid(0.05, 0.05),
        Transform::from_translation(Vec2::new(-3.0, 0.0)),
    ));
    let n = run(&mut sim, 60)
        .iter()
        .filter(|(name, _, _)| name == "door")
        .count();
    assert_eq!(
        n, 1,
        "a porta gritou {n} vezes para UMA travessia -- o canal esta' vazando o \
         ultimo evento"
    );
}

/// **Um scrub que VOLTA para dentro de uma sobreposição é silencioso.**
///
/// O conjunto permanente é derivado da geometria, então rebobinar para um tick em
/// que algo estava dentro devolve esse conjunto — e um diff contra a baseline
/// PRÉ-scrub reportaria uma chegada que o artista não tocou. A disciplina é a
/// mesma do canal de contato: `rewind` re-baseliza em silêncio.
///
/// ⚠️ **Este gate substitui um meu que afirmava o OPOSTO da lei do módulo.** Eu
/// havia escrito que pôr algo dentro de um sensor com a física DESARMADA e depois
/// re-armar tinha de ser silencioso — e ele falhou **no código correto**, porque a
/// lei declarada do canal irmão é a leitura da Unity: *a baseline nasce vazia e o
/// 1º frame simulado reporta o que achar; antes do primeiro passo não existe
/// verdade anterior*. (O mecanismo: durante o `hold` a fase estreita fica PARADA,
/// então o conjunto lido ali ainda é o de antes da mão.)
///
/// Mutação (o `rewind` não descartar a história) ⇒ o scrub grita.
#[test]
fn a_scrub_back_into_an_overlap_is_silent() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Door"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(2.0, 0.4)
        },
        SignalOnHit("door".to_string()),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    ball(&mut sim, 3.0, None);
    let mut bridge = PhysicsBridge::new();
    // Acha um tick em que a bola ENTROU, e segue até ela estar longe.
    let mut inside_at = None;
    for t in 0..=180u64 {
        bridge.dispatch(&mut sim, true, t);
        if !bridge.trigger_events().is_empty() {
            inside_at = Some(t);
        }
    }
    let inside_at = inside_at.expect("a bola nunca atravessou a porta");
    // O relógio está no fim, com a bola LONGE. Volta ao instante da travessia:
    // uma descontinuidade, não uma chegada.
    bridge.dispatch(&mut sim, true, inside_at);
    let fired = bridge.signal_events(&sim);
    assert!(
        fired.is_empty(),
        "o scrub de volta para dentro da sobreposição gritou: {fired:?}"
    );
}
