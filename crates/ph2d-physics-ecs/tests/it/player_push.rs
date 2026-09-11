//! **O EMPURRÃO do modo cinemático** (W-KinPush) — os gates de comportamento.
//!
//! ⚠️ **O CONTROLE é o corpo DINÂMICO**, e ele não é decoração: sob Spring o
//! solver já empurra o que o personagem esbarra, então uma coluna que empurra é
//! o que separa *"a lei funciona"* de *"a cena tem um caixote leve"*. Todo gate
//! daqui que afirma um empurrão traz ao lado ou o dinâmico ou o `push = 0`.
//!
//! Os números que estes bares usam saem de `measure_kinematic_push.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, MassOverride, PhysicsBridge, PlatformPlayer,
    PlayerMode, RigidBody,
};
use ph2d_platformer::PlayerInput;

const FLOAT_HEIGHT: f32 = 0.9;

/// Um chão plano, um CAIXOTE solto à direita e o personagem à esquerda dele.
fn scene(crate_density: f32, kinematic: bool, push: f32) -> (SimWorld, PhysicsBridge, Entity) {
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
                reaction_push: push,
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

/// Anda para a direita por 3 s e devolve quanto o CAIXOTE andou.
fn crate_travel(crate_density: f32, kinematic: bool, push: f32) -> f32 {
    let (mut sim, mut bridge, who) = scene(crate_density, kinematic, push);
    // ⚠️ Assenta antes de medir: o primeiríssimo tique de uma cena tem o BVH
    // vazio e o personagem cai um tique antes de a perna o pegar.
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let c0 = x_of(&sim, "Crate");
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 61..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    x_of(&sim, "Crate") - c0
}

/// **O personagem cinemático EMPURRA o que está no caminho**, e o corpo
/// dinâmico é o controle.
///
/// ⚠️ **`push = 0` é a segunda metade do gate**, e é ela que prova que a fixture
/// contém o fenômeno: sem o canal, um corpo cinemático tem massa infinita para o
/// solver e o caixote não anda um milímetro.
#[test]
fn a_walking_player_shoves_a_loose_crate() {
    let dynamic = crate_travel(1.0, false, 1.0);
    let off = crate_travel(1.0, true, 0.0);
    let on = crate_travel(1.0, true, 1.0);
    assert!(
        off.abs() < 0.01,
        "sem o canal o cinematico e' um fantasma de lado: {off:.4} m"
    );
    assert!(
        dynamic > 5.0,
        "a fixture tem de CONTER o fenomeno: o dinamico levou {dynamic:.4} m"
    );
    assert!(
        on > 0.8 * dynamic,
        "com o canal ele empurra como o dinamico: {on:.4} contra {dynamic:.4}"
    );
}

/// **O escalar é um DIAL, não um interruptor** — meio empurra menos que inteiro.
///
/// ⚠️ **Este gate nasceu de uma mutação que sobreviveu a todos os outros:**
/// apagar o `s` da multiplicação em `push_transfer` deixa a suíte inteira verde,
/// porque o early-out em `push == 0` continua lá — o canal ainda LIGA e DESLIGA,
/// e nenhum outro gate autora um valor no meio. Um knob que só tem dois estados
/// é um checkbox com cara de slider.
///
/// ⚠️ **O caixote é PESADO de propósito:** um leve é limitado pela VELOCIDADE do
/// personagem (ele foge até deixar de bloquear, e a partir daí mais impulso não
/// o leva mais longe — a mesma propriedade auto-limitada que impede a
/// ressonância), então metade do empurrão daria quase o mesmo número.
///
/// ⚠️ **E a resposta é MUITO não-linear aqui, de propósito na leitura:** medido,
/// meia força leva o caixote pesado a `0,038 m` e a força inteira a mais de um
/// metro. Não é a lei que é não-linear — é o ATRITO ESTÁTICO do caixote, que
/// meia força mal vence. O gate afirma a ordem, não a razão.
#[test]
fn the_push_scale_is_a_dial_not_a_switch() {
    let half = crate_travel(16.0, true, 0.5);
    let full = crate_travel(16.0, true, 1.0);
    assert!(
        half > 0.01,
        "meia forca ainda empurra: {half:.4} (senao isto vira o gate do zero)"
    );
    assert!(
        full > half * 1.3,
        "e inteiro empurra mais que meio: {full:.4} contra {half:.4}"
    );
}

/// **A MASSA do caixote manda** — o leve anda mais que o pesado sob o MESMO
/// personagem.
#[test]
fn the_crates_mass_decides_how_far_it_goes() {
    let light = crate_travel(1.0, true, 1.0);
    let heavy = crate_travel(16.0, true, 1.0);
    assert!(
        light > heavy + 1.0,
        "o leve tem de andar mais: leve {light:.4} pesado {heavy:.4}"
    );
    assert!(
        heavy > 0.1,
        "e o pesado tem de andar ALGUMA coisa: {heavy:.4}"
    );
}

/// **A MASSA do personagem manda também** — o impulso é `m·Δv`, e o `m` é o
/// dele.
///
/// ⚠️ É este gate que torna a **W-KinWeight** viva de lado: a massa autorada
/// deixou de governar só o peso que o chão sente.
#[test]
fn the_authored_mass_of_the_player_is_what_pushes() {
    let plain = crate_travel(16.0, true, 1.0);
    let (mut sim, mut bridge, who) = scene(16.0, true, 1.0);
    sim.world_mut().entity_mut(who).insert(MassOverride(20.0));
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let c0 = x_of(&sim, "Crate");
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 61..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let heavy_player = x_of(&sim, "Crate") - c0;
    assert!(
        heavy_player > plain + 1.0,
        "um personagem pesado empurra mais longe: {heavy_player:.4} contra {plain:.4}"
    );
}

/// **Uma parede ESTÁTICA absorve tudo e não se mexe** — e o personagem PARA
/// nela, sem ganhar nada de volta.
///
/// ⚠️ O ledger fecha em zero por construção: `apply_impulse_at_point` do rapier
/// é no-op num corpo de massa infinita. Este gate existe para o dia em que
/// alguém "consertar" isso com um caso especial.
#[test]
fn a_static_wall_absorbs_the_push_and_does_not_move() {
    let (mut sim, mut bridge, who) = scene(1.0, true, 1.0);
    // Uma parede logo à frente, ANTES do caixote.
    let wall = sim
        .world_mut()
        .spawn((
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
            Transform::from_translation(Vec2::new(0.8, 1.0)),
        ))
        .id();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let w0 = sim.world().get::<Transform>(wall).unwrap().translation;
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 61..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let w1 = sim.world().get::<Transform>(wall).unwrap().translation;
    assert!(
        (w1.x - w0.x).abs() < 1.0e-4 && (w1.y - w0.y).abs() < 1.0e-4,
        "massa infinita nao se move: {w0:?} -> {w1:?}"
    );
    let px = x_of(&sim, "Player");
    assert!(
        px < 0.8,
        "e o personagem PARA na parede, sem atravessar: x={px:.4}"
    );
}

/// **O CHÃO não é empurrado duas vezes.**
///
/// ⚠️ A 3ª lei vertical (K6) já entrega o PESO ao corpo em que o personagem está
/// de pé; se o canal lateral também o entregasse, a jangada afundaria por dois
/// caminhos — a doença que este módulo mais encontra. Medido, os dois valores do
/// knob dão **exactamente** a mesma aceleração.
///
/// ⚠️ **As três linhas que fazem esta fixture funcionar:** a jangada não tem
/// peso próprio (senão o que se mede é a queda livre dela), tem a rotação
/// travada (senão um empurrão fora do centro a INCLINA e o `y` do centro deixa
/// de ser oráculo) e é LARGA (senão o personagem sai dela em meio segundo).
#[test]
fn the_push_does_not_shove_the_floor() {
    fn raft_drop(push: f32) -> f32 {
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
                        half_x: 30.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                ph2d_physics_ecs::GravityScale(0.0),
                LockRotation,
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        let who = sim
            .world_mut()
            .spawn((
                Name::new("Player"),
                RigidBody {
                    kind: BodyKind::Kinematic,
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
        sim.world_mut()
            .entity_mut(who)
            .insert(PlayerMode::Kinematic);
        let mut bridge = PhysicsBridge::new();
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let y0 = sim.world().get::<Transform>(raft).unwrap().translation.y;
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        y0 - sim.world().get::<Transform>(raft).unwrap().translation.y
    }
    let off = raft_drop(0.0);
    let on = raft_drop(1.0);
    assert!(
        off > 0.05,
        "a fixture tem de conter o PESO: a jangada afundou {off:.4} m"
    );
    assert!(
        (on - off).abs() < 0.01 * off,
        "o canal lateral nao pode tocar no vertical: sem {off:.4} com {on:.4}"
    );
}

/// **O empurrão não RESSOA** — o kill-criterion desta wave.
///
/// ⚠️ O risco nomeado no plano é estabilidade: empurra, o caixote foge, o slide
/// o segue, empurra outra vez. O caixote está **encostado numa parede**, que é o
/// caso duro (o que se empurra tem massa efetiva infinita e o contato nunca
/// acaba). Medido, a folga em regime é **constante** — a lei é auto-limitada por
/// construção, porque o que volta é o que foi BLOQUEADO: se o caixote foge,
/// menos é bloqueado, menos é empurrado.
#[test]
fn the_push_does_not_resonate_against_a_blocked_crate() {
    let (mut sim, mut bridge, who) = scene(1.0, true, 1.0);
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
    for t in 61..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for t in 181..=300u64 {
        bridge.dispatch(&mut sim, true, t);
        let gap = x_of(&sim, "Crate") - x_of(&sim, "Player");
        lo = lo.min(gap);
        hi = hi.max(gap);
    }
    assert!(
        lo.is_finite() && hi - lo < 0.01,
        "a folga em regime tem de ser estavel: {lo:.4}..{hi:.4}"
    );
}

/// **Um caixote deixado para trás PARA de ser empurrado.**
///
/// ⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU:** apagar o `hits.clear()`
/// do `move_character` deixou os SEIS gates acima verdes. O motivo é que a lei
/// recompõe o empurrão a partir do bloqueio do tique ATUAL, então uma lista que
/// acumula relatórios da mesma parede dá o mesmo número — a dedup colapsa-os.
/// O que ela **não** colapsa é um contato VELHO com um bloqueio NOVO: o
/// personagem larga o caixote, vai embater noutra coisa, e o bloqueio dessa
/// outra coisa é entregue ao caixote pela normal que ele tinha.
///
/// A cena é essa: empurra o caixote para a direita, vira-se e vai contra uma
/// parede à esquerda. O caixote não pode andar para trás — nada o puxa.
#[test]
fn a_crate_left_behind_stops_being_pushed() {
    let (mut sim, mut bridge, who) = scene(1.0, true, 1.0);
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
        Transform::from_translation(Vec2::new(-2.0, 1.0)),
    ));
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    // Empurra para a direita por um segundo.
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for t in 61..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let pushed = x_of(&sim, "Crate");
    assert!(
        pushed > 1.6,
        "a fixture tem de CONTER o fenomeno: o caixote foi empurrado ate' {pushed:.4}"
    );
    // Vira-se e vai contra a parede, longe do caixote.
    bridge.set_player_input(
        who,
        PlayerInput {
            drive: -1.0,
            ..PlayerInput::default()
        },
    );
    let mut lowest = f32::INFINITY;
    for t in 121..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        lowest = lowest.min(x_of(&sim, "Crate"));
    }
    assert!(
        lowest > pushed - 0.02,
        "nada puxa o caixote de volta: ele foi de {pushed:.4} a {lowest:.4}"
    );
}
