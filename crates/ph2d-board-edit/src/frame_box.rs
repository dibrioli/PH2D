//! A caixa RODADA da moldura de selecção (centro, tamanho, ângulo) e onde ficam as pegas dela.
//! ⚠️ Não confundir com o frame do quadro (`ph2d_board_model::Frame`, W5): esta é geometria das
//! pegas, aquele é um elemento.

use ph2d_board_model::Element;

use crate::{Dir, Handle, Metrics};

/// Uma caixa rodada: o centro, o tamanho e o ângulo (mundo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub center: [f64; 2],
    pub w: f64,
    pub h: f64,
    pub angle: f64,
}

impl Frame {
    #[must_use]
    pub fn of(el: &Element) -> Self {
        Self {
            center: el.center(),
            w: el.w,
            h: el.h,
            angle: el.angle,
        }
    }

    /// Ponto local (relativo ao centro, antes de rodar) → mundo.
    #[must_use]
    pub fn point(&self, local: [f64; 2]) -> [f64; 2] {
        let (s, c) = self.angle.sin_cos();
        [
            self.center[0] + local[0] * c - local[1] * s,
            self.center[1] + local[0] * s + local[1] * c,
        ]
    }

    /// Mundo → ponto local (relativo ao centro, antes de rodar).
    #[must_use]
    pub fn local(&self, world: [f64; 2]) -> [f64; 2] {
        let (s, c) = (-self.angle).sin_cos();
        let (dx, dy) = (world[0] - self.center[0], world[1] - self.center[1]);
        [dx * c - dy * s, dx * s + dy * c]
    }

    /// Onde está a pega `h` (mundo), com `px` = um px de ecrã e as medidas da interface.
    #[must_use]
    pub fn handle(&self, h: Handle, px: f64, m: &Metrics) -> [f64; 2] {
        match h {
            Handle::Resize(d) => {
                let (sx, sy) = d.sign();
                self.point([sx * self.w / 2.0, sy * self.h / 2.0])
            }
            Handle::Rotate => self.point([0.0, -self.h / 2.0 - m.rotate_offset * px]),
        }
    }
}
