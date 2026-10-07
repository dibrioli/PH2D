//! ⭐⭐⭐ **Smoke da W18 — a lama nos JOGOS** (plano 30 §27.11). `PH2D_NAV_SMOKE=6`.
//!
//! # A cena: **quatro usos de uma `Nav Cost Area`, lado a lado**
//!
//! Quatro corredores separados por paredes. Em cada um, um corredor VERMELHO anda para a sua bandeira
//! VERDE; a área de custo decide o caminho. Os quatro são o MESMO agente.
//!
//! | corredor | a área | o que tem de acontecer |
//! |---|---|---|
//! | 1 | `Rough Ground` (pedregoso, `Cost 4`) à volta de uma estrada em U (chão normal) | **segue a estrada** em vez de cortar a direito pelas pedras |
//! | 2 | `River` (azul, `Cost 6`) — um rio de lado a lado, menos a `Bridge` | **vai à ponte**: atravessar a água custaria muito mais |
//! | 3 | `Garden` (verde, `Forbidden`) — um canteiro no meio | **dá a volta**: ninguém entra, e não há parede nenhuma |
//! | 4 | `Guard Light` (amarelo, `Cost 8`) — a luz de um guarda | **contorna a luz** pela sombra (furtividade) |
//!
//! ⚠️ O `Rough Ground` vem escolhido: ponha o `Cost` dele em `1` (chão normal) e o corredor 1 corta a
//! direito.
//!
//! ⛔ (medido) A estrada como área BARATA (`Cost 0.3`) não ensina: a área conta a partir do CORPO (recuada
//! pelo raio, a lei da W7), e o caminho mais barato segue a berma por FORA — o corredor andava ao lado da
//! estrada, encostado (`0,25–0,27 m` do centro à estrada). O custo vai no terreno à volta.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavCostArea, NavRegion, NavTarget, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO, Y_CHAO, parede, perseguidor};

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const TERRENO_RGBA: [f32; 4] = [0.50, 0.42, 0.30, 1.0];
const ESTRADA_RGBA: [f32; 4] = [0.07, 0.07, 0.08, 1.0];
const RIO_RGBA: [f32; 4] = [0.22, 0.42, 0.75, 1.0];
const PONTE_RGBA: [f32; 4] = [0.62, 0.46, 0.30, 1.0];
const JARDIM_RGBA: [f32; 4] = [0.28, 0.58, 0.30, 1.0];
const LUZ_RGBA: [f32; 4] = [0.95, 0.88, 0.50, 0.55];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const BANDEIRA_RGBA: [f32; 4] = [0.35, 0.80, 0.45, 1.0];

const MEIA_PAREDE: f32 = 0.25;
const MEIA_DIVISAO: f32 = 0.12;
/// O raio dos corredores (mais pequenos que os da `=5`: os corredores desta cena têm `2,7 m`).
pub const RAIO: f32 = 0.25;
const VELOCIDADE: f32 = 0.8;
/// O `x` das três paredes que separam os quatro corredores.
pub const DIVISOES: [f32; 3] = [-2.9, 0.0, 2.9];
/// Onde nascem e onde estão as bandeiras (o `x` é o meio de cada corredor).
pub const NASCEM_Y: f32 = -1.5;
pub const BANDEIRA_Y: f32 = 3.1;
/// A faixa do meio de cada corredor (rio, canteiro, luz).
pub const MEIO_Y: f32 = 0.8;

pub const CUSTO_TERRENO: f32 = 4.0;
/// A largura da estrada em U (o centro de um corpo de `0,25` anda numa faixa de `0,3`).
pub const ESTRADA: f32 = 0.8;
pub const CUSTO_RIO: f32 = 6.0;
pub const CUSTO_LUZ: f32 = 8.0;
/// A largura da ponte (o vão sem água junto à parede da direita do corredor 2).
pub const PONTE: f32 = 0.8;
pub const RAIO_DA_LUZ: f32 = 0.9;
/// A luz fica à esquerda do meio: a sombra da direita (`0,8 m`) deixa passar um corpo de `0,5`, a da
/// esquerda não (medido: centrada, as duas sombras tinham `0,49 m` e o corredor raspava a luz).
pub const DESVIO_DA_LUZ: f32 = 0.3;

/// As peças da cena `=6`.
pub struct Usos {
    pub terreno: Entity,
    pub rio: Entity,
    pub jardim: Entity,
    pub luz: Entity,
    /// Os corredores 1–4 (estrada, rio, canteiro, luz).
    pub corredores: [Entity; 4],
}

/// Os limites `[x0, x1]` do corredor `k` (0–3).
#[must_use]
pub fn corredor(k: usize) -> [f32; 2] {
    let mx = MEIO_RECINTO[0];
    let bordas = [-mx, DIVISOES[0], DIVISOES[1], DIVISOES[2], mx];
    let x0 = bordas[k] + if k == 0 { 0.0 } else { MEIA_DIVISAO };
    let x1 = bordas[k + 1] - if k == 3 { 0.0 } else { MEIA_DIVISAO };
    [x0, x1]
}

/// O meio do corredor `k`.
#[must_use]
pub fn meio(k: usize) -> f32 {
    let [a, b] = corredor(k);
    0.5 * (a + b)
}

/// O `y` do meio das faixas de baixo e de cima da estrada em U.
fn estrada_y() -> (f32, f32) {
    (
        Y_CHAO + 0.5 * ESTRADA,
        CENTRO[1] + MEIO_RECINTO[1] - 0.5 * ESTRADA,
    )
}

impl Usos {
    /// Os rectângulos (centro, meias-medidas) do terreno, do rio e do canteiro — a régua dos gates. O
    /// terreno é o corredor 1 menos a estrada em U (em baixo, à esquerda, em cima).
    #[must_use]
    pub fn terreno_rect() -> ([f32; 2], [f32; 2]) {
        let [x0, x1] = corredor(0);
        let (y0, y1) = (Y_CHAO + ESTRADA, CENTRO[1] + MEIO_RECINTO[1] - ESTRADA);
        let a = x0 + ESTRADA;
        (
            [0.5 * (a + x1), 0.5 * (y0 + y1)],
            [0.5 * (x1 - a), 0.5 * (y1 - y0)],
        )
    }
    #[must_use]
    pub fn rio_rect() -> ([f32; 2], [f32; 2]) {
        let [x0, x1] = corredor(1);
        let x1 = x1 - PONTE;
        ([0.5 * (x0 + x1), MEIO_Y], [0.5 * (x1 - x0), 0.4])
    }
    #[must_use]
    pub fn jardim_rect() -> ([f32; 2], [f32; 2]) {
        ([meio(2), MEIO_Y], [0.6, 0.6])
    }
    /// O centro da luz.
    #[must_use]
    pub fn luz_centro() -> [f32; 2] {
        [meio(3) - DESVIO_DA_LUZ, MEIO_Y]
    }
}

fn caixa(
    world: &mut World,
    nome: &str,
    r: ([f32; 2], [f32; 2]),
    area: NavCostArea,
    cor: [f32; 4],
) -> Entity {
    let (c, h) = r;
    world
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: h[0],
                    half_y: h[1],
                },
                is_sensor: true,
                ..Collider::default()
            },
            area,
            Sprite::atlas(WHITE_TILE_KEY, [h[0] * 2.0, h[1] * 2.0], cor),
            Transform::from_translation(Vec2::new(c[0], c[1])),
        ))
        .id()
}

/// Um desenho sem corpo (a estrada, a ponte, as bandeiras).
fn desenho(world: &mut World, nome: &str, c: [f32; 2], tam: [f32; 2], cor: [f32; 4]) {
    world.spawn((
        Name::new(nome),
        Sprite::atlas(WHITE_TILE_KEY, tam, cor),
        Transform::from_translation(Vec2::new(c[0], c[1])),
    ));
}

fn custo(cost: f32) -> NavCostArea {
    NavCostArea {
        cost,
        forbidden: false,
    }
}

/// **Monta a cena `=6`.**
pub fn montar(world: &mut World) -> Usos {
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
    let mut paredes = vec![
        ("Wall N".to_owned(), [cx, topo + e], [mx + 2.0 * e, e]),
        ("Wall S".to_owned(), [cx, Y_CHAO - e], [mx + 2.0 * e, e]),
        ("Wall W".to_owned(), [cx - mx - e, cy], [e, my]),
        ("Wall E".to_owned(), [cx + mx + e, cy], [e, my]),
    ];
    for (k, x) in DIVISOES.iter().enumerate() {
        paredes.push((format!("Wall {}", k + 1), [*x, cy], [MEIA_DIVISAO, my]));
    }
    for (nome, centro, meio) in &paredes {
        parede(
            world,
            nome,
            Vec2::new(centro[0], centro[1]),
            Vec2::new(meio[0], meio[1]),
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
    // 1 — a estrada em U é o CHÃO (custo `1`, só o desenho); o terreno à volta é que custa.
    let [x0, x1] = corredor(0);
    let (yb, yt) = estrada_y();
    let (largo, alto) = ([x1 - x0, ESTRADA], [ESTRADA, yt - yb]);
    desenho(world, "Road", [0.5 * (x0 + x1), yb], largo, ESTRADA_RGBA);
    desenho(
        world,
        "Road",
        [x0 + 0.5 * ESTRADA, 0.5 * (yb + yt)],
        alto,
        ESTRADA_RGBA,
    );
    desenho(world, "Road", [0.5 * (x0 + x1), yt], largo, ESTRADA_RGBA);
    let terreno = caixa(
        world,
        "Rough Ground",
        Usos::terreno_rect(),
        custo(CUSTO_TERRENO),
        TERRENO_RGBA,
    );
    // 2 — o rio, e a ponte: só o desenho do vão sem água (o chão).
    let rio = caixa(world, "River", Usos::rio_rect(), custo(CUSTO_RIO), RIO_RGBA);
    let [_, r1] = corredor(1);
    desenho(
        world,
        "Bridge",
        [r1 - 0.5 * PONTE, MEIO_Y],
        [PONTE, 0.8],
        PONTE_RGBA,
    );
    // 3 — o canteiro proibido.
    let jardim = caixa(
        world,
        "Garden",
        Usos::jardim_rect(),
        NavCostArea {
            cost: 1.0,
            forbidden: true,
        },
        JARDIM_RGBA,
    );
    // 4 — a luz do guarda.
    let lc = Usos::luz_centro();
    let luz = world
        .spawn((
            Name::new("Guard Light"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Ball {
                    radius: RAIO_DA_LUZ,
                },
                is_sensor: true,
                ..Collider::default()
            },
            custo(CUSTO_LUZ),
            crate::smoke_desenho::disco(RAIO_DA_LUZ, LUZ_RGBA),
            Transform::from_translation(Vec2::new(lc[0], lc[1])),
        ))
        .id();
    let nomes = ["Road", "River", "Garden", "Light"];
    // O corredor 1 nasce e chega NA estrada (em baixo e em cima do U).
    let ys = |k: usize| {
        if k == 0 {
            (yb, yt)
        } else {
            (NASCEM_Y, BANDEIRA_Y)
        }
    };
    let corredores = std::array::from_fn(|k| {
        let (x, (y0, y1)) = (meio(k), ys(k));
        let bandeira = format!("Flag {}", nomes[k]);
        desenho(world, &bandeira, [x, y1], [0.3, 0.3], BANDEIRA_RGBA);
        let r = perseguidor(
            world,
            &format!("Runner {}", nomes[k]),
            Vec2::new(x, y0),
            (RAIO, 0.0),
            VELOCIDADE,
            VERMELHO_RGBA,
            ("", ""),
        );
        if let Some(mut a) = world.get_mut::<NavAgent>(r) {
            (a.target, a.arrive_distance) = (NavTarget::Point([x, y1]), 0.2);
        }
        r
    });
    Usos {
        terreno,
        rio,
        jardim,
        luz,
        corredores,
    }
}

/// A linha do terminal da cena.
pub fn anuncia() {
    println!(
        "[nav-smoke] =6 a LAMA NOS JOGOS: quatro corredores, o mesmo agente em cada um. 1 a ESTRADA: o \
         terreno a' volta e' pedregoso (Rough Ground, Cost 4) e o vermelho segue a estrada em U. 2 o RIO \
         (Cost 6): vai a' PONTE. 3 o CANTEIRO (Forbidden): da' a volta, sem parede nenhuma. 4 a LUZ DO \
         GUARDA (Cost 8): contorna-a pela sombra. O Rough Ground esta' escolhido: ponha o Cost dele em 1 \
         e o corredor 1 corta a direito"
    );
}

#[cfg(test)]
#[path = "nav_smoke_usos_tests.rs"]
mod tests;
