//! **O Look da forma de colisão** (GEF02 onda 8, dono 10/11: «a seção Look completa do sprite para o collision
//! shape»): o PIVOT dela e a cor com que o overlay a pinta. O pivô mora em FRAÇÃO da meia-extensão, e é isso que
//! faz a forma crescer em volta dele quando Width/Height andam (cartão, curva, tique a tique) sem estado escondido.
//!
//! A porta única é [`forma_resolvida`]: devolve o `Collider` com o `offset` já descontado do pivô, e todo leitor
//! que trata o `offset` como «o centro da forma» (o solver, a emenda, o overlay, as alças) lê por ela.

use crate::{Collider, ColliderShape};
use ph2d_ecs::{Component, Entity, SimComponent, World};
use serde::{Deserialize, Serialize};

/// O Look de uma forma de colisão. Ausente = o neutro ([`ShapeLook::default`]): pivô no centro, branco, opaco.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeLook {
    /// Onde o pivô (a origem do objeto-forma) cai NA forma, em fração da meia-extensão a partir do centro, `y` para
    /// cima: `[-1, -1]` = canto de baixo à esquerda, `[0, 0]` = centro, `[1, 1]` = canto de cima à direita. Fora de
    /// `[-1, 1]` é um pivô fora da forma, legal como o Custom da Sprite.
    pub pivot: [f32; 2],
    /// Multiplica a cor que o tema dá à forma no overlay (o Tint da Sprite sobre a textura). Branco = a do tema.
    pub tint: [f32; 4],
    /// Multiplica a opacidade do overlay da forma, `0..=1`.
    pub opacity: f32,
}

impl SimComponent for ShapeLook {}

impl Default for ShapeLook {
    fn default() -> Self {
        Self {
            pivot: [0.0, 0.0],
            tint: [1.0; 4],
            opacity: 1.0,
        }
    }
}

impl ShapeLook {
    /// O multiplicador da cor do overlay: o Tint com a Opacity no alfa.
    #[must_use]
    pub fn tom(&self) -> [f32; 4] {
        let [r, g, b, a] = self.tint;
        [r, g, b, a * self.opacity]
    }

    /// Números finitos; cor e opacidade em `0..=1`. O pivô não tem faixa (fora da forma é legal).
    pub fn sanitize(&mut self) {
        let fin = |v: f32, d: f32| if v.is_finite() { v } else { d };
        self.pivot = [fin(self.pivot[0], 0.0), fin(self.pivot[1], 0.0)];
        for c in &mut self.tint {
            *c = fin(*c, 1.0).clamp(0.0, 1.0);
        }
        self.opacity = fin(self.opacity, 1.0).clamp(0.0, 1.0);
    }
}

/// A meia-extensão local da forma em cada eixo (m, antes da escala): a régua da fração do pivô.
#[must_use]
pub fn meia_extensao(shape: ColliderShape) -> [f32; 2] {
    match shape {
        ColliderShape::Ball { radius } => [radius, radius],
        ColliderShape::Cuboid { half_x, half_y } | ColliderShape::Outline { half_x, half_y } => {
            [half_x, half_y]
        }
        ColliderShape::Capsule {
            half_height,
            radius,
        } => [radius, half_height + radius],
    }
}

/// O centro local da forma (m, antes da escala) a partir da origem do objeto: `offset − pivô ⊙ meia-extensão`.
#[must_use]
pub fn centro_local(col: &Collider, pivot: [f32; 2]) -> [f32; 2] {
    let h = meia_extensao(col.shape);
    [
        col.offset[0] - pivot[0] * h[0],
        col.offset[1] - pivot[1] * h[1],
    ]
}

/// ⭐ **A porta única**: o `col` de `e` com o `offset` trocado pelo centro real da forma ([`centro_local`]). Sem
/// [`ShapeLook`] devolve `col` intacto, bit a bit.
#[must_use]
pub fn forma_resolvida(world: &World, e: Entity, col: &Collider) -> Collider {
    match world.get::<ShapeLook>(e) {
        Some(l) if l.pivot != [0.0, 0.0] => Collider {
            offset: centro_local(col, l.pivot),
            ..*col
        },
        _ => *col,
    }
}

/// O inverso da porta para quem ESCREVE a forma pelo centro (as alças): o `offset` cru que põe o centro em
/// `centro` com a forma `shape` e o pivô de `e`.
#[must_use]
pub fn offset_para_centro(
    world: &World,
    e: Entity,
    shape: ColliderShape,
    centro: [f32; 2],
) -> [f32; 2] {
    let p = world.get::<ShapeLook>(e).map_or([0.0, 0.0], |l| l.pivot);
    let h = meia_extensao(shape);
    [centro[0] + p[0] * h[0], centro[1] + p[1] * h[1]]
}

#[cfg(test)]
#[path = "shape_look_tests.rs"]
mod tests;
