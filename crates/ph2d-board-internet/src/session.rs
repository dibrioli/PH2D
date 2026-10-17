//! O que o app usa: abrir a entrada pela internet num `Host` (dá o convite) e entrar por um link.
//!
//! O anfitrião: um nó, o STUN (o endereço de fora), o convite com os candidatos e a chave, e uma
//! thread que publica no ponto de encontro e fura para cada convidado que lá aparece. O convidado:
//! liga-se a todos os candidatos do convite ao mesmo tempo, publica os seus no ponto de encontro (para
//! o anfitrião furar até ele) e junta os que o anfitrião lá tiver; o primeiro caminho que responde é o
//! fio, e por cima dele vai o aperto de mão com a chave.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ph2d_board_net::{Guest, Host, Invite, InviteKey, JoinError, Liveness, Waker};

use crate::candidates::{Mapping, STUN_SERVERS, candidates, mapping, resolve};
use crate::node::{DialAdder, Node, NodeConfig};
use crate::rendezvous::{DhtMode, Rendezvous, Role};

/// Espera pelos servidores STUN (em paralelo; a sonda dá 2 s a cada um).
const STUN_WAIT: Duration = Duration::from_secs(2);
/// O anfitrião olha o item dos convidados a este ritmo. Recurso: uma leitura na DHT (~dezenas de
/// datagramas) por volta, durante a sessão inteira.
const HOST_POLL: Duration = Duration::from_secs(2);
/// O anfitrião volta a publicar os seus candidatos (o BEP 44 guarda ~2 h; o endereço de fora pode
/// mudar com o router).
const HOST_REPUBLISH: Duration = Duration::from_secs(600);
/// O convidado volta a publicar enquanto não entra (outro convidado pode ter escrito por cima).
const GUEST_REPUBLISH: Duration = Duration::from_secs(4);
/// O convidado volta a olhar o item do anfitrião.
const GUEST_POLL: Duration = Duration::from_millis(500);
/// Quanto tempo o anfitrião fura para um convidado novo (o convidado reenvia o 1.º pacote do QUIC
/// com recuo exponencial: 1 s, 2 s, 4 s…).
pub const PUNCH_FOR: Duration = Duration::from_secs(15);
/// O prazo do aperto de mão, depois de o fio existir.
const SHAKE: Duration = Duration::from_secs(10);

/// Onde e como abrir.
#[derive(Clone, Debug)]
pub struct InternetConfig {
    /// Servidores STUN (já resolvidos).
    pub stun: Vec<SocketAddr>,
    pub dht: DhtMode,
    pub v6: bool,
    /// A porta UDP IPv4 (`0` = uma qualquer).
    pub port: u16,
    /// SÓ TESTES: o NAT simulado.
    #[doc(hidden)]
    pub via: Option<SocketAddr>,
}

impl InternetConfig {
    /// A internet a sério: os servidores STUN da sonda (resolve o DNS: BLOQUEIA) e a DHT pública.
    #[must_use]
    pub fn public() -> Self {
        Self {
            stun: resolve(STUN_SERVERS),
            dht: DhtMode::Public,
            v6: true,
            port: 0,
            via: None,
        }
    }

    fn node(&self) -> io::Result<Node> {
        Node::bind(&NodeConfig {
            port: self.port,
            v6: self.v6,
            via: self.via,
        })
    }
}

fn nap(stop: &AtomicBool, d: Duration) {
    let end = Instant::now() + d;
    while !stop.load(Ordering::Acquire) && Instant::now() < end {
        thread::sleep(Duration::from_millis(50).min(end.saturating_duration_since(Instant::now())));
    }
}

/// A entrada pela internet de um anfitrião.
pub struct InternetHost {
    node: Node,
    invite: Invite,
    mapping: Mapping,
    dht: bool,
    stop: Arc<AtomicBool>,
    watcher: Option<JoinHandle<()>>,
}

impl InternetHost {
    /// [`InternetHost::prepare`] e [`InternetHost::attach`] de uma vez.
    ///
    /// # Errors
    /// O socket UDP não abriu.
    pub fn open(host: &mut Host, cfg: &InternetConfig) -> io::Result<Self> {
        let ih = Self::prepare(cfg)?;
        ih.attach(host);
        Ok(ih)
    }

    /// O nó, o STUN, o convite e a vigia do ponto de encontro. BLOQUEIA o STUN (até 2 s): numa
    /// thread, fora da UI. Quem chegar antes do [`InternetHost::attach`] é recusado e volta a tentar.
    ///
    /// # Errors
    /// O socket UDP não abriu.
    pub fn prepare(cfg: &InternetConfig) -> io::Result<Self> {
        let node = cfg.node()?;
        let seen = node.stun(&cfg.stun, STUN_WAIT);
        let invite = Invite {
            key: InviteKey::random(),
            addrs: candidates(&node, &seen),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let rv = Rendezvous::new(&cfg.dht, &invite.key).ok().flatten();
        let dht = rv.is_some();
        let watcher = rv.map(|rv| {
            let (node, mine, stop) = (node.clone(), invite.addrs.clone(), Arc::clone(&stop));
            thread::spawn(move || watch_guests(&rv, &node, &mine, &stop))
        });
        Ok(Self {
            node,
            invite,
            mapping: mapping(&seen),
            dht,
            stop,
            watcher,
        })
    }

    /// Abre a entrada no `host` (instantâneo): quem tiver o link entra.
    pub fn attach(&self, host: &mut Host) {
        host.open_door(self.node.door(), self.invite.key.clone());
    }

    /// O link a mandar a quem entra.
    #[must_use]
    pub fn link(&self) -> String {
        self.invite.link()
    }

    #[must_use]
    pub fn invite(&self) -> &Invite {
        &self.invite
    }

    /// Como o NAT desta casa mapeia (o que os STUN viram).
    #[must_use]
    pub fn mapping(&self) -> Mapping {
        self.mapping
    }

    /// O ponto de encontro está ligado.
    #[must_use]
    pub fn rendezvous(&self) -> bool {
        self.dht
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.node.port()
    }
}

impl Drop for InternetHost {
    /// A vigia acaba sozinha depois da operação na DHT em curso (segundos); não se espera por ela.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.watcher.take().filter(|t| t.is_finished()) {
            let _ = t.join();
        }
    }
}

fn watch_guests(rv: &Rendezvous, node: &Node, mine: &[SocketAddr], stop: &AtomicBool) {
    let (mut published, mut seen) = (None::<Instant>, None);
    while !stop.load(Ordering::Acquire) {
        if published.is_none_or(|t| t.elapsed() >= HOST_REPUBLISH) && rv.publish(Role::Host, mine) {
            published = Some(Instant::now());
        }
        if let Some((seq, addrs)) = rv.fetch(Role::Guest, seen) {
            seen = Some(seq);
            node.punch(&addrs, PUNCH_FOR);
        }
        nap(stop, HOST_POLL);
    }
}

/// Entra pelo convite: o fio pelo primeiro caminho que responder em `timeout`, e o aperto de mão.
///
/// # Errors
/// `Unreachable` = nenhum caminho até ao anfitrião — logo que todos caem e o ponto de encontro já
/// respondeu, sem esperar o prazo (o porquê em [`crate::Dialer::wait`]); `WrongCode` = a chave não é
/// a dele (o link é de outra sala ou a sala fechou e abriu de novo); os outros, como na rede local.
pub fn join(
    invite: &Invite,
    cfg: &InternetConfig,
    timeout: Duration,
    live: Liveness,
    waker: Waker,
) -> Result<Guest, JoinError> {
    join_noting(invite, cfg, timeout, live, waker, |_| {})
}

/// [`join`], e `nat` recebe como o NAT DESTE lado mapeia, logo que os STUN respondem (o painel avisa
/// do simétrico) — sempre ANTES do resultado.
///
/// # Errors
/// Os de [`join`].
pub fn join_noting(
    invite: &Invite,
    cfg: &InternetConfig,
    timeout: Duration,
    live: Liveness,
    waker: Waker,
    nat: impl FnOnce(Mapping) + Send + 'static,
) -> Result<Guest, JoinError> {
    let node = cfg.node().map_err(|e| JoinError::Unreachable(e.kind()))?;
    // Aberta: o ponto de encontro ainda pode trazer os endereços do anfitrião depois.
    let dialer = node.dial_open(&invite.addrs);
    let stop = Arc::new(AtomicBool::new(false));
    let helper = {
        let (node, adder, key, cfg, stop) = (
            node.clone(),
            dialer.adder(),
            invite.key.clone(),
            cfg.clone(),
            Arc::clone(&stop),
        );
        thread::spawn(move || announce_guest(&node, &adder, &key, &cfg, &stop, nat))
    };
    let got = dialer.wait(timeout);
    stop.store(true, Ordering::Release);
    drop(dialer);
    if helper.is_finished() {
        let _ = helper.join();
    }
    let (wire, _) = got.map_err(|e| JoinError::Unreachable(e.kind()))?;
    Guest::join_with_key(wire, &invite.key, SHAKE, live, waker)
}

/// O convidado no ponto de encontro: publica os seus candidatos (para o anfitrião furar) e junta os do
/// anfitrião à tentativa. A lista de endereços FECHA ([`DialAdder::done`]) quando o item do anfitrião
/// chegou, ou quando não há ponto de encontro (nenhum, ou a DHT não responde): só então a tentativa
/// sem caminhos vivos desiste antes do prazo.
fn announce_guest(
    node: &Node,
    adder: &DialAdder,
    key: &InviteKey,
    cfg: &InternetConfig,
    stop: &AtomicBool,
    nat: impl FnOnce(Mapping),
) {
    let seen = node.stun(&cfg.stun, STUN_WAIT);
    nat(mapping(&seen));
    let Ok(Some(rv)) = Rendezvous::new(&cfg.dht, key) else {
        adder.done();
        return;
    };
    let mine = candidates(node, &seen);
    let (mut published, mut host_seq, mut asked) = (None::<Instant>, None, false);
    while !stop.load(Ordering::Acquire) {
        if published.is_none_or(|t| t.elapsed() >= GUEST_REPUBLISH) {
            if rv.publish(Role::Guest, &mine) {
                published = Some(Instant::now());
            } else if !std::mem::replace(&mut asked, true) && !rv.reachable() {
                adder.done();
                return;
            }
        }
        if stop.load(Ordering::Acquire) {
            break;
        }
        if let Some((seq, addrs)) = rv.fetch(Role::Host, host_seq) {
            adder.add(&addrs);
            if host_seq.is_none() {
                adder.done();
            }
            host_seq = Some(seq);
        }
        nap(stop, GUEST_POLL);
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
