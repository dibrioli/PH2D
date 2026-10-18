//! A sessão em estrela: entrar, a versão recusada, a presença (o tecto de envio e a interpolação),
//! «trazer todos», e a convergência com convidados que CAEM (perdem o que ia no fio) e voltam.

use std::collections::VecDeque;

use super::*;
use crate::presence::{RENDER_DELAY_MS, Track};
use ph2d_board_model::{
    BoardOp, BoardSet, Element, ElementId, History, Rgba, RichText, Shape, ShapeType, Style,
};

fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    set.get_mut(id).unwrap().clone()
}

fn rect(b: &mut Board, x: f64) -> ElementId {
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: RichText::plain("x"),
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

fn presence(x: f64, sel: Vec<ElementId>) -> Presence {
    Presence {
        peer: 0,
        cursor: Some([x, 0.0]),
        selection: sel,
        camera: Camera::default(),
    }
}

/// O ecrã de mentira de um convidado: guarda o quadro que chega no 1.º `Welcome`.
fn deliver(
    g: &mut Guest,
    slot: &mut Option<Board>,
    bytes: &[u8],
    now: u64,
) -> (Vec<Msg>, Vec<Event>) {
    let (out, mut ev) = g.frame(slot.as_mut(), bytes, now);
    ev.retain(|e| match e {
        Event::Arrived(b) => {
            *slot = Some((**b).clone());
            false
        }
        _ => true,
    });
    (out, ev)
}

fn shared(b: &Board) -> Vec<u8> {
    let els: Vec<&Element> = b.doc.live_with_hidden().collect();
    postcard::to_allocvec(&(els, &b.name, b.sketch)).unwrap()
}

/// Quem fala outra versão do protocolo é recusado, e o convidado sabe porquê.
#[test]
fn another_protocol_is_refused_with_a_reason() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let hello = Msg::Hello {
        protocol: PROTOCOL + 1,
        peer: 2,
        name: "Bia".into(),
        key: 2,
    };
    let (out, _) = host.frame(&mut hb, 7, &hello.encode(), 0);
    assert_eq!(out, vec![(To::One(7), Msg::Refused { protocol: PROTOCOL })]);
    let mut g = Guest::new(2, "Bia".into(), 2, 2);
    let mut gb = None;
    let (_, ev) = deliver(&mut g, &mut gb, &out[0].1.encode(), 0);
    assert_eq!(ev, vec![Event::Refused { theirs: PROTOCOL }]);
    assert!(gb.is_none());
}

/// ⭐ O id de cada pessoa corre a sala inteira, a chave só vem ao anfitrião: quem traz o id da Bia (ou o
/// do anfitrião) sem a chave dela é recusado e não recebe NADA — nem o quadro, nem o que é dela (o
/// cofre, o boletim) — e a Bia continua a ser a Bia; ela própria volta de uma queda com a sua chave.
#[test]
fn an_id_without_its_key_is_refused_and_its_owner_comes_back() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let bia = Guest::new(2, "Bia".into(), 2, 0xB1A);
    let (out, _) = host.frame(&mut hb, 7, &bia.hello().encode(), 0);
    assert!(matches!(&out[0].1, Msg::Welcome { you, .. } if you.peer == 2));
    let refused = vec![(To::One(9), Msg::Refused { protocol: PROTOCOL })];
    for (peer, key, who) in [
        (2, 0xBAD, "a Bia"),
        (1, 0xBAD, "o anfitrião"),
        (1, 1, "o anfitrião"),
    ] {
        let forged = Guest::new(peer, "Bia".into(), 9, key);
        let (out, ev) = host.frame(&mut hb, 9, &forged.hello().encode(), 0);
        assert_eq!(out, refused, "fez-se passar por {who}");
        assert!(ev.is_empty(), "{who}: {ev:?}");
    }
    // O impostor recusado não fala por ninguém: o fio dele não tem identidade.
    let (out, _) = host.frame(&mut hb, 9, &Msg::Ballot(Ballot::default()).encode(), 0);
    assert!(out.is_empty(), "{out:?}");
    // A Bia cai e volta noutro fio, com a SUA chave: é ela, com a cor dela.
    let _ = host.left(7);
    let (out, _) = host.frame(&mut hb, 8, &bia.hello().encode(), 0);
    assert!(
        matches!(&out[0].1, Msg::Welcome { you, .. } if you.peer == 2 && you.color == 1),
        "{out:?}"
    );
}

/// O `Hello` do protocolo 3 (sem a chave) é recusado com a frase de sempre (antes de se ler, o par antigo
/// ficava à espera); e o `Hello` de agora, lido por um anfitrião do protocolo 3, dá-lhe a versão para o
/// recusar (a chave vai no FIM e ele ignora-a).
#[test]
fn the_old_hello_and_the_new_one_are_refused_across_versions() {
    #[derive(serde::Serialize, serde::Deserialize)]
    enum V3 {
        Hello {
            protocol: u32,
            peer: PeerId,
            name: String,
        },
    }
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let old = V3::Hello {
        protocol: 3,
        peer: 2,
        name: "Bia".into(),
    };
    let (out, _) = host.frame(&mut hb, 7, &postcard::to_allocvec(&old).unwrap(), 0);
    assert_eq!(out, vec![(To::One(7), Msg::Refused { protocol: PROTOCOL })]);
    let new = Guest::new(2, "Bia".into(), 2, 2).hello().encode();
    let Ok(V3::Hello { protocol, .. }) = postcard::from_bytes::<V3>(&new) else {
        panic!("um anfitrião do protocolo 3 não leria o Hello de agora");
    };
    assert_eq!(protocol, PROTOCOL);
    // O controlo: nada que não seja um `Hello` antigo recebe resposta.
    let (out, _) = host.frame(&mut hb, 7, &[0xFF, 0xFF, 0xFF], 0);
    assert!(out.is_empty());
}

/// Entrar: o convidado recebe o quadro do anfitrião, a cor seguinte, e os outros sabem que chegou;
/// quem volta guarda a cor.
#[test]
fn joining_gives_the_hosts_board_and_the_next_colour() {
    let mut hb = board();
    rect(&mut hb, 3.0);
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let g = Guest::new(2, "Bia".into(), 2, 2);
    let (out, ev) = host.frame(&mut hb, 7, &g.hello().encode(), 0);
    let Msg::Welcome { you, .. } = &out[0].1 else {
        panic!("{out:?}");
    };
    assert_eq!((you.color, &ev[0]), (1, &Event::Joined(you.clone())));
    assert_eq!(
        out.iter()
            .find(|(_, m)| matches!(m, Msg::Joined(_)))
            .map(|(to, _)| *to),
        Some(To::AllBut(7))
    );
    assert!(
        matches!(&out[1], (To::One(7), Msg::Fac(_))),
        "quem entra recebe já o estado da facilitação"
    );
    let mut g = g;
    let mut gb = None;
    let (_, ev) = deliver(&mut g, &mut gb, &out[0].1.encode(), 0);
    assert_eq!(ev, vec![Event::Welcomed]);
    assert_eq!(shared(gb.as_ref().unwrap()), shared(&hb));
    host.left(7);
    let (out, _) = host.frame(&mut hb, 8, &g.hello().encode(), 1);
    let Msg::Welcome { you, .. } = &out[0].1 else {
        panic!();
    };
    assert_eq!(you.color, 1, "quem volta guarda a cor");
}

/// A presença sai no máximo a cada `SEND_EVERY_MS`; a última nunca se perde (sai no tique seguinte);
/// uma selecção nova sai já.
#[test]
fn presence_is_capped_and_the_last_one_is_never_lost() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    assert_eq!(host.presence(0, presence(1.0, vec![])).len(), 1);
    assert!(host.presence(10, presence(2.0, vec![])).is_empty());
    let id = rect(&mut hb, 0.0);
    let _ = host.tick(&mut hb, 11);
    assert_eq!(
        host.presence(12, presence(3.0, vec![id])).len(),
        1,
        "selecção"
    );
    assert!(host.presence(20, presence(4.0, vec![id])).is_empty());
    assert!(host.tick(&mut hb, 30).is_empty(), "ainda não");
    let out = host.tick(&mut hb, 12 + SEND_EVERY_MS);
    assert!(
        matches!(&out[..], [(To::All, Msg::Presence(p))] if p.cursor == Some([4.0, 0.0])),
        "{out:?}"
    );
}

/// O cursor alheio desenha-se `RENDER_DELAY_MS` no passado, ENTRE as amostras; nunca adivinha para a
/// frente nem volta atrás.
#[test]
fn the_others_cursor_glides_between_samples() {
    let mut t: Track<2> = Track::default();
    t.push(1000, [0.0, 0.0]);
    t.push(1000 + SEND_EVERY_MS, [33.0, 0.0]);
    let mid = t.at(1000 + RENDER_DELAY_MS + 10).unwrap();
    assert!((mid[0] - 10.0).abs() < 1e-9, "{mid:?}");
    assert_eq!(t.at(1000 + RENDER_DELAY_MS + 999), Some([33.0, 0.0]));
    assert_eq!(t.at(0), Some([0.0, 0.0]));
    t.push(5, [99.0, 0.0]);
    assert_eq!(
        t.at(5000),
        Some([33.0, 0.0]),
        "uma amostra velha não volta atrás"
    );
}

/// «Trazer todos»: quem chama não se chama a si; o anfitrião e os outros recebem a câmara.
#[test]
fn summon_reaches_everyone_but_the_caller() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let mut b = Guest::new(2, "Bia".into(), 2, 2);
    let mut bb = None;
    let (out, _) = host.frame(&mut hb, 7, &b.hello().encode(), 0);
    deliver(&mut b, &mut bb, &out[0].1.encode(), 0);
    let cam = Camera {
        center_x: 500.0,
        center_y: -20.0,
        zoom: 2.0,
    };
    let (out, ev) = host.frame(&mut hb, 7, &b.summon(cam).encode(), 1);
    assert_eq!(
        ev,
        vec![Event::Summoned {
            from: 2,
            camera: cam
        }]
    );
    assert_eq!(out[0].0, To::AllBut(7));
    let (_, ev) = deliver(&mut b, &mut bb, &out[0].1.encode(), 2);
    assert!(ev.is_empty(), "quem chamou não se chama");
}

/// Quem entra vê JÁ o cursor de quem está parado: o do anfitrião e o dos outros convidados (a
/// presença só sai quando muda — sem o retrato dela, o recém-chegado não via ninguém).
#[test]
fn a_newcomer_sees_where_everyone_already_is() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    assert_eq!(
        host.presence(0, presence(5.0, vec![])).len(),
        1,
        "a Ana pôs o rato"
    );
    let mut b = Guest::new(2, "Bia".into(), 2, 2);
    let mut bb = None;
    let (out, _) = host.frame(&mut hb, 7, &b.hello().encode(), 10);
    for (_, m) in &out {
        deliver(&mut b, &mut bb, &m.encode(), 10);
    }
    let pb = b.presence(20, presence(9.0, vec![])).remove(0);
    let _ = host.frame(&mut hb, 7, &pb.encode(), 20);
    let mut c = Guest::new(3, "Caio".into(), 3, 3);
    let mut cb = None;
    let (out, _) = host.frame(&mut hb, 8, &c.hello().encode(), 30);
    let to_caio: Vec<u64> = out
        .iter()
        .filter_map(|(to, m)| match (to, m) {
            (To::One(8), Msg::Presence(p)) => Some(p.peer),
            _ => None,
        })
        .collect();
    assert_eq!(
        to_caio,
        vec![1, 2],
        "o Caio não soube onde estavam a Ana e a Bia"
    );
    for (to, m) in &out {
        if *to == To::One(8) {
            deliver(&mut c, &mut cb, &m.encode(), 30);
        }
    }
    let late = 30 + RENDER_DELAY_MS + 1;
    assert!(
        c.seen(1).and_then(|s| s.cursor_at(late)).is_some(),
        "o cursor da Ana"
    );
    assert!(
        c.seen(2).and_then(|s| s.cursor_at(late)).is_some(),
        "o cursor da Bia"
    );
}

/// Quem já saiu não aparece a quem entra depois (o anfitrião guarda-o só para a cor de quem volta).
#[test]
fn a_newcomer_is_not_told_about_who_already_left() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let b = Guest::new(2, "Bia".into(), 2, 2);
    let _ = host.frame(&mut hb, 7, &b.hello().encode(), 0);
    let _ = host.left(7);
    let mut c = Guest::new(3, "Caio".into(), 3, 3);
    let mut cb = None;
    let (out, _) = host.frame(&mut hb, 8, &c.hello().encode(), 10);
    for (to, m) in &out {
        if *to == To::One(8) {
            deliver(&mut c, &mut cb, &m.encode(), 10);
        }
    }
    let here: Vec<u64> = c.peers().iter().map(|p| p.peer).collect();
    assert!(!here.contains(&2), "a Bia saiu e o Caio vê-a: {here:?}");
    assert!(here.contains(&1) && here.contains(&3), "{here:?}");
}

/// O anfitrião fecha a sala de propósito: o convidado sabe-o (e não confunde com uma queda), e o
/// adeus é o ÚLTIMO byte da enumeração (um índice que viaja não se reordena).
#[test]
fn the_host_ending_the_room_is_told_to_the_guest() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let mut b = Guest::new(2, "Bia".into(), 2, 2);
    let mut bb = None;
    let (out, _) = host.frame(&mut hb, 7, &b.hello().encode(), 0);
    deliver(&mut b, &mut bb, &out[0].1.encode(), 0);
    let (_, ev) = deliver(&mut b, &mut bb, &Msg::End.encode(), 1);
    assert_eq!(ev, vec![Event::Ended]);
    assert_eq!(Msg::End.encode(), vec![8], "o adeus mudou de byte");
    assert!(bb.is_some(), "a cópia fica");
}

/// O transporte de mentira: cada sentido de cada ligação é uma fila FIFO (como o TCP) com atraso
/// sorteado; cair esvazia as duas filas.
struct Link {
    conn: ConnId,
    up: VecDeque<(u64, Vec<u8>)>,
    down: VecDeque<(u64, Vec<u8>)>,
    online: bool,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn send_from_host(links: &mut [Link], out: Vec<(To, Msg)>, at: u64) {
    for (to, m) in out {
        let bytes = m.encode();
        for l in links.iter_mut().filter(|l| l.online) {
            let hit = match to {
                To::One(c) => c == l.conn,
                To::All => true,
                To::AllBut(c) => c != l.conn,
            };
            if hit {
                let t = l.down.back().map_or(at, |(t, _)| at.max(*t));
                l.down.push_back((t, bytes.clone()));
            }
        }
    }
}

fn edit(b: &mut Board, h: &mut History, rng: &mut Rng) {
    let ids: Vec<ElementId> = b.doc.live().map(|e| e.id).collect();
    if ids.is_empty() || rng.below(4) == 0 {
        rect(b, rng.below(300) as f64);
        return;
    }
    let id = ids[rng.below(ids.len() as u64) as usize];
    let mut el = b.doc.get(id).unwrap().clone();
    match rng.below(5) {
        0 => el.x = rng.below(300) as f64,
        1 => el.style_mut().fill = Some(Rgba([rng.below(255) as u8, 1, 1, 255])),
        2 => {
            h.apply(&mut b.doc, vec![BoardOp::Delete(id)]);
            return;
        }
        3 => {
            h.undo(&mut b.doc);
            return;
        }
        _ => {
            el.add_stamp(ph2d_board_model::Stamp::Heart, 1);
        }
    }
    h.apply(&mut b.doc, vec![BoardOp::Put(el)]);
}

/// ⭐⭐ Convergência em estrela: anfitrião + 3 convidados, atraso sorteado por ligação, edições e
/// desfazer em todos, e convidados que CAEM (o que ia no fio perde-se, nos dois sentidos) e voltam.
/// No fim, os quatro com o mesmo quadro.
#[test]
fn a_star_room_with_drops_ends_with_one_board() {
    for seed in 1..=25u64 {
        let mut rng = Rng(seed * 0x2545_f491 + 3);
        let mut hb = board();
        rect(&mut hb, 0.0);
        let mut host = Host::open(1, "Ana".into(), &mut hb, seed);
        let mut hh = History::default();
        let n = 3;
        let mut guests: Vec<(Guest, Option<Board>, History)> = (0..n)
            .map(|i| {
                (
                    Guest::new(10 + i, format!("p{i}"), seed * 7 + i, seed * 7 + i),
                    None,
                    History::default(),
                )
            })
            .collect();
        let mut links: Vec<Link> = (0..n)
            .map(|i| Link {
                conn: i,
                up: VecDeque::new(),
                down: VecDeque::new(),
                online: true,
            })
            .collect();
        let mut next_conn = n;
        for (i, l) in links.iter_mut().enumerate() {
            l.up.push_back((0, guests[i].0.hello().encode()));
        }
        for now in 0..2000u64 {
            // Edições.
            if rng.below(3) == 0 {
                edit(&mut hb, &mut hh, &mut rng);
            }
            for (_, b, h) in &mut guests {
                if rng.below(4) == 0
                    && let Some(b) = b
                {
                    edit(b, h, &mut rng);
                }
            }
            // Quedas e voltas.
            if rng.below(150) == 0 {
                let i = rng.below(n) as usize;
                if links[i].online {
                    links[i].online = false;
                    links[i].up.clear();
                    links[i].down.clear();
                    let out = host.left(links[i].conn);
                    send_from_host(&mut links, out, now);
                } else {
                    links[i].online = true;
                    links[i].conn = next_conn;
                    next_conn += 1;
                    links[i].up.push_back((now, guests[i].0.hello().encode()));
                }
            }
            // Entregas (o que chegou até agora).
            for i in 0..n as usize {
                while links[i].online && links[i].up.front().is_some_and(|(t, _)| *t <= now) {
                    let (_, bytes) = links[i].up.pop_front().unwrap();
                    let (out, _) = host.frame(&mut hb, links[i].conn, &bytes, now);
                    send_from_host(&mut links, out, now + rng.below(20));
                }
                while links[i].online && links[i].down.front().is_some_and(|(t, _)| *t <= now) {
                    let (_, bytes) = links[i].down.pop_front().unwrap();
                    let (g, b, _) = &mut guests[i];
                    let (out, _) = deliver(g, b, &bytes, now);
                    for m in out {
                        links[i].up.push_back((now + rng.below(20), m.encode()));
                    }
                }
            }
            // Os tiques.
            let out = host.tick(&mut hb, now);
            send_from_host(&mut links, out, now + rng.below(20));
            for (i, (g, b, _)) in guests.iter_mut().enumerate() {
                if let Some(b) = b
                    && links[i].online
                {
                    for m in g.tick(b, now) {
                        let t = links[i].up.back().map_or(now, |(t, _)| now.max(*t));
                        links[i].up.push_back((t + rng.below(20), m.encode()));
                    }
                }
            }
        }
        // Todos voltam e a rede esvazia-se.
        for (i, l) in links.iter_mut().enumerate() {
            if !l.online {
                l.online = true;
                l.conn = next_conn;
                next_conn += 1;
                l.up.push_back((2000, guests[i].0.hello().encode()));
            }
        }
        for now in 2000..4000u64 {
            for i in 0..n as usize {
                while let Some((_, bytes)) = links[i].up.pop_front() {
                    let (out, _) = host.frame(&mut hb, links[i].conn, &bytes, now);
                    send_from_host(&mut links, out, now);
                }
                while let Some((_, bytes)) = links[i].down.pop_front() {
                    let (g, b, _) = &mut guests[i];
                    let (out, _) = deliver(g, b, &bytes, now);
                    links[i]
                        .up
                        .extend(out.into_iter().map(|m| (now, m.encode())));
                }
            }
            let out = host.tick(&mut hb, now);
            send_from_host(&mut links, out, now);
            for (i, (g, b, _)) in guests.iter_mut().enumerate() {
                if let Some(b) = b {
                    links[i]
                        .up
                        .extend(g.tick(b, now).into_iter().map(|m| (now, m.encode())));
                }
            }
            if links.iter().all(|l| l.up.is_empty() && l.down.is_empty()) {
                break;
            }
        }
        for (i, (_, b, _)) in guests.iter().enumerate() {
            let b = b.as_ref().unwrap();
            assert!(
                shared(b) == shared(&hb),
                "semente {seed}: o convidado {i} divergiu ({} × {} elementos)",
                b.doc.live().count(),
                hb.doc.live().count()
            );
        }
    }
}

/// 📏 Medição (à mão, `--ignored --nocapture`): os BYTES de arrastar 1, 10 e 50 formas — o que sai a
/// cada imagem (60 Hz) e por segundo, sem a cifra (+20 B por quadro: 4 do comprimento e 16 da
/// etiqueta). Comparar com o oráculo (c19): Excalidraw 681 B / 5,2 KB / 25 KB por mensagem.
#[test]
#[ignore = "medição: corre-se à mão"]
fn measure_the_bytes_of_dragging_1_10_and_50_shapes() {
    for n in [1usize, 10, 50] {
        let mut hb = board();
        let ids: Vec<ElementId> = (0..n).map(|i| rect(&mut hb, i as f64 * 20.0)).collect();
        let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
        let _ = host.tick(&mut hb, 0);
        let mut total = 0usize;
        let frames = 60;
        for k in 1..=frames {
            for id in &ids {
                let mut el = hb.doc.get(*id).unwrap().clone();
                el.translate([1.0, 0.0]);
                let _ = BoardOp::Put(el).apply(&mut hb.doc);
            }
            let out = host.tick(&mut hb, k * 16);
            total += out
                .iter()
                .map(|(_, m)| m.encode().len() + 20)
                .sum::<usize>();
        }
        eprintln!(
            "{n} formas: {} B por imagem · {:.1} KB/s a 60 Hz",
            total / frames as usize,
            total as f64 / 1024.0
        );
    }
}

/// Um anfitrião com um retângulo e a Bia já dentro (o quadro dela é o 2.º).
fn bia_inside() -> (Host, Board, Guest, Board) {
    let mut hb = board();
    rect(&mut hb, 0.0);
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let mut g = Guest::new(2, "Bia".into(), 3, 4);
    let (out, _) = host.frame(&mut hb, 7, &g.hello().encode(), 0);
    let mut slot = None;
    for (_, m) in out {
        deliver(&mut g, &mut slot, &m.encode(), 0);
    }
    (host, hb, g, slot.expect("a Bia recebeu o quadro"))
}

fn slide(b: &mut Board, x: f64) {
    let mut el = b.doc.live().next().unwrap().clone();
    el.x = x;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
}

fn writes(out: &[Msg]) -> usize {
    out.iter().filter(|m| matches!(m, Msg::Writes(_))).count()
}

/// ⭐ B8 (W23) — com ritmo, as escritas saem no máximo uma vez por período: a primeira depois de uma
/// pausa sai JÁ, um arrasto a 60 Hz sai a ~30 Hz, e a última posição chega sempre. Sem ritmo, cada
/// imagem leva a sua (o controlo).
#[test]
fn paced_writes_leave_once_per_period_and_the_last_position_always_arrives() {
    for (every, lo, hi) in [(0u64, 60usize, 60usize), (33, 28, 31)] {
        let (mut host, mut hb, mut g, mut b) = bia_inside();
        g.write_every(every);
        // Imagens sem nada a mandar não gastam a vez.
        assert_eq!(writes(&g.tick(&mut b, 990)), 0);
        slide(&mut b, 1.0);
        assert_eq!(writes(&g.tick(&mut b, 1_000)), 1, "a 1.ª esperou ({every})");
        let mut out = Vec::new();
        let mut sent = 0;
        for k in 2..=61u64 {
            slide(&mut b, k as f64);
            let o = g.tick(&mut b, 1_000 + (k - 1) * 16);
            sent += writes(&o);
            out.extend(o);
        }
        assert!(
            (lo..=hi).contains(&sent),
            "{every} ms: {sent} escritas em 60 imagens"
        );
        // Parado o arrasto, a próxima vez leva o que faltava.
        out.extend(g.tick(&mut b, 1_000 + 60 * 16 + every));
        for m in out {
            let _ = host.frame(&mut hb, 7, &m.encode(), 2_000);
        }
        assert_eq!(
            hb.doc.live().next().unwrap().x,
            61.0,
            "{every} ms: a última posição"
        );
    }
}

/// B8 — fechar a sala leva o que o ritmo ainda guardava (o adeus nunca deixa uma posição para trás).
#[test]
fn closing_the_room_sends_what_the_pace_was_holding() {
    let (mut host, mut hb, mut g, mut b) = bia_inside();
    host.write_every(33);
    slide(&mut hb, 5.0);
    let first = host.tick(&mut hb, 1_000);
    slide(&mut hb, 9.0);
    let held = host.tick(&mut hb, 1_010);
    assert!(
        held.iter().all(|(_, m)| !matches!(m, Msg::Writes(_))),
        "saiu antes da vez"
    );
    let last = host.close(&mut hb, 1_011);
    for (_, m) in first.into_iter().chain(last) {
        let _ = g.frame(Some(&mut b), &m.encode(), 1_011);
    }
    assert_eq!(b.doc.live().next().unwrap().x, 9.0);
}

/// B8 (W23) — o tecto da presença é EM MÉDIA: com imagens de 16 ms, um cursor que mexe sai a ~30 Hz
/// (contado «desde o último envio», só saía a cada três imagens: 20 Hz).
#[test]
fn a_moving_cursor_goes_out_at_thirty_hertz_with_sixty_hertz_frames() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let mut sent = 0;
    for k in 0..60u64 {
        let now = 1_000 + k * 16;
        sent += host.presence(now, presence(k as f64, vec![])).len();
        sent += host.tick(&mut hb, now).len();
    }
    assert!((28..=31).contains(&sent), "{sent} presenças em 60 imagens");
}
