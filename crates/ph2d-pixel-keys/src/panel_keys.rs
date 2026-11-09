//! ⭐⭐ **O painel sob o ponteiro fica com as teclas dele** (dono: «nenhuma tecla do Pixel rouba uma
//! tecla do app»; `docs/PixelArt/P12_teclas.md` §4). O Pixel tem o teclado antes da cadeia da shell
//! (`pixel_owns_the_keyboard`); sem esta cedência o `Ctrl+D` com o cursor sobre a Hierarquia
//! virava Deselect no sprite em vez de duplicar a linha.

use winit::keyboard::{KeyCode, KeyCode as K};

/// Os painéis do app com teclas próprias que estão sob o ponteiro: a Hierarquia
/// (`keyboard_hierarchy.rs`, pela área do ponteiro) e o dope-sheet da timeline da cena
/// (`keyboard_timeline.rs`). O canvas e a timeline do sprite não contam: lá as teclas são do Pixel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PanelUnderPointer {
    pub hierarchy: bool,
    pub dope_sheet: bool,
}

impl PanelUnderPointer {
    /// A tecla `key` (com `ctrl` = Ctrl ou Cmd) é de um painel sob o ponteiro? Então o Pixel cede-a.
    /// Os conjuntos são os braços das cadeias dos painéis (o Shift não conta em nenhum deles).
    #[must_use]
    pub fn keeps(self, key: KeyCode, ctrl: bool, alt: bool) -> bool {
        let apaga = matches!(key, K::Delete | K::Backspace) && !ctrl;
        let hierarquia = apaga || (key == K::KeyD && ctrl);
        let dope = apaga
            || (key == K::KeyM && !ctrl && !alt)
            || (ctrl
                && matches!(
                    key,
                    K::KeyC | K::KeyX | K::KeyV | K::KeyD | K::KeyR | K::KeyE
                ));
        (self.hierarchy && hierarquia) || (self.dope_sheet && dope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SHADOWED;

    const HIER: PanelUnderPointer = PanelUnderPointer {
        hierarchy: true,
        dope_sheet: false,
    };
    const DOPE: PanelUnderPointer = PanelUnderPointer {
        hierarchy: false,
        dope_sheet: true,
    };
    const CANVAS: PanelUnderPointer = PanelUnderPointer {
        hierarchy: false,
        dope_sheet: false,
    };

    /// O passo do dono: `Ctrl+D` sobre a Hierarquia duplica a linha (o Pixel cede); sobre o canvas
    /// é o Deselect do Pixel (ninguém o reclama).
    #[test]
    fn ctrl_d_over_the_hierarchy_is_the_hierarchys() {
        assert!(HIER.keeps(K::KeyD, true, false));
        assert!(DOPE.keeps(K::KeyD, true, false));
        assert!(!CANVAS.keeps(K::KeyD, true, false));
        // As teclas do Pixel que não são de painel continuam dele mesmo sobre a Hierarquia.
        assert!(!HIER.keeps(K::KeyB, false, false));
        assert!(!HIER.keeps(K::KeyM, false, false));
        assert!(!HIER.keeps(K::KeyC, true, false));
    }

    /// ⭐ Ligado à tabela: toda linha de `SHADOWED` cujo dono é um painel por área cede ao painel.
    #[test]
    fn every_shadowed_panel_chord_goes_to_the_panel_under_the_pointer() {
        let mut vistos = 0;
        for &(c, onde, _) in SHADOWED {
            for (ficheiro, painel) in [("keyboard_hierarchy", HIER), ("keyboard_timeline", DOPE)] {
                if onde.contains(ficheiro) {
                    vistos += 1;
                    assert!(painel.keeps(c.key, c.ctrl, c.alt), "{c} ({onde})");
                    assert!(!CANVAS.keeps(c.key, c.ctrl, c.alt), "{c} no canvas");
                }
            }
        }
        assert!(vistos >= 10, "controlo positivo: {vistos} linhas de painel");
    }
}
