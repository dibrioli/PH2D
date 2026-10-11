//! W14 — as imagens ao vivo: quem entra recebe o retrato SEM os bytes e depois cada ficheiro numa
//! mensagem só dele; a imagem que um convidado põe chega aos outros pelo anfitrião.

use super::*;
use ph2d_board_model::{BoardOp, BoardSet, Element, FileId, Image, ImageFile, Rgba, Style};

fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    set.get_mut(id).unwrap().clone()
}

/// Uma imagem de `n` (os bytes são `n` repetido) no quadro; devolve a chave.
fn image(b: &mut Board, n: u8) -> FileId {
    let file = b.put_file(ImageFile::new(vec![n; 64], [8, 8]));
    let img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        img,
        [f64::from(n), 0.0, 8.0, 8.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    file
}

/// Entrega as mensagens do anfitrião a um convidado, guardando o quadro que chega no `Welcome`.
fn deliver(g: &mut Guest, slot: &mut Option<Board>, msgs: &[Msg]) -> Vec<Msg> {
    let mut back = Vec::new();
    for m in msgs {
        let (out, ev) = g.frame(slot.as_mut(), &m.encode(), 0);
        back.extend(out);
        for e in ev {
            if let Event::Arrived(b) = e {
                *slot = Some(*b);
            }
        }
    }
    back
}

fn to_conn(out: &[(To, Msg)], conn: ConnId) -> Vec<Msg> {
    out.iter()
        .filter(|(to, _)| match to {
            To::One(c) => *c == conn,
            To::All => true,
            To::AllBut(c) => *c != conn,
        })
        .map(|(_, m)| m.clone())
        .collect()
}

fn files_in(m: &Msg) -> usize {
    match m {
        Msg::Writes(ws) => ws
            .iter()
            .filter(|w| matches!(w, Write::File { .. }))
            .count(),
        _ => 0,
    }
}

/// ⭐ O retrato de quem entra vai sem os bytes das imagens; cada ficheiro segue na sua mensagem; no
/// fim o convidado tem as imagens E os bytes delas.
#[test]
fn a_joiner_gets_the_lean_board_then_each_file_alone() {
    let mut hb = board();
    let keys = [image(&mut hb, 1), image(&mut hb, 2)];
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let mut g = Guest::new(2, "Bia".into(), 2, 2);
    let (out, _) = host.frame(&mut hb, 7, &g.hello().encode(), 0);
    let msgs = to_conn(&out, 7);
    let Msg::Welcome { snapshot, .. } = &msgs[0] else {
        panic!("o 1.º não é o retrato: {:?}", msgs[0]);
    };
    let lean = BoardSet::board_from_bytes(&snapshot.board).unwrap();
    assert!(lean.files.is_empty(), "o retrato leva os bytes");
    let each: Vec<usize> = msgs.iter().map(files_in).filter(|n| *n > 0).collect();
    assert_eq!(each, [1, 1], "um ficheiro por mensagem");
    let mut gb = None;
    deliver(&mut g, &mut gb, &msgs);
    let gb = gb.expect("entrou");
    assert_eq!(gb.doc.live_len(), 2);
    for k in keys {
        assert_eq!(gb.file(k), hb.file(k), "faltam os bytes de {k:?}");
    }
}

/// ⭐ A imagem que a Bia põe chega ao Rui pelo anfitrião (os bytes também), numa mensagem do ficheiro
/// à parte das escritas do elemento; e o anfitrião não a devolve a ela.
#[test]
fn a_guest_image_reaches_the_others_through_the_host() {
    let mut hb = board();
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let (mut bia, mut rui) = (
        Guest::new(2, "Bia".into(), 2, 2),
        Guest::new(3, "Rui".into(), 3, 3),
    );
    let (mut bb, mut rb) = (None, None);
    for (g, slot, conn) in [(&mut bia, &mut bb, 7), (&mut rui, &mut rb, 8)] {
        let (out, _) = host.frame(&mut hb, conn, &g.hello().encode(), 0);
        let back = deliver(g, slot, &to_conn(&out, conn));
        for m in back {
            let _ = host.frame(&mut hb, conn, &m.encode(), 0);
        }
    }
    let key = image(bb.as_mut().unwrap(), 5);
    let sent = bia.tick(bb.as_mut().unwrap(), 10);
    assert_eq!(sent.iter().map(files_in).filter(|n| *n > 0).count(), 1);
    let mut relayed = Vec::new();
    for m in &sent {
        let (out, _) = host.frame(&mut hb, 7, &m.encode(), 10);
        assert!(
            to_conn(&out, 7).iter().all(|m| files_in(m) == 0),
            "devolveu à Bia"
        );
        relayed.extend(to_conn(&out, 8));
    }
    assert!(hb.file(key).is_some(), "o anfitrião não guardou os bytes");
    deliver(&mut rui, &mut rb, &relayed);
    let rb = rb.unwrap();
    assert!(rb.file(key).is_some(), "o Rui não tem os bytes");
    assert!(
        rb.doc
            .live()
            .any(|e| e.image().is_some_and(|i| i.file == key))
    );
}

/// Auditoria W14 — quem entra não recebe os bytes que nenhum elemento refere (o resto de uma colagem,
/// de uma imagem que nunca ficou); recebe os de uma imagem APAGADA (um desfazer trá-la de volta).
#[test]
fn a_joiner_gets_only_the_files_something_refers_to() {
    let mut hb = board();
    let gone = image(&mut hb, 1);
    let id = hb.doc.live().find(|e| e.image().is_some()).unwrap().id;
    let _ = BoardOp::Delete(id).apply(&mut hb.doc);
    let stray = hb.put_file(ImageFile::new(vec![9; 16], [4, 4]));
    let mut host = Host::open(1, "Ana".into(), &mut hb, 1);
    let g = Guest::new(2, "Bia".into(), 2, 2);
    let (out, _) = host.frame(&mut hb, 7, &g.hello().encode(), 0);
    let sent: Vec<FileId> = to_conn(&out, 7)
        .iter()
        .flat_map(|m| match m {
            Msg::Writes(ws) => ws.clone(),
            _ => Vec::new(),
        })
        .filter_map(|w| match w {
            Write::File { file, .. } => Some(file.id()),
            _ => None,
        })
        .collect();
    assert_eq!(sent, [gone], "o resto ({stray:?}) viajou, ou o apagado não");
}
