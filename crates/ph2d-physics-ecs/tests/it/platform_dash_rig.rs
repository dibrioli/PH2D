//! **A cena do ARRANQUE** (W14) — a fixture que o gate e a sonda partilham.
//!
//! ⚠️ **Incluída por `#[path]` nos dois arquivos, e não copiada**, pelo motivo
//! que o `platform_wall_rig` ao lado já escreve: cada arquivo em `tests/` é um
//! crate próprio, e uma segunda cópia da cena seria uma segunda resposta a
//! *"como é um personagem num chão comprido"* — as duas divergiriam no dia em
//! que a cápsula ou a altura de flutuação mudassem, com a que não foi
//! actualizada a seguir VERDE sobre outra premissa.
//!
//! ⚠️ O `allow(dead_code)` é do formato, não do descuido: cada consumidor usa o
//! subconjunto de que precisa.
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A altura de flutuação desta fixture.
pub const FLOAT_HEIGHT: f32 = 0.9;
/// O topo do chão.
pub const FLOOR_TOP: f32 = 0.0;
/// A velocidade do arranque desta fixture, m/s.
pub const DASH_SPEED: f32 = 18.0;
/// A duração do arranque desta fixture, s.
pub const DASH_TIME: f32 = 0.15;

pub struct Rig {
    pub sim: SimWorld,
    pub bridge: PhysicsBridge,
    pub player: ph2d_ecs::Entity,
}

impl Rig {
    /// Afina a config autorada deste player.
    pub fn player_cfg(&mut self, f: impl FnOnce(&mut PlatformPlayer)) {
        let mut c = self
            .sim
            .world_mut()
            .get_mut::<PlatformPlayer>(self.player)
            .expect("o player tem PlatformPlayer");
        f(&mut c);
    }

    /// Corre `ticks` tiques com a entrada dada, a partir de `from`, e devolve o
    /// tique a que chegou.
    pub fn run(&mut self, from: u64, ticks: u64, input: PlayerInput) -> u64 {
        let mut t = from;
        for _ in 0..ticks {
            self.bridge.set_player_input(self.player, input);
            t += 1;
            self.bridge.dispatch(&mut self.sim, true, t);
        }
        t
    }
}

pub fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

/// Andar para a direita.
pub fn walk_right() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    }
}

/// Andar para a direita **e apertar o arranque**.
pub fn dash_right() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        dash: true,
        ..PlayerInput::default()
    }
}

/// **Um chão comprido e um personagem em cima dele.**
///
/// `speed` em `0` dá o CONTROLE — a mesma cena, os mesmos tiques, com a
/// capacidade desligada. `start_y` põe o personagem no ar quando é preciso medir
/// um arranque em voo.
pub fn rig(speed: f32, start_y: f32) -> Rig {
    let mut sim = SimWorld::new();
    // ⚠️ Comprido de propósito: um arranque default cobre 2,7 m, e um chão curto
    // mediria a beirada em vez do arranque.
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP - 0.5)),
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
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                // ⚠️ As assistências que não são desta wave ficam FORA do
                // caminho, e declará-lo é o que mantém a fixture a medir UMA
                // coisa: sem isto, um número que se mexesse deixaria três
                // suspeitos.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                dash_speed: speed,
                dash_time: DASH_TIME,
                dash_cooldown: 0.2,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, start_y)),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}
