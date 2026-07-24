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

    /// **Todo estado de sessão chaveado por QUADRO acompanha a chave MOVIDA** — hoje são
    /// dois: os pins do light table e a **seleção do multiframe**.
    ///
    /// Sem isto o artista fixa (ou marca) o quadro 11, arrasta aquela mesma célula para o
    /// 13, e o fantasma some / o acento apaga — a lista ficou apontando um quadro que não
    /// tem mais chave, e nada na tela explica por quê. O que ele fixou/marcou foi *aquele
    /// desenho*; mover o desenho no tempo não é soltá-lo. Uma porta só, de propósito: o
    /// próximo estado chaveado por quadro entra AQUI, não numa 3ª cópia da regra (foi
    /// exatamente assim que a seleção ficou de fora quando os pins ganharam a sua).
    pub(crate) fn remap_session_after_move(&mut self, from: Frame, to: Frame) {
        for list in [&mut self.pinned, &mut self.selection] {
            if let Some(i) = list.iter().position(|&k| k == from) {
                list[i] = to;
                list.sort_unstable();
                list.dedup();
            }
        }
    }

    /// **E acompanha o EMPURRÃO da exposição.**
    ///
    /// `set_exposure` desloca todas as chaves depois de `key` pelo delta (é a semântica da
    /// tira de exposição: esticar empurra, encolher puxa). Um pin ou uma marca numa dessas
    /// chaves tem de ir junto — e este é o caso comum, não o exótico: esticar o primeiro
    /// quadro empurra a fila inteira, então um único gesto orfanaria TODOS os pins e
    /// marcas à direita.
    pub(crate) fn remap_session_after_hold(&mut self, key: Frame, delta: i32) {
        if delta == 0 {
            return;
        }
        for list in [&mut self.pinned, &mut self.selection] {
            for k in list.iter_mut() {
                if *k > key {
                    *k += delta;
                }
            }
            list.sort_unstable();
            list.dedup();
        }
    }
}
