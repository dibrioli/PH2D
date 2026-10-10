use super::*;
use crate::{
    Anchor, Connector, Dash, ElementId, End, FillStyle, FracKey, Frame, GroupId, Head, Ink, Letter,
    Look, Mind, Pen, Rgba, RichText, Route, Shape, ShapeType, Side, Stamp, StampCount, Style, Tag,
    TextAlign, VAlign,
};

const ID: ElementId = ElementId(42);

fn plain_style() -> Style {
    Style::new(None, None, Rgba([0, 0, 0, 255]))
}

/// Todos os campos FORA do valor de nascença.
fn rich_style() -> Style {
    Style {
        fill: Some(Rgba([1, 2, 3, 4])),
        stroke: Some(Rgba([5, 6, 7, 8])),
        stroke_width: 3.25,
        dash: Dash::Dotted,
        round: true,
        opacity: 40,
        text_color: Rgba([9, 10, 11, 12]),
        font_size: 33.5,
        sketch: true,
        look: Look {
            fill_style: FillStyle::CrossHatch,
            cartoonist: true,
            align: TextAlign::Right,
            valign: VAlign::Bottom,
            letter: Letter::Hand,
        },
    }
}

fn rich_org() -> Org {
    Org {
        groups: vec![GroupId(5), GroupId(6)],
        locked: true,
        link: Some(ElementId(9)),
        tags: vec![Tag {
            name: "t".into(),
            color: Rgba([1, 1, 1, 255]),
        }],
        mind: Some(Mind {
            parent: Some(ElementId(3)),
            order: FracKey::between(None, None),
            side: Side::Before,
            color: Rgba([2, 2, 2, 255]),
            collapsed: true,
            manual: true,
            vertical: true,
            hidden: false,
        }),
        votes: 3,
        stamps: vec![StampCount {
            stamp: Stamp::Heart,
            count: 2,
        }],
        seed: Some(77),
    }
}

fn z0() -> FracKey {
    FracKey::between(None, None)
}

/// Um elemento de cada tipo, de nascença e com tudo mudado.
fn pairs() -> Vec<(Element, Element)> {
    let shape = |st: Style, kind, text: &str| Shape {
        kind,
        style: st,
        text: RichText::plain(text),
    };
    let conn = |st: Style, rich: bool| Connector {
        start: if rich {
            End::Bound {
                target: ElementId(7),
                anchor: Anchor::Fixed([0.25, 0.75]),
            }
        } else {
            End::Free([0.0, 0.0])
        },
        end: End::Free(if rich { [9.0, 8.0] } else { [1.0, 1.0] }),
        route: if rich { Route::Curved } else { Route::Straight },
        heads: if rich {
            [Head::Diamond, Head::Many]
        } else {
            [Head::None, Head::Arrow]
        },
        style: st,
        label: if rich {
            "rótulo".into()
        } else {
            String::new()
        },
        waypoints: if rich { vec![[4.0, 5.0]] } else { vec![] },
    };
    let ink = |st: Style, rich: bool| Ink {
        style: st,
        pen: if rich { Pen::Highlighter } else { Pen::Pen },
        points: if rich {
            vec![[1.0, 2.0, 0.5], [3.0, 4.0, 0.25]]
        } else {
            vec![[0.0, 0.0, 1.0]]
        },
        base: if rich { [3.0, 4.0] } else { [0.0, 0.0] },
        pressure: rich,
    };
    let frame = |st: Style, name: &str| Frame {
        name: name.into(),
        style: st,
    };
    let bx = [0.0, 0.0, 10.0, 10.0];
    let plain = [
        Element::new_shape(ID, z0(), shape(plain_style(), ShapeType::Rectangle, ""), bx),
        Element::new_connector(ID, z0(), conn(plain_style(), false)),
        Element::new_ink(ID, z0(), ink(plain_style(), false), bx),
        Element::new_frame(ID, z0(), frame(plain_style(), ""), bx),
    ];
    let rich = [
        Element::new_shape(ID, z0(), shape(rich_style(), ShapeType::Hexagon, "olá"), bx),
        Element::new_connector(ID, z0(), conn(rich_style(), true)),
        Element::new_ink(ID, z0(), ink(rich_style(), true), bx),
        Element::new_frame(ID, z0(), frame(rich_style(), "F"), bx),
    ]
    .map(|mut el| {
        (el.x, el.y, el.w, el.h) = (1.5, -2.5, 30.0, 40.0);
        el.angle = 0.75;
        el.z = FracKey::between(Some(&z0()), None);
        el.frame = Some(ElementId(11));
        el.deleted = true;
        el.org = Some(Box::new(rich_org()));
        el
    });
    plain.into_iter().zip(rich).collect()
}

/// Escrever num elemento TODOS os registos de outro (do mesmo tipo) dá esse outro — nos dois
/// sentidos (o de volta apaga o `org`: todo por omissão é `None`). Um campo que nenhum registo leva
/// fica de fora e isto fica vermelho.
#[test]
fn writing_every_register_of_an_element_reproduces_it() {
    for (plain, rich) in pairs() {
        for (from, to) in [(&plain, &rich), (&rich, &plain)] {
            let mut el = from.clone();
            for f in Field::ALL {
                if let Some(v) = to.read(f) {
                    assert!(el.write(f, &v), "{f:?} não se escreveu");
                }
            }
            assert_eq!(&el, to);
            assert!(el.changed_fields(to).is_empty());
        }
        assert!(!plain.changed_fields(&rich).is_empty());
    }
}

/// Um registo que o tipo não tem não se lê nem se escreve (o texto de uma seta, a ponta de uma forma).
#[test]
fn a_register_of_another_kind_is_absent() {
    let (shape, _) = pairs().remove(0);
    assert!(shape.read(Field::Start).is_none() && shape.read(Field::InkBody).is_none());
    let mut el = shape.clone();
    let start = pairs()[1].1.read(Field::Start).unwrap();
    assert!(!el.write(Field::Start, &start));
    assert!(
        !el.write(Field::Box, &[1, 2, 3]),
        "bytes que não são uma caixa"
    );
    assert_eq!(el, shape);
}

/// Mover e mudar a cor tocam registos DIFERENTES; mover e redimensionar pelo canto, o MESMO.
#[test]
fn moving_and_recolouring_are_different_registers() {
    let (a, _) = pairs().remove(0);
    let mut moved = a.clone();
    moved.translate([5.0, 0.0]);
    let mut painted = a.clone();
    painted.style_mut().fill = Some(Rgba([200, 0, 0, 255]));
    assert_eq!(a.changed_fields(&moved), vec![Field::Box]);
    assert_eq!(a.changed_fields(&painted), vec![Field::Fill]);
}

/// ⛔ A lista é TODOS os campos: um campo novo num destes tipos deixa de compilar aqui — ponha-o num
/// registo (`fields.rs`) e no `rich_*` acima.
#[test]
fn every_field_of_the_model_is_named_here() {
    for (_, el) in pairs() {
        let Element {
            id: _,
            kind,
            x: _,
            y: _,
            w: _,
            h: _,
            angle: _,
            z: _,
            frame: _,
            org,
            version: _,
            nonce: _,
            deleted: _,
        } = &el;
        let style = match kind {
            ElementKind::Shape(Shape {
                kind: _,
                style,
                text: _,
            }) => style,
            ElementKind::Connector(c) => {
                let Connector {
                    start: _,
                    end: _,
                    route: _,
                    heads: [_, _],
                    style,
                    label: _,
                    waypoints: _,
                } = &**c;
                style
            }
            ElementKind::Ink(i) => {
                let Ink {
                    style,
                    pen: _,
                    points: _,
                    base: _,
                    pressure: _,
                } = &**i;
                style
            }
            ElementKind::Frame(Frame { name: _, style }) => style,
        };
        let Style {
            fill: _,
            stroke: _,
            stroke_width: _,
            dash: _,
            round: _,
            opacity: _,
            text_color: _,
            font_size: _,
            sketch: _,
            look:
                Look {
                    fill_style: _,
                    cartoonist: _,
                    align: _,
                    valign: _,
                    letter: _,
                },
        } = style;
        let Org {
            groups: _,
            locked: _,
            link: _,
            tags: _,
            mind,
            votes: _,
            stamps: _,
            seed: _,
        } = org.as_deref().unwrap();
        let Mind {
            parent: _,
            order: _,
            side: _,
            color: _,
            collapsed: _,
            manual: _,
            vertical: _,
            hidden: _,
        } = mind.as_ref().unwrap();
    }
    let mut all = Field::ALL.to_vec();
    all.dedup();
    assert_eq!(all.len(), Field::ALL.len());
}
