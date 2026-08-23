//! **O que a FITA grava hoje** (W17, a medição que abre a wave).
//!
//! `cargo test -p ph2d-host-desktop measure_the_tape -- --ignored --nocapture`
//!
//! ⚠️ **Sonda, não gate.** Ela existe porque *persistir a fita* só faz sentido se
//! o que ela contém for uma CORRIDA — e a pergunta de antes de qualquer linha é
//! *o que ela contém quando ninguém correu?*

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};
use ph2d_timeline::TimelineDoc;

use super::physics_bridge::dispatch;

const DT: f64 = 1.0 / 60.0;

/// Uma cena com um corpo comum — e NENHUM player.
fn scene_without_player() -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 4.0)),
    ));
    sim
}

/// A mesma cena, mas com um personagem.
fn scene_with_player() -> (SimWorld, Entity) {
    let mut sim = scene_without_player();
    let p = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.4,
                    radius: 0.3,
                },
                ..Collider::default()
            },
            PlatformPlayer::default(),
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    (sim, p)
}

/// Roda `frames` frames pela porta do produto e devolve o tamanho da fita.
fn run(sim: &mut SimWorld, simulate: bool, frames: u64, held: PlayerInput) -> InputTape {
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    let mut tape = InputTape::new();
    playhead.play();
    for _ in 0..frames {
        playhead.advance();
        dispatch(
            &mut bridge,
            sim,
            &playhead,
            DT,
            &mut doc,
            simulate,
            held,
            &mut tape,
            &mut crate::preview_drive::PreviewDrive::default(),
        );
    }
    tape
}

/// **Quem entra na fita** — as quatro células que decidem o desenho da wave.
#[test]
#[ignore]
fn measure_the_tape_records() {
    let walk = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };

    let mut a = scene_without_player();
    let no_player_armed = run(&mut a, true, 120, walk).len();

    let mut b = scene_without_player();
    let no_player_held = run(&mut b, false, 120, PlayerInput::default()).len();

    let (mut c, _) = scene_with_player();
    let player_armed = run(&mut c, true, 120, walk).len();

    let (mut d, _) = scene_with_player();
    let player_held = run(&mut d, false, 120, PlayerInput::default()).len();

    eprintln!("== o que a fita grava em 120 frames ==");
    eprintln!("  sem player, Physics ARMADO ... {no_player_armed:4} tiques");
    eprintln!("  sem player, Physics  OFF   ... {no_player_held:4} tiques");
    eprintln!("  com player, Physics ARMADO ... {player_armed:4} tiques");
    eprintln!("  com player, Physics  OFF   ... {player_held:4} tiques");
    eprintln!("  (o default do toggle Physics e' OFF)");
}

/// **Quanto pesa uma corrida** — o número que decide se ela cabe num arquivo.
#[test]
#[ignore]
fn measure_what_a_run_weighs() {
    let (mut sim, _) = scene_with_player();
    let secs = [1.0_f64, 10.0, 60.0];
    eprintln!("== o peso de uma corrida ==");
    for s in secs {
        let frames = (s / DT).round() as u64;
        let tape = run(
            &mut sim,
            true,
            frames,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        // O tamanho em memória do que a fita guarda, do jeito que ela guarda.
        let bytes = tape.len() * core::mem::size_of::<PlayerInput>();
        eprintln!(
            "  {s:5.1} s = {:5} tiques = {:7} B ({:.1} kB)",
            tape.len(),
            bytes,
            bytes as f64 / 1024.0
        );
    }
}
