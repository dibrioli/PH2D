//! **A PORTA** (`PH2D_PHYSICS_SMOKE=73`, W-Signal).
//!
//! Os quatro canais de leitura da física existem desde o W7 — *quem está dentro
//! de um sensor*, *quem toca quem*, *as transições*, *o pico do impacto* — todos
//! gateados, e **nenhum fazia nada acontecer**. Esta cena é a primeira em que uma
//! colisão CAUSA alguma coisa.
//!
//! ⚠️ O consumidor não é novo: é o MESMO outbox em que a timeline emite os sinais
//! dos markers (ADR-0143), cujo v1 é um toast. O que faltava era o publicador.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody, SignalOnHit};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

const DOOR_RGBA: [f32; 4] = [0.90, 0.55, 0.88, 1.0];
const BELL_RGBA: [f32; 4] = [0.98, 0.75, 0.25, 1.0];
const QUIET_RGBA: [f32; 4] = [0.55, 0.58, 0.62, 1.0];
const BALL_RGBA: [f32; 4] = [0.45, 0.80, 0.95, 1.0];
const GROUND_RGBA: [f32; 4] = [0.35, 0.45, 0.40, 1.0];

const CAMERA_CENTRE: [f32; 2] = [0.0, 1.6];
const CAMERA_HEIGHT: f32 = 10.0;

/// A altura de onde as bolas caem — a MESMA nas três pistas, para a única
/// diferença ser o que há embaixo.
const DROP_Y: f32 = 5.0;

/// Onde cada pista fica, e o que ela responde.
pub(crate) const LANES: [f32; 3] = [-3.5, 0.0, 3.5];
pub(crate) const LANE_NAMES: [&str; 3] = ["Door", "Bell", "Quiet"];

fn cuboid(hx: f32, hy: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        ..Collider::default()
    }
}

pub(crate) fn build_signal_scene(world: &mut World) {
    world.spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(8.0, 0.3),
        Sprite::atlas(WHITE_TILE_KEY, [16.0, 0.6], GROUND_RGBA),
        Transform::from_translation(Vec2::new(0.0, -0.3)),
    ));

    // 1. A PORTA — um SENSOR marcado. A bola atravessa e a porta grita UMA vez.
    world.spawn((
        Name::new(LANE_NAMES[0]),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            ..cuboid(1.0, 0.5)
        },
        SignalOnHit("door".to_string()),
        Sprite::atlas(WHITE_TILE_KEY, [2.0, 1.0], DOOR_RGBA),
        Transform::from_translation(Vec2::new(LANES[0], 2.0)),
    ));

    // 2. O SINO — um corpo SÓLIDO marcado. A bola BATE nele e ele grita.
    world.spawn((
        Name::new(LANE_NAMES[1]),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(1.0, 0.2),
        SignalOnHit("bell".to_string()),
        Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.4], BELL_RGBA),
        Transform::from_translation(Vec2::new(LANES[1], 2.0)),
    ));

    // 3. O CONTROLE — a MESMA plataforma, sem componente. Sem ele a cena não
    //    distingue *"o sinal disparou"* de *"tudo dispara"*.
    world.spawn((
        Name::new(LANE_NAMES[2]),
        RigidBody {
            kind: BodyKind::Static,
        },
        cuboid(1.0, 0.2),
        Sprite::atlas(WHITE_TILE_KEY, [2.0, 0.4], QUIET_RGBA),
        Transform::from_translation(Vec2::new(LANES[2], 2.0)),
    ));

    for (i, x) in LANES.iter().enumerate() {
        world.spawn((
            Name::new(format!("Ball {}", i + 1)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            cuboid(0.25, 0.25),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], BALL_RGBA),
            Transform::from_translation(Vec2::new(*x, DROP_Y)),
        ));
    }
}

#[cfg(test)]
#[path = "physics_smoke_signal_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 73 (W-Signal).** Uma colisão CAUSA alguma coisa.
    pub(crate) fn physics_smoke_signal(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        build_signal_scene(gfx.sim.world_mut());
        gfx.camera.center = CAMERA_CENTRE;
        gfx.camera.height_world = CAMERA_HEIGHT;
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 73] A PORTA -- a primeira cena em que uma COLISAO faz\n  \
               alguma coisa acontecer.\n\n  \
               Ate esta wave a fisica publicava QUATRO canais de leitura (quem esta'\n  \
               dentro de um sensor, quem toca quem, as transicoes, o pico do impacto),\n  \
               todos gateados, e nenhum deles fazia nada. Faltava o publicador -- nao o\n  \
               consumidor: o outbox e' o MESMO em que a timeline emite os sinais dos\n  \
               markers, e o consumidor v1 dele e' um TOAST.\n\n  \
               1. Toque Play e olhe o canto superior: tres bolas iguais caem de {drop:.0} m.\n     \
                  - ESQUERDA '{n0}' -- um SENSOR marcado. A bola ATRAVESSA e sobe um\n       \
                    toast 'Signal: door'. Um sensor nunca gera contato, entao um canal\n       \
                    so' de colisao solida deixaria a porta -- o caso canonico de\n       \
                    gameplay -- em silencio.\n     \
                  - MEIO '{n1}' -- um corpo SOLIDO marcado. A bola BATE e sobe\n       \
                    'Signal: bell'.\n     \
                  - DIREITA '{n2}' -- a MESMA plataforma, SEM o componente. Nada sobe,\n       \
                    e ela e' o CONTROLE: sem ela a cena nao distingue *o sinal\n       \
                    disparou* de *tudo dispara*.\n\n  \
               2. UMA vez, nao uma por quadro. A bola do sino fica encostada nele e o\n     \
                  toast NAO se repete -- o canal reporta a CHEGADA, nao o estado. Um som\n     \
                  que toca sessenta vezes por segundo nao e' um som.\n\n  \
               3. AUTORE VOCE MESMO: selecione '{n2}' (a plataforma quieta) na\n     \
                  Hierarquia. Na secao Physics Body, a ultima row e' um campo de texto\n     \
                  'Signal on hit...'. Escreva 'quiet' e de Play de novo: agora ela\n     \
                  grita tambem. Apague o texto e ela volta a calar -- um nome em BRANCO\n     \
                  nao e' um sinal (a mesma regra do marker da timeline: um sinal sem\n     \
                  nome nao e' um contrato que alguem possa casar).\n\n  \
               (!) Arraste a regua para TRAS: nada dispara. Um evento descreve uma\n      \
                   transicao que a simulacao ATRAVESSOU, e um scrub nao e' uma chegada.\n\n  \
               (!) O toast e' o consumidor v1, e de proposito: ele e' a prova visivel de\n      \
                   que o canal desacoplado fecha a volta. Um script Luau ou uma pista de\n      \
                   audio ouvindo o MESMO nome nao muda uma linha da fisica.\n",
            drop = DROP_Y,
            n0 = LANE_NAMES[0],
            n1 = LANE_NAMES[1],
            n2 = LANE_NAMES[2],
        );
    }
}
