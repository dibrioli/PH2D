//! Encontrar o quadro pelo nome: mDNS (`_ph2d-board._tcp.local.`). O TXT leva o nome do quadro e do
//! anfitrião — NUNCA o código.

use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use mdns_sd::{IfKind, Receiver, ServiceDaemon, ServiceEvent, ServiceInfo};
use rand_core::{OsRng, RngCore};

use crate::Announce;
use crate::lan::is_lan;

pub const SERVICE: &str = "_ph2d-board._tcp.local.";
/// Um valor de TXT cabe em 255 B com a chave; o nome corta-se numa fronteira de carácter.
const TXT_MAX: usize = 200;

/// Uma sala encontrada na rede.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub board: String,
    pub host: String,
    pub addr: SocketAddr,
}

fn cut(s: &str) -> &str {
    let mut end = s.len().min(TXT_MAX);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn tag() -> String {
    format!("{:016x}", OsRng.next_u64())
}

fn stop_daemon(d: &ServiceDaemon, deadline: Instant) {
    if let Ok(rx) = d.shutdown() {
        let _ = rx.recv_timeout(deadline.saturating_duration_since(Instant::now()));
    }
}

/// O anúncio de uma sala aberta.
pub(crate) struct Announcer {
    daemon: ServiceDaemon,
    fullname: String,
}

impl Announcer {
    pub(crate) fn start(a: &Announce, port: u16) -> Result<Self, String> {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        // Só IPv4: a sala ouve em IPv4.
        let _ = daemon.disable_interface(IfKind::IPv6);
        // Nome de máquina e de instância PRÓPRIOS: o do sistema já é anunciado pelo avahi, e dois
        // anunciantes do mesmo nome com conjuntos de endereços diferentes entram em conflito.
        let t = tag();
        let props = [("board", cut(&a.board)), ("host", cut(&a.host)), ("v", "1")];
        let info = ServiceInfo::new(
            SERVICE,
            &format!("PH2D {t}"),
            &format!("ph2d-{t}.local."),
            (),
            port,
            &props[..],
        )
        .map_err(|e| e.to_string())?
        .enable_addr_auto();
        let fullname = info.get_fullname().to_owned();
        daemon.register(info).map_err(|e| e.to_string())?;
        Ok(Self { daemon, fullname })
    }

    pub(crate) fn stop(self, deadline: Instant) {
        if let Ok(rx) = self.daemon.unregister(&self.fullname) {
            // O adeus (TTL 0) sai antes de o daemon parar: quem procura tira a sala da lista já.
            let _ = rx.recv_timeout(
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(500)),
            );
        }
        stop_daemon(&self.daemon, deadline);
    }
}

/// Uma procura contínua (a lista da UI cresce enquanto está aberta).
pub struct Discovery {
    daemon: ServiceDaemon,
    rx: Receiver<ServiceEvent>,
    found: BTreeMap<String, Found>,
}

impl Discovery {
    /// # Errors
    /// O mDNS não abriu (sem rede, porta 5353 recusada).
    pub fn start() -> Result<Self, String> {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let _ = daemon.disable_interface(IfKind::IPv6);
        let rx = daemon.browse(SERVICE).map_err(|e| e.to_string())?;
        Ok(Self {
            daemon,
            rx,
            found: BTreeMap::new(),
        })
    }

    fn take(&mut self, e: ServiceEvent) {
        match e {
            ServiceEvent::ServiceResolved(s) => {
                let txt = |k: &str| s.get_property_val_str(k).unwrap_or_default().to_owned();
                let ip = best(s.get_addresses().iter().map(mdns_sd::ScopedIp::to_ip_addr));
                if let Some(ip) = ip {
                    let f = Found {
                        board: txt("board"),
                        host: txt("host"),
                        addr: SocketAddr::new(ip, s.get_port()),
                    };
                    self.found.insert(s.fullname.clone(), f);
                }
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                self.found.remove(&fullname);
            }
            _ => {}
        }
    }

    /// As salas vistas até agora, por nome de quadro.
    pub fn poll(&mut self) -> Vec<Found> {
        while let Ok(e) = self.rx.try_recv() {
            self.take(e);
        }
        self.list()
    }

    /// Espera até `deadline` ou até `done` dizer que chega.
    pub fn wait(
        &mut self,
        deadline: Instant,
        mut done: impl FnMut(&[Found]) -> bool,
    ) -> Vec<Found> {
        loop {
            let list = self.poll();
            let left = deadline.saturating_duration_since(Instant::now());
            if done(&list) || left.is_zero() {
                return list;
            }
            if let Ok(e) = self.rx.recv_timeout(left) {
                self.take(e);
            }
        }
    }

    fn list(&self) -> Vec<Found> {
        let mut v: Vec<Found> = self.found.values().cloned().collect();
        v.sort_by(|a, b| (&a.board, &a.host, a.addr).cmp(&(&b.board, &b.host, b.addr)));
        v
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        stop_daemon(&self.daemon, Instant::now() + Duration::from_secs(1));
    }
}

/// O endereço a usar: IPv4 da rede local, de preferência não-loopback.
fn best(ips: impl Iterator<Item = IpAddr>) -> Option<IpAddr> {
    let mut v: Vec<IpAddr> = ips.filter(|ip| ip.is_ipv4() && is_lan(*ip)).collect();
    v.sort_by_key(|ip| (ip.is_loopback(), *ip));
    v.first().copied()
}

/// Procura salas durante `wait`.
#[must_use]
pub fn discover(wait: Duration) -> Vec<Found> {
    let Ok(mut d) = Discovery::start() else {
        return Vec::new();
    };
    d.wait(Instant::now() + wait, |_| false)
}
