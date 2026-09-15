//! ⭐⭐⭐ **Smoke do PROJÉCTIL** (TOP-20 #14, W4). `PH2D_PROJECTILE_SMOKE=1|2`.
//!
//! # `=1` — **a galeria de tiro**
//!
//! Quatro balas lado a lado, cada uma com UM knob diferente, todas disparadas do mesmo sítio para a
//! mesma parede. Da esquerda para a direita:
//!
//! | bala | o que ela demonstra |
//! |---|---|
//! | **amarela** | o tiro recto — ela bate na parede e **morre ali** (`Max Bounces = 0`) |
//! | **laranja** | o **RICOCHETE** — ela volta, e volta outra vez (`Max Bounces = 4`) |
//! | **azul** | o **ARCO** — a gravidade puxa-a, e a flecha **aponta para onde ela voa** |
//! | **verde** | o **ALCANCE** — ela morre a meio do caminho, sem bater em nada |
//!
//! ⚠️ **As quatro nascem com a MESMA rapidez e no MESMO instante** — o que muda entre elas é um
//! campo, e é isso que torna cada coluna legível. *Quatro balas com quatro rapidezes não ensinariam
//! nada: tudo seria diferente de tudo.*
//!
//! # `=2` — **o míssil que persegue**
//!
//! Um alvo cinzento que **anda**, e dois projécteis disparados a apontar para o lado errado: o
//! **vermelho** persegue (`Homing`), o **cinzento** não (o CONTROLO, com tudo igual menos esse
//! campo).
//!
//! **O que tem de acontecer:** o vermelho **curva** e vai atrás do alvo; o cinzento segue a direito
//! e passa ao largo.
//!
//! ⚠️ **O alvo ANDA de propósito** — um alvo parado não distingue *«ele vai até lá»* de *«ele foi
//! disparado para lá»*.
//!
//! ⚠️ Se a linha `[projectile-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ProjectileMotion, RigidBody, TopDownPlayer,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 2;

const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const CONTROLO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

/// A rapidez das quatro balas da `=1`, m/s — ⚠️ **a MESMA para todas**, ver o cabeçalho.
const RAPIDEZ: f32 = 9.0;

fn parede(world: &mut World, nome: &str, em: Vec2, meio: Vec2) {
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
        Transform::from_translation(em),
    ));
}

/// Uma bala: um corpo **cinemático** com o componente da wave.
///
/// ⚠️ **Cinemático e não dinâmico**, e é o desenho do componente: uma bala de arcade **não tem
/// massa** — ela atravessa a cena com a mesma rapidez seja o que for que bata. Um corpo dinâmico
/// desvia-se e perde rapidez (medido: `12,001 → 10,252` contra uma caixa leve).
fn bala(world: &mut World, nome: &str, em: Vec2, cor: [f32; 4], law: ProjectileLaw, alvo: u64) {
    world.spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.18 },
            ..Collider::default()
        },
        // ⚠️ **Um rectângulo COMPRIDO** e não um quadrado: é ele que torna o *Face Velocity*
        // visível — uma bala redonda aponta para todo o lado.
        Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.22], cor),
        ProjectileMotion::from_law(law, alvo),
        Transform::from_translation(em),
    ));
}

/// A cena `=1` — a galeria de tiro.
fn cena_um(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 14.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // ⚠️ **O chão é a PRIMEIRA raiz criada**, e desde 2026-09-15 isso quer dizer que ele desenha
    // ATRÁS de tudo. A varredura que numera as raízes invertia a ordem de criação até essa data.
    parede(world, "Wall", Vec2::new(7.0, 0.0), Vec2::new(0.6, 7.0));

    let base = ProjectileLaw {
        initial_speed: RAPIDEZ,
        ..ProjectileLaw::default()
    };
    // ⚠️ **Uma linha por knob**, e a ordem é a de leitura: recta, ricochete, arco, alcance.
    bala(
        world,
        "Straight",
        Vec2::new(-7.0, 4.5),
        [0.95, 0.72, 0.25, 1.0],
        base,
        0,
    );
    bala(
        world,
        "Bouncy",
        Vec2::new(-7.0, 1.5),
        [0.95, 0.45, 0.20, 1.0],
        ProjectileLaw {
            max_bounces: 4,
            bounciness: 0.85,
            ..base
        },
        0,
    );
    bala(
        world,
        "Arc",
        Vec2::new(-7.0, -1.5),
        [0.35, 0.65, 0.95, 1.0],
        ProjectileLaw {
            gravity: 6.0,
            ..base
        },
        0,
    );
    bala(
        world,
        "Short Range",
        Vec2::new(-7.0, -4.5),
        [0.45, 0.85, 0.45, 1.0],
        ProjectileLaw { range: 6.0, ..base },
        0,
    );
}

/// A cena `=2` — o míssil que persegue, com o controlo ao lado.
fn cena_dois(world: &mut World) {
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [22.0, 14.0], CHAO_RGBA),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    // ⭐⭐ **O alvo ANDA** — um alvo parado não distingue «ele vai até lá» de «ele foi disparado
    // para lá». Ele é um mover de vista de cima, que o artista conduz com as setas.
    world.spawn((
        Name::new("Target"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.4 },
            ..Collider::default()
        },
        Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], CONTROLO_RGBA),
        TopDownPlayer::from_law(TopDownLaw {
            speed: 3.0,
            direction: DirectionMode::EightWay,
            ..TopDownLaw::default()
        }),
        Transform::from_translation(Vec2::new(5.0, 3.0)),
    ));

    // ⚠️ **As duas apontam para o lado ERRADO de propósito** — a `+y`, com o alvo à direita: é a
    // curva que se quer ver, e um tiro já apontado ao alvo não a mostraria.
    let mut recto = Transform::from_translation(Vec2::new(-6.0, -4.0));
    recto.rotation = core::f32::consts::FRAC_PI_2;
    let base = ProjectileLaw {
        initial_speed: 6.0,
        range: 40.0,
        ..ProjectileLaw::default()
    };
    for (nome, cor, homing) in [
        ("Missile", [0.95, 0.30, 0.30, 1.0], 900.0),
        // ⭐⭐ **O CONTROLO, na mesma cena**: tudo igual menos a perseguição. Sem ele o artista não
        // distingue «o homing funciona» de «ele ia para lá de qualquer maneira».
        ("Control (no homing)", CONTROLO_RGBA, 0.0),
    ] {
        let mut t = recto;
        // A segunda nasce um pouco à frente, para as duas se verem.
        if homing == 0.0 {
            t.translation.x += 1.2;
        }
        world.spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.18 },
                ..Collider::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.22], cor),
            ProjectileMotion::from_law(
                ProjectileLaw {
                    homing_accel: homing,
                    ..base
                },
                ph2d_ecs::stable_name_id("Target"),
            ),
            t,
        ));
    }
}

/// **Monta a cena `nivel`** e devolve qual foi — o roteador.
pub fn montar(world: &mut World, nivel: u32) -> u32 {
    match nivel {
        2 => {
            cena_dois(world);
            println!(
                "[projectile-smoke] =2 o missil VERMELHO curva e vai atras do alvo cinzento; o \
                 projectil cinzento (o controlo, sem perseguicao) segue a direito. Ande com o \
                 alvo pelas setas e veja o missil corrigir"
            );
            2
        }
        _ => {
            cena_um(world);
            println!(
                "[projectile-smoke] =1 quatro balas, um knob cada: amarela recta (morre na \
                 parede) · laranja RICOCHETEIA · azul faz ARCO e aponta para onde voa · verde \
                 morre a meio por ALCANCE"
            );
            1
        }
    }
}

#[cfg(test)]
#[path = "projectile_smoke_tests.rs"]
mod tests;
