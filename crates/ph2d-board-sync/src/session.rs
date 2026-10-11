//! **A sessão**: o anfitrião (abre o quadro e retransmite — a rede é uma estrela) e o convidado, como
//! máquinas de estado PURAS: bytes que chegam (e o relógio, injectado) → bytes a enviar e eventos para
//! o ecrã. Não sabem que transporte as leva.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, BoardOp, Camera, ElementId, Field, VOTES_MAX};
use serde::{Deserialize, Serialize};

use crate::fac::{Ask, Ballot, Fac, Private, REVEAL_COUNTDOWN_MS, Refusal, VoteRoom};
use crate::presence::{Presence, SEND_EVERY_MS, Seen};
use crate::vault::{Vault, sealed_text};
use crate::{PeerId, Replica, Snapshot, Write};

/// A versão do protocolo: quem fala outra é recusado com uma frase (nunca uma fusão às cegas).
/// 2 = a facilitação ao vivo (ADR-0182).
pub const PROTOCOL: u32 = 2;

/// Uma ligação, no transporte do anfitrião.
pub type ConnId = u64;

/// Quem está na sala: o nome (o autor da W7) e a cor (o índice na paleta de pessoas, por ordem de
/// entrada; o anfitrião é o 0).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer: PeerId,
    pub name: String,
    pub color: u8,
}

/// O que viaja.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    Hello {
        protocol: u32,
        peer: PeerId,
        name: String,
    },
    Welcome {
        you: PeerInfo,
        peers: Vec<PeerInfo>,
        snapshot: Box<Snapshot>,
    },
    Refused {
        protocol: u32,
    },
    Writes(Vec<Write>),
    Presence(Presence),
    Joined(PeerInfo),
    Left(PeerId),
    /// «Trazer todos» (o do Miro): a câmara de quem chamou.
    Summon {
        from: PeerId,
        camera: Camera,
    },
    /// O anfitrião fechou a sala DE PROPÓSITO: quem está fica com a cópia, sem tentar voltar (uma
    /// queda tenta voltar). ⛔ Só se ACRESCENTA no fim: o índice da variante é o byte que viaja.
    End,
    /// ADR-0182 — o texto de notas escrito em modo privado: de quem o escreveu SÓ para o cofre do
    /// anfitrião (nunca retransmitido).
    Sealed(Vec<Write>),
    /// O meu boletim inteiro, para o cofre.
    Ballot(Ballot),
    /// O estado da facilitação, do anfitrião para cada um (à medida de quem o recebe).
    Fac(Fac),
    /// Um pedido ao anfitrião.
    Ask(Ask),
    /// O anfitrião recusou o meu pedido.
    Denied(Refusal),
}

impl Msg {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("postcard escreve qualquer mensagem")
    }

    /// `None` = bytes que não são uma mensagem deste protocolo.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<Msg> {
        postcard::from_bytes(bytes).ok()
    }
}

/// Para quem vai uma mensagem do anfitrião.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum To {
    One(ConnId),
    All,
    AllBut(ConnId),
}

/// O que o ecrã tem de saber.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Entrei (ou voltei) na sala.
    Welcomed,
    /// O quadro da sala, na 1.ª entrada: o ecrã abre-o numa aba e passa-o daí em diante.
    Arrived(Box<Board>),
    /// O anfitrião fala outra versão.
    Refused {
        theirs: u32,
    },
    Joined(PeerInfo),
    Left(PeerId),
    /// Alguém chamou todos para a vista dele.
    Summoned {
        from: PeerId,
        camera: Camera,
    },
    /// O anfitrião fechou a sala.
    Ended,
    /// Um pedido meu foi recusado (a frase que o artista lê).
    Denied(Refusal),
}

/// O que é comum às duas pontas: quem está, o que se vê deles, e a minha presença por enviar.
#[derive(Clone, Debug, Default)]
struct Room {
    peers: BTreeMap<PeerId, PeerInfo>,
    seen: BTreeMap<PeerId, Seen>,
    sent_at: Option<u64>,
    sent: Option<Presence>,
    queued: Option<Presence>,
}

impl Room {
    /// A minha presença: sai já se a selecção mudou; senão no máximo a cada [`SEND_EVERY_MS`] — e a
    /// última nunca se perde (sai no próximo [`Room::flush`]).
    fn presence(&mut self, now_ms: u64, p: Presence) -> Option<Presence> {
        if self.sent.as_ref() == Some(&p) {
            self.queued = None;
            return None;
        }
        let selection = self.sent.as_ref().map(|s| &s.selection) != Some(&p.selection);
        let due = self
            .sent_at
            .is_none_or(|t| now_ms.saturating_sub(t) >= SEND_EVERY_MS);
        if selection || due {
            self.sent_at = Some(now_ms);
            self.sent = Some(p.clone());
            self.queued = None;
            Some(p)
        } else {
            self.queued = Some(p);
            None
        }
    }

    fn flush(&mut self, now_ms: u64) -> Option<Presence> {
        let p = self.queued.take()?;
        self.presence(now_ms, p.clone()).or_else(|| {
            self.queued = Some(p);
            None
        })
    }

    fn heard(&mut self, now_ms: u64, p: Presence) {
        self.seen.entry(p.peer).or_default().push(now_ms, p);
    }
}

/// O que o anfitrião decide da facilitação (ADR-0182): o estado, o cofre e os dois relógios.
#[derive(Debug, Default)]
struct Chair {
    fac: Fac,
    vault: Vault,
    reveal_at: Option<u64>,
    vote_until: Option<u64>,
    /// Escritas soltas do cofre, por enviar a todos.
    loose: Vec<Write>,
    /// O estado mudou: sai a cada um no próximo tique, DEPOIS das escritas.
    dirty: bool,
}

/// O anfitrião.
#[derive(Debug)]
pub struct Host {
    replica: Replica,
    pub me: PeerInfo,
    guests: BTreeMap<ConnId, PeerId>,
    room: Room,
    chair: Chair,
}

impl Host {
    /// Abre `board` numa sessão. `seed` vem do sistema (os ids novos).
    pub fn open(peer: PeerId, name: String, board: &mut Board, seed: u64) -> Self {
        let me = PeerInfo {
            peer,
            name,
            color: 0,
        };
        let mut room = Room::default();
        room.peers.insert(peer, me.clone());
        Self {
            replica: Replica::host(peer, board, seed),
            me,
            guests: BTreeMap::new(),
            room,
            chair: Chair::default(),
        }
    }

    /// O estado da facilitação como `peer` o vê (`used` só para quem iniciou a votação).
    fn fac_for(&self, peer: PeerId, now_ms: u64) -> Fac {
        let c = &self.chair;
        let mut f = c.fac.clone();
        f.hidden = c.vault.hidden();
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
    pub fn fac(&self, now_ms: u64) -> Fac {
        self.fac_for(self.me.peer, now_ms)
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
        self.replica
            .released(ws.iter().filter_map(|w| sealed_text(w).map(|(id, _)| id)));
        self.chair.loose.extend(ws);
        self.chair.dirty = true;
    }

    /// SOMAR: os totais vão para o quadro como escrita da sala (o resultado anterior sai), e a
    /// votação fica «acabou».
    fn end_vote(&mut self, board: &mut Board) {
        let totals = self.chair.vault.tally();
        let ops: Vec<BoardOp> = board
            .doc
            .live_with_hidden()
            .filter_map(|e| {
                let want = totals.get(&e.id).copied().unwrap_or(0);
                (e.votes() != want).then(|| {
                    let mut e = e.clone();
                    e.set_votes(want);
                    BoardOp::Put(e)
                })
            })
            .collect();
        for op in ops {
            let _ = op.apply(&mut board.doc);
        }
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
        self.take_ballot(me, b)
    }

    fn take_ballot(&mut self, from: PeerId, b: Ballot) -> bool {
        let Some(v) = self
            .chair
            .fac
            .vote
            .as_ref()
            .filter(|v| !v.ended && v.round == b.round)
        else {
            return false;
        };
        // Uma nota escondida não é votável por quem não a lê.
        let unread = self.chair.vault.hidden_to(from);
        if b.votes.iter().any(|(id, n)| *n > 0 && unread.contains(id)) {
            return false;
        }
        let (setup, targets) = (v.setup, v.targets.clone());
        let ok = self.chair.vault.ballot(from, b, setup, &targets);
        self.chair.dirty |= ok;
        ok
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

    /// Bytes que chegaram de `conn`.
    pub fn frame(
        &mut self,
        board: &mut Board,
        conn: ConnId,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<(To, Msg)>, Vec<Event>) {
        let mut out = Vec::new();
        let mut events = Vec::new();
        let Some(msg) = Msg::decode(bytes) else {
            return (out, events);
        };
        if let Msg::Hello {
            protocol,
            peer,
            name,
        } = msg
        {
            if protocol != PROTOCOL {
                out.push((To::One(conn), Msg::Refused { protocol: PROTOCOL }));
                return (out, events);
            }
            // Quem volta guarda a cor; quem chega leva a seguinte.
            let color = self.room.peers.get(&peer).map_or_else(
                || u8::try_from(self.room.peers.len()).unwrap_or(u8::MAX),
                |p| p.color,
            );
            let info = PeerInfo { peer, name, color };
            // O que eu tinha por enviar vai aos outros; o novo recebe-o no retrato.
            // (O que era a UM fica a esse: o estado da facilitação é à medida de quem o recebe.)
            for (to, m) in self.tick(board, now_ms) {
                out.push((if to == To::All { To::AllBut(conn) } else { to }, m));
            }
            self.guests.insert(conn, peer);
            self.room.peers.insert(peer, info.clone());
            out.push((
                To::One(conn),
                Msg::Welcome {
                    you: info.clone(),
                    // Só quem ESTÁ (o `peers` guarda quem saiu, para a cor de quem volta).
                    peers: self
                        .room
                        .peers
                        .values()
                        .filter(|p| self.is_here(p.peer))
                        .cloned()
                        .collect(),
                    snapshot: Box::new(self.replica.snapshot(board)),
                },
            ));
            // Quem entra vê JÁ onde estão os outros: a presença só sai quando muda, e quem estava
            // parado ficava invisível ao recém-chegado até mexer o rato (a foto b2 da cena 12).
            let mine = self.room.sent.iter().cloned();
            let theirs = self
                .room
                .seen
                .iter()
                .filter(|(p, _)| **p != peer)
                .filter_map(|(_, s)| s.last.clone());
            for p in mine.chain(theirs).collect::<Vec<_>>() {
                out.push((To::One(conn), Msg::Presence(p)));
            }
            // Quem entra a meio sabe já do modo privado e da votação (Scrumlr v02, Miro VO).
            out.push((To::One(conn), Msg::Fac(self.fac_for(peer, now_ms))));
            out.push((To::AllBut(conn), Msg::Joined(info.clone())));
            events.push(Event::Joined(info));
            return (out, events);
        }
        let Some(&from) = self.guests.get(&conn) else {
            return (out, events);
        };
        match msg {
            Msg::Writes(ws) => {
                self.replica.receive(board, &ws);
                out.push((To::AllBut(conn), Msg::Writes(ws)));
            }
            Msg::Presence(p) if p.peer == from => {
                self.room.heard(now_ms, p.clone());
                out.push((To::AllBut(conn), Msg::Presence(p)));
            }
            Msg::Summon { from: f, camera } if f == from => {
                out.push((To::AllBut(conn), Msg::Summon { from, camera }));
                events.push(Event::Summoned { from, camera });
            }
            Msg::Sealed(ws) => {
                let mut public = Vec::new();
                for w in ws {
                    // Um selado que não é mais novo do que o que TODOS têm perdeu (ou é um reenvio
                    // depois de revelar): não muda nada em ninguém e NUNCA se retransmite — o texto
                    // dele pode nunca ter sido mostrado. Com o modo ligado vai ao cofre; desligado
                    // (chegou depois da revelação) é uma escrita como as outras.
                    let Some((id, at)) = sealed_text(&w) else {
                        continue;
                    };
                    if at <= self.replica.public_stamp(id, Field::Text) {
                        continue;
                    }
                    if self.chair.fac.private.is_some() {
                        self.chair.vault.keep(from, w);
                        self.chair.dirty = true;
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
                self.take_ballot(from, b);
            }
            Msg::Ask(a) => {
                if let Err(r) = self.ask_from(board, from, a, now_ms) {
                    out.push((To::One(conn), Msg::Denied(r)));
                }
            }
            _ => {}
        }
        (out, events)
    }

    /// `conn` saiu (ou caiu).
    pub fn left(&mut self, conn: ConnId) -> Vec<(To, Msg)> {
        let Some(peer) = self.guests.remove(&conn) else {
            return Vec::new();
        };
        if self.guests.values().any(|p| *p == peer) {
            return Vec::new();
        }
        self.room.seen.remove(&peer);
        vec![(To::All, Msg::Left(peer))]
    }

    /// A cada imagem: os relógios da facilitação, o que mudei no quadro (o meu texto selado vai ao
    /// cofre), o que se soltou do cofre, o estado da facilitação a cada um, e a minha presença.
    pub fn tick(&mut self, board: &mut Board, now_ms: u64) -> Vec<(To, Msg)> {
        let mut out = Vec::new();
        self.clock(board, now_ms);
        let mut ws = std::mem::take(&mut self.chair.loose);
        ws.extend(self.replica.local(board));
        for w in self.replica.take_sealed() {
            if self.chair.fac.private.is_some() {
                self.chair.vault.keep(self.me.peer, w);
                self.chair.dirty = true;
            } else {
                ws.push(w);
            }
        }
        if !ws.is_empty() {
            out.push((To::All, Msg::Writes(ws)));
        }
        if std::mem::take(&mut self.chair.dirty) {
            for (conn, peer) in &self.guests {
                out.push((To::One(*conn), Msg::Fac(self.fac_for(*peer, now_ms))));
            }
        }
        if let Some(p) = self.room.flush(now_ms) {
            out.push((To::All, Msg::Presence(p)));
        }
        out
    }

    /// A minha presença (o cursor, a selecção, a câmara).
    pub fn presence(&mut self, now_ms: u64, mut p: Presence) -> Vec<(To, Msg)> {
        p.peer = self.me.peer;
        self.room
            .presence(now_ms, p)
            .map(|p| vec![(To::All, Msg::Presence(p))])
            .unwrap_or_default()
    }

    /// «Trazer todos» para a minha vista.
    pub fn summon(&mut self, camera: Camera) -> Vec<(To, Msg)> {
        vec![(
            To::All,
            Msg::Summon {
                from: self.me.peer,
                camera,
            },
        )]
    }

    #[must_use]
    pub fn peers(&self) -> Vec<&PeerInfo> {
        self.room
            .peers
            .values()
            .filter(|p| self.is_here(p.peer))
            .collect()
    }

    fn is_here(&self, peer: PeerId) -> bool {
        peer == self.me.peer || self.guests.values().any(|p| *p == peer)
    }

    #[must_use]
    pub fn seen(&self, peer: PeerId) -> Option<&Seen> {
        self.room.seen.get(&peer)
    }
}

/// Um convidado.
#[derive(Debug)]
pub struct Guest {
    peer: PeerId,
    name: String,
    seed: u64,
    replica: Option<Replica>,
    pub me: Option<PeerInfo>,
    room: Room,
    /// O estado da facilitação e quando chegou (os relógios descontam-se daí).
    fac: (Fac, u64),
    /// O meu último boletim (reenvia-se ao voltar de uma queda).
    ballot: Option<Ballot>,
}

impl Guest {
    #[must_use]
    pub fn new(peer: PeerId, name: String, seed: u64) -> Self {
        Self {
            peer,
            name,
            seed,
            replica: None,
            me: None,
            room: Room::default(),
            fac: (Fac::default(), 0),
            ballot: None,
        }
    }

    /// O estado da facilitação agora (os relógios descontados do que passou desde que chegou).
    #[must_use]
    pub fn fac(&self, now_ms: u64) -> Fac {
        let (mut f, at) = self.fac.clone();
        let gone = now_ms.saturating_sub(at);
        if let Some(p) = &mut f.private {
            p.reveal_in_ms = p.reveal_in_ms.map(|t| t.saturating_sub(gone));
        }
        if let Some(v) = &mut f.vote {
            v.left_ms = v.left_ms.map(|t| t.saturating_sub(gone));
        }
        f
    }

    /// As minhas notas com texto por revelar.
    #[must_use]
    pub fn my_sealed(&self) -> BTreeSet<ElementId> {
        self.replica
            .as_ref()
            .map(Replica::my_sealed)
            .unwrap_or_default()
    }

    /// Um pedido ao anfitrião.
    #[must_use]
    pub fn ask(&self, ask: Ask) -> Msg {
        Msg::Ask(ask)
    }

    /// O meu boletim inteiro (a cada mudança).
    pub fn ballot(&mut self, b: Ballot) -> Msg {
        self.ballot = Some(b.clone());
        Msg::Ballot(b)
    }

    /// A 1.ª mensagem de cada ligação (também ao voltar depois de cair).
    #[must_use]
    pub fn hello(&self) -> Msg {
        Msg::Hello {
            protocol: PROTOCOL,
            peer: self.peer,
            name: self.name.clone(),
        }
    }

    /// Bytes que chegaram do anfitrião. Antes de entrar não há `board`: o 1.º `Welcome` devolve-o
    /// em [`Event::Arrived`].
    pub fn frame(
        &mut self,
        mut board: Option<&mut Board>,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<Msg>, Vec<Event>) {
        let mut out = Vec::new();
        let mut events = Vec::new();
        let Some(msg) = Msg::decode(bytes) else {
            return (out, events);
        };
        match msg {
            Msg::Welcome {
                you,
                peers,
                snapshot,
            } => {
                let seed = self.seed;
                self.me = Some(you);
                self.room.peers = peers.into_iter().map(|p| (p.peer, p)).collect();
                self.room.sent = None;
                match (&mut self.replica, board) {
                    (Some(r), Some(b)) => {
                        if r.rejoin(b, &snapshot, seed).is_err() {
                            return (out, events);
                        }
                        events.push(Event::Welcomed);
                        out.extend(self.tick(b, now_ms));
                    }
                    _ => {
                        let Ok((r, mut b)) = Replica::join(self.peer, &snapshot, seed) else {
                            return (out, events);
                        };
                        self.replica = Some(r);
                        events.push(Event::Welcomed);
                        out.extend(self.tick(&mut b, now_ms));
                        events.push(Event::Arrived(Box::new(b)));
                    }
                }
                // Um boletim que ia no fio quando caiu não se perde (o anfitrião guarda o último).
                if let Some(b) = &self.ballot {
                    out.push(Msg::Ballot(b.clone()));
                }
            }
            Msg::Fac(f) => {
                if let Some(r) = &mut self.replica {
                    let gone: Vec<ElementId> = self
                        .fac
                        .0
                        .hidden
                        .difference(&f.hidden)
                        .copied()
                        .collect();
                    r.released(gone);
                    r.set_sealing(f.private.is_some());
                }
                // Uma votação nova (ou acabada) leva o meu boletim antigo.
                let open = f.vote.as_ref().filter(|v| !v.ended).map(|v| v.round);
                if self.ballot.as_ref().map(|b| b.round) != open {
                    self.ballot = None;
                }
                self.fac = (f, now_ms);
            }
            Msg::Denied(r) => events.push(Event::Denied(r)),
            Msg::Refused { protocol } => events.push(Event::Refused { theirs: protocol }),
            Msg::Writes(ws) => {
                if let (Some(r), Some(b)) = (&mut self.replica, board.as_mut()) {
                    r.receive(b, &ws);
                }
            }
            Msg::Presence(p) if p.peer != self.peer => self.room.heard(now_ms, p),
            Msg::Joined(p) => {
                self.room.peers.insert(p.peer, p.clone());
                events.push(Event::Joined(p));
            }
            Msg::Left(peer) => {
                self.room.peers.remove(&peer);
                self.room.seen.remove(&peer);
                events.push(Event::Left(peer));
            }
            Msg::Summon { from, camera } if from != self.peer => {
                events.push(Event::Summoned { from, camera });
            }
            Msg::End => events.push(Event::Ended),
            _ => {}
        }
        (out, events)
    }

    /// A cada imagem: o que mudei, e a minha presença por enviar.
    pub fn tick(&mut self, board: &mut Board, now_ms: u64) -> Vec<Msg> {
        let mut out = Vec::new();
        if let Some(r) = &mut self.replica {
            let ws = r.local(board);
            if !ws.is_empty() {
                out.push(Msg::Writes(ws));
            }
            let sealed = r.take_sealed();
            if !sealed.is_empty() {
                out.push(Msg::Sealed(sealed));
            }
        }
        if let Some(p) = self.room.flush(now_ms) {
            out.push(Msg::Presence(p));
        }
        out
    }

    pub fn presence(&mut self, now_ms: u64, mut p: Presence) -> Vec<Msg> {
        p.peer = self.peer;
        self.room
            .presence(now_ms, p)
            .map(|p| vec![Msg::Presence(p)])
            .unwrap_or_default()
    }

    #[must_use]
    pub fn summon(&self, camera: Camera) -> Msg {
        Msg::Summon {
            from: self.peer,
            camera,
        }
    }

    #[must_use]
    pub fn peers(&self) -> Vec<&PeerInfo> {
        self.room.peers.values().collect()
    }

    #[must_use]
    pub fn seen(&self, peer: PeerId) -> Option<&Seen> {
        self.room.seen.get(&peer)
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
