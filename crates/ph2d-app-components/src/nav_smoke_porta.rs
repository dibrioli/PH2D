//! ⭐⭐⭐ **Smoke do DESVIO** (plano 30, W5). `PH2D_NAV_SMOKE=2`.
//!
//! # A cena: **a porta de dois sentidos, e o CONTROLO por baixo**
//!
//! Duas faixas iguais, uma por cima da outra, cada uma com uma parede ao meio e UMA porta. Em cada
//! faixa quatro corpos vão da esquerda para a direita e quatro da direita para a esquerda — todos pela
//! mesma porta, ao mesmo tempo.
//!
//! | faixa | quem | o que tem de acontecer |
//! |---|---|---|
//! | de CIMA | VERMELHOS, com *Avoid Others* | **cruzam-se na porta** (cada um passa pela sua direita) e chegam todos ao outro lado |
//! | de BAIXO (o CONTROLO) | CINZENTOS, o mesmo agente SEM o desvio | **entalam-se na porta** de frente uns para os outros e ficam |
//!
//! ⚠️ **A única diferença entre as faixas é a caixa *Avoid Others*** — o raio, a velocidade, a porta e
//! os alvos são os mesmos. Medido antes de a mandar: porque uma porta de UM sentido não ensina nada —
//! sem o desvio oito corpos também a passam, só mais roçados (`349` tiques contra `343`); é o
//! frente-a-frente que entala (`tests/it/nav_desvio.rs`).
//!
//! ⚠️ Se a linha `[nav-smoke] =2` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, RigidBody, TopDownPlayer,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO};

const PAREDE_RGBA: [f32; 4] = [0.38, 0.40, 0.46, 1.0];
const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const CINZENTO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

const MEIA_PAREDE: f32 = 0.25;
/// O raio de cada corpo.
pub const RAIO: f32 = 0.25;
/// A largura da porta: dois corpos lado a lado com folga (`4·r = 1,0 < 1,4`) — o desvio TEM espaço
/// para os cruzar, e o controlo entala por não o usar, nunca por falta de lugar.
pub const PORTA: f32 = 1.4;
const VELOCIDADE: f32 = 2.0;

const _: () = assert!(4.0 * RAIO < PORTA);

/// As peças da cena `=2`.
pub struct Porta {
    /// De cima, com o desvio — os quatro primeiros vão para a direita.
    pub vermelhos: [Entity; 8],
    /// De baixo, sem o desvio (o CONTROLO) — a mesma ordem.
    pub cinzentos: [Entity; 8],
    /// Onde cada um tem de chegar: `[vermelhos, cinzentos]`, pela mesma ordem.
    pub alvos: [[[f32; 2]; 8]; 2],
    /// O `y` do meio de cada faixa (cima, baixo).
    pub faixas: [f32; 2],
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

fn corpo(world: &mut World, nome: &str, em: [f32; 2], alvo: [f32; 2], desvio: bool) -> Entity {
    let cor = if desvio { VERMELHO_RGBA } else { CINZENTO_RGBA };
    world
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: RAIO },
                ..Collider::default()
            },
            crate::smoke_desenho::disco(RAIO, cor),
            TopDownPlayer::from_law(TopDownLaw {
                speed: VELOCIDADE,
                direction: DirectionMode::Free,
                default_controls: false,
                ..TopDownLaw::default()
            }),
            NavAgent {
                target: NavTarget::Point(alvo),
                arrive_distance: 0.05,
                avoidance: desvio,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em[0], em[1])),
        ))
        .id()
}

/// Os oito lugares de uma faixa, à volta de `yc`: quatro à esquerda e o espelho deles à direita.
fn lugares(yc: f32) -> [[f32; 2]; 8] {
    let mut out = [[0.0; 2]; 8];
    let col = [-4.6, -3.7];
    let lin = [-0.45, 0.45];
    let mut k = 0;
    for x in col {
        for dy in lin {
            out[k] = [x, yc + dy];
            out[k + 4] = [-x, yc - dy];
            k += 1;
        }
    }
    out
}

/// **Monta a cena `=2`.**
pub fn montar(world: &mut World) -> Porta {
    let [mx, my] = MEIO_RECINTO;
    let [cx, cy] = CENTRO;
    let c = Vec2::new(cx, cy);
    let e = MEIA_PAREDE;
    let topo = cy + my;
    let chao = cy - my;
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [mx * 2.0, my * 2.0], CHAO_RGBA),
        Transform::from_translation(c),
    ));
    parede(
        world,
        "Wall N",
        Vec2::new(cx, topo + e),
        Vec2::new(mx + 2.0 * e, e),
    );
    parede(
        world,
        "Wall S",
        Vec2::new(cx, chao - e),
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
    // A divisória que faz as duas faixas.
    parede(world, "Divider", c, Vec2::new(mx, e));
    // As duas faixas e a parede-com-porta de cada uma.
    let cima = (cy + e + topo) * 0.5;
    let baixo = (chao + cy - e) * 0.5;
    for (faixa, y0, y1) in [("top", cy + e, topo), ("bottom", chao, cy - e)] {
        let meio = (y0 + y1) * 0.5;
        let (a0, a1) = (meio - PORTA * 0.5, meio + PORTA * 0.5);
        let (baixo_da_porta, cima_da_porta) = (
            format!("Door Wall ({faixa}, below)"),
            format!("Door Wall ({faixa}, above)"),
        );
        parede(
            world,
            &baixo_da_porta,
            Vec2::new(cx, (y0 + a0) * 0.5),
            Vec2::new(e, (a0 - y0) * 0.5),
        );
        parede(
            world,
            &cima_da_porta,
            Vec2::new(cx, (a1 + y1) * 0.5),
            Vec2::new(e, (y1 - a1) * 0.5),
        );
    }
    world.spawn((
        Name::new("Nav Region"),
        NavRegion {
            half_extents: MEIO_RECINTO,
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(c),
    ));

    let mut vermelhos = [Entity::PLACEHOLDER; 8];
    let mut cinzentos = [Entity::PLACEHOLDER; 8];
    let mut alvos = [[[0.0; 2]; 8]; 2];
    let (lc, lb) = (lugares(cima), lugares(baixo));
    for k in 0..8 {
        // Os quatro primeiros começam à esquerda e acabam no lugar espelhado à direita, e vice-versa.
        let destino = (k + 4) % 8;
        vermelhos[k] = corpo(world, &format!("Red {}", k + 1), lc[k], lc[destino], true);
        cinzentos[k] = corpo(world, &format!("Grey {}", k + 1), lb[k], lb[destino], false);
        alvos[0][k] = lc[destino];
        alvos[1][k] = lb[destino];
    }
    Porta {
        vermelhos,
        cinzentos,
        alvos,
        faixas: [cima, baixo],
    }
}

#[cfg(test)]
#[path = "nav_smoke_porta_tests.rs"]
mod tests;
