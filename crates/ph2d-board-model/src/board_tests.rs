use super::{BoardSet, Camera, FORMAT_VERSION};
use crate::{BoardOp, Element, Rgba, RichText, Shape, ShapeType, Style};

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
    c.captions = Connector::captions_of("sim".into());
    let arrow = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    BoardOp::Put(arrow).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    assert_eq!(FORMAT_VERSION, 19);
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

/// ⭐ B7 — a ordem de desenho é a das chaves (z, depois id) com os vídeos por cima, num quadro de z BARALHADO,
/// com empates (duas pessoas criaram no mesmo sítio) e chaves além dos 8 bytes do prefixo.
#[test]
fn the_paint_order_is_the_key_order_on_a_shuffled_board() {
    use crate::{Element, ElementId, FracKey};
    let mut set = BoardSet::default();
    let id = set.create("z".into());
    let b = set.get_mut(id).unwrap();
    let mut zs: Vec<FracKey> = vec![FracKey::between(None, None)];
    for _ in 0..300 {
        let last = zs.last().cloned();
        zs.push(FracKey::between(last.as_ref(), None));
    }
    let (mut lo, hi) = (zs[10].clone(), zs[11].clone());
    for _ in 0..30 {
        lo = FracKey::between(Some(&lo), Some(&hi));
        zs.push(lo.clone());
    }
    let n = zs.len();
    let mut s = 0x2545_F491_4F6C_DD1D_u64;
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let z = zs[(s % n as u64) as usize].clone(); // com repetição: empates de z
        let el = if i % 50 == 7 {
            let embed = crate::Embed {
                link: "https://youtu.be/dQw4w9WgXcQ".into(),
                video: crate::Video::YouTube {
                    id: "dQw4w9WgXcQ".into(),
                    start: 0,
                    vertical: false,
                },
                style: Style::new(None, None, Rgba([0, 0, 0, 255])),
            };
            Element::new_embed(ElementId(i as u64 + 1), z, embed, [0.0, 0.0, 10.0, 10.0])
        } else {
            let shape = Shape {
                kind: ShapeType::Rectangle,
                style: Style::new(None, None, Rgba([0, 0, 0, 255])),
                text: RichText::plain(""),
            };
            Element::new_shape(ElementId(i as u64 + 1), z, shape, [0.0, 0.0, 10.0, 10.0])
        };
        BoardOp::Put(el).apply(&mut b.doc);
    }
    // Chaves que partilham os 8 bytes do prefixo e só diferem na cauda, com os ids AO CONTRÁRIO delas: só o
    // desempate pelo texto as põe na ordem.
    let (mut lo, hi) = (zs[20].clone(), zs[21].clone());
    let mut tail = Vec::new();
    for _ in 0..200 {
        lo = FracKey::between(Some(&lo), Some(&hi));
        tail.push(lo.clone());
    }
    let long: Vec<FracKey> = tail
        .into_iter()
        .filter(|k| k.as_str().len() > 9)
        .take(20)
        .collect();
    assert!(long.len() == 20 && long[0].prefix() == long[19].prefix() && long[0] < long[19]);
    for (j, z) in long.into_iter().enumerate() {
        let shape = Shape {
            kind: ShapeType::Rectangle,
            style: Style::new(None, None, Rgba([0, 0, 0, 255])),
            text: RichText::plain(""),
        };
        let el = Element::new_shape(
            ElementId(10_000 - j as u64),
            z,
            shape,
            [0.0, 0.0, 10.0, 10.0],
        );
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let mut want: Vec<&Element> = b.doc.live().collect();
    want.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)));
    let ids = |v: &[&Element]| v.iter().map(|e| e.id).collect::<Vec<_>>();
    assert_eq!(ids(&b.doc.live_in_z_order()), ids(&want));
    want.sort_by_key(|e| e.embed().is_some());
    let paint = b.doc.live_in_paint_order();
    assert_eq!(ids(&paint), ids(&want));
    assert!(
        paint.last().unwrap().embed().is_some(),
        "o vídeo fica por cima"
    );
}

/// O `Style` cabe nos 40 bytes: a [`crate::BirthScale`] é um `i8` para entrar na folga, não crescê-lo.
#[test]
fn the_style_stays_in_forty_bytes() {
    let n = std::mem::size_of::<Style>();
    assert!(n <= 40, "{n}");
}
