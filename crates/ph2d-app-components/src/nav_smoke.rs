//! ⭐⭐⭐ **Smoke da NAVEGAÇÃO** (plano 30, W3). `PH2D_NAV_SMOKE=1`.
//!
//! # A cena: **o labirinto em S, e três perseguidores**
//!
//! O herói AMARELO anda com as **setas**. Três coisas querem apanhá-lo, e cada uma ensina uma metade:
//!
//! | quem | o quê | o que tem de acontecer |
//! |---|---|---|
//! | VERMELHO | agente de navegação, corpo pequeno | **dá a volta às paredes** pelo caminho mais curto e apanha o herói onde quer que ele vá |
//! | ROXO | o MESMO agente, corpo GRANDE | a porta de baixo (`1 m`) é mais estreita que ele: **fica do lado de cá da parede, o mais perto do herói que consegue**, e diz *«too big for the door»* |
//! | CINZENTO (o CONTROLO) | uma bala que persegue em linha recta | **bate nas paredes** e fica presa — é o que seria um inimigo sem navegação |
//!
//! ⭐ **A linha azul-aço** que sai de cada agente é o CAMINHO que ele planeou (o overlay de física,
//! tecla `B`): ela dobra nos cantos das paredes, e muda quando o herói foge.
//!
//! ⚠️ **O roxo e o vermelho são o MESMO componente** — só o raio do corpo muda. É isso que torna a
//! porta legível: a malha andável é recuada pelo raio de quem anda nela.
//!
//! ⚠️ **O cinzento é o controlo**: sem ele, o artista não tem como saber se o vermelho dá a volta
//! porque acha o caminho ou porque a cena está desenhada para isso.
//!
//! ⚠️ Se a linha `[nav-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, ProjectileMotion, RigidBody,
    TopDownPlayer,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — contado do `match` do [`montar`].
pub const CENAS: u32 = 1;

const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const HEROI_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const ROXO_RGBA: [f32; 4] = [0.62, 0.38, 0.85, 1.0];
const CONTROLO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

/// O CENTRO do recinto. ⚠️ **Não é a origem, e a razão é a FOTO:** com a timeline aberta a banda
/// que sobra do ecrã enquadra `y ∈ [−2,6 ; 4,1]` a `100 px/m` (medido na 1.ª foto desta cena, que
/// escondia a porta debaixo do painel) — o recinto sobe para caber nela.
pub const CENTRO: [f32; 2] = [0.0, 0.8];
/// O interior do recinto (meio): `x ∈ [−5,8 ; 5,8]`, `y ∈ [−2,0 ; 3,6]`.
pub const MEIO_RECINTO: [f32; 2] = [5.8, 2.8];
/// Espessura das paredes (meia).
const MEIA_PAREDE: f32 = 0.25;
/// A parede de baixo-a-meio (`x = 2,7`) deixa uma PORTA junto ao chão com esta largura.
pub const PORTA: f32 = 1.0;
/// O raio do corpo do VERMELHO — passa a porta com folga (`2·r < PORTA`).
pub const RAIO_PEQUENO: f32 = 0.35;
/// O raio do corpo do ROXO — não passa a porta (`2·r > PORTA`).
pub const RAIO_GRANDE: f32 = 0.65;
/// O raio do herói.
pub const RAIO_HEROI: f32 = 0.35;
/// Onde a parede da porta está.
pub const X_PORTA: f32 = 2.0;
/// O `y` do chão do recinto (a porta abre junto dele).
pub const Y_CHAO: f32 = CENTRO[1] - MEIO_RECINTO[1];

const _: () = assert!(2.0 * RAIO_PEQUENO < PORTA && 2.0 * RAIO_GRANDE > PORTA);

/// O que o roteador montou — o nível e quem fica escolhido.
pub struct Montada {
    pub nivel: u32,
    pub vermelho: Entity,
    pub roxo: Entity,
    pub controlo: Entity,
    pub heroi: Entity,
}

fn parede(world: &mut World, nome: &str, centro: Vec2, meio: Vec2) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.x,
                half_y: meio.y,
            },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [meio.x * 2.0, meio.y * 2.0], PAREDE_RGBA),
        Transform::from_translation(centro),
    ));
}

/// Uma parede vertical em `x`, de `y0` a `y1`.
fn parede_v(world: &mut World, nome: &str, x: f32, y0: f32, y1: f32) {
    parede(
        world,
        nome,
        Vec2::new(x, (y0 + y1) * 0.5),
        Vec2::new(MEIA_PAREDE, (y1 - y0) * 0.5),
    );
}

/// Um perseguidor: o mover de vista de cima com os controlos DESLIGADOS (quem escreve a intenção é
/// a navegação) e o agente.
fn perseguidor(
    world: &mut World,
    nome: &str,
    em: Vec2,
    raio: f32,
    velocidade: f32,
    cor: [f32; 4],
    sinais: (&str, &str),
) -> Entity {
    world
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: raio },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [raio * 2.0, raio * 2.0], cor),
            TopDownPlayer::from_law(TopDownLaw {
                speed: velocidade,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Named(stable_name_id("Hero")),
                // ⚠️ **Os corpos COLIDEM**: o centro do perseguidor nunca chega a menos de
                // `r + r_herói` do centro do herói, logo «chegar» é encostar — com uma folga.
                arrive_distance: raio + RAIO_HEROI + 0.15,
                on_arrived: sinais.0.to_owned(),
                on_no_path: sinais.1.to_owned(),
                ..NavAgent::default()
            },
            Transform::from_translation(em),
        ))
        .id()
}

/// A cena `=1`.
fn cena_um(world: &mut World) -> Montada {
    let [mx, my] = MEIO_RECINTO;
    let [cx, cy] = CENTRO;
    let c = Vec2::new(cx, cy);
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [mx * 2.0, my * 2.0], CHAO_RGBA),
        Transform::from_translation(c),
    ));
    // O recinto.
    let e = MEIA_PAREDE;
    let topo = cy + my;
    parede(
        world,
        "Wall N",
        Vec2::new(cx, topo + e),
        Vec2::new(mx + 2.0 * e, e),
    );
    parede(
        world,
        "Wall S",
        Vec2::new(cx, Y_CHAO - e),
        Vec2::new(mx + 2.0 * e, e),
    );
    parede(
        world,
        "Wall W",
        Vec2::new(cx - mx - e, cy),
        Vec2::new(e, my),
    );
    parede(
        world,
        "Wall E",
        Vec2::new(cx + mx + e, cy),
        Vec2::new(e, my),
    );
    // ⭐ O S: a 1.ª parede deixa a passagem LARGA em cima (`2 m`), a 2.ª deixa a PORTA estreita em
    // baixo.
    parede_v(world, "Wall Left", -X_PORTA, Y_CHAO, topo - 2.0);
    parede_v(world, "Wall Door", X_PORTA, Y_CHAO + PORTA, topo);
    // A região andável: o interior inteiro.
    world.spawn((
        Name::new("Nav Region"),
        NavRegion {
            half_extents: MEIO_RECINTO,
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(c),
    ));

    let heroi = world
        .spawn((
            Name::new("Hero"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: RAIO_HEROI },
                ..Collider::default()
            },
            Sprite::atlas(
                WHITE_TILE_KEY,
                [RAIO_HEROI * 2.0, RAIO_HEROI * 2.0],
                HEROI_RGBA,
            ),
            TopDownPlayer::from_law(TopDownLaw {
                speed: 4.0,
                direction: DirectionMode::EightWay,
                ..TopDownLaw::default()
            }),
            Transform::from_translation(Vec2::new(4.2, 2.2)),
        ))
        .id();

    let vermelho = perseguidor(
        world,
        "Chaser",
        Vec2::new(-4.5, -1.2),
        RAIO_PEQUENO,
        2.5,
        VERMELHO_RGBA,
        ("caught you", ""),
    );
    let roxo = perseguidor(
        world,
        "Big Chaser",
        Vec2::new(-4.5, 1.0),
        RAIO_GRANDE,
        2.0,
        ROXO_RGBA,
        ("", "too big for the door"),
    );
    // ⭐⭐ **O CONTROLO**: uma bala que persegue o herói em linha recta, SEM navegação. Ela bate nas
    // paredes e ricocheteia — é o inimigo que a casa sabia fazer antes desta wave.
    let controlo = world
        .spawn((
            Name::new("Control (homing)"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], CONTROLO_RGBA),
            ProjectileMotion::from_law(
                ProjectileLaw {
                    initial_speed: 2.5,
                    max_speed: 2.5,
                    gravity: 0.0,
                    homing_accel: 6.0,
                    max_bounces: u8::MAX,
                    bounciness: 0.4,
                    range: 0.0,
                    ..ProjectileLaw::default()
                },
                stable_name_id("Hero"),
            ),
            Transform::from_translation(Vec2::new(-4.5, 3.0)),
        ))
        .id();
    Montada {
        nivel: 1,
        vermelho,
        roxo,
        controlo,
        heroi,
    }
}

/// **Monta a cena `nivel`** — o roteador.
pub fn montar(world: &mut World, nivel: u32) -> Montada {
    // Uma cena só (`CENAS = 1`): todo nível pede a `=1`.
    let _ = nivel;
    let m = cena_um(world);
    println!(
        "[nav-smoke] =1 setas para andar com o AMARELO. O VERMELHO da' a volta as paredes para te \
         apanhar; o ROXO e' grande demais para a porta de baixo e fica do lado de ca' da parede; o CINZENTO (o \
         controlo) persegue em linha recta e bate nas paredes. A linha azul de cada um e' o caminho \
         que ele planeou (tecla B)"
    );
    m
}

#[cfg(test)]
#[path = "nav_smoke_tests.rs"]
mod tests;
