//! **O quadro ao vivo na FAMÍLIA** ([ADR-0181]): a sessão (`ph2d-board-sync`) e o transporte da rede
//! local (`ph2d-board-net`) servidos a cada imagem — os pedidos do painel, o que chega da rede, a minha
//! presença, a câmara de quem sigo, os avisos. O ecrã só vê `documents.live_view()` e só pede por
//! `ask_live` (o mesmo desenho do `io::service`).
//!
//! [ADR-0181]: ../../../docs/architecture/decisions/0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md

use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use ph2d_board_model::BoardId;
use ph2d_board_net as net;
use ph2d_board_net::firewall::Change;
use ph2d_board_sync::{self as sync, Event, Msg, Presence};
use ph2d_editor_core::documents::live::{
    LiveFac, LiveFound, LiveInternet, LiveOther, LivePeer, LiveRequest, LiveRole,
};
use ph2d_editor_core::screens::HeroScreen;
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_i18n::{tr, tr_with};

/// Quanto tempo se tenta voltar depois de a ligação cair (s): o Wi-Fi da casa que soluça volta em
/// segundos; um anfitrião que fechou o portátil não.
const RECONNECT_FOR: Duration = Duration::from_secs(60);
/// De quanto em quanto se tenta voltar.
const RECONNECT_EVERY: Duration = Duration::from_secs(2);
/// O prazo de uma tentativa de ligar (ligação + aperto de mão), numa thread à parte.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
/// A procura na rede pára sozinha se o painel deixar de a pedir.
const DISCOVER_FOR: Duration = Duration::from_secs(10);

enum Live {
    Host {
        net: net::Host,
        s: sync::Host,
        board: BoardId,
    },
    Guest {
        net: Option<net::Guest>,
        s: sync::Guest,
        board: Option<BoardId>,
        route: Route,
        /// Uma tentativa de ligar a decorrer (numa thread) e o seu resultado.
        dialing: Option<internet::Dialing>,
        /// Desde quando está caída (`None` = ligada, ou a 1.ª entrada).
        lost: Option<Instant>,
        last_try: Option<Instant>,
    },
}

#[path = "live_firewall.rs"]
mod firewall;

#[path = "live_fac.rs"]
mod fac;

#[path = "live_internet.rs"]
mod internet;
use fac::{announce, fac_view, refusal_key};
use internet::{Internet, Route, dial, internet_tick, invite_internet, join_by};
#[cfg(test)]
pub(crate) use internet::fake_nat_for_tests;

#[derive(Default)]
struct State {
    live: Option<Live>,
    discovery: Option<(net::Discovery, Instant)>,
    fw: firewall::Fw,
    internet: Internet,
    /// Os testes: a entrada pela internet sem STUN nem DHT (só os endereços desta máquina).
    internet_local: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    u64::try_from(START.get_or_init(Instant::now).elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Um número ao acaso, do sistema operativo.
fn random() -> u64 {
    net::random_u64()
}

fn waker() -> net::Waker {
    // A app redesenha a cada imagem (`ControlFlow::Poll`): não há nada a acordar.
    Arc::new(|| {})
}

/// ⭐ Uma vez por imagem (chamado por `io::service`).
pub fn service(hero: &mut HeroScreen) {
    let now = now_ms();
    STATE.with_borrow_mut(|st| {
        for req in hero.documents.take_live() {
            request(st, hero, req);
        }
        discover(st, hero);
        st.fw.service(hero);
        match st.live.take() {
            Some(Live::Host { net, s, board }) => st.live = host_tick(hero, net, s, board, now),
            Some(g @ Live::Guest { .. }) => st.live = guest_tick(hero, g, now),
            None => {}
        }
        internet_tick(st, hero);
        if st.live.is_none() && hero.documents.live_view().role != LiveRole::Off {
            stop(hero);
        }
    });
}

fn request(st: &mut State, hero: &mut HeroScreen, req: LiveRequest) {
    match req {
        LiveRequest::Host => {
            if st.live.is_some() {
                return;
            }
            let Some(id) = hero.documents.active() else {
                return;
            };
            let me = hero.documents.author().to_owned();
            let name = hero
                .documents
                .boards()
                .get(id)
                .map(|b| b.name.clone())
                .unwrap_or_default();
            let ann = net::Announce {
                board: name,
                host: me.clone(),
            };
            let n = match net::Host::open(&ann, waker()) {
                Ok(n) => n,
                Err(e) => {
                    hero.documents.tell(tr_with(
                        "board.live.unreachable",
                        &[("error", &e.to_string())],
                    ));
                    return;
                }
            };
            let Some(board) = hero.documents.board_mut(id) else {
                return;
            };
            let s = sync::Host::open(random(), me, board, random());
            let code = n.code();
            let addrs = n.addrs().iter().map(SocketAddr::to_string).collect();
            let v = hero.documents.live_view_mut();
            v.role = LiveRole::Hosting {
                code: code.clone(),
                addrs,
            };
            v.board = Some(id);
            v.link = Default::default();
            hero.documents
                .tell(tr_with("board.live.opened", &[("code", &code)]));
            st.fw.probe(Some(n.port()));
            st.live = Some(Live::Host {
                net: n,
                s,
                board: id,
            });
        }
        LiveRequest::Join { addr, code } => {
            if st.live.is_some() {
                return;
            }
            let found = hero
                .documents
                .live_view()
                .found
                .first()
                .map(|f| f.addr.clone());
            let Some(addr) = addr.or(found).and_then(|a| parse_addr(&a)) else {
                hero.documents.tell(tr("board.live.unreachable").to_owned());
                return;
            };
            join_by(st, hero, Route::Lan { addr, code });
        }
        LiveRequest::JoinLink { link } => {
            if st.live.is_some() {
                return;
            }
            match net::Invite::parse(&link) {
                Ok(invite) => {
                    let local = st.internet_local;
                    join_by(st, hero, Route::Link { invite, local });
                }
                Err(_) => hero.documents.tell(tr("board.live.link_bad").to_owned()),
            }
        }
        LiveRequest::InviteInternet => invite_internet(st, hero),
        LiveRequest::FirewallAllow => st.fw.request(hero, Change::Allow),
        LiveRequest::FirewallRemove => st.fw.request(hero, Change::Remove),
        LiveRequest::Discover => {
            st.fw.probe(None);
            match &mut st.discovery {
                Some((_, asked)) => *asked = Instant::now(),
                None => {
                    if let Ok(d) = net::Discovery::start() {
                        st.discovery = Some((d, Instant::now()));
                    }
                }
            }
        }
        LiveRequest::Leave => {
            if let Some(net) = leave(st, hero) {
                // Numa thread: esvaziar as filas não prende o ecrã.
                std::thread::spawn(move || net.close(Msg::End.encode()));
            }
        }
        LiveRequest::Summon => match &mut st.live {
            Some(Live::Host { net, s, board }) => {
                if let Some(b) = hero.documents.boards().get(*board) {
                    let cam = b.camera;
                    send_host(net, s.summon(cam));
                }
            }
            Some(Live::Guest {
                net: Some(n),
                s,
                board: Some(board),
                ..
            }) => {
                if let Some(b) = hero.documents.boards().get(*board) {
                    let _ = n.send(s.summon(b.camera).encode());
                }
            }
            _ => {}
        },
        // ADR-0182 — o modo privado e a votação.
        r => fac::request(st, hero, r),
    }
}

/// Sair: o anfitrião revela o escondido e acaba a votação (ADR-0182 §5) — sobre o quadro da sala,
/// mesmo que ele tenha acabado de sair das abas — e devolve a rede para o adeus; o convidado só fecha
/// o fio.
fn leave(st: &mut State, hero: &mut HeroScreen) -> Option<net::Host> {
    let mut parting = hero.documents.take_parting();
    let out = match st.live.take() {
        Some(Live::Host { net, mut s, board }) => {
            let b = match parting.as_mut().filter(|p| p.id == board) {
                Some(p) => Some(p),
                None => hero.documents.board_mut(board),
            };
            if let Some(b) = b {
                send_host(&net, s.close(b, now_ms()));
            }
            Some(net)
        }
        _ => None,
    };
    stop(hero);
    out
}

/// ⭐ A app vai fechar (a janela): a sala aberta fecha como «End session», e o adeus SAI antes de a
/// app acabar — o `Drop` sozinho cortava os fios e os convidados ficavam a religar.
pub fn shutdown(hero: &mut HeroScreen) {
    STATE.with_borrow_mut(|st| {
        if let Some(net) = leave(st, hero) {
            net.close(Msg::End.encode());
        }
    });
}

fn parse_addr(s: &str) -> Option<SocketAddr> {
    let s = s.trim();
    s.parse::<SocketAddr>().ok().or_else(|| {
        s.parse::<IpAddr>()
            .ok()
            .map(|ip| SocketAddr::new(ip, net::PORT))
    })
}

fn discover(st: &mut State, hero: &mut HeroScreen) {
    let Some((d, asked)) = &mut st.discovery else {
        return;
    };
    // A procura corre enquanto o painel a pede, e a lista dela é a verdade: um quadro que fechou SAI
    // (o mDNS diz adeus, ~1 s, ADR-0181 §Medições). Quando ela pára, a lista apaga-se — a seguinte
    // começa vazia em vez de mostrar o que já fechou (a foto b3 da cena 12 tinha-o).
    if asked.elapsed() > DISCOVER_FOR {
        st.discovery = None;
        hero.documents.live_view_mut().found.clear();
        return;
    }
    hero.documents.live_view_mut().found = d
        .poll()
        .into_iter()
        .map(|f| LiveFound {
            board: f.board,
            host: f.host,
            addr: f.addr.to_string(),
        })
        .collect();
}

#[path = "live_send.rs"]
mod send;
use send::{host as send_host, tell_too_big};

/// A minha presença no quadro da sala.
fn my_presence(hero: &HeroScreen, board: BoardId) -> Option<Presence> {
    let b = hero.documents.boards().get(board)?;
    let v = hero.documents.live_view();
    let here = hero.documents.active() == Some(board);
    Some(Presence {
        peer: 0,
        cursor: v.my_cursor.filter(|_| here),
        selection: if here {
            hero.documents.active_selection()
        } else {
            Vec::new()
        },
        camera: b.camera,
    })
}

fn host_tick(
    hero: &mut HeroScreen,
    net: net::Host,
    mut s: sync::Host,
    board: BoardId,
    now: u64,
) -> Option<Live> {
    let names = |s: &sync::Host| -> Vec<(u64, String)> {
        s.peers().iter().map(|p| (p.peer, p.name.clone())).collect()
    };
    for ev in net.poll() {
        match ev {
            net::NetEvent::Frame(c, bytes) => {
                let b = hero.documents.board_mut(board)?;
                let (out, events) = s.frame(b, c, &bytes, now);
                send_host(&net, out);
                for e in events {
                    event(hero, board, e, &names(&s));
                }
            }
            net::NetEvent::Left(c) => {
                let before = names(&s);
                let out = s.left(c);
                for (_, m) in &out {
                    if let Msg::Left(p) = m {
                        event(hero, board, Event::Left(*p), &before);
                    }
                }
                send_host(&net, out);
            }
            net::NetEvent::Joined(..) | net::NetEvent::WrongCode(_) => {}
        }
    }
    let p = my_presence(hero, board)?;
    send_host(&net, s.presence(now, p));
    let b = hero.documents.board_mut(board)?;
    // W13 — o fim de uma votação da sala data-se pelo relógio de parede do anfitrião.
    s.set_wall(ph2d_editor_core::screens::hero::board_nav::team::now_unix());
    let big = send_host(&net, s.tick(b, now));
    let me = s.me.peer;
    let mut peers: Vec<LivePeer> = s
        .peers()
        .iter()
        .map(|p| LivePeer {
            peer: p.peer,
            name: p.name.clone(),
            color: p.color,
            me: p.peer == me,
        })
        .collect();
    let room = s.fac(now, b);
    tell_too_big(hero, big);
    fac::hide_names(&room, &mut peers);
    let fac = fac_view(&room, me, me, s.my_sealed(), &peers);
    announce(hero, fac);
    view(hero, board, &peers, |p| s.seen(p), now);
    Some(Live::Host { net, s, board })
}

#[allow(clippy::too_many_lines)]
fn guest_tick(hero: &mut HeroScreen, live: Live, now: u64) -> Option<Live> {
    let Live::Guest {
        mut net,
        mut s,
        mut board,
        route,
        mut dialing,
        mut lost,
        mut last_try,
    } = live
    else {
        return Some(live);
    };
    // Uma tentativa de ligar acabou?
    if let Some(d) = &dialing
        && let Some(r) = d.poll(hero)
    {
        dialing = None;
        match r {
            Ok(n) => {
                let _ = n.send(s.hello().encode());
                net = Some(n);
                if lost.take().is_some() {
                    hero.documents.tell(tr("board.live.reconnected").to_owned());
                }
            }
            Err(e) if board.is_none() => {
                if !internet::link_failed(hero, &route, &e) {
                    hero.documents.tell(join_error(&e));
                }
                return None;
            }
            // Acaba já, com a cópia (antes gastava o minuto e os códigos velhos travavam a sala nova).
            Err(e) if room_is_gone(&e) => {
                hero.documents.tell(tr("board.live.host_left").to_owned());
                return None;
            }
            // Sem rede, prazo: o Wi-Fi que soluça — tenta outra vez.
            Err(_) => {}
        }
    }
    let mut dropped = false;
    if let Some(n) = &net {
        for ev in n.poll() {
            match ev {
                net::NetEvent::Frame(_, bytes) => {
                    let b = board.and_then(|id| hero.documents.board_mut(id));
                    let (out, events) = s.frame(b, &bytes, now);
                    for m in out {
                        let _ = n.send(m.encode());
                    }
                    let names: Vec<(u64, String)> =
                        s.peers().iter().map(|p| (p.peer, p.name.clone())).collect();
                    for e in events {
                        match e {
                            Event::Arrived(b) => {
                                let id = hero.documents.adopt_board(*b);
                                document_tabs::register_board(&mut hero.store, id);
                                board = Some(id);
                            }
                            Event::Refused { .. } => {
                                hero.documents.tell(tr("board.live.refused").to_owned());
                                return None;
                            }
                            // Fechou de propósito: a cópia fica, sem o minuto a tentar voltar.
                            Event::Ended => {
                                hero.documents.tell(tr("board.live.host_left").to_owned());
                                return None;
                            }
                            e => {
                                if let Some(id) = board {
                                    event(hero, id, e, &names);
                                }
                            }
                        }
                    }
                }
                net::NetEvent::Left(_) => dropped = true,
                net::NetEvent::Joined(..) | net::NetEvent::WrongCode(_) => {}
            }
        }
    }
    if dropped {
        net = None;
        if board.is_none() {
            hero.documents.tell(tr("board.live.unreachable").to_owned());
            return None;
        }
        lost = Some(Instant::now());
        hero.documents
            .tell(tr("board.live.reconnecting").to_owned());
    }
    // Caída: tenta voltar de tempos a tempos, até desistir.
    if let Some(t) = lost
        && net.is_none()
        && dialing.is_none()
    {
        if t.elapsed() > RECONNECT_FOR {
            hero.documents.tell(tr("board.live.host_left").to_owned());
            return None;
        }
        if last_try.is_none_or(|l| l.elapsed() >= RECONNECT_EVERY) {
            dialing = Some(dial(&route));
            last_try = Some(Instant::now());
        }
    }
    if let (Some(n), Some(id)) = (&net, board) {
        if let Some(p) = my_presence(hero, id) {
            for m in s.presence(now, p) {
                let _ = n.send(m.encode());
            }
        }
        if let Some(b) = hero.documents.board_mut(id) {
            let big = send::guest(n, s.tick(b, now));
            tell_too_big(hero, big);
        }
        let me = s.me.as_ref().map(|m| m.peer);
        let mut peers: Vec<LivePeer> = s
            .peers()
            .iter()
            .map(|p| LivePeer {
                peer: p.peer,
                name: p.name.clone(),
                color: p.color,
                me: Some(p.peer) == me,
            })
            .collect();
        let room = s.fac(now);
        fac::hide_names(&room, &mut peers);
        let host = peers
            .iter()
            .find(|p| p.color == 0)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        hero.documents.live_view_mut().role = LiveRole::Guest { host };
        let host_peer = peers.iter().find(|p| p.color == 0).map_or(0, |p| p.peer);
        let fac = fac_view(&room, me.unwrap_or(0), host_peer, s.my_sealed(), &peers);
        announce(hero, fac);
        view(hero, id, &peers, |p| s.seen(p), now);
    }
    Some(Live::Guest {
        net,
        s,
        board,
        route,
        dialing,
        lost,
        last_try,
    })
}

/// A voltar: a porta RECUSADA (a máquina responde, ninguém ouve — a sala fechou) ou um código
/// errado/travado (ali abriu OUTRA sala) dizem que a sala já não é esta; o resto pode ser o Wi-Fi.
fn room_is_gone(e: &net::JoinError) -> bool {
    matches!(
        e,
        net::JoinError::WrongCode
            | net::JoinError::Locked(_)
            | net::JoinError::Unreachable(std::io::ErrorKind::ConnectionRefused)
    )
}

fn join_error(e: &net::JoinError) -> String {
    match e {
        net::JoinError::WrongCode | net::JoinError::InvalidCode => {
            tr("board.live.wrong_code").to_owned()
        }
        net::JoinError::Locked(d) => tr_with(
            "board.live.locked",
            &[("s", &d.as_secs().max(1).to_string())],
        ),
        net::JoinError::Incompatible => tr("board.live.refused").to_owned(),
        _ => tr("board.live.unreachable").to_owned(),
    }
}

/// Um aviso da sessão, e «trazer todos».
fn event(hero: &mut HeroScreen, board: BoardId, e: Event, names: &[(u64, String)]) {
    let name = |p: u64| {
        names
            .iter()
            .find(|(q, _)| *q == p)
            .map(|(_, n)| n.clone())
            .unwrap_or_default()
    };
    match e {
        Event::Joined(p) => {
            let n = fac::someone(hero, p.name);
            hero.documents
                .tell(tr_with("board.live.joined", &[("name", &n)]));
        }
        Event::Left(p) => {
            let n = fac::someone(hero, name(p));
            hero.documents
                .tell(tr_with("board.live.left", &[("name", &n)]));
        }
        Event::Summoned { from, camera } => {
            if let Some(b) = hero.documents.board_mut(board) {
                b.camera = camera;
            }
            hero.documents.activate(Some(board));
            hero.documents.live_view_mut().following = None;
            let n = fac::someone(hero, name(from));
            hero.documents
                .tell(tr_with("board.live.summoned", &[("name", &n)]));
        }
        Event::Denied(r) => hero.documents.tell(tr(refusal_key(r)).to_owned()),
        Event::Welcomed | Event::Refused { .. } | Event::Arrived(_) | Event::Ended => {}
    }
}

/// O que o ecrã desenha: as pessoas, os outros (cursor interpolado e selecção), e a câmara de quem
/// sigo.
fn view<'a>(
    hero: &mut HeroScreen,
    board: BoardId,
    peers: &[LivePeer],
    seen: impl Fn(u64) -> Option<&'a sync::Seen>,
    now: u64,
) {
    // W12 — «Make names anonymous»: o nome nem chega ao ecrã, e ninguém se segue.
    let anon = hero
        .documents
        .live_view()
        .fac
        .private
        .as_ref()
        .is_some_and(|p| p.anonymous);
    let others: Vec<LiveOther> = peers
        .iter()
        .filter(|p| !p.me)
        .map(|p| {
            let s = seen(p.peer);
            LiveOther {
                peer: p.peer,
                name: if anon { String::new() } else { p.name.clone() },
                anonymous: anon,
                color: p.color,
                cursor: s.and_then(|s| s.cursor_at(now)),
                selection: s
                    .and_then(|s| s.last.as_ref())
                    .map(|l| l.selection.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();
    let follow = hero.documents.live_view().following.filter(|_| !anon);
    let camera = follow.and_then(&seen).and_then(|s| s.camera_at(now));
    let v = hero.documents.live_view_mut();
    v.board = Some(board);
    v.peers = peers.to_vec();
    v.others = others;
    if anon || follow.is_some_and(|p| !peers.iter().any(|q| q.peer == p)) {
        v.following = None;
    }
    if let Some(cam) = camera
        && let Some(b) = hero.documents.board_mut(board)
    {
        b.camera = cam;
    }
}

/// A sessão acabou (saí, fechei, ou ela caiu de vez): o quadro fica, como cópia minha.
fn stop(hero: &mut HeroScreen) {
    let v = hero.documents.live_view_mut();
    let board = v.board.take();
    v.role = LiveRole::Off;
    v.peers.clear();
    v.others.clear();
    v.fac = LiveFac::default();
    v.internet = LiveInternet::Off;
    v.link.ended();
    v.following = None;
    if let Some(b) = board.and_then(|id| hero.documents.board_mut(id)) {
        b.set_session(None);
    }
}

/// Para os testes da família: a firewall desta thread lê-se como `probe` sobre `subnets`, e nada
/// corre (pedir a senha num teste é um defeito).
#[cfg(test)]
pub(crate) fn fake_firewall_for_tests(
    probe: ph2d_board_net::firewall::Probe,
    subnets: Vec<ph2d_board_net::Subnet>,
) {
    STATE.with_borrow_mut(|st| {
        st.fw.env.probe = Arc::new(move || probe.clone());
        st.fw.env.subnets = Arc::new(move || subnets.clone());
        st.fw.env.run = Arc::new(|cmd| panic!("um teste pediu a senha: {cmd:?}"));
        st.fw.env.file = None;
    });
}

/// Para os testes da família: a sessão desta thread acaba.
#[doc(hidden)]
pub fn reset_for_tests() {
    STATE.with_borrow_mut(|st| *st = State::default());
}

/// ADR-0187 — nesta thread, a internet é só esta máquina (sem STUN nem DHT: os testes não saem
/// para a rede pública).
pub fn internet_local_for_tests() {
    STATE.with_borrow_mut(|st| st.internet_local = true);
}

#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
