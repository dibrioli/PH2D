use super::*;

fn addrs() -> Vec<SocketAddr> {
    vec![
        "[2001:db8::7]:4000".parse().unwrap(),
        "203.0.113.5:47321".parse().unwrap(),
        "192.168.0.10:47321".parse().unwrap(),
    ]
}

#[test]
fn a_cifra_volta_aos_mesmos_enderecos_e_cabe_no_bep44() {
    let k = keys(&InviteKey::random());
    let v = seal(&k, Role::Guest, &addrs());
    assert_eq!(open(&k, Role::Guest, &v), Some(addrs()));
    // O pior caso: o tecto de candidatos, todos IPv6 — o BEP 44 aceita até 1000 B por valor.
    let many = vec!["[2001:db8::1]:1".parse().unwrap(); MAX_ADDRS];
    assert!(seal(&k, Role::Host, &many).len() <= 1000);
}

#[test]
fn outro_convite_ou_outro_papel_nao_abre() {
    let k = keys(&InviteKey::random());
    let v = seal(&k, Role::Host, &addrs());
    assert_eq!(open(&k, Role::Guest, &v), None, "o papel é dado associado");
    assert_eq!(open(&keys(&InviteKey::random()), Role::Host, &v), None);
}

#[test]
fn as_chaves_de_encontro_sao_do_convite() {
    let inv = InviteKey::random();
    let (a, b) = (keys(&inv), keys(&inv));
    assert_eq!(a.host.to_bytes(), b.host.to_bytes(), "os dois lados derivam igual");
    assert_ne!(a.host.to_bytes(), a.guest.to_bytes());
    assert_ne!(a.host.to_bytes(), keys(&InviteKey::random()).host.to_bytes());
}

/// Uma DHT própria em 127.0.0.1: o anfitrião publica, o convidado acha; e o contrário.
#[test]
fn encontram_se_numa_dht_local() {
    let net = mainline::Testnet::builder(10).build().unwrap();
    let inv = InviteKey::random();
    let mode = DhtMode::Bootstrap(net.bootstrap.clone());
    let host = Rendezvous::new(&mode, &inv).unwrap().unwrap();
    let guest = Rendezvous::new(&mode, &inv).unwrap().unwrap();
    assert!(host.publish(Role::Host, &addrs()));
    let (seq, got) = guest.fetch(Role::Host, None).expect("o convidado acha o anfitrião");
    assert_eq!(got, addrs());
    assert_eq!(guest.fetch(Role::Host, Some(seq)), None, "nada mais novo");

    let mine: Vec<SocketAddr> = vec!["198.51.100.9:5000".parse().unwrap()];
    assert!(guest.publish(Role::Guest, &mine));
    assert_eq!(host.fetch(Role::Guest, None).map(|f| f.1), Some(mine));

    // Outro convite na mesma DHT não vê nada.
    let other = Rendezvous::new(&mode, &InviteKey::random()).unwrap().unwrap();
    assert_eq!(other.fetch(Role::Host, None), None);
}
