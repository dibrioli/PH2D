//! **O cofre do anfitrião** ([ADR-0182] §1): o que é de UMA pessoa e ainda não se mostra — o texto
//! selado das notas escritas em modo privado e o boletim de cada votante. Mora fora do quadro e da
//! réplica do anfitrião: nada daqui entra num retrato nem numa escrita para outro até ser SOLTO, e
//! solta-se de duas maneiras — PUBLICAR (as notas: as escritas saem como estavam) ou SOMAR (os
//! votos: sai só o total).
//!
//! [ADR-0182]: ../../../docs/architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{ElementId, Field, VoteSetup};

use crate::fac::Ballot;
use crate::{PeerId, Write};

#[derive(Clone, Debug, Default)]
pub(crate) struct Vault {
    /// Por nota: o autor e a última escrita selada do texto dela.
    notes: BTreeMap<ElementId, (PeerId, Write)>,
    ballots: BTreeMap<PeerId, Ballot>,
}

/// O texto de uma nota numa escrita selada (a única forma que se sela).
pub(crate) fn sealed_text(w: &Write) -> Option<(ElementId, crate::Lamport)> {
    match w {
        Write::Set {
            id,
            field: Field::Text,
            at,
            ..
        } => Some((*id, *at)),
        _ => None,
    }
}

impl Vault {
    /// Guarda uma escrita selada de `author`. Fica a de carimbo maior (reenviar é idempotente).
    /// `false` = não era uma escrita selável.
    pub(crate) fn keep(&mut self, author: PeerId, w: Write) -> bool {
        let Some((id, at)) = sealed_text(&w) else {
            return false;
        };
        let newer = self
            .notes
            .get(&id)
            .and_then(|(_, old)| sealed_text(old))
            .is_none_or(|(_, old)| at > old);
        if newer {
            self.notes.insert(id, (author, w));
        }
        true
    }

    /// As notas escondidas (sem autor — é o que vai a todos).
    pub(crate) fn hidden(&self) -> BTreeSet<ElementId> {
        self.notes.keys().copied().collect()
    }

    /// As notas escondidas que `peer` não escreveu (não as lê).
    pub(crate) fn hidden_to(&self, peer: PeerId) -> BTreeSet<ElementId> {
        self.notes
            .iter()
            .filter(|(_, (a, _))| *a != peer)
            .map(|(id, _)| *id)
            .collect()
    }

    /// PUBLICAR: solta as notas de todos (`None`) ou só as de `author`.
    pub(crate) fn release(&mut self, author: Option<PeerId>) -> Vec<Write> {
        let ids: Vec<ElementId> = self
            .notes
            .iter()
            .filter(|(_, (a, _))| author.is_none_or(|x| x == *a))
            .map(|(id, _)| *id)
            .collect();
        ids.into_iter()
            .filter_map(|id| self.notes.remove(&id).map(|(_, w)| w))
            .collect()
    }

    /// O boletim de `peer`, se cumpre as regras da votação (só alvos, nunca mais votos do que tem,
    /// um por objecto se assim foi aberta). `false` = recusado (fica o anterior).
    pub(crate) fn ballot(
        &mut self,
        peer: PeerId,
        b: Ballot,
        setup: VoteSetup,
        targets: &BTreeSet<ElementId>,
    ) -> bool {
        let ok = b.used() <= setup.votes
            && b.votes
                .iter()
                .all(|(id, n)| targets.contains(id) && (!setup.one_per_object || *n <= 1));
        if ok {
            self.ballots.insert(peer, b);
        }
        ok
    }

    pub(crate) fn done(&self) -> BTreeSet<PeerId> {
        self.ballots
            .iter()
            .filter(|(_, b)| b.done)
            .map(|(p, _)| *p)
            .collect()
    }

    pub(crate) fn used(&self) -> BTreeMap<PeerId, u32> {
        self.ballots.iter().map(|(p, b)| (*p, b.used())).collect()
    }

    /// SOMAR: os totais por alvo (de todos os boletins — quem saiu conta, Scrumlr v03) e o cofre dos
    /// votos fica vazio.
    pub(crate) fn tally(&mut self) -> BTreeMap<ElementId, u32> {
        let mut t: BTreeMap<ElementId, u32> = BTreeMap::new();
        for b in std::mem::take(&mut self.ballots).into_values() {
            for (id, n) in b.votes {
                *t.entry(id).or_default() += n;
            }
        }
        t.retain(|_, n| *n > 0);
        t
    }
}
