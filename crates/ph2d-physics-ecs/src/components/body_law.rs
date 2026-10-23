//! **A lei do valor das linhas da §11** (o cartão do corpo) — o que o cartão e o script
//! (`ph2d.set`) aceitam (doc CodeEditor 08 §3.5). Mora com os TIPOS porque o cartão aplica em
//! `ph2d-app-physics` e o script em `ph2d-app-components`, e as duas famílias não se vêem.
//!
//! Só o clamp por-VALOR: o que depende do gesto (converter a forma ao trocar Ball↔Box, destacar o
//! componente no neutro) fica no `apply_physics_edit`. Cada `sanitize` é idempotente.

use super::{
    AreaBuoyancy, AreaDrag, AreaFalloff, AreaFormDrag, DampingOverride, MassOverride, WalkSurface,
};
use crate::{Collider, ColliderShape, MAX_LAYERS};

/// O menor raio / meia-extensão que o solver aceita.
const MIN_EXTENT: f32 = 1e-3;

impl Collider {
    /// Tamanhos positivos (a meia-altura da cápsula pode ser 0 = uma bola), densidade e atrito
    /// ≥ 0, quique em `[0, 1]`, camada `< MAX_LAYERS`. O desvio do colisor é uma posição: sem lei.
    pub fn sanitize(&mut self) {
        self.shape = match self.shape {
            ColliderShape::Ball { radius } => ColliderShape::Ball {
                radius: radius.max(MIN_EXTENT),
            },
            ColliderShape::Cuboid { half_x, half_y } => ColliderShape::Cuboid {
                half_x: half_x.max(MIN_EXTENT),
                half_y: half_y.max(MIN_EXTENT),
            },
            ColliderShape::Capsule {
                half_height,
                radius,
            } => ColliderShape::Capsule {
                half_height: half_height.max(0.0),
                radius: radius.max(MIN_EXTENT),
            },
        };
        self.density = self.density.max(0.0);
        self.restitution = self.restitution.clamp(0.0, 1.0);
        self.friction = self.friction.max(0.0);
        self.layer = self.layer.min(MAX_LAYERS as u8 - 1);
    }
}

impl MassOverride {
    /// Uma massa nula ou negativa não é um corpo.
    pub fn sanitize(&mut self) {
        self.0 = self.0.max(MIN_EXTENT);
    }
}

impl DampingOverride {
    /// Um amortecimento negativo ACRESCENTARIA energia.
    pub fn sanitize(&mut self) {
        self.linear = self.linear.max(0.0);
        self.angular = self.angular.max(0.0);
    }
}

impl WalkSurface {
    /// A aderência é ≥ 0; a esteira tem sinal (o sentido).
    pub fn sanitize(&mut self) {
        self.grip = self.grip.max(0.0);
    }
}

impl AreaDrag {
    /// Um arrasto negativo acrescentaria energia.
    pub fn sanitize(&mut self) {
        self.0 = self.0.max(0.0);
    }
}

impl AreaBuoyancy {
    /// Uma densidade de fluido negativa afundaria o que flutua.
    pub fn sanitize(&mut self) {
        self.0 = self.0.max(0.0);
    }
}

impl AreaFormDrag {
    /// O mesmo piso dos irmãos de arrasto.
    pub fn sanitize(&mut self) {
        self.0 = self.0.max(0.0);
    }
}

impl AreaFalloff {
    /// Uma fração da borda ao centro.
    pub fn sanitize(&mut self) {
        self.0 = self.0.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lei_da_secao_do_corpo_e_idempotente() {
        let mut c = Collider {
            shape: ColliderShape::Capsule {
                half_height: -1.0,
                radius: 0.0,
            },
            density: -2.0,
            restitution: 3.0,
            friction: -1.0,
            layer: 200,
            is_sensor: false,
            offset: [-4.0, 5.0],
        };
        c.sanitize();
        let uma = c;
        c.sanitize();
        assert_eq!(c, uma);
        assert_eq!(
            c.shape,
            ColliderShape::Capsule {
                half_height: 0.0,
                radius: MIN_EXTENT
            }
        );
        assert_eq!((c.density, c.restitution, c.friction), (0.0, 1.0, 0.0));
        assert_eq!(c.layer, MAX_LAYERS as u8 - 1);
        assert_eq!(c.offset, [-4.0, 5.0], "o desvio é uma posição: sem lei");
        let mut f = AreaFalloff(2.0);
        f.sanitize();
        assert_eq!(f.0, 1.0);
    }
}
