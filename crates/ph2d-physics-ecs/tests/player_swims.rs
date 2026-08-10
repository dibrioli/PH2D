//! **O PERSONAGEM NADA** — os gates da W-Swim, na água de verdade.
//!
//! O `player_in_water.rs` ao lado pergunta *o que a água FAZ ao personagem*
//! (empuxo, arrasto, o bobeio); este pergunta *o que o personagem faz na água*.
//!
//! # ⚠️ O oráculo é DIRECIONAL, e é deliberado
//!
//! Um literal de altura seria um espelho da aritmética do servo: ele passaria a
//! afirmar *"o nado empurra exatamente 1,84 m em três segundos"*, que é uma
//! frase sobre a `swim_speed` da fixture e não sobre a lei. O que estes gates
//! afirmam é o que o jogador vê — *ele SOBE quando eu peço para subir, DESCE
//! quando peço para descer, e sem eu pedir nada não faz nem uma coisa nem
//! outra* —, sempre contra o MESMO tique da capacidade desligada.
//!
//! ⚠️ **E a paridade entre modos entra em cada um**, como no irmão: a espécie do
//! corpo não pode ser uma pergunta que a água faça.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test player_swims`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

/// A cápsula e a poça das fixtures do player — os MESMOS números do
/// `player_in_water.rs` e do `measure_the_swim_threshold`, para os três falarem
/// da mesma cena.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const FLUID: f32 = 4.0;
const DRAG: f32 = 0.6;

/// Um nado ligado. ⚠️ A capacidade nasce DESLIGADA, então toda fixture que a
/// queira tem de a ligar — e é isso que faz o `off` destes gates ser o produto
/// de ontem, e não um caso especial.
const SWIM_SPEED: f32 = 4.0;

/// A autoridade do servo — o ponto de partida da lei.
const SWIM_ACCEL: f32 = 12.0;

/// Onde o sujeito é largado: **já submerso**, para o gate medir o REGIME e não a
/// entrada na água (o transiente que o irmão mede).
const START: f32 = -1.0;

/// A poça funda: superfície em `y = 0`, nada ao alcance do sensor de chão.
fn pool(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 6.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        AreaDrag(DRAG),
        Transform::from_translation(Vec2::new(0.0, -6.0)),
    ));
}

/// Um chão sólido, para o gate de vadear.
fn floor(sim: &mut SimWorld, y: f32) {
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, y - 0.5)),
    ));
}

fn player(sim: &mut SimWorld, kinematic: bool, swim: f32, accel: f32, start: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            swim_speed: swim,
            swim_acceleration: accel,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, start)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn pos_of(sim: &SimWorld) -> Vec2 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation;
        }
    }
    panic!("o sujeito tem de existir");
}

/// Seis segundos de nado numa poça funda, com a entrada que se pedir.
///
/// Devolve `(x percorrido, altura MÉDIA da segunda metade)` — as duas grandezas
/// que o jogador julga.
///
/// ⚠️ **A vertical é uma MÉDIA, e a primeira versão deste harness lia um
/// instante.** O personagem na água OSCILA (o `1,44 m` que o `player_in_water`
/// mede e o plano 07 §8.4 explica), então uma amostra em `t = 180` compara a
/// FASE de duas oscilações: medido, *segurar para baixo* lia `1,191` contra
/// `1,136` de *não pedir nada* — a ordem invertida, por 5%, sobre leis que na
/// saturação produzem o MESMO motor. É a mesma lição que o gate irmão pagou
/// (*"uma amostra única de um sistema que oscila não é um repouso"*), e ela
/// reincide sempre que alguém escreve um harness novo.
fn swims(kinematic: bool, swim: f32, input: PlayerInput) -> Vec2 {
    swims_with(kinematic, swim, SWIM_ACCEL, input)
}

/// Como a [`swims`], com a AUTORIDADE do servo escolhida.
fn swims_with(kinematic: bool, swim: f32, accel: f32, input: PlayerInput) -> Vec2 {
    let mut sim = SimWorld::new();
    pool(&mut sim);
    let who = player(&mut sim, kinematic, swim, accel, START);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, input);
    let (mut sum, mut n) = (0.0f64, 0u32);
    let mut last = Vec2::new(0.0, 0.0);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        last = pos_of(&sim);
        if t > 180 {
            sum += f64::from(last.y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    Vec2::new(last.x, (sum / f64::from(n)) as f32 - START)
}

fn holding(jump: bool, down: bool, drive: f32) -> PlayerInput {
    PlayerInput {
        drive,
        jump,
        down,
        ..PlayerInput::default()
    }
}

/// **A CAPACIDADE DESLIGADA É O MUNDO DE ONTEM** — o controle de todos os
/// outros, e o gate que qualquer termo desta wave tem de deixar em paz.
///
/// ⚠️ Com o nado em zero o personagem faz o que sempre fez numa poça: sobe pelo
/// empuxo, e os botões não significam nada dentro d'água.
#[test]
fn the_capability_off_is_yesterdays_world() {
    for kinematic in [false, true] {
        let idle = swims(kinematic, 0.0, PlayerInput::default());
        // ⚠️ **Só o BOTÃO muda entre as duas chamadas.** A primeira versão deste
        // gate variava o `drive` junto e comparava a VERTICAL — e o `drive` move
        // o personagem 17 m de lado pelo controle aéreo, o que muda o arrasto e
        // portanto a altura. *Uma ablação que mexe em duas entradas não atribui
        // a nenhuma.*
        let pressing = swims(kinematic, 0.0, holding(true, true, 0.0));
        assert!(
            (idle.y - pressing.y).abs() < 1.0e-3,
            "desligado, os botoes nao movem a vertical ({kinematic}): {idle:?} vs {pressing:?}"
        );
        assert!(
            idle.y > 0.0,
            "e o empuxo continua a levantar ({kinematic}): {idle:?}"
        );
    }
}

/// **PEDIR PARA SUBIR SOBE, PEDIR PARA DESCER DESCE** — a wave inteira, num
/// oráculo que é o do jogador.
///
/// ⚠️ **O controle é a MESMA cena com o nado desligado**, e não um número
/// escrito à mão: o que este gate afirma é que o botão MOVE a resposta, em cada
/// modo, e não que ela vale tanto.
#[test]
fn up_rises_and_down_dives_in_both_modes() {
    for kinematic in [false, true] {
        let up = swims(kinematic, SWIM_SPEED, holding(true, false, 0.0)).y;
        let idle = swims(kinematic, SWIM_SPEED, PlayerInput::default()).y;
        let down = swims(kinematic, SWIM_SPEED, holding(false, true, 0.0)).y;
        // ⚠️ **A ordem, e não um sinal absoluto:** nesta poça o empuxo líquido
        // vale `g·(4−1) ≈ 29,4 m/s²` e a autoridade default do servo é `12`,
        // então **pedir para descer ainda SOBE** — só mais devagar. Quem afirma
        // que dá para mergulhar de facto é o gate seguinte, e é ele que carrega
        // a relação medida.
        assert!(
            down < idle && idle < up,
            "{down} < {idle} < {up} ({kinematic})"
        );
    }
}

/// **O eixo horizontal responde ao `drive`**, e nos dois sentidos.
#[test]
fn the_stroke_carries_him_sideways() {
    for kinematic in [false, true] {
        let right = swims(kinematic, SWIM_SPEED, holding(false, false, 1.0)).x;
        let left = swims(kinematic, SWIM_SPEED, holding(false, false, -1.0)).x;
        assert!(right > 1.0, "nada para a direita ({kinematic}): {right}");
        assert!(left < -1.0, "e para a esquerda ({kinematic}): {left}");
        assert!(
            (right + left).abs() < 0.1 * right,
            "e os dois lados sao simetricos ({kinematic}): {right} vs {left}"
        );
    }
}

/// ⚠️ **ATRAVESSAR UMA POÇA RASA É ANDAR** — o limiar existe para isto, e o
/// preço de não o ter seria o personagem a parar de caminhar ao molhar os pés.
///
/// ⚠️ **A primeira versão deste gate montou uma cena IMPOSSÍVEL:** ela punha o
/// personagem de pé no fundo de uma poça funda e exigia que ele lá ficasse — mas
/// nesta fixture o fluido é **quatro vezes mais denso** que ele, então o empuxo
/// líquido aponta para cima com três pesos e **ninguém consegue ficar de pé lá**
/// (medido: ele subia a `+0,24` depois de andar 7,3 m). *A física estava certa e
/// a fixture é que descrevia um mundo que não existe.*
///
/// A poça que se atravessa a pé é rasa por definição: aqui o chão deixa o
/// personagem **20% submerso**, que a tabela do `measure_the_swim_threshold` lê
/// como `buoyed = 0,68` — abaixo do limiar default de `1,0`.
#[test]
fn crossing_a_shallow_puddle_is_walking() {
    for kinematic in [false, true] {
        let mut sim = SimWorld::new();
        // A poça: superfície em `y = 0`.
        sim.world_mut().spawn((
            Name::new("Puddle"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                is_sensor: true,
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 2.0,
                },
                ..Collider::default()
            },
            AreaBuoyancy(FLUID),
            AreaDrag(DRAG),
            Transform::from_translation(Vec2::new(0.0, -2.0)),
        ));
        // O chão a `-0,6`: o personagem paira `FLOAT` acima dele, ou seja em
        // `+0,3` — os 20% submersos da tabela.
        floor(&mut sim, -0.6);
        let who = player(&mut sim, kinematic, SWIM_SPEED, SWIM_ACCEL, 0.3);
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(who, holding(false, false, 1.0));
        for t in 1..=120u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let p = pos_of(&sim);
        assert!(
            p.x > 2.0,
            "ele tem de ATRAVESSAR a poca ({kinematic}): {p:?}"
        );
        // E continua com os pés no chão — a caminhada nunca virou braçada.
        assert!(
            (p.y - 0.3).abs() < 0.15,
            "e a pe', na altura de flutuacao ({kinematic}): {p:?}"
        );
    }
}

/// **O EMPUXO continua a ser quem o segura** — o nado não cancela a gravidade,
/// então com o dedo parado o personagem sobe **na mesma direção** que subiria
/// sem a capacidade.
///
/// ⚠️ Ele sobe MENOS: o servo freia para o alvo zero, e é isso que um nadador
/// parado faz. O que este gate recusa é a inversão — um nado que cancelasse a
/// gravidade deixaria o corpo pendurado onde estivesse, e um que a pagasse duas
/// vezes o atiraria para fora da poça.
#[test]
fn an_idle_swimmer_still_answers_to_the_water() {
    for kinematic in [false, true] {
        let off = swims(kinematic, 0.0, PlayerInput::default()).y;
        let idle = swims(kinematic, SWIM_SPEED, PlayerInput::default()).y;
        assert!(
            idle > 0.0,
            "parado, o empuxo ainda o levanta ({kinematic}): {idle}"
        );
        assert!(
            idle < off,
            "mas menos do que sem o freio da bracada ({kinematic}): {idle} vs {off}"
        );
    }
}

/// ⚠️ **MERGULHAR PEDE MAIS AUTORIDADE DO QUE A ÁGUA TEM** — o achado desta
/// wave, e a razão de a `swim_acceleration` ser um número que o artista mexe.
///
/// A doc do campo diz que ele é *autoridade contra o empuxo*; este gate torna a
/// frase um NÚMERO. Numa poça `d` vezes mais densa que o corpo, o empuxo líquido
/// sobre um corpo submerso vale `|g|·(d − 1)` — aqui `9,81 · 3 ≈ 29,4 m/s²` —,
/// então um servo de `12` **não consegue descer**: ele apenas sobe mais devagar.
///
/// ⚠️ **Isto não é um defeito, é a física da cena**, e o gate existe para que
/// ninguém o "conserte" capando o empuxo: uma rolha não mergulha por querer. O
/// que a lei promete é que **subir a autoridade acima do empuxo devolve o
/// mergulho**, e é isso que a segunda metade afirma.
#[test]
fn diving_needs_more_authority_than_the_water_has() {
    // O empuxo líquido que o servo tem de vencer, submerso — derivado da cena,
    // nunca de um literal: `|g| · (densidade_fluido/densidade_corpo − 1)`.
    let net_lift = 9.81 * (FLUID - 1.0);
    for kinematic in [false, true] {
        let weak = swims_with(kinematic, SWIM_SPEED, SWIM_ACCEL, holding(false, true, 0.0)).y;
        let strong = swims_with(
            kinematic,
            SWIM_SPEED,
            net_lift * 1.5,
            holding(false, true, 0.0),
        )
        .y;
        assert!(
            weak > 0.0,
            "com autoridade abaixo do empuxo ele ainda SOBE ({kinematic}): {weak}"
        );
        assert!(
            strong < 0.0,
            "acima do empuxo, o mergulho existe ({kinematic}): {strong}"
        );
    }
}
