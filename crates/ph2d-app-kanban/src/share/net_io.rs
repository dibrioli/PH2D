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
use std::time::{Duration, Instant};

/// O prazo de ligar pela rede local (o do Quadro) e pelo link (a DHT leva segundos).
const LAN_TIMEOUT: Duration = Duration::from_secs(4);
const LINK_TIMEOUT: Duration = Duration::from_secs(30);
/// J4 — a voltar depois de cair: a 1.ª tentativa logo, depois o dobro de cada vez até ao tecto; o
/// minuto inteiro é o do Quadro (`ph2d-app-board::live::RECONNECT_FOR`). Porquê: `MEDICOES.md` §J4.
const RETRY_FIRST: Duration = Duration::from_millis(250);
const RETRY_MAX: Duration = Duration::from_secs(8);
const RECONNECT_FOR: Duration = Duration::from_secs(60);

/// A espera antes da tentativa `n` (0 = a primeira).
pub(crate) fn backoff(n: u32) -> Duration {
    RETRY_FIRST
        .saturating_mul(1u32.checked_shl(n.min(16)).unwrap_or(u32::MAX))
        .min(RETRY_MAX)
}

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
    /// J4 — os retratos de quem entra, feitos num fio (MEDICOES.md §J4).
    done_tx: mpsc::Sender<(net::ConnId, Vec<ph2d_kanban_sync::KMsg>)>,
    done_rx: mpsc::Receiver<(net::ConnId, Vec<ph2d_kanban_sync::KMsg>)>,
}

/// Por onde se volta a ligar.
#[derive(Clone)]
enum Dial {
    Link(net::Invite),
    Lan(std::net::SocketAddr, String),
}

type Dialing = mpsc::Receiver<Result<net::Guest, net::JoinError>>;

fn dial(d: &Dial, local: bool) -> Dialing {
    let (tx, rx) = mpsc::channel();
    let d = d.clone();
    std::thread::spawn(move || {
        let r = match d {
            Dial::Link(invite) => ph2d_board_internet::join(
                &invite,
                &internet_config(local),
                LINK_TIMEOUT,
                net::Liveness::default(),
                waker(),
            ),
            Dial::Lan(addr, code) => net::Guest::connect(addr, &code, LAN_TIMEOUT, waker()),
        };
        let _ = tx.send(r);
    });
    rx
}

pub(crate) struct GuestSide {
    net: Option<net::Guest>,
    dialing: Option<Dialing>,
    g: KanbanGuest,
    route: Dial,
    local: bool,
    /// Caída desde quando, quantas tentativas, e quando a próxima.
    lost: Option<Instant>,
    tries: u32,
    next_try: Option<Instant>,
}

/// Porque o texto colado não abre a sala do Kanban.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JoinRefusal {
    /// Nem link nem `endereço código`.
    Bad,
    /// O convite de um QUADRO: vai para o «Join» do Quadro.
    Board,
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
    /// Caiu e está a voltar.
    pub reconnecting: bool,
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
        let mut s = KanbanHost::open(
            net::random_u64(),
            me.to_owned(),
            net::random_u64(),
            set,
            &[kanban, KanbanId::AGENDA],
        );
        s.room.replica.arm(set);
        s.room.portraits_off_window();
        let (done_tx, done_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(InternetHost::prepare(&internet_config(local)));
        });
        Ok(Self::Host(Box::new(HostSide {
            net: n,
            s,
            internet: Internet::Preparing(rx),
            _door: None,
            done_tx,
            done_rx,
        })))
    }

    /// Liga (numa thread) pelo link com chave, ou por `endereço código` da rede local.
    pub fn join(text: &str, me: &str, local: bool) -> Result<Self, JoinRefusal> {
        let route = match net::Invite::parse_as(text, net::InviteKind::Kanban) {
            Ok(invite) => Dial::Link(invite),
            Err(net::InviteError::OtherKind(net::InviteKind::Board)) => {
                return Err(JoinRefusal::Board);
            }
            Err(_) => {
                let mut parts = text.split_whitespace();
                let addr = parts
                    .next()
                    .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
                    .ok_or(JoinRefusal::Bad)?;
                let code = parts.next().ok_or(JoinRefusal::Bad)?.to_owned();
                Dial::Lan(addr, code)
            }
        };
        let rx = dial(&route, local);
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
            route,
            local,
            lost: None,
            tries: 0,
            next_try: None,
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

    /// Os testes: o fio do convidado cai.
    pub fn drop_link(&mut self) {
        if let Self::Guest(g) = self {
            g.drop_link();
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
                    t.link = Some(ih.invite().link_as(net::InviteKind::Kanban));
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
        for p in self.s.room.take_portraits() {
            let tx = self.done_tx.clone();
            let _ = std::thread::Builder::new()
                .name("kanban-portrait".into())
                .spawn(move || {
                    let conn = p.conn;
                    let _ = tx.send((conn, p.messages()));
                });
        }
        while let Ok((c, ms)) = self.done_rx.try_recv() {
            route(
                &self.net,
                ms.into_iter()
                    .map(|m| (To::One(c), SoloMsg::K(m)))
                    .collect(),
            );
        }
    }
}

impl GuestSide {
    /// Os testes: o fio cai (como o Wi-Fi que soluça) — o anfitrião vê a saída e eu volto sozinho.
    pub(crate) fn drop_link(&mut self) {
        if self.net.take().is_some() {
            self.lost = Some(Instant::now());
            self.tries = 0;
            self.next_try = None;
        }
    }

    /// Uma tentativa de ligar acabou? `false` = a sala acabou aqui.
    fn dialed(&mut self, t: &mut Tick) -> bool {
        let Some(rx) = &self.dialing else {
            return true;
        };
        match rx.try_recv() {
            Ok(Ok(n)) => {
                let _ = n.send(self.g.hello().encode());
                self.net = Some(n);
                self.dialing = None;
                if self.lost.take().is_some() {
                    t.notices
                        .push(ph2d_i18n::tr("kanban.share.reconnected").to_owned());
                }
                self.tries = 0;
                true
            }
            // A voltar: só a sala que já não está (a porta recusada, o código errado) acaba.
            Ok(Err(e)) if self.lost.is_some() && !room_is_gone(&e) => {
                self.dialing = None;
                true
            }
            Ok(Err(e)) => {
                let key = if self.lost.is_some() {
                    "kanban.share.ended"
                } else {
                    "kanban.share.join_failed"
                };
                t.notices
                    .push(ph2d_i18n::tr_with(key, &[("error", &format!("{e:?}"))]));
                t.ended = true;
                false
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                t.ended = true;
                false
            }
            Err(mpsc::TryRecvError::Empty) => true,
        }
    }

    /// Caída: a próxima tentativa, com o retrocesso; passado o minuto, desiste.
    fn retry(&mut self, t: &mut Tick) {
        let Some(since) = self.lost else {
            return;
        };
        t.reconnecting = true;
        if self.net.is_some() || self.dialing.is_some() {
            return;
        }
        if since.elapsed() > RECONNECT_FOR {
            t.notices
                .push(ph2d_i18n::tr("kanban.share.lost").to_owned());
            t.ended = true;
            return;
        }
        let now = Instant::now();
        let due = *self.next_try.get_or_insert(now + backoff(self.tries));
        if now >= due {
            self.dialing = Some(dial(&self.route, self.local));
            self.tries += 1;
            self.next_try = Some(now + backoff(self.tries));
        }
    }

    fn tick(&mut self, set: &mut KanbanSet, now: u64, p: KanbanPresence, t: &mut Tick) {
        if !self.dialed(t) {
            return;
        }
        self.retry(t);
        if t.ended {
            return;
        }
        let Some(n) = &self.net else {
            return;
        };
        let mut dropped = false;
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
                net::NetEvent::Left(_) => dropped = true,
                _ => {}
            }
        }
        if t.ended {
            return;
        }
        if dropped {
            // Nunca entrou: não há a que voltar.
            if self.g.me.is_none() {
                t.notices
                    .push(ph2d_i18n::tr("kanban.share.lost").to_owned());
                t.ended = true;
                return;
            }
            t.notices
                .push(ph2d_i18n::tr("kanban.share.reconnecting").to_owned());
            self.drop_link();
            t.reconnecting = true;
            return;
        }
        for m in self.g.tick(set, now) {
            let _ = n.send(m.encode());
        }
        for m in self.g.presence(now, p) {
            let _ = n.send(m.encode());
        }
    }
}

/// A voltar: a porta RECUSADA ou um código errado dizem que a sala já não é esta (a lei do Quadro).
fn room_is_gone(e: &net::JoinError) -> bool {
    matches!(
        e,
        net::JoinError::WrongCode
            | net::JoinError::Locked(_)
            | net::JoinError::Unreachable(std::io::ErrorKind::ConnectionRefused)
    )
}

#[cfg(test)]
mod tests {
    /// J4 — o retrocesso: a 1.ª tentativa logo, depois o dobro, até ao tecto (e nunca transborda).
    #[test]
    fn the_backoff_doubles_up_to_its_ceiling() {
        let ms = |n| super::backoff(n).as_millis();
        assert_eq!(
            [ms(0), ms(1), ms(3), ms(5), ms(40)],
            [250, 500, 2000, 8000, 8000]
        );
    }
}
