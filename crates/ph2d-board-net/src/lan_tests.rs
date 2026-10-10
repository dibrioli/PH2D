//! Quem pode ligar-se: só a rede local.

use std::net::IpAddr;

use crate::{is_lan, lan_addrs};

#[test]
fn a_classificacao_das_origens() {
    let yes = [
        "127.0.0.1",
        "127.8.9.10",
        "10.0.0.1",
        "10.255.255.255",
        "172.16.0.1",
        "172.31.255.254",
        "192.168.0.1",
        "192.168.15.12",
        "169.254.1.1",
        "::1",
        "fc00::1",
        "fd12:3456::1",
        "fe80::eab:b5d9:1953:6c31",
        "febf::1",
        "::ffff:192.168.1.5",
    ];
    let no = [
        "8.8.8.8",
        "172.15.255.255",
        "172.32.0.1",
        "192.169.0.1",
        "11.0.0.1",
        "100.64.0.1",
        "0.0.0.0",
        "255.255.255.255",
        "224.0.0.251",
        "2804:7f7:1a40:765c:2bfe:c5d6:611d:e93b",
        "fec0::1",
        "::ffff:8.8.8.8",
        "::",
        "ff02::fb",
    ];
    for s in yes {
        assert!(is_lan(s.parse::<IpAddr>().unwrap()), "{s} devia ser local");
    }
    for s in no {
        assert!(!is_lan(s.parse::<IpAddr>().unwrap()), "{s} não é local");
    }
}

#[test]
fn os_enderecos_a_mostrar_sao_ipv4_locais() {
    let v = lan_addrs(47_320);
    assert!(!v.is_empty());
    for a in &v {
        assert!(a.is_ipv4() && is_lan(a.ip()) && a.port() == 47_320, "{a}");
    }
}
