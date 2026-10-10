//! Dois PROCESSOS reais por localhost: este binário de teste relança-se a si próprio como anfitrião e
//! como convidado (`PH2D_NET_ROLE`), e o pai só coordena e mata pelo `Child` se passar do prazo.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ph2d_board_net::{Guest, NetEvent};

use crate::{bell, gather, host, host_mdns, local};

const ROLE: &str = "PH2D_NET_ROLE";
const N: usize = 10;
const DEADLINE: Duration = Duration::from_secs(30);

pub fn child(role: &str, envs: &[(&str, String)]) -> Child {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args([
        "--exact",
        "dois_processos::papel_filho",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ])
    .env(ROLE, role)
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit());
    for (k, v) in envs {
        c.env(k, v);
    }
    c.spawn().unwrap()
}

/// As linhas do filho, por um canal (para esperar com prazo).
pub fn lines(c: &mut Child) -> mpsc::Receiver<String> {
    let out = c.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for l in BufReader::new(out).lines().map_while(Result::ok) {
            let _ = tx.send(l);
        }
    });
    rx
}

/// A porta e o código que o anfitrião-filho anuncia (as linhas antes são do `libtest`).
pub fn ready(rx: &mpsc::Receiver<String>, timeout: Duration) -> Option<(u16, String)> {
    let end = Instant::now() + timeout;
    while let Ok(l) = rx.recv_timeout(end.saturating_duration_since(Instant::now())) {
        if let Some(rest) = l.strip_prefix("PRONTO ") {
            let mut it = rest.split(' ');
            return Some((it.next()?.parse().ok()?, it.next()?.to_owned()));
        }
    }
    None
}

pub fn wait(c: &mut Child, end: Instant) -> bool {
    while Instant::now() < end {
        if let Ok(Some(s)) = c.try_wait() {
            return s.success();
        }
        thread::sleep(Duration::from_millis(20));
    }
    let _ = c.kill();
    let _ = c.wait();
    false
}

#[test]
fn um_anfitriao_e_um_convidado_em_processos_diferentes() {
    let end = Instant::now() + DEADLINE;
    let mut h = child("host", &[]);
    let hl = lines(&mut h);
    let Some((port, code)) = ready(&hl, DEADLINE) else {
        let _ = h.kill();
        let _ = h.wait();
        panic!("o anfitrião não ficou pronto");
    };
    // O código como uma pessoa o escreveria: minúsculas, espaços.
    let typed = code.to_lowercase().replace('-', " ");
    let mut g = child(
        "guest",
        &[
            ("PH2D_NET_PORT", port.to_string()),
            ("PH2D_NET_CODE", typed),
        ],
    );
    let gl = lines(&mut g);
    let g_ok = wait(&mut g, end);
    let h_ok = wait(&mut h, end);
    let gout: Vec<String> = gl.try_iter().collect();
    let hout: Vec<String> = hl.try_iter().collect();
    assert!(
        g_ok && gout.iter().any(|l| l == "OK"),
        "convidado: {gout:?}"
    );
    assert!(
        h_ok && hout.iter().any(|l| l == "FIM"),
        "anfitrião: {hout:?}"
    );
}

/// O corpo dos filhos (só corre relançado, com `PH2D_NET_ROLE`). Cada marca abre linha: o `libtest`
/// escreve `test <nome> ... ` antes dela, sem quebra.
#[test]
#[ignore = "corpo dos processos filhos de um_anfitriao_e_um_convidado_em_processos_diferentes"]
fn papel_filho() {
    let t = Duration::from_secs(10);
    match std::env::var(ROLE).as_deref() {
        Ok("host") => {
            // Com `PH2D_NET_MDNS=<quadro>` a sala anuncia-se (a descoberta entre processos).
            let (h, hb) = match std::env::var("PH2D_NET_MDNS") {
                Ok(board) => host_mdns(&board),
                Err(_) => host(),
            };
            println!("\nPRONTO {} {}", h.port(), h.code());
            let ev = gather(
                || {
                    let ev = h.poll();
                    for e in &ev {
                        if let NetEvent::Frame(id, b) = e {
                            let n = String::from_utf8_lossy(b).replace("ping", "pong");
                            h.send(*id, n.into_bytes()).unwrap();
                        }
                    }
                    ev
                },
                &hb,
                t,
                |e| e.iter().any(|e| matches!(e, NetEvent::Left(_))),
            );
            let pings = ev
                .iter()
                .filter(|e| matches!(e, NetEvent::Frame(..)))
                .count();
            assert_eq!(pings, N, "{ev:?}");
            println!("\nFIM");
        }
        Ok("guest") => {
            let port: u16 = std::env::var("PH2D_NET_PORT").unwrap().parse().unwrap();
            let code = std::env::var("PH2D_NET_CODE").unwrap();
            let (w, gb) = bell();
            let g = Guest::connect(local(port), &code, t, w).expect("entra");
            for i in 0..N {
                g.send(format!("ping {i}").into_bytes()).unwrap();
            }
            let ev = gather(|| g.poll(), &gb, t, |e| e.len() >= N);
            let want: Vec<NetEvent> = (0..N)
                .map(|i| NetEvent::Frame(0, format!("pong {i}").into_bytes()))
                .collect();
            assert_eq!(ev, want);
            println!("\nOK");
        }
        _ => {}
    }
}
