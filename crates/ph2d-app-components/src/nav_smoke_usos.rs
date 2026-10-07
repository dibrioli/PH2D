//! ⭐⭐⭐ **Smoke da W18 — as áreas de custo nos JOGOS** (plano 30 §27.11). `PH2D_NAV_SMOKE=6`.
//!
//! # A cena: **quatro usos de uma `Nav Cost Area`, lado a lado**
//!
//! Quatro corredores separados por paredes. Em cada um, um corredor VERMELHO anda para a sua bandeira
//! VERDE; a área de custo decide o caminho. Os quatro são o MESMO agente.
//!
//! | corredor | a área | o que tem de acontecer |
//! |---|---|---|
//! | 1 | `Road` (escura, `Cost 0.3`) — uma estrada em U, mais BARATA que o chão | **segue a estrada, com o corpo por cima dela**, em vez de cortar a direito |
//! | 2 | `River` (azul, `Cost 6`) — um rio de lado a lado, menos a `Bridge` | **vai à ponte**: atravessar a água custaria muito mais |
//! | 3 | `Garden` (verde, `Forbidden`) — um canteiro no meio | **dá a volta**: ninguém entra, e não há parede nenhuma |
//! | 4 | `Guard Light` (amarelo, `Cost 8`) — a luz de um guarda | **contorna a luz** pela sombra (furtividade) |
//!
//! ⚠️ A `Road` comprida (a da esquerda) vem escolhida: ponha o `Cost` dela em `1` e o corredor 1 corta a
//! direito pelo meio.
//!
//! ⭐ A estrada barata só ensina com a lei da W18 (`ph2d_navmesh::Area::dentro`): uma área mais barata que
//! o chão recua para DENTRO, e o desconto vale com o corpo inteiro nela. Antes (a área recuada para fora,
//! como a lama) o corredor seguia a berma POR FORA, a `0,25–0,27 m` dela (plano 30 §27.11).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavCostArea, NavRegion, NavTarget, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO, Y_CHAO, parede, perseguidor};

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
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

pub const CUSTO_ESTRADA: f32 = 0.3;
/// A largura da estrada em U (o centro de um corpo de `0,25` inteiro nela anda numa faixa de `0,3`).
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
    /// A tira comprida da estrada (a da esquerda) — a escolhida.
    pub estrada: Entity,
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
    /// As três tiras da estrada em U (centro, meias-medidas): a de baixo, a da esquerda (da parede de
    /// baixo à de cima — as tiras encolhidas pelo raio tocam-se nos cantos) e a de cima.
    #[must_use]
    pub fn estrada_rects() -> [([f32; 2], [f32; 2]); 3] {
        let [x0, x1] = corredor(0);
        let (yb, yt) = estrada_y();
        let (topo, h) = (CENTRO[1] + MEIO_RECINTO[1], 0.5 * ESTRADA);
        [
            ([0.5 * (x0 + x1), yb], [0.5 * (x1 - x0), h]),
            ([x0 + h, 0.5 * (Y_CHAO + topo)], [h, 0.5 * (topo - Y_CHAO)]),
            ([0.5 * (x0 + x1), yt], [0.5 * (x1 - x0), h]),
        ]
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

pub(crate) fn caixa(
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
pub(crate) fn desenho(world: &mut World, nome: &str, c: [f32; 2], tam: [f32; 2], cor: [f32; 4]) {
    world.spawn((
        Name::new(nome),
        Sprite::atlas(WHITE_TILE_KEY, tam, cor),
        Transform::from_translation(Vec2::new(c[0], c[1])),
    ));
}

pub(crate) fn custo(cost: f32) -> NavCostArea {
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
    // 1 — a estrada em U, mais BARATA que o chão.
    let [baixo, comprida, cima] = Usos::estrada_rects();
    caixa(
        world,
        "Road Bottom",
        baixo,
        custo(CUSTO_ESTRADA),
        ESTRADA_RGBA,
    );
    let estrada = caixa(world, "Road", comprida, custo(CUSTO_ESTRADA), ESTRADA_RGBA);
    caixa(world, "Road Top", cima, custo(CUSTO_ESTRADA), ESTRADA_RGBA);
    let (yb, yt) = estrada_y();
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
        estrada,
        rio,
        jardim,
        luz,
        corredores,
    }
}

/// A linha do terminal da cena.
pub fn anuncia() {
    println!(
        "[nav-smoke] =6 as AREAS DE CUSTO NOS JOGOS: quatro corredores, o mesmo agente em cada um. 1 a \
         ESTRADA (Road, Cost 0.3, mais barata que o chao): o vermelho segue-a por cima. 2 o RIO (Cost 6): \
         vai a' PONTE. 3 o CANTEIRO (Forbidden): da' a volta, sem parede nenhuma. 4 a LUZ DO GUARDA \
         (Cost 8): contorna-a pela sombra. A Road da esquerda esta' escolhida: ponha o Cost dela em 1 e o \
         corredor 1 corta a direito"
    );
}

#[cfg(test)]
#[path = "nav_smoke_usos_tests.rs"]
mod tests;
