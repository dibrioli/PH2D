//! **Um traço da caneta** (W4): os pontos por onde a caneta passou, com a pressão, guardados na
//! caixa do elemento — mover é mudar `x, y`, redimensionar é mudar `w, h` (os pontos escalam com a
//! caixa ao desenhar), como uma forma. O contorno que se preenche é derivado (`ph2d-board-rough`).
//!
//! ⚠️ postcard é posicional: variante NOVA só no fim de cada enum.

use serde::{Deserialize, Serialize};

use crate::Style;

/// Que caneta fez o traço — as duas do painel da caneta do Miro.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pen {
    #[default]
    Pen,
    /// O marcador: largo e TRANSLÚCIDO (a transparência do Miro não se ajusta — `help` «Pen», FAQ).
    Highlighter,
}

/// Um traço. `style.stroke` é a cor, `style.stroke_width` o DIÂMETRO; o resto do estilo não se usa.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ink {
    pub style: Style,
    pub pen: Pen,
    /// `[x, y, pressão]`, `x, y` relativos ao canto da caixa quando ela media [`Self::base`].
    pub points: Vec<[f32; 3]>,
    /// O tamanho da caixa a que `points` se referem.
    pub base: [f64; 2],
    /// A pressão foi MEDIDA por um dispositivo (senão o traço é de largura constante: o Miro não
    /// varia a espessura com a caneta — ideia «Open» na comunidade dele, 7583).
    pub pressure: bool,
}

impl Ink {
    /// Um traço de pontos do MUNDO `[x, y, pressão]`. Devolve-o com a caixa `[x, y, w, h]` que o
    /// contém com meia espessura de folga (um ponto só, ou uma recta, também têm caixa).
    #[must_use]
    pub fn from_world(
        world: &[[f64; 3]],
        style: Style,
        pen: Pen,
        pressure: bool,
    ) -> (Ink, [f64; 4]) {
        let r = style.stroke_width / 2.0;
        let (mut x0, mut y0, mut x1, mut y1) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in world {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
        let (x, y) = (x0 - r, y0 - r);
        let (w, h) = (x1 - x0 + 2.0 * r, y1 - y0 + 2.0 * r);
        let points = world
            .iter()
            .map(|p| [(p[0] - x) as f32, (p[1] - y) as f32, p[2] as f32])
            .collect();
        let ink = Ink {
            style,
            pen,
            points,
            base: [w, h],
            pressure,
        };
        (ink, [x, y, w, h])
    }

    /// Os pontos na caixa `[x, y, w, h]` (sem rotação: quem desenha roda o elemento inteiro).
    #[must_use]
    pub fn placed(&self, [x, y, w, h]: [f64; 4]) -> Vec<[f64; 3]> {
        let sx = if self.base[0] > 0.0 {
            w / self.base[0]
        } else {
            1.0
        };
        let sy = if self.base[1] > 0.0 {
            h / self.base[1]
        } else {
            1.0
        };
        self.points
            .iter()
            .map(|p| {
                [
                    x + f64::from(p[0]) * sx,
                    y + f64::from(p[1]) * sy,
                    f64::from(p[2]),
                ]
            })
            .collect()
    }
}
