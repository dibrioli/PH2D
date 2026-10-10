//! ⭐ **A FOLGA DE UM TOQUE** — uma lei, três superfícies capturadas (o grafo do Motion, a
//! dope-sheet da timeline e a tira do Flip).
//!
//! Um toque é um aperto que DESLIZOU: a mão treme um ou dois píxeis entre apertar e soltar. Sem
//! folga, o soltar escolhe `End` (arrasto) em vez de `Click`, e todo alvo que só responde ao clique
//! vira cara-ou-coroa. A timeline e o Flip já tinham a folga, cada um com a sua cópia; o grafo não
//! tinha nenhuma — e era aí que o dono via *«apertar botões … impreciso e inconsistente»*
//! (report de 10/10, BUGS do Motion #18). Uma porta só, para não haver uma quarta cópia.

use super::drag::DRAG_THRESHOLD_PX;

/// O percurso (px lógicos, Chebyshev) a partir do aperto antes de o gesto contar como arrasto.
/// ⚠️ É o [`DRAG_THRESHOLD_PX`] da casa — a mesma pergunta, *«a mão andou o suficiente para isto não
/// ser um toque?»*.
pub const PRESS_SLOP_PX: f32 = DRAG_THRESHOLD_PX;

/// O aperto em curso: de onde se mede a folga, e se ela já foi passada.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PressSlop {
    press: (f32, f32),
    moved: bool,
}

impl PressSlop {
    /// Um aperto novo: ainda é um toque, e é daqui que a folga se mede.
    pub fn begin(&mut self, x: f32, y: f32) {
        self.press = (x, y);
        self.moved = false;
    }

    /// O ponteiro está em `(x, y)`: passou a folga? (uma vez passada, fica passada.)
    pub fn note(&mut self, x: f32, y: f32) -> bool {
        if !self.moved {
            let (px, py) = self.press;
            self.moved = (x - px).abs().max((y - py).abs()) > PRESS_SLOP_PX;
        }
        self.moved
    }

    /// Lê e zera — o soltar escolhe `End` contra `Click`.
    pub fn take_moved(&mut self) -> bool {
        std::mem::take(&mut self.moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tremble_inside_the_slop_is_still_a_tap_and_past_it_a_drag_for_good() {
        let mut s = PressSlop::default();
        s.begin(100.0, 50.0);
        assert!(!s.note(103.0, 47.0), "3 px de tremor ainda é toque");
        assert!(s.note(100.0, 55.0), "5 px é arrasto");
        assert!(
            s.note(100.0, 50.0),
            "e voltar ao ponto não desfaz o arrasto"
        );
        assert!(s.take_moved());
        assert!(!s.take_moved(), "a marca zera na leitura");
        s.begin(0.0, 0.0);
        assert!(!s.note(4.0, 4.0), "medida a partir do aperto NOVO");
    }
}
