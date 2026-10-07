//! **Ler os formatos antigos** e trazê-los para o actual. O postcard é posicional: cada formato só
//! se lê com as structs DELE.
//!
//! - 1 (W0): rectângulos cheios, sem rotação — cada um vira uma [`Shape`] rectangular com o mesmo
//!   preenchimento, sem contorno nem texto, e ângulo zero.
//! - 2 (W1–W2): o texto das formas era uma `String` — vira um [`RichText`] sem trechos.
//! - 3 (W3): sem rascunho (`Style::sketch`, `Board::sketch`) nem traços da caneta — tudo final.
//!
//! Do 1 ao 3 o elemento, o documento e o quadro têm a mesma forma a menos do que o elemento É:
//! [`Old`] é genérico nisso.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    Board, BoardDoc, BoardId, BoardSet, Camera, Connector, Dash, Element, ElementId, ElementKind,
    End, FracKey, Head, Rgba, RichText, Route, Shape, ShapeType, Style,
};

/// O `Style` dos formatos 1–3 (sem `sketch`).
#[derive(Deserialize)]
struct StyleV3 {
    fill: Option<Rgba>,
    stroke: Option<Rgba>,
    stroke_width: f64,
    dash: Dash,
    round: bool,
    opacity: u8,
    text_color: Rgba,
    font_size: f64,
}

impl From<StyleV3> for Style {
    fn from(s: StyleV3) -> Self {
        Style {
            fill: s.fill,
            stroke: s.stroke,
            stroke_width: s.stroke_width,
            dash: s.dash,
            round: s.round,
            opacity: s.opacity,
            text_color: s.text_color,
            font_size: s.font_size,
            sketch: false,
        }
    }
}

/// O `Connector` dos formatos 2–3 (com o estilo antigo).
#[derive(Deserialize)]
struct ConnectorV3 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: StyleV3,
    label: String,
    waypoints: Vec<[f64; 2]>,
}

impl From<ConnectorV3> for Connector {
    fn from(c: ConnectorV3) -> Self {
        Connector {
            start: c.start,
            end: c.end,
            route: c.route,
            heads: c.heads,
            style: c.style.into(),
            label: c.label,
            waypoints: c.waypoints,
        }
    }
}

#[derive(Deserialize)]
struct ShapeV2 {
    kind: ShapeType,
    style: StyleV3,
    text: String,
}

#[derive(Deserialize)]
enum KindV2 {
    Shape(ShapeV2),
    Connector(Box<ConnectorV3>),
}

impl From<KindV2> for ElementKind {
    fn from(k: KindV2) -> Self {
        match k {
            KindV2::Shape(s) => ElementKind::Shape(Shape {
                kind: s.kind,
                style: s.style.into(),
                text: RichText::from(s.text),
            }),
            KindV2::Connector(c) => ElementKind::Connector(Box::new((*c).into())),
        }
    }
}

#[derive(Deserialize)]
struct ShapeV3 {
    kind: ShapeType,
    style: StyleV3,
    text: RichText,
}

#[derive(Deserialize)]
enum KindV3 {
    Shape(ShapeV3),
    Connector(Box<ConnectorV3>),
}

impl From<KindV3> for ElementKind {
    fn from(k: KindV3) -> Self {
        match k {
            KindV3::Shape(s) => ElementKind::Shape(Shape {
                kind: s.kind,
                style: s.style.into(),
                text: s.text,
            }),
            KindV3::Connector(c) => ElementKind::Connector(Box::new((*c).into())),
        }
    }
}

/// O elemento dos formatos 2–3, genérico no que ele É.
#[derive(Deserialize)]
struct ElementOld<K> {
    id: ElementId,
    kind: K,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    version: u64,
    nonce: u32,
    deleted: bool,
}

#[derive(Deserialize)]
struct DocOld<E> {
    elements: BTreeMap<ElementId, E>,
    next_id: u64,
    top_z: Option<FracKey>,
}

#[derive(Deserialize)]
struct BoardOld<E> {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<E>,
}

/// Um `BoardSet` dos formatos 1–3: `E` é o elemento do formato.
#[derive(Deserialize)]
struct Old<E> {
    _version: u32,
    boards: Vec<BoardOld<E>>,
    next_id: u64,
}

fn read_old<E: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    element: impl Fn(E) -> Element,
) -> Result<BoardSet, String> {
    let old: Old<E> = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| Board {
            id: b.id,
            name: b.name,
            camera: b.camera,
            doc: BoardDoc {
                elements: b
                    .doc
                    .elements
                    .into_iter()
                    .map(|(id, e)| (id, element(e)))
                    .collect(),
                next_id: b.doc.next_id,
                top_z: b.doc.top_z,
                rev: Default::default(),
            },
            sketch: false,
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

fn element_old<K: Into<ElementKind>>(e: ElementOld<K>) -> Element {
    Element {
        id: e.id,
        kind: e.kind.into(),
        x: e.x,
        y: e.y,
        w: e.w,
        h: e.h,
        angle: e.angle,
        z: e.z,
        version: e.version,
        nonce: e.nonce,
        deleted: e.deleted,
    }
}

/// Os bytes de um `BoardSet` no formato 3, já no formato actual.
pub(crate) fn read_v3(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ElementOld<KindV3>>(bytes, element_old)
}

/// Os bytes de um `BoardSet` no formato 2, já no formato actual.
pub(crate) fn read_v2(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ElementOld<KindV2>>(bytes, element_old)
}

#[derive(Deserialize)]
enum KindV1 {
    Rect { fill: Rgba },
}

#[derive(Deserialize)]
struct ElementV1 {
    id: ElementId,
    kind: KindV1,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    z: FracKey,
    version: u64,
    nonce: u32,
    deleted: bool,
}

/// Os bytes de um `BoardSet` no formato 1, já no formato actual.
pub(crate) fn read_v1(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ElementV1>(bytes, element_v1)
}

fn element_v1(e: ElementV1) -> Element {
    let KindV1::Rect { fill } = e.kind;
    // O formato 1 não tinha texto: a tinta e o tamanho são os de nascença do quadro.
    let style = Style::new(Some(fill), None, Rgba(crate::DEFAULT_INK));
    Element {
        id: e.id,
        kind: ElementKind::Shape(Shape {
            kind: ShapeType::Rectangle,
            style,
            text: RichText::default(),
        }),
        x: e.x,
        y: e.y,
        w: e.w,
        h: e.h,
        angle: 0.0,
        z: e.z,
        version: e.version,
        nonce: e.nonce,
        deleted: e.deleted,
    }
}
