//! **Trocar a forma de um corpo em jogo** — o agachar com corpo baixo
//! (`docs/GameEngineFuncional/09_abaixar_plano.md`).
//!
//! Só o collider PRINCIPAL (o 1.º do corpo, o do spawn) muda; handle, camada, sensor e superfície
//! ficam. A massa fica a de pé: um corpo agachado não fica mais leve. Repor a forma autorada repõe a
//! fonte de massa autorada (densidade ou massa), bit a bit com o spawn.

use crate::rmath::Vector;
use rapier2d::dynamics::RigidBodyHandle;

use super::desc::BodyDesc;
use super::shape::ShapeDesc;
use super::{PhysicsWorld, collider_build};

impl ShapeDesc {
    /// Quanto o topo pode baixar com a base no sítio: o troço recto (cápsula, estádio, caixa).
    /// Bola e elipse não têm troço recto — só a perna as agacha.
    #[must_use]
    pub fn top_shrink_room(self) -> f32 {
        match self {
            Self::Ball { .. } | Self::Ellipse { .. } => 0.0,
            Self::Cuboid { half_y, .. } => 2.0 * half_y,
            Self::Capsule { half_height, .. } | Self::Stadium { half_height, .. } => {
                2.0 * half_height
            }
        }
    }

    /// A forma com o topo `s` mais abaixo e a base no mesmo sítio: a meia-altura perde `s/2` e o
    /// centro desce `s/2` (quem chama desce a translação). Sem troço recto que sobre, a cápsula
    /// vira bola e o estádio elipse.
    #[must_use]
    pub fn shrunk_from_top(self, s: f32) -> Self {
        let s = s.clamp(0.0, self.top_shrink_room());
        match self {
            Self::Cuboid { half_x, half_y } => Self::Cuboid {
                half_x,
                half_y: (half_y - 0.5 * s).max(f32::MIN_POSITIVE),
            },
            Self::Capsule {
                half_height,
                radius,
            } => match half_height - 0.5 * s {
                hh if hh > 0.0 => Self::Capsule {
                    half_height: hh,
                    radius,
                },
                _ => Self::Ball { radius },
            },
            Self::Stadium {
                half_height,
                rx,
                ry,
            } => match half_height - 0.5 * s {
                hh if hh > 0.0 => Self::Stadium {
                    half_height: hh,
                    rx,
                    ry,
                },
                _ => Self::Ellipse { rx, ry },
            },
            other => other,
        }
    }
}

impl PhysicsWorld {
    /// Põe `shape` com a translação local `offset` no collider principal de `body`; `rest` é a
    /// receita do spawn (a forma de pé). Idempotente pela translação: devolve `false` quando ela já é
    /// a pedida — e por isso um `rewind_to`, que reconstrói de pé, cura-se na chamada seguinte.
    pub fn reshape_body(
        &mut self,
        body: RigidBodyHandle,
        rest: &BodyDesc,
        shape: ShapeDesc,
        offset: [f32; 2],
    ) -> bool {
        let Some(&ch) = self.bodies.get(body).and_then(|b| b.colliders().first()) else {
            return false;
        };
        let Some(c) = self.colliders.get_mut(ch) else {
            return false;
        };
        let now = c
            .position_wrt_parent()
            .map(|p| [p.translation.x, p.translation.y]);
        if now == Some(offset) {
            return false;
        }
        let mass = c.mass();
        c.set_shape(collider_build::shape_builder(shape).shape);
        c.set_translation_wrt_parent(Vector::new(offset[0], offset[1]));
        if shape == rest.shape && offset == rest.offset {
            match rest.mass_override {
                Some(m) => c.set_mass(m),
                None => c.set_density(rest.density),
            }
        } else {
            c.set_mass(mass);
        }
        if let Some(b) = self.bodies.get_mut(body) {
            b.recompute_mass_properties_from_colliders(&self.colliders);
            b.wake_up(true);
        }
        true
    }
}

#[cfg(test)]
#[path = "reshape_tests.rs"]
mod tests;
