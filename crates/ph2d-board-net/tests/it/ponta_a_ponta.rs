//! Um anfitrião e convidados reais em 127.0.0.1.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use ph2d_board_net::{Guest, HostConfig, JoinError, LOCKOUT, MAX_FAILS, NetEvent, SendError};

use crate::{bell, gather, host, loadavg, local};

const T: Duration = Duration::from_secs(5);

fn frames(ev: &[NetEvent]) -> Vec<(u64, Vec<u8>)> {
    ev.iter()
        .filter_map(|e| match e {
            NetEvent::Frame(id, b) => Some((*id, b.clone())),
            _ => None,
        })
        .collect()
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
fn um_anfitriao_e_dois_convidados_trocam_quadros() {
    let (h, hb) = host();
    let addr = local(h.port());
    let code = h.code();
    let (w1, b1) = bell();
    let (w2, b2) = bell();
    let (c1, c2) = (code.clone(), code.to_lowercase().replace('-', " "));
    let t1 = thread::spawn(move || Guest::connect(addr, &c1, T, w1));
    let t2 = thread::spawn(move || Guest::connect(addr, &c2, T, w2));
    let g1 = t1.join().unwrap().expect("o código certo entra");
    let g2 = t2.join().unwrap().expect("minúsculas e espaços também");

    let ev = gather(|| h.poll(), &hb, T, |e| joined(e).len() == 2);
    assert_eq!(joined(&ev).len(), 2, "{ev:?}");

    // Quem é quem: cada um manda o seu nome.
    g1.send(b"g1".to_vec()).unwrap();
    g2.send(b"g2".to_vec()).unwrap();
    let ev = gather(|| h.poll(), &hb, T, |e| frames(e).len() == 2);
    let f = frames(&ev);
    let id1 = f.iter().find(|(_, b)| b == b"g1").unwrap().0;
    let id2 = f.iter().find(|(_, b)| b == b"g2").unwrap().0;
    assert_ne!(id1, id2);

    // `broadcast` com `except`: o g1 não recebe o seu.
    h.broadcast(b"de g1".to_vec(), Some(id1)).unwrap();
    h.send(id1, b"so para g1".to_vec()).unwrap();
    h.broadcast(b"todos".to_vec(), None).unwrap();
    let e1 = gather(|| g1.poll(), &b1, T, |e| frames(e).len() == 2);
    let e2 = gather(|| g2.poll(), &b2, T, |e| frames(e).len() == 2);
    let f1: Vec<Vec<u8>> = frames(&e1).into_iter().map(|f| f.1).collect();
    let f2: Vec<Vec<u8>> = frames(&e2).into_iter().map(|f| f.1).collect();
    assert_eq!(f1, vec![b"so para g1".to_vec(), b"todos".to_vec()]);
    assert_eq!(f2, vec![b"de g1".to_vec(), b"todos".to_vec()]);
    assert!(frames(&e1).iter().all(|f| f.0 == 0));

    // `kick`: o g1 sai, dos dois lados.
    h.kick(id1);
    let ev = gather(|| h.poll(), &hb, T, |e| e.contains(&NetEvent::Left(id1)));
    assert!(ev.contains(&NetEvent::Left(id1)), "{ev:?}");
    let e1 = gather(|| g1.poll(), &b1, T, |e| e.contains(&NetEvent::Left(0)));
    assert!(e1.contains(&NetEvent::Left(0)));
    assert_eq!(h.send(id1, b"x".to_vec()), Err(SendError::Gone));
    assert_eq!(g1.send(b"x".to_vec()), Err(SendError::Gone));

    // Um convidado que cai: `Left`.
    drop(g2);
    let ev = gather(|| h.poll(), &hb, T, |e| e.contains(&NetEvent::Left(id2)));
    assert!(ev.contains(&NetEvent::Left(id2)), "{ev:?}");
}

#[test]
fn codigo_errado_nos_dois_lados_e_nenhum_quadro() {
    let (h, hb) = host();
    let (w, _b) = bell();
    let wrong = if h.code().starts_with('A') {
        "BBB-BBB"
    } else {
        "AAA-AAA"
    };
    let r = Guest::connect(local(h.port()), wrong, T, w);
    assert_eq!(r.err(), Some(JoinError::WrongCode));
    let ev = gather(|| h.poll(), &hb, T, |e| !e.is_empty());
    assert!(
        matches!(ev[..], [NetEvent::WrongCode(a)] if a.ip().is_loopback()),
        "{ev:?}"
    );
    thread::sleep(Duration::from_millis(100));
    assert!(h.poll().is_empty());
}

#[test]
fn cinco_errados_e_o_certo_fica_a_espera() {
    let (h, hb) = host();
    let wrong = if h.code().starts_with('A') {
        "BBB-BBB"
    } else {
        "AAA-AAA"
    };
    for _ in 0..MAX_FAILS {
        let (w, _b) = bell();
        assert_eq!(
            Guest::connect(local(h.port()), wrong, T, w).err(),
            Some(JoinError::WrongCode)
        );
    }
    // O convidado sabe que errou antes de o anfitrião ver o fio fechar: espera-se o veredito dele
    // (um aperto ainda em curso conta como palpite e daria uma espera de 1 s, não o bloqueio).
    let wrong_events = |e: &[NetEvent]| {
        e.iter()
            .filter(|e| matches!(e, NetEvent::WrongCode(_)))
            .count()
    };
    let ev = gather(
        || h.poll(),
        &hb,
        T,
        |e| wrong_events(e) == MAX_FAILS as usize,
    );
    assert_eq!(wrong_events(&ev), MAX_FAILS as usize, "{ev:?}");
    let (w, _b) = bell();
    match Guest::connect(local(h.port()), &h.code(), T, w).err() {
        Some(JoinError::Locked(d)) => assert!(
            d <= LOCKOUT + Duration::from_secs(1) && d >= LOCKOUT - Duration::from_secs(5),
            "{d:?}"
        ),
        other => panic!("{other:?}"),
    }
    let more = gather(|| h.poll(), &hb, Duration::from_millis(300), |_| false);
    assert!(more.is_empty() && joined(&ev).is_empty(), "{more:?}");
}

#[test]
fn codigo_mal_escrito_e_endereco_de_fora_nem_ligam() {
    let (w, _b) = bell();
    let any = local(1);
    assert_eq!(
        Guest::connect(any, "ABC", T, w.clone()).err(),
        Some(JoinError::InvalidCode)
    );
    assert_eq!(
        Guest::connect(any, "ABC-DE0", T, w.clone()).err(),
        Some(JoinError::InvalidCode)
    );
    let public = "8.8.8.8:47320".parse().unwrap();
    assert_eq!(
        Guest::connect(public, "ABC-DEF", T, w.clone()).err(),
        Some(JoinError::NotLan)
    );
    assert!(matches!(
        Guest::connect(any, "ABC-DEF", T, w).err(),
        Some(JoinError::Unreachable(_))
    ));
}

#[test]
fn lixo_antes_do_codigo_nao_conta_como_palpite() {
    let (h, hb) = host();
    for _ in 0..(MAX_FAILS + 2) {
        let mut s = TcpStream::connect(local(h.port())).unwrap();
        s.write_all(&[200, 0, 0, 0]).unwrap();
        s.write_all(&[0; 200]).unwrap();
        let mut buf = Vec::new();
        let _ = s.read_to_end(&mut buf);
        assert!(buf.is_empty(), "o anfitrião não responde a lixo");
    }
    let (w, _b) = bell();
    let g = Guest::connect(local(h.port()), &h.code(), T, w);
    assert!(g.is_ok(), "{:?}", g.err());
    let ev = gather(|| h.poll(), &hb, T, |e| !joined(e).is_empty());
    assert!(!ev.iter().any(|e| matches!(e, NetEvent::WrongCode(_))));
}

/// Um intermediário que troca UM bit do primeiro quadro cifrado do convidado.
fn corrupting_proxy(to: std::net::SocketAddr, at: usize) -> u16 {
    let l = TcpListener::bind(local(0)).unwrap();
    let port = l.local_addr().unwrap().port();
    thread::spawn(move || {
        let (a, _) = l.accept().unwrap();
        let b = TcpStream::connect(to).unwrap();
        let (mut a2, mut b2) = (a.try_clone().unwrap(), b.try_clone().unwrap());
        thread::spawn(move || {
            let _ = std::io::copy(&mut b2, &mut a2);
            let _ = a2.shutdown(Shutdown::Both);
        });
        let (mut a, mut b) = (a, b);
        let mut seen = 0usize;
        let mut buf = [0u8; 4096];
        while let Ok(n) = a.read(&mut buf) {
            if n == 0 {
                break;
            }
            if (seen..seen + n).contains(&at) {
                buf[at - seen] ^= 0x01;
            }
            seen += n;
            if b.write_all(&buf[..n]).is_err() {
                break;
            }
        }
        let _ = b.shutdown(Shutdown::Both);
    });
    port
}

#[test]
fn um_bit_trocado_no_fio_fecha_a_ligacao_sem_entregar_nada() {
    let (h, hb) = host();
    // Convidado → anfitrião: olá (4 + 8 + 1 + 33) e confirmação (4 + 32) = 82 B; depois o
    // comprimento do 1.º quadro (4 B) e o texto cifrado: troca-se o 1.º byte dele.
    let port = corrupting_proxy(local(h.port()), 82 + 4);
    let (w, gb) = bell();
    let g = Guest::connect(local(port), &h.code(), T, w).expect("o aperto passa intacto");
    g.send(b"um quadro que vai chegar adulterado".to_vec())
        .unwrap();
    let ev = gather(
        || h.poll(),
        &hb,
        T,
        |e| e.iter().any(|e| matches!(e, NetEvent::Left(_))),
    );
    assert!(
        matches!(ev[..], [NetEvent::Joined(..), NetEvent::Left(_)]),
        "{ev:?}"
    );
    let ge = gather(|| g.poll(), &gb, T, |e| e.contains(&NetEvent::Left(0)));
    assert_eq!(ge, vec![NetEvent::Left(0)]);
}

#[test]
fn porta_ocupada_tenta_as_seguintes() {
    let held = TcpListener::bind(("0.0.0.0", 0)).unwrap();
    let p = held.local_addr().unwrap().port();
    let (w, _b) = bell();
    let cfg = HostConfig {
        port: p,
        tries: 10,
        mdns: false,
        ..HostConfig::default()
    };
    if let Ok(h) = ph2d_board_net::Host::open_with(&crate::announce("Q"), &cfg, w.clone()) {
        assert!(h.port() > p && h.port() < p + 10, "{} sobre {p}", h.port());
    }
    let cfg = HostConfig {
        port: p,
        tries: 1,
        mdns: false,
        ..HostConfig::default()
    };
    let e = ph2d_board_net::Host::open_with(&crate::announce("Q"), &cfg, w)
        .err()
        .unwrap();
    assert_eq!(e.kind(), std::io::ErrorKind::AddrInUse);
}

#[test]
fn fechar_o_anfitriao_avisa_os_convidados_e_nao_pendura() {
    let (h, _hb) = host();
    let (w, gb) = bell();
    let g = Guest::connect(local(h.port()), &h.code(), T, w).unwrap();
    let t = Instant::now();
    drop(h);
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
    let ge = gather(|| g.poll(), &gb, T, |e| e.contains(&NetEvent::Left(0)));
    assert_eq!(ge, vec![NetEvent::Left(0)]);
}

/// A latência ida-e-volta (convidado → anfitrião → convidado), 100 B e 25 KB intercalados em blocos
/// rodados; o resumo é o MÍNIMO, a mediana ao lado (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
fn latencia_ida_e_volta() {
    let (h, hb) = host();
    let (w, gb) = bell();
    let g = Guest::connect(local(h.port()), &h.code(), T, w).unwrap();
    let _ = gather(|| h.poll(), &hb, T, |e| !joined(e).is_empty());
    let echo = thread::spawn(move || {
        let mut n = 0usize;
        loop {
            for e in h.poll() {
                match e {
                    NetEvent::Frame(id, b) => {
                        h.send(id, b).unwrap();
                        n += 1;
                    }
                    NetEvent::Left(_) => return n,
                    _ => {}
                }
            }
            let _ = hb.recv_timeout(Duration::from_millis(100));
        }
    });
    let sizes = [100usize, 25_000];
    let (rounds, block) = (7, 30);
    let mut rtt: [Vec<Duration>; 2] = [Vec::new(), Vec::new()];
    let before = loadavg();
    for r in 0..rounds {
        for k in 0..2 {
            let v = (k + r) % 2;
            let msg: Vec<u8> = (0..sizes[v]).map(|i| (i * 31 + r) as u8).collect();
            for _ in 0..block {
                let t = Instant::now();
                g.send(msg.clone()).unwrap();
                let back = loop {
                    if let Some(NetEvent::Frame(0, b)) = g.poll().into_iter().next() {
                        break b;
                    }
                    gb.recv_timeout(T).expect("o eco não voltou");
                };
                rtt[v].push(t.elapsed());
                assert_eq!(back, msg);
            }
        }
    }
    let after = loadavg();
    drop(g);
    assert_eq!(echo.join().unwrap(), 2 * rounds * block);
    for (v, s) in sizes.iter().enumerate() {
        let mut x = rtt[v].clone();
        x.sort();
        println!(
            "latência {s:>6} B: mínimo {:>7.1} µs · mediana {:>7.1} µs · n {}",
            x[0].as_secs_f64() * 1e6,
            x[x.len() / 2].as_secs_f64() * 1e6,
            x.len()
        );
    }
    println!("loadavg antes {before} · depois {after}");
}

/// O quadro maior que se espera: o retrato de 100 mil rectângulos ao vivo (57,5 MiB, `MAX_FRAME`),
/// do anfitrião ao convidado por 127.0.0.1 — o custo é a cifra, não a rede. À mão, em `smoke`.
#[test]
#[ignore = "medição; corre-se à mão com --profile smoke"]
fn um_retrato_grande_de_ponta_a_ponta() {
    let (h, hb) = host();
    let (w, gb) = bell();
    let g = Guest::connect(local(h.port()), &h.code(), T, w).unwrap();
    let ev = gather(|| h.poll(), &hb, T, |e| !joined(e).is_empty());
    let id = joined(&ev)[0];
    let big: Vec<u8> = (0..60_284_299usize).map(|i| (i % 251) as u8).collect();
    let before = loadavg();
    let mut ms = Vec::new();
    for _ in 0..5 {
        let t = Instant::now();
        h.send(id, big.clone()).unwrap();
        let got = gather(
            || g.poll(),
            &gb,
            Duration::from_secs(120),
            |e| !e.is_empty(),
        );
        ms.push(t.elapsed().as_secs_f64() * 1e3);
        assert!(matches!(&got[..], [NetEvent::Frame(0, b)] if *b == big));
    }
    ms.sort_by(f64::total_cmp);
    println!(
        "retrato de 57,5 MiB: mínimo {:.0} ms · mediana {:.0} ms · loadavg antes {before} · depois {}",
        ms[0],
        ms[2],
        loadavg()
    );
}

/// O SINAL DE VIDA: quem se cala sem fechar o fio (portátil fechado, Wi-Fi caído) cai pelo
/// SILÊNCIO, dos dois lados; quem só não tem nada a dizer fica, porque o sinal de vida corre.
#[test]
fn o_silencio_derruba_quem_se_calou_e_o_sinal_de_vida_segura_quem_so_nao_fala() {
    use ph2d_board_net::Liveness;
    let quick = Liveness {
        beat: Duration::from_millis(100),
        silence: Duration::from_millis(500),
    };
    let mute = Liveness {
        beat: Duration::from_secs(3600),
        silence: Duration::from_secs(3600),
    };
    let left = |ev: &[NetEvent]| ev.iter().any(|e| matches!(e, NetEvent::Left(_)));

    // O anfitrião vigia: um convidado vivo e calado fica, um mudo cai.
    let (w, hb) = bell();
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
        live: quick,
    };
    let h = ph2d_board_net::Host::open_with(&crate::announce("Q"), &cfg, w).unwrap();
    let addr = local(h.port());
    let (wq, bq) = bell();
    let quiet = Guest::connect_with(addr, &h.code(), T, quick, wq).expect("entra");
    let ev = gather(|| h.poll(), &hb, T, |e| joined(e).len() == 1);
    let quiet_id = joined(&ev)[0];
    let ev = gather(|| h.poll(), &hb, Duration::from_millis(1500), |_| false);
    assert!(
        !left(&ev),
        "o calado caiu com o sinal de vida a correr: {ev:?}"
    );
    let ev = gather(|| quiet.poll(), &bq, Duration::from_millis(10), |_| false);
    assert!(!left(&ev), "o calado viu o anfitrião cair: {ev:?}");
    let (wm, _bm) = bell();
    let _mudo = Guest::connect_with(addr, &h.code(), T, mute, wm).expect("entra");
    let ev = gather(|| h.poll(), &hb, T, |e| left(e));
    let gone: Vec<u64> = ev
        .iter()
        .filter_map(|e| match e {
            NetEvent::Left(id) => Some(*id),
            _ => None,
        })
        .collect();
    assert_eq!(gone.len(), 1, "o mudo não caiu pelo silêncio: {ev:?}");
    assert_ne!(gone[0], quiet_id, "caiu o calado em vez do mudo");
    drop(h);

    // O convidado vigia: um anfitrião mudo cai do lado de cá (e a sessão tenta voltar).
    let (w, _hb) = bell();
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
        live: mute,
    };
    let h = ph2d_board_net::Host::open_with(&crate::announce("Q"), &cfg, w).unwrap();
    let (wg, bg) = bell();
    let g = Guest::connect_with(local(h.port()), &h.code(), T, quick, wg).expect("entra");
    let ev = gather(|| g.poll(), &bg, T, |e| left(e));
    assert!(left(&ev), "o convidado não deu pelo anfitrião mudo: {ev:?}");
    assert_eq!(g.send(b"x".to_vec()), Err(SendError::Gone));
    assert_eq!(
        g.send(Vec::new()),
        Err(SendError::Empty),
        "o vazio é o sinal de vida"
    );
}

/// Fechar a sala DE PROPÓSITO: o adeus chega a cada convidado ANTES de o fio fechar (o `Drop`
/// sozinho cortava o que estava na fila).
#[test]
fn fechar_a_sala_entrega_o_adeus_antes_de_cortar() {
    let (h, hb) = host();
    let addr = local(h.port());
    let (w1, b1) = bell();
    let (w2, b2) = bell();
    let g1 = Guest::connect(addr, &h.code(), T, w1).expect("entra");
    let g2 = Guest::connect(addr, &h.code(), T, w2).expect("entra");
    let _ = gather(|| h.poll(), &hb, T, |e| joined(e).len() == 2);
    h.close(b"adeus".to_vec());
    for (g, b) in [(&g1, &b1), (&g2, &b2)] {
        let ev = gather(
            || g.poll(),
            b,
            T,
            |e| e.iter().any(|e| matches!(e, NetEvent::Left(_))),
        );
        let at = |want: &dyn Fn(&NetEvent) -> bool| ev.iter().position(|e| want(e));
        let bye = at(&|e| matches!(e, NetEvent::Frame(_, b) if b == b"adeus"));
        let left = at(&|e| matches!(e, NetEvent::Left(_)));
        assert!(
            bye.is_some() && bye < left,
            "o adeus não chegou antes do fim: {ev:?}"
        );
    }
}
