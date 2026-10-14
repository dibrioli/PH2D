//! O nó a sério em 127.0.0.1 e ::1: o fio, o anfitrião/convidado por cima, perdas, STUN, e o furo no
//! NAT simulado (com o controlo SEM furo, que tem de falhar).

use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ph2d_board_net::{
    Announce, Guest, Host, HostConfig, InviteKey, JoinError, Liveness, NetEvent, Waker, Wire,
};

use crate::test_net::{lossy, nat, stun_server};
use crate::{CLOSE_GRACE, Node, NodeConfig};

const T: Duration = Duration::from_secs(5);

fn node(v6: bool) -> Node {
    Node::bind(&NodeConfig {
        v6,
        ..NodeConfig::default()
    })
    .unwrap()
}

fn behind(nat: SocketAddr) -> Node {
    Node::bind(&NodeConfig {
        via: Some(nat),
        ..NodeConfig::default()
    })
    .unwrap()
}

fn loop4(n: &Node) -> SocketAddr {
    (Ipv4Addr::LOCALHOST, n.port()).into()
}

/// O fio do anfitrião (pela entrada) e o do convidado (pela tentativa), ligados.
fn pair(server: &Node, client: &Node, to: SocketAddr) -> (Box<dyn Wire>, Box<dyn Wire>) {
    let door = server.door();
    let acc = thread::spawn(move || door.accept().map(|(w, _)| w));
    let (mut cw, peer) = client.dial(&[to]).wait(T).expect("liga");
    assert_eq!(peer, to);
    // O anfitrião só vê o fluxo quando o primeiro byte chega.
    cw.write_all(b"!").unwrap();
    let mut sw = acc.join().unwrap().expect("aceita");
    let mut one = [0u8; 1];
    sw.read_exact(&mut one).unwrap();
    assert_eq!(&one, b"!");
    (sw, cw)
}

fn pattern(n: usize) -> Vec<u8> {
    let mut x: u32 = 0x9e37_79b9;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x.to_le_bytes()[0]
        })
        .collect()
}

fn exchange(mut sw: Box<dyn Wire>, mut cw: Box<dyn Wire>, big: usize) {
    sw.set_read_timeout(Some(T)).unwrap();
    cw.set_read_timeout(Some(T)).unwrap();
    cw.write_all(b"ola").unwrap();
    let mut b3 = [0u8; 3];
    sw.read_exact(&mut b3).unwrap();
    assert_eq!(&b3, b"ola");
    sw.write_all(b"de volta").unwrap();
    let mut b8 = [0u8; 8];
    cw.read_exact(&mut b8).unwrap();
    assert_eq!(&b8, b"de volta");

    // Um quadro grande, maior que o tubo (4 MiB): o controlo de fluxo segura, nada se perde.
    let data = pattern(big);
    let reader = thread::spawn(move || {
        let mut got = vec![0u8; big];
        sw.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
        sw.read_exact(&mut got).unwrap();
        // O fecho do outro lado chega DEPOIS de tudo: aqui é o fim.
        let mut rest = [0u8; 1];
        assert_eq!(sw.read(&mut rest).unwrap(), 0, "fim depois dos dados");
        got
    });
    cw.write_all(&data).unwrap();
    cw.shutdown();
    let mut after = [0u8; 1];
    assert_eq!(cw.read(&mut after).unwrap(), 0, "fechado deste lado: fim");
    assert!(cw.write_all(b"x").is_err(), "fechado deste lado: não escreve");
    let got = reader.join().unwrap();
    assert!(got == data, "o quadro grande chegou trocado");
}

#[test]
fn um_fio_nos_dois_sentidos_em_127_0_0_1() {
    let (a, b) = (node(false), node(false));
    let (sw, cw) = pair(&a, &b, loop4(&a));
    exchange(sw, cw, 10 << 20);
}

#[test]
fn um_fio_nos_dois_sentidos_em_ipv6_loopback() {
    let (a, b) = (node(true), node(true));
    let port = a.port_v6().expect("o IPv6 abriu (::1 tem de existir na máquina de teste)");
    assert!(b.port_v6().is_some());
    let (sw, cw) = pair(&a, &b, (Ipv6Addr::LOCALHOST, port).into());
    exchange(sw, cw, 2 << 20);
}

#[test]
fn com_pacotes_perdidos_chega_tudo_e_na_ordem() {
    let (a, b) = (node(false), node(false));
    let (via, lost, _svc) = lossy(loop4(&a), 7);
    let (sw, cw) = pair(&a, &b, via);
    exchange(sw, cw, 2 << 20);
    assert!(lost.load(Ordering::Relaxed) > 10, "o caminho perdeu de facto");
}

#[test]
fn o_tempo_de_leitura_esgota_como_no_tcp() {
    let (a, b) = (node(false), node(false));
    let (_sw, mut cw) = pair(&a, &b, loop4(&a));
    cw.set_read_timeout(Some(Duration::from_millis(150))).unwrap();
    let t = Instant::now();
    let e = cw.read(&mut [0u8; 4]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::TimedOut);
    assert!(t.elapsed() >= Duration::from_millis(140));
}

#[test]
fn ninguem_a_escuta_e_a_tentativa_esgota() {
    let (a, b) = (node(false), node(false));
    // `a` não abriu entrada: recusa.
    let e = b
        .dial(&[loop4(&a)])
        .wait(Duration::from_millis(1500))
        .err()
        .unwrap();
    assert_eq!(e.kind(), ErrorKind::TimedOut);
}

#[test]
fn o_stun_ve_o_endereco_pelo_mesmo_socket() {
    let (srv, _svc) = stun_server();
    let a = node(false);
    let seen = a.stun(&[srv], T);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].server, srv);
    assert_eq!(seen[0].mapped, loop4(&a));
}

fn bell() -> (Waker, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    (
        Arc::new(move || {
            let _ = tx.send(());
        }),
        rx,
    )
}

fn gather(poll: impl Fn() -> Vec<NetEvent>, done: impl Fn(&[NetEvent]) -> bool) -> Vec<NetEvent> {
    let end = Instant::now() + T;
    let mut all = Vec::new();
    while !done(&all) && Instant::now() < end {
        all.extend(poll());
        thread::sleep(Duration::from_millis(10));
    }
    all
}

fn host_with_internet(key: &InviteKey) -> (Host, Node) {
    let (w, _) = bell();
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
    let mut h = Host::open_with(&announce, &cfg, w).unwrap();
    let a = node(false);
    h.open_door(a.door(), key.clone());
    (h, a)
}

#[test]
fn anfitriao_e_convidado_pela_internet_e_o_adeus_chega_antes_do_corte() {
    let key = InviteKey::random();
    let (h, a) = host_with_internet(&key);
    let b = node(false);
    let (wire, _) = b.dial(&[loop4(&a)]).wait(T).unwrap();
    let (w, _) = bell();
    let g = Guest::join_with_key(wire, &key, T, Liveness::default(), w).expect("a chave entra");
    let ev = gather(|| h.poll(), |e| e.iter().any(|x| matches!(x, NetEvent::Joined(..))));
    let Some(NetEvent::Joined(id, _)) = ev.iter().find(|x| matches!(x, NetEvent::Joined(..)))
    else {
        panic!("{ev:?}");
    };
    g.send(b"do convidado".to_vec()).unwrap();
    let ev = gather(|| h.poll(), |e| !e.is_empty());
    assert_eq!(ev, vec![NetEvent::Frame(*id, b"do convidado".to_vec())]);
    h.send(*id, b"do anfitriao".to_vec()).unwrap();
    let ev = gather(|| g.poll(), |e| !e.is_empty());
    assert_eq!(ev, vec![NetEvent::Frame(0, b"do anfitriao".to_vec())]);

    h.close(b"adeus".to_vec());
    let ev = gather(|| g.poll(), |e| e.contains(&NetEvent::Left(0)));
    assert_eq!(
        ev,
        vec![NetEvent::Frame(0, b"adeus".to_vec()), NetEvent::Left(0)],
        "o adeus chega e SÓ depois o fim"
    );
}

#[test]
fn chave_errada_pela_internet_nao_entra() {
    let key = InviteKey::random();
    let (_h, a) = host_with_internet(&key);
    let b = node(false);
    let (wire, _) = b.dial(&[loop4(&a)]).wait(T).unwrap();
    let (w, _) = bell();
    let r = Guest::join_with_key(wire, &InviteKey::random(), T, Liveness::default(), w);
    assert_eq!(r.err(), Some(JoinError::WrongCode));
}

/// Dois nós, cada um atrás do seu NAT de casa; o anfitrião sabe o seu endereço de fora pelo STUN.
fn two_homes() -> (crate::test_net::Nat, crate::test_net::Nat, Node, Node, SocketAddr) {
    let (na, nb) = (nat(), nat());
    let (a, b) = (behind(na.inside), behind(nb.inside));
    let (srv, svc) = stun_server();
    let seen_a = a.stun(&[srv], T);
    let seen_b = b.stun(&[srv], T);
    assert_eq!(seen_a[0].mapped, na.outside, "o STUN vê o NAT, não o nó");
    assert_eq!(seen_b[0].mapped, nb.outside);
    drop(svc);
    (na, nb, a, b, srv)
}

#[test]
fn sem_furo_o_nat_do_anfitriao_barra_o_convidado() {
    let (na, nb, a, b, _) = two_homes();
    let _door = a.door();
    let r = b.dial(&[na.outside]).wait(Duration::from_secs(3));
    assert_eq!(r.err().map(|e| e.kind()), Some(ErrorKind::TimedOut));
    assert!(na.dropped.load(Ordering::Relaxed) > 0, "o filtro recusou");
    drop(nb);
}

#[test]
fn com_furo_o_convidado_entra_pelo_nat_do_anfitriao() {
    let (na, nb, a, b, _) = two_homes();
    let door = a.door();
    let acc = thread::spawn(move || door.accept().map(|(w, _)| w));
    let dialer = b.dial(&[na.outside]);
    // O anfitrião soube do convidado DEPOIS (o ponto de encontro demora): o 1.º pacote foi barrado.
    thread::sleep(Duration::from_millis(300));
    a.punch(&[nb.outside], Duration::from_secs(10));
    let (mut cw, _) = dialer.wait(T).expect("o furo abriu o caminho");
    cw.write_all(b"pelo furo").unwrap();
    let mut sw = acc.join().unwrap().unwrap();
    let mut got = [0u8; 9];
    sw.set_read_timeout(Some(T)).unwrap();
    sw.read_exact(&mut got).unwrap();
    assert_eq!(&got, b"pelo furo");
    assert!(na.dropped.load(Ordering::Relaxed) > 0, "antes do furo, barrado");
}

#[test]
fn fechar_o_no_solta_a_porta() {
    let a = node(false);
    let port = a.port();
    drop(a);
    let end = Instant::now() + CLOSE_GRACE + Duration::from_secs(1);
    loop {
        if UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).is_ok() {
            break;
        }
        assert!(Instant::now() < end, "a porta {port} ficou presa");
        thread::sleep(Duration::from_millis(20));
    }
}
