//! **A forma «Outline»** (W2 do plano `f2_contorno_e_formas_plano.md`): N peças convexas.
//!
//! As peças (já em metros locais e escaladas) entram por [`PhysicsWorld::register_outline`] e ficam
//! numa tabela endereçada pelo CONTEÚDO — só insere, então um `BodyDesc` re-spawnado (rewind,
//! `retune_body`, `reshape_body`) acha sempre as mesmas peças. O build é UM collider compound de
//! `convex_hull` por peça; chave ausente ou sem peça válida cai na caixa `half_*`, nunca pânico.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::rmath::{Pose, Vector};
use rapier2d::geometry::{ColliderBuilder, SharedShape};

use super::PhysicsWorld;

/// Chave de conteúdo de um conjunto de peças ([`PhysicsWorld::register_outline`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutlineKey(pub u64);

/// As peças registadas, por chave.
pub(super) type Outlines = BTreeMap<OutlineKey, Arc<[Vec<Vector>]>>;

/// blake3 sobre os bytes LE, cada peça prefixada pelo seu número de pontos (sem ambiguidade).
fn key_of(pieces: &[Vec<[f32; 2]>]) -> OutlineKey {
    let mut h = blake3::Hasher::new();
    for p in pieces {
        h.update(&(p.len() as u64).to_le_bytes());
        for &[x, y] in p {
            h.update(&x.to_le_bytes());
            h.update(&y.to_le_bytes());
        }
    }
    let mut k = [0u8; 8];
    k.copy_from_slice(&h.finalize().as_bytes()[..8]);
    OutlineKey(u64::from_le_bytes(k))
}

/// O hull de uma peça, ou `None` onde o do parry recusaria ou entraria em pânico
/// (menos de 3 pontos, não finitos, todos coincidentes, colineares).
fn hull(piece: &[Vector]) -> Option<SharedShape> {
    let first = *piece.first()?;
    if piece.len() < 3
        || !piece.iter().all(|p| p.x.is_finite() && p.y.is_finite())
        || piece.iter().all(|&p| p == first)
    {
        return None;
    }
    SharedShape::convex_hull(piece)
}

/// O builder de um `ShapeDesc::Outline`: compound das peças, ou a caixa.
pub(super) fn builder(
    outlines: &Outlines,
    half_x: f32,
    half_y: f32,
    key: OutlineKey,
) -> ColliderBuilder {
    let parts: Vec<(Pose, SharedShape)> = outlines
        .get(&key)
        .into_iter()
        .flat_map(|ps| ps.iter())
        .filter_map(|p| hull(p))
        .map(|s| (Pose::IDENTITY, s))
        .collect();
    if parts.is_empty() {
        return ColliderBuilder::cuboid(half_x, half_y);
    }
    ColliderBuilder::compound(parts)
}

impl PhysicsWorld {
    /// **Regista as peças convexas de uma forma** (metros locais do collider, y para cima) e
    /// devolve a chave que um `ShapeDesc::Outline` leva. A mesma lista dá a mesma chave em
    /// qualquer mundo; registar de novo não troca nada (a tabela só cresce).
    pub fn register_outline(&mut self, pieces: Vec<Vec<[f32; 2]>>) -> OutlineKey {
        let key = key_of(&pieces);
        self.outlines.entry(key).or_insert_with(|| {
            pieces
                .into_iter()
                .map(|p| p.into_iter().map(|[x, y]| Vector::new(x, y)).collect())
                .collect()
        });
        key
    }

    /// Se a chave tem peças registadas neste mundo.
    #[must_use]
    pub fn has_outline(&self, key: OutlineKey) -> bool {
        self.outlines.contains_key(&key)
    }

    /// Leva a tabela de peças de `old` para este mundo — o rewind do ECS troca o `PhysicsWorld`
    /// por um novo e re-spawna por `BodyDesc`, cujas chaves têm de continuar a achar as peças.
    pub fn carry_outlines_from(&mut self, old: &mut PhysicsWorld) {
        self.outlines.append(&mut old.outlines);
    }
}
