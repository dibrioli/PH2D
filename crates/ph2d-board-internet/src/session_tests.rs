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
    assert!(ev.iter().any(|e| matches!(e, NetEvent::Joined(..))), "{ev:?}");
    g.send(b"oi".to_vec()).unwrap();
    let mut got = Vec::new();
    while got.is_empty() && Instant::now() < end {
        got.extend(h.poll());
        thread::sleep(Duration::from_millis(10));
    }
    assert!(matches!(&got[..], [NetEvent::Frame(_, b)] if b == b"oi"), "{got:?}");
}

#[test]
fn pelo_link_copiado_entra_sem_ponto_de_encontro() {
    let mut h = lan_host();
    let ih = InternetHost::open(&mut h, &offline()).unwrap();
    assert!(!ih.rendezvous());
    assert!(!ih.invite().addrs.is_empty(), "os endereços desta máquina vão no link");
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
