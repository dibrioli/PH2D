//! W23 B5 — o cursor que o quadro pede sob o ponteiro: sobre uma pega de redimensionar, a seta dupla
//! do EIXO dela (com a rotação da moldura); sobre a de rodar, a de agarrar. A janela traduz para o
//! cursor do sistema.

use ph2d_board_model::BoardDoc;

use crate::{Editor, Handle, Pointer};

/// O cursor de uma pega.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardCursor {
    /// ↕
    ResizeNs,
    /// ↔
    ResizeEw,
    /// ⤢ (de baixo-esquerda a cima-direita)
    ResizeNesw,
    /// ⤡ (de cima-esquerda a baixo-direita)
    ResizeNwse,
    Rotate,
}

impl Editor {
    /// O cursor da pega da moldura sob `p`, se há — ou, sobre o segmento do meio de um cotovelo, a seta
    /// dupla do eixo em que ele anda (W23 B5(1)).
    #[must_use]
    pub fn cursor_at(&self, doc: &BoardDoc, p: Pointer) -> Option<BoardCursor> {
        if let Some((_, _, e)) = self.segment_at(doc, p) {
            return Some(if e.vertical {
                BoardCursor::ResizeEw
            } else {
                BoardCursor::ResizeNs
            });
        }
        let angle = self.frame(doc)?.angle;
        Some(match self.handle_at(doc, p)? {
            Handle::Rotate => BoardCursor::Rotate,
            Handle::Resize(d) => {
                let (sx, sy) = d.sign();
                let (sin, cos) = angle.sin_cos();
                let (x, y) = (sx * cos - sy * sin, sx * sin + sy * cos);
                // O eixo (sem sentido) em oitavos de volta: 0 ↔, 1 ⤡, 2 ↕, 3 ⤢ (y para baixo).
                let eighth = (y.atan2(x) / std::f64::consts::FRAC_PI_4).round() as i64;
                match eighth.rem_euclid(4) {
                    0 => BoardCursor::ResizeEw,
                    1 => BoardCursor::ResizeNwse,
                    2 => BoardCursor::ResizeNs,
                    _ => BoardCursor::ResizeNesw,
                }
            }
        })
    }
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
