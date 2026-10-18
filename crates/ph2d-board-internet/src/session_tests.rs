//! A sessão inteira em 127.0.0.1: pelo link, pelo ponto de encontro, e atrás de dois NATs de casa —
//! com o controlo sem ponto de encontro, que tem de falhar.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ph2d_board_net::{Announce, Host, HostConfig, Invite, JoinError, Liveness, NetEvent, Waker};

use super::*;
use crate::test_net::{nat, stun_server};

fn waker() -> Waker {
    Arc::new(|| {})
}

fn lan_host() -> Host {
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
        ..HostConfig::default()
    };
    let announce = Announce {
        board: "Q".into(),
        host: "Teste".into(),
    };
    Host::open_with(&announce, &cfg, waker()).unwrap()
}

fn offline() -> InternetConfig {
    InternetConfig {
        stun: vec![],
        dht: DhtMode::Off,
        v6: false,
        port: 0,
        via: None,
    }
}

fn joined_and_talk(h: &Host, g: &ph2d_board_net::Guest) {
    let end = Instant::now() + Duration::from_secs(5);
    let mut ev = Vec::new();
    while !ev.iter().any(|e| matches!(e, NetEvent::Joined(..))) && Instant::now() < end {
        ev.extend(h.poll());
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        ev.iter().any(|e| matches!(e, NetEvent::Joined(..))),
        "{ev:?}"
    );
    g.send(b"oi".to_vec()).unwrap();
    let mut got = Vec::new();
    while got.is_empty() && Instant::now() < end {
        got.extend(h.poll());
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        matches!(&got[..], [NetEvent::Frame(_, b)] if b == b"oi"),
        "{got:?}"
    );
}

#[test]
fn pelo_link_copiado_entra_sem_ponto_de_encontro() {
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &offline()).unwrap();
    assert!(!ih.rendezvous());
    assert!(
        !ih.invite().addrs.is_empty(),
        "os endereços desta máquina vão no link"
    );
    let pasted = Invite::parse(&ih.link()).unwrap();
    let g = join(
        &pasted,
        &offline(),
        Duration::from_secs(5),
        Liveness::default(),
        waker(),
    )
    .expect("entra pelo link");
    joined_and_talk(&h, &g);
}

#[test]
fn um_link_de_outra_sala_diz_que_a_chave_nao_serve() {
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &offline()).unwrap();
    let mut other = ih.invite().clone();
    other.key = ph2d_board_net::InviteKey::random();
    let r = join(
        &other,
        &offline(),
        Duration::from_secs(5),
        Liveness::default(),
        waker(),
    );
    assert_eq!(r.err(), Some(JoinError::WrongCode));
}

#[test]
fn um_link_sem_enderecos_acha_o_anfitriao_no_ponto_de_encontro() {
    let net = mainline::Testnet::builder(10).build().unwrap();
    let cfg = InternetConfig {
        dht: DhtMode::Bootstrap(net.bootstrap.clone()),
        ..offline()
    };
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &cfg).unwrap();
    assert!(ih.rendezvous());
    let bare = Invite {
        key: ih.invite().key.clone(),
        addrs: vec![],
    };
    let g = join(
        &bare,
        &cfg,
        Duration::from_secs(20),
        Liveness::default(),
        waker(),
    )
    .expect("o ponto de encontro deu os endereços");
    joined_and_talk(&h, &g);
}

/// O que o painel avisa: dois destinos STUN a ver portas DIFERENTES = NAT simétrico; o mesmo endereço
/// = furar funciona; um só destino (ou o mesmo IP duas vezes) não chega para dizer.
#[test]
fn o_nat_simetrico_le_se_pelos_destinos_do_stun() {
    use crate::Reflexive;
    let r = |server: &str, mapped: &str| Reflexive {
        server: server.parse().unwrap(),
        mapped: mapped.parse().unwrap(),
    };
    let same = [
        r("198.51.100.1:3478", "203.0.113.5:40000"),
        r("198.51.100.2:3478", "203.0.113.5:40000"),
    ];
    assert_eq!(mapping(&same), Mapping::Independent);
    let moved = [
        r("198.51.100.1:3478", "203.0.113.5:40000"),
        r("198.51.100.2:3478", "203.0.113.5:40007"),
    ];
    assert_eq!(mapping(&moved), Mapping::Dependent);
    let one_ip = [
        r("198.51.100.1:3478", "203.0.113.5:40000"),
        r("198.51.100.1:19302", "203.0.113.5:40007"),
    ];
    assert_eq!(mapping(&one_ip), Mapping::Unknown);
}

/// Um nó que responde e recusa (sem entrada aberta): a máquina do anfitrião ainda lá, a sala não.
fn refusing() -> (Node, std::net::SocketAddr) {
    let n = Node::bind(&NodeConfig::default()).unwrap();
    let at = (std::net::Ipv4Addr::LOCALHOST, n.port()).into();
    (n, at)
}

/// ⭐ Auditoria A2 — sem ponto de encontro, todos os caminhos do link caídos ⇒ o convidado sabe LOGO
/// (com a recusa, que o app lê como «a sala fechou»), e não aos 30 s.
#[test]
fn um_link_de_uma_sala_fechada_falha_logo_e_nao_no_prazo() {
    let (_n, at) = refusing();
    let invite = Invite {
        key: ph2d_board_net::InviteKey::random(),
        addrs: vec![at],
    };
    let t = Instant::now();
    let r = join(
        &invite,
        &offline(),
        Duration::from_secs(30),
        Liveness::default(),
        waker(),
    );
    assert_eq!(
        r.err(),
        Some(JoinError::Unreachable(
            std::io::ErrorKind::ConnectionRefused
        ))
    );
    assert!(t.elapsed() < Duration::from_secs(3), "{:?}", t.elapsed());
}

/// ⭐ A2, o outro lado — com ponto de encontro, o caminho do link caído NÃO acaba a espera: o anfitrião
/// mudou de porta (o link tem a velha) e o ponto de encontro traz a nova.
#[test]
fn com_ponto_de_encontro_o_caminho_caido_nao_acaba_a_espera() {
    let net = mainline::Testnet::builder(10).build().unwrap();
    let cfg = InternetConfig {
        dht: DhtMode::Bootstrap(net.bootstrap.clone()),
        ..offline()
    };
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &cfg).unwrap();
    let (_n, old) = refusing();
    let stale = Invite {
        key: ih.invite().key.clone(),
        addrs: vec![old],
    };
    let g = join(
        &stale,
        &cfg,
        Duration::from_secs(20),
        Liveness::default(),
        waker(),
    )
    .expect("o ponto de encontro deu o endereço novo");
    joined_and_talk(&h, &g);
}

/// A DHT que não responde (o nó de arranque calado) não segura a espera inteira: sem ponto de encontro
/// que funcione e com os caminhos caídos, desiste antes do prazo.
#[test]
fn uma_dht_calada_nao_segura_a_espera_inteira() {
    let mute = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let cfg = InternetConfig {
        dht: DhtMode::Bootstrap(vec![mute.local_addr().unwrap().to_string()]),
        ..offline()
    };
    let (_n, at) = refusing();
    let invite = Invite {
        key: ph2d_board_net::InviteKey::random(),
        addrs: vec![at],
    };
    let t = Instant::now();
    let r = join(
        &invite,
        &cfg,
        Duration::from_secs(30),
        Liveness::default(),
        waker(),
    );
    assert_eq!(
        r.err(),
        Some(JoinError::Unreachable(
            std::io::ErrorKind::ConnectionRefused
        ))
    );
    eprintln!("DHT calada: desistiu em {:?}", t.elapsed());
    assert!(t.elapsed() < Duration::from_secs(20), "{:?}", t.elapsed());
}

/// Duas casas, cada uma atrás do seu NAT (mapeamento independente, filtro por endereço): o convite só
/// leva o endereço de FORA do anfitrião. Sem ponto de encontro o anfitrião nunca sabe para onde furar.
fn two_homes(with_dht: bool) -> Result<(), JoinError> {
    let net = mainline::Testnet::builder(10).build().unwrap();
    let (srv, _stun) = stun_server();
    let (na, nb) = (nat(), nat());
    let dht = if with_dht {
        DhtMode::Bootstrap(net.bootstrap.clone())
    } else {
        DhtMode::Off
    };
    let home = |via| InternetConfig {
        stun: vec![srv],
        dht: dht.clone(),
        v6: false,
        port: 0,
        via: Some(via),
    };
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &home(na.inside)).unwrap();
    assert_eq!(ih.invite().addrs, vec![na.outside], "só o endereço de fora");
    let g = join(
        &Invite::parse(&ih.link()).unwrap(),
        &home(nb.inside),
        Duration::from_secs(if with_dht { 25 } else { 6 }),
        Liveness::default(),
        waker(),
    )?;
    joined_and_talk(&h, &g);
    Ok(())
}

#[test]
fn atras_de_dois_nats_o_ponto_de_encontro_faz_o_anfitriao_furar() {
    two_homes(true).expect("o anfitrião furou até ao convidado");
}

#[test]
fn atras_de_dois_nats_sem_ponto_de_encontro_nao_entra() {
    assert!(matches!(two_homes(false), Err(JoinError::Unreachable(_))));
}

/// ⭐ A12 (W23) — quem volta pelo link depois de cair volta pela MESMA porta (o anfitrião vê o mesmo
/// endereço) e com o MESMO cliente do ponto de encontro; antes, cada tentativa abria um nó e uma DHT.
#[test]
fn quem_volta_pelo_link_volta_pela_mesma_porta_e_o_mesmo_ponto_de_encontro() {
    let net = mainline::Testnet::builder(10).build().unwrap();
    let cfg = InternetConfig {
        dht: DhtMode::Bootstrap(net.bootstrap.clone()),
        ..offline()
    };
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &cfg).unwrap();
    let lg = LinkGuest::new(ih.invite().clone());
    let arrive = |h: &Host| {
        let end = Instant::now() + Duration::from_secs(10);
        while Instant::now() < end {
            if let Some((id, peer)) = h.poll().into_iter().find_map(|e| match e {
                NetEvent::Joined(id, peer) => Some((id, peer)),
                _ => None,
            }) {
                return (id, peer);
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("ninguém chegou");
    };
    let rv_now = |lg: &LinkGuest| {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(rv) = lg.rv.lock().unwrap().clone() {
                return rv;
            }
            assert!(Instant::now() < end, "o ponto de encontro não abriu");
            thread::sleep(Duration::from_millis(10));
        }
    };
    let join = || {
        lg.join(
            &cfg,
            Duration::from_secs(20),
            Liveness::default(),
            waker(),
            |_| {},
        )
    };
    let g = join().expect("entra");
    let (id, first) = arrive(&h);
    let rv = rv_now(&lg);
    h.kick(id);
    drop(g);
    let _g = join().expect("volta");
    let (_, again) = arrive(&h);
    assert_eq!(again, first, "voltou por outra porta");
    assert!(Arc::ptr_eq(&rv, &rv_now(&lg)), "outro cliente da DHT");
}
