//! **A sessão**: o anfitrião (abre o quadro e retransmite — a rede é uma estrela) e o convidado, como
//! máquinas de estado PURAS: bytes que chegam (e o relógio, injectado) → bytes a enviar e eventos para
//! o ecrã. Não sabem que transporte as leva.

use std::collections::BTreeMap;

use ph2d_board_model::{Board, Camera};
use serde::{Deserialize, Serialize};

use crate::presence::{Presence, SEND_EVERY_MS, Seen};
use crate::{PeerId, Replica, Snapshot, Write};

/// A versão do protocolo: quem fala outra é recusado com uma frase (nunca uma fusão às cegas).
pub const PROTOCOL: u32 = 1;

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
    Refused { theirs: u32 },
    Joined(PeerInfo),
    Left(PeerId),
    /// Alguém chamou todos para a vista dele.
    Summoned { from: PeerId, camera: Camera },
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

/// O anfitrião.
#[derive(Debug)]
pub struct Host {
    replica: Replica,
    pub me: PeerInfo,
    guests: BTreeMap<ConnId, PeerId>,
    room: Room,
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
                || {
                    u8::try_from(self.room.peers.len()).unwrap_or(u8::MAX)
                },
                |p| p.color,
            );
            let info = PeerInfo { peer, name, color };
            // O que eu tinha por enviar vai aos outros; o novo recebe-o no retrato.
            for (_, m) in self.tick(board, now_ms) {
                out.push((To::AllBut(conn), m));
            }
            self.guests.insert(conn, peer);
            self.room.peers.insert(peer, info.clone());
            out.push((
                To::One(conn),
                Msg::Welcome {
                    you: info.clone(),
                    peers: self.room.peers.values().cloned().collect(),
                    snapshot: Box::new(self.replica.snapshot(board)),
                },
            ));
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

    /// A cada imagem: o que mudei no quadro, e a minha presença por enviar.
    pub fn tick(&mut self, board: &mut Board, now_ms: u64) -> Vec<(To, Msg)> {
        let mut out = Vec::new();
        let ws = self.replica.local(board);
        if !ws.is_empty() {
            out.push((To::All, Msg::Writes(ws)));
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
        self.room.peers.values().filter(|p| self.is_here(p.peer)).collect()
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
        }
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
            }
            Msg::Refused { protocol } => events.push(Event::Refused { theirs: protocol }),
            Msg::Writes(ws) => {
                if let (Some(r), Some(b)) = (&mut self.replica, board.as_deref_mut()) {
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
