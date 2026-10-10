//! **Os carimbos de uma nota numa sessão**: cada carimbo dado é uma FICHA (o carimbo de Lamport de quem
//! o deu); tirar um é tirar uma ficha. A contagem é o número de fichas vivas — a ordem e a duplicação
//! não a mudam, e nunca fica negativa (um conjunto observado-removido). A lista da nota segue a ordem
//! em que cada carimbo NASCEU (a ficha mais antiga), a mesma em todos os pares.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Element, ElementId, Stamp, StampCount};
use serde::{Deserialize, Serialize};

use crate::Lamport;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Tokens {
    live: BTreeMap<(ElementId, Stamp), BTreeSet<Lamport>>,
    taken: BTreeSet<Lamport>,
    /// Os elementos cujas fichas de partida (as contagens que já tinham) já se puseram.
    seeded: BTreeSet<ElementId>,
}

impl Tokens {
    /// Põe as fichas de partida de `el` (as contagens que já tinha) na 1.ª vez que se mexe nele.
    /// Iguais em todos os pares: a ficha é tirada da POSIÇÃO na lista, com contador zero.
    pub(crate) fn seed(&mut self, el: &Element) {
        if !self.seeded.insert(el.id) {
            return;
        }
        for (pos, c) in el.stamps().iter().enumerate() {
            let set = self.live.entry((el.id, c.stamp)).or_default();
            for j in 0..u64::from(c.count) {
                set.insert(Lamport {
                    n: 0,
                    peer: ((pos as u64) << 32) | j,
                });
            }
        }
    }

    pub(crate) fn give(&mut self, id: ElementId, s: Stamp, tok: Lamport) {
        if !self.taken.contains(&tok) {
            self.live.entry((id, s)).or_default().insert(tok);
        }
    }

    pub(crate) fn take(&mut self, id: ElementId, s: Stamp, tok: Lamport) {
        self.taken.insert(tok);
        if let Some(set) = self.live.get_mut(&(id, s)) {
            set.remove(&tok);
        }
    }

    /// A ficha que um «tirar» local leva: a mais nova.
    pub(crate) fn pick(&self, id: ElementId, s: Stamp) -> Option<Lamport> {
        self.live.get(&(id, s))?.last().copied()
    }

    /// A lista de carimbos de `id`, pela ordem de nascença.
    pub(crate) fn list(&self, id: ElementId) -> Vec<StampCount> {
        let mut v: Vec<(Lamport, StampCount)> = Stamp::ALL
            .into_iter()
            .filter_map(|s| {
                let set = self.live.get(&(id, s))?;
                let first = *set.first()?;
                Some((
                    first,
                    StampCount {
                        stamp: s,
                        count: u32::try_from(set.len()).unwrap_or(u32::MAX),
                    },
                ))
            })
            .collect();
        v.sort_by_key(|(first, _)| *first);
        v.into_iter().map(|(_, c)| c).collect()
    }
}
