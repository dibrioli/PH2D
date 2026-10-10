//! A rede LOCAL: quem pode ligar-se, e os endereços desta máquina para mostrar.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Loopback, privada (10/8, 172.16/12, 192.168/16), ligação local (169.254/16), IPv6 único local
/// (fc00::/7) ou de ligação local (fe80::/10). Um IPv4 dentro de IPv6 (`::ffff:a.b.c.d`) julga-se como IPv4.
#[must_use]
pub fn is_lan(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => lan_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return lan_v4(v4);
            }
            let s = v6.segments()[0];
            v6.is_loopback() || (s & 0xfe00) == 0xfc00 || (s & 0xffc0) == 0xfe80
        }
    }
}

fn lan_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local()
}

/// Os IPv4 da rede local desta máquina com `port` (para o anfitrião mostrar); só loopback se não houver.
#[must_use]
pub fn lan_addrs(port: u16) -> Vec<SocketAddr> {
    let mut out: Vec<SocketAddr> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .map(|i| i.ip())
        .filter(|ip| ip.is_ipv4() && !ip.is_loopback() && is_lan(*ip))
        .map(|ip| SocketAddr::new(ip, port))
        .collect();
    out.sort();
    out.dedup();
    if out.is_empty() {
        out.push(SocketAddr::from((Ipv4Addr::LOCALHOST, port)));
    }
    out
}
