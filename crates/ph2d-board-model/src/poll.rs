//! **O histórico das votações** (W13, [ADR-0183]): cada votação acabada é um INSTANTÂNEO na equipa
//! do quadro — os totais por elemento no momento do fim, nunca recontados (os boletins não saem do
//! cofre do anfitrião, ADR-0182 §7). O selo do quadro deriva da sessão MOSTRADA ([`Team::shown`]).
//!
//! [ADR-0183]: ../../../docs/architecture/decisions/0183-o-historico-das-votacoes-e-um-instantaneo-da-equipa.md

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ElementId, Team};

/// Identidade de uma sessão de votação dentro do seu quadro (sai do [`Team::mint`]). Nunca reusada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PollId(pub u64);

/// Uma votação acabada.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Poll {
    pub id: PollId,
    /// O `N` de «Voting N» (1 + o maior vivo quando nasceu).
    pub number: u32,
    /// O nome que o artista lhe deu; vazio = sem nome (o ecrã pinta «Voting N» no idioma dele).
    pub name: String,
    /// O instante do fim, em segundos UNIX; `0` = sem data (os votos de um ficheiro antigo).
    pub at: i64,
    /// Quem a iniciou; vazio = anónimo («Make names anonymous») ou desconhecido.
    pub by: String,
    /// Quantas pessoas votaram.
    pub voters: u32,
    /// Os totais por elemento no fim, só os `> 0`.
    pub totals: BTreeMap<ElementId, u32>,
}

impl Poll {
    /// A soma dos votos.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.totals.values().sum()
    }

    /// Os elementos votados, do mais votado ao menos (empate: o id mais antigo primeiro).
    #[must_use]
    pub fn ranked(&self) -> Vec<(ElementId, u32)> {
        let mut v: Vec<_> = self.totals.iter().map(|(&id, &n)| (id, n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}

impl Team {
    /// Guarda uma votação acabada e passa a mostrá-la (o resultado aparece a todos).
    pub fn add_poll(
        &mut self,
        totals: BTreeMap<ElementId, u32>,
        by: &str,
        voters: u32,
        at: i64,
    ) -> PollId {
        let id = PollId(self.mint());
        let number = self.labels().last().map_or(0, |&(_, n)| n) + 1;
        self.polls.push(Poll {
            id,
            number,
            name: String::new(),
            at,
            by: by.to_owned(),
            voters,
            totals: totals.into_iter().filter(|&(_, n)| n > 0).collect(),
        });
        self.shown = Some(id);
        id
    }

    /// O `N` que o ecrã pinta em «Voting N»: o número guardado, a menos que dois pares tenham acabado
    /// uma votação ao mesmo tempo (o MESMO número) — pela ordem (número, fim, id) cada um leva pelo menos
    /// o seguinte do anterior. Uma função do CONJUNTO, nunca gravada: igual em todos os pares, qualquer
    /// que seja a ordem em que as sessões chegaram (renumerar e gravar divergia — portão W13). `0` = não
    /// existe.
    #[must_use]
    pub fn label_number(&self, id: PollId) -> u32 {
        self.labels()
            .into_iter()
            .find(|&(p, _)| p == id)
            .map_or(0, |(_, n)| n)
    }

    /// `(sessão, N)` pela ordem (número, fim, id).
    fn labels(&self) -> Vec<(PollId, u32)> {
        let mut v: Vec<&Poll> = self.polls.iter().collect();
        v.sort_by_key(|p| (p.number, p.at, p.id));
        let mut last = 0;
        v.into_iter()
            .map(|p| {
                last = p.number.max(last + 1);
                (p.id, last)
            })
            .collect()
    }

    #[must_use]
    pub fn poll(&self, id: PollId) -> Option<&Poll> {
        self.polls.iter().find(|p| p.id == id)
    }

    /// As sessões da mais nova à mais velha (Miro: «Voting 4» em cima).
    pub fn polls_newest_first(&self) -> impl Iterator<Item = &Poll> {
        self.polls.iter().rev()
    }

    /// Dá nome a uma sessão (espaços nas pontas fora; vazio = volta a «Voting N»). `false` se não existe
    /// ou o nome não muda.
    pub fn rename_poll(&mut self, id: PollId, name: &str) -> bool {
        let name = name.trim();
        match self.polls.iter_mut().find(|p| p.id == id) {
            Some(p) if p.name != name => {
                name.clone_into(&mut p.name);
                true
            }
            _ => false,
        }
    }

    /// Apaga uma sessão; se era a mostrada, o quadro deixa de mostrar resultados. `false` se não existe.
    pub fn remove_poll(&mut self, id: PollId) -> bool {
        let before = self.polls.len();
        self.polls.retain(|p| p.id != id);
        if self.shown == Some(id) {
            self.shown = None;
        }
        self.polls.len() != before
    }

    /// Que sessão o quadro mostra («Show results on the board»); `None` = nenhuma. Uma que não existe
    /// lê-se como nenhuma.
    pub fn show_poll(&mut self, id: Option<PollId>) {
        self.shown = id.filter(|&id| self.poll(id).is_some());
    }

    /// A sessão que o quadro mostra.
    #[must_use]
    pub fn shown_poll(&self) -> Option<&Poll> {
        self.shown.and_then(|id| self.poll(id))
    }

    /// Os votos de um ficheiro de antes do histórico (formato ≤ 10, `.excalidraw` com `customData`):
    /// UMA sessão, sem data nem autor, mostrada. Nada a fazer se não há votos.
    pub fn adopt_old_votes(&mut self, totals: BTreeMap<ElementId, u32>) {
        if totals.values().any(|&n| n > 0) {
            self.add_poll(totals, "", 0, 0);
        }
    }
}

#[cfg(test)]
#[path = "poll_tests.rs"]
mod tests;
