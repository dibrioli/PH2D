//! **A cadeira do anfitrião** ([ADR-0182]), módulo filho de [`super`]: o estado da facilitação que
//! todos vêem, o cofre, os dois relógios (revelar; o fim da votação) e quem pode o quê. Só o anfitrião
//! decide; os pedidos de todos (o dele também) passam por [`Host::ask_from`].
//!
//! [ADR-0182]: ../../../docs/architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, ElementId, Field, VOTES_MAX};

use super::{ConnId, Event, Host, Msg, To};
use crate::fac::{Ask, Ballot, Fac, Private, REVEAL_COUNTDOWN_MS, Refusal, VoteRoom};
use crate::replica::sealed_field;
use crate::vault::Vault;
use crate::{PeerId, Write};

/// O que o anfitrião decide da facilitação (ADR-0182): o estado, o cofre e os dois relógios.
#[derive(Debug, Default)]
pub(super) struct Chair {
    fac: Fac,
    vault: Vault,
    reveal_at: Option<u64>,
    vote_until: Option<u64>,
    /// Escritas soltas do cofre, por enviar a todos.
    loose: Vec<Write>,
    /// O estado mudou: sai a cada um no próximo tique, DEPOIS das escritas.
    dirty: bool,
    /// W13 — o nome de quem iniciou a votação aberta (vazio com «Make names anonymous»): fica na
    /// sessão do histórico mesmo que a pessoa saia antes do fim.
    vote_by: String,
    /// W13 — o relógio de parede (segundos UNIX) do instante do fim ([`Host::set_wall`]).
    wall_s: i64,
}

impl Host {
    /// O estado da facilitação como `peer` o vê (`used` só para quem iniciou a votação).
    pub(super) fn fac_for(&self, peer: PeerId, now_ms: u64, board: &Board) -> Fac {
        let c = &self.chair;
        let mut f = c.fac.clone();
        // As que existem (um rascunho apagado não conta como escondido).
        f.hidden = c.vault.hidden();
        f.hidden.retain(|id| board.doc.get(*id).is_some());
        if let Some(p) = &mut f.private {
            p.reveal_in_ms = c.reveal_at.map(|t| t.saturating_sub(now_ms));
        }
        if let Some(v) = &mut f.vote
            && !v.ended
        {
            v.left_ms = c.vote_until.map(|t| t.saturating_sub(now_ms));
            v.done = c.vault.done();
            v.used = if peer == v.by {
                c.vault.used()
            } else {
                BTreeMap::new()
            };
        }
        f
    }

    /// O estado da facilitação como eu (o anfitrião) o vejo.
    #[must_use]
    pub fn fac(&self, now_ms: u64, board: &Board) -> Fac {
        self.fac_for(self.me.peer, now_ms, board)
    }

    /// As minhas notas com texto por revelar.
    #[must_use]
    pub fn my_sealed(&self) -> BTreeSet<ElementId> {
        self.replica.my_sealed()
    }

    /// Quem pode acabar o que `by` começou: o próprio, o anfitrião (o dono da sala) ou, com a opção,
    /// qualquer um.
    fn may_end(&self, from: PeerId, by: PeerId, anyone: bool) -> bool {
        anyone || from == by || from == self.me.peer
    }

    /// Um pedido de `from` (o meu também passa por aqui).
    fn ask_from(
        &mut self,
        board: &mut Board,
        from: PeerId,
        ask: Ask,
        now_ms: u64,
    ) -> Result<(), Refusal> {
        match ask {
            Ask::PrivateOn {
                anonymous,
                anyone_can_end,
            } => {
                if self.chair.fac.private.is_none() {
                    self.chair.fac.private = Some(Private {
                        by: from,
                        anonymous,
                        anyone_can_end,
                        anyone_can_reveal: false,
                        reveal_in_ms: None,
                    });
                    self.replica.set_sealing(true);
                    self.chair.dirty = true;
                }
            }
            Ask::PrivateOff => {
                let Some(p) = &self.chair.fac.private else {
                    return Ok(());
                };
                if !self.may_end(from, p.by, p.anyone_can_end) {
                    return Err(Refusal::OnlyStarterEndsPrivate);
                }
                if self.chair.reveal_at.is_none() {
                    self.chair.reveal_at = Some(now_ms + REVEAL_COUNTDOWN_MS);
                    self.chair.dirty = true;
                }
            }
            Ask::AnyoneCanReveal(on) => {
                let Some(p) = &mut self.chair.fac.private else {
                    return Ok(());
                };
                if from != p.by {
                    return Err(Refusal::OnlyStarterEndsPrivate);
                }
                p.anyone_can_reveal = on;
                self.chair.dirty = true;
            }
            Ask::RevealMine => {
                let Some(p) = &self.chair.fac.private else {
                    return Ok(());
                };
                if !p.anyone_can_reveal {
                    return Err(Refusal::RevealMineOff);
                }
                self.release(board, Some(from));
            }
            Ask::VoteStart { setup, targets } => {
                if self.chair.fac.vote.as_ref().is_some_and(|v| !v.ended) {
                    return Err(Refusal::VoteRunning);
                }
                let targets: BTreeSet<ElementId> = targets
                    .into_iter()
                    .filter(|id| board.doc.get(*id).is_some())
                    .collect();
                if targets.is_empty() {
                    return Err(Refusal::NothingToVote);
                }
                let setup = ph2d_board_model::VoteSetup {
                    votes: setup.votes.clamp(1, VOTES_MAX),
                    ..setup
                };
                let _ = self.chair.vault.tally();
                let anonymous = self.chair.fac.private.as_ref().is_some_and(|p| p.anonymous);
                self.chair.vote_by = if anonymous {
                    String::new()
                } else if from == self.me.peer {
                    self.me.name.clone()
                } else {
                    self.room
                        .peers
                        .get(&from)
                        .map(|p| p.name.clone())
                        .unwrap_or_default()
                };
                self.chair.vote_until =
                    (setup.minutes > 0).then(|| now_ms + u64::from(setup.minutes) * 60_000);
                let round = self.chair.fac.vote.as_ref().map_or(1, |v| v.round + 1);
                self.chair.fac.vote = Some(VoteRoom {
                    round,
                    by: from,
                    setup,
                    targets,
                    left_ms: None,
                    done: BTreeSet::new(),
                    used: BTreeMap::new(),
                    ended: false,
                });
                self.chair.dirty = true;
            }
            Ask::VoteEnd => {
                let Some(v) = self.chair.fac.vote.as_ref().filter(|v| !v.ended) else {
                    return Ok(());
                };
                if !self.may_end(from, v.by, false) {
                    return Err(Refusal::OnlyStarterEndsVote);
                }
                self.end_vote(board);
            }
        }
        Ok(())
    }

    /// PUBLICAR o cofre (todas as notas, ou as de `author`): aplica aqui e põe-nas a sair a todos.
    fn release(&mut self, board: &mut Board, author: Option<PeerId>) {
        let ws = self.chair.vault.release(author);
        if ws.is_empty() {
            return;
        }
        self.replica.receive(board, &ws);
        self.replica.released(
            ws.iter()
                .filter_map(|w| sealed_field(w).map(|(id, _, _)| id)),
        );
        self.chair.loose.extend(ws);
        self.chair.dirty = true;
    }

    /// O relógio de parede (segundos UNIX) com que o fim de uma votação se data (W13). O `now_ms` dos
    /// tiques é monótono, sem data.
    pub fn set_wall(&mut self, unix_s: i64) {
        self.chair.wall_s = unix_s;
    }

    /// SOMAR: a sessão vai para o histórico do quadro como escrita da sala e passa a ser a mostrada
    /// (ADR-0183 §10: UMA escrita da equipa, que sai no tique antes do estado «acabou»).
    fn end_vote(&mut self, board: &mut Board) {
        let (totals, voters) = self.chair.vault.tally();
        let by = std::mem::take(&mut self.chair.vote_by);
        // Ligado a meio também conta: a sessão diria o nome para sempre (auditoria W13).
        let anonymous = self.chair.fac.private.as_ref().is_some_and(|p| p.anonymous);
        let by = if anonymous { String::new() } else { by };
        board.team.add_poll(totals, &by, voters, self.chair.wall_s);
        if let Some(v) = &mut self.chair.fac.vote {
            v.ended = true;
            v.left_ms = None;
            v.done.clear();
            v.used.clear();
        }
        self.chair.vote_until = None;
        self.chair.dirty = true;
    }

    /// Os relógios do anfitrião: a contagem de revelar e o fim da votação.
    fn clock(&mut self, board: &mut Board, now_ms: u64) {
        if self.chair.reveal_at.is_some_and(|t| now_ms >= t) {
            self.reveal_all(board);
        }
        if self.chair.vote_until.is_some_and(|t| now_ms >= t) {
            self.end_vote(board);
        }
    }

    fn reveal_all(&mut self, board: &mut Board) {
        self.chair.reveal_at = None;
        self.chair.fac.private = None;
        self.replica.set_sealing(false);
        self.release(board, None);
        self.chair.dirty = true;
    }

    /// Um pedido MEU. O que fica por enviar sai no próximo [`Self::tick`].
    pub fn ask(&mut self, board: &mut Board, ask: Ask, now_ms: u64) -> Option<Event> {
        self.ask_from(board, self.me.peer, ask, now_ms)
            .err()
            .map(Event::Denied)
    }

    /// O MEU boletim (o anfitrião também vota). `false` = recusado pelas regras.
    pub fn ballot(&mut self, b: Ballot) -> bool {
        let me = self.me.peer;
        self.take_ballot(me, b) != Some(false)
    }

    /// `None` = não é desta votação (uma rodada velha reenviada ao voltar: ignora-se em silêncio);
    /// `Some(false)` = recusado pelas regras (fica o anterior, e quem votou sabe).
    fn take_ballot(&mut self, from: PeerId, b: Ballot) -> Option<bool> {
        let v = self
            .chair
            .fac
            .vote
            .as_ref()
            .filter(|v| !v.ended && v.round == b.round)?;
        let (setup, targets) = (v.setup, v.targets.clone());
        let ok = self.chair.vault.ballot(from, b, setup, &targets);
        self.chair.dirty |= ok;
        Some(ok)
    }

    /// Fecho a sala DE PROPÓSITO: o que estava escondido revela-se e a votação acaba antes do adeus
    /// (o trabalho de ninguém se perde, ADR-0182 §5). Envie isto e depois o [`Msg::End`].
    pub fn close(&mut self, board: &mut Board, now_ms: u64) -> Vec<(To, Msg)> {
        if self.chair.fac.private.is_some() {
            self.reveal_all(board);
        }
        if self.chair.fac.vote.as_ref().is_some_and(|v| !v.ended) {
            self.end_vote(board);
        }
        self.tick(board, now_ms)
    }

    /// As escritas às claras de `from`: com o modo ligado, o TEXTO de uma nota que chega às claras (quem
    /// escreveu ainda não sabia do modo — a corrida da virada) vai ao cofre em vez de a todos, como o
    /// servidor do Scrumlr; e uma escrita mais nova de um registo selado tira o selado do cofre.
    pub(super) fn chair_writes(
        &mut self,
        board: &Board,
        from: PeerId,
        ws: Vec<Write>,
    ) -> Vec<Write> {
        let on = self.chair.fac.private.is_some();
        let mut public = Vec::with_capacity(ws.len());
        for w in ws {
            if let Some((id, f, at)) = sealed_field(&w) {
                let note = board
                    .doc
                    .get(id)
                    .and_then(|e| e.shape())
                    .is_some_and(|s| s.kind.is_note());
                // (Uma nota com outro dono no cofre: o texto é de quem o escreveu às claras e para
                // todos — segue, e passa à frente do selado mais velho.)
                if on
                    && note
                    && f == Field::Text
                    && at > self.replica.public_stamp(id, f)
                    && self.chair.vault.keep(from, w.clone())
                {
                    self.chair.dirty = true;
                    continue;
                }
                self.chair.vault.overtaken(id, f, at);
                self.chair.dirty = true;
            }
            public.push(w);
        }
        public
    }

    /// As mensagens da facilitação de `from` (o selado, o boletim, os pedidos).
    pub(super) fn chair_frame(
        &mut self,
        board: &mut Board,
        conn: ConnId,
        from: PeerId,
        msg: Msg,
        now_ms: u64,
    ) -> Vec<(To, Msg)> {
        let mut out = Vec::new();
        match msg {
            Msg::Sealed(ws) => {
                let mut public = Vec::new();
                for w in ws {
                    // Um selado que não é mais novo do que o que TODOS têm perdeu (ou é um reenvio
                    // depois de revelar): não muda nada em ninguém e NUNCA se retransmite — o texto
                    // dele pode nunca ter sido mostrado. Com o modo ligado vai ao cofre; desligado
                    // (chegou depois da revelação) é uma escrita como as outras.
                    let Some((id, f, at)) = sealed_field(&w) else {
                        continue;
                    };
                    if at <= self.replica.public_stamp(id, f) {
                        continue;
                    }
                    if self.chair.fac.private.is_some() {
                        self.chair.dirty |= self.chair.vault.keep(from, w);
                    } else {
                        public.push(w);
                    }
                }
                if !public.is_empty() {
                    self.replica.receive(board, &public);
                    out.push((To::AllBut(conn), Msg::Writes(public)));
                }
            }
            Msg::Ballot(b) => {
                if self.take_ballot(from, b) == Some(false) {
                    out.push((To::One(conn), Msg::Denied(Refusal::BallotRefused)));
                }
            }
            Msg::Ask(a) => {
                if let Err(r) = self.ask_from(board, from, a, now_ms) {
                    out.push((To::One(conn), Msg::Denied(r)));
                }
            }
            _ => {}
        }
        out
    }

    /// A parte da facilitação de cada tique: os relógios; o texto que não leio não se escreve aqui; o
    /// que mudei (o meu selado vai ao cofre) e o que se soltou; o estado a cada um, DEPOIS das escritas.
    pub(super) fn chair_tick(&mut self, board: &mut Board, now_ms: u64) -> Vec<(To, Msg)> {
        let mut out = Vec::new();
        self.clock(board, now_ms);
        self.replica
            .set_unread(self.chair.vault.hidden_to(self.me.peer));
        let mut ws = std::mem::take(&mut self.chair.loose);
        ws.extend(self.replica.local(board));
        let mut gone = Vec::new();
        for w in self.replica.take_sealed() {
            if self.chair.fac.private.is_some() {
                self.chair.dirty |= self.chair.vault.keep(self.me.peer, w);
            } else {
                // O modo acabou entre a captura e o tique: o meu selado sai às claras.
                gone.extend(sealed_field(&w).map(|(id, _, _)| id));
                ws.push(w);
            }
        }
        self.replica.released(gone);
        if !ws.is_empty() {
            out.push((To::All, Msg::Writes(ws)));
        }
        if std::mem::take(&mut self.chair.dirty) {
            for (conn, peer) in &self.guests {
                out.push((To::One(*conn), Msg::Fac(self.fac_for(*peer, now_ms, board))));
            }
        }
        out
    }
}
