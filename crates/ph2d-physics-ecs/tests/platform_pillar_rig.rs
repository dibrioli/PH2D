//! **A cena do PILAR ESTREITO** (`W-ShapeCast`) — a fixture que o gate e a sonda
//! partilham.
//!
//! ⚠️ **Incluída por `#[path]` nos dois arquivos, e não copiada**, pelo motivo
//! que os rigs do agachar, da parede e do arranque já escrevem: cada arquivo em
//! `tests/` é um crate próprio, e uma segunda cópia da cena seria uma segunda
//! resposta a *"como é um personagem debaixo de uma pedra estreita"*.
//!
//! # ⚠️ Por que o PILAR, e não a marquise do `platform_crouch_rig`
//!
//! Aquela cena tem uma laje **larga**, que qualquer um dos três raios do sensor
//! antigo via. Esta tem uma pedra de **8 cm** posta no vão entre duas amostras —
//! a única geometria em que *ler o teto por amostragem* e *varrer o corpo*
//! respondem coisas diferentes, que é exactamente o que a wave existe para
//! mudar.
//!
//! ⚠️ O `allow(dead_code)` é do formato, não do descuido.
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// Meia-altura do segmento reto da cápsula do player.
pub const HALF_H: f32 = 0.3;
/// Raio da cápsula. ⚠️ **A caixa envolvente mede exactamente isto** de
/// meia-largura, e é essa coincidência que torna a cena legível: os três raios
/// do sensor antigo nasciam em `−0,20 · 0,00 · +0,20`.
pub const RADIUS: f32 = 0.2;
/// A altura de flutuação DE PÉ.
pub const FLOAT_HEIGHT: f32 = 1.1;
/// A altura de flutuação AGACHADO.
pub const CROUCH_HEIGHT: f32 = 0.6;
/// Meia-altura da caixa envolvente (`half_height + radius`).
pub const BODY_HALF: f32 = HALF_H + RADIUS;

pub struct Rig {
    pub sim: SimWorld,
    pub bridge: PhysicsBridge,
    pub player: Entity,
}

impl Rig {
    /// Corre `ticks` tiques com a entrada dada.
    pub fn run(&mut self, from: u64, ticks: u64, input: PlayerInput) -> u64 {
        let mut t = from;
        for _ in 0..ticks {
            self.bridge.set_player_input(self.player, input);
            t += 1;
            self.bridge.dispatch(&mut self.sim, true, t);
        }
        t
    }

    /// Agacha por `hold` tiques e solta por `free`. Devolve `(topo, x)`.
    ///
    /// ⚠️ **O `x` é metade da resposta**, e sem ele a tabela mente: um pilar
    /// estreito é uma coisa de que o solver consegue **escorregar para o lado**,
    /// e *subiu porque a pedra é invisível* e *subiu porque fugiu dela* são
    /// vereditos diferentes sobre o mesmo número de altura.
    pub fn crouch_then_release(&mut self, hold: u64, free: u64) -> (f32, f32) {
        let t = self.run(0, hold, down());
        self.run(t, free, PlayerInput::default());
        let (x, y) = pose(&self.sim);
        (y + BODY_HALF, x)
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

/// Segurar BAIXO, parado.
pub fn down() -> PlayerInput {
    PlayerInput {
        down: true,
        ..PlayerInput::default()
    }
}

/// **Um chão comprido, um player, e uma PEDRA** — `None` dá o controle (céu
/// limpo).
///
/// A pedra é dada por `(x, meia-largura, face de baixo)`; ela é grossa (2 m) de
/// propósito, para que a pergunta seja sobre a face de baixo e nunca sobre o
/// corpo passar por cima.
pub fn rig(stone: Option<(f32, f32, f32)>) -> Rig {
    let mut sim = SimWorld::new();
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
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));

    if let Some((x, half_x, bottom)) = stone {
        const HALF_Y: f32 = 1.0;
        sim.world_mut().spawn((
            Name::new("Stone"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x,
                    half_y: HALF_Y,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, bottom + HALF_Y)),
        ));
    }

    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: HALF_H,
                    radius: RADIUS,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                crouch_height: CROUCH_HEIGHT,
                // As assistências que não são desta cena ficam FORA do caminho —
                // a de quina especialmente, que existe para EMPURRAR um corpo
                // que sobe contra uma beirada, e esta cena é feita de beiradas.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, CROUCH_HEIGHT)),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}
