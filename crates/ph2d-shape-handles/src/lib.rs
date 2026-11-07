//! **As alças da forma de colisão no canvas** (ficha f1, onda 3; molde: `ph2d-camera-frame`, doc 17b).
//!
//! A lei trabalha no «referencial escalado» do objecto: `q = R⁻¹ (p − T)`, a mesma conta do contorno
//! (`overlay::outline`) e do solver. Nele o centro é `offset × escala` (com sinal: o espelho dobra o Offset) e as
//! meias-medidas são `medida × |escala|`, por isso a rotação, o pai e o espelho entram numa só conta.

mod ponteiro;
mod roteiro;
pub mod tinta;

pub use ponteiro::{
    CursorDaAlca, FormaNoCanvas, Porta, a_meio, alterna, ativo, clique_parado, cursor_em, editando,
    quadro,
};
pub use roteiro::{Botao, PassoDaMao, Plano, passo_da_mao};

use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor_core::PhysicsFieldEdit;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{Collider, ColliderShape, MIN_EXTENT};
use ph2d_render::Camera2d;

/// O raio do dedo sobre uma alça, em píxeis de ecrã.
pub const AGARRE_PX: f32 = 9.0; // LITERAL-PX-OK: dedo da alça, em ecrã

/// A folga de ecrã abaixo da qual um `Down` seguido de `Up` é um clique.
pub const ARMA_PX: f32 = 2.0; // LITERAL-PX-OK: limiar do arrasto, em ecrã

/// A pose no mundo: a que o contorno e o solver usam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub t: [f32; 2],
    pub rot: f32,
    pub s: [f32; 2],
}

impl Pose {
    /// `None` sem pose ou com escala nula: nesse caso nenhum ponto do ecrã volta à forma.
    #[must_use]
    pub fn de(world: &World, e: Entity) -> Option<Self> {
        let mut chain = Vec::new();
        let t = ph2d_ecs::world_transform_into(world, e, &mut chain)?;
        let s = [t.scale.x, t.scale.y];
        (s[0] != 0.0 && s[1] != 0.0).then_some(Self {
            t: [t.translation.x, t.translation.y],
            rot: t.rotation,
            s,
        })
    }

    #[must_use]
    pub fn para_mundo(&self, q: [f32; 2]) -> [f32; 2] {
        let (sin, cos) = self.rot.sin_cos();
        [
            self.t[0] + q[0] * cos - q[1] * sin,
            self.t[1] + q[0] * sin + q[1] * cos,
        ]
    }

    #[must_use]
    pub fn do_mundo(&self, p: [f32; 2]) -> [f32; 2] {
        let (sin, cos) = self.rot.sin_cos();
        let (x, y) = (p[0] - self.t[0], p[1] - self.t[1]);
        [x * cos + y * sin, -x * sin + y * cos]
    }
}

/// Uma alça: o miolo, ou um lado ou canto `(x, y)` com cada um em `-1..=1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alca {
    Miolo,
    Lado(i8, i8),
}

/// O que as alças mexem.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Medida {
    pub shape: ColliderShape,
    pub offset: [f32; 2],
}

impl Medida {
    #[must_use]
    pub const fn de(c: &Collider) -> Self {
        Self {
            shape: c.shape,
            offset: c.offset,
        }
    }

    /// A única edição do gesto: forma e Offset juntos, como um valor só.
    #[must_use]
    pub const fn edicao(&self) -> PhysicsFieldEdit {
        let medidas = match self.shape {
            ColliderShape::Cuboid { half_x, half_y } => [half_x, half_y],
            ColliderShape::Ball { radius } => [radius, 0.0],
            ColliderShape::Capsule {
                half_height,
                radius,
            } => [radius, half_height],
        };
        PhysicsFieldEdit::ShapeDrag {
            medidas,
            offset: self.offset,
        }
    }
}

/// As alças de cada forma, o miolo por último (é o que perde um empate).
#[must_use]
pub fn alcas(shape: ColliderShape) -> Vec<Alca> {
    let mut v = match shape {
        ColliderShape::Cuboid { .. } => vec![
            Alca::Lado(-1, -1),
            Alca::Lado(1, -1),
            Alca::Lado(1, 1),
            Alca::Lado(-1, 1),
            Alca::Lado(0, -1),
            Alca::Lado(1, 0),
            Alca::Lado(0, 1),
            Alca::Lado(-1, 0),
        ],
        ColliderShape::Ball { .. } => vec![Alca::Lado(1, 0)],
        ColliderShape::Capsule { .. } => {
            vec![Alca::Lado(1, 0), Alca::Lado(0, 1), Alca::Lado(0, -1)]
        }
    };
    v.push(Alca::Miolo);
    v
}

/// O centro e as meias-medidas, no referencial escalado.
#[must_use]
pub fn caixa(pose: &Pose, m: &Medida) -> ([f32; 2], [f32; 2]) {
    let [sx, sy] = [pose.s[0].abs(), pose.s[1].abs()];
    let c = [m.offset[0] * pose.s[0], m.offset[1] * pose.s[1]];
    let h = match m.shape {
        ColliderShape::Cuboid { half_x, half_y } => [half_x * sx, half_y * sy],
        ColliderShape::Ball { radius } => [radius * sx, radius * sy],
        ColliderShape::Capsule {
            half_height,
            radius,
        } => [radius * sx, (half_height + radius) * sy],
    };
    (c, h)
}

/// Onde a alça está, no referencial escalado.
#[must_use]
pub fn ponto_local(pose: &Pose, m: &Medida, a: Alca) -> [f32; 2] {
    let (c, h) = caixa(pose, m);
    match a {
        Alca::Miolo => c,
        Alca::Lado(x, y) => [c[0] + f32::from(x) * h[0], c[1] + f32::from(y) * h[1]],
    }
}

/// Onde a alça está, em píxeis de ecrã.
#[must_use]
pub fn no_ecra(pose: &Pose, m: &Medida, a: Alca, cam: &Camera2d, win: WindowSize) -> (f32, f32) {
    cam.world_to_screen(pose.para_mundo(ponto_local(pose, m, a)), win)
}

/// `q` (referencial escalado) cai dentro da forma?
fn dentro(pose: &Pose, m: &Medida, q: [f32; 2]) -> bool {
    let (c, h) = caixa(pose, m);
    let (dx, dy) = ((q[0] - c[0]).abs(), (q[1] - c[1]).abs());
    match m.shape {
        ColliderShape::Cuboid { .. } => dx <= h[0] && dy <= h[1],
        ColliderShape::Ball { .. } => (dx / h[0]).powi(2) + (dy / h[1]).powi(2) <= 1.0,
        ColliderShape::Capsule { half_height, .. } => {
            let reta = half_height * pose.s[1].abs();
            let ry = (h[1] - reta).max(MIN_EXTENT);
            (dx / h[0]).powi(2) + ((dy - reta).max(0.0) / ry).powi(2) <= 1.0
        }
    }
}

/// A alça sob o ponteiro: a mais próxima em ECRÃ dentro do dedo (o ponto do miolo conta: numa forma pequena ele
/// tem de continuar agarrável); senão o miolo, se o ponteiro cair na forma.
#[must_use]
pub fn alca_em(
    pose: &Pose,
    m: &Medida,
    cam: &Camera2d,
    win: WindowSize,
    ponteiro: (f32, f32),
) -> Option<Alca> {
    let perto = alcas(m.shape)
        .into_iter()
        .map(|a| {
            let (x, y) = no_ecra(pose, m, a, cam, win);
            (a, (x - ponteiro.0).hypot(y - ponteiro.1))
        })
        .filter(|(_, d)| *d <= AGARRE_PX)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    perto.map(|(a, _)| a).or_else(|| {
        let q = pose.do_mundo(cam.screen_to_world(ponteiro, win));
        dentro(pose, m, q).then_some(Alca::Miolo)
    })
}

/// Modificadores do gesto.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Teclas {
    /// Simétrico a partir do centro: o Offset fica.
    pub alt: bool,
    /// No canto: mantém a proporção.
    pub shift: bool,
}

/// Um eixo: o lado puxado vai a `e`, e o oposto fica (ou, com Alt, o centro). Devolve `(centro, meia-medida)`.
fn eixo(c: f32, h: f32, lado: f32, e: f32, piso: f32, alt: bool) -> (f32, f32) {
    if alt {
        (c, (lado * (e - c)).max(piso))
    } else {
        let parado = c - lado * h;
        let meia = (lado * (e - parado) * 0.5).max(piso);
        (parado + lado * meia, meia)
    }
}

/// A medida depois de levar a alça a `e` (referencial escalado; já sem a folga da pega). Parte sempre do `Down`.
#[must_use]
pub fn puxa(inicio: &Medida, a: Alca, pose: &Pose, e: [f32; 2], t: Teclas) -> Medida {
    let (c, h) = caixa(pose, inicio);
    let abs = [pose.s[0].abs(), pose.s[1].abs()];
    let mut m = *inicio;
    let Alca::Lado(lx, ly) = a else {
        m.offset = [e[0] / pose.s[0], e[1] / pose.s[1]];
        return m;
    };
    let (fx, fy) = (f32::from(lx), f32::from(ly));
    match inicio.shape {
        ColliderShape::Cuboid { half_x, half_y } => {
            let piso = [MIN_EXTENT * abs[0], MIN_EXTENT * abs[1]];
            if t.shift && lx != 0 && ly != 0 {
                let medir = |i: usize, lado: f32| {
                    if t.alt {
                        lado * (e[i] - c[i]) / h[i]
                    } else {
                        lado * (e[i] - (c[i] - lado * h[i])) * 0.5 / h[i]
                    }
                };
                let k = medir(0, fx)
                    .max(medir(1, fy))
                    .max(piso[0] / h[0])
                    .max(piso[1] / h[1]);
                m.shape = ColliderShape::Cuboid {
                    half_x: half_x * k,
                    half_y: half_y * k,
                };
                if !t.alt {
                    m.offset = [
                        (c[0] - fx * h[0] + fx * h[0] * k) / pose.s[0],
                        (c[1] - fy * h[1] + fy * h[1] * k) / pose.s[1],
                    ];
                }
                return m;
            }
            let (mut nx, mut ny) = (half_x, half_y);
            if lx != 0 {
                let (cx, meia) = eixo(c[0], h[0], fx, e[0], piso[0], t.alt);
                nx = meia / abs[0];
                m.offset[0] = cx / pose.s[0];
            }
            if ly != 0 {
                let (cy, meia) = eixo(c[1], h[1], fy, e[1], piso[1], t.alt);
                ny = meia / abs[1];
                m.offset[1] = cy / pose.s[1];
            }
            m.shape = ColliderShape::Cuboid {
                half_x: nx,
                half_y: ny,
            };
        }
        ColliderShape::Ball { .. } => {
            m.shape = ColliderShape::Ball {
                radius: (fx * (e[0] - c[0]) / abs[0]).max(MIN_EXTENT),
            };
        }
        ColliderShape::Capsule {
            half_height,
            radius,
        } => {
            // Height = 2 × (Half Height + Radius): cada alça mexe só naquilo que puxa e a outra medida fica.
            let total = half_height + radius;
            if lx != 0 {
                let r = (fx * (e[0] - c[0]) / abs[0]).max(MIN_EXTENT).min(total);
                m.shape = ColliderShape::Capsule {
                    half_height: (total - r).max(0.0),
                    radius: r,
                };
            } else {
                let (cy, meia) = eixo(c[1], h[1], fy, e[1], radius * abs[1], t.alt);
                m.offset[1] = cy / pose.s[1];
                m.shape = ColliderShape::Capsule {
                    half_height: (meia / abs[1] - radius).max(0.0),
                    radius,
                };
            }
        }
    }
    m
}

#[cfg(test)]
mod tests;
