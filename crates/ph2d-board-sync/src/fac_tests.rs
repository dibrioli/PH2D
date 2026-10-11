//! A facilitação ao vivo (ADR-0182) numa sala de mentira: o anfitrião (Ana) e convidados, cada
//! mensagem entregue GUARDADA por quem a recebeu — o gate de FUGA procura o texto selado nesses bytes.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ph2d_board_model::{
    Board, BoardOp, BoardSet, Element, ElementId, Rgba, RichText, Shape, ShapeType, Style,
    VoteSetup,
};

use crate::{Ask, Ballot, ConnId, Event, Fac, Guest, Host, Msg, REVEAL_COUNTDOWN_MS, Refusal, To};

const ANA: u64 = 1;

fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    set.get_mut(id).unwrap().clone()
}

fn note(b: &mut Board, text: &str, x: f64) -> ElementId {
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(ink), None, ink),
        text: RichText::plain(text),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [x, 0.0, 10.0, 10.0],
    );
    let id = el.id;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    id
}

fn write(b: &mut Board, id: ElementId, text: &str) {
    let mut el = b.doc.get(id).unwrap().clone();
    el.shape_mut().unwrap().text = RichText::plain(text);
    let _ = BoardOp::Put(el).apply(&mut b.doc);
}

fn text(b: &Board, id: ElementId) -> String {
    b.doc
        .get_any(id)
        .unwrap()
        .shape()
        .unwrap()
        .text
        .as_str()
        .to_owned()
}

fn shared(b: &Board) -> Vec<u8> {
    let els: Vec<&Element> = b.doc.live_with_hidden().collect();
    postcard::to_allocvec(&(els, &b.name, b.sketch)).unwrap()
}

fn has(bytes: &[u8], needle: &str) -> bool {
    bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
}

struct Peer {
    g: Guest,
    b: Option<Board>,
    conn: ConnId,
    online: bool,
    up: VecDeque<Vec<u8>>,
    down: VecDeque<Vec<u8>>,
    /// Tudo o que esta pessoa recebeu.
    heard: Vec<Vec<u8>>,
    events: Vec<Event>,
}

struct Room {
    host: Host,
    hb: Board,
    host_events: Vec<Event>,
    peers: Vec<Peer>,
    next_conn: ConnId,
    now: u64,
}

impl Room {
    fn new(hb: Board, names: &[&str]) -> Self {
        let mut hb = hb;
        let host = Host::open(ANA, "Ana".into(), &mut hb, 1);
        let mut r = Self {
            host,
            hb,
            host_events: Vec::new(),
            peers: Vec::new(),
            next_conn: 100,
            now: 0,
        };
        for n in names {
            r.join(n);
        }
        r
    }

    fn join(&mut self, name: &str) -> usize {
        let i = self.peers.len();
        let peer = 10 + i as u64;
        let g = Guest::new(peer, name.into(), 50 + peer);
        let hello = g.hello().encode();
        self.peers.push(Peer {
            g,
            b: None,
            conn: self.next_conn,
            online: true,
            up: VecDeque::from([hello]),
            down: VecDeque::new(),
            heard: Vec::new(),
            events: Vec::new(),
        });
        self.next_conn += 1;
        self.pump();
        i
    }

    fn route(&mut self, out: Vec<(To, Msg)>) {
        for (to, m) in out {
            let bytes = m.encode();
            for p in self.peers.iter_mut().filter(|p| p.online) {
                let hit = match to {
                    To::All => true,
                    To::One(c) => c == p.conn,
                    To::AllBut(c) => c != p.conn,
                };
                if hit {
                    p.down.push_back(bytes.clone());
                }
            }
        }
    }

    /// Entrega tudo até a sala ficar quieta (com um tique de cada um a cada volta).
    fn pump(&mut self) {
        for _ in 0..50 {
            let mut moved = false;
            for i in 0..self.peers.len() {
                while self.peers[i].online
                    && let Some(bytes) = self.peers[i].up.pop_front()
                {
                    moved = true;
                    let conn = self.peers[i].conn;
                    let (out, ev) = self.host.frame(&mut self.hb, conn, &bytes, self.now);
                    self.host_events.extend(ev);
                    self.route(out);
                }
            }
            let out = self.host.tick(&mut self.hb, self.now);
            moved |= !out.is_empty();
            self.route(out);
            for p in self.peers.iter_mut().filter(|p| p.online) {
                while let Some(bytes) = p.down.pop_front() {
                    moved = true;
                    p.heard.push(bytes.clone());
                    let (out, ev) = p.g.frame(p.b.as_mut(), &bytes, self.now);
                    for e in ev {
                        match e {
                            Event::Arrived(b) => p.b = Some(*b),
                            e => p.events.push(e),
                        }
                    }
                    p.up.extend(out.into_iter().map(|m| m.encode()));
                }
                if let Some(b) = &mut p.b {
                    let out = p.g.tick(b, self.now);
                    moved |= !out.is_empty();
                    p.up.extend(out.into_iter().map(|m| m.encode()));
                }
            }
            if !moved {
                return;
            }
        }
        panic!("a sala não sossega");
    }

    fn at(&mut self, ms: u64) {
        self.now = ms;
        self.pump();
    }

    fn ask(&mut self, i: usize, a: Ask) {
        let m = self.peers[i].g.ask(a).encode();
        self.peers[i].up.push_back(m);
        self.pump();
    }

    fn host_ask(&mut self, a: Ask) {
        if let Some(e) = self.host.ask(&mut self.hb, a, self.now) {
            self.host_events.push(e);
        }
        self.pump();
    }

    fn vote(&mut self, i: usize, b: Ballot) {
        let m = self.peers[i].g.ballot(b).encode();
        self.peers[i].up.push_back(m);
        self.pump();
    }

    fn b(&mut self, i: usize) -> &mut Board {
        self.peers[i].b.as_mut().unwrap()
    }

    fn drop(&mut self, i: usize) {
        let p = &mut self.peers[i];
        p.online = false;
        p.up.clear();
        p.down.clear();
        let out = self.host.left(p.conn);
        self.route(out);
        self.pump();
    }

    fn back(&mut self, i: usize) {
        let conn = self.next_conn;
        self.next_conn += 1;
        let p = &mut self.peers[i];
        p.online = true;
        p.conn = conn;
        p.up.push_back(p.g.hello().encode());
        self.pump();
    }

    fn heard(&self, i: usize, needle: &str) -> bool {
        self.peers[i].heard.iter().any(|m| has(m, needle))
    }

    fn all_same(&self) {
        for (i, p) in self.peers.iter().enumerate() {
            assert!(
                shared(p.b.as_ref().unwrap()) == shared(&self.hb),
                "o convidado {i} divergiu"
            );
        }
    }

    fn fac(&self, i: usize) -> Fac {
        self.peers[i].g.fac(self.now)
    }
}

fn private_on() -> Ask {
    Ask::PrivateOn {
        anonymous: false,
        anyone_can_end: false,
    }
}

/// ⭐⭐ O gate de FUGA: com o modo privado ligado, o texto de uma nota nova e o de uma nota editada
/// pela Bia não chegam em NENHUM byte ao Caio nem ao quadro da Ana (o anfitrião guarda-o no cofre,
/// fora do quadro), nem ao Dani que entra a meio; os três vêem as duas notas escondidas. Revelar:
/// nada antes dos 5 s, tudo depois — e os quatro quadros iguais.
#[test]
fn a_private_note_reaches_nobody_until_the_reveal() {
    let mut hb = board();
    let old = note(&mut hb, "antes", 0.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(0, private_on());
    assert!(
        r.fac(1).private.is_some(),
        "o Caio sabe que o modo está ligado"
    );
    let new = note(r.b(0), "segredo da Bia", 20.0);
    write(r.b(0), old, "outro segredo");
    r.at(10);
    for needle in ["segredo da Bia", "outro segredo"] {
        assert!(!r.heard(1, needle), "o Caio ouviu «{needle}»");
        assert!(
            !has(&shared(&r.hb), needle),
            "o quadro da Ana tem «{needle}»"
        );
    }
    assert_eq!(text(&r.hb, new), "", "nasce vazia para os outros");
    assert_eq!(
        text(r.b(1), old),
        "antes",
        "o Caio fica com o texto de antes"
    );
    assert_eq!(text(r.b(0), new), "segredo da Bia", "a autora lê o seu");
    assert_eq!(r.fac(1).hidden, BTreeSet::from([old, new]));
    assert_eq!(r.host.fac(10, &r.hb).hidden, BTreeSet::from([old, new]));
    assert_eq!(r.peers[0].g.my_sealed(), BTreeSet::from([old, new]));
    let d = r.join("Dani");
    assert!(
        !r.heard(d, "segredo"),
        "quem entra a meio não recebe o selado"
    );
    assert_eq!(r.fac(d).hidden.len(), 2);
    // Só a Bia (quem ligou) desliga; a contagem é do relógio do anfitrião.
    r.ask(0, Ask::PrivateOff);
    assert_eq!(
        r.fac(1).private.unwrap().reveal_in_ms,
        Some(REVEAL_COUNTDOWN_MS)
    );
    r.at(10 + REVEAL_COUNTDOWN_MS - 1);
    assert!(!r.heard(1, "segredo da Bia"), "antes dos 5 s");
    r.at(10 + REVEAL_COUNTDOWN_MS);
    assert!(r.fac(1).private.is_none() && r.fac(1).hidden.is_empty());
    assert_eq!(text(r.b(1), new), "segredo da Bia");
    assert_eq!(text(&r.hb, old), "outro segredo");
    assert_eq!(text(r.b(d), old), "outro segredo");
    assert!(r.peers[0].g.my_sealed().is_empty());
    r.all_same();
}

/// O anfitrião também escreve em privado: o quadro DELE é a fonte do retrato, e o retrato tapa-lhe o
/// texto com o valor E o carimbo públicos — quem entra a meio recebe-o vazio, e a escrita solta
/// entra depois (sem o carimbo público, empatava e ficava de fora).
#[test]
fn the_hosts_own_private_note_is_masked_in_the_snapshot_and_lands_after_the_reveal() {
    let mut r = Room::new(board(), &["Bia"]);
    r.host_ask(private_on());
    let id = note(&mut r.hb, "da Ana", 0.0);
    r.at(5);
    assert!(!r.heard(0, "da Ana"));
    let c = r.join("Caio");
    assert!(!r.heard(c, "da Ana"), "o retrato tapa o texto da Ana");
    assert_eq!(text(r.b(c), id), "");
    assert_eq!(text(&r.hb, id), "da Ana", "a Ana lê o seu");
    assert_eq!(r.host.my_sealed(), BTreeSet::from([id]));
    r.host_ask(Ask::PrivateOff);
    r.at(5 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(r.b(c), id), "da Ana");
    assert_eq!(text(r.b(0), id), "da Ana");
    r.all_same();
}

/// «Reveal all my sticky notes»: só com a opção (de quem ligou), e solta SÓ as do autor.
#[test]
fn reveal_mine_needs_the_option_and_releases_only_the_authors_notes() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let bia = note(r.b(0), "nota da Bia", 0.0);
    let caio = note(r.b(1), "nota do Caio", 20.0);
    r.at(1);
    r.ask(1, Ask::RevealMine);
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::RevealMineOff))
    );
    r.ask(1, Ask::AnyoneCanReveal(true));
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::OnlyStarterEndsPrivate)),
        "a opção é de quem ligou"
    );
    r.ask(0, Ask::AnyoneCanReveal(true));
    r.ask(1, Ask::RevealMine);
    assert_eq!(text(r.b(0), caio), "nota do Caio");
    assert!(!r.heard(1, "nota da Bia"), "a da Bia continua selada");
    assert_eq!(r.fac(1).hidden, BTreeSet::from([bia]));
    assert!(r.fac(0).private.is_some(), "o modo continua ligado");
}

/// Quem desliga: quem ligou, o anfitrião (o dono da sala), ou qualquer um com a opção.
#[test]
fn only_the_starter_or_the_host_turns_it_off_unless_anyone_may() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    r.ask(1, Ask::PrivateOff);
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::OnlyStarterEndsPrivate))
    );
    assert_eq!(r.fac(1).private.unwrap().reveal_in_ms, None);
    r.host_ask(Ask::PrivateOff);
    assert!(
        r.fac(1).private.unwrap().reveal_in_ms.is_some(),
        "o anfitrião pode"
    );
    r.at(REVEAL_COUNTDOWN_MS);
    r.ask(
        0,
        Ask::PrivateOn {
            anonymous: true,
            anyone_can_end: true,
        },
    );
    r.ask(1, Ask::PrivateOff);
    assert!(
        r.fac(1).private.unwrap().reveal_in_ms.is_some(),
        "com a opção, qualquer um"
    );
}

/// Só o comando de revelar solta (o p07 do Scrumlr: tirar um destaque revelava tudo): mover, pintar e
/// apagar a nota escondida de outro, votar, sair e voltar — o texto continua sem chegar ao Caio.
#[test]
fn nothing_but_the_reveal_releases_the_vault() {
    let mut hb = board();
    let other = note(&mut hb, "vizinha", 40.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "escondido", 0.0);
    r.at(1);
    let mut el = r.b(1).doc.get(id).unwrap().clone();
    el.translate([5.0, 5.0]);
    let _ = BoardOp::Put(el).apply(&mut r.b(1).doc);
    let mut el = r.hb.doc.get(id).unwrap().clone();
    el.shape_mut().unwrap().style.fill = Some(Rgba([200, 0, 0, 255]));
    let _ = BoardOp::Put(el).apply(&mut r.hb.doc);
    r.at(2);
    r.ask(
        1,
        Ask::VoteStart {
            setup: VoteSetup::default(),
            targets: BTreeSet::from([id, other]),
        },
    );
    r.vote(
        1,
        Ballot {
            round: 1,
            votes: BTreeMap::from([(other, 1)]),
            done: true,
        },
    );
    r.ask(1, Ask::VoteEnd);
    r.drop(0);
    r.back(0);
    let mut el = r.b(1).doc.get(id).unwrap().clone();
    el.deleted = true;
    let _ = BoardOp::Put(el).apply(&mut r.b(1).doc);
    r.at(3);
    assert!(!r.heard(1, "escondido"));
    assert!(!has(&shared(&r.hb), "escondido"));
    assert_eq!(text(r.b(0), id), "escondido", "a Bia continua a ler o seu");
}

/// Cair e voltar: o que ia selado volta ao COFRE (nunca a todos), também o que a Bia escreveu com a
/// ligação em baixo; e a revelação entrega-o.
#[test]
fn a_drop_sends_the_sealed_text_back_to_the_vault() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "primeiro", 0.0);
    r.at(1);
    r.drop(0);
    write(r.b(0), id, "escrito sem rede");
    r.back(0);
    r.at(2);
    assert!(!r.heard(1, "primeiro") && !r.heard(1, "escrito sem rede"));
    assert_eq!(
        text(r.b(0), id),
        "escrito sem rede",
        "a autora volta com o seu"
    );
    assert_eq!(r.fac(1).hidden, BTreeSet::from([id]));
    r.ask(0, Ask::PrivateOff);
    r.at(2 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(r.b(1), id), "escrito sem rede");
    r.all_same();
}

/// Fechar a sala com o modo ligado REVELA antes do adeus (o trabalho de ninguém se perde).
#[test]
fn closing_the_room_reveals_first() {
    let mut r = Room::new(board(), &["Bia"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "não se perde", 0.0);
    r.at(1);
    let out = r.host.close(&mut r.hb, 2);
    r.route(out);
    r.pump();
    assert_eq!(text(&r.hb, id), "não se perde");
    assert_eq!(text(r.b(0), id), "não se perde");
}

/// Ligar o modo outra vez esconde só o NOVO (Miro; não o p04 do Scrumlr): o que já se revelou fica.
#[test]
fn turning_it_on_again_hides_only_what_is_new() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "revelada", 0.0);
    r.at(1);
    r.ask(0, Ask::PrivateOff);
    r.at(1 + REVEAL_COUNTDOWN_MS);
    r.ask(0, private_on());
    assert!(r.fac(1).hidden.is_empty());
    assert_eq!(text(r.b(1), id), "revelada");
    write(r.b(0), id, "nova versão");
    r.at(2 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(r.b(1), id), "revelada", "a edição nova esconde-se");
    assert!(!r.heard(1, "nova versão"));
    assert_eq!(r.fac(1).hidden, BTreeSet::from([id]));
}

fn ballot(round: u32, votes: &[(ElementId, u32)], done: bool) -> Ballot {
    Ballot {
        round,
        votes: votes.iter().copied().collect(),
        done,
    }
}

/// ⭐⭐ O voto entre pessoas: ninguém ouve as escolhas de ninguém (nenhuma mensagem com o boletim de
/// outro, e o «quantos votos usou» só a quem iniciou); quem acabou é público; quem entra a meio
/// vota; o boletim de quem SAI conta; o total chega no fim, igual em todos, e é só o total.
#[test]
fn no_one_hears_a_vote_before_the_end_and_a_leavers_vote_counts() {
    let mut hb = board();
    let a = note(&mut hb, "a", 0.0);
    let b = note(&mut hb, "b", 20.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup {
                votes: 3,
                one_per_object: false,
                minutes: 0,
            },
            targets: BTreeSet::from([a, b]),
        },
    );
    let round = r.fac(1).vote.unwrap().round;
    let heard_before = r.peers[1].heard.len();
    r.vote(0, ballot(round, &[(a, 2), (b, 1)], true));
    for m in &r.peers[1].heard[heard_before..] {
        assert!(
            !matches!(Msg::decode(m), Some(Msg::Ballot(_))),
            "o Caio recebeu um boletim"
        );
    }
    let caio = r.fac(1).vote.unwrap();
    assert!(caio.used.is_empty(), "só quem iniciou vê os votos usados");
    assert_eq!(caio.done, BTreeSet::from([10]), "quem acabou é público");
    assert_eq!(r.fac(0).vote.unwrap().used, BTreeMap::from([(10, 3)]));
    assert_eq!(
        r.hb.doc.get(a).unwrap().votes(),
        0,
        "nada no quadro até ao fim"
    );
    // O Caio vota e sai; o Dani entra a meio e vota.
    r.vote(1, ballot(round, &[(b, 1)], false));
    r.drop(1);
    let d = r.join("Dani");
    assert_eq!(r.fac(d).vote.unwrap().round, round);
    r.vote(d, ballot(round, &[(a, 1)], true));
    r.host
        .ballot(ballot(round, &[(b, 3)], true))
        .then_some(())
        .expect("a Ana também vota");
    r.pump();
    // Só quem iniciou (ou o anfitrião) acaba.
    r.ask(d, Ask::VoteEnd);
    assert!(
        r.peers[d]
            .events
            .contains(&Event::Denied(Refusal::OnlyStarterEndsVote))
    );
    r.ask(0, Ask::VoteEnd);
    assert_eq!(r.hb.doc.get(a).unwrap().votes(), 3);
    assert_eq!(
        r.hb.doc.get(b).unwrap().votes(),
        5,
        "o voto de quem saiu conta"
    );
    assert!(r.fac(d).vote.unwrap().ended);
    assert!(r.fac(d).vote.unwrap().used.is_empty());
    assert_eq!(r.b(d).doc.get(b).unwrap().votes(), 5);
}

/// O relógio do anfitrião acaba a votação (Miro; o do Scrumlr não acabava nada, v05).
#[test]
fn the_hosts_clock_ends_the_vote() {
    let mut hb = board();
    let a = note(&mut hb, "a", 0.0);
    let mut r = Room::new(hb, &["Bia"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup {
                votes: 1,
                one_per_object: false,
                minutes: 1,
            },
            targets: BTreeSet::from([a]),
        },
    );
    assert_eq!(r.fac(0).vote.unwrap().left_ms, Some(60_000));
    r.vote(0, ballot(1, &[(a, 1)], false));
    r.at(30_000);
    assert_eq!(r.fac(0).vote.unwrap().left_ms, Some(30_000));
    r.at(59_999);
    assert_eq!(r.b(0).doc.get(a).unwrap().votes(), 0);
    r.at(60_000);
    assert_eq!(r.b(0).doc.get(a).unwrap().votes(), 1);
    assert!(r.fac(0).vote.unwrap().ended);
}

/// O cofre só aceita um boletim que cumpre as regras e é DESTA rodada; uma nota escondida não é
/// votável por quem não a lê; com uma votação a correr não se abre outra.
#[test]
fn the_vault_refuses_a_ballot_out_of_the_rules() {
    let mut hb = board();
    let a = note(&mut hb, "a", 0.0);
    let b = note(&mut hb, "b", 20.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(0, private_on());
    let hidden = note(r.b(0), "da Bia", 40.0);
    r.at(1);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup {
                votes: 2,
                one_per_object: true,
                minutes: 0,
            },
            targets: BTreeSet::from([a, b, hidden]),
        },
    );
    r.ask(
        1,
        Ask::VoteStart {
            setup: VoteSetup::default(),
            targets: BTreeSet::from([a]),
        },
    );
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::VoteRunning))
    );
    let used = |r: &Room| r.fac(0).vote.unwrap().used.get(&11).copied();
    r.vote(1, ballot(1, &[(a, 1), (b, 1), (hidden, 0)], false));
    assert_eq!(used(&r), Some(2));
    for bad in [
        ballot(1, &[(a, 2)], false),
        ballot(1, &[(a, 1), (b, 1), (hidden, 1)], false),
        ballot(1, &[(hidden, 1)], false),
        ballot(2, &[(a, 1)], false),
    ] {
        r.vote(1, bad.clone());
        assert_eq!(used(&r), Some(2), "aceitou {bad:?}");
    }
    r.vote(0, ballot(1, &[(hidden, 1)], false));
    assert_eq!(
        r.fac(0).vote.unwrap().used.get(&10),
        Some(&1),
        "a autora lê a sua"
    );
}

/// ⭐ Convergência com o modo privado: anfitrião + 3, edições de texto e de posição ao acaso com o
/// modo ligado, quedas e voltas — e nenhum texto selado de outro em nada que cada um ouviu; depois
/// da revelação, os quatro com o mesmo quadro.
#[test]
fn a_private_room_with_drops_ends_with_one_board_and_no_early_text() {
    for seed in 1..=12u64 {
        let mut s = seed * 0x9e37_79b9 + 7;
        let mut rnd = |n: u64| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s % n
        };
        let mut hb = board();
        let ids: Vec<ElementId> = (0..4)
            .map(|i| note(&mut hb, "", f64::from(i) * 20.0))
            .collect();
        let mut r = Room::new(hb, &["p0", "p1", "p2"]);
        r.ask(0, private_on());
        let mut written: Vec<(usize, String)> = Vec::new();
        for step in 0..120u64 {
            let who = rnd(4) as usize;
            let id = ids[rnd(4) as usize];
            let secret = format!("s{seed}x{step}w{who}");
            match rnd(3) {
                0 => {
                    if who == 3 {
                        write(&mut r.hb, id, &secret);
                    } else if r.peers[who].online {
                        write(r.b(who), id, &secret);
                    } else {
                        continue;
                    }
                    written.push((who, secret));
                }
                1 if who < 3 => {
                    if r.peers[who].online {
                        r.drop(who);
                    } else {
                        r.back(who);
                    }
                }
                _ => {
                    let b = if who == 3 || !r.peers[who].online {
                        &mut r.hb
                    } else {
                        r.b(who)
                    };
                    let mut el = b.doc.get(id).unwrap().clone();
                    el.translate([1.0, 0.0]);
                    let _ = BoardOp::Put(el).apply(&mut b.doc);
                }
            }
            r.at(step);
        }
        for i in 0..3 {
            if !r.peers[i].online {
                r.back(i);
            }
        }
        r.at(200);
        for (author, secret) in &written {
            for i in (0..3).filter(|i| i != author) {
                assert!(!r.heard(i, secret), "semente {seed}: {i} ouviu {secret}");
            }
        }
        r.ask(0, Ask::PrivateOff);
        r.at(200 + REVEAL_COUNTDOWN_MS);
        r.all_same();
        assert!(r.fac(0).hidden.is_empty());
    }
}

/// ⛔ Auditoria W12 (rede, F1): uma VERSÃO guardada com o modo ligado levava o quadro inteiro da autora
/// — comprimido, e por isso o gate de fuga (bytes crus) não a via. Leva o quadro como todos o vêem.
#[test]
fn a_version_saved_in_private_mode_does_not_carry_the_sealed_text() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "segredo na versão", 0.0);
    r.at(1);
    r.b(0).save_version("v".into(), 0);
    r.at(2);
    let v = r.b(1).team.versions[0].read().unwrap();
    assert!(
        !has(&shared(&v), "segredo na versão"),
        "a versão levou o selado"
    );
    let own = r.b(0).team.versions[0].read().unwrap();
    assert_eq!(
        text(&own, id),
        "",
        "a da autora é a mesma (a versão é do quadro da sala)"
    );
    let c = r.join("Dani");
    let v = r.b(c).team.versions[0].read().unwrap();
    assert!(!has(&shared(&v), "segredo na versão"));
}

/// ⛔ Auditoria W12 (rede, F2): a nota CRESCIA para os outros enquanto a autora escrevia (o comprimento
/// do texto). A caixa da minha nota escondida sai com a posição nova e o tamanho de antes; o tamanho
/// verdadeiro chega ao revelar.
#[test]
fn a_hidden_note_does_not_grow_for_the_others_but_still_moves() {
    let mut r = Room::new(board(), &["Bia", "Caio"]);
    r.ask(0, private_on());
    let id = note(r.b(0), "uma linha", 0.0);
    r.at(1);
    let mut el = r.b(0).doc.get(id).unwrap().clone();
    el.shape_mut().unwrap().text = RichText::plain("muitas\nlinhas\nde\nsegredo");
    el.h = 40.0;
    let _ = BoardOp::Put(el).apply(&mut r.b(0).doc);
    r.at(2);
    let seen = r.b(1).doc.get(id).unwrap().clone();
    assert_eq!(seen.h, 10.0, "a nota cresceu para o Caio");
    let mut el = r.b(0).doc.get(id).unwrap().clone();
    el.translate([30.0, 0.0]);
    let _ = BoardOp::Put(el).apply(&mut r.b(0).doc);
    r.at(3);
    let seen = r.b(1).doc.get(id).unwrap().clone();
    assert_eq!((seen.x, seen.h), (30.0, 10.0), "mexeu-se, sem crescer");
    let c = r.join("Dani");
    assert_eq!(r.b(c).doc.get(id).unwrap().h, 10.0, "o retrato também");
    r.ask(0, Ask::PrivateOff);
    r.at(3 + REVEAL_COUNTDOWN_MS);
    assert_eq!(r.b(1).doc.get(id).unwrap().h, 40.0);
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F3; interface, F1): quem NÃO lê uma nota escondida não lhe escreve o texto
/// por caminho nenhum (o desfazer, o negrito — aqui uma escrita directa no documento): a escrita
/// desfaz-se ao ser capturada, nunca vai ao cofre, e o texto da autora sai ao revelar.
#[test]
fn someone_who_cannot_read_a_note_cannot_overwrite_its_text() {
    let mut hb = board();
    let id = note(&mut hb, "", 0.0);
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    write(r.b(1), id, "do Caio");
    r.at(1);
    r.ask(0, private_on());
    write(r.b(0), id, "segredo da Bia");
    r.at(2);
    write(r.b(1), id, "");
    r.at(3);
    assert_eq!(text(r.b(1), id), "do Caio", "a escrita do Caio desfez-se");
    write(&mut r.hb, id, "da Ana");
    r.at(4);
    assert_eq!(text(&r.hb, id), "do Caio", "e a da anfitriã também");
    r.ask(0, Ask::PrivateOff);
    r.at(4 + REVEAL_COUNTDOWN_MS);
    assert_eq!(text(&r.hb, id), "segredo da Bia");
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F4): um voto dado ANTES de a nota se esconder não congela o boletim — fica,
/// pode sair, e os votos seguintes contam; só um voto NOVO nela é recusado (e quem votou sabe).
#[test]
fn a_vote_cast_before_a_note_was_hidden_does_not_freeze_the_ballot() {
    let mut hb = board();
    let (n, m) = (note(&mut hb, "n", 0.0), note(&mut hb, "m", 20.0));
    let mut r = Room::new(hb, &["Bia", "Caio"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup {
                votes: 3,
                one_per_object: false,
                minutes: 0,
            },
            targets: BTreeSet::from([n, m]),
        },
    );
    r.vote(1, ballot(1, &[(n, 1)], false));
    r.ask(0, private_on());
    write(r.b(0), n, "agora privada");
    r.at(1);
    r.vote(1, ballot(1, &[(n, 1), (m, 1)], true));
    assert!(
        r.fac(0).vote.unwrap().done.contains(&11),
        "o boletim congelou"
    );
    r.vote(1, ballot(1, &[(n, 2), (m, 1)], true));
    assert!(
        r.peers[1]
            .events
            .contains(&Event::Denied(Refusal::BallotRefused))
    );
    assert_eq!(r.fac(0).vote.unwrap().used.get(&11), Some(&2));
    r.vote(1, ballot(1, &[(m, 1)], true));
    assert_eq!(
        r.fac(0).vote.unwrap().used.get(&11),
        Some(&1),
        "o voto antigo sai"
    );
}

/// ⛔ Auditoria W12 (rede, F5): um texto às claras (de quem ainda não sabia do modo) que passa à frente
/// do selado da anfitriã: quem entra depois converge, e depois de revelar todos têm o mesmo.
#[test]
fn a_newcomer_converges_when_a_clear_text_overtakes_the_hosts_sealed_one() {
    let mut hb = board();
    let id = note(&mut hb, "", 0.0);
    let mut r = Room::new(hb, &["Bia"]);
    if let Some(e) = r.host.ask(&mut r.hb, private_on(), 0) {
        panic!("{e:?}");
    }
    write(&mut r.hb, id, "da Ana");
    let out = r.host.tick(&mut r.hb, 0);
    let p = &mut r.peers[0];
    let b = p.b.as_mut().unwrap();
    write(b, id, "da Bia às claras");
    let up = p.g.tick(b, 0);
    p.up.extend(up.into_iter().map(|m| m.encode()));
    r.route(out);
    r.pump();
    let c = r.join("Caio");
    r.all_same();
    assert!(!r.heard(c, "da Ana"));
    r.host_ask(Ask::PrivateOff);
    r.at(REVEAL_COUNTDOWN_MS);
    r.all_same();
}

/// ⛔ Auditoria W12 (rede, F7): um boletim cuja soma não cabe (dava a volta e passava) é recusado.
#[test]
fn a_ballot_whose_sum_overflows_is_refused() {
    let mut hb = board();
    let (a, b) = (note(&mut hb, "a", 0.0), note(&mut hb, "b", 20.0));
    let mut r = Room::new(hb, &["Bia"]);
    r.ask(
        0,
        Ask::VoteStart {
            setup: VoteSetup::default(),
            targets: BTreeSet::from([a, b]),
        },
    );
    r.vote(0, ballot(1, &[(a, u32::MAX), (b, 2)], false));
    assert_eq!(r.host.fac(0, &r.hb).vote.unwrap().used.get(&10), None);
    assert!(
        r.peers[0]
            .events
            .contains(&Event::Denied(Refusal::BallotRefused))
    );
}
