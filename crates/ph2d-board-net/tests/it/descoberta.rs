//! A descoberta por mDNS — depende da rede da máquina (avahi, systemd-resolved, firewall): corre-se à
//! mão e o resultado regista-se no ADR-0181 §Medições:
//! `cargo test -p ph2d-board-net --test it descoberta -- --ignored --nocapture --test-threads=1`

use std::time::{Duration, Instant};

use ph2d_board_net::{Discovery, Guest, NetEvent};

use crate::dois_processos::{child, lines, ready, wait};
use crate::{bell, gather, host_mdns, loadavg};

const RUNS: usize = 5;
const WAIT: Duration = Duration::from_secs(10);

fn unique(tag: &str) -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{tag} {:x}", t.as_nanos())
}

fn summary(what: &str, mut ms: Vec<f64>) {
    ms.sort_by(f64::total_cmp);
    println!(
        "{what}: mínimo {:.0} ms · mediana {:.0} ms · máximo {:.0} ms · n {} · loadavg {}",
        ms[0],
        ms[ms.len() / 2],
        ms[ms.len() - 1],
        ms.len(),
        loadavg()
    );
}

/// Descobre-se e entra pelo endereço descoberto, no mesmo processo.
#[test]
#[ignore = "depende da rede da máquina; à mão"]
fn descobre_se_no_mesmo_processo() {
    let mut found_ms = Vec::new();
    let mut gone_ms = Vec::new();
    for run in 0..RUNS {
        let board = unique(&format!("Quadro ç {run}"));
        let (h, hb) = host_mdns(&board);
        let t = Instant::now();
        let mut d = Discovery::start().unwrap();
        let list = d.wait(t + WAIT, |l| l.iter().any(|f| f.board == board));
        let f = list
            .iter()
            .find(|f| f.board == board)
            .unwrap_or_else(|| panic!("não se viu: {list:?}"));
        found_ms.push(t.elapsed().as_secs_f64() * 1e3);
        assert_eq!(f.addr.port(), h.port());
        assert_eq!(f.host, "Teste");

        let (w, _gb) = bell();
        let g = Guest::connect(f.addr, &h.code(), WAIT, w).expect("entra pelo endereço descoberto");
        let ev = gather(
            || h.poll(),
            &hb,
            WAIT,
            |e| e.iter().any(|e| matches!(e, NetEvent::Joined(..))),
        );
        assert!(ev.iter().any(|e| matches!(e, NetEvent::Joined(..))));
        drop(g);

        // Fechar a sala tira-a da lista de quem procura (o adeus do mDNS).
        drop(h);
        let t = Instant::now();
        let list = d.wait(t + WAIT, |l| !l.iter().any(|f| f.board == board));
        assert!(
            !list.iter().any(|f| f.board == board),
            "ainda na lista: {list:?}"
        );
        gone_ms.push(t.elapsed().as_secs_f64() * 1e3);
    }
    summary("mesmo processo, até aparecer", found_ms);
    summary(
        "mesmo processo, até sair da lista depois de fechar",
        gone_ms,
    );
}

/// Um processo anuncia, o outro procura, entra e troca um quadro.
#[test]
#[ignore = "depende da rede da máquina; à mão"]
fn descobre_se_entre_dois_processos() {
    let mut found_ms = Vec::new();
    for run in 0..RUNS {
        let board = unique(&format!("Outro processo {run}"));
        let end = Instant::now() + Duration::from_secs(30);
        let mut h = child("host", &[("PH2D_NET_MDNS", board.clone())]);
        let hl = lines(&mut h);
        let (_, code) = ready(&hl, WAIT).expect("o anfitrião não ficou pronto");
        let t = Instant::now();
        let mut d = Discovery::start().unwrap();
        let list = d.wait(t + WAIT, |l| l.iter().any(|f| f.board == board));
        let Some(f) = list.iter().find(|f| f.board == board).cloned() else {
            let _ = h.kill();
            let _ = h.wait();
            panic!("não se viu: {list:?}");
        };
        found_ms.push(t.elapsed().as_secs_f64() * 1e3);

        let (w, gb) = bell();
        let g = Guest::connect(f.addr, &code, WAIT, w).expect("entra");
        g.send(b"ping 0".to_vec()).unwrap();
        let ev = gather(|| g.poll(), &gb, WAIT, |e| !e.is_empty());
        assert_eq!(ev, vec![NetEvent::Frame(0, b"pong 0".to_vec())]);
        // O anfitrião-filho espera 10 pings; manda os restantes para ele acabar com `FIM`.
        for i in 1..10 {
            g.send(format!("ping {i}").into_bytes()).unwrap();
        }
        let _ = gather(|| g.poll(), &gb, WAIT, |e| e.len() >= 9);
        drop(g);
        assert!(wait(&mut h, end), "o anfitrião-filho não acabou bem");
    }
    summary("dois processos, até aparecer", found_ms);
}

/// O caso real: a sala já está aberta (o anúncio assentou) quando alguém começa a procurar.
#[test]
#[ignore = "depende da rede da máquina; à mão"]
fn descobre_uma_sala_ja_anunciada() {
    let board = unique("Já aberta");
    let (_h, _hb) = host_mdns(&board);
    std::thread::sleep(Duration::from_secs(3));
    let mut ms = Vec::new();
    for _ in 0..RUNS {
        let t = Instant::now();
        let mut d = Discovery::start().unwrap();
        let list = d.wait(t + WAIT, |l| l.iter().any(|f| f.board == board));
        assert!(list.iter().any(|f| f.board == board), "{list:?}");
        ms.push(t.elapsed().as_secs_f64() * 1e3);
    }
    summary("sala já anunciada, procura nova até aparecer", ms);
}

/// Coexistência com o avahi desta máquina, nos dois sentidos: o `avahi-browse` vê a nossa sala, e a
/// nossa procura vê um serviço que o `avahi-publish-service` anuncia.
#[test]
#[ignore = "depende do avahi da máquina; à mão"]
fn coexiste_com_o_avahi() {
    use std::process::{Command, Stdio};
    let board = unique("Visto pelo avahi");
    let (h, _hb) = host_mdns(&board);
    std::thread::sleep(Duration::from_secs(2));
    let out = Command::new("timeout")
        .args(["5", "avahi-browse", "-rpt", "_ph2d-board._tcp"])
        .output()
        .expect("avahi-browse");
    let text = String::from_utf8_lossy(&out.stdout);
    let seen: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with('=') && l.contains(&board))
        .collect();
    println!("avahi-browse vê a sala: {} linha(s) resolvidas", seen.len());
    for l in &seen {
        println!("  {l}");
    }
    assert!(!seen.is_empty(), "{text}");
    assert!(!text.contains(&h.code()), "o código nunca vai para a rede");

    let other = unique("Anunciado pelo avahi");
    let mut p = Command::new("avahi-publish-service")
        .args([
            other.as_str(),
            "_ph2d-board._tcp",
            "47999",
            &format!("board={other}"),
            "host=avahi",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("avahi-publish-service");
    let t = Instant::now();
    let mut d = Discovery::start().unwrap();
    let list = d.wait(t + WAIT, |l| l.iter().any(|f| f.board == other));
    let _ = p.kill();
    let _ = p.wait();
    let f = list.iter().find(|f| f.board == other);
    println!(
        "a nossa procura vê o avahi em {:.0} ms: {f:?}",
        t.elapsed().as_secs_f64() * 1e3
    );
    assert!(
        f.is_some_and(|f| f.host == "avahi" && f.addr.port() == 47_999),
        "{list:?}"
    );
}
