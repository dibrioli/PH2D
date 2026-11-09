//! **A forma «Outline» do lado ECS** (W2b do plano `f2_contorno_e_formas_plano.md`, «W2 — desenho
//! fechado»): o componente [`ColliderOutline`] (a VERDADE editável, anéis em Q4) e a conversão para as
//! peças em metros que o motor regista. As peças são DERIVADAS (`ph2d_outline_trace::convex_pieces`),
//! nunca gravadas; o cache da ponte ([`OutlineCache`]) deriva-as uma vez por `(anéis, detail)`.

use std::collections::BTreeMap;
use std::sync::Arc;

use bevy_ecs::component::Component;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimComponent, World};
use ph2d_physics::{OutlineKey, PhysicsWorld, ShapeDesc};
use serde::{Deserialize, Serialize};

use crate::ColliderShape;

/// Um anel em Q4 (1/16 px), y para baixo, origem no canto da imagem de `picture_px`.
pub type Ring = Vec<[i32; 2]>;

/// **O contorno de uma forma `ColliderShape::Outline`**, na mesma entidade do `Collider` (a forma
/// FILHA, opção B). Externo CCW, buracos CW (a convenção do `ph2d-outline-trace`).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColliderOutline {
    pub rings: Arc<[Ring]>,
    /// O retângulo da imagem que os anéis cobrem, em px: mapeia para `[-half, half]` da forma.
    pub picture_px: [u32; 2],
    /// O Detail PEDIDO (o do cartão e do canal); o traço usa [`Self::detail_used_q4`].
    pub detail_q4: u16,
    pub alpha_cutoff: u8,
    pub min_island_px: u16,
    /// O Detail que o tecto de peças/pontos impôs (Q4); `0` = o pedido serviu.
    pub raised_q4: u16,
}

impl Default for ColliderOutline {
    /// Sem anéis (= a caixa); Detail 2 px e Alpha cutoff 0,5 (decisões do dono, 08/11).
    fn default() -> Self {
        Self {
            rings: Arc::from(Vec::new()),
            picture_px: [1, 1],
            detail_q4: 2 * 16,
            alpha_cutoff: 128,
            min_island_px: 0,
            raised_q4: 0,
        }
    }
}

impl SimComponent for ColliderOutline {}

impl ColliderOutline {
    /// O Detail com que os anéis foram traçados (o pedido, ou o que o tecto subiu).
    #[must_use]
    pub fn detail_used_q4(&self) -> u16 {
        self.detail_q4.max(self.raised_q4)
    }

    /// Hash de conteúdo dos anéis (blake3, cada anel prefixado pelo comprimento).
    #[must_use]
    pub fn rings_hash(&self) -> u64 {
        let mut h = blake3::Hasher::new();
        for r in self.rings.iter() {
            h.update(&(r.len() as u64).to_le_bytes());
            for &[x, y] in r {
                h.update(&x.to_le_bytes());
                h.update(&y.to_le_bytes());
            }
        }
        let mut k = [0u8; 8];
        k.copy_from_slice(&h.finalize().as_bytes()[..8]);
        u64::from_le_bytes(k)
    }

    /// Um ponto Q4 em metros locais da forma: `x = (qx/(16·pw) − ½)·2·half_x·sx`,
    /// `y = −(qy/(16·ph) − ½)·2·half_y·sy`, escala COM SINAL (um espelho é outra forma).
    #[must_use]
    pub fn to_metres(&self, q: [i32; 2], half: [f32; 2], scale: Vec2) -> [f32; 2] {
        let pw = 16.0 * self.picture_px[0].max(1) as f32;
        let ph = 16.0 * self.picture_px[1].max(1) as f32;
        [
            (q[0] as f32 / pw - 0.5) * 2.0 * half[0] * scale.x,
            -(q[1] as f32 / ph - 0.5) * 2.0 * half[1] * scale.y,
        ]
    }

    /// Os anéis em metros locais (o que o overlay desenha): linear, sem peças.
    #[must_use]
    pub fn rings_m(&self, half: [f32; 2], scale: Vec2) -> Vec<Vec<[f32; 2]>> {
        self.rings
            .iter()
            .map(|r| r.iter().map(|&q| self.to_metres(q, half, scale)).collect())
            .collect()
    }
}

/// **A figura de uma forma `Outline` em MUNDO** (o que o overlay desenha): os anéis do
/// [`ColliderOutline`] de `e` — buracos incluídos, espelhados pela escala COM SINAL — girados por
/// `rotation` em torno de `at`. `None` = não é um `Outline` com anéis (a figura é a caixa).
#[must_use]
pub fn outline_rings_world(
    world: &World,
    e: Entity,
    shape: ColliderShape,
    scale: Vec2,
    at: [f32; 2],
    rotation: f32,
) -> Option<Vec<Vec<[f32; 2]>>> {
    let ColliderShape::Outline { half_x, half_y } = shape else {
        return None;
    };
    let o = world
        .get::<ColliderOutline>(e)
        .filter(|o| !o.rings.is_empty())?;
    let (s, c) = rotation.sin_cos();
    let mut rings = o.rings_m([half_x, half_y], scale);
    for p in rings.iter_mut().flatten() {
        *p = [at[0] + p[0] * c - p[1] * s, at[1] + p[0] * s + p[1] * c];
    }
    Some(rings)
}

/// Peças em Q4 por `(hash dos anéis, detail)`, e a chave do motor por conversão.
#[derive(Default)]
pub(crate) struct OutlineCache {
    pieces: BTreeMap<(u64, u16), Arc<[Ring]>>,
    keys: BTreeMap<(u64, u16, [u32; 2], [u32; 4]), OutlineKey>,
}

impl OutlineCache {
    /// As peças convexas (Q4) dos anéis, derivadas UMA vez por `(anéis, detail)`.
    fn pieces_q4(&mut self, o: &ColliderOutline, hash: u64) -> Arc<[Ring]> {
        self.pieces
            .entry((hash, o.detail_used_q4()))
            .or_insert_with(|| {
                let outline = ph2d_outline_trace::Outline {
                    rings: o.rings.to_vec(),
                };
                ph2d_outline_trace::convex_pieces(&outline, o.detail_used_q4()).into()
            })
            .clone()
    }

    /// `shape` com a chave das peças registadas, se for um `Outline` com anéis; senão `shape`
    /// tal como veio (a caixa). `scale` é a escala de mundo COM SINAL; o `half_*` do `shape` já
    /// vem em módulo (`scaled_shape`).
    pub(crate) fn resolve(
        &mut self,
        world: &mut PhysicsWorld,
        o: Option<&ColliderOutline>,
        shape: ShapeDesc,
        scale: Vec2,
    ) -> ShapeDesc {
        let (ShapeDesc::Outline { half_x, half_y, .. }, Some(o)) = (shape, o) else {
            return shape;
        };
        if o.rings.is_empty() {
            return shape;
        }
        let sign = Vec2::new(sign_of(scale.x), sign_of(scale.y));
        let hash = o.rings_hash();
        let memo = (
            hash,
            o.detail_used_q4(),
            o.picture_px,
            [
                half_x.to_bits(),
                half_y.to_bits(),
                sign.x.to_bits(),
                sign.y.to_bits(),
            ],
        );
        let key = match self.keys.get(&memo) {
            Some(&k) if world.has_outline(k) => k,
            _ => {
                let pieces = self.pieces_q4(o, hash);
                let m = pieces
                    .iter()
                    .map(|p| {
                        p.iter()
                            .map(|&q| o.to_metres(q, [half_x, half_y], sign))
                            .collect()
                    })
                    .collect();
                let k = world.register_outline(m);
                self.keys.insert(memo, k);
                k
            }
        };
        ShapeDesc::Outline {
            half_x,
            half_y,
            key,
        }
    }
}

fn sign_of(s: f32) -> f32 {
    if s < 0.0 { -1.0 } else { 1.0 }
}
