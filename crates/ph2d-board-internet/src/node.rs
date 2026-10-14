//! O NÓ UDP: um socket por família e UMA thread (o motor, [`crate::driver`]) que conduz o QUIC e
//! responde a STUN e furos no MESMO socket — a porta que o NAT mapeia é uma só para tudo, e é ela que
//! vai no convite. Vive enquanto alguém o segurar (o `Node`, a entrada, cada fio).

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use bytes::{Bytes, BytesMut};
use ph2d_board_net::{Door, Wire, random_u64};
use quinn_proto::ConnectionHandle;
use socket2::{Domain, Protocol, Socket, Type};

use crate::driver::Driver;
use crate::pipe::{Accepted, QuicDoor};
use crate::stun::TxId;

/// O maior datagrama UDP.
const MAX_DATAGRAM: usize = 65_535;
/// Cada thread de leitura acorda a este ritmo para ver se o nó fechou (sem datagramas, não há outro
/// sinal). Recurso: a latência do fecho dos sockets.
const RECV_POLL: Duration = Duration::from_millis(200);
/// Reenvio dos pedidos STUN sem resposta (a sonda espera 2 s por servidor; aqui são em paralelo).
const STUN_RETRY: Duration = Duration::from_millis(500);

/// Onde e como abrir o nó.
#[derive(Clone, Debug, Default)]
pub struct NodeConfig {
    /// A porta IPv4 (`0` = uma qualquer do sistema).
    pub port: u16,
    /// Abrir também o socket IPv6 (caminho DIRECTO quando os dois lados têm IPv6 global).
    pub v6: bool,
    /// SÓ TESTES: todo o tráfego passa por um NAT simulado neste endereço (cada datagrama leva o
    /// destino/origem à frente, [`encode_via`]).
    #[doc(hidden)]
    pub via: Option<SocketAddr>,
}

/// O que o motor faz por pedido das outras threads.
pub(crate) enum Msg {
    Datagram(SocketAddr, BytesMut),
    Dial {
        id: u64,
        addrs: Vec<SocketAddr>,
        reply: Option<Sender<io::Result<Accepted>>>,
    },
    Cancel(u64),
    Listen(Sender<Option<Accepted>>),
    Write(ConnectionHandle, Bytes),
    Poke(ConnectionHandle),
    Close(ConnectionHandle),
    Punch(Vec<SocketAddr>, Instant),
    Stun(SocketAddr, TxId, Sender<(TxId, SocketAddr)>),
    Stop,
}

pub(crate) struct Inner {
    tx: Sender<Msg>,
    v4: SocketAddr,
    v6: Option<SocketAddr>,
    via: bool,
    next_dial: AtomicU64,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl Inner {
    /// `false` = o motor já parou.
    pub(crate) fn send(&self, m: Msg) -> bool {
        self.tx.send(m).is_ok()
    }
}

impl Drop for Inner {
    /// Pede ao motor que feche as ligações com graça (o que foi escrito chega) e NÃO espera: ele sai
    /// sozinho dentro do prazo de fecho, e as threads de leitura logo a seguir.
    fn drop(&mut self) {
        self.send(Msg::Stop);
        let me = thread::current().id();
        let threads = std::mem::take(&mut *self.threads.lock().unwrap_or_else(PoisonError::into_inner));
        for t in threads.into_iter().filter(|t| t.thread().id() != me) {
            if t.is_finished() {
                let _ = t.join();
            }
        }
    }
}

/// O nó UDP do quadro pela internet.
#[derive(Clone)]
pub struct Node {
    inner: Arc<Inner>,
}

/// Uma tentativa de ligação a vários endereços ao mesmo tempo (o primeiro que responde ganha).
pub struct Dialer {
    node: Arc<Inner>,
    id: u64,
    rx: Receiver<io::Result<Accepted>>,
}

/// O que um servidor STUN viu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reflexive {
    pub server: SocketAddr,
    pub mapped: SocketAddr,
}

fn udp(domain: Domain, addr: SocketAddr) -> io::Result<UdpSocket> {
    let s = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    if domain == Domain::IPV6 {
        // Só IPv6: o IPv4 tem o seu socket (e a sua porta mapeada).
        s.set_only_v6(true)?;
    }
    s.bind(&addr.into())?;
    Ok(s.into())
}

impl Node {
    /// Abre os sockets e o motor.
    ///
    /// # Errors
    /// O socket IPv4 não abre (porta ocupada, por exemplo). O IPv6 que falha só fica de fora.
    pub fn bind(cfg: &NodeConfig) -> io::Result<Self> {
        let s4 = udp(Domain::IPV4, (Ipv4Addr::UNSPECIFIED, cfg.port).into())?;
        let s6 = if cfg.v6 {
            udp(Domain::IPV6, (Ipv6Addr::UNSPECIFIED, 0).into()).ok()
        } else {
            None
        };
        let v4 = s4.local_addr()?;
        let v6 = s6.as_ref().and_then(|s| s.local_addr().ok());
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let mut readers = Vec::new();
        for s in [Some(&s4), s6.as_ref()].into_iter().flatten() {
            let (s, tx, stop) = (s.try_clone()?, tx.clone(), Arc::clone(&stop));
            s.set_read_timeout(Some(RECV_POLL))?;
            readers.push(thread::spawn(move || recv_loop(&s, &tx, &stop)));
        }
        let driver = Driver::new(s4, s6, cfg.via, Arc::clone(&stop))?;
        let inner = Arc::new_cyclic(|weak| {
            let weak = weak.clone();
            let mut threads = readers;
            threads.push(thread::spawn(move || driver.run(&rx, &weak)));
            Inner {
                tx,
                v4,
                v6,
                via: cfg.via.is_some(),
                next_dial: AtomicU64::new(1),
                threads: Mutex::new(threads),
            }
        });
        Ok(Self { inner })
    }

    /// A porta IPv4 local (a que o NAT mapeia).
    #[must_use]
    pub fn port(&self) -> u16 {
        self.inner.v4.port()
    }

    /// Atrás do NAT simulado dos testes (os endereços das interfaces não contam).
    #[must_use]
    pub(crate) fn simulated(&self) -> bool {
        self.inner.via
    }

    /// A porta IPv6 local, se o IPv6 abriu.
    #[must_use]
    pub fn port_v6(&self) -> Option<u16> {
        self.inner.v6.map(|a| a.port())
    }

    /// A entrada do anfitrião por este nó (uma por nó: outra chamada substitui a anterior).
    #[must_use]
    pub fn door(&self) -> Arc<dyn Door> {
        let (tx, rx) = mpsc::channel();
        self.inner.send(Msg::Listen(tx.clone()));
        Arc::new(QuicDoor {
            _node: Arc::clone(&self.inner),
            rx: Mutex::new(rx),
            tx,
        })
    }

    /// Começa a ligar-se a `addrs` (todos ao mesmo tempo).
    #[must_use]
    pub fn dial(&self, addrs: &[SocketAddr]) -> Dialer {
        let id = self.inner.next_dial.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.inner.send(Msg::Dial {
            id,
            addrs: addrs.to_vec(),
            reply: Some(tx),
        });
        Dialer {
            node: Arc::clone(&self.inner),
            id,
            rx,
        }
    }

    /// Manda furos a `to` até `for_` passar: abre o NAT e a firewall DESTE lado para os pacotes de quem
    /// está a tentar ligar-se a nós.
    pub fn punch(&self, to: &[SocketAddr], for_: Duration) {
        self.inner
            .send(Msg::Punch(to.to_vec(), Instant::now() + for_));
    }

    /// Pergunta a cada servidor STUN que endereço vê (pelo MESMO socket das ligações).
    #[must_use]
    pub fn stun(&self, servers: &[SocketAddr], timeout: Duration) -> Vec<Reflexive> {
        let (tx, rx) = mpsc::channel();
        let mut ask: Vec<(TxId, SocketAddr)> = servers
            .iter()
            .map(|s| {
                let mut id = [0u8; 12];
                id[..8].copy_from_slice(&random_u64().to_le_bytes());
                id[8..].copy_from_slice(&random_u64().to_le_bytes()[..4]);
                (id, *s)
            })
            .collect();
        let end = Instant::now() + timeout;
        let mut out = Vec::new();
        while !ask.is_empty() && Instant::now() < end {
            for (id, s) in &ask {
                self.inner.send(Msg::Stun(*s, *id, tx.clone()));
            }
            let round = (Instant::now() + STUN_RETRY).min(end);
            while let Ok((id, mapped)) =
                rx.recv_timeout(round.saturating_duration_since(Instant::now()))
            {
                if let Some(i) = ask.iter().position(|a| a.0 == id) {
                    out.push(Reflexive {
                        server: ask.swap_remove(i).1,
                        mapped,
                    });
                }
                if ask.is_empty() {
                    break;
                }
            }
        }
        out
    }
}

/// Junta endereços a uma tentativa em curso, de outra thread (o ponto de encontro trouxe-os depois).
#[derive(Clone)]
pub struct DialAdder {
    node: Arc<Inner>,
    id: u64,
}

impl DialAdder {
    pub fn add(&self, addrs: &[SocketAddr]) {
        self.node.send(Msg::Dial {
            id: self.id,
            addrs: addrs.to_vec(),
            reply: None,
        });
    }
}

impl Dialer {
    #[must_use]
    pub fn adder(&self) -> DialAdder {
        DialAdder {
            node: Arc::clone(&self.node),
            id: self.id,
        }
    }

    /// Espera o primeiro fio.
    ///
    /// # Errors
    /// `TimedOut` = ninguém respondeu a tempo; outro = todos os caminhos falharam.
    pub fn wait(&self, timeout: Duration) -> io::Result<(Box<dyn Wire>, SocketAddr)> {
        match self.rx.recv_timeout(timeout) {
            Ok(r) => r,
            Err(mpsc::RecvTimeoutError::Timeout) => Err(io::ErrorKind::TimedOut.into()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(io::ErrorKind::BrokenPipe.into()),
        }
    }
}

impl Drop for Dialer {
    /// Fecha os caminhos que perderam (o que ganhou é do fio).
    fn drop(&mut self) {
        self.node.send(Msg::Cancel(self.id));
    }
}

fn recv_loop(s: &UdpSocket, tx: &Sender<Msg>, stop: &AtomicBool) {
    let mut buf = vec![0u8; MAX_DATAGRAM];
    while !stop.load(Ordering::Acquire) {
        match s.recv_from(&mut buf) {
            Ok((n, from)) => {
                if tx
                    .send(Msg::Datagram(from, BytesMut::from(&buf[..n])))
                    .is_err()
                {
                    break;
                }
            }
            // Prazo sem nada, ou um ICMP de porta fechada (o Windows devolve-o no `recv`).
            Err(_) => {}
        }
    }
}

/// SÓ TESTES: o cabeçalho do NAT simulado — família, 16 B de endereço, porta.
pub(crate) const VIA_HEADER: usize = 19;

#[doc(hidden)]
#[must_use]
pub fn encode_via(addr: SocketAddr, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(VIA_HEADER + payload.len());
    match addr {
        SocketAddr::V4(a) => {
            out.push(4);
            out.extend_from_slice(&a.ip().octets());
            out.extend_from_slice(&[0; 12]);
        }
        SocketAddr::V6(a) => {
            out.push(6);
            out.extend_from_slice(&a.ip().octets());
        }
    }
    out.extend_from_slice(&addr.port().to_be_bytes());
    out.extend_from_slice(payload);
    out
}

#[doc(hidden)]
#[must_use]
pub fn decode_via(d: &[u8]) -> Option<(SocketAddr, &[u8])> {
    let head = d.get(..VIA_HEADER)?;
    let port = u16::from_be_bytes([head[17], head[18]]);
    let ip: std::net::IpAddr = match head[0] {
        4 => Ipv4Addr::new(head[1], head[2], head[3], head[4]).into(),
        6 => Ipv6Addr::from(<[u8; 16]>::try_from(&head[1..17]).ok()?).into(),
        _ => return None,
    };
    Some((SocketAddr::new(ip, port), &d[VIA_HEADER..]))
}
