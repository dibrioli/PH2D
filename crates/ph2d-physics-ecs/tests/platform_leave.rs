//! **O QUE A PLATAFORMA DÁ AO PULO** (`W-Leave`) — os gates de comportamento,
//! com o rapier de verdade.
//!
//! A sonda irmã (`measure_platform_leave`) mediu o buraco: pular de um elevador
//! a **descer** a 4 m/s dá um pico de **0,378 m** (Spring) e **0,016** (Snap)
//! contra os ~1,87 de um chão parado — o artista autora dois metros e recebe um
//! centímetro e meio. A causa não é o solver nem divergência entre modos: é que
//! a altura autorada é medida **contra a plataforma** (o `ADD_VELOCITY` do
//! Godot), e não havia política.
//!
//! # ⚠️ O oráculo é o CONTROLE, não um número maior
//!
//! Cada gate compara contra a **mesma cena com a plataforma parada**. A política
//! existe para entregar em MUNDO a altura que o artista escreveu, então
//! *"funciona"* significa **igualar o controle** — um pico maior seria outro
//! defeito, e é exatamente o que a coluna `Full` do elevador que SOBE mostra.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformLift, PlatformPlayer,
    PlayerInput, PlayerMode, RigidBody,
};
use scene_fixture::FLOAT_HEIGHT;

/// A velocidade da plataforma — o número que o plano nomeia.
const SPEED: f32 = 4.0;
/// ⚠️ **LONGA de propósito** — a lição da `measure_kinematic_carry`: uma
/// plataforma curta deixa o personagem sair de cima dela e a coluna passa a
/// medir um deslize sem chão.
const HALF: f32 = 30.0;
/// Um segundo a andar nela antes de largar (a tração assenta numa fração disto).
const RIDE: u64 = 60;
/// **A janela do voo, FIXA** — medir *"até aterrar"* daria a cada linha uma
/// duração própria, e um pico maior poderia ser apenas mais tempo no ar.
const FLIGHT: u64 = 30;

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
                    half_x: HALF,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(ph2d_core::Vec2::new(0.0, 0.0)),
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
            Transform::from_translation(ph2d_core::Vec2::new(0.0, 0.25 + FLOAT_HEIGHT)),
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

fn pose_y(sim: &SimWorld, who: Entity) -> f32 {
    sim.world()
        .get::<Transform>(who)
        .map(|t| t.translation.y)
        .expect("o personagem tem de existir")
}

/// Assenta, anda com a plataforma, **pula**, e devolve o pico acima do ponto de
/// partida na janela do voo.
///
/// ⚠️ O botão fica premido durante o voo (pulo cheio, determinístico) — seguro
/// porque `air_jumps` nasce em **0**.
fn peak(mode: Option<PlayerMode>, axis: [f32; 2], lift: PlatformLift) -> f32 {
    let (mut sim, mut bridge, wagon, who) = scene(mode, lift);
    for t in 1..=60u64 {
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }
    let step = [axis[0] * SPEED / 60.0, axis[1] * SPEED / 60.0];
    let nudge = |sim: &mut SimWorld| {
        let mut e = sim.world_mut().entity_mut(wagon);
        if let Some(mut tr) = e.get_mut::<Transform>() {
            tr.translation.x += step[0];
            tr.translation.y += step[1];
        }
    };
    for t in 61..=(60 + RIDE) {
        nudge(&mut sim);
        bridge.set_player_input(who, PlayerInput::default());
        bridge.dispatch(&mut sim, true, t);
    }
    let y0 = pose_y(&sim, who);
    let mut best = 0.0f32;
    for t in (61 + RIDE)..=(60 + RIDE + FLIGHT) {
        nudge(&mut sim);
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        best = best.max(pose_y(&sim, who) - y0);
    }
    best
}

const MODES: [(Option<PlayerMode>, &str); 3] = [
    (None, "Spring"),
    (Some(PlayerMode::Kinematic), "Snap"),
    (Some(PlayerMode::Pure), "Pure"),
];

const STILL: [f32; 2] = [0.0, 0.0];
const DOWN: [f32; 2] = [0.0, -1.0];
const UP: [f32; 2] = [0.0, 1.0];
const SIDEWAYS: [f32; 2] = [1.0, 0.0];

/// **O DEFEITO, e o número dele** — este gate nasceu vermelho antes da política
/// existir, e continua a descrever o que a variante `Full` faz.
///
/// ⚠️ **Ele não é um bug pinado: `Full` é uma escolha legítima** (é o default do
/// Godot e o mundo que já shipava). O que o gate impede é a política ser
/// silenciosamente reescrita — se alguém *"consertar"* o `Full`, isto sangra e
/// diz que o consertou no lugar errado.
#[test]
fn full_measures_the_authored_height_against_the_platform() {
    for (mode, tag) in MODES {
        let control = peak(mode, STILL, PlatformLift::Full);
        let falling = peak(mode, DOWN, PlatformLift::Full);
        let rising = peak(mode, UP, PlatformLift::Full);
        assert!(
            falling < control * 0.25,
            "{tag}: sob Full um elevador a DESCER quase anula o pulo \
             (controle {control:.4}, a descer {falling:.4})"
        );
        assert!(
            rising > control * 1.8,
            "{tag}: sob Full um elevador a SUBIR da' um super-pulo \
             (controle {control:.4}, a subir {rising:.4})"
        );
    }
}

/// **A ENTREGA: com a política, a altura autorada chega em MUNDO.**
///
/// ⚠️ **A barra é o CONTROLE**, e a folga de 12% tem causa nomeada: no tique do
/// pulo o personagem ainda está `grounded`, então o `ground_carry` ainda paga a
/// componente normal da plataforma (a descer, 4 m/s × dt) por um ou dois tiques
/// antes de o pé sair. É um erro de fase de dois tiques, não da lei — medido,
/// Spring **1,9033 contra 1,9033** (exato) e Snap **1,7316 contra 1,8650**.
#[test]
fn the_policy_delivers_the_authored_height_in_the_world() {
    for (mode, tag) in MODES {
        let control = peak(mode, STILL, PlatformLift::UpOnly);
        for lift in [PlatformLift::UpOnly, PlatformLift::Nothing] {
            let falling = peak(mode, DOWN, lift);
            assert!(
                falling > control * 0.88,
                "{tag}/{lift:?}: largar um elevador a DESCER tem de entregar a altura \
                 autorada (controle {control:.4}, a descer {falling:.4})"
            );
        }
    }
}

/// **AS TRÊS SÃO DISTINGUÍVEIS — nenhuma variante morta.**
///
/// O elevador que SOBE é o único lugar onde `UpOnly` e `Nothing` divergem, e é
/// isso que justifica haver três em vez de duas: uma mantém o impulso de subida,
/// a outra o descarta.
#[test]
fn each_policy_is_reachable_and_none_is_a_duplicate() {
    for (mode, tag) in MODES {
        let control = peak(mode, STILL, PlatformLift::Full);
        let full = peak(mode, UP, PlatformLift::Full);
        let up_only = peak(mode, UP, PlatformLift::UpOnly);
        let nothing = peak(mode, UP, PlatformLift::Nothing);
        assert!(
            (full - up_only).abs() < 0.01,
            "{tag}: a subir, Full e UpOnly sao a MESMA politica por construcao \
             ({full:.4} contra {up_only:.4})"
        );
        assert!(
            nothing < full * 0.7,
            "{tag}: Nothing tem de DESCARTAR o impulso de subida \
             ({nothing:.4} contra {full:.4})"
        );
        assert!(
            nothing > control * 0.88,
            "{tag}: ...e entregar a altura autorada ({nothing:.4} contra {control:.4})"
        );
    }
}

/// **A política não toca a HORIZONTAL, e isso é GEOMETRIA.**
///
/// O `delta` do pulo é ao longo do `up`, então a velocidade de um vagão nunca
/// passa por aqui — ela vive na velocidade que o corpo possui e no referencial
/// que o `lift_momentum` segura. Um gate para ninguém *"completar"* a política
/// alcançando o eixo errado.
#[test]
fn the_policy_leaves_the_horizontal_alone() {
    for (mode, tag) in MODES {
        let full = peak(mode, SIDEWAYS, PlatformLift::Full);
        for lift in [PlatformLift::UpOnly, PlatformLift::Nothing] {
            let got = peak(mode, SIDEWAYS, lift);
            assert!(
                (full - got).abs() < 1e-4,
                "{tag}/{lift:?}: um VAGAO horizontal nao pode sentir a politica \
                 ({full:.4} contra {got:.4})"
            );
        }
    }
}

/// **EM CHÃO ESTÁTICO AS TRÊS SÃO A MESMA, ao bit** — é isto que torna o default
/// `Full` inerte para todo projeto já salvo que não tenha plataforma móvel.
#[test]
fn on_still_ground_the_three_policies_are_bit_identical() {
    for (mode, tag) in MODES {
        let full = peak(mode, STILL, PlatformLift::Full);
        for lift in [PlatformLift::UpOnly, PlatformLift::Nothing] {
            let got = peak(mode, STILL, lift);
            assert_eq!(
                full.to_bits(),
                got.to_bits(),
                "{tag}/{lift:?}: com o chao PARADO as tres politicas tem de ser a \
                 mesma, ao bit ({full} contra {got})"
            );
        }
    }
}

/// **O componente ATRAVESSA até a lei** — o fold, e o tag como porta única.
#[test]
fn the_component_carries_the_policy_to_the_law() {
    for lift in [
        PlatformLift::Full,
        PlatformLift::UpOnly,
        PlatformLift::Nothing,
    ] {
        let p = PlatformPlayer {
            platform_lift: lift,
            ..PlatformPlayer::default()
        };
        assert_eq!(p.config().jump.platform_lift, lift.law());
        // A ida e a volta pela fronteira da UI.
        assert_eq!(PlatformLift::from_tag(lift.tag()), Some(lift));
    }
    // ⚠️ Um tag que nenhuma variante reivindica devolve `None`, e não um
    // plausível — a disciplina do `BodyKind::from_tag`.
    assert_eq!(PlatformLift::from_tag(3), None);
    assert_eq!(PlatformLift::default(), PlatformLift::Full);
}
