//! O transporte a sério: fios TCP em 127.0.0.1, dois processos, e a descoberta (à mão, `#[ignore]`).

mod descoberta;
mod dois_processos;
mod ponta_a_ponta;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use ph2d_board_net::{Announce, Host, HostConfig, NetEvent, Waker};

/// Um `Waker` que toca um sino; quem espera ouve-o.
pub fn bell() -> (Waker, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let w: Waker = Arc::new(move || {
        let _ = tx.send(());
    });
    (w, rx)
}

pub fn announce(board: &str) -> Announce {
    Announce {
        board: board.into(),
        host: "Teste".into(),
    }
}

/// Um anfitrião numa porta qualquer, sem anúncio.
pub fn host() -> (Host, Receiver<()>) {
    let (w, rx) = bell();
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
    };
    (Host::open_with(&announce("Q"), &cfg, w).unwrap(), rx)
}

/// Um anfitrião que se anuncia por mDNS com o quadro `board`.
pub fn host_mdns(board: &str) -> (Host, Receiver<()>) {
    let (w, rx) = bell();
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: true,
    };
    let h = Host::open_with(&announce(board), &cfg, w).unwrap();
    assert_eq!(h.mdns_error(), None);
    (h, rx)
}

pub fn local(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

/// Junta eventos até `done` dizer que chega (ou o prazo acabar).
pub fn gather(
    poll: impl Fn() -> Vec<NetEvent>,
    bell: &Receiver<()>,
    timeout: Duration,
    done: impl Fn(&[NetEvent]) -> bool,
) -> Vec<NetEvent> {
    let end = Instant::now() + timeout;
    let mut all = Vec::new();
    loop {
        all.extend(poll());
        if done(&all) {
            return all;
        }
        let left = end.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return all;
        }
        let _ = bell.recv_timeout(left.min(Duration::from_millis(50)));
    }
}

pub fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_owned()
}
