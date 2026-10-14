//! A rede de TESTE em 127.0.0.1: um servidor STUN, um NAT de casa e um caminho que perde pacotes.
//!
//! O NAT é o da casa medida (doc 11 §1): mapeamento INDEPENDENTE do destino (um socket de fora só) e
//! filtro dependente do endereço e da porta (só entra quem já recebeu um pacote de dentro) — o caso em
//! que furar é necessário e suficiente.

use std::collections::BTreeSet;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::node::{decode_via, encode_via};
use crate::stun;

const TICK: Duration = Duration::from_millis(50);

fn sock() -> UdpSocket {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    s.set_read_timeout(Some(TICK)).unwrap();
    s
}

/// Pára e espera as threads ao cair.
pub(crate) struct Service {
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl Drop for Service {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

/// Um servidor STUN: responde com o endereço de onde o pedido veio.
pub(crate) fn stun_server() -> (SocketAddr, Service) {
    let s = sock();
    let addr = s.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let st = Arc::clone(&stop);
    let t = thread::spawn(move || {
        let mut buf = [0u8; 1500];
        while !st.load(Ordering::Acquire) {
            let Ok((n, from)) = s.recv_from(&mut buf) else {
                continue;
            };
            if n >= 20 && buf[..2] == [0, 1] {
                let tx: stun::TxId = buf[8..20].try_into().unwrap();
                let _ = s.send_to(&stun::answer(&tx, from), from);
            }
        }
    });
    (
        addr,
        Service {
            stop,
            threads: vec![t],
        },
    )
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// O NAT de uma casa. O nó por trás dele abre com `NodeConfig::via = Some(nat.inside)`.
pub(crate) struct Nat {
    pub(crate) inside: SocketAddr,
    pub(crate) outside: SocketAddr,
    /// Datagramas de fora recusados pelo filtro.
    pub(crate) dropped: Arc<AtomicUsize>,
    _svc: Service,
}

pub(crate) fn nat() -> Nat {
    let (inside, outside) = (sock(), sock());
    let (ia, oa) = (inside.local_addr().unwrap(), outside.local_addr().unwrap());
    let allowed = Arc::new(Mutex::new(BTreeSet::<SocketAddr>::new()));
    let host = Arc::new(Mutex::new(None::<SocketAddr>));
    let dropped = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let out = {
        let (i, o) = (inside.try_clone().unwrap(), outside.try_clone().unwrap());
        let (al, ho, st) = (Arc::clone(&allowed), Arc::clone(&host), Arc::clone(&stop));
        thread::spawn(move || {
            let mut buf = vec![0u8; 65_535];
            while !st.load(Ordering::Acquire) {
                let Ok((n, from)) = i.recv_from(&mut buf) else {
                    continue;
                };
                let Some((dst, payload)) = decode_via(&buf[..n]) else {
                    continue;
                };
                *lock(&ho) = Some(from);
                lock(&al).insert(dst);
                let _ = o.send_to(payload, dst);
            }
        })
    };
    let back = {
        let (i, o) = (inside, outside);
        let (al, ho, st, dr) = (
            Arc::clone(&allowed),
            Arc::clone(&host),
            Arc::clone(&stop),
            Arc::clone(&dropped),
        );
        thread::spawn(move || {
            let mut buf = vec![0u8; 65_535];
            while !st.load(Ordering::Acquire) {
                let Ok((n, from)) = o.recv_from(&mut buf) else {
                    continue;
                };
                let to = *lock(&ho);
                match to {
                    Some(to) if lock(&al).contains(&from) => {
                        let _ = i.send_to(&encode_via(from, &buf[..n]), to);
                    }
                    _ => {
                        dr.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        })
    };
    Nat {
        inside: ia,
        outside: oa,
        dropped,
        _svc: Service {
            stop,
            threads: vec![out, back],
        },
    }
}

/// Um caminho que perde 1 datagrama em cada `every`, nos dois sentidos, até `server`.
pub(crate) fn lossy(server: SocketAddr, every: usize) -> (SocketAddr, Arc<AtomicUsize>, Service) {
    let s = sock();
    let addr = s.local_addr().unwrap();
    let lost = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let (st, lo) = (Arc::clone(&stop), Arc::clone(&lost));
    let t = thread::spawn(move || {
        let mut buf = vec![0u8; 65_535];
        let (mut client, mut n_seen) = (None, 0usize);
        while !st.load(Ordering::Acquire) {
            let Ok((n, from)) = s.recv_from(&mut buf) else {
                continue;
            };
            n_seen += 1;
            if n_seen % every == 0 {
                lo.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            let to = if from == server {
                let Some(c) = client else {
                    continue;
                };
                c
            } else {
                client = Some(from);
                server
            };
            let _ = s.send_to(&buf[..n], to);
        }
    });
    (
        addr,
        lost,
        Service {
            stop,
            threads: vec![t],
        },
    )
}
