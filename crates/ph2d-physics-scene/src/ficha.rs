//! **O ITEM INTEIRO NUMA TELA** (`PH2D_PHYSICS_SMOKE=120`, ficha `f1_physics_static_shape.md`, onda 4): chão em
//! Repeated, plataforma One-way, Crate, bola esticada, zona que empurra para cima, forma solta, forma dentro de um
//! corpo, Player de cápsula, quatro camadas com nome e
//! (W6) a Coin «Outline» traçada da imagem, a Ring com buraco e a Marble dentro, e a Table de três formas. Nasce PARADA; cada afirmação da mensagem tem gate ao lado.
//!
//! `PH2D_PHYSICS_SMOKE_SELECT=<nome>` nasce com esse objecto selecionado, `PH2D_PHYSICS_SMOKE_TOCA=1` nasce a tocar e
//! `PH2D_PHYSICS_SMOKE_VISTA=<m>` escolhe a altura da vista; `PH2D_PHYSICS_SMOKE_PAINEL=1` abre o painel Physics no
//! lugar do Inspector: as fotos, numa tela onde a mão não chega.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, Entity, ImageRegion, Name, SpriteRegion, TiledDraw, Transform, World, ZIndexOverride,
};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, CharacterAnimations, Collider, ColliderOutline, ColliderShape,
    LayerNames, LockRotation, MaterialCombine, OneWayPlatform, PhysicsSettings, PlatformPlayer,
    RigidBody,
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
/// As camadas 4 a 7 ficam no padrão de fábrica: sem objecto nem nome, não aparecem nos quadros.
const POR_USAR: u8 = 0xF0;
/// Quem toca quem: cada camada com os seus quadros acesos. O Player só o chão; a zona só os adereços.
pub const MATRIZ: [u8; 8] = [
    1 << PLAYER | 1 << PROPS | POR_USAR,
    1 << GROUND | POR_USAR,
    1 << GROUND | 1 << PROPS | 1 << ZONES | POR_USAR,
    1 << PROPS | POR_USAR,
    u8::MAX,
    u8::MAX,
    u8::MAX,
    u8::MAX,
];

pub const FLOOR_HALF: [f32; 2] = [7.0, 0.5];
pub const ONE_WAY_AT: [f32; 2] = [-3.5, 0.85];
const ONE_WAY_HALF: [f32; 2] = [1.5, 0.25];
pub const PLAYER_AT: [f32; 2] = [-5.5, HERO_SHEET_SOLE_BELOW_CENTRE];
pub const CRATE_AT: [f32; 2] = [0.5, 5.0];
/// A forma solta fica no caminho do Crate: ele passa-lhe por dentro, porque sem corpo ela não segura nada.
pub const LOOSE_AT: [f32; 2] = [0.5, 2.5];
pub const KINEMATIC_AT: [f32; 2] = [-3.0, 4.5];
pub const BALL_AT: [f32; 2] = [4.0, 6.5];
/// Um corpo SEM forma (dono 08/11): o cartão Body avisa e oferece «Add Shape».
pub const NO_SHAPE_AT: [f32; 2] = [-1.5, 7.5];
pub const ZONE_AT: [f32; 2] = [4.0, 2.0];
pub const ZONE_HALF: [f32; 2] = [1.5, 2.0];
/// N, para cima. MEDIDO (`probe_cena_120`): vence o peso da bola 2 × 1, que pára de cair e sobe.
pub const ZONE_FORCE: f32 = 40.0;
/// O chão fica acima da timeline numa janela de 1080: a timeline tapa o terço de baixo da vista.
const ZONE_TILE: [f32; 2] = [3.0, 1.0];
/// A primeira moeda da folha, em Region: uma imagem parada, sem o aviso de folha do uso Still.
const COIN_CELL: [f32; 4] = [0.0, 0.0, 128.0, 128.0];
/// W6: a moeda com a forma «Outline» traçada da imagem; a argola com buraco e a bolinha dentro; a mesa de 3 formas.
pub const COIN_AT: [f32; 2] = [-0.9, 6.0];
pub const RING_AT: [f32; 2] = [-5.5, 6.0];
pub const MARBLE_AT: [f32; 2] = [-5.5, 6.2];
pub const MARBLE_RADIUS: f32 = 0.2;
/// A argola: 2 m de lado, buraco de 1 m no meio (Q4 de uma imagem de 4 px).
pub const RING_HALF: f32 = 1.0;
pub const TABLE_AT: [f32; 2] = [6.3, 3.0];
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

/// Monta a cena no mundo (a sonda e os gates usam esta mesma porta): cada corpo com a forma numa filha
/// «Collision Shape» (opção B, ADR-0199).
pub fn monta(world: &mut World) {
    monta_com(world, true);
}

/// A MESMA cena no formato `<= 219`, a forma no próprio corpo: a fixtura dos gates da migração
/// (`ficha_migra_tests`, `project_formas_tests`). Nenhum smoke a usa.
pub fn monta_v219(world: &mut World) {
    monta_com(world, false);
}

/// `na_filha`: a forma de cada corpo passa à filha DEPOIS de a cena inteira nascer, para os corpos
/// terem os mesmos `StableId` nos dois formatos (as filhas numeram-se por último, como na migração).
fn monta_com(world: &mut World, na_filha: bool) {
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
    let mut corpos = vec![floor];
    corpos.push(
        world
            .spawn((
                Name::new("Left Wall"),
                pose([-FLOOR_HALF[0] - 0.5, 3.5]),
                azulejo(DIRT_TILE, [0.5, 4.0], [1.0; 4]),
                corpo(BodyKind::Static),
                caixa([0.5, 4.0], GROUND),
            ))
            .id(),
    );
    corpos.push(
        world
            .spawn((
                Name::new("One-way Platform"),
                pose(ONE_WAY_AT),
                azulejo(PLATFORM_TILE, ONE_WAY_HALF, [1.0; 4]),
                corpo(BodyKind::Static),
                caixa(ONE_WAY_HALF, GROUND),
                OneWayPlatform,
            ))
            .id(),
    );
    // Onda 9g: o terceiro tipo de corpo na cena, para o traço pontilhado do Kinematic se ver.
    corpos.push(
        world
            .spawn((
                Name::new("Kinematic Platform"),
                pose(KINEMATIC_AT),
                azulejo(PLATFORM_TILE, ONE_WAY_HALF, [1.0; 4]),
                corpo(BodyKind::Kinematic),
                caixa(ONE_WAY_HALF, GROUND),
            ))
            .id(),
    );
    world.spawn((
        Name::new("Loose Shape"),
        pose(LOOSE_AT),
        caixa([1.0, 0.25], GROUND),
    ));
    corpos.push(
        world
            .spawn((
                Name::new("Crate"),
                pose(CRATE_AT),
                azulejo(DIRT_TILE, [0.5, 0.5], [1.0; 4]),
                corpo(BodyKind::Dynamic),
                caixa([0.5, 0.5], PROPS),
            ))
            .id(),
    );
    let mut bola = pose(BALL_AT);
    bola.scale = Vec2::new(2.0, 1.0);
    corpos.push(
        world
            .spawn((
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
            ))
            .id(),
    );
    corpos.push(
        world
            .spawn((
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
            ))
            .id(),
    );
    world.spawn((
        Name::new("No Shape Body"),
        pose(NO_SHAPE_AT),
        azulejo(DIRT_TILE, [0.5, 0.5], [1.0; 4]),
        corpo(BodyKind::Static),
    ));
    corpos.push(player(world));
    corpos.extend(outline_argola_e_mesa(world));
    ph2d_ecs::assign_missing_stable_ids(world);
    if na_filha {
        for c in corpos {
            ph2d_physics_ecs::move_forma_para_filha(world, c).expect("um corpo com forma");
        }
    }
    ph2d_ecs::assign_missing_root_order(world);
}

fn com_layer(shape: ColliderShape, layer: u8, offset: [f32; 2]) -> Collider {
    Collider {
        shape,
        layer,
        offset,
        ..Collider::default()
    }
}

fn moeda(lado: f32) -> (Sprite, SpriteRegion, ImageRegion) {
    (
        Sprite::atlas(COIN_SHEET.key, [lado, lado], [1.0; 4]),
        SpriteRegion::for_atlas(COIN_CELL),
        ImageRegion {
            grid: COIN_SHEET.grid,
        },
    )
}

/// W6: «Coin» (a forma Outline sem anéis: a cola traça-a da imagem no 1.º quadro), «Ring» (dois anéis à mão, a
/// bolinha «Marble» nasce no buraco) e «Table» (um corpo, três formas: o tampo nele e as pernas filhas).
fn outline_argola_e_mesa(world: &mut World) -> [Entity; 4] {
    let outline = |h: f32| ColliderShape::Outline {
        half_x: h,
        half_y: h,
    };
    let coin = world
        .spawn((
            Name::new("Coin"),
            pose(COIN_AT),
            moeda(1.0),
            corpo(BodyKind::Dynamic),
            com_layer(outline(0.5), PROPS, [0.0; 2]),
        ))
        .id();
    let anel = ColliderOutline {
        rings: vec![
            vec![[0, 0], [64, 0], [64, 64], [0, 64]],
            vec![[16, 16], [16, 48], [48, 48], [48, 16]],
        ]
        .into(),
        picture_px: [4, 4],
        custom: true,
        ..ColliderOutline::default()
    };
    let ring = world
        .spawn((
            Name::new("Ring"),
            pose(RING_AT),
            corpo(BodyKind::Static),
            com_layer(outline(RING_HALF), GROUND, [0.0; 2]),
            anel,
        ))
        .id();
    let marble = world
        .spawn((
            Name::new("Marble"),
            pose(MARBLE_AT),
            moeda(2.0 * MARBLE_RADIUS),
            corpo(BodyKind::Dynamic),
            com_layer(
                ColliderShape::Ball {
                    radius: MARBLE_RADIUS,
                },
                PROPS,
                [0.0; 2],
            ),
        ))
        .id();
    let caixa = |x, y| ColliderShape::Cuboid {
        half_x: x,
        half_y: y,
    };
    let table = world
        .spawn((
            Name::new("Table"),
            pose(TABLE_AT),
            azulejo(PLATFORM_TILE, [0.6, 0.1], [1.0; 4]),
            corpo(BodyKind::Dynamic),
            com_layer(caixa(0.6, 0.1), PROPS, [0.0; 2]),
        ))
        .id();
    for (nome, x) in [("Left Leg", -0.5), ("Right Leg", 0.5)] {
        world.spawn((
            Name::new(nome),
            Transform::default(),
            com_layer(caixa(0.1, 0.4), PROPS, [x, -0.5]),
            ChildOf(table),
        ));
    }
    [coin, ring, marble, table]
}

/// As fotos da matriz: `PH2D_PHYSICS_SMOKE_CAMADAS=<n>` camadas nomeadas, além das quatro da cena.
pub fn nomeia_para_as_fotos(world: &mut World) {
    let nomeadas = std::env::var("PH2D_PHYSICS_SMOKE_CAMADAS")
        .ok()
        .and_then(|v| v.trim().parse::<u8>().ok());
    for i in 4..nomeadas.unwrap_or(0) {
        LayerNames::rename(world, i, &format!("Layer {i}"));
    }
}

/// As fotos do Outline (`PH2D_PHYSICS_SMOKE_OUTLINE=<corpo>`, W6: `Coin`, `Ring`, `Table`): a forma principal desse
/// corpo nasce selecionada e a vista centra-se nele. Devolve a forma e o centro.
pub fn outline_para_as_fotos(world: &mut World) -> Option<(Entity, [f32; 2])> {
    let nome = std::env::var("PH2D_PHYSICS_SMOKE_OUTLINE").ok()?;
    let mut q = world.query::<(Entity, &Name, &Transform)>();
    let (corpo, _, t) = q.iter(world).find(|(_, n, _)| n.0 == nome.trim())?;
    let centro = [t.translation.x, t.translation.y];
    Some((ph2d_physics_ecs::main_shape(world, corpo)?, centro))
}

/// A foto da exceção (`PH2D_PHYSICS_SMOKE_EXCECAO=1`, dono 09/11): o Crate ganha mais duas formas e uma delas desliga
/// Ground e Props, o exemplo do dono. Devolve essa forma, que nasce selecionada.
pub fn excecao_para_as_fotos(world: &mut World) -> Option<Entity> {
    std::env::var_os("PH2D_PHYSICS_SMOKE_EXCECAO")?;
    let crate_ = {
        let mut q = world.query::<(Entity, &Name)>();
        q.iter(world)
            .find(|(_, n)| n.0 == "Crate")
            .map(|(e, _)| e)?
    };
    let formas = [[-0.25, 0.25], [0.25, 0.25]].map(|offset| {
        let f = ph2d_physics_ecs::nasce_com_forma(world, crate_, caixa([0.2, 0.2], PROPS).shape);
        if let Some(mut c) = world.get_mut::<Collider>(f) {
            c.layer = PROPS;
            c.offset = offset;
        }
        f
    });
    let excecao = formas[0];
    world
        .entity_mut(excecao)
        .insert(ph2d_physics_ecs::CollisionOverride {
            include: 0,
            exclude: (1 << GROUND) | (1 << PROPS),
        });
    // A segunda forma desliga só Ground: a camada Ground mostra duas que diferem, com o traço entre elas.
    world
        .entity_mut(formas[1])
        .insert(ph2d_physics_ecs::CollisionOverride {
            include: 0,
            exclude: 1 << GROUND,
        });
    Some(excecao)
}

/// A foto do olho (`PH2D_PHYSICS_SMOKE_OLHO=<nome exato>`, dono 09/11): o objecto com esse nome nasce com o olho
/// da Hierarquia fechado — a única forma do Player desligada: ele cai e atravessa o Floor.
pub fn olho_para_as_fotos(world: &mut World) {
    let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_OLHO") else {
        return;
    };
    let alvo = {
        let mut q = world.query::<(Entity, &Name)>();
        q.iter(world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e)
    };
    if let Some(e) = alvo {
        world.entity_mut(e).insert(ph2d_ecs::Visibility::hidden());
    }
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

#[cfg(test)]
#[path = "ficha_migra_tests.rs"]
mod migra_tests;
