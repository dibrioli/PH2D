//! Onde um nó se deixa encontrar: as interfaces (rede local, IPv6 global) e o que os servidores STUN
//! vêem de fora. Vão no convite e no ponto de encontro.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use ph2d_board_net::{MAX_ADDRS, lan_addrs};

use crate::node::{Node, Reflexive};

/// Os servidores STUN da sonda (`sonda_rede.py`), um por DESTINO distinto (o `stun1` do Google cai no
/// mesmo destino que o `stun`, doc 11 §1): três destinos dizem se o mapeamento depende do destino.
pub const STUN_SERVERS: &[(&str, u16)] = &[
    ("stun.l.google.com", 19_302),
    ("stun.cloudflare.com", 3_478),
    ("global.stun.twilio.com", 3_478),
];

/// Resolve os nomes (DNS, bloqueante): IPv4 e IPv6 de cada um.
#[must_use]
pub fn resolve(servers: &[(&str, u16)]) -> Vec<SocketAddr> {
    let mut out: Vec<SocketAddr> = servers
        .iter()
        .filter_map(|s| s.to_socket_addrs().ok())
        .flat_map(|it| {
            let all: Vec<SocketAddr> = it.collect();
            // Um por família e por servidor: o mapeamento conta destinos, não registos de DNS.
            let v4 = all.iter().find(|a| a.is_ipv4()).copied();
            let v6 = all.iter().find(|a| a.is_ipv6()).copied();
            [v4, v6].into_iter().flatten()
        })
        .collect();
    out.dedup();
    out
}

/// Como o NAT deste lado mapeia, pelo que os servidores STUN viram (IPv4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mapping {
    /// Menos de dois destinos IPv4 responderam.
    Unknown,
    /// O mesmo endereço para todos os destinos: furar funciona.
    Independent,
    /// Endereços diferentes por destino (NAT simétrico): furar não chega — o caso do retransmissor.
    Dependent,
}

#[must_use]
pub fn mapping(seen: &[Reflexive]) -> Mapping {
    let v4: Vec<&Reflexive> = seen.iter().filter(|r| r.server.is_ipv4()).collect();
    let mut servers: Vec<IpAddr> = v4.iter().map(|r| r.server.ip()).collect();
    servers.sort();
    servers.dedup();
    if servers.len() < 2 {
        return Mapping::Unknown;
    }
    if v4.iter().all(|r| r.mapped == v4[0].mapped) {
        Mapping::Independent
    } else {
        Mapping::Dependent
    }
}

fn global_v6(ip: IpAddr) -> bool {
    let IpAddr::V6(v6) = ip else {
        return false;
    };
    let s = v6.segments()[0];
    // Global unicast (2000::/3) ou único local (fc00::/7); nunca a de ligação local (pede o `%scope`).
    !v6.is_loopback() && ((s & 0xe000) == 0x2000 || (s & 0xfe00) == 0xfc00)
}

/// Os candidatos deste nó: IPv6 primeiro (directo, sem NAT), depois o que o STUN viu, depois a rede
/// local. Sem repetidos, no máximo [`MAX_ADDRS`].
#[must_use]
pub fn candidates(node: &Node, seen: &[Reflexive]) -> Vec<SocketAddr> {
    let v6: Vec<SocketAddr> = node.port_v6().map_or_else(Vec::new, |port| {
        if_addrs::get_if_addrs()
            .unwrap_or_default()
            .into_iter()
            .map(|i| i.ip())
            .filter(|ip| global_v6(*ip))
            .map(|ip| SocketAddr::new(ip, port))
            .collect()
    });
    let mut out = Vec::new();
    if node.simulated() {
        out.extend(seen.iter().map(|r| r.mapped));
        out.dedup();
        return out;
    }
    for a in v6
        .into_iter()
        .chain(seen.iter().map(|r| r.mapped))
        .chain(lan_addrs(node.port()))
    {
        if !out.contains(&a) && out.len() < MAX_ADDRS {
            out.push(a);
        }
    }
    out
}
