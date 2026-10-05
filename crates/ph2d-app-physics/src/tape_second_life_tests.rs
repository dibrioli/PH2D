//! ⭐⭐ (report do dono, 05/10) **Na 2.ª vida o herói não repete a 1.ª.** Na arena (`PH2D_VIDA_SMOKE=4`)
//! o herói «rodava e andava sozinho às vezes» depois de um recomeço. Pela porta do PRODUTO (a mesma
//! `dispatch` que a shell chama), com o relógio a voltar ao zero como a shell o volta no recomeço.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, PhysicsBridge, PlayerInput, RigidBody,
    TopDownPlayer,
};
use ph2d_timeline::TimelineDoc;
use ph2d_topdown::{TopDownLaw, direction::DirectionMode, rotation::RotationMode};

use crate::bridge::dispatch::dispatch;

const DT: f64 = 1.0 / 60.0;

/// O herói da arena: cinemático, de vista de cima, livre, a virar-se para onde anda.
fn heroi() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::Free,
                rotation: RotationMode::ToMovement,
                ..TopDownLaw::default()
            }),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    (sim, e)
}

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("o herói");
    (t.translation.x, t.translation.y, t.rotation)
}

/// A 1.ª vida a andar para a direita e para cima, o recomeço, e a 2.ª vida SEM tecla — com quadros
/// que devem dois tiques (`soluco`) ou só um. Devolve as poses da 2.ª vida.
fn duas_vidas(soluco: bool, esquece: bool) -> Vec<(f32, f32, f32)> {
    let (mut sim, e) = heroi();
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    let mut tape = InputTape::new();
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    playhead.play();
    let anda = PlayerInput {
        drive: 1.0,
        drive_y: 0.6,
        ..PlayerInput::default()
    };
    for _ in 0..120 {
        playhead.advance();
        dispatch(
            &mut bridge,
            &mut sim,
            &playhead,
            DT,
            &mut doc,
            true,
            anda,
            &mut tape,
            &mut drive,
        );
    }
    // O recomeço: o relógio volta ao zero e a corrida CONTINUA a jogar (`fase_fabrica_e_morte`).
    playhead.rewind();
    if esquece {
        tape.clear();
    }
    let mut vida = Vec::new();
    for f in 0..90 {
        playhead.advance();
        if soluco && f % 3 == 2 {
            playhead.advance();
        }
        dispatch(
            &mut bridge,
            &mut sim,
            &playhead,
            DT,
            &mut doc,
            true,
            PlayerInput::default(),
            &mut tape,
            &mut drive,
        );
        vida.push(pose(&sim, e));
    }
    vida
}

#[test]
fn na_segunda_vida_sem_tecla_o_heroi_nao_anda_nem_roda() {
    // A fixtura contém o fenómeno: a 1.ª vida andou.
    let parado = |v: &[(f32, f32, f32)]| v.windows(2).all(|w| w[0] == w[1]);
    let com_soluco = duas_vidas(true, false);
    assert!(
        parado(&com_soluco[1..]),
        "a 2.ª vida, sem tecla e com quadros que devem dois tiques, mexeu: {:?}",
        com_soluco.windows(2).find(|w| w[0] != w[1])
    );
    // CONTROLOS: um tique por quadro, e a fita esquecida no recomeço.
    assert!(
        parado(&duas_vidas(false, false)[1..]),
        "um tique por quadro"
    );
    assert!(parado(&duas_vidas(true, true)[1..]), "a fita esquecida");
}
