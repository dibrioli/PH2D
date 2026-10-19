//! **O formato 13** (W15–W23): o de hoje com a seta sem o segmento do meio do cotovelo
//! (`Connector::mids`, W23 B5(1)). Só a variante `Connector` do elemento muda; o resto lê-se com as
//! structs de hoje. Os leitores do 9 ao 12 lêem a seta pela [`ConnectorV13`] (antes do 13 ela já era
//! esta) e o elemento pelo [`KindV13`] (as variantes novas só se acrescentam no fim).

use serde::Deserialize;

use super::{DocOld, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, Connector, Element, ElementId, ElementKind, End, Files,
    FracKey, Frame, Head, Image, Ink, Org, Route, Shape, Style, Team,
};

/// A seta dos formatos 9–13.
#[derive(Deserialize)]
pub(super) struct ConnectorV13 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: Style,
    label: String,
    waypoints: Vec<[f64; 2]>,
}

/// O que um elemento É, nos formatos 9–13 (o 12 lê a imagem pela dele).
#[derive(Deserialize)]
pub(super) enum KindV13 {
    Shape(Shape),
    Connector(Box<ConnectorV13>),
    Ink(Box<Ink>),
    Frame(Frame),
    Image(Box<Image>),
    Embed(Box<crate::Embed>),
}

impl From<ConnectorV13> for Connector {
    fn from(c: ConnectorV13) -> Self {
        Connector {
            start: c.start,
            end: c.end,
            route: c.route,
            heads: c.heads,
            style: c.style,
            label: c.label,
            waypoints: c.waypoints,
            mids: Vec::new(),
        }
    }
}

impl From<KindV13> for ElementKind {
    fn from(k: KindV13) -> Self {
        match k {
            KindV13::Shape(s) => ElementKind::Shape(s),
            KindV13::Connector(c) => ElementKind::Connector(Box::new((*c).into())),
            KindV13::Ink(i) => ElementKind::Ink(i),
            KindV13::Frame(f) => ElementKind::Frame(f),
            KindV13::Image(i) => ElementKind::Image(i),
            KindV13::Embed(e) => ElementKind::Embed(e),
        }
    }
}

/// O elemento dos formatos 11–13.
#[derive(Deserialize)]
pub(super) struct ElementV13 {
    id: ElementId,
    kind: KindV13,
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

pub(super) fn element_v13(e: ElementV13) -> Element {
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
struct BoardV13 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV13>,
    sketch: bool,
    team: Team,
    files: Files,
}

#[derive(Deserialize)]
struct SetV13 {
    _version: u32,
    boards: Vec<BoardV13>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 13, já no formato actual.
pub(crate) fn read_v13(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV13 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &element_v13);
            doc.reindex();
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team: b.team,
                files: b.files,
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
