//! O anfitrião: ouve na porta fixa, autentica quem entra pelo código e é o retransmissor (estrela).

use std::collections::BTreeMap;
use std::io::{self, BufReader, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::code::{Code, Gate};
use crate::discover::Announcer;
use crate::handshake::{self, HostShake};
use crate::lan::{is_lan, lan_addrs};
use crate::wire::{MAX_FRAME, Open, Seal};
use crate::{Announce, ConnId, NetEvent, SendError, Waker};

/// A porta do quadro ao vivo: livre de registos IANA (47101–47556 sem dono) e longe da faixa do
/// Moonlight/Sunshine (47984–48010). Fixa para o dono a poder liberar UMA vez na firewall.
pub const PORT: u16 = 47_320;
/// Porta ocupada ⇒ tenta as seguintes (47320–47329).
pub const PORT_TRIES: u16 = 10;

/// O prazo do aperto de mão (quem pára a meio ocupa um palpite do [`Gate`]).
const SHAKE_TIMEOUT: Duration = Duration::from_secs(5);
/// Um convidado que não lê há este tempo (portátil fechado, Wi-Fi caído) é dado como saído.
pub(crate) const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
/// O `Drop` espera as threads até aqui (fios fechados: acabam em milissegundos).
pub(crate) const JOIN_DEADLINE: Duration = Duration::from_secs(2);

/// O SINAL DE VIDA da ligação: sem ele, quem se cala sem fechar (portátil fechado, Wi-Fi caído) só
/// caía quando uma escrita passasse [`WRITE_TIMEOUT`] — e com ninguém a mexer, nunca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Liveness {
    /// Sem nada a mandar há este tempo, manda-se um quadro VAZIO (20 B na rede).
    pub beat: Duration,
    /// Nada chegou há este tempo (nem o sinal de vida) ⇒ a ligação caiu. É o mesmo prazo da escrita:
    /// o que o Wi-Fi de casa pode calar-se sem a sessão cair; 5 sinais perdidos seguidos.
    pub silence: Duration,
}

impl Default for Liveness {
    fn default() -> Self {
        Self {
            beat: Duration::from_secs(2),
            silence: WRITE_TIMEOUT,
        }
    }
}

pub(crate) type Out = Arc<Vec<u8>>;

/// Onde e como abrir.
#[derive(Clone, Debug)]
pub struct HostConfig {
    /// A primeira porta a tentar (`0` = uma qualquer do sistema; só testes).
    pub port: u16,
    pub tries: u16,
    /// Anunciar por mDNS (os testes ponta-a-ponta não anunciam).
    pub mdns: bool,
    pub live: Liveness,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            port: PORT,
            tries: PORT_TRIES,
            mdns: true,
            live: Liveness::default(),
        }
    }
}

#[derive(Default)]
struct State {
    stop: bool,
    /// Todos os fios aceites (para o `kick` e para o `Drop` os fecharem).
    wires: BTreeMap<ConnId, TcpStream>,
    /// O canal do escritor de cada ligação autenticada.
    senders: BTreeMap<ConnId, Sender<Out>>,
    threads: Vec<JoinHandle<()>>,
}

struct Shared {
    state: Mutex<State>,
    gate: Mutex<Gate>,
    code: Code,
    events: Mutex<Sender<NetEvent>>,
    waker: Waker,
    live: Liveness,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn emit(&self, e: NetEvent) {
        let _ = self
            .events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .send(e);
        (self.waker)();
    }

    /// Regista uma thread; recusada (e devolvida ao chamador) depois do `Drop`.
    fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        let mut st = self.state();
        if st.stop {
            return;
        }
        st.threads.retain(|t| !t.is_finished());
        st.threads.push(thread::spawn(f));
    }
}

pub struct Host {
    shared: Arc<Shared>,
    events: Receiver<NetEvent>,
    port: u16,
    accept: Option<JoinHandle<()>>,
    announcer: Option<Announcer>,
    mdns_error: Option<String>,
}

impl Host {
    /// Abre a sala na porta fixa (ou numa das seguintes) e anuncia-a.
    ///
    /// # Errors
    /// Nenhuma porta livre na faixa.
    pub fn open(announce: &Announce, waker: Waker) -> io::Result<Self> {
        Self::open_with(announce, &HostConfig::default(), waker)
    }

    /// # Errors
    /// Nenhuma porta livre na faixa.
    pub fn open_with(announce: &Announce, cfg: &HostConfig, waker: Waker) -> io::Result<Self> {
        let listener = bind(cfg)?;
        let port = listener.local_addr()?.port();
        let (tx, events) = mpsc::channel();
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            gate: Mutex::new(Gate::default()),
            code: Code::random(),
            events: Mutex::new(tx),
            waker,
            live: cfg.live,
        });
        let sh = Arc::clone(&shared);
        let accept = thread::spawn(move || accept_loop(&listener, &sh));
        let (announcer, mdns_error) = if cfg.mdns {
            match Announcer::start(announce, port) {
                Ok(a) => (Some(a), None),
                Err(e) => (None, Some(e)),
            }
        } else {
            (None, None)
        };
        Ok(Self {
            shared,
            events,
            port,
            accept: Some(accept),
            announcer,
            mdns_error,
        })
    }

    /// O código a mostrar, `ABC-DEF`.
    #[must_use]
    pub fn code(&self) -> String {
        self.shared.code.display()
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Os endereços da rede local onde se pode entrar (para mostrar).
    #[must_use]
    pub fn addrs(&self) -> Vec<SocketAddr> {
        lan_addrs(self.port)
    }

    /// Porque o anúncio na rede falhou (a sala funciona; só não aparece na lista de quem procura).
    #[must_use]
    pub fn mdns_error(&self) -> Option<&str> {
        self.mdns_error.as_deref()
    }

    #[must_use]
    pub fn poll(&self) -> Vec<NetEvent> {
        self.events.try_iter().collect()
    }

    /// # Errors
    /// Acima de [`MAX_FRAME`], ou a ligação já saiu.
    pub fn send(&self, to: ConnId, bytes: Vec<u8>) -> Result<(), SendError> {
        check(bytes.len())?;
        let st = self.shared.state();
        let tx = st.senders.get(&to).ok_or(SendError::Gone)?;
        tx.send(Arc::new(bytes)).map_err(|_| SendError::Gone)
    }

    /// A todas as ligações autenticadas menos `except` (quem mandou).
    ///
    /// # Errors
    /// Acima de [`MAX_FRAME`].
    pub fn broadcast(&self, bytes: Vec<u8>, except: Option<ConnId>) -> Result<(), SendError> {
        check(bytes.len())?;
        let bytes = Arc::new(bytes);
        let st = self.shared.state();
        for (_, tx) in st.senders.iter().filter(|(id, _)| Some(**id) != except) {
            let _ = tx.send(Arc::clone(&bytes));
        }
        Ok(())
    }

    /// Expulsa: o fio fecha e chega o `Left`.
    pub fn kick(&self, id: ConnId) {
        if let Some(w) = self.shared.state().wires.get(&id) {
            let _ = w.shutdown(Shutdown::Both);
        }
    }
}

pub(crate) fn check(len: usize) -> Result<(), SendError> {
    if len == 0 {
        return Err(SendError::Empty);
    }
    if len > MAX_FRAME {
        return Err(SendError::TooBig {
            len,
            max: MAX_FRAME,
        });
    }
    Ok(())
}

fn bind(cfg: &HostConfig) -> io::Result<TcpListener> {
    if cfg.port == 0 {
        return TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0));
    }
    let mut last = io::Error::from(io::ErrorKind::AddrInUse);
    for p in (0..cfg.tries.max(1)).filter_map(|i| cfg.port.checked_add(i)) {
        match TcpListener::bind((Ipv4Addr::UNSPECIFIED, p)) {
            Ok(l) => return Ok(l),
            Err(e) => last = e,
        }
    }
    Err(last)
}

fn accept_loop(listener: &TcpListener, sh: &Arc<Shared>) {
    let mut next: ConnId = 1;
    for stream in listener.incoming() {
        if sh.state().stop {
            break;
        }
        let Ok(stream) = stream else {
            // Sem descritores, por exemplo: não girar a seco.
            thread::sleep(Duration::from_millis(50));
            continue;
        };
        let Ok(peer) = stream.peer_addr() else {
            continue;
        };
        if !is_lan(peer.ip()) {
            continue;
        }
        let id = next;
        next += 1;
        {
            let mut st = sh.state();
            let Ok(clone) = stream.try_clone() else {
                continue;
            };
            st.wires.insert(id, clone);
        }
        let s2 = Arc::clone(sh);
        sh.spawn(move || serve(&s2, id, stream, peer));
    }
}

fn serve(sh: &Arc<Shared>, id: ConnId, mut stream: TcpStream, peer: SocketAddr) {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(SHAKE_TIMEOUT));
    let _ = stream.set_write_timeout(Some(SHAKE_TIMEOUT));
    let keys = match handshake::host(&mut stream, &sh.code, &sh.gate, &Instant::now) {
        HostShake::Joined(k) => k,
        HostShake::Wrong => {
            forget(sh, id);
            sh.emit(NetEvent::WrongCode(peer));
            return;
        }
        HostShake::Refused => {
            forget(sh, id);
            return;
        }
    };
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
    let Ok(out) = stream.try_clone() else {
        forget(sh, id);
        return;
    };
    let (tx, rx) = mpsc::channel::<Out>();
    {
        let mut st = sh.state();
        if st.stop {
            return;
        }
        st.senders.insert(id, tx);
    }
    let (seal, live) = (keys.seal, sh.live);
    sh.spawn(move || write_loop(out, seal, &rx, live.beat));
    sh.emit(NetEvent::Joined(id, peer));
    read_loop(&stream, keys.open, live.silence, |bytes| {
        sh.emit(NetEvent::Frame(id, bytes))
    });
    forget(sh, id);
    sh.emit(NetEvent::Left(id));
}

/// A ligação acabou: fecha o fio e larga o canal (o escritor sai).
fn forget(sh: &Shared, id: ConnId) {
    let mut st = sh.state();
    st.senders.remove(&id);
    if let Some(w) = st.wires.remove(&id) {
        let _ = w.shutdown(Shutdown::Both);
    }
}

/// Lê até o fio fechar, chegar um quadro que não abre, ou nada chegar em `silence`; fecha o fio ao
/// sair. O quadro vazio é o sinal de vida: conta como chegada e não se entrega.
pub(crate) fn read_loop(
    stream: &TcpStream,
    mut open: Open,
    silence: Duration,
    mut on: impl FnMut(Vec<u8>),
) {
    let _ = stream.set_read_timeout(Some(silence));
    let mut r = BufReader::with_capacity(64 << 10, stream);
    while let Ok(bytes) = open.read(&mut r) {
        if !bytes.is_empty() {
            on(bytes);
        }
    }
    let _ = stream.shutdown(Shutdown::Both);
}

/// Escreve o que chega pelo canal, e o sinal de vida quando nada chega em `beat`; sai quando o canal
/// fecha ou o fio falha.
pub(crate) fn write_loop(
    mut stream: TcpStream,
    mut seal: Seal,
    rx: &Receiver<Out>,
    beat: Duration,
) {
    loop {
        let bytes = match rx.recv_timeout(beat) {
            Ok(b) => b,
            Err(mpsc::RecvTimeoutError::Timeout) => Arc::new(Vec::new()),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if seal
            .seal(&bytes)
            .and_then(|w| stream.write_all(&w))
            .is_err()
        {
            break;
        }
    }
    let _ = stream.shutdown(Shutdown::Both);
}

/// Espera as threads até ao prazo; as que ainda correm ficam soltas (os fios delas já fecharam).
pub(crate) fn join_until(threads: Vec<JoinHandle<()>>, deadline: Instant) {
    let mut left = threads;
    while !left.is_empty() && Instant::now() < deadline {
        let (done, run): (Vec<_>, Vec<_>) = left.into_iter().partition(JoinHandle::is_finished);
        for t in done {
            let _ = t.join();
        }
        left = run;
        if !left.is_empty() {
            thread::sleep(Duration::from_millis(2));
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let deadline = Instant::now() + JOIN_DEADLINE;
        if let Some(a) = self.announcer.take() {
            a.stop(deadline);
        }
        let threads = {
            let mut st = self.shared.state();
            st.stop = true;
            for w in st.wires.values() {
                let _ = w.shutdown(Shutdown::Both);
            }
            st.wires.clear();
            st.senders.clear();
            std::mem::take(&mut st.threads)
        };
        // O `accept` está bloqueado: uma ligação a nós próprios acorda-o para ver o `stop`.
        let _ = TcpStream::connect_timeout(
            &SocketAddr::from((Ipv4Addr::LOCALHOST, self.port)),
            Duration::from_millis(200),
        );
        let mut all = threads;
        all.extend(self.accept.take());
        join_until(all, deadline);
    }
}
