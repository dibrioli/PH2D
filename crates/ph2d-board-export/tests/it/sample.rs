//! Um quadro de amostra com tudo o que o export desenha (W8): frames, notas com texto, etiquetas,
//! carimbos e votos (que NÃO saem), uma forma em rascunho, uma rodada a meia opacidade, um traço da
//! caneta e uma seta com rótulo.

use ph2d_board_model::{
    Anchor, Board, BoardOp, BoardSet, Connector, Element, ElementId, End, Frame, Ink, Pen, Rgba,
    RichText, Route, Shape, ShapeType, Stamp, Style, Tag,
};

pub(crate) const INK: Rgba = Rgba([30, 30, 30, 255]);

pub(crate) fn put(b: &mut Board, el: Element) -> ElementId {
    let id = el.id;
    BoardOp::Put(el).apply(&mut b.doc);
    id
}

pub(crate) fn shape(b: &mut Board, kind: ShapeType, text: &str, bx: [f64; 4]) -> Element {
    let fill = if kind.is_note() {
        Some(Rgba([255, 249, 177, 255]))
    } else {
        Some(Rgba([200, 230, 255, 255]))
    };
    let stroke = (!kind.is_note()).then_some(INK);
    Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Shape {
            kind,
            style: Style::new(fill, stroke, INK),
            text: RichText::plain(text),
        },
        bx,
    )
}

/// O quadro de amostra e os ids dos dois frames.
pub(crate) fn sample() -> (BoardSet, [ElementId; 2]) {
    let mut set = BoardSet::default();
    let a = set.create("Launch".into());
    let b = set.get_mut(a).unwrap();
    let el = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Frame {
            name: "Problem".into(),
            style: Style::new(Some(Rgba([255; 4])), None, INK),
        },
        [0.0, 0.0, 960.0, 540.0],
    );
    let f1 = put(b, el);
    let mut n = shape(
        b,
        ShapeType::Sticky,
        "Slow builds ação",
        [40.0, 40.0, 199.0, 199.0],
    );
    n.frame = Some(f1);
    n.add_stamp(Stamp::Heart, 2);
    n.tags_mut().push(Tag {
        name: "urgent".into(),
        color: Rgba([240, 147, 157, 255]),
    });
    let note = put(b, n);
    // W13 — uma sessão de votação mostrada no quadro: a exportação nunca pinta os selos (Miro).
    b.team
        .add_poll(std::collections::BTreeMap::from([(note, 3)]), "", 1, 0);
    let mut sk = shape(b, ShapeType::Diamond, "Sketch", [320.0, 60.0, 200.0, 140.0]);
    sk.frame = Some(f1);
    if let Some(s) = sk.shape_mut() {
        s.style.sketch = true;
    }
    put(b, sk);
    let mut rot = shape(b, ShapeType::Ellipse, "Half", [600.0, 80.0, 220.0, 120.0]);
    rot.frame = Some(f1);
    rot.angle = 0.4;
    if let Some(s) = rot.shape_mut() {
        s.style.opacity = 50;
    }
    let half = put(b, rot);
    let pen = Style::new(None, Some(Rgba([66, 98, 255, 255])), INK);
    let pts: Vec<[f64; 3]> = (0..40)
        .map(|i| {
            let t = f64::from(i) / 39.0;
            [60.0 + 400.0 * t, 400.0 + 40.0 * (t * 9.0).sin(), 0.5]
        })
        .collect();
    let (ink, bx) = Ink::from_world(
        &pts,
        Style {
            stroke_width: 4.0,
            ..pen
        },
        Pen::Pen,
        false,
    );
    let mut stroke = Element::new_ink(b.doc.mint_id(), b.doc.z_on_top(), ink, bx);
    stroke.frame = Some(f1);
    put(b, stroke);
    let mut c = Connector::new(
        End::Bound {
            target: note,
            anchor: Anchor::Center,
        },
        End::Bound {
            target: half,
            anchor: Anchor::Center,
        },
        Route::Curved,
        Style::new(None, Some(INK), INK),
    );
    c.captions = Connector::captions_of("next".into());
    let el = Element::new_connector(b.doc.mint_id(), b.doc.z_on_top(), c);
    put(b, el);
    let el = Element::new_frame(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Frame {
            name: "Plan".into(),
            style: Style::new(Some(Rgba([230, 245, 230, 255])), None, INK),
        },
        [1100.0, 0.0, 600.0, 540.0],
    );
    let f2 = put(b, el);
    let mut r = shape(
        b,
        ShapeType::Cylinder,
        "Cache",
        [1200.0, 120.0, 180.0, 140.0],
    );
    r.frame = Some(f2);
    put(b, r);
    (set, [f1, f2])
}
