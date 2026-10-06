//! ⭐⭐⭐ **Smoke da W18 — a LAMA** (plano 30 §27.1). `PH2D_NAV_SMOKE=5`.
//!
//! # A cena: **duas pistas iguais, uma lama leve e uma pesada**
//!
//! Uma parede ao meio divide o recinto em duas pistas. Em cada pista, três corredores VERMELHOS nascem
//! em baixo e andam cada um para a sua bandeira VERDE, por cima de uma faixa de lama que atravessa a
//! pista quase toda — só fica uma passagem livre junto à parede de fora. As duas faixas têm as MESMAS
//! medidas; muda só o custo.
//!
//! | pista | lama | o que tem de acontecer |
//! |---|---|---|
//! | esquerda | LEVE (castanho claro, `Cost 2`) | os três **atravessam-na a direito**: a volta pela passagem é mais longa que o que a lama custa |
//! | direita | PESADA (castanho escuro, `Cost 10`) | os três **dão a volta** pela passagem da direita |
//!
//! ⚠️ **A lama não atrasa o corpo** — diz a cada agente quanto vale um metro ali na hora de ESCOLHER o
//! caminho. Os seis são o MESMO agente; a pista da esquerda é o controlo da da direita.
//!
//! ⚠️ A lama pesada vem escolhida: com o relógio parado (`Espaço`), ponha o `Cost` dela em `2` no
//! Inspector e solte o relógio — os da direita viram e cortam A DIREITO pela lama (mudar o custo refaz
//! o caminho de quem anda).
//!
//! ⛔ (medido) Um herói e um perseguidor nesta cena: os seis cercavam o herói e prendiam-no; e o
//! perseguidor, ao passar pelas bandeiras, empurrava um corredor parado para a borda da lama.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavCostArea, NavRegion, NavTarget, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO, RAIO_PEQUENO, Y_CHAO, parede, perseguidor};

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const LEVE_RGBA: [f32; 4] = [0.68, 0.53, 0.34, 1.0];
const PESADA_RGBA: [f32; 4] = [0.40, 0.27, 0.16, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const BANDEIRA_RGBA: [f32; 4] = [0.35, 0.80, 0.45, 1.0];

const MEIA_PAREDE: f32 = 0.25;
const MEIA_DIVISAO: f32 = 0.12;
/// Onde a parede do meio acaba: acima dela as duas pistas são uma sala só, a do herói.
pub const DIVISAO_TOPO: f32 = 2.0;
/// A faixa de lama: `y ∈ [LAMA_Y0, LAMA_Y1]`, da parede do meio até `PASSAGEM` da parede de fora.
pub const LAMA_Y0: f32 = 0.0;
pub const LAMA_Y1: f32 = 0.8;
/// A passagem livre junto à parede de fora — larga para TRÊS corpos de `0,35` lado a lado (com `1,2`
/// os três empurravam-se nela e um raspava a ponta da lama: medido, `2` «atravessaram»).
pub const PASSAGEM: f32 = 1.6;
pub const CUSTO_LEVE: f32 = 2.0;
pub const CUSTO_PESADA: f32 = 10.0;
/// O `x` (pista da direita; a da esquerda é o espelho) e o `y` de onde nascem os corredores…
pub const NASCEM_X: [f32; 3] = [1.6, 2.4, 3.2];
pub const NASCEM_Y: f32 = -1.3;
/// …e as bandeiras, por cima da lama. Na pista da direita a ordem inverte-se: quem sai da passagem
/// PRIMEIRO vai à mais funda (medido: com a mesma ordem, o 1.º a chegar estacionava no caminho dos
/// outros e o desvio deles empurrava um para a borda da lama).
pub const BANDEIRAS_X: [f32; 3] = [1.6, 2.4, 3.2];
pub const BANDEIRAS_Y: f32 = 1.5;
/// Devagar: a travessia da esquerda leva `≈ 4 s` e a volta da direita `≈ 7 s` — o dono vê-as, e vê-os
/// virar quando muda o `Cost` (a `1,2 m/s` a esquerda já tinha atravessado quando a janela abria).
const VELOCIDADE: f32 = 0.8;

/// As peças da cena `=5`.
pub struct Lama {
    pub leve: Entity,
    pub pesada: Entity,
    /// Os corredores da pista da LEVE (esquerda) e os da PESADA (direita).
    pub na_leve: [Entity; 3],
    pub na_pesada: [Entity; 3],
}

impl Lama {
    /// O rectângulo de uma faixa (centro, meias-medidas): `lado = -1` a leve, `+1` a pesada.
    #[must_use]
    pub fn faixa(lado: f32) -> ([f32; 2], [f32; 2]) {
        let x0 = MEIA_DIVISAO;
        let x1 = MEIO_RECINTO[0] - PASSAGEM;
        let meio = [(x1 - x0) * 0.5, (LAMA_Y1 - LAMA_Y0) * 0.5];
        ([lado * (x0 + meio[0]), LAMA_Y0 + meio[1]], meio)
    }
}

fn faixa(world: &mut World, nome: &str, lado: f32, custo: f32, cor: [f32; 4]) -> Entity {
    let (c, h) = Lama::faixa(lado);
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
            NavCostArea {
                cost: custo,
                forbidden: false,
            },
            Sprite::atlas(WHITE_TILE_KEY, [h[0] * 2.0, h[1] * 2.0], cor),
            Transform::from_translation(Vec2::new(c[0], c[1])),
        ))
        .id()
}

/// **Monta a cena `=5`.**
pub fn montar(world: &mut World) -> Lama {
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
    let bordas = [
        ("Wall N", [cx, topo + e], [mx + 2.0 * e, e]),
        ("Wall S", [cx, Y_CHAO - e], [mx + 2.0 * e, e]),
        ("Wall W", [cx - mx - e, cy], [e, my]),
        ("Wall E", [cx + mx + e, cy], [e, my]),
        (
            "Wall Middle",
            [0.0, (Y_CHAO + DIVISAO_TOPO) * 0.5],
            [MEIA_DIVISAO, (DIVISAO_TOPO - Y_CHAO) * 0.5],
        ),
    ];
    for (nome, centro, meio) in bordas {
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
    let leve = faixa(world, "Light Mud", -1.0, CUSTO_LEVE, LEVE_RGBA);
    let pesada = faixa(world, "Heavy Mud", 1.0, CUSTO_PESADA, PESADA_RGBA);
    let pista = |world: &mut World, lado: f32, letra: char| {
        let mut k = 0;
        NASCEM_X.map(|x| {
            let b = [
                lado * BANDEIRAS_X[if lado > 0.0 { 2 - k } else { k }],
                BANDEIRAS_Y,
            ];
            k += 1;
            world.spawn((
                Name::new(format!("Flag {letra}{k}")),
                Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], BANDEIRA_RGBA),
                Transform::from_translation(Vec2::new(b[0], b[1])),
            ));
            let e = perseguidor(
                world,
                &format!("Runner {letra}{k}"),
                Vec2::new(lado * x, NASCEM_Y),
                (RAIO_PEQUENO, 0.0),
                VELOCIDADE,
                VERMELHO_RGBA,
                ("", ""),
            );
            if let Some(mut a) = world.get_mut::<NavAgent>(e) {
                (a.target, a.arrive_distance) = (NavTarget::Point(b), 0.2);
            }
            e
        })
    };
    let na_leve = pista(world, -1.0, 'L');
    let na_pesada = pista(world, 1.0, 'R');
    Lama {
        leve,
        pesada,
        na_leve,
        na_pesada,
    }
}

/// A linha do terminal da cena.
pub fn anuncia() {
    println!(
        "[nav-smoke] =5 a LAMA. Duas pistas iguais: a da esquerda tem lama LEVE (castanho claro, Cost 2), \
         a da direita lama PESADA (castanho escuro, Cost 10). Cada VERMELHO anda para a sua bandeira \
         VERDE: os tres da esquerda atravessam a lama leve a direito, os tres da direita dao a volta pela \
         passagem junto a parede. A lama nao atrasa o corpo: diz quanto vale cada metro na hora de \
         escolher o caminho. A lama pesada esta' escolhida: pare o relogio (Espaco), ponha o Cost dela \
         em 2 no Inspector, solte o relogio, e os da direita cortam pela lama"
    );
}

#[cfg(test)]
#[path = "nav_smoke_lama_tests.rs"]
mod tests;
