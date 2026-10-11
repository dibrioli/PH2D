use super::{BoardSet, Camera, FORMAT_VERSION};
use crate::{BoardOp, Element, Rgba, RichText, Shape, ShapeType, Style};

/// ⭐ Um ficheiro do formato 2 (W1–W2: o texto era uma `String`) continua a abrir — os bytes abaixo
/// foram gravados por esse build (`d8a331ec3`): um quadro «Ideias» com uma pílula rodada, cantos
/// redondos, com «olá» dentro, e uma seta em cotovelo presa a ela, com rótulo e um ponto de ajuste.
#[test]
fn a_format_2_file_still_opens_with_its_text_and_arrow() {
    let set = BoardSet::from_bytes(V2_BYTES).expect("o formato 2 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Ideias");
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 2);
    let s = els[0].shape().expect("a 1.ª é a forma");
    assert_eq!(s.kind, ShapeType::Pill);
    assert_eq!(s.text.as_str(), "olá");
    assert!(s.text.spans().is_empty());
    assert!(s.style.round);
    assert_eq!(els[0].angle, 0.5);
    let c = els[1].connector().expect("a 2.ª é a seta");
    assert_eq!(c.label, "sim");
    assert_eq!(c.waypoints, vec![[5.0, 6.0]]);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 2 (as fontes de `d8a331ec3` compiladas à parte, `BoardSet::to_bytes`
/// escrito para o ficheiro) — nunca copiados à mão (uma cópia à mão de 251 números saiu com 253).
const V2_BYTES: &[u8] = include_bytes!("../fixtures/format_v2.bin");

/// O texto com trechos (W3) e uma nota grava-se e lê-se.
#[test]
fn rich_text_and_a_sticky_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let a = set.create("Notas".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([1, 2, 3, 255]);
    let mut text = RichText::plain("uma boa ideia");
    text.toggle(4..7, crate::Mark::Bold);
    text.restyle(8..13, |m| m.color = Some(Rgba([200, 0, 0, 255])));
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(Rgba(crate::STICKY_COLORS[2])), None, ink),
        text,
    };
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 199.0, 199.0],
    );
    BoardOp::Put(el).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

#[test]
fn tabs_keep_their_order_through_create_duplicate_move_and_remove() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.create("B".into());
    let a2 = set.duplicate(a, "A cópia".into()).unwrap();
    let names = |s: &BoardSet| {
        s.boards()
            .iter()
            .map(|b| b.name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&set), ["A", "A cópia", "B"]);
    assert!(set.move_tab(b, 0));
    assert_eq!(names(&set), ["B", "A", "A cópia"]);
    assert!(set.move_tab(b, 99), "posição além do fim vai para o fim");
    assert_eq!(names(&set), ["A", "A cópia", "B"]);
    assert_eq!(set.remove(a2).map(|b| b.name), Some("A cópia".into()));
    assert!(set.remove(a2).is_none());
    assert!(!set.rename(a2, "x".into()));
}

#[test]
fn ids_are_never_reused_after_a_remove() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    set.remove(a);
    let b = set.create("B".into());
    assert_ne!(a, b);
}

#[test]
fn bytes_round_trip_with_content_camera_and_order() {
    let mut set = BoardSet::default();
    let a = set.create("Retro".into());
    set.create("Ideias".into());
    let board = set.get_mut(a).unwrap();
    board.camera = Camera {
        center_x: 120.5,
        center_y: -40.0,
        zoom: 2.5,
    };
    let id = board.doc.mint_id();
    let z = board.doc.z_on_top();
    let shape = Shape {
        kind: ShapeType::Diamond,
        style: Style::new(
            Some(Rgba([1, 2, 3, 4])),
            Some(Rgba([5, 6, 7, 8])),
            Rgba([9, 9, 9, 9]),
        ),
        text: "linha 1\nlinha 2".into(),
    };
    let mut el = Element::new_shape(id, z, shape, [1.0, 2.0, 3.0, 4.0]);
    el.nonce = 7;
    el.angle = 0.25;
    BoardOp::Put(el).apply(&mut board.doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

#[test]
fn empty_bytes_are_a_project_without_boards() {
    assert!(BoardSet::from_bytes(&[]).unwrap().is_empty());
}

#[test]
fn another_format_version_is_refused_not_misread() {
    let mut set = BoardSet::default();
    set.create("A".into());
    set.version = FORMAT_VERSION + 1;
    let err = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap_err();
    assert!(err.contains("format version"), "{err}");
}

#[test]
fn garbage_is_an_error_not_a_panic() {
    assert!(BoardSet::from_bytes(&[0xFF, 0xFF, 0xFF]).is_err());
}

/// ⭐ Um ficheiro do formato 1 (W0: rectângulos cheios) continua a abrir — os bytes abaixo foram
/// gravados por esse build: um quadro «Retro» com um rectângulo `(1,2,3,4)` de cor `[1,2,3,4]`.
#[test]
fn a_format_1_file_still_opens_as_rectangles() {
    let set = BoardSet::from_bytes(V1_BYTES).expect("o formato 1 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Retro");
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 1);
    let el = els[0];
    assert_eq!(
        [el.x, el.y, el.w, el.h, el.angle],
        [1.0, 2.0, 3.0, 4.0, 0.0]
    );
    let s = el.shape().unwrap();
    assert_eq!(s.kind, ShapeType::Rectangle);
    assert_eq!(s.style.fill, Some(Rgba([1, 2, 3, 4])));
    assert!(s.text.is_empty());
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 1 (`e74c81f32`, `BoardSet::to_bytes`), não reescritos à mão.
const V1_BYTES: &[u8] = &[
    1, 1, 1, 5, 82, 101, 116, 114, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 240, 63, 1, 1, 1, 0, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0,
    0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 16, 64, 2, 97, 48, 1, 0, 0, 1, 1, 2, 97, 48, 1,
];

/// Uma seta (W2) grava-se e lê-se: pontas presas e soltas, rota, pontas de seta, rótulo. A variante
/// entrou no FIM do `ElementKind` — o formato 2 não sobe, e um ficheiro só de formas lê-se igual.
#[test]
fn an_arrow_round_trips_through_the_bytes() {
    use crate::{Anchor, Connector, End, Head, Route};
    let mut set = BoardSet::default();
    let a = set.create("Fluxo".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([1, 2, 3, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: RichText::default(),
    };
    let box_el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 10.0, 10.0]);
    let target = box_el.id;
    BoardOp::Put(box_el).apply(doc);
    let mut c = Connector::new(
        End::Bound {
            target,
            anchor: Anchor::Fixed([1.0, 0.5]),
        },
        End::Free([40.0, -3.5]),
        Route::Curved,
        Style::new(None, Some(ink), ink),
    );
    c.heads = [Head::Circle, Head::Triangle];
    c.label = "sim".into();
    let arrow = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    BoardOp::Put(arrow).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    assert_eq!(FORMAT_VERSION, 12);
}

/// A revisão da sessão muda a cada operação aplicada e nunca se repete entre documentos (a cache de
/// rotas confia nisso para não confundir dois quadros).
#[test]
fn the_session_revision_moves_with_each_op_and_is_unique() {
    let mut set = BoardSet::default();
    let (a, b) = (set.create("A".into()), set.create("B".into()));
    let (ra, rb) = (set.get(a).unwrap().doc.rev(), set.get(b).unwrap().doc.rev());
    assert_ne!(ra, rb);
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([0, 0, 0, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: RichText::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 1.0, 1.0]);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    assert_ne!(doc.rev(), ra);
    let r = doc.rev();
    assert!(BoardOp::Delete(id).apply(doc).is_some());
    assert_ne!(doc.rev(), r);
    let r = doc.rev();
    assert!(
        BoardOp::Delete(id).apply(doc).is_none(),
        "já apagado: nada mudou"
    );
    assert_eq!(doc.rev(), r, "uma op que não muda nada não mexe na revisão");
}

/// ⭐ Um ficheiro do formato 3 (W3) continua a abrir — gravado pelo build de `ea38b5e61` (o último
/// do formato 3): um rectângulo rodado com cantos redondos e texto com trechos, uma nota laranja e
/// uma seta curva presa ao rectângulo, com rótulo e um ponto de ajuste. Tudo nasce FINAL.
#[test]
fn a_format_3_file_still_opens_and_everything_in_it_is_final() {
    let set = BoardSet::from_bytes(V3_BYTES).expect("o formato 3 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Esboço");
    assert_eq!(b.camera.zoom, 1.5);
    assert!(!b.sketch);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 3);
    let r = els[0].shape().expect("forma");
    assert_eq!(r.kind, ShapeType::Rectangle);
    assert_eq!(r.text.as_str(), "fazer o quê");
    assert_eq!(r.text.spans().len(), 2, "os trechos sobrevivem");
    assert!(r.style.round);
    assert_eq!(els[0].angle, 0.25);
    assert_eq!(els[1].shape().expect("nota").kind, ShapeType::Sticky);
    let c = els[2].connector().expect("seta");
    assert_eq!(c.label, "sim");
    assert_eq!(c.waypoints, vec![[300.0, 250.0]]);
    assert!(els.iter().all(|e| !e.style().sketch));
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 3 (`ea38b5e61`, um teste temporário que escreveu `to_bytes`).
const V3_BYTES: &[u8] = include_bytes!("../fixtures/format_v3.bin");

/// O rascunho (do quadro e de um elemento) e um traço da caneta gravam-se e lêem-se.
#[test]
fn sketch_and_a_pen_stroke_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let a = set.create("Caneta".into());
    let board = set.get_mut(a).unwrap();
    board.sketch = true;
    let doc = &mut board.doc;
    let ink = Rgba([1, 2, 3, 255]);
    let mut style = Style::new(None, Some(ink), ink);
    style.sketch = true;
    let shape = Shape {
        kind: ShapeType::Ellipse,
        style: style.clone(),
        text: RichText::default(),
    };
    let id = doc.mint_id();
    BoardOp::Put(Element::new_shape(
        id,
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 50.0, 30.0],
    ))
    .apply(doc);
    let (stroke, bx) = crate::Ink::from_world(
        &[[10.0, 10.0, 0.5], [30.0, 12.0, 0.7], [50.0, 40.0, 0.4]],
        Style::new(None, Some(ink), ink),
        crate::Pen::Highlighter,
        true,
    );
    let id = doc.mint_id();
    BoardOp::Put(Element::new_ink(id, doc.z_on_top(), stroke, bx)).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

/// ⭐ Um ficheiro do formato 4 (W4) continua a abrir — gravado pelo build de `90856427c` (o último
/// do formato 4): um quadro em rascunho com um rectângulo rodado à mão com texto a negrito, uma nota,
/// uma seta curva presa aos dois com rótulo e ponto de ajuste, e um traço da caneta. Tudo nasce
/// solto (sem frame nem grupo), destravado, sem link nem etiquetas. ⚠️ O traço tem de voltar TRAÇO:
/// é o que a variante `Frame` posta antes de `Ink` no enum teria lido como frame.
#[test]
fn a_format_4_file_still_opens_and_everything_in_it_is_loose() {
    let set = BoardSet::from_bytes(V4_BYTES).expect("o formato 4 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Mesa");
    assert_eq!(b.camera.zoom, 0.75);
    assert!(b.sketch);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 4);
    let r = els[0].shape().expect("forma");
    assert_eq!(r.text.as_str(), "plano A");
    assert!(r.style.sketch && r.style.round);
    assert_eq!(els[0].angle, 0.125);
    assert_eq!(els[1].shape().expect("nota").kind, ShapeType::Sticky);
    assert_eq!(els[2].connector().expect("seta").label, "depois");
    assert!(els[3].ink().is_some(), "o traço da caneta continua traço");
    for e in &els {
        assert!(e.frame.is_none() && e.groups().is_empty() && !e.locked());
        assert!(e.link().is_none() && e.tags().is_empty());
    }
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 4 (`90856427c`, um teste temporário que escreveu `to_bytes`).
const V4_BYTES: &[u8] = include_bytes!("../fixtures/format_v4.bin");

/// Os campos de organizar (W5) gravam-se e lêem-se: um frame dentro de outro, um elemento num
/// grupo de dois níveis, travado, com link e etiquetas.
#[test]
fn frames_groups_lock_link_and_tags_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let a = set.create("Org".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([1, 2, 3, 255]);
    let frame = |name: &str| crate::Frame {
        name: name.into(),
        style: Style::new(Some(Rgba([255, 255, 255, 255])), None, ink),
    };
    let outer = Element::new_frame(
        doc.mint_id(),
        doc.z_on_top(),
        frame("fora"),
        [0.0, 0.0, 500.0, 400.0],
    );
    let mut inner = Element::new_frame(
        doc.mint_id(),
        doc.z_on_top(),
        frame("dentro"),
        [10.0, 10.0, 200.0, 200.0],
    );
    inner.frame = Some(outer.id);
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(ink), None, ink),
        text: RichText::default(),
    };
    let mut note = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [20.0, 20.0, 50.0, 50.0],
    );
    note.frame = Some(inner.id);
    *note.groups_mut() = vec![doc.mint_group(), doc.mint_group()];
    note.set_locked(true);
    note.set_link(Some(outer.id));
    *note.tags_mut() = vec![crate::Tag {
        name: "urgente".into(),
        color: Rgba([200, 0, 0, 255]),
    }];
    for e in [outer, inner, note] {
        BoardOp::Put(e).apply(doc);
    }
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

/// ⭐ Um ficheiro do formato 8 (W8) continua a abrir com o aspecto de nascença (W9: `Style::look`) —
/// gravado pelo build de `3b4ec7784` (o último do formato 8, um teste temporário que escreveu
/// `to_bytes`): «Retro8» em rascunho, um frame «Sprint», um rectângulo redondo à mão dentro dele, uma
/// nota, uma seta curva (ponto fixo → centro, barra → triângulo, rótulo, um ponto de ajuste), um
/// traço com pressão e um comentário com resposta.
#[test]
fn a_format_8_file_still_opens_with_the_default_look() {
    use crate::{Anchor, End, Head, Look, Route};
    let set = BoardSet::from_bytes(V8_BYTES).expect("o formato 8 lê-se");
    let b = &set.boards()[0];
    assert_eq!(
        (b.name.as_str(), b.camera.zoom, b.sketch),
        ("Retro8", 0.5, true)
    );
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 5);
    assert_eq!(els[0].as_frame().unwrap().name, "Sprint");
    let rect = els[1].shape().unwrap();
    assert_eq!(els[1].frame, Some(els[0].id));
    assert!(rect.style.sketch && rect.style.round);
    assert_eq!(rect.text.as_str(), "caixa");
    assert_eq!(els[2].shape().unwrap().kind, ShapeType::Sticky);
    let c = els[3].connector().unwrap();
    assert_eq!(c.route, Route::Curved);
    assert_eq!(c.heads, [Head::Bar, Head::Triangle]);
    assert_eq!(c.label, "sim");
    assert_eq!(c.waypoints, vec![[260.0, 30.0]]);
    assert!(matches!(
        c.start,
        End::Bound {
            anchor: Anchor::Fixed([1.0, 0.5]),
            ..
        }
    ));
    assert!(matches!(
        c.end,
        End::Bound {
            anchor: Anchor::Center,
            ..
        }
    ));
    assert!(els[4].ink().unwrap().pressure);
    assert_eq!(b.team.threads.len(), 1);
    assert!(els.iter().all(|e| e.style().look == Look::default()));
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 8 (`3b4ec7784`).
const V8_BYTES: &[u8] = include_bytes!("../fixtures/format_v8.bin");

/// ⭐ Um ficheiro do formato 9 (W9: mensagens e versões sem id) continua a abrir — gravado pelo build
/// de `c9f29c3e6` (teste temporário que escreveu `to_bytes`, apagado): «Retro9» (zoom 0,5) com um
/// frame «Sprint» escondido da apresentação e uma nota dentro; três fios abertos e o do meio apagado
/// (o contador antigo ia em 3 para 2 fios), um de 3 mensagens com a 3.ª editada, um resolvido de 2;
/// duas versões. «Outro»: um fio de 2 e uma versão. Os ids saem do contador: cada fio, as mensagens
/// dele pela ordem, as versões no fim.
#[test]
fn a_format_9_file_still_opens_and_its_team_gets_ids() {
    use crate::{ElementId, MessageId, ThreadId, VersionId};
    let set = BoardSet::from_bytes(V9_BYTES).expect("o formato 9 lê-se");
    let [a, b] = set.boards() else {
        panic!("dois quadros");
    };
    assert_eq!((a.name.as_str(), a.camera.zoom), ("Retro9", 0.5));
    assert_eq!(a.doc.live_len(), 2);
    let frame = ElementId(1);
    assert_eq!(a.doc.get(frame).unwrap().as_frame().unwrap().name, "Sprint");
    assert!(a.team.hidden_slides.contains(&frame));
    let ids = |t: &crate::Thread| -> Vec<u64> { t.messages.iter().map(|m| m.id.0).collect() };
    let [t1, t3] = &a.team.threads[..] else {
        panic!("dois fios");
    };
    assert_eq!((t1.id, ids(t1)), (ThreadId(1), vec![2, 3, 4]));
    let texts: Vec<_> = t1
        .messages
        .iter()
        .map(|m| (m.author.as_str(), m.text.as_str(), m.at, m.edited))
        .collect();
    assert_eq!(
        texts,
        [
            ("Ana", "aqui", 1000, false),
            ("Rui", "sim", 1010, false),
            ("Ana", "ok!", 1020, true)
        ]
    );
    assert_eq!(t1.read_by, ["Ana", "Rui"]);
    assert!(matches!(
        t1.pin,
        crate::Pin::Element {
            id: ElementId(2),
            ..
        }
    ));
    assert_eq!((t3.id, ids(t3)), (ThreadId(5), vec![6, 7]));
    assert!(t3.resolved);
    let vs: Vec<_> = a
        .team
        .versions
        .iter()
        .map(|v| (v.id, v.name.as_str(), v.at))
        .collect();
    assert_eq!(vs, [(VersionId(8), "v1", 2000), (VersionId(9), "v2", 2100)]);
    let saved = a.team.versions[0]
        .read()
        .expect("a versão do formato 9 abre");
    assert_eq!(saved.doc.live_len(), 2);
    assert!(saved.team.hidden_slides.contains(&frame) && saved.team.threads.is_empty());
    assert_eq!(b.team.threads[0].id, ThreadId(1));
    assert_eq!(b.team.threads[0].messages[1].id, MessageId(3));
    assert_eq!(b.team.versions[0].id, VersionId(4));
    assert_eq!(
        (a.team.clone().mint(), b.team.clone().mint()),
        (10, 5),
        "o contador fica acima de todos"
    );
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        FORMAT_VERSION,
        "regrava-se no formato actual"
    );
    assert_eq!(
        BoardSet::from_bytes(&bytes).unwrap(),
        set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 9 (`c9f29c3e6`).
const V9_BYTES: &[u8] = include_bytes!("../fixtures/format_v9.bin");
