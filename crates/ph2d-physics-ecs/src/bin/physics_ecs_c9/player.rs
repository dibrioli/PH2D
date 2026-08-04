//! **A lane do PLAYER DE PLATAFORMA** (W7) — irmão próprio pelo cap de 700 LOC,
//! e cortado por assunto pela mesma razão dos outros três: uma zona é um corpo
//! com um flag, um rig é uma montagem, um joint é um vínculo — e um player é a
//! única lane cujo estado depende de um **fluxo de entrada por tique**.
//!
//! # ⚠️ Por que ele TEM de estar aqui
//!
//! O `physics_ecs_c9` existe para provar que o **NOSSO** código é bit-idêntico
//! nos três OSes — a ordem de iteração, a fronteira metros↔rapier, o readback.
//! O controlador de player acrescenta caminhos que nenhuma outra lane exercita:
//! um **ray cast** contra o BVH (a perna), uma **escrita direta de velocidade**
//! (o amortecedor e a caminhada), um **impulso resistido pela massa** (a
//! decolagem) e a **reação da 3ª lei** de volta ao chão. Fora do hash, nada
//! provaria que eles atravessam a fronteira igual em toda plataforma.
//!
//! # ⚠️ A fita é ROTEIRIZADA, nunca gravada
//!
//! Uma fita saída de um teclado descreve uma corrida que ninguém consegue
//! repetir — e um harness de determinismo cuja ENTRADA não é reproduzível não
//! mede nada. Aqui ela é função pura do tique.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, LockRotation, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// O chão do player, longe das outras lanes (o piso principal do harness vai só
/// até x = 50) — uma lane que empurra outra mediria as duas.
const GROUND_X: f32 = 90.0;

/// **A corrida roteirizada.** Anda para a direita, pula no meio, e para no fim —
/// os três regimes da lei (caminhada, decolagem+arco, freio) num run de 120
/// tiques.
///
/// ⚠️ **Função pura do tique**: nenhum relógio de parede, nenhum RNG, nenhuma
/// entrada de dispositivo. É o que a torna parte do harness em vez de ruído
/// dentro dele.
pub fn tape(ticks: u64) -> InputTape {
    let mut t = InputTape::new();
    for k in 1..=ticks {
        t.record(
            k,
            PlayerInput {
                drive: if k < 90 { 1.0 } else { 0.0 },
                jump: (40..48).contains(&k),
            },
        );
    }
    t
}

/// O player e o chão dele.
pub fn spawn(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("C9 Player Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 10.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(GROUND_X, 0.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Player"),
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
        // ⚠️ Um personagem que TOMBA não é um personagem — a mesma razão pela
        // qual todo platformer 2D trava o DOF angular.
        LockRotation,
        PlatformPlayer::default(),
        Transform::from_translation(Vec2::new(GROUND_X - 8.0, 1.5)),
    ));
}
