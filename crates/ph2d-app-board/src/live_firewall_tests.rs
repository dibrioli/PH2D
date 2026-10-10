//! O botão da firewall na família, com o SISTEMA trocado por um falso (nenhum teste corre o
//! `pkexec`): o que se lê, o comando que se pede, a senha que espera numa thread, o que se lembra.

use std::sync::Mutex;

use super::super::{STATE, service};
use super::*;
use ph2d_a11y::NodeId;
use ph2d_editor_core::documents::live::{LiveRequest, LiveRole};

fn net(s: &str) -> Subnet {
    s.parse().expect("sub-rede")
}

/// O sistema falso: a firewall, a rede, e o que o «pkexec» devolve. Guarda os comandos pedidos.
struct Fake {
    probe: Probe,
    subnets: Vec<Subnet>,
    answer: (Option<i32>, String),
    asked: Mutex<Vec<String>>,
    /// Quando há, o «pkexec» espera por um sinal (a senha a ser escrita).
    gate: Mutex<Option<mpsc::Receiver<()>>>,
}

fn ufw_on() -> Probe {
    Probe {
        ufw_conf: Some("ENABLED=yes\n".into()),
        pkexec: true,
        ..Probe::default()
    }
}

fn install(fake: Arc<Fake>, dir: &Path) -> PathBuf {
    let file = dir.join(".ph2d").join("live_firewall.txt");
    let (p, s, r) = (fake.clone(), fake.clone(), fake);
    let env = Env {
        probe: Arc::new(move || p.probe.clone()),
        subnets: Arc::new(move || s.subnets.clone()),
        run: Arc::new(move |cmd| {
            r.asked.lock().unwrap().push(cmd.join(" "));
            let gate = r.gate.lock().unwrap().take();
            if let Some(g) = gate {
                let _ = g.recv();
            }
            r.answer.clone()
        }),
        file: Some(file.clone()),
    };
    super::super::reset_for_tests();
    STATE.with_borrow_mut(|st| st.fw.env = env);
    file
}

fn fake(probe: Probe, subnets: &[&str], answer: (Option<i32>, &str)) -> Arc<Fake> {
    Arc::new(Fake {
        probe,
        subnets: subnets.iter().map(|s| net(s)).collect(),
        answer: (answer.0, answer.1.to_owned()),
        asked: Mutex::new(Vec::new()),
        gate: Mutex::new(None),
    })
}

fn hero() -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author("Ana".into());
    h
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d-fw-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Serve até `done` (o que as threads devolvem chega em imagens seguintes).
fn until(h: &mut HeroScreen, mut done: impl FnMut(&HeroScreen) -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        service(h);
        if done(h) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    false
}

fn view(h: &HeroScreen) -> LiveFirewall {
    h.documents.live_view().firewall
}

fn told(h: &HeroScreen) -> String {
    h.documents.told().unwrap_or_default().to_owned()
}

/// ⭐ O painel aberto lê a firewall; «Permitir» pede a senha UMA vez com as regras da sub-rede, o
/// ecrã continua a andar enquanto ela não vem, lembra-se, e «Remover» desfaz exactamente o mesmo.
#[test]
fn allow_asks_the_password_off_the_screen_thread_remembers_and_remove_undoes_it() {
    let dir = tmp("allow");
    let f = fake(ufw_on(), &["192.168.15.0/24"], (Some(0), ""));
    let (open, gate) = mpsc::channel();
    *f.gate.lock().unwrap() = Some(gate);
    let file = install(f.clone(), &dir);
    let mut h = hero();
    h.documents.ask_live(LiveRequest::Discover);
    assert!(until(&mut h, |h| view(h).can_allow), "não leu a firewall");
    assert!(!view(&h).can_remove);

    h.documents.ask_live(LiveRequest::FirewallAllow);
    service(&mut h);
    assert!(view(&h).busy, "não mostrou que espera a senha");
    for _ in 0..20 {
        service(&mut h); // o ecrã anda: a senha espera noutra thread
    }
    assert!(view(&h).busy && !file.exists());
    open.send(()).unwrap();
    assert!(until(&mut h, |h| !view(h).busy), "a resposta não chegou");
    assert_eq!(told(&h), tr("board.live.fw_allowed"));
    assert_eq!(
        view(&h),
        LiveFirewall {
            can_allow: false,
            can_remove: true,
            busy: false
        }
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "ufw\n192.168.15.0/24\n"
    );
    let allow = fw::command(Firewall::Ufw, Change::Allow, &[net("192.168.15.0/24")])
        .unwrap()
        .join(" ");
    assert_eq!(*f.asked.lock().unwrap(), vec![allow]);

    h.documents.ask_live(LiveRequest::FirewallRemove);
    assert!(until(&mut h, |h| told(h) == tr("board.live.fw_removed")));
    assert!(!file.exists(), "esqueceu-se do que liberou?");
    assert!(view(&h).can_allow && !view(&h).can_remove);
    let remove = fw::command(Firewall::Ufw, Change::Remove, &[net("192.168.15.0/24")])
        .unwrap()
        .join(" ");
    assert_eq!(f.asked.lock().unwrap()[1], remove);
}

/// A senha recusada (126/127) e o erro do comando dizem-se no ecrã, e nada se lembra.
#[test]
fn a_refused_password_and_a_failure_are_said_and_nothing_is_remembered() {
    for (answer, want) in [
        ((Some(126), ""), tr("board.live.fw_cancelled").to_owned()),
        ((Some(127), ""), tr("board.live.fw_cancelled").to_owned()),
        (
            (Some(1), "ERROR: Could not load logging rules\n"),
            tr_with(
                "board.live.fw_failed",
                &[("error", &"ERROR: Could not load logging rules")],
            ),
        ),
    ] {
        let dir = tmp(&format!("refused{}", answer.0.unwrap_or(0)));
        let file = install(fake(ufw_on(), &["192.168.15.0/24"], answer), &dir);
        let mut h = hero();
        h.documents.ask_live(LiveRequest::Discover);
        assert!(until(&mut h, |h| view(h).can_allow));
        h.documents.ask_live(LiveRequest::FirewallAllow);
        assert!(
            until(&mut h, |h| told(h) == want),
            "{answer:?}: {}",
            told(&h)
        );
        assert!(!file.exists(), "{answer:?}");
        assert!(view(&h).can_allow && !view(&h).can_remove && !view(&h).busy);
    }
}

/// Outra rede (o portátil no trabalho): o botão volta, liberar SOMA, e o «Remover» continua.
#[test]
fn a_new_network_offers_the_button_again_and_allowing_adds_it() {
    let dir = tmp("two");
    let f = fake(ufw_on(), &["192.168.15.0/24"], (Some(0), ""));
    let file = install(f, &dir);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "ufw\n10.0.0.0/8\n").unwrap();
    let mut h = hero();
    h.documents.ask_live(LiveRequest::Discover);
    assert!(until(&mut h, |h| view(h).can_allow && view(h).can_remove));
    h.documents.ask_live(LiveRequest::FirewallAllow);
    assert!(until(&mut h, |h| told(h) == tr("board.live.fw_allowed")));
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "ufw\n10.0.0.0/8\n192.168.15.0/24\n"
    );
    assert!(!view(&h).can_allow && view(&h).can_remove);
}

/// Sem firewall não há botão; com uma à mão (nftables) também não — só a frase com a porta.
#[test]
fn without_a_supported_firewall_there_is_no_button() {
    for (probe, name) in [
        (Probe::default(), "off"),
        (
            Probe {
                other_active: true,
                pkexec: true,
                ..Probe::default()
            },
            "manual",
        ),
        (
            Probe {
                ufw_conf: Some("ENABLED=yes\n".into()),
                ..Probe::default()
            },
            "sem pkexec",
        ),
    ] {
        let dir = tmp(&format!("none-{}", name.replace(' ', "")));
        install(fake(probe, &["192.168.15.0/24"], (Some(0), "")), &dir);
        let mut h = hero();
        h.documents.ask_live(LiveRequest::Discover);
        for _ in 0..50 {
            service(&mut h);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(view(&h), LiveFirewall::default(), "{name}");
    }
}

/// A sala abre: com a firewall fechada diz-se onde está o botão; com uma à mão, a porta; sem
/// nenhuma, só o código.
#[test]
fn opening_the_room_says_how_to_open_the_firewall() {
    let cases = [
        (ufw_on(), "hint"),
        (
            Probe {
                other_active: true,
                ..Probe::default()
            },
            "manual",
        ),
        (Probe::default(), "off"),
    ];
    for (probe, name) in cases {
        let dir = tmp(&format!("host-{name}"));
        install(fake(probe, &["192.168.15.0/24"], (Some(0), "")), &dir);
        let mut h = hero();
        let id = h.documents.create("Retro".into());
        h.documents.activate(Some(id));
        h.documents.ask_live(LiveRequest::Host);
        service(&mut h);
        let LiveRole::Hosting { addrs, .. } = h.documents.live_view().role.clone() else {
            panic!("não abriu");
        };
        let port = addrs[0].rsplit(':').next().unwrap().to_owned();
        let opened = told(&h);
        assert!(opened.starts_with("Live: "), "{opened}");
        for _ in 0..50 {
            service(&mut h);
            std::thread::sleep(Duration::from_millis(2));
        }
        let want = match name {
            "hint" => tr("board.live.fw_hint").to_owned(),
            "manual" => tr_with("board.live.firewall", &[("port", &port)]),
            _ => opened,
        };
        assert_eq!(told(&h), want, "{name}");
        h.documents.ask_live(LiveRequest::Leave);
        service(&mut h);
    }
}
