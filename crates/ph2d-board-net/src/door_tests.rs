//! Duas ENTRADAS no mesmo anfitrião (decisões 2 e 5 do handoff W22b): a da rede local com o código
//! curto e o travão, outra com a chave do convite e SEM travão. Quem entra por qualquer uma é igual.

use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crate::tcp::TcpDoor;
use crate::{
    Announce, Guest, Host, HostConfig, InviteKey, JoinError, Liveness, MAX_FAILS, NetEvent, Waker,
};

const T: Duration = Duration::from_secs(5);

fn bell() -> (Waker, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    (
        Arc::new(move || {
            let _ = tx.send(());
        }),
        rx,
    )
}

fn host_with_key(key: &InviteKey) -> (Host, Receiver<()>, SocketAddr) {
    let (w, rx) = bell();
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
    let door = TcpDoor::bind(0, 1).unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], door.port()));
    h.open_door(Arc::new(door), key.clone());
    (h, rx, addr)
}

fn by_key(addr: SocketAddr, key: &InviteKey) -> Result<Guest, JoinError> {
    let (w, _) = bell();
    let s = TcpStream::connect_timeout(&addr, T).unwrap();
    Guest::join_with_key(Box::new(s), key, T, Liveness::default(), w)
}

fn gather(h: &Host, bell: &Receiver<()>, done: impl Fn(&[NetEvent]) -> bool) -> Vec<NetEvent> {
    let end = Instant::now() + T;
    let mut all = Vec::new();
    while !done(&all) && Instant::now() < end {
        let _ = bell.recv_timeout(Duration::from_millis(50));
        all.extend(h.poll());
    }
    all
}

fn joined(ev: &[NetEvent]) -> Vec<u64> {
    ev.iter()
        .filter_map(|e| match e {
            NetEvent::Joined(id, _) => Some(*id),
            _ => None,
        })
        .collect()
}

#[test]
fn a_chave_entra_pela_sua_porta_e_o_codigo_pela_dele() {
    let key = InviteKey::random();
    let (h, hb, addr) = host_with_key(&key);
    let lan = SocketAddr::from(([127, 0, 0, 1], h.port()));
    let (w, gb) = bell();
    let g_lan = Guest::connect(lan, &h.code(), T, w).expect("o código entra pela rede local");
    let g_net = by_key(addr, &key).expect("a chave entra pela outra porta");
    let ev = gather(&h, &hb, |e| joined(e).len() == 2);
    let ids = joined(&ev);
    assert_eq!(ids.len(), 2, "{ev:?}");
    assert_ne!(ids[0], ids[1], "um só contador de ligações para as duas portas");

    // Os dois são iguais para o anfitrião: o `broadcast` chega a ambos.
    h.broadcast(b"todos".to_vec(), None).unwrap();
    let end = Instant::now() + T;
    let mut got = Vec::new();
    while got.is_empty() && Instant::now() < end {
        let _ = gb.recv_timeout(Duration::from_millis(50));
        got.extend(g_lan.poll());
    }
    assert!(got.contains(&NetEvent::Frame(0, b"todos".to_vec())));
    g_net.send(b"da internet".to_vec()).unwrap();
    let ev = gather(&h, &hb, |e| {
        e.iter()
            .any(|x| matches!(x, NetEvent::Frame(_, b) if b == b"da internet"))
    });
    assert!(
        ev.iter()
            .any(|x| matches!(x, NetEvent::Frame(_, b) if b == b"da internet")),
        "{ev:?}"
    );
}

#[test]
fn chave_errada_nao_tranca_a_sala_nem_gasta_o_travao_da_rede_local() {
    let key = InviteKey::random();
    let (h, hb, addr) = host_with_key(&key);
    for _ in 0..=MAX_FAILS {
        assert_eq!(
            by_key(addr, &InviteKey::random()).err(),
            Some(JoinError::WrongCode)
        );
    }
    let ev = gather(&h, &hb, |e| {
        e.iter()
            .filter(|x| matches!(x, NetEvent::WrongCode(_)))
            .count()
            > usize::try_from(MAX_FAILS).unwrap()
    });
    assert!(
        ev.iter()
            .filter(|x| matches!(x, NetEvent::WrongCode(_)))
            .count()
            > usize::try_from(MAX_FAILS).unwrap(),
        "o anfitrião vê cada tentativa: {ev:?}"
    );
    // Sem bloqueio: a chave certa entra logo a seguir, e o código também.
    let _g = by_key(addr, &key).expect("a chave certa entra sem esperar");
    let (w, _) = bell();
    let lan = SocketAddr::from(([127, 0, 0, 1], h.port()));
    let _l = Guest::connect(lan, &h.code(), T, w).expect("o travão da rede local não foi tocado");
}

#[test]
fn o_codigo_nao_abre_a_porta_da_chave_nem_a_chave_a_do_codigo() {
    let key = InviteKey::random();
    let (h, _hb, addr) = host_with_key(&key);
    // O código curto como segredo na porta da chave.
    let (w, _) = bell();
    let s = TcpStream::connect_timeout(&addr, T).unwrap();
    let code = crate::code::Code::parse(&h.code()).unwrap();
    let r = Guest::join(Box::new(s), code.bytes(), T, Liveness::default(), w);
    assert_eq!(r.err(), Some(JoinError::WrongCode));
    // A chave na porta da rede local.
    let lan = SocketAddr::from(([127, 0, 0, 1], h.port()));
    assert_eq!(by_key(lan, &key).err(), Some(JoinError::WrongCode));
}

#[test]
fn fechar_acorda_todas_as_portas() {
    let key = InviteKey::random();
    let (h, _hb, _) = host_with_key(&key);
    let t = Instant::now();
    drop(h);
    assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
}

/// Auditoria W22b (B1): a entrada da chave não tem travão, mas tem TECTO de apertos ao mesmo tempo
/// — acima dele o fio fecha logo, sem thread; os que estavam a meio continuam.
#[test]
fn a_porta_da_chave_tem_tecto_de_apertos_ao_mesmo_tempo() {
    use std::io::Read;
    let key = InviteKey::random();
    let (_h, _hb, addr) = host_with_key(&key);
    let mut mudos: Vec<TcpStream> = (0..crate::MAX_KEY_SHAKES)
        .map(|_| TcpStream::connect_timeout(&addr, T).unwrap())
        .collect();
    // Dá tempo ao `accept` de os contar a todos.
    std::thread::sleep(Duration::from_millis(300));
    let mut demais = TcpStream::connect_timeout(&addr, T).unwrap();
    demais
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let t = Instant::now();
    assert_eq!(demais.read(&mut [0u8; 1]).unwrap_or(0), 0, "fechado logo");
    assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
    // Os mudos continuam abertos (a esperar o aperto até ao prazo).
    mudos[0]
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let e = mudos[0].read(&mut [0u8; 1]).unwrap_err();
    assert!(matches!(
        e.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ));
    // Acabado o prazo deles, há vaga outra vez: a chave certa entra.
    drop(mudos);
    std::thread::sleep(Duration::from_millis(300));
    by_key(addr, &key).expect("com vaga, entra");
}
