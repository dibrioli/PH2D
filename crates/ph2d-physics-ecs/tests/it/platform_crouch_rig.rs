//! **A cena do AGACHAR** (W15) — a fixture que o gate e a sonda partilham.
//!
//! ⚠️ **Incluída por `#[path]` nos dois arquivos, e não copiada**, pelo motivo
//! que os rigs da parede e do arranque já escrevem: cada arquivo em `tests/` é
//! um crate próprio, e uma segunda cópia da cena seria uma segunda resposta a
//! *"como é um personagem debaixo de uma marquise"*.
//!
//! # ⚠️ A altura agachada tem um PISO, e ele é GEOMETRIA do collider
//!
//! A cápsula desta fixture mede `half_height + radius = 0,5` do centro para
//! baixo, então uma altura de flutuação abaixo disso enterra o corpo no chão e o
//! solver passa a resolver uma penetração. **A lei pura não pode clampar isso** —
//! ela não conhece formas, de propósito —, então o piso vive onde a geometria
//! vive: nesta fixture, no componente do artista, e no aviso do smoke.
//!
//! ⚠️ O `allow(dead_code)` é do formato, não do descuido.
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A altura de flutuação DE PÉ desta fixture.
pub const FLOAT_HEIGHT: f32 = 1.1;
/// A altura de flutuação AGACHADO — acima do piso geométrico de 0,5 (ver o topo).
pub const CROUCH_HEIGHT: f32 = 0.6;
/// A velocidade de cruzeiro agachado desta fixture, m/s.
pub const CROUCH_SPEED: f32 = 2.0;
/// O topo do chão.
pub const FLOOR_TOP: f32 = 0.0;
/// Meia-altura da CAIXA envolvente da cápsula (`half_height + radius`).
pub const BODY_HALF: f32 = 0.5;
/// Onde a marquise começa, em x.
pub const OVERHANG_X0: f32 = 5.0;

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

    /// Corre `ticks` tiques com a entrada dada, a partir de `from`.
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

/// Andar para a direita **segurando BAIXO**.
///
/// ⚠️ É o MESMO botão da W12 (descer através de uma plataforma jump-through) —
/// esta wave não acrescenta entrada nenhuma, e é isso que torna o gesto barato.
pub fn crouch_right() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        down: true,
        ..PlayerInput::default()
    }
}

/// **Um chão comprido, um personagem, e opcionalmente uma MARQUISE.**
///
/// - `crouch_height` em `0` dá o CONTROLE (a capacidade desligada).
/// - `overhang_bottom` põe uma laje cuja face de BAIXO fica nessa altura,
///   começando em [`OVERHANG_X0`] e indo para a direita.
pub fn rig(crouch_height: f32, overhang_bottom: Option<f32>) -> Rig {
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
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP - 0.5)),
    ));

    if let Some(bottom) = overhang_bottom {
        // A laje é grossa e comprida: o que se mede é passar por baixo dela, e
        // uma laje curta deixaria o personagem sair pelo outro lado durante a
        // janela do gate.
        const HALF_X: f32 = 12.0;
        const HALF_Y: f32 = 1.0;
        sim.world_mut().spawn((
            Name::new("Overhang"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: HALF_X,
                    half_y: HALF_Y,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(OVERHANG_X0 + HALF_X, bottom + HALF_Y)),
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
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                // ⚠️ As assistências que não são desta wave ficam FORA do
                // caminho — sem isto, um número que se mexesse deixaria vários
                // suspeitos. ⚠️ O `corner_reach` em zero é especialmente
                // load-bearing aqui: a correção de quina existe para EMPURRAR
                // um corpo que sobe contra uma beirada, e esta cena é feita de
                // beiradas.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                crouch_height,
                crouch_speed: CROUCH_SPEED,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT_HEIGHT)),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}
