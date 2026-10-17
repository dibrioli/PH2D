//! As TENTATIVAS de ligação do convidado no motor (módulo filho de [`super`]): os caminhos de cada uma,
//! o porquê de cada caminho que cai, e a desistência logo que todos caíram com a lista de endereços
//! fechada (auditoria A2, ADR-0187 decisão 8).

use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::time::Instant;

use quinn_proto::{ConnectionError, ConnectionHandle, TransportErrorCode};

use super::{Conn, Driver, canonical};
use crate::pipe::Accepted;
use crate::tls;

#[derive(Default)]
pub(super) struct Dial {
    pub(super) reply: Option<Sender<io::Result<Accepted>>>,
    pub(super) conns: Vec<ConnectionHandle>,
    pub(super) tried: Vec<SocketAddr>,
    pub(super) winner: Option<ConnectionHandle>,
    /// Mais endereços ainda podem chegar: com todos os caminhos caídos, espera em vez de desistir.
    pub(super) open: bool,
    /// Porque caiu o último caminho.
    pub(super) why: Option<ErrorKind>,
}

/// O pacote nem sai desta máquina para lá (sem rota, sem rede, endereço local que já não existe): o
/// caminho está morto agora, não daqui a um prazo.
pub(super) fn unroutable(k: ErrorKind) -> bool {
    matches!(
        k,
        ErrorKind::NetworkUnreachable
            | ErrorKind::HostUnreachable
            | ErrorKind::NetworkDown
            | ErrorKind::AddrNotAvailable
    )
}

/// Porque uma ligação a meio caminho caiu, na língua do `io` (o `room_is_gone` do app lê a recusa).
pub(super) fn lost_kind(e: &ConnectionError) -> ErrorKind {
    match e {
        ConnectionError::ConnectionClosed(c)
            if c.error_code == TransportErrorCode::CONNECTION_REFUSED =>
        {
            ErrorKind::ConnectionRefused
        }
        ConnectionError::TimedOut => ErrorKind::TimedOut,
        ConnectionError::Reset => ErrorKind::ConnectionReset,
        _ => ErrorKind::ConnectionAborted,
    }
}

impl Driver {
    pub(super) fn dial(
        &mut self,
        id: u64,
        addrs: &[SocketAddr],
        reply: Option<Sender<io::Result<Accepted>>>,
        open: bool,
    ) {
        if self.stopping.is_some() {
            if let Some(r) = reply {
                let _ = r.send(Err(ErrorKind::BrokenPipe.into()));
            }
            return;
        }
        // Endereços que chegam depois de a tentativa acabar (o ponto de encontro é lento; ela ganhou,
        // foi cancelada ou desistiu) não a ressuscitam.
        let first = reply.is_some();
        if !first && self.dials.get(&id).is_none_or(|d| d.reply.is_none()) {
            return;
        }
        let now = Instant::now();
        let d = self.dials.entry(id).or_default();
        if first {
            d.reply = reply;
            d.open = open;
        }
        for a in addrs.iter().copied().map(canonical) {
            if d.tried.contains(&a) {
                continue;
            }
            d.tried.push(a);
            if !self.net.reaches(a) {
                d.why = Some(ErrorKind::HostUnreachable);
                continue;
            }
            match self.ep.connect(now, self.client.clone(), a, tls::NAME) {
                Ok((h, q)) => {
                    self.conns.insert(h, Conn::new(q, Some(id)));
                    d.conns.push(h);
                }
                Err(_) => d.why = Some(ErrorKind::AddrNotAvailable),
            }
        }
        self.give_up_if_lost(id);
    }

    /// ⭐ Auditoria A2 — todos os caminhos da tentativa caíram e não vêm mais endereços: o convidado
    /// sabe JÁ (com o porquê do último), em vez de esperar o prazo inteiro.
    pub(super) fn give_up_if_lost(&mut self, id: u64) {
        let Some(d) = self.dials.get_mut(&id) else {
            return;
        };
        if d.open || d.winner.is_some() {
            return;
        }
        let alive = d
            .conns
            .iter()
            .any(|h| self.conns.get(h).is_some_and(|c| !c.closed));
        if alive {
            return;
        }
        if let Some(r) = d.reply.take() {
            let why = d.why.unwrap_or(ErrorKind::HostUnreachable);
            let _ = r.send(Err(why.into()));
        }
    }

    /// Um caminho da tentativa `dial` caiu por `why`.
    pub(super) fn path_lost(&mut self, dial: Option<u64>, why: ErrorKind) {
        let Some(id) = dial else {
            return;
        };
        if let Some(d) = self.dials.get_mut(&id) {
            d.why = Some(why);
        }
        self.give_up_if_lost(id);
    }

    pub(super) fn cancel(&mut self, id: u64) {
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
}
