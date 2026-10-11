//! **A equipa numa sessão** (os comentários em fio, as versões nomeadas, os frames fora da
//! apresentação): fios, mensagens e versões são REGISTOS com identidade (`ThreadId`, `MessageId`,
//! `VersionId`); apagar deixa lápide (nada ressuscita); texto, resolver, alfinete e nome são registos
//! LWW; o «lido» é uma marca por PESSOA (até que mensagem leu), e por isso duas pessoas a ler ao mesmo
//! tempo não se apagam. A ordem é a mesma em todos os pares: fios pela 1.ª mensagem, mensagens e
//! versões pelo instante e pelo id. A equipa é pequena: captura-se por comparação com a sombra. As
//! votações acabadas (W13, ADR-0183) são registos como as versões, e a sessão mostrada um registo LWW.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{
    ElementId, Message, MessageId, Pin, Poll, PollId, Team, Thread, ThreadId, Version, VersionId,
};
use serde::{Deserialize, Serialize};

use crate::Lamport;

/// Uma mudança da equipa que viaja.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TeamWrite {
    Thread(Thread),
    Pin(ThreadId, Pin),
    Resolved(ThreadId, bool),
    ThreadGone(ThreadId),
    Message(ThreadId, Message),
    Text(ThreadId, MessageId, String),
    MessageGone(ThreadId, MessageId),
    /// `reader` leu o fio até esta mensagem.
    Read(ThreadId, String, MessageId),
    Version(Version),
    VersionName(VersionId, String),
    VersionGone(VersionId),
    Slide(ElementId, bool),
    /// W13 — uma votação acabada (o instantâneo).
    Poll(Poll),
    PollName(PollId, String),
    PollGone(PollId),
    /// W13 — a sessão que o quadro mostra («Show results on the board»).
    Shown(Option<PollId>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Key {
    Pin(ThreadId),
    Resolved(ThreadId),
    Text(MessageId),
    VersionName(VersionId),
    Slide(ElementId),
    PollName(PollId),
    Shown,
}

/// O que a equipa de uma sessão sabe além do que está na `Team`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamState {
    stamps: BTreeMap<Key, Lamport>,
    reads: BTreeMap<(ThreadId, String), (Lamport, MessageId)>,
    threads_gone: BTreeSet<ThreadId>,
    messages_gone: BTreeSet<MessageId>,
    versions_gone: BTreeSet<VersionId>,
    /// Mensagens de um fio que ainda não chegou aqui.
    pending: Vec<(Lamport, TeamWrite)>,
    polls_gone: BTreeSet<PollId>,
}

/// A sombra: a equipa como estava na última captura (as versões sem os bytes).
#[derive(Clone, Debug, Default)]
pub(crate) struct Shadow {
    threads: Vec<Thread>,
    versions: Vec<(VersionId, String)>,
    slides: BTreeSet<ElementId>,
    polls: Vec<(PollId, String)>,
    shown: Option<PollId>,
}

impl Shadow {
    pub(crate) fn of(team: &Team) -> Self {
        Self {
            threads: team.threads.clone(),
            versions: team
                .versions
                .iter()
                .map(|v| (v.id, v.name.clone()))
                .collect(),
            slides: team.hidden_slides.clone(),
            polls: team.polls.iter().map(|p| (p.id, p.name.clone())).collect(),
            shown: team.shown,
        }
    }
}

impl TeamState {
    /// As marcas de «lido» que o ficheiro já trazia (contador zero: iguais em todos os pares).
    pub(crate) fn seed(&mut self, team: &Team) {
        for t in &team.threads {
            let Some(last) = t.messages.last() else {
                continue;
            };
            for r in &t.read_by {
                self.reads
                    .entry((t.id, r.clone()))
                    .or_insert((Lamport::default(), last.id));
            }
        }
    }

    /// O que mudou desde a sombra, como escritas (cada uma com o seu carimbo, de `tick`). Aplicar
    /// de novo à `team` uma mudança que ela já tem não muda nada: é o que regista os carimbos.
    pub(crate) fn capture(
        &mut self,
        team: &mut Team,
        shadow: &Shadow,
        mut tick: impl FnMut() -> Lamport,
    ) -> Vec<(Lamport, TeamWrite)> {
        let writes = self.diff(team, shadow);
        let out: Vec<(Lamport, TeamWrite)> = writes.into_iter().map(|w| (tick(), w)).collect();
        for (at, w) in &out {
            self.apply(team, *at, w);
        }
        if !out.is_empty() {
            self.settle(team);
        }
        out
    }

    fn diff(&self, team: &Team, shadow: &Shadow) -> Vec<TeamWrite> {
        let mut out = Vec::new();
        let old: BTreeMap<ThreadId, &Thread> = shadow.threads.iter().map(|t| (t.id, t)).collect();
        for t in &team.threads {
            let id = t.id;
            let last = t.messages.last().map(|m| m.id);
            let Some(o) = old.get(&id) else {
                out.push(TeamWrite::Thread(t.clone()));
                if let Some(last) = last {
                    out.extend(
                        t.read_by
                            .iter()
                            .map(|r| TeamWrite::Read(id, r.clone(), last)),
                    );
                }
                continue;
            };
            if t.pin != o.pin {
                out.push(TeamWrite::Pin(id, t.pin));
            }
            if t.resolved != o.resolved {
                out.push(TeamWrite::Resolved(id, t.resolved));
            }
            let om: BTreeMap<MessageId, &Message> = o.messages.iter().map(|m| (m.id, m)).collect();
            for m in &t.messages {
                match om.get(&m.id) {
                    None => out.push(TeamWrite::Message(id, m.clone())),
                    Some(o) if o.text != m.text => {
                        out.push(TeamWrite::Text(id, m.id, m.text.clone()));
                    }
                    Some(_) => {}
                }
            }
            let nm: BTreeSet<MessageId> = t.messages.iter().map(|m| m.id).collect();
            out.extend(
                om.keys()
                    .filter(|m| !nm.contains(m))
                    .map(|m| TeamWrite::MessageGone(id, *m)),
            );
            if let Some(last) = last {
                for r in &t.read_by {
                    if self.reads.get(&(id, r.clone())).map(|x| x.1) != Some(last) {
                        out.push(TeamWrite::Read(id, r.clone(), last));
                    }
                }
            }
        }
        let now: BTreeSet<ThreadId> = team.threads.iter().map(|t| t.id).collect();
        out.extend(
            old.keys()
                .filter(|id| !now.contains(id))
                .map(|id| TeamWrite::ThreadGone(*id)),
        );
        let ov: BTreeMap<VersionId, &String> =
            shadow.versions.iter().map(|(i, n)| (*i, n)).collect();
        for v in &team.versions {
            match ov.get(&v.id) {
                None => out.push(TeamWrite::Version(v.clone())),
                Some(n) if **n != v.name => out.push(TeamWrite::VersionName(v.id, v.name.clone())),
                Some(_) => {}
            }
        }
        let nv: BTreeSet<VersionId> = team.versions.iter().map(|v| v.id).collect();
        out.extend(
            ov.keys()
                .filter(|id| !nv.contains(id))
                .map(|id| TeamWrite::VersionGone(*id)),
        );
        out.extend(
            team.hidden_slides
                .symmetric_difference(&shadow.slides)
                .map(|f| TeamWrite::Slide(*f, team.hidden_slides.contains(f))),
        );
        let op: BTreeMap<PollId, &String> = shadow.polls.iter().map(|(i, n)| (*i, n)).collect();
        for p in &team.polls {
            match op.get(&p.id) {
                None => out.push(TeamWrite::Poll(p.clone())),
                Some(n) if **n != p.name => out.push(TeamWrite::PollName(p.id, p.name.clone())),
                Some(_) => {}
            }
        }
        let np: BTreeSet<PollId> = team.polls.iter().map(|p| p.id).collect();
        out.extend(
            op.keys()
                .filter(|id| !np.contains(id))
                .map(|id| TeamWrite::PollGone(*id)),
        );
        // Depois da sessão: quem recebe já a tem quando a passa a mostrar.
        if team.shown != shadow.shown {
            out.push(TeamWrite::Shown(team.shown));
        }
        out
    }

    /// Funde escritas que chegaram.
    pub(crate) fn receive(&mut self, team: &mut Team, writes: &[(Lamport, TeamWrite)]) {
        let mut todo: Vec<(Lamport, TeamWrite)> = std::mem::take(&mut self.pending);
        todo.extend_from_slice(writes);
        // Até não haver progresso: uma mensagem pode vir antes do fio no mesmo lote.
        loop {
            let before = todo.len();
            todo.retain(|(at, w)| !self.apply(team, *at, w));
            if todo.is_empty() || todo.len() == before {
                break;
            }
        }
        self.pending = todo;
        self.settle(team);
    }

    /// Aplica UMA escrita. `false` = o fio dela ainda não chegou aqui.
    fn apply(&mut self, team: &mut Team, at: Lamport, w: &TeamWrite) -> bool {
        let lww = |s: &mut Self, k: Key| -> bool {
            let win = at > s.stamps.get(&k).copied().unwrap_or_default();
            if win {
                s.stamps.insert(k, at);
            }
            win
        };
        match w {
            TeamWrite::Thread(t) => {
                if self.threads_gone.contains(&t.id) {
                    return true;
                }
                lww(self, Key::Pin(t.id));
                lww(self, Key::Resolved(t.id));
                match team.thread_mut(t.id) {
                    Some(have) => {
                        for m in &t.messages {
                            if !have.messages.iter().any(|x| x.id == m.id)
                                && !self.messages_gone.contains(&m.id)
                            {
                                have.messages.push(m.clone());
                            }
                        }
                    }
                    None => {
                        let mut t = t.clone();
                        t.messages.retain(|m| !self.messages_gone.contains(&m.id));
                        team.threads.push(t);
                    }
                }
            }
            TeamWrite::Pin(id, _) | TeamWrite::Resolved(id, _) => {
                if self.threads_gone.contains(id) {
                    return true;
                }
                let Some(t) = team.thread_mut(*id) else {
                    return false;
                };
                let key = match w {
                    TeamWrite::Pin(..) => Key::Pin(*id),
                    _ => Key::Resolved(*id),
                };
                if at > self.stamps.get(&key).copied().unwrap_or_default() {
                    self.stamps.insert(key, at);
                    match w {
                        TeamWrite::Pin(_, p) => t.pin = *p,
                        TeamWrite::Resolved(_, r) => t.resolved = *r,
                        _ => unreachable!("o braço"),
                    }
                }
            }
            TeamWrite::ThreadGone(id) => {
                self.threads_gone.insert(*id);
                team.threads.retain(|t| t.id != *id);
            }
            TeamWrite::Message(id, m) => {
                if self.threads_gone.contains(id) || self.messages_gone.contains(&m.id) {
                    return true;
                }
                let Some(t) = team.thread_mut(*id) else {
                    return false;
                };
                if !t.messages.iter().any(|x| x.id == m.id) {
                    t.messages.push(m.clone());
                }
            }
            TeamWrite::Text(id, m, text) => {
                if self.threads_gone.contains(id) || self.messages_gone.contains(m) {
                    return true;
                }
                let Some(x) = team
                    .thread_mut(*id)
                    .and_then(|t| t.messages.iter_mut().find(|x| x.id == *m))
                else {
                    return false;
                };
                if lww(self, Key::Text(*m)) {
                    x.text.clone_from(text);
                    x.edited = true;
                }
            }
            TeamWrite::MessageGone(id, m) => {
                self.messages_gone.insert(*m);
                if let Some(t) = team.thread_mut(*id) {
                    t.messages.retain(|x| x.id != *m);
                }
            }
            TeamWrite::Read(id, r, upto) => {
                let mark = self.reads.entry((*id, r.clone())).or_default();
                if at > mark.0 {
                    *mark = (at, *upto);
                }
            }
            TeamWrite::Version(v) => {
                if !self.versions_gone.contains(&v.id) {
                    lww(self, Key::VersionName(v.id));
                    if !team.versions.iter().any(|x| x.id == v.id) {
                        team.versions.push(v.clone());
                    }
                }
            }
            TeamWrite::VersionName(id, name) => {
                if self.versions_gone.contains(id) {
                    return true;
                }
                let Some(v) = team.versions.iter_mut().find(|v| v.id == *id) else {
                    return false;
                };
                if lww(self, Key::VersionName(*id)) {
                    v.name.clone_from(name);
                }
            }
            TeamWrite::VersionGone(id) => {
                self.versions_gone.insert(*id);
                team.versions.retain(|v| v.id != *id);
            }
            TeamWrite::Slide(f, hidden) => {
                if lww(self, Key::Slide(*f)) {
                    team.set_slide_hidden(*f, *hidden);
                }
            }
            TeamWrite::Poll(p) => {
                if !self.polls_gone.contains(&p.id) {
                    lww(self, Key::PollName(p.id));
                    if team.poll(p.id).is_none() {
                        team.polls.push(p.clone());
                    }
                }
            }
            TeamWrite::PollName(id, name) => {
                if self.polls_gone.contains(id) {
                    return true;
                }
                let Some(p) = team.polls.iter_mut().find(|p| p.id == *id) else {
                    return false;
                };
                if lww(self, Key::PollName(*id)) {
                    p.name.clone_from(name);
                }
            }
            TeamWrite::PollGone(id) => {
                self.polls_gone.insert(*id);
                team.remove_poll(*id);
            }
            // Pode chegar antes da sessão (lê-se como nenhuma até ela chegar); uma apagada, nunca.
            TeamWrite::Shown(id) => {
                if lww(self, Key::Shown) {
                    team.shown = id.filter(|id| !self.polls_gone.contains(id));
                }
            }
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn probe(&self, id: ThreadId) -> (bool, usize) {
        (self.threads_gone.contains(&id), self.pending.len())
    }

    /// Escritas à espera de um fio, mensagem ou versão que ainda não chegou.
    pub(crate) fn pending(&self) -> usize {
        self.pending.len()
    }

    /// A forma ÚNICA da equipa: a ordem de fios, mensagens e versões, e o «lido» das marcas.
    pub(crate) fn settle(&self, team: &mut Team) {
        for t in &mut team.threads {
            t.messages.sort_by_key(|m| (m.at, m.id));
            let last = t.messages.last().map(|m| m.id);
            let mut read: Vec<String> = self
                .reads
                .range((t.id, String::new())..)
                .take_while(|((id, _), _)| *id == t.id)
                .filter(|(_, (_, upto))| Some(*upto) == last)
                .map(|((_, r), _)| r.clone())
                .collect();
            read.sort();
            t.read_by = read;
        }
        team.threads.retain(|t| !t.messages.is_empty());
        team.threads
            .sort_by_key(|t| (t.messages.first().map_or(0, |m| m.at), t.id));
        team.versions.sort_by_key(|v| (v.at, v.id));
        // Pelo número (o «Voting N»), nunca pelo relógio; dois iguais (dois pares acabaram ao mesmo
        // tempo) separam-se pelo fim e pelo id — e o ecrã pinta-os com `Team::label_number`.
        team.polls.sort_by_key(|p| (p.number, p.at, p.id));
        if team.shown.is_some_and(|id| self.polls_gone.contains(&id)) {
            team.shown = None;
        }
    }
}
