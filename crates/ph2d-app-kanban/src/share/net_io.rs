//! A sala e o fio: o anfitrião (rede local + a entrada da internet, preparada numa thread como no
//! Quadro) e o convidado (a ligação por link numa thread). Tudo o que vem da rede passa pela sala da
//! `ph2d-kanban-sync`.

use super::Seen;
use ph2d_board_internet::{DhtMode, InternetConfig, InternetHost};
use ph2d_board_net as net;
use ph2d_board_sync::To;
use ph2d_kanban_model::{CardId, KanbanId, KanbanSet, Member};
use ph2d_kanban_sync::{KEvent, KanbanGuest, KanbanHost, KanbanPresence, SoloMsg, Viewers};
use std::collections::BTreeMap;
use std::sync::{Arc, mpsc};
use std::time::Duration;

/// O prazo de ligar pela rede local (o do Quadro) e pelo link (a DHT leva segundos).
const LAN_TIMEOUT: Duration = Duration::from_secs(4);
const LINK_TIMEOUT: Duration = Duration::from_secs(30);

fn waker() -> net::Waker {
    // A app redesenha a cada imagem: não há nada a acordar.
    Arc::new(|| {})
}

/// A internet: a sério, ou só esta máquina (os testes, e `PH2D_BOARD_INTERNET=local`, como o Quadro).
fn internet_config(local: bool) -> InternetConfig {
    let local = local || std::env::var("PH2D_BOARD_INTERNET").is_ok_and(|v| v.trim() == "local");
    if local {
        InternetConfig {
            stun: vec![],
            dht: DhtMode::Off,
            v6: false,
            port: 0,
            via: None,
        }
    } else {
        InternetConfig::public()
    }
}

enum Internet {
    Preparing(mpsc::Receiver<std::io::Result<InternetHost>>),
    Done,
}

pub(crate) struct HostSide {
    net: net::Host,
    s: KanbanHost,
    internet: Internet,
    /// A entrada da internet vive enquanto a sala (largá-la fecha-a).
    _door: Option<InternetHost>,
}

pub(crate) struct GuestSide {
    net: Option<net::Guest>,
    dialing: Option<mpsc::Receiver<Result<net::Guest, net::JoinError>>>,
    g: KanbanGuest,
}

/// Uma sala aberta (uma de cada vez).
pub(crate) enum Session {
    Host(Box<HostSide>),
    Guest(Box<GuestSide>),
}

/// O que uma imagem da sala conta ao ecrã.
#[derive(Default)]
pub(crate) struct Tick {
    pub arrived: Vec<KanbanId>,
    pub welcomed: bool,
    pub link: Option<String>,
    pub notices: Vec<String>,
    pub ended: bool,
}

fn route(net: &net::Host, out: Vec<(To, SoloMsg)>) {
    for (to, m) in out {
        let b = m.encode();
        let _ = match to {
            To::One(c) => net.send(c, b),
            To::All => net.broadcast(b, None),
            To::AllBut(c) => net.broadcast(b, Some(c)),
        };
    }
}

impl Session {
    /// Abre a sala do Kanban `kanban` (e da AGENDA do projecto, se a há).
    pub fn host(
        set: &mut KanbanSet,
        kanban: KanbanId,
        me: &str,
        local: bool,
    ) -> Result<Self, String> {
        let name = set
            .get(kanban)
            .map(|k| k.header.name.clone())
            .unwrap_or_default();
        let ann = net::Announce {
            board: name,
            host: me.to_owned(),
        };
        // Sem mDNS: a procura do Quadro lista quadros, e esta sala fala o Kanban.
        let cfg = net::HostConfig {
            port: if local { 0 } else { net::PORT },
            mdns: false,
            ..net::HostConfig::default()
        };
        let n = net::Host::open_with(&ann, &cfg, waker()).map_err(|e| e.to_string())?;
        let s = KanbanHost::open(
            net::random_u64(),
            me.to_owned(),
            net::random_u64(),
            set,
            &[kanban, KanbanId::AGENDA],
        );
        s.room.replica.arm(set);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(InternetHost::prepare(&internet_config(local)));
        });
        Ok(Self::Host(Box::new(HostSide {
            net: n,
            s,
            internet: Internet::Preparing(rx),
            _door: None,
        })))
    }

    /// Liga (numa thread) pelo link com chave, ou por `endereço código` da rede local.
    pub fn join(text: &str, me: &str, local: bool) -> Result<Self, ()> {
        let (tx, rx) = mpsc::channel();
        if let Ok(invite) = net::Invite::parse(text) {
            std::thread::spawn(move || {
                let r = ph2d_board_internet::join(
                    &invite,
                    &internet_config(local),
                    LINK_TIMEOUT,
                    net::Liveness::default(),
                    waker(),
                );
                let _ = tx.send(r);
            });
        } else {
            let mut parts = text.split_whitespace();
            let addr = parts
                .next()
                .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
                .ok_or(())?;
            let code = parts.next().ok_or(())?.to_owned();
            std::thread::spawn(move || {
                let _ = tx.send(net::Guest::connect(addr, &code, LAN_TIMEOUT, waker()));
            });
        }
        let g = KanbanGuest::new(
            net::random_u64(),
            me.to_owned(),
            net::random_u64(),
            net::random_u64(),
        );
        Ok(Self::Guest(Box::new(GuestSide {
            net: None,
            dialing: Some(rx),
            g,
        })))
    }

    /// `endereço código` da rede local (o que se dita a quem está na mesma sala).
    pub fn lan_text(&self) -> String {
        match self {
            Self::Host(h) => {
                let addr = h
                    .net
                    .addrs()
                    .first()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                format!("{addr} {}", h.net.code())
            }
            Self::Guest(_) => String::new(),
        }
    }

    /// O id da sala do meu Kanban `local`.
    pub fn wire_of(&self, local: KanbanId) -> Option<KanbanId> {
        match self {
            Self::Host(h) => h.s.room.replica.wire_id(local),
            Self::Guest(g) => g.g.room.replica.as_ref()?.wire_id(local),
        }
    }

    /// Fecha: o anfitrião diz adeus a todos (numa thread: esvaziar as filas não prende o ecrã).
    pub fn close(self, set: &mut KanbanSet) {
        match self {
            Self::Host(h) => {
                h.s.room.replica.disarm(set);
                let n = h.net;
                std::thread::spawn(move || n.close(SoloMsg::End.encode()));
            }
            Self::Guest(g) => {
                if let Some(r) = &g.g.room.replica {
                    r.disarm(set);
                }
            }
        }
    }

    /// ⭐ Uma imagem da sala: a rede, o que mudei, a minha presença.
    pub fn tick(
        &mut self,
        set: &mut KanbanSet,
        now: u64,
        me: Option<(KanbanId, Option<CardId>, Option<[f64; 2]>)>,
    ) -> Tick {
        let mut t = Tick::default();
        let p = KanbanPresence {
            peer: 0,
            kanban: me.map(|m| m.0),
            card: me.and_then(|m| m.1),
            cursor: me.and_then(|m| m.2),
        };
        match self {
            Self::Host(h) => h.tick(set, now, p, &mut t),
            Self::Guest(g) => g.tick(set, now, p, &mut t),
        }
        t
    }

    /// Quem vê cada Kanban da sala (ids locais): eu primeiro, depois os colegas.
    pub fn seen(&self, me: &Member, now: u64) -> BTreeMap<KanbanId, Seen> {
        let (viewers, names, kanbans): (
            &Viewers,
            BTreeMap<u64, String>,
            Vec<(KanbanId, KanbanId)>,
        ) = match self {
            Self::Host(h) => (
                h.s.viewers(),
                h.s.peers()
                    .into_iter()
                    .map(|p| (p.peer, p.name.clone()))
                    .collect(),
                h.s.room.replica.kanbans(),
            ),
            Self::Guest(g) => (
                g.g.viewers(),
                g.g.peers()
                    .into_iter()
                    .map(|p| (p.peer, p.name.clone()))
                    .collect(),
                g.g.room
                    .replica
                    .as_ref()
                    .map(|r| r.kanbans())
                    .unwrap_or_default(),
            ),
        };
        let mut out = BTreeMap::new();
        for (wire, local) in kanbans {
            let mut s = Seen {
                people: vec![me.clone()],
                ..Seen::default()
            };
            for peer in viewers.on_kanban(wire) {
                let who = Member(names.get(&peer).cloned().unwrap_or_default());
                if let Some(c) = viewers.get(peer).and_then(|p| p.card) {
                    s.cards.entry(c).or_default().push(who.clone());
                }
                if let Some(at) = viewers.cursor(peer, now) {
                    s.cursors.push((who.clone(), at));
                }
                s.people.push(who);
            }
            out.insert(local, s);
        }
        out
    }
}

impl HostSide {
    fn tick(&mut self, set: &mut KanbanSet, now: u64, p: KanbanPresence, t: &mut Tick) {
        if let Internet::Preparing(rx) = &self.internet {
            match rx.try_recv() {
                Ok(Ok(ih)) => {
                    ih.attach(&mut self.net);
                    t.link = Some(ih.link());
                    self._door = Some(ih);
                    self.internet = Internet::Done;
                }
                Ok(Err(e)) => {
                    t.notices.push(ph2d_i18n::tr_with(
                        "kanban.share.internet_failed",
                        &[("error", &e.to_string())],
                    ));
                    self.internet = Internet::Done;
                }
                Err(mpsc::TryRecvError::Disconnected) => self.internet = Internet::Done,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        for e in self.net.poll() {
            match e {
                net::NetEvent::Frame(c, b) => {
                    let (out, ev) = self.s.frame(set, c, &b, now);
                    route(&self.net, out);
                    for e in ev {
                        if let KEvent::Arrived(ids) = e {
                            t.arrived.extend(ids);
                        }
                    }
                }
                net::NetEvent::Left(c) => route(&self.net, self.s.left(c)),
                _ => {}
            }
        }
        let out = self.s.tick(set, now);
        route(&self.net, out);
        let out = self.s.presence(now, p);
        route(&self.net, out);
    }
}

impl GuestSide {
    fn tick(&mut self, set: &mut KanbanSet, now: u64, p: KanbanPresence, t: &mut Tick) {
        if let Some(rx) = &self.dialing {
            match rx.try_recv() {
                Ok(Ok(n)) => {
                    let _ = n.send(self.g.hello().encode());
                    self.net = Some(n);
                    self.dialing = None;
                }
                Ok(Err(e)) => {
                    t.notices.push(ph2d_i18n::tr_with(
                        "kanban.share.join_failed",
                        &[("error", &format!("{e:?}"))],
                    ));
                    t.ended = true;
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    t.ended = true;
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => return,
            }
        }
        let Some(n) = &self.net else {
            return;
        };
        for e in n.poll() {
            match e {
                net::NetEvent::Frame(_, b) => {
                    let (out, ev) = self.g.frame(set, &b, now);
                    for m in out {
                        let _ = n.send(m.encode());
                    }
                    for e in ev {
                        match e {
                            KEvent::Welcomed => t.welcomed = true,
                            KEvent::Arrived(ids) => t.arrived.extend(ids),
                            KEvent::Refused { .. } => {
                                t.notices
                                    .push(ph2d_i18n::tr("kanban.share.refused").to_owned());
                                t.ended = true;
                            }
                            KEvent::Ended => {
                                t.notices
                                    .push(ph2d_i18n::tr("kanban.share.ended").to_owned());
                                t.ended = true;
                            }
                            KEvent::Joined(_) | KEvent::Left(_) => {}
                        }
                    }
                }
                net::NetEvent::Left(_) => {
                    t.notices
                        .push(ph2d_i18n::tr("kanban.share.lost").to_owned());
                    t.ended = true;
                }
                _ => {}
            }
        }
        for m in self.g.tick(set, now) {
            let _ = n.send(m.encode());
        }
        for m in self.g.presence(now, p) {
            let _ = n.send(m.encode());
        }
    }
}
