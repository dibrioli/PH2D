//! ⭐ J4 — **o retrato fora da janela**. Na janela só se CLONA o que o retrato lê (os Kanbans da sala e
//! os mapas de carimbos); escrever os bytes, compactar os carimbos e partir em blocos corre num fio
//! ([`Portrait::messages`]). Medido a 20 000 cartões (MEDICOES.md §J4): 39 ms → 9,4 ms na janela.
//! Quem entra guarda as escritas que chegam antes do retrato (`GuestRoom`), por isso a ordem entre o
//! fio e as mensagens da janela não importa.

use std::collections::BTreeMap;

use ph2d_board_sync::{ConnId, Lamport};
use ph2d_kanban_model::{Kanban, KanbanId, KanbanSet};

use crate::msg::{Elem, ElemKey, FieldKey, KMsg, KSnapshot, Target};
use crate::replica::KanbanReplica;

/// O que o retrato de quem entra lê, já copiado (é `Send`: vai para um fio).
#[derive(Clone, Debug)]
pub struct Portrait {
    /// A ligação de quem entra (as mensagens vão só a ela).
    pub conn: ConnId,
    kanbans: Vec<(KanbanId, Kanban)>,
    stamps: BTreeMap<FieldKey, Lamport>,
    elems: BTreeMap<ElemKey, Elem>,
    tops: BTreeMap<(KanbanId, Target), Lamport>,
    clock: u64,
}

impl Portrait {
    /// ⭐ No fio: o retrato em mensagens (uma, ou os blocos de [`crate::SNAPSHOT_BLOCK`]).
    #[must_use]
    pub fn messages(self) -> Vec<KMsg> {
        let snap = KSnapshot {
            kanbans: self
                .kanbans
                .iter()
                .map(|(w, k)| (*w, crate::replica::bytes_of(k, *w)))
                .collect(),
            stamps: crate::snapshot::pack(&self.stamps, &self.elems, &self.tops),
            clock: self.clock,
        };
        crate::snapshot::messages(snap, crate::SNAPSHOT_BLOCK)
    }
}

impl KanbanReplica {
    /// Na janela: a cópia de que o retrato precisa (chame [`Self::local`] antes, como no
    /// [`Self::snapshot`]).
    #[must_use]
    pub fn portrait(&self, set: &KanbanSet, conn: ConnId) -> Portrait {
        Portrait {
            conn,
            kanbans: self
                .map
                .iter()
                .filter_map(|(w, l)| set.get(*l).map(|k| (*w, k.clone())))
                .collect(),
            stamps: self.stamps.clone(),
            elems: self.elems.clone(),
            tops: self.tops.clone(),
            clock: self.clock.n,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{K, base};
    use crate::{GuestRoom, HostRoom, KMsg, KanbanReplica};

    /// O retrato pelo fio é o MESMO retrato (os mesmos bytes), e quem entra por ele converge mesmo
    /// que as escritas da janela cheguem ANTES dele.
    #[test]
    fn a_portrait_off_the_window_is_the_same_snapshot_and_writes_before_it_wait() {
        let mut hs = base();
        let mut host = HostRoom::new(KanbanReplica::host(1, 11, &hs, &[K]));
        let _ = host.replica.local(&mut hs);
        let same = crate::snapshot::messages(host.replica.snapshot(&hs), crate::SNAPSHOT_BLOCK);
        assert_eq!(host.replica.portrait(&hs, 5).messages(), same);

        host.portraits_off_window();
        let now = host.welcome(&mut hs, 5);
        assert!(
            !now.iter()
                .any(|(_, m)| matches!(m, KMsg::Snapshot(_) | KMsg::SnapshotPart { .. })),
            "o retrato não sai na janela"
        );
        let mut ps = host.take_portraits();
        assert_eq!(ps.len(), 1);
        let p = ps.pop().unwrap();
        assert_eq!(p.conn, 5);
        // A Ana mexe depois do retrato: a escrita chega à Bia ANTES dele.
        let id = *crate::test_support::kanban(&hs)
            .cards
            .keys()
            .next()
            .unwrap();
        crate::test_support::edit_card(&mut hs, id, |c| c.title = "depois".into());
        let early = host.tick(&mut hs, 0);
        let mut gs = ph2d_kanban_model::KanbanSet::default();
        let mut g = GuestRoom::new(2, 7);
        let _ = g.ask();
        for (_, m) in early.into_iter().chain(now) {
            let _ = g.frame(&mut gs, m, 0);
        }
        assert!(g.replica.is_none());
        let handle = std::thread::spawn(move || p.messages());
        for m in handle.join().unwrap() {
            let _ = g.frame(&mut gs, m, 0);
        }
        assert_eq!(crate::test_support::kanban(&gs).cards[&id].title, "depois");
        assert_eq!(hs.to_bytes().unwrap(), gs.to_bytes().unwrap());
    }
}
