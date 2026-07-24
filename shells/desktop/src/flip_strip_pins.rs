//! O **light table** do lado do shell (T3.9) — módulo-irmão do `flip_strip` pelo cap de LOC.
//!
//! Quais quadros estão fixados como referência, e o toggle que os fixa. É a metade que
//! GUARDA; quem os MOSTRA é o passe de fantasmas (`render_loop::flip_pass_ghosts`, que os
//! recebe pelo `GhostSources`) e quem decide o que fazer com eles é a função pura
//! `ph2d_flip::ghosts`.

use crate::flip_strip::FlipStrip;
use ph2d_flip::Frame;

impl FlipStrip {
    /// As chaves fixadas no light table (o passe de fantasmas as lê SEMPRE).
    pub(crate) fn pinned_keys(&self) -> &[Frame] {
        &self.pinned
    }

    /// Fixa/desafixa `key` no light table. Devolve `true` se passou a estar fixada.
    ///
    /// Um toggle, e não dois botões: o artista aponta o quadro e diz *"este eu quero
    /// ver"* — dizer isso de novo é o que desfaz.
    pub(crate) fn toggle_pin(&mut self, key: Frame) -> bool {
        match self.pinned.iter().position(|&k| k == key) {
            Some(i) => {
                self.pinned.remove(i);
                false
            }
            None => {
                self.pinned.push(key);
                self.pinned.sort_unstable();
                true
            }
        }
    }
}
