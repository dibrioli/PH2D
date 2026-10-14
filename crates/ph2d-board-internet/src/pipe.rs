//! O fio de uma ligação QUIC visto como [`Wire`]: um tubo partilhado entre quem lê/escreve (threads
//! do `Host`/`Guest`) e o motor (a thread do nó), com a mesma semântica do TCP — leitura e escrita
//! bloqueantes com prazo, `shutdown` acorda todos os clones, e o que foi escrito antes dele chega.

use std::collections::VecDeque;
use std::io::{self, ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use bytes::{Buf, Bytes};
use ph2d_board_net::{Door, Wire};
use quinn_proto::ConnectionHandle;

use crate::node::{Inner, Msg};

/// Bytes recebidos à espera de quem lê, por ligação: acima disto o motor deixa de ler e o controlo de
/// fluxo do QUIC trava o emissor. Recurso: RAM por ligação; acima da janela por fluxo do `quinn`
/// (1,25 MB por omissão) para nunca ser ele o gargalo.
pub(crate) const RECV_HIGH: usize = 4 << 20;
/// Abaixo disto quem lê volta a acordar o motor.
pub(crate) const RECV_LOW: usize = RECV_HIGH / 2;
/// Bytes escritos que o motor ainda não passou ao QUIC: acima disto a escrita espera (e o prazo de
/// escrita do `Host` derruba quem não lê). Recurso: RAM por ligação.
pub(crate) const SEND_HIGH: usize = 4 << 20;

#[derive(Default)]
pub(crate) struct PipeState {
    pub(crate) recv: VecDeque<Bytes>,
    pub(crate) recv_len: usize,
    /// O outro lado acabou de escrever (e tudo chegou).
    pub(crate) eof: bool,
    /// A ligação caiu.
    pub(crate) lost: bool,
    /// Fechado DESTE lado: leituras dão fim, escritas falham.
    pub(crate) shut: bool,
    /// O motor parou de ler (o tubo encheu); quem lê acorda-o abaixo de [`RECV_LOW`].
    pub(crate) paused: bool,
    /// Escritos e ainda não aceites pelo QUIC.
    pub(crate) pending: usize,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

#[derive(Default)]
pub(crate) struct Pipe {
    st: Mutex<PipeState>,
    cv: Condvar,
}

impl Pipe {
    pub(crate) fn lock(&self) -> MutexGuard<'_, PipeState> {
        self.st.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn notify(&self) {
        self.cv.notify_all();
    }

    /// A ligação caiu: acorda todos.
    pub(crate) fn lose(&self) {
        self.lock().lost = true;
        self.notify();
    }

    /// O QUIC aceitou `n` bytes da fila.
    pub(crate) fn sent(&self, n: usize) {
        let mut st = self.lock();
        st.pending = st.pending.saturating_sub(n);
        drop(st);
        self.notify();
    }

    fn wait<'a>(
        &self,
        g: MutexGuard<'a, PipeState>,
        deadline: Option<Instant>,
    ) -> io::Result<MutexGuard<'a, PipeState>> {
        let Some(d) = deadline else {
            return Ok(self.cv.wait(g).unwrap_or_else(PoisonError::into_inner));
        };
        let left = d.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(ErrorKind::TimedOut.into());
        }
        Ok(self
            .cv
            .wait_timeout(g, left)
            .unwrap_or_else(PoisonError::into_inner)
            .0)
    }
}

pub(crate) struct QuicWire {
    pub(crate) node: Arc<Inner>,
    pub(crate) h: ConnectionHandle,
    pub(crate) pipe: Arc<Pipe>,
}

impl Read for QuicWire {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut st = self.pipe.lock();
        let deadline = st.read_timeout.map(|t| Instant::now() + t);
        loop {
            if st.shut {
                return Ok(0);
            }
            if st.recv_len > 0 {
                let mut n = 0;
                while n < buf.len() {
                    let Some(front) = st.recv.front_mut() else {
                        break;
                    };
                    let k = front.len().min(buf.len() - n);
                    buf[n..n + k].copy_from_slice(&front[..k]);
                    front.advance(k);
                    if front.is_empty() {
                        st.recv.pop_front();
                    }
                    n += k;
                }
                st.recv_len -= n;
                let poke = st.paused && st.recv_len <= RECV_LOW;
                if poke {
                    st.paused = false;
                }
                drop(st);
                if poke {
                    self.node.send(Msg::Poke(self.h));
                }
                return Ok(n);
            }
            if st.eof || st.lost {
                return Ok(0);
            }
            st = self.pipe.wait(st, deadline)?;
        }
    }
}

impl Write for QuicWire {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut st = self.pipe.lock();
        let deadline = st.write_timeout.map(|t| Instant::now() + t);
        let n = loop {
            if st.shut || st.lost {
                return Err(ErrorKind::BrokenPipe.into());
            }
            if st.pending < SEND_HIGH {
                let n = buf.len().min(SEND_HIGH - st.pending);
                st.pending += n;
                break n;
            }
            st = self.pipe.wait(st, deadline)?;
        };
        drop(st);
        if !self
            .node
            .send(Msg::Write(self.h, Bytes::copy_from_slice(&buf[..n])))
        {
            return Err(ErrorKind::BrokenPipe.into());
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Wire for QuicWire {
    fn try_clone(&self) -> io::Result<Box<dyn Wire>> {
        Ok(Box::new(Self {
            node: Arc::clone(&self.node),
            h: self.h,
            pipe: Arc::clone(&self.pipe),
        }))
    }

    fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.pipe.lock().read_timeout = t;
        Ok(())
    }

    fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.pipe.lock().write_timeout = t;
        Ok(())
    }

    fn shutdown(&self) {
        let first = !std::mem::replace(&mut self.pipe.lock().shut, true);
        self.pipe.notify();
        if first {
            self.node.send(Msg::Close(self.h));
        }
    }
}

pub(crate) type Accepted = (Box<dyn Wire>, SocketAddr);

/// A entrada pela internet: o motor entrega aqui cada ligação que abriu o seu fluxo.
pub(crate) struct QuicDoor {
    /// Segura o nó vivo enquanto o `Host` tiver a entrada.
    pub(crate) _node: Arc<Inner>,
    pub(crate) rx: Mutex<Receiver<Option<Accepted>>>,
    pub(crate) tx: Sender<Option<Accepted>>,
}

impl Door for QuicDoor {
    fn accept(&self) -> io::Result<Accepted> {
        let rx = self.rx.lock().unwrap_or_else(PoisonError::into_inner);
        match rx.recv() {
            Ok(Some(a)) => Ok(a),
            Ok(None) => Err(ErrorKind::Interrupted.into()),
            Err(_) => Err(ErrorKind::BrokenPipe.into()),
        }
    }

    fn wake(&self) {
        let _ = self.tx.send(None);
    }
}
