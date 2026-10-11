//! **A sessão**: o anfitrião (abre o quadro e retransmite — a rede é uma estrela) e o convidado, como
//! máquinas de estado PURAS: bytes que chegam (e o relógio, injectado) → bytes a enviar e eventos para
//! o ecrã. Não sabem que transporte as leva.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, Camera, ElementId};
use serde::{Deserialize, Serialize};

use crate::fac::{Ask, Ballot, Fac, Refusal};
use crate::presence::{Presence, SEND_EVERY_MS, Seen};
use crate::{PeerId, Replica, Snapshot, Write};

/// A versão do protocolo: quem fala outra é recusado com uma frase (nunca uma fusão às cegas).
/// 2 = a facilitação ao vivo (ADR-0182); 3 = o histórico das votações (`TeamWrite::Poll…`, o `Field`
/// sem `Votes`, ADR-0183); 4 = a identidade presa à chave do `Hello` (ADR-0182 §10); 5 = as imagens
/// (`Write::File`, o retrato sem os bytes delas, W14).
pub const PROTOCOL: u32 = 5;

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
        /// O segredo de quem abriu este `peer`: só vai ao anfitrião, e voltar pede o mesmo. ⛔ No FIM:
        /// um anfitrião do protocolo 3 lê os três de cima, ignora este e recusa pela versão.
        key: u64,
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

/// O `Hello` até ao protocolo 3 (sem a chave): só se lê para o recusar com a frase de sempre — sem
/// isto o par antigo ficava à espera de uma resposta que nunca vinha.
#[derive(Deserialize)]
#[allow(dead_code)] // lêem-se só para saber que é um `Hello`
enum OldHello {
    Hello {
        protocol: u32,
        peer: PeerId,
        name: String,
    },
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

#[path = "session_chair.rs"]
mod chair;
use chair::Chair;

/// O anfitrião.
#[derive(Debug)]
pub struct Host {
    replica: Replica,
    pub me: PeerInfo,
    guests: BTreeMap<ConnId, PeerId>,
    /// A chave de cada identidade, desde o 1.º `Hello` dela (fica com quem sai: é a de quem volta).
    keys: BTreeMap<PeerId, u64>,
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
            keys: BTreeMap::new(),
            room,
            chair: Chair::default(),
        }
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
            if postcard::from_bytes::<OldHello>(bytes).is_ok() {
                out.push((To::One(conn), Msg::Refused { protocol: PROTOCOL }));
            }
            return (out, events);
        };
        if let Msg::Hello {
            protocol,
            peer,
            name,
            key,
        } = msg
        {
            // Os ids correm a sala inteira; a chave só vem ao anfitrião. Quem traz o id de OUTRO (ou
            // o meu) sem a chave dele é recusado ANTES de receber o que é dessa pessoa (o cofre, o
            // boletim) e de poder agir em nome dela.
            let impostor = peer == self.me.peer || self.keys.get(&peer).is_some_and(|k| *k != key);
            if protocol != PROTOCOL || impostor {
                out.push((To::One(conn), Msg::Refused { protocol: PROTOCOL }));
                return (out, events);
            }
            self.keys.insert(peer, key);
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
            out.push((To::One(conn), Msg::Fac(self.fac_for(peer, now_ms, board))));
            // W14 — os bytes das imagens, um ficheiro por mensagem, a seguir ao retrato magro.
            for w in self.replica.all_files(board) {
                out.push((To::One(conn), Msg::Writes(vec![w])));
            }
            out.push((To::AllBut(conn), Msg::Joined(info.clone())));
            events.push(Event::Joined(info));
            return (out, events);
        }
        let Some(&from) = self.guests.get(&conn) else {
            return (out, events);
        };
        match msg {
            Msg::Writes(ws) => {
                let ws = self.chair_writes(board, from, ws);
                if !ws.is_empty() {
                    self.replica.receive(board, &ws);
                    out.push((To::AllBut(conn), Msg::Writes(ws)));
                }
            }
            Msg::Presence(p) if p.peer == from => {
                self.room.heard(now_ms, p.clone());
                out.push((To::AllBut(conn), Msg::Presence(p)));
            }
            Msg::Summon { from: f, camera } if f == from => {
                out.push((To::AllBut(conn), Msg::Summon { from, camera }));
                events.push(Event::Summoned { from, camera });
            }
            m @ (Msg::Sealed(_) | Msg::Ballot(_) | Msg::Ask(_)) => {
                out.extend(self.chair_frame(board, conn, from, m, now_ms));
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
        let mut out = self.chair_tick(board, now_ms);
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
    /// O segredo que prende `peer` a mim (só vai no `Hello`).
    key: u64,
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
    /// `key` vem do sistema e nunca sai daqui senão no `Hello` (é o que me deixa voltar como eu).
    pub fn new(peer: PeerId, name: String, seed: u64, key: u64) -> Self {
        Self {
            peer,
            name,
            seed,
            key,
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
            key: self.key,
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
                    let gone: Vec<ElementId> =
                        self.fac.0.hidden.difference(&f.hidden).copied().collect();
                    r.released(gone);
                    // Desligado: nada meu continua escondido (um selado que chegou depois da
                    // revelação saiu às claras — o anfitrião não mo devolve).
                    if f.private.is_none() {
                        let mine: Vec<ElementId> = r
                            .my_sealed()
                            .into_iter()
                            .filter(|id| !f.hidden.contains(id))
                            .collect();
                        r.released(mine);
                    }
                    let unread = f.hidden.difference(&r.my_sealed()).copied().collect();
                    r.set_unread(unread);
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
            out.extend(crate::replica::by_file(r.local(board)));
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
#[cfg(test)]
#[path = "session_files_tests.rs"]
mod files_tests;
