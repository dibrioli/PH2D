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
use crate::replica::sealed_field;
use crate::{Lamport, PeerId, Write};

#[derive(Clone, Debug, Default)]
pub(crate) struct Vault {
    /// Por (nota, registo — o texto e a caixa): o autor e a última escrita selada.
    notes: BTreeMap<(ElementId, Field), (PeerId, Write)>,
    ballots: BTreeMap<PeerId, Ballot>,
}

impl Vault {
    /// Guarda uma escrita selada de `author`. Fica a de carimbo maior (reenviar é idempotente); uma nota
    /// tem UM dono até revelar — o selado de outra pessoa sobre ela (que não a lê) não entra. `false`
    /// = não entrou.
    pub(crate) fn keep(&mut self, author: PeerId, w: Write) -> bool {
        let Some((id, f, at)) = sealed_field(&w) else {
            return false;
        };
        if self
            .notes
            .iter()
            .any(|((n, _), (a, _))| *n == id && *a != author)
        {
            return false;
        }
        let newer = self
            .notes
            .get(&(id, f))
            .and_then(|(_, old)| sealed_field(old))
            .is_none_or(|(_, _, old)| at > old);
        if newer {
            self.notes.insert((id, f), (author, w));
        }
        true
    }

    /// Uma escrita às claras mais nova passou à frente do selado deste registo: ele já não sai.
    /// `true` = saiu do cofre (o que todos vêem escondido mudou).
    pub(crate) fn overtaken(&mut self, id: ElementId, f: Field, at: Lamport) -> bool {
        let old = self.notes.get(&(id, f)).and_then(|(_, w)| sealed_field(w));
        old.is_some_and(|(_, _, old)| at > old) && self.notes.remove(&(id, f)).is_some()
    }

    /// As notas escondidas (sem autor — é o que vai a todos).
    pub(crate) fn hidden(&self) -> BTreeSet<ElementId> {
        self.notes.keys().map(|(id, _)| *id).collect()
    }

    /// As notas escondidas que `peer` não escreveu (não as lê).
    pub(crate) fn hidden_to(&self, peer: PeerId) -> BTreeSet<ElementId> {
        self.notes
            .iter()
            .filter(|(_, (a, _))| *a != peer)
            .map(|((id, _), _)| *id)
            .collect()
    }

    /// PUBLICAR: solta as notas de todos (`None`) ou só as de `author`.
    pub(crate) fn release(&mut self, author: Option<PeerId>) -> Vec<Write> {
        let keys: Vec<(ElementId, Field)> = self
            .notes
            .iter()
            .filter(|(_, (a, _))| author.is_none_or(|x| x == *a))
            .map(|(k, _)| *k)
            .collect();
        keys.into_iter()
            .filter_map(|k| self.notes.remove(&k).map(|(_, w)| w))
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
        // Um voto NOVO numa nota que não leio não entra; o que lá estava antes de ela se esconder fica
        // (e pode sair).
        let unread = self.hidden_to(peer);
        let before = |id: &ElementId| {
            self.ballots
                .get(&peer)
                .and_then(|o| o.votes.get(id))
                .copied()
                .unwrap_or(0)
        };
        let ok = b.used().is_some_and(|n| n <= setup.votes)
            && b.votes.iter().all(|(id, n)| {
                targets.contains(id)
                    && (!setup.one_per_object || *n <= 1)
                    && (!unread.contains(id) || *n <= before(id))
            });
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
        self.ballots
            .iter()
            .map(|(p, b)| (*p, b.used().unwrap_or(0)))
            .collect()
    }

    /// SOMAR: os totais por alvo (de todos os boletins — quem saiu conta, Scrumlr v03) e quantas
    /// pessoas votaram; o cofre dos votos fica vazio.
    pub(crate) fn tally(&mut self) -> (BTreeMap<ElementId, u32>, u32) {
        let mut t: BTreeMap<ElementId, u32> = BTreeMap::new();
        let mut voters = 0u32;
        for b in std::mem::take(&mut self.ballots).into_values() {
            if b.votes.values().any(|&n| n > 0) {
                voters += 1;
            }
            for (id, n) in b.votes {
                let s = t.entry(id).or_default();
                *s = s.saturating_add(n);
            }
        }
        t.retain(|_, n| *n > 0);
        (t, voters)
    }
}
