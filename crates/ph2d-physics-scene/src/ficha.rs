//! **O ITEM INTEIRO NUMA TELA** (`PH2D_PHYSICS_SMOKE=120`, ficha `f1_physics_static_shape.md`, onda 4): chão em
//! Repeated, plataforma One-way, Crate, bola esticada, zona que empurra para cima, forma solta, forma dentro de um
//! corpo, Player de cápsula e quatro camadas com nome. Nasce PARADA; cada afirmação da mensagem tem gate ao lado.
//!
//! `PH2D_PHYSICS_SMOKE_SELECT=<nome>` nasce com esse objecto selecionado, `PH2D_PHYSICS_SMOKE_TOCA=1` nasce a tocar e
//! `PH2D_PHYSICS_SMOKE_VISTA=<m>` escolhe a altura da vista; `PH2D_PHYSICS_SMOKE_PAINEL=1` abre o painel Physics no
//! lugar do Inspector: as fotos, numa tela onde a mão não chega.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, Entity, ImageRegion, Name, SpriteRegion, TiledDraw, Transform, World, ZIndexOverride,
};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, CharacterAnimations, Collider, ColliderShape, LayerNames, LockRotation,
    MaterialCombine, OneWayPlatform, PhysicsSettings, PlatformPlayer, RigidBody,
};
use ph2d_render::{
    CLOUDS_TILE, COIN_SHEET, DIRT_TILE, FactoryArt, GROUND_TILE, HERO_SHEET_SOLE_BELOW_CENTRE,
    PLATFORM_TILE, Sprite,
};

const GROUND: u8 = 0;
const PLAYER: u8 = 1;
const PROPS: u8 = 2;
const ZONES: u8 = 3;
pub const CAMADAS: [&str; 4] = ["Ground", "Player", "Props", "Zones"];
/// Quem toca quem: cada camada com um «Hits» diferente. O Player só o chão; a zona só os adereços.
pub const MATRIZ: [u8; 8] = [
    1 << PLAYER | 1 << PROPS,
    1 << GROUND,
    1 << GROUND | 1 << PROPS | 1 << ZONES,
    1 << PROPS,
    0,
    0,
    0,
    0,
];

pub const FLOOR_HALF: [f32; 2] = [7.0, 0.5];
pub const ONE_WAY_AT: [f32; 2] = [-3.5, 0.85];
const ONE_WAY_HALF: [f32; 2] = [1.5, 0.25];
pub const PLAYER_AT: [f32; 2] = [-5.5, HERO_SHEET_SOLE_BELOW_CENTRE];
pub const CRATE_AT: [f32; 2] = [0.5, 5.0];
/// A forma solta fica no caminho do Crate: ele passa-lhe por dentro, porque sem corpo ela não segura nada.
pub const LOOSE_AT: [f32; 2] = [0.5, 2.5];
pub const BALL_AT: [f32; 2] = [4.0, 6.5];
pub const ZONE_AT: [f32; 2] = [4.0, 2.0];
pub const ZONE_HALF: [f32; 2] = [1.5, 2.0];
/// N, para cima. MEDIDO (`probe_cena_120`): vence o peso da bola 2 × 1, que pára de cair e sobe.
pub const ZONE_FORCE: f32 = 40.0;
/// O chão fica acima da timeline numa janela de 1080: a timeline tapa o terço de baixo da vista.
const ZONE_TILE: [f32; 2] = [3.0, 1.0];
/// A primeira moeda da folha, em Region: uma imagem parada, sem o aviso de folha do uso Still.
const COIN_CELL: [f32; 4] = [0.0, 0.0, 128.0, 128.0];
pub const VISTA: [f32; 2] = [0.5, 2.4];
pub const VISTA_ALTURA: f32 = 13.6;

/// Um bloco em Repeated: a arte de fábrica, Width e Height pelo tamanho, escala 1 e a caixa igual ao desenho.
fn azulejo(art: FactoryArt, meia: [f32; 2], tint: [f32; 4]) -> (Sprite, TiledDraw) {
    let [cw, ch] = art.cell_px();
    #[allow(clippy::cast_precision_loss)]
    let tile = [1.0, ch as f32 / cw as f32];
    (
        Sprite::atlas(art.key, [2.0 * meia[0], 2.0 * meia[1]], tint),
        TiledDraw { tile },
    )
}

fn caixa(meia: [f32; 2], layer: u8) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: meia[0],
            half_y: meia[1],
        },
        layer,
        ..Collider::default()
    }
}

fn pose(at: [f32; 2]) -> Transform {
    Transform::from_translation(Vec2::new(at[0], at[1]))
}

fn corpo(kind: BodyKind) -> RigidBody {
    RigidBody { kind }
}

/// O Player como o menu Add ▸ Character o faz: o boneco de fábrica, a cápsula e a lei de plataforma.
fn player(world: &mut World) -> Entity {
    let e = world
        .spawn((
            Name::new("Player"),
            pose(PLAYER_AT),
            corpo(BodyKind::Dynamic),
            LockRotation,
            CharacterAnimations::default(),
            PlatformPlayer {
                float_height: HERO_SHEET_SOLE_BELOW_CENTRE,
                ..PlatformPlayer::default()
            },
        ))
        .id();
    ph2d_render::veste_o_heroi(world, e).expect("o boneco de fábrica");
    let mut capsula = MaterialCombine::sem_atrito(ColliderShape::Capsule {
        half_height: 0.15,
        radius: 0.2,
    });
    capsula.0.layer = PLAYER;
    world.entity_mut(e).insert(capsula);
    e
}

/// Monta a cena no mundo (a sonda e os gates usam esta mesma porta).
pub fn monta(world: &mut World) {
    for (i, nome) in CAMADAS.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        LayerNames::rename(world, i as u8, nome);
    }
    let floor = world
        .spawn((
            Name::new("Floor"),
            pose([0.0, -FLOOR_HALF[1]]),
            azulejo(GROUND_TILE, FLOOR_HALF, [1.0; 4]),
            corpo(BodyKind::Static),
            caixa(FLOOR_HALF, GROUND),
        ))
        .id();
    // Dentro do Floor: «Part of: Floor». É ela que fecha o nível à direita; à esquerda, uma parede que se vê.
    world.spawn((
        Name::new("Invisible Wall"),
        pose([FLOOR_HALF[0] + 0.5, 4.0]),
        caixa([0.5, 4.0], GROUND),
        ChildOf(floor),
    ));
    world.spawn((
        Name::new("Left Wall"),
        pose([-FLOOR_HALF[0] - 0.5, 3.5]),
        azulejo(DIRT_TILE, [0.5, 4.0], [1.0; 4]),
        corpo(BodyKind::Static),
        caixa([0.5, 4.0], GROUND),
    ));
    world.spawn((
        Name::new("One-way Platform"),
        pose(ONE_WAY_AT),
        azulejo(PLATFORM_TILE, ONE_WAY_HALF, [1.0; 4]),
        corpo(BodyKind::Static),
        caixa(ONE_WAY_HALF, GROUND),
        OneWayPlatform,
    ));
    world.spawn((
        Name::new("Collision Shape"),
        pose(LOOSE_AT),
        caixa([1.0, 0.25], GROUND),
    ));
    world.spawn((
        Name::new("Crate"),
        pose(CRATE_AT),
        azulejo(DIRT_TILE, [0.5, 0.5], [1.0; 4]),
        corpo(BodyKind::Dynamic),
        caixa([0.5, 0.5], PROPS),
    ));
    let mut bola = pose(BALL_AT);
    bola.scale = Vec2::new(2.0, 1.0);
    world.spawn((
        Name::new("Stretched Ball"),
        bola,
        Sprite::atlas(COIN_SHEET.key, [1.0, 1.0], [1.0; 4]),
        SpriteRegion::for_atlas(COIN_CELL),
        ImageRegion {
            grid: COIN_SHEET.grid,
        },
        corpo(BodyKind::Dynamic),
        Collider {
            shape: ColliderShape::Ball { radius: 0.5 },
            layer: PROPS,
            ..Collider::default()
        },
    ));
    world.spawn((
        Name::new("Updraft Zone"),
        pose(ZONE_AT),
        Sprite::atlas(
            CLOUDS_TILE.key,
            [2.0 * ZONE_HALF[0], 2.0 * ZONE_HALF[1]],
            [0.55, 0.8, 1.0, 1.0],
        ),
        TiledDraw { tile: ZONE_TILE },
        ZIndexOverride(-1),
        corpo(BodyKind::Static),
        Collider {
            is_sensor: true,
            ..caixa(ZONE_HALF, ZONES)
        },
        AreaEffector {
            force: [0.0, ZONE_FORCE],
        },
    ));
    player(world);
    ph2d_ecs::assign_missing_stable_ids(world);
    ph2d_ecs::assign_missing_root_order(world);
}

/// As regras do mundo da cena: a matriz das quatro camadas.
#[must_use]
pub fn regras(base: PhysicsSettings) -> PhysicsSettings {
    PhysicsSettings {
        layer_matrix: ph2d_physics_ecs::LayerMatrix::rows_from_legacy(MATRIZ),
        ..base
    }
}

#[cfg(test)]
#[path = "ficha_tests.rs"]
mod tests;
