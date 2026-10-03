//! ⭐⭐⭐ **Smoke da W7 — a LAVA e o PORTAL** (plano 30). `PH2D_NAV_SMOKE=4`.
//!
//! # A cena: **um rio de lava, dois portais, e o CONTROLO ao lado**
//!
//! Um rio de lava (LARANJA) atravessa o recinto de baixo até quase em cima — só fica uma passagem
//! estreita junto à parede de cima. Em baixo, um portal AZUL de cada lado: quem entra no da esquerda
//! sai no da direita. O herói AMARELO está do lado direito e anda com as setas.
//!
//! | quem | o quê | o que tem de acontecer |
//! |---|---|---|
//! | VERMELHO | um inimigo com vida, *Avoid Harm* ligado | **não pisa a lava**: vai ao portal da esquerda, SALTA para o da direita e apanha o herói (a volta por cima é mais longa: `8,2 m` contra `5,5`) |
//! | CINZENTO (o CONTROLO) | o MESMO inimigo, *Avoid Harm* desligado | **atravessa a lava a direito** (para ele é o caminho mais curto) e a barra de vida desce |
//!
//! ⚠️ **A única diferença entre os dois é a caixa *Avoid Harm*** — a vida, o raio e a velocidade
//! são os mesmos. O VERMELHO vem escolhido: no Inspector, tire-lhe o visto e ele passa a atravessar
//! a lava como o cinzento; escolha o portal da esquerda e tire o *Nav Link* — ele dá a volta por cima.
//!
//! ⚠️ Se a linha `[nav-smoke] =4` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Damage, Health, HealthBar, NavAgent, NavLink, NavRegion,
    RigidBody, TopDownPlayer,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

use crate::nav_smoke::{
    CENTRO, MEIO_RECINTO, RAIO_HEROI, RAIO_PEQUENO, Y_CHAO, parede, perseguidor,
};

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const LAVA_RGBA: [f32; 4] = [0.95, 0.42, 0.12, 1.0];
const PORTAL_RGBA: [f32; 4] = [0.30, 0.65, 0.95, 1.0];
const HEROI_RGBA: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const CINZENTO_RGBA: [f32; 4] = [0.55, 0.57, 0.60, 1.0];

const MEIA_PAREDE: f32 = 0.25;
/// A meia-largura do rio de lava.
pub const LAVA_MEIA: f32 = 0.6;
/// O `y` do topo do rio: a passagem de cima fica entre ele e a parede (`1,2 m`; o centro de um corpo
/// de raio `0,35` passa numa faixa de `0,5 m`).
pub const LAVA_TOPO: f32 = 2.4;
/// O dano da lava, por segundo, e o tipo (o nome que as resistências procuram).
pub const LAVA_DANO: f32 = 25.0;
pub const LAVA_TIPO: &str = "fire";
/// Os portais: a entrada (esquerda) e a saída (direita).
pub const PORTAL_A: [f32; 2] = [-4.8, -1.4];
pub const PORTAL_B: [f32; 2] = [4.8, -1.4];
/// Onde cada um começa.
pub const HEROI_EM: [f32; 2] = [2.5, 0.5];
pub const VERMELHO_EM: [f32; 2] = [-2.5, -0.5];
pub const CINZENTO_EM: [f32; 2] = [-2.5, 1.5];
const VELOCIDADE: f32 = 2.5;
const VIDA: f32 = 100.0;

/// As peças da cena `=4`.
pub struct Lava {
    pub vermelho: Entity,
    pub cinzento: Entity,
    pub heroi: Entity,
    pub lava: Entity,
    pub portal_a: Entity,
    pub portal_b: Entity,
}

impl Lava {
    /// O rectângulo do rio (centro, meias-medidas) — a régua dos gates.
    #[must_use]
    pub fn rio() -> ([f32; 2], [f32; 2]) {
        let meio_y = (LAVA_TOPO - Y_CHAO) * 0.5;
        ([0.0, Y_CHAO + meio_y], [LAVA_MEIA, meio_y])
    }
}

fn marca(world: &mut World, nome: &str, em: [f32; 2], cor: [f32; 4]) -> Entity {
    world
        .spawn((
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], cor),
            Transform::from_translation(Vec2::new(em[0], em[1])),
        ))
        .id()
}

/// Um inimigo da cena: o perseguidor do labirinto, com vida e barra, e a caixa *Avoid Harm*.
fn inimigo(world: &mut World, nome: &str, em: [f32; 2], cor: [f32; 4], evita: bool) -> Entity {
    let e = perseguidor(
        world,
        nome,
        Vec2::new(em[0], em[1]),
        RAIO_PEQUENO,
        VELOCIDADE,
        cor,
        ("caught you", ""),
    );
    world.entity_mut(e).insert((
        Health {
            max: VIDA,
            start: VIDA,
            ..Health::default()
        },
        HealthBar {
            offset_y: RAIO_PEQUENO + 0.25,
            ..HealthBar::default()
        },
    ));
    if let Some(mut a) = world.get_mut::<NavAgent>(e) {
        a.avoid_harm = evita;
    }
    e
}

/// **Monta a cena `=4`.**
pub fn montar(world: &mut World) -> Lava {
    let [mx, my] = MEIO_RECINTO;
    let [cx, cy] = CENTRO;
    let c = Vec2::new(cx, cy);
    world.spawn((
        Name::new("Floor"),
        Sprite::atlas(WHITE_TILE_KEY, [mx * 2.0, my * 2.0], CHAO_RGBA),
        Transform::from_translation(c),
    ));
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
    world.spawn((
        Name::new("Nav Region"),
        NavRegion {
            half_extents: MEIO_RECINTO,
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(c),
    ));
    // ⭐ A LAVA: um sensor (atravessa-se) que fere por segundo — o mesmo `Damage` da Vida e Dano.
    let (rc, rh) = Lava::rio();
    let lava = world
        .spawn((
            Name::new("Lava"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: rh[0],
                    half_y: rh[1],
                },
                is_sensor: true,
                ..Collider::default()
            },
            Damage {
                amount: LAVA_DANO,
                per_second: true,
                kind: LAVA_TIPO.to_owned(),
                ..Damage::default()
            },
            Sprite::atlas(WHITE_TILE_KEY, [rh[0] * 2.0, rh[1] * 2.0], LAVA_RGBA),
            Transform::from_translation(Vec2::new(rc[0], rc[1])),
        ))
        .id();
    // ⭐ O PORTAL: a entrada leva o atalho, a saída é só um nome.
    let portal_a = marca(world, "Portal A", PORTAL_A, PORTAL_RGBA);
    let portal_b = marca(world, "Portal B", PORTAL_B, PORTAL_RGBA);
    world.entity_mut(portal_a).insert(NavLink {
        to: stable_name_id("Portal B"),
        teleport: true,
        on_crossed: "through the portal".to_owned(),
        ..NavLink::default()
    });
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
            Transform::from_translation(Vec2::new(HEROI_EM[0], HEROI_EM[1])),
        ))
        .id();
    let vermelho = inimigo(world, "Red", VERMELHO_EM, VERMELHO_RGBA, true);
    let cinzento = inimigo(
        world,
        "Control (no Avoid Harm)",
        CINZENTO_EM,
        CINZENTO_RGBA,
        false,
    );
    Lava {
        vermelho,
        cinzento,
        heroi,
        lava,
        portal_a,
        portal_b,
    }
}

/// A linha do terminal da cena.
pub fn anuncia() {
    println!(
        "[nav-smoke] =4 a LAVA e o PORTAL. O VERMELHO (Avoid Harm ligado) nao pisa a lava LARANJA: \
         vai ao portal AZUL da esquerda, salta para o da direita e apanha-te. O CINZENTO (o controlo: \
         o mesmo inimigo com Avoid Harm desligado) atravessa a lava a direito e a barra de vida dele \
         desce. Setas para andar com o AMARELO. O VERMELHO esta' escolhido: tire-lhe o visto de Avoid \
         Harm no Inspector e ele passa a atravessar a lava; escolha o Portal A e apague o Nav Link - \
         ele da' a volta por cima"
    );
}

#[cfg(test)]
#[path = "nav_smoke_lava_tests.rs"]
mod tests;
