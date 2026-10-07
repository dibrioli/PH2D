//! ⭐⭐ **Smoke da W19 — a área BARATA mais estreita que o corpo** (plano 30 §28.3). `PH2D_NAV_SMOKE=7`.
//!
//! # A cena: **a mesma estrada em U, larga e estreita, lado a lado**
//!
//! Dois corredores separados por uma parede, o mesmo agente VERMELHO em cada um (`r = 0,25`), a sua
//! bandeira em cima. Em cada corredor uma estrada em U (`Cost 0.3`, mais barata que o chão):
//!
//! | corredor | a estrada | o que tem de acontecer |
//! |---|---|---|
//! | esquerda | `Wide Road` — `0,8 m`, cabe o corpo (`0,5 m`) | **segue a estrada**, com o corpo por cima dela |
//! | direita | `Narrow Road` — `0,4 m`, mais estreita que o corpo | **vai a direito**: nenhum corpo cabe inteiro nela, e ela não faz nada |
//!
//! ⚠️ A `Narrow Road` comprida vem escolhida: o Inspector diz porquê, na secção `Nav Cost Area`
//! (*«Narrower than the body that walks it (radius 0.25 m)…»*, `NavCostAreaNow`, publicado pela ponte).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{NavAgent, NavRegion, NavTarget};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::nav_smoke::{CENTRO, MEIO_RECINTO, Y_CHAO, parede, perseguidor};
use crate::nav_smoke_usos::{CUSTO_ESTRADA, ESTRADA, RAIO, caixa, custo, desenho};

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const ESTRADA_RGBA: [f32; 4] = [0.07, 0.07, 0.08, 1.0];
const VERMELHO_RGBA: [f32; 4] = [0.90, 0.30, 0.28, 1.0];
const BANDEIRA_RGBA: [f32; 4] = [0.35, 0.80, 0.45, 1.0];
const MEIA_PAREDE: f32 = 0.25;
const MEIA_DIVISAO: f32 = 0.12;
const VELOCIDADE: f32 = 0.8;
/// A estrada estreita: mais estreita que o corpo (`2 · RAIO`).
pub const ESTREITA: f32 = 0.4;
/// Onde nascem e chegam (acima da faixa de baixo, abaixo da de cima, nos dois corredores).
pub const FOLGA_Y: f32 = 0.5;

const _: () = assert!(ESTREITA < 2.0 * RAIO && ESTRADA > 2.0 * RAIO);

/// As peças da cena `=7`.
pub struct Estreita {
    /// A tira comprida da estrada estreita — a escolhida.
    pub estreita: Entity,
    /// A tira comprida da larga.
    pub larga: Entity,
    /// Os corredores: esquerda (larga), direita (estreita).
    pub corredores: [Entity; 2],
}

/// Os limites `[x0, x1]` do corredor `k` (`0` esquerda, `1` direita).
#[must_use]
pub fn corredor(k: usize) -> [f32; 2] {
    let mx = MEIO_RECINTO[0];
    let c = CENTRO[0];
    if k == 0 {
        [c - mx, c - MEIA_DIVISAO]
    } else {
        [c + MEIA_DIVISAO, c + mx]
    }
}

/// As três tiras do U do corredor `k` (centro, meias-medidas) com a largura `l`: a de baixo, a da
/// esquerda (da parede de baixo à de cima) e a de cima.
#[must_use]
pub fn estrada_rects(k: usize, l: f32) -> [([f32; 2], [f32; 2]); 3] {
    let [x0, x1] = corredor(k);
    let topo = CENTRO[1] + MEIO_RECINTO[1];
    let h = 0.5 * l;
    [
        ([0.5 * (x0 + x1), Y_CHAO + h], [0.5 * (x1 - x0), h]),
        ([x0 + h, 0.5 * (Y_CHAO + topo)], [h, 0.5 * (topo - Y_CHAO)]),
        ([0.5 * (x0 + x1), topo - h], [0.5 * (x1 - x0), h]),
    ]
}

/// A largura da estrada do corredor `k`.
#[must_use]
pub const fn largura(k: usize) -> f32 {
    if k == 0 { ESTRADA } else { ESTREITA }
}

/// **Monta a cena `=7`.**
pub fn montar(world: &mut World) -> Estreita {
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
    for (nome, centro, meio) in [
        ("Wall N", [cx, topo + e], [mx + 2.0 * e, e]),
        ("Wall S", [cx, Y_CHAO - e], [mx + 2.0 * e, e]),
        ("Wall W", [cx - mx - e, cy], [e, my]),
        ("Wall E", [cx + mx + e, cy], [e, my]),
        ("Wall Middle", [cx, cy], [MEIA_DIVISAO, my]),
    ] {
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
    let nomes = ["Wide Road", "Narrow Road"];
    let compridas: [Entity; 2] = std::array::from_fn(|k| {
        let [baixo, comprida, cima] = estrada_rects(k, largura(k));
        let area = custo(CUSTO_ESTRADA);
        caixa(
            world,
            &format!("{} Bottom", nomes[k]),
            baixo,
            area,
            ESTRADA_RGBA,
        );
        let e = caixa(world, nomes[k], comprida, area, ESTRADA_RGBA);
        caixa(
            world,
            &format!("{} Top", nomes[k]),
            cima,
            area,
            ESTRADA_RGBA,
        );
        e
    });
    let corredores = std::array::from_fn(|k| {
        let [x0, x1] = corredor(k);
        let x = 0.5 * (x0 + x1);
        let (y0, y1) = (Y_CHAO + FOLGA_Y, topo - FOLGA_Y);
        let quem = ["Wide", "Narrow"][k];
        desenho(
            world,
            &format!("Flag {quem}"),
            [x, y1],
            [0.3, 0.3],
            BANDEIRA_RGBA,
        );
        let r = perseguidor(
            world,
            &format!("Runner {quem}"),
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
    Estreita {
        estreita: compridas[1],
        larga: compridas[0],
        corredores,
    }
}

/// A linha do terminal da cena.
pub fn anuncia() {
    println!(
        "[nav-smoke] =7 a ESTRADA MAIS ESTREITA QUE O CORPO: a mesma estrada em U (Cost 0.3) dos dois \
         lados. A esquerda (Wide Road, 0,8 m) o vermelho segue-a. A direita (Narrow Road, 0,4 m, mais \
         estreita que o corpo de 0,5 m) ele vai a direito: ela nao faz nada. A Narrow Road esta' \
         escolhida: o Inspector diz porque"
    );
}

#[cfg(test)]
#[path = "nav_smoke_estreita_tests.rs"]
mod tests;
