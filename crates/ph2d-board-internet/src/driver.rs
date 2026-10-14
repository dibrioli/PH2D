//! O MOTOR do nó: uma thread que conduz o `quinn-proto` (QUIC sem I/O). Recebe os datagramas e os
//! pedidos dos fios por um canal, separa STUN e furos do QUIC, e escreve tudo no mesmo socket.
//!
//! Cada ligação leva UM fluxo bidireccional, aberto pelo convidado; esse fluxo é o [`Wire`]. Fechar é
//! com GRAÇA, como no TCP: o envio termina depois da fila, e a ligação só fecha quando o outro lado
//! confirmou tudo e acabou o seu (ou o prazo [`CLOSE_GRACE`] passou) — o adeus do anfitrião chega.

use std::collections::{BTreeMap, VecDeque};
use std::io::{self, ErrorKind};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use bytes::{Buf, Bytes, BytesMut};
use ph2d_board_net::Wire;
use quinn_proto::{
    ClientConfig, Connection, ConnectionHandle, DatagramEvent, Dir, Endpoint, EndpointConfig,
    Event, ReadError, StreamEvent, StreamId, TransportConfig, VarInt, WriteError,
};

use crate::node::{Inner, Msg, decode_via, encode_via};
use crate::pipe::{Accepted, Pipe, QuicWire, RECV_HIGH};
use crate::stun::{self, TxId};
use crate::tls;

/// O prazo do fecho com graça (o outro lado confirmar e acabar o seu envio). Corre no motor, que os
/// fios mantêm vivo: não pesa no `JOIN_DEADLINE` do `Host`. Folga para um ida-e-volta lento.
pub const CLOSE_GRACE: Duration = Duration::from_secs(3);
/// Um furo a cada tanto, enquanto durar o pedido. Recurso: pacotes de ~10 B por candidato.
const PUNCH_EVERY: Duration = Duration::from_millis(200);
/// O furo: começa por `0` (nem QUIC, que tem o bit fixo, nem STUN, que tem o cookie).
pub(crate) const PUNCH: &[u8] = b"\0ph2d-furo";
/// Um pedido STUN sem resposta é esquecido depois disto.
const STUN_TTL: Duration = Duration::from_secs(10);
/// Mensagens tratadas por volta antes de conduzir as ligações.
const BATCH: usize = 256;

/// Os sockets, para enviar.
struct Net {
    s4: UdpSocket,
    s6: Option<UdpSocket>,
    via: Option<SocketAddr>,
}

impl Net {
    fn send(&self, to: SocketAddr, payload: &[u8]) {
        if let Some(via) = self.via {
            let _ = self.s4.send_to(&encode_via(to, payload), via);
            return;
        }
        let _ = match (to, &self.s6) {
            (SocketAddr::V4(_), _) => self.s4.send_to(payload, to),
            (SocketAddr::V6(_), Some(s6)) => s6.send_to(payload, to),
            (SocketAddr::V6(_), None) => return,
        };
    }

    fn reaches(&self, to: SocketAddr) -> bool {
        to.is_ipv4() || self.s6.is_some() || self.via.is_some()
    }
}

struct Conn {
    q: Connection,
    /// A tentativa do convidado a que pertence (`None` = aceite pelo anfitrião).
    dial: Option<u64>,
    stream: Option<(StreamId, Arc<Pipe>)>,
    out: VecDeque<Bytes>,
    /// Terminar o envio quando a fila esvaziar.
    finish: bool,
    /// O envio terminado foi todo confirmado.
    acked: bool,
    /// O prazo do fecho com graça.
    closing: Option<Instant>,
    /// `close` já chamado.
    closed: bool,
}

impl Conn {
    fn new(q: Connection, dial: Option<u64>) -> Self {
        Self {
            q,
            dial,
            stream: None,
            out: VecDeque::new(),
            finish: false,
            acked: false,
            closing: None,
            closed: false,
        }
    }

    fn close(&mut self, now: Instant) {
        if !self.closed {
            self.q.close(now, VarInt::from_u32(0), Bytes::new());
            self.closed = true;
        }
    }
}

#[derive(Default)]
struct Dial {
    reply: Option<Sender<io::Result<Accepted>>>,
    conns: Vec<ConnectionHandle>,
    tried: Vec<SocketAddr>,
    winner: Option<ConnectionHandle>,
}

pub(crate) struct Driver {
    ep: Endpoint,
    client: ClientConfig,
    net: Net,
    stop: Arc<AtomicBool>,
    conns: BTreeMap<ConnectionHandle, Conn>,
    listen: Option<Sender<Option<Accepted>>>,
    dials: BTreeMap<u64, Dial>,
    punches: Vec<(SocketAddr, Instant)>,
    next_punch: Option<Instant>,
    stuns: BTreeMap<TxId, (Sender<(TxId, SocketAddr)>, Instant)>,
    stopping: Option<Instant>,
    buf: Vec<u8>,
}

/// Um IPv4 dentro de IPv6 é IPv4 (o QUIC identifica a ligação pelo endereço: uma forma só).
fn canonical(a: SocketAddr) -> SocketAddr {
    match a.ip() {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map_or(a, |v4| SocketAddr::new(IpAddr::V4(v4), a.port())),
        IpAddr::V4(_) => a,
    }
}

impl Driver {
    pub(crate) fn new(
        s4: UdpSocket,
        s6: Option<UdpSocket>,
        via: Option<SocketAddr>,
        stop: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        let mut t = TransportConfig::default();
        // Um fluxo por ligação (o fio); nenhum unidireccional.
        t.max_concurrent_bidi_streams(VarInt::from_u32(1))
            .max_concurrent_uni_streams(VarInt::from_u32(0));
        let t = Arc::new(t);
        let mut ec = EndpointConfig::default();
        // Sem «graxa» no bit fixo: todo pacote QUIC nosso tem os 2 bits de cima ≠ 00, o que separa STUN
        // e furos no mesmo socket.
        ec.grease_quic_bit(false);
        let server = Arc::new(tls::server(&t)?);
        Ok(Self {
            ep: Endpoint::new(Arc::new(ec), Some(server), true, None),
            client: tls::client(&t)?,
            net: Net { s4, s6, via },
            stop,
            conns: BTreeMap::new(),
            listen: None,
            dials: BTreeMap::new(),
            punches: Vec::new(),
            next_punch: None,
            stuns: BTreeMap::new(),
            stopping: None,
            buf: Vec::new(),
        })
    }

    pub(crate) fn run(mut self, rx: &Receiver<Msg>, node: &Weak<Inner>) {
        loop {
            let msg = match self.deadline() {
                Some(d) => rx.recv_timeout(d.saturating_duration_since(Instant::now())),
                None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match msg {
                Ok(m) => self.on(m),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => self.begin_stop(),
            }
            for _ in 0..BATCH {
                match rx.try_recv() {
                    Ok(m) => self.on(m),
                    Err(_) => break,
                }
            }
            let now = Instant::now();
            self.drive(now, node);
            if let Some(end) = self.stopping
                && (now >= end || self.conns.values().all(|c| c.closed))
            {
                for c in self.conns.values_mut() {
                    c.close(now);
                }
                self.drive(now, node);
                break;
            }
        }
        self.stop.store(true, Ordering::Release);
    }

    fn deadline(&mut self) -> Option<Instant> {
        let conns = self
            .conns
            .values_mut()
            .flat_map(|c| [c.q.poll_timeout(), c.closing.filter(|_| !c.closed)]);
        conns
            .chain([self.next_punch, self.stopping])
            .flatten()
            .min()
    }

    fn on(&mut self, m: Msg) {
        match m {
            Msg::Datagram(from, data) => self.datagram(from, data),
            Msg::Dial { id, addrs, reply } => self.dial(id, &addrs, reply),
            Msg::Cancel(id) => self.cancel(id),
            Msg::Listen(tx) => {
                if self.stopping.is_none() {
                    self.listen = Some(tx);
                }
            }
            Msg::Write(h, b) => {
                if let Some(c) = self.conns.get_mut(&h) {
                    c.out.push_back(b);
                }
                self.flush(h);
            }
            Msg::Poke(h) => self.read(h),
            Msg::Close(h) => self.close_gracefully(h, Instant::now()),
            Msg::Punch(to, until) => {
                for a in to.into_iter().map(canonical) {
                    self.punches.retain(|p| p.0 != a);
                    self.punches.push((a, until));
                }
                self.next_punch = Some(Instant::now());
            }
            Msg::Stun(to, id, reply) => {
                self.stuns.insert(id, (reply, Instant::now()));
                self.net.send(to, &stun::request(&id));
            }
            Msg::Stop => self.begin_stop(),
        }
    }

    fn begin_stop(&mut self) {
        if self.stopping.is_some() {
            return;
        }
        let now = Instant::now();
        self.stopping = Some(now + CLOSE_GRACE);
        self.listen = None;
        self.dials.clear();
        let hs: Vec<_> = self.conns.keys().copied().collect();
        for h in hs {
            self.close_gracefully(h, now);
        }
    }

    fn datagram(&mut self, from: SocketAddr, data: BytesMut) {
        let (from, data) = if Some(from) == self.net.via {
            let Some((src, p)) = decode_via(&data) else {
                return;
            };
            (src, BytesMut::from(p))
        } else if self.net.via.is_some() {
            // Atrás do NAT simulado nada chega por fora dele.
            return;
        } else {
            (from, data)
        };
        let from = canonical(from);
        if data.first().is_none_or(|b| b & 0xC0 == 0) {
            // Não é QUIC: uma resposta STUN, ou um furo (que só serve para abrir o caminho).
            if let Some((id, mapped)) = stun::response(&data)
                && let Some((r, _)) = self.stuns.remove(&id)
            {
                let _ = r.send((id, mapped));
            }
            return;
        }
        let now = Instant::now();
        self.buf.clear();
        match self.ep.handle(now, from, None, None, data, &mut self.buf) {
            Some(DatagramEvent::ConnectionEvent(h, ev)) => {
                if let Some(c) = self.conns.get_mut(&h) {
                    c.q.handle_event(ev);
                }
            }
            Some(DatagramEvent::NewConnection(inc)) => {
                self.buf.clear();
                if self.listen.is_none() {
                    let t = self.ep.refuse(inc, &mut self.buf);
                    self.net.send(t.destination, &self.buf[..t.size]);
                    return;
                }
                match self.ep.accept(inc, now, &mut self.buf, None) {
                    Ok((h, q)) => {
                        self.conns.insert(h, Conn::new(q, None));
                    }
                    Err(e) => {
                        if let Some(t) = e.response {
                            self.net.send(t.destination, &self.buf[..t.size]);
                        }
                    }
                }
            }
            Some(DatagramEvent::Response(t)) => self.net.send(t.destination, &self.buf[..t.size]),
            None => {}
        }
    }

    fn dial(&mut self, id: u64, addrs: &[SocketAddr], reply: Option<Sender<io::Result<Accepted>>>) {
        if self.stopping.is_some() {
            if let Some(r) = reply {
                let _ = r.send(Err(ErrorKind::BrokenPipe.into()));
            }
            return;
        }
        // Endereços que chegam depois de a tentativa acabar (o ponto de encontro é lento) não a
        // ressuscitam.
        if reply.is_none() && !self.dials.contains_key(&id) {
            return;
        }
        let now = Instant::now();
        let d = self.dials.entry(id).or_default();
        if reply.is_some() {
            d.reply = reply;
        }
        if d.winner.is_some() {
            return;
        }
        for a in addrs.iter().copied().map(canonical) {
            if d.tried.contains(&a) || !self.net.reaches(a) {
                continue;
            }
            d.tried.push(a);
            if let Ok((h, q)) = self.ep.connect(now, self.client.clone(), a, tls::NAME) {
                self.conns.insert(h, Conn::new(q, Some(id)));
                d.conns.push(h);
            }
        }
    }

    fn cancel(&mut self, id: u64) {
        let Some(d) = self.dials.remove(&id) else {
            return;
        };
        let now = Instant::now();
        for h in d.conns.into_iter().filter(|h| Some(*h) != d.winner) {
            if let Some(c) = self.conns.get_mut(&h) {
                c.close(now);
            }
        }
    }

    fn drive(&mut self, now: Instant, node: &Weak<Inner>) {
        let hs: Vec<_> = self.conns.keys().copied().collect();
        for h in hs {
            self.drive_one(h, now, node);
        }
        if self.next_punch.is_some_and(|t| t <= now) {
            self.punches.retain(|p| p.1 > now);
            for (a, _) in &self.punches {
                self.net.send(*a, PUNCH);
            }
            self.next_punch = (!self.punches.is_empty()).then(|| now + PUNCH_EVERY);
        }
        self.stuns.retain(|_, s| now.duration_since(s.1) < STUN_TTL);
    }

    fn drive_one(&mut self, h: ConnectionHandle, now: Instant, node: &Weak<Inner>) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        if c.q.poll_timeout().is_some_and(|t| t <= now) {
            c.q.handle_timeout(now);
        }
        if c.closing.is_some_and(|t| t <= now) {
            c.close(now);
        }
        let mut drained = false;
        while let Some(ev) = c.q.poll_endpoint_events() {
            drained |= ev.is_drained();
            if let Some(ce) = self.ep.handle_event(h, ev) {
                c.q.handle_event(ce);
            }
        }
        let mut events = Vec::new();
        while let Some(e) = c.q.poll() {
            events.push(e);
        }
        for e in events {
            self.event(h, e, now, node);
        }
        self.flush(h);
        if let Some(c) = self.conns.get_mut(&h) {
            let peer_done = c.stream.as_ref().is_some_and(|(_, p)| p.lock().eof);
            if c.closing.is_some() && c.acked && peer_done {
                c.close(now);
            }
            loop {
                self.buf.clear();
                let Some(t) = c.q.poll_transmit(now, 1, &mut self.buf) else {
                    break;
                };
                self.net.send(t.destination, &self.buf[..t.size]);
            }
        }
        if drained && let Some((_, p)) = self.conns.remove(&h).and_then(|c| c.stream) {
            p.lose();
        }
    }

    fn event(&mut self, h: ConnectionHandle, e: Event, now: Instant, node: &Weak<Inner>) {
        match e {
            Event::Connected => self.connected(h, now, node),
            Event::ConnectionLost { .. } => {
                if let Some(c) = self.conns.get_mut(&h) {
                    c.closed = true;
                    if let Some((_, p)) = &c.stream {
                        p.lose();
                    }
                }
            }
            Event::Stream(StreamEvent::Opened { dir: Dir::Bi }) => self.accept_streams(h, node),
            Event::Stream(StreamEvent::Readable { .. }) => self.read(h),
            Event::Stream(StreamEvent::Writable { .. }) => self.flush(h),
            Event::Stream(StreamEvent::Finished { .. }) => {
                if let Some(c) = self.conns.get_mut(&h) {
                    c.acked = true;
                }
            }
            Event::Stream(StreamEvent::Stopped { .. }) => {
                if let Some(c) = self.conns.get_mut(&h) {
                    c.out.clear();
                    if let Some((_, p)) = &c.stream {
                        p.lose();
                    }
                }
            }
            _ => {}
        }
    }

    /// O convidado chegou a um caminho: o primeiro abre o fluxo e é o fio; os outros fecham.
    fn connected(&mut self, h: ConnectionHandle, now: Instant, node: &Weak<Inner>) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        let Some(id) = c.dial else {
            return;
        };
        let Some(d) = self.dials.get_mut(&id).filter(|d| d.winner.is_none()) else {
            c.close(now);
            return;
        };
        let (Some(sid), Some(node)) = (c.q.streams().open(Dir::Bi), node.upgrade()) else {
            c.close(now);
            return;
        };
        let pipe = Arc::new(Pipe::default());
        c.stream = Some((sid, Arc::clone(&pipe)));
        d.winner = Some(h);
        let wire: Box<dyn Wire> = Box::new(QuicWire { node, h, pipe });
        let delivered = d
            .reply
            .take()
            .is_some_and(|r| r.send(Ok((wire, c.q.remote_address()))).is_ok());
        if !delivered {
            c.close(now);
        }
        let losers: Vec<_> = d.conns.iter().copied().filter(|x| *x != h).collect();
        for x in losers {
            if let Some(o) = self.conns.get_mut(&x) {
                o.close(now);
            }
        }
    }

    /// O anfitrião: o fluxo do convidado chegou — é o fio, e vai para a entrada.
    fn accept_streams(&mut self, h: ConnectionHandle, node: &Weak<Inner>) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        while let Some(sid) = c.q.streams().accept(Dir::Bi) {
            if c.stream.is_some() {
                let _ = c.q.recv_stream(sid).stop(VarInt::from_u32(0));
                let _ = c.q.send_stream(sid).reset(VarInt::from_u32(0));
                continue;
            }
            let Some(node) = node.upgrade() else {
                return;
            };
            let pipe = Arc::new(Pipe::default());
            c.stream = Some((sid, Arc::clone(&pipe)));
            let wire: Box<dyn Wire> = Box::new(QuicWire { node, h, pipe });
            let peer = c.q.remote_address();
            let delivered = self
                .listen
                .as_ref()
                .is_some_and(|l| l.send(Some((wire, peer))).is_ok());
            if !delivered {
                c.close(Instant::now());
            }
        }
        self.read(h);
    }

    /// Passa o que o QUIC tem para o tubo, até [`RECV_HIGH`]. Fechado deste lado: lê só para ver o fim.
    fn read(&mut self, h: ConnectionHandle) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        let Some((sid, pipe)) = &c.stream else {
            return;
        };
        let mut st = pipe.lock();
        let discard = st.shut;
        let mut room = if discard {
            usize::MAX
        } else {
            RECV_HIGH.saturating_sub(st.recv_len)
        };
        if room == 0 {
            st.paused = true;
            return;
        }
        let mut rs = c.q.recv_stream(*sid);
        let Ok(mut chunks) = rs.read(true) else {
            return;
        };
        let mut woke = false;
        loop {
            if room == 0 {
                st.paused = true;
                break;
            }
            match chunks.next(room) {
                Ok(Some(ch)) => {
                    woke = true;
                    if !discard {
                        room -= ch.bytes.len();
                        st.recv_len += ch.bytes.len();
                        st.recv.push_back(ch.bytes);
                    }
                }
                Ok(None) => {
                    st.eof = true;
                    woke = true;
                    break;
                }
                Err(ReadError::Blocked) => break,
                Err(ReadError::Reset(_)) => {
                    st.lost = true;
                    woke = true;
                    break;
                }
            }
        }
        let _ = chunks.finalize();
        drop(st);
        if woke {
            pipe.notify();
        }
    }

    /// Passa a fila ao QUIC até ele travar; termina o envio quando ela esvazia e foi pedido.
    fn flush(&mut self, h: ConnectionHandle) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        let Some((sid, pipe)) = &c.stream else {
            c.out.clear();
            return;
        };
        let mut sent = 0;
        while let Some(front) = c.out.front_mut() {
            match c.q.send_stream(*sid).write(front) {
                Ok(n) => {
                    sent += n;
                    if n >= front.len() {
                        c.out.pop_front();
                    } else {
                        front.advance(n);
                    }
                }
                Err(WriteError::Blocked) => break,
                Err(_) => {
                    c.out.clear();
                    pipe.lose();
                    break;
                }
            }
        }
        if sent > 0 {
            pipe.sent(sent);
        }
        if c.finish && c.out.is_empty() {
            c.finish = false;
            let _ = c.q.send_stream(*sid).finish();
        }
    }

    fn close_gracefully(&mut self, h: ConnectionHandle, now: Instant) {
        let Some(c) = self.conns.get_mut(&h) else {
            return;
        };
        if c.closed || c.closing.is_some() {
            return;
        }
        let Some((_, p)) = &c.stream else {
            c.close(now);
            return;
        };
        p.lock().shut = true;
        p.notify();
        c.finish = true;
        c.closing = Some(now + CLOSE_GRACE);
        self.flush(h);
        self.read(h);
    }
}
