//! **O formato 12** (W14): o de hoje com a imagem sem filtro (`Image::filter`, W15). Só a variante
//! `Image` do elemento muda; o resto lê-se com as structs de hoje.

use serde::Deserialize;

use super::{DocOld, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, Connector, Crop, Element, ElementId, ElementKind, FileId,
    Files, FracKey, Frame, Image, ImageFilter, Ink, Org, Shape, Style,
};

#[derive(Deserialize)]
struct ImageV12 {
    file: FileId,
    style: Style,
    crop: Option<Crop>,
    flip: [bool; 2],
}

#[derive(Deserialize)]
enum KindV12 {
    Shape(Shape),
    Connector(Box<super::v13::ConnectorV13>),
    Ink(Box<Ink>),
    Frame(Frame),
    Image(Box<ImageV12>),
}

impl From<KindV12> for ElementKind {
    fn from(k: KindV12) -> Self {
        match k {
            KindV12::Shape(s) => ElementKind::Shape(s),
            KindV12::Connector(c) => ElementKind::Connector(Box::new(Connector::from(*c))),
            KindV12::Ink(i) => ElementKind::Ink(i),
            KindV12::Frame(f) => ElementKind::Frame(f),
            KindV12::Image(i) => ElementKind::Image(Box::new(Image {
                file: i.file,
                style: i.style,
                crop: i.crop,
                flip: i.flip,
                filter: ImageFilter::Original,
            })),
        }
    }
}

#[derive(Deserialize)]
struct ElementV12 {
    id: ElementId,
    kind: KindV12,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    frame: Option<ElementId>,
    org: Option<Box<Org>>,
    version: u64,
    nonce: u32,
    deleted: bool,
}

fn element_v12(e: ElementV12) -> Element {
    Element {
        id: e.id,
        kind: e.kind.into(),
        x: e.x,
        y: e.y,
        w: e.w,
        h: e.h,
        angle: e.angle,
        z: e.z,
        frame: e.frame,
        org: e.org,
        version: e.version,
        nonce: e.nonce,
        deleted: e.deleted,
    }
}

#[derive(Deserialize)]
struct BoardV12 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV12>,
    sketch: bool,
    team: super::TeamV15,
    files: Files,
}

#[derive(Deserialize)]
struct SetV12 {
    _version: u32,
    boards: Vec<BoardV12>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 12 (W14), já no formato actual.
pub(crate) fn read_v12(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV12 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &element_v12);
            doc.reindex();
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team: b.team.into(),
                files: b.files,
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
