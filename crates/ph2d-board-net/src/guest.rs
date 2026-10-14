//! O convidado: liga-se ao anfitrião, prova o código e troca quadros só com ele.

use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::code::Code;
use crate::handshake;
use crate::invite::InviteKey;
use crate::host::{
    JOIN_DEADLINE, Liveness, Out, WRITE_TIMEOUT, check, join_until, read_loop, write_loop,
};
use crate::lan::is_lan;
use crate::link::Wire;
use crate::{JoinError, NetEvent, SendError, Waker};

pub struct Guest {
    tx: Option<Sender<Out>>,
    /// Desliga quando a leitura acaba (o escritor só o descobriria na próxima escrita).
    alive: Arc<AtomicBool>,
    events: Receiver<NetEvent>,
    wire: Box<dyn Wire>,
    threads: Vec<JoinHandle<()>>,
}

impl Guest {
    /// Entra na sala em `addr` com o código que a pessoa escreveu (minúsculas, espaços e hífen
    /// aceites). `timeout` vale para ligar e para o aperto de mão.
    ///
    /// # Errors
    /// Ver [`JoinError`].
    pub fn connect(
        addr: SocketAddr,
        code: &str,
        timeout: Duration,
        waker: Waker,
    ) -> Result<Self, JoinError> {
        Self::connect_with(addr, code, timeout, Liveness::default(), waker)
    }

    /// [`Guest::connect`] com o sinal de vida dado.
    ///
    /// # Errors
    /// Ver [`JoinError`].
    pub fn connect_with(
        addr: SocketAddr,
        code: &str,
        timeout: Duration,
        live: Liveness,
        waker: Waker,
    ) -> Result<Self, JoinError> {
        let code = Code::parse(code).ok_or(JoinError::InvalidCode)?;
        if !is_lan(addr.ip()) {
            return Err(JoinError::NotLan);
        }
        let stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(|e| JoinError::Unreachable(e.kind()))?;
        let _ = stream.set_nodelay(true);
        Self::join(Box::new(stream), code.bytes(), timeout, live, waker)
    }

    /// Entra pela entrada da internet por um fio já aberto, com a chave do convite. `timeout` vale
    /// para o aperto de mão.
    ///
    /// # Errors
    /// Ver [`JoinError`] (`WrongCode` = a chave não é a do anfitrião).
    pub fn join_with_key(
        wire: Box<dyn Wire>,
        key: &InviteKey,
        timeout: Duration,
        live: Liveness,
        waker: Waker,
    ) -> Result<Self, JoinError> {
        Self::join(wire, key.bytes(), timeout, live, waker)
    }

    /// Entra por um fio já aberto (de qualquer transporte): aperto de mão, e as threads.
    pub(crate) fn join(
        mut wire: Box<dyn Wire>,
        secret: &[u8],
        timeout: Duration,
        live: Liveness,
        waker: Waker,
    ) -> Result<Self, JoinError> {
        let _ = wire.set_read_timeout(Some(timeout));
        let _ = wire.set_write_timeout(Some(timeout));
        let keys = match handshake::guest_with(&mut wire, secret) {
            Ok(k) => k,
            Err(e) => {
                // O fio é nosso: fecha já (pela internet ficava aberto até ao prazo do QUIC).
                wire.shutdown();
                return Err(e);
            }
        };
        let _ = wire.set_write_timeout(Some(WRITE_TIMEOUT));
        let (Ok(rd), Ok(wr)) = (wire.try_clone(), wire.try_clone()) else {
            wire.shutdown();
            return Err(JoinError::Broken);
        };

        let (ev_tx, events) = mpsc::channel();
        let (tx, rx) = mpsc::channel::<Out>();
        let seal = keys.seal;
        let open = keys.open;
        let alive = Arc::new(AtomicBool::new(true));
        let up = Arc::clone(&alive);
        let writer = thread::spawn(move || write_loop(wr, seal, &rx, live.beat));
        let reader = thread::spawn(move || {
            read_loop(rd, open, live.silence, |bytes| {
                let _ = ev_tx.send(NetEvent::Frame(0, bytes));
                waker();
            });
            up.store(false, Ordering::Release);
            let _ = ev_tx.send(NetEvent::Left(0));
            waker();
        });
        Ok(Self {
            tx: Some(tx),
            alive,
            events,
            wire,
            threads: vec![writer, reader],
        })
    }

    #[must_use]
    pub fn poll(&self) -> Vec<NetEvent> {
        self.events.try_iter().collect()
    }

    /// # Errors
    /// Vazio ou acima de [`crate::MAX_FRAME`], ou a ligação caiu.
    pub fn send(&self, bytes: Vec<u8>) -> Result<(), SendError> {
        check(bytes.len())?;
        let tx = self.tx.as_ref().ok_or(SendError::Gone)?;
        if !self.alive.load(Ordering::Acquire) {
            return Err(SendError::Gone);
        }
        tx.send(Arc::new(bytes)).map_err(|_| SendError::Gone)
    }
}

impl Drop for Guest {
    fn drop(&mut self) {
        let deadline = Instant::now() + JOIN_DEADLINE;
        self.wire.shutdown();
        self.tx = None;
        join_until(std::mem::take(&mut self.threads), deadline);
    }
}
