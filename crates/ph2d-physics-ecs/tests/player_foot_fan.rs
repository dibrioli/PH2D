//! **A PERNA É UM LEQUE** (`W-Probes2`) — o defeito que a pergunta do Enio
//! (*"Por que a perna não poderia ter mais de um?"*) expôs, com o número ao lado.
//!
//! ⚠️ **Não é pedido de feature, é bug medido.** Com um raio só no centro, um
//! personagem PARADO sobre uma fenda de 10 cm — num corpo de 40 cm, cujas bordas
//! a suportam fisicamente — afunda **0,411 m, 46% do `float_height`**, e a 40 cm
//! ele cai para fora do mundo (113 m). É exatamente a doença que o flanco teve
//! na W13, onde uma fresta recusava o pulo de parede por inteiro; lá a cura foram
//! três amostras, e a perna nunca tinha sido medida.
//!
//! ⚠️ **A regra de redução não é uma decisão nova** — é a que o `cling` já ship a
//! no flanco: fica o acerto mais PRÓXIMO, e o raio do meio (índice 0 do
//! `wall_offsets`) ganha todo empate. Sobre chão plano os N respondem o mesmo
//! número e o resultado é o de sempre.
//!
//! ⚠️ **E o CONTROLE é metade do gate:** a cura não é *"nunca cair"* — é *"não
//! cair onde o corpo de facto atravessa"*. Uma fenda mais larga que o corpo tem
//! de continuar a derrubá-lo, senão o gate ficaria verde sobre um personagem que
//! flutua sobre abismos.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    ProbeKind, RigidBody,
};

const FLOAT: f32 = 0.9;
/// Meia-largura do corpo: a cápsula tem raio 0,2, logo ele mede 0,40 m.
const HALF_W: f32 = 0.2;

/// Deixa o personagem PARADO sobre o meio de uma fenda de `gap` metros e devolve
/// **quanto ele afundou** abaixo da altura de repouso.
///
/// ⚠️ **PARADO de propósito.** O primeiro corte desta medição usava `drive = 1.0`
/// e mediu **0,000 m em toda fenda**: a `drive` cheia atravessa em poucos tiques
/// e a gravidade mal age, então o número media a velocidade e não o sensor.
fn dip_over_gap(gap: f32, foot_samples: u16) -> f32 {
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
    slab("Left", Vec2::new(-10.0, -0.5), [10.0, 0.5]);
    slab("Right", Vec2::new(gap + 10.0, -0.5), [10.0, 0.5]);

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
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                foot_samples,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(gap * 0.5, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let mut lowest = f32::INFINITY;
    for i in 1..=240u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
        let t = sim.world().get::<Transform>(player).expect("transform");
        lowest = lowest.min(t.translation.y);
    }
    FLOAT - lowest
}

/// **O leque segura o que o corpo atravessa — e larga o que ele não atravessa.**
///
/// ⚠️ Os dois lados vivem no MESMO teste de propósito: separados, o primeiro
/// passaria com uma perna que nunca solta o chão, que é o bug oposto.
#[test]
fn the_foot_fan_holds_a_gap_the_body_spans_and_still_drops_a_wider_one() {
    // 0,10 m e 0,30 m: o corpo (0,40 m) cobre as duas por larga margem.
    for gap in [0.10_f32, 0.20, 0.30] {
        let dip = dip_over_gap(gap, 3);
        assert!(
            dip < 0.01,
            "uma fenda de {gap:.2} m que o corpo de {:.2} m atravessa nao pode \
             afundar a perna, e afundou {dip:.3} m",
            HALF_W * 2.0,
        );
    }
    // O CONTROLE: 0,60 m é mais largo que o corpo — ele TEM de cair.
    let wide = dip_over_gap(0.60, 3);
    assert!(
        wide > 1.0,
        "uma fenda mais larga que o corpo tem de o deixar cair, e ele so' desceu {wide:.3} m"
    );
}

/// **O DEFEITO que a wave curou, com o número** — a perna de um raio só.
///
/// ⚠️ Este gate afirma que a cura foi *necessária*, não só que o produto de hoje
/// está bem: sem ele, alguém que devolvesse o default a `1` teria a suíte inteira
/// verde menos o irmão de cima, sem saber o quanto isso custa.
#[test]
fn a_single_ray_leg_sinks_over_a_gap_the_body_spans_and_this_is_its_number() {
    let one = dip_over_gap(0.10, 1);
    let fan = dip_over_gap(0.10, 3);
    assert!(
        one > 0.3,
        "a perna de UM raio afundava 0,411 m sobre esta fenda; medido {one:.3} m"
    );
    assert!(fan < 0.01, "e a de tres nao afunda; medido {fan:.3} m");
}

/// **A contagem autorada chega aos raios que a ponte casta** — a quarta condição
/// da política desta linha (*a sequência leva a algum lugar*).
///
/// ⚠️ E o par ímpar é a mesma lei do flanco: uma contagem PAR sobe para a ímpar
/// seguinte, porque o raio do meio é a âncora do desempate.
#[test]
fn the_authored_foot_count_reaches_the_rays_the_bridge_casts() {
    let count = |n: u16| {
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
                        radius: HALF_W,
                    },
                    ..Collider::default()
                },
                LockRotation,
                PlatformPlayer {
                    float_height: FLOAT,
                    foot_samples: n,
                    ..PlatformPlayer::default()
                },
                Transform::from_translation(Vec2::new(0.0, FLOAT)),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        for i in 1..=30u64 {
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, i);
        }
        bridge
            .player_probe_marks()
            .iter()
            .filter(|m| m.kind == ProbeKind::Ground)
            .count()
    };
    assert_eq!(count(1), 1, "um raio pedido, um castado");
    assert_eq!(count(3), 3, "o default sao tres");
    assert_eq!(count(9), 9, "nove pedidos, nove castados");
    assert_eq!(
        count(8),
        9,
        "uma contagem PAR sobe para a impar seguinte (o meio e' a ancora do desempate)"
    );
}

/// **Cada pé mostra o que ELE achou** — o overlay é um relato, não um resumo.
///
/// ⚠️ Antes desta wave a distância era carimbada no raio do MEIO e os outros
/// desenhavam `Clear`, o que era verdade enquanto só o do meio era castado.
/// Depois da redução o vencedor pode ser o da borda, e um desenho que insistisse
/// no meio diria que o pé do meio achou chão num tique em que quem achou foi
/// outro — a segunda porta que este codebase já pagou muitas vezes.
///
/// Parado sobre uma fenda de 0,30 m com três pés: o do meio está sobre o vazio
/// (`Clear`) e os dois de fora estão sobre as bordas (`Hit`). **É este contraste
/// que torna o leque legível na tela.**
#[test]
fn each_foot_draws_the_answer_it_got_not_the_reduced_verdict() {
    use ph2d_physics_ecs::ProbeState;

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
    let gap = 0.30_f32;
    slab("Left", Vec2::new(-10.0, -0.5), [10.0, 0.5]);
    slab("Right", Vec2::new(gap + 10.0, -0.5), [10.0, 0.5]);

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
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(gap * 0.5, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    for i in 1..=60u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
    }

    let feet: Vec<_> = bridge
        .player_probe_marks()
        .iter()
        .filter(|m| m.kind == ProbeKind::Ground)
        .map(|m| m.state)
        .collect();
    assert_eq!(feet.len(), 3, "o default sao tres pes");
    assert_eq!(
        feet[0],
        ProbeState::Clear,
        "o pe do MEIO esta' sobre o vazio da fenda"
    );
    assert!(
        feet[1..].iter().all(|s| *s == ProbeState::Hit),
        "os dois pes de FORA estao sobre as bordas, e e' esse contraste que o \
         leque existe para mostrar: {feet:?}"
    );
}

/// **O leque cavalga o chão mais alto que um pé alcança — desde que ele seja
/// CAMINHÁVEL** — a regra de redução, no único sítio onde ela é observável.
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Trocar *"fica o mais
/// próximo"* por *"fica o último"* passava nos quatro gates de cima, porque
/// sobre uma FENDA os dois pés de fora acham chão à MESMA distância — qualquer
/// regra de desempate dá o mesmo número. A propriedade só é observável sobre
/// chão DESIGUAL, e é essa a fixture: um degrau sob o pé ESQUERDO.
///
/// ⚠️ **O lado é escolhido, não arbitrário:** o `wall_offsets` põe o pé negativo
/// no índice 1 e o positivo no 2, então um *"fica o último"* escolheria o pé
/// direito — o do chão BAIXO — e o personagem afundaria no degrau em vez de o
/// subir. Pôr o degrau à direita deixaria as duas regras concordar.
///
/// ⚠️ **E o degrau de 0,15 m é CAMINHÁVEL, o que é metade da lei:** sobre um pé
/// afastado 0,2 m ele é uma rampa de 36,9°, dentro do `max_slope` de 45°. A
/// outra metade — *um degrau mais íngreme que o limite NÃO é chão* — **não tem
/// fixture de unidade, e a ausência é honesta:** um degrau de 0,6 m ao lado do
/// corpo é uma parede em que a cápsula não cabe, então o único jeito de a
/// encostar é o solver a parar contra ela. Quem segura essa metade é o
/// `measure_push_spin`, que foi onde ela foi DESCOBERTA: sem o limite, um
/// personagem que empurrava um caixote de 0,6 m passa a SUBIR nele e o
/// deslocamento do caixote cai de 7,27 para −0,02 m.
#[test]
fn the_leg_rides_the_highest_walkable_ground_any_foot_reaches() {
    let step_top = 0.15_f32;
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
    // Chao baixo: topo em y = 0. Degrau a ESQUERDA: topo em y = step_top.
    slab("Low", Vec2::new(5.0, -0.5), [15.0, 0.5]);
    slab("Step", Vec2::new(-10.0, step_top - 0.5), [10.0, 0.5]);

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
                    radius: HALF_W,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                ..PlatformPlayer::default()
            },
            // Centro exatamente sobre a costura: o pe do meio na aresta, o
            // esquerdo sobre o degrau, o direito sobre o chao baixo.
            Transform::from_translation(Vec2::new(0.0, FLOAT + step_top)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    for i in 1..=180u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
    }
    let y = sim
        .world()
        .get::<Transform>(player)
        .expect("transform")
        .translation
        .y;
    // Ele assenta a `float_height` acima do topo do DEGRAU, nao do chao baixo.
    let over_step = y - step_top;
    assert!(
        (over_step - FLOAT).abs() < 0.05,
        "a perna tem de cavalgar o chao MAIS ALTO que um pe alcanca \
         (esperado ~{FLOAT:.2} m acima do degrau, medido {over_step:.3} m; \
         y = {y:.3}, um pe no degrau de {step_top:.2} m e outro no chao baixo)"
    );
}
