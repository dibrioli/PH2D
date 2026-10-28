//! **Uma aba de Kanban partilhada SOZINHA** (plano §3.4): a mesma sala em estrela do Quadro — a
//! mesma versão do protocolo, a identidade presa a uma chave, as cores por ordem de entrada — com o
//! Kanban por dentro ([`KMsg`]). O transporte é o do Quadro: a rede local (código curto) e a internet
//! (link com chave), por quem compõe.

use std::collections::BTreeMap;

use ph2d_board_sync::{ConnId, PROTOCOL, PeerId, PeerInfo, To};
use ph2d_kanban_model::{KanbanId, KanbanSet};
use serde::{Deserialize, Serialize};

use crate::msg::KMsg;
use crate::presence::{KanbanPresence, Viewers};
use crate::replica::KanbanReplica;
use crate::room::{GuestRoom, HostRoom, KEvent};

/// O que viaja numa sessão só do Kanban. ⛔ Só se ACRESCENTA no fim.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SoloMsg {
    Hello {
        protocol: u32,
        peer: PeerId,
        name: String,
        /// O segredo de quem abriu este `peer` (só vai ao anfitrião; voltar pede o mesmo).
        key: u64,
    },
    Welcome {
        you: PeerInfo,
        peers: Vec<PeerInfo>,
    },
    Refused {
        protocol: u32,
    },
    Joined(PeerInfo),
    Left(PeerId),
    /// O anfitrião fechou a sala de propósito.
    End,
    K(KMsg),
}

impl SoloMsg {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("postcard escreve qualquer mensagem")
    }

    #[must_use]
    pub fn decode(bytes: &[u8]) -> Option<SoloMsg> {
        postcard::from_bytes(bytes).ok()
    }
}

fn wrap(v: Vec<(To, KMsg)>) -> impl Iterator<Item = (To, SoloMsg)> {
    v.into_iter().map(|(to, m)| (to, SoloMsg::K(m)))
}

/// O anfitrião de um Kanban partilhado sozinho.
#[derive(Debug)]
pub struct KanbanHost {
    pub room: HostRoom,
    pub me: PeerInfo,
    guests: BTreeMap<ConnId, PeerId>,
    keys: BTreeMap<PeerId, u64>,
    peers: BTreeMap<PeerId, PeerInfo>,
}

impl KanbanHost {
    /// Abre os Kanbans `shared` (normalmente um: a aba). `seed` vem do sistema.
    #[must_use]
    pub fn open(
        peer: PeerId,
        name: String,
        seed: u64,
        set: &KanbanSet,
        shared: &[KanbanId],
    ) -> Self {
        let me = PeerInfo {
            peer,
            name,
            color: 0,
        };
        Self {
            room: HostRoom::new(KanbanReplica::host(peer, seed, set, shared)),
            peers: BTreeMap::from([(peer, me.clone())]),
            me,
            guests: BTreeMap::new(),
            keys: BTreeMap::new(),
        }
    }

    /// Bytes que chegaram de `conn`.
    pub fn frame(
        &mut self,
        set: &mut KanbanSet,
        conn: ConnId,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<(To, SoloMsg)>, Vec<KEvent>) {
        let mut out = Vec::new();
        let mut events = Vec::new();
        let Some(msg) = SoloMsg::decode(bytes) else {
            return (out, events);
        };
        match msg {
            SoloMsg::Hello {
                protocol,
                peer,
                name,
                key,
            } => {
                let impostor =
                    peer == self.me.peer || self.keys.get(&peer).is_some_and(|k| *k != key);
                if protocol != PROTOCOL || impostor {
                    out.push((To::One(conn), SoloMsg::Refused { protocol: PROTOCOL }));
                    return (out, events);
                }
                self.keys.insert(peer, key);
                let color = self.peers.get(&peer).map_or_else(
                    || u8::try_from(self.peers.len()).unwrap_or(u8::MAX),
                    |p| p.color,
                );
                let info = PeerInfo { peer, name, color };
                self.guests.insert(conn, peer);
                self.peers.insert(peer, info.clone());
                let here: Vec<PeerInfo> = self
                    .peers
                    .values()
                    .filter(|p| self.is_here(p.peer))
                    .cloned()
                    .collect();
                out.push((
                    To::One(conn),
                    SoloMsg::Welcome {
                        you: info.clone(),
                        peers: here,
                    },
                ));
                out.extend(wrap(self.room.welcome(set, conn)));
                out.push((To::AllBut(conn), SoloMsg::Joined(info.clone())));
                events.push(KEvent::Joined(info));
            }
            SoloMsg::K(m) => {
                if let Some(&from) = self.guests.get(&conn) {
                    let (o, e) = self.room.frame(set, conn, from, m, now_ms);
                    out.extend(wrap(o));
                    events.extend(e);
                }
            }
            _ => {}
        }
        (out, events)
    }

    /// `conn` saiu (ou caiu).
    pub fn left(&mut self, conn: ConnId) -> Vec<(To, SoloMsg)> {
        let Some(peer) = self.guests.remove(&conn) else {
            return Vec::new();
        };
        if self.guests.values().any(|p| *p == peer) {
            return Vec::new();
        }
        let mut out = vec![(To::All, SoloMsg::Left(peer))];
        out.extend(wrap(self.room.left(peer)));
        out
    }

    /// A cada imagem.
    pub fn tick(&mut self, set: &mut KanbanSet, now_ms: u64) -> Vec<(To, SoloMsg)> {
        wrap(self.room.tick(set, now_ms)).collect()
    }

    pub fn presence(&mut self, now_ms: u64, p: KanbanPresence) -> Vec<(To, SoloMsg)> {
        wrap(self.room.presence(now_ms, p)).collect()
    }

    #[must_use]
    pub fn peers(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| self.is_here(p.peer))
            .collect()
    }

    #[must_use]
    pub fn viewers(&self) -> &Viewers {
        &self.room.viewers
    }

    fn is_here(&self, peer: PeerId) -> bool {
        peer == self.me.peer || self.guests.values().any(|p| *p == peer)
    }
}

/// Um convidado de um Kanban partilhado sozinho.
#[derive(Debug)]
pub struct KanbanGuest {
    pub room: GuestRoom,
    name: String,
    key: u64,
    pub me: Option<PeerInfo>,
    peers: BTreeMap<PeerId, PeerInfo>,
}

impl KanbanGuest {
    /// `seed` e `key` vêm do sistema; `key` só sai no `Hello`.
    #[must_use]
    pub fn new(peer: PeerId, name: String, seed: u64, key: u64) -> Self {
        Self {
            room: GuestRoom::new(peer, seed),
            name,
            key,
            me: None,
            peers: BTreeMap::new(),
        }
    }

    /// A 1.ª mensagem de cada ligação (também ao voltar depois de cair).
    #[must_use]
    pub fn hello(&self) -> SoloMsg {
        SoloMsg::Hello {
            protocol: PROTOCOL,
            peer: self.room.peer,
            name: self.name.clone(),
            key: self.key,
        }
    }

    /// Bytes que chegaram do anfitrião.
    pub fn frame(
        &mut self,
        set: &mut KanbanSet,
        bytes: &[u8],
        now_ms: u64,
    ) -> (Vec<SoloMsg>, Vec<KEvent>) {
        let mut out = Vec::new();
        let mut events = Vec::new();
        let Some(msg) = SoloMsg::decode(bytes) else {
            return (out, events);
        };
        match msg {
            SoloMsg::Welcome { you, peers } => {
                // O retrato vem a seguir (talvez de um fio): até lá as escritas esperam.
                self.room.entered();
                self.me = Some(you);
                self.peers = peers.into_iter().map(|p| (p.peer, p)).collect();
                events.push(KEvent::Welcomed);
            }
            SoloMsg::Refused { protocol } => events.push(KEvent::Refused { theirs: protocol }),
            SoloMsg::Joined(p) => {
                self.peers.insert(p.peer, p.clone());
                events.push(KEvent::Joined(p));
            }
            SoloMsg::Left(p) => {
                self.peers.remove(&p);
                self.room.viewers.gone(p);
                events.push(KEvent::Left(p));
            }
            SoloMsg::End => events.push(KEvent::Ended),
            SoloMsg::K(m) => {
                let (o, e) = self.room.frame(set, m, now_ms);
                out.extend(o.into_iter().map(SoloMsg::K));
                events.extend(e);
            }
            SoloMsg::Hello { .. } => {}
        }
        (out, events)
    }

    /// A cada imagem.
    pub fn tick(&mut self, set: &mut KanbanSet, now_ms: u64) -> Vec<SoloMsg> {
        self.room
            .tick(set, now_ms)
            .into_iter()
            .map(SoloMsg::K)
            .collect()
    }

    pub fn presence(&mut self, now_ms: u64, p: KanbanPresence) -> Vec<SoloMsg> {
        self.room
            .presence(now_ms, p)
            .into_iter()
            .map(SoloMsg::K)
            .collect()
    }

    #[must_use]
    pub fn peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }

    #[must_use]
    pub fn viewers(&self) -> &Viewers {
        &self.room.viewers
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
