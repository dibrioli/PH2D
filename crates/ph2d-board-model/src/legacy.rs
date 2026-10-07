//! **Ler os formatos antigos** e trazê-los para o actual. O postcard é posicional: cada formato só
//! se lê com as structs DELE.
//!
//! - 1 (W0): rectângulos cheios, sem rotação — cada um vira uma [`Shape`] rectangular com o mesmo
//!   preenchimento, sem contorno nem texto, e ângulo zero.
//! - 2 (W1–W2): o texto das formas era uma `String` — vira um [`RichText`] sem trechos.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    Board, BoardDoc, BoardId, BoardSet, Camera, Connector, Dash, Element, ElementId, ElementKind,
    FracKey, Rgba, RichText, Shape, ShapeType, Style,
};

#[derive(Deserialize)]
struct ShapeV2 {
    kind: ShapeType,
    style: Style,
    text: String,
}

#[derive(Deserialize)]
enum KindV2 {
    Shape(ShapeV2),
    Connector(Box<Connector>),
}

#[derive(Deserialize)]
struct ElementV2 {
    id: ElementId,
    kind: KindV2,
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
struct DocV2 {
    elements: BTreeMap<ElementId, ElementV2>,
    next_id: u64,
    top_z: Option<FracKey>,
}

#[derive(Deserialize)]
struct BoardV2 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocV2,
}

#[derive(Deserialize)]
struct SetV2 {
    _version: u32,
    boards: Vec<BoardV2>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 2, já no formato actual.
pub(crate) fn read_v2(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV2 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
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
                    .map(|(id, e)| (id, element_v2(e)))
                    .collect(),
                next_id: b.doc.next_id,
                top_z: b.doc.top_z,
                rev: Default::default(),
            },
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

fn element_v2(e: ElementV2) -> Element {
    Element {
        id: e.id,
        kind: match e.kind {
            KindV2::Shape(s) => ElementKind::Shape(Shape {
                kind: s.kind,
                style: s.style,
                text: RichText::from(s.text),
            }),
            KindV2::Connector(c) => ElementKind::Connector(c),
        },
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

#[derive(Deserialize)]
struct DocV1 {
    elements: BTreeMap<ElementId, ElementV1>,
    next_id: u64,
    top_z: Option<FracKey>,
}

#[derive(Deserialize)]
struct BoardV1 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocV1,
}

#[derive(Deserialize)]
struct SetV1 {
    _version: u32,
    boards: Vec<BoardV1>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 1, já no formato actual.
pub(crate) fn read_v1(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV1 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
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
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

fn element(e: ElementV1) -> Element {
    let KindV1::Rect { fill } = e.kind;
    Element {
        id: e.id,
        kind: ElementKind::Shape(Shape {
            kind: ShapeType::Rectangle,
            style: Style {
                fill: Some(fill),
                stroke: None,
                stroke_width: crate::DEFAULT_STROKE_WIDTH,
                dash: Dash::Solid,
                round: false,
                opacity: 100,
                // O formato 1 não tinha texto: a tinta e o tamanho são os de nascença do quadro.
                text_color: Rgba(crate::DEFAULT_INK),
                font_size: crate::DEFAULT_FONT_SIZE,
            },
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
