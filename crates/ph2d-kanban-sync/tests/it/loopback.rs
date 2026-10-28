//! ⭐ **A prova em DOIS PROCESSOS** (plano §3.4): este binário de teste lança-se a si próprio como
//! convidado; os dois ligam-se por 127.0.0.1 — pela rede local (TCP + o código curto) e pela
//! internet (QUIC + o link com chave, sem ponto de encontro) —, mexem no MESMO Kanban ao mesmo
//! tempo, e os bytes de `KanbanSet::to_bytes` têm de sair iguais nos dois. O furo do NAT entre
//! máquinas reais não se prova aqui (`docs/TrelloClone/loop/BLOQUEIOS.md`).

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_internet::{DhtMode, InternetConfig, InternetHost};
use ph2d_board_net::{Announce, Guest, Host, HostConfig, Invite, Liveness, NetEvent, Waker};
use ph2d_board_sync::To;
use ph2d_kanban_model::{
    Card, CardId, Checklist, ChecklistId, FracKey, KanbanId, KanbanOp, KanbanSet, Label, LabelId,
    List, ListId, Member, Meta, Moment, Record,
};
use ph2d_kanban_sync::{KanbanGuest, KanbanHost, KanbanReplica, SoloMsg};

const CHILD: &str = "PH2D_KANBAN_SYNC_CHILD";
const EDITS: usize = 60;
const DEADLINE: Duration = Duration::from_secs(30);

fn waker() -> Waker {
    Arc::new(|| {})
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

fn put(set: &mut KanbanSet, r: Record) {
    let k = set.doc_mut(KanbanId(1)).expect("o Kanban da sessão");
    let _ = KanbanOp::put(r).apply(k);
}

fn scene() -> KanbanSet {
    let mut set = KanbanSet::default();
    let id = set.create("Ao vivo");
    let k = set.doc_mut(id).unwrap();
    for t in ["To Do", "Doing", "Done"] {
        let l = List {
            id: ListId(k.mint()),
            title: t.into(),
            pos: k.pos_after_lists(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        let _ = KanbanOp::put(Record::List(l)).apply(k);
    }
    let lists: Vec<ListId> = k.lists.keys().copied().collect();
    for i in 0..8 {
        let l = lists[i % 3];
        let c = Card::new(
            CardId(k.mint()),
            l,
            k.pos_after_cards(l),
            &format!("c{i}"),
            Moment(0),
        );
        let _ = KanbanOp::put(Record::card(c)).apply(k);
    }
    set
}

/// O gesto `i` de `who`: arrastar, renomear, votar, uma checklist — sobre os MESMOS cartões que o
/// outro processo está a mexer.
fn edit(set: &mut KanbanSet, r: &mut KanbanReplica, who: &str, i: usize) {
    let k = set.get(KanbanId(1)).unwrap();
    let cards: Vec<CardId> = k.cards.keys().copied().collect();
    let lists: Vec<ListId> = k.lists.keys().copied().collect();
    let id = cards[(i * 7 + who.len()) % cards.len()];
    let mut c = k.cards[&id].clone();
    match i % 4 {
        0 => {
            c.list = lists[i % lists.len()];
            c.pos = k.pos_after_cards(c.list);
        }
        1 => c.title = format!("{who} {i}"),
        2 => c.votes.push(Member(format!("{who}{i}"))),
        _ => {
            let cl = Checklist {
                id: ChecklistId(r.mint()),
                card: id,
                pos: FracKey::between(None, None),
                title: format!("{who} {i}"),
                hide_checked: false,
                meta: Meta::default(),
            };
            put(set, Record::Checklist(cl));
            return;
        }
    }
    put(set, Record::card(c));
}

fn done(set: &KanbanSet, who: &str) -> bool {
    set.get(KanbanId(1))
        .is_some_and(|k| k.labels.values().any(|l| l.name == format!("DONE {who}")))
}

fn mark_done(set: &mut KanbanSet, r: &mut KanbanReplica, who: &str) {
    let l = Label {
        id: LabelId(r.mint()),
        name: format!("DONE {who}"),
        color: None,
        meta: Meta::default(),
    };
    put(set, Record::Label(l));
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// O convidado — corre SÓ como processo filho (`PH2D_KANBAN_SYNC_CHILD=lan:<addr>:<código>` ou
/// `quic:<link>`); sem a variável não faz nada.
#[test]
fn child_process() {
    let Ok(how) = std::env::var(CHILD) else {
        return;
    };
    let net = if let Some(rest) = how.strip_prefix("lan:") {
        let (addr, code) = rest.rsplit_once(':').unwrap();
        Guest::connect(addr.parse().unwrap(), code, Duration::from_secs(5), waker()).unwrap()
    } else {
        let link = how.strip_prefix("quic:").unwrap();
        let invite = Invite::parse(link).unwrap();
        ph2d_board_internet::join(
            &invite,
            &offline(),
            Duration::from_secs(10),
            Liveness::default(),
            waker(),
        )
        .unwrap()
    };
    let mut set = KanbanSet::default();
    let mut g = KanbanGuest::new(2, "Bia".into(), 0xB1A, 0x5EC);
    net.send(g.hello().encode()).unwrap();
    let start = Instant::now();
    let (mut i, mut joined, mut quiet) = (0, false, None::<Instant>);
    loop {
        assert!(start.elapsed() < DEADLINE, "o convidado não acabou");
        let now = start.elapsed().as_millis() as u64;
        for e in net.poll() {
            if let NetEvent::Frame(_, b) = e {
                let (out, ev) = g.frame(&mut set, &b, now);
                joined |= ev
                    .iter()
                    .any(|e| matches!(e, ph2d_kanban_sync::KEvent::Arrived(_)));
                for m in out {
                    net.send(m.encode()).unwrap();
                }
            }
        }
        if joined {
            let r = g.room.replica.as_mut().unwrap();
            if i < EDITS {
                edit(&mut set, r, "guest", i);
                i += 1;
            } else if i == EDITS {
                mark_done(&mut set, r, "guest");
                i += 1;
            }
            for m in g.tick(&mut set, now) {
                net.send(m.encode()).unwrap();
            }
        }
        if done(&set, "guest") && done(&set, "host") {
            let q = *quiet.get_or_insert_with(Instant::now);
            if q.elapsed() > Duration::from_millis(600) {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    println!("STATE {}", hex(&set.to_bytes().unwrap()));
}

fn route(net: &Host, out: Vec<(To, SoloMsg)>) {
    for (to, m) in out {
        let b = m.encode();
        let _ = match to {
            To::One(c) => net.send(c, b),
            To::All => net.broadcast(b, None),
            To::AllBut(c) => net.broadcast(b, Some(c)),
        };
    }
}

/// O anfitrião neste processo; o convidado noutro, pelo caminho `how`.
fn two_processes(internet: bool) {
    let cfg = HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
        ..HostConfig::default()
    };
    let announce = Announce {
        board: "Kanban".into(),
        host: "Ana".into(),
    };
    let mut net = Host::open_with(&announce, &cfg, waker()).unwrap();
    // A entrada da internet vive até ao fim do teste.
    let (how, _door) = if internet {
        let ih = InternetHost::open(&mut net, &offline()).unwrap();
        (format!("quic:{}", ih.link()), Some(ih))
    } else {
        (format!("lan:127.0.0.1:{}:{}", net.port(), net.code()), None)
    };
    let mut set = scene();
    let mut host = KanbanHost::open(1, "Ana".into(), 0xA4A, &set, &[KanbanId(1)]);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "loopback::child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, &how)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    let (mut i, mut someone) = (0, false);
    while !(done(&set, "guest") && i > EDITS) {
        assert!(
            start.elapsed() < DEADLINE,
            "o anfitrião não viu o convidado acabar ({how})"
        );
        let now = start.elapsed().as_millis() as u64;
        for e in net.poll() {
            match e {
                NetEvent::Frame(c, b) => {
                    let (out, _) = host.frame(&mut set, c, &b, now);
                    someone = true;
                    route(&net, out);
                }
                NetEvent::Left(c) => route(&net, host.left(c)),
                _ => {}
            }
        }
        if someone {
            let r = &mut host.room.replica;
            if i < EDITS {
                edit(&mut set, r, "host", i);
            } else if i == EDITS {
                mark_done(&mut set, r, "host");
            }
            i += 1;
        }
        let out = host.tick(&mut set, now);
        route(&net, out);
        std::thread::sleep(Duration::from_millis(2));
    }
    // Continua a servir até o filho acabar (ele espera pelo meu «DONE» e sossega).
    let stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let lines: Vec<String> = BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
            .collect();
        lines
            .iter()
            .find_map(|l| l.split_once("STATE ").map(|(_, s)| s.trim().to_owned()))
            .ok_or(lines)
    });
    while child.try_wait().unwrap().is_none() {
        assert!(start.elapsed() < DEADLINE, "o convidado não saiu ({how})");
        let now = start.elapsed().as_millis() as u64;
        for e in net.poll() {
            if let NetEvent::Frame(c, b) = e {
                let (out, _) = host.frame(&mut set, c, &b, now);
                route(&net, out);
            }
        }
        route(&net, host.tick(&mut set, now));
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        child.wait().unwrap().success(),
        "o convidado falhou ({how})"
    );
    let theirs = reader.join().unwrap().expect("o convidado disse o estado");
    let mine = hex(&set.to_bytes().unwrap());
    assert_eq!(theirs, mine, "os dois processos divergiram ({how})");
    let k = set.get(KanbanId(1)).unwrap();
    assert!(k.checklists.values().any(|c| c.title.starts_with("guest")));
    assert!(k.checklists.values().any(|c| c.title.starts_with("host")));
}

#[test]
fn two_processes_converge_over_the_lan() {
    two_processes(false);
}

#[test]
fn two_processes_converge_over_quic() {
    two_processes(true);
}
