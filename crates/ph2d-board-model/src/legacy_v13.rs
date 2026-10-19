//! **Os formatos 13 e 14** (W15–W23): o de hoje com a seta de um rótulo único a meio — no 13 sem o
//! segmento do meio do cotovelo (`Connector::mids`, W23 B5(1)), no 14 com ele; o 15 trocou o rótulo
//! pelos textos ao longo da linha (`Connector::captions`, W23 B5(2)). Só a variante `Connector` do
//! elemento muda: o resto lê-se com as structs de hoje, e as do elemento são genéricas na seta. Os
//! leitores do 9 ao 12 lêem a seta pela [`ConnectorV13`] (antes do 13 ela já era esta) e o elemento
//! pelo [`KindV13`] (as variantes novas só se acrescentam no fim).

use serde::Deserialize;

use super::{DocOld, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, Connector, Element, ElementId, ElementKind, End, Files,
    FracKey, Frame, Head, Image, Ink, Org, Route, Shape, Style, Team, TextTurn,
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

/// A seta do formato 14.
#[derive(Deserialize)]
pub(super) struct ConnectorV14 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: Style,
    label: String,
    waypoints: Vec<[f64; 2]>,
    mids: Vec<Option<f64>>,
}

/// O que um elemento É, nos formatos 9–14 (o 12 lê a imagem pela dele), com a seta `C` do formato.
#[derive(Deserialize)]
pub(super) enum KindOld<C> {
    Shape(Shape),
    Connector(Box<C>),
    Ink(Box<Ink>),
    Frame(Frame),
    Image(Box<Image>),
    Embed(Box<crate::Embed>),
}

pub(super) type KindV13 = KindOld<ConnectorV13>;

impl From<ConnectorV13> for Connector {
    fn from(c: ConnectorV13) -> Self {
        ConnectorV14 {
            start: c.start,
            end: c.end,
            route: c.route,
            heads: c.heads,
            style: c.style,
            label: c.label,
            waypoints: c.waypoints,
            mids: Vec::new(),
        }
        .into()
    }
}

/// O rótulo único fica UM texto a meio, deitado na horizontal como o artista o via.
impl From<ConnectorV14> for Connector {
    fn from(c: ConnectorV14) -> Self {
        Connector {
            start: c.start,
            end: c.end,
            route: c.route,
            heads: c.heads,
            style: c.style,
            captions: Connector::captions_of(c.label),
            turn: TextTurn::Level,
            waypoints: c.waypoints,
            mids: c.mids,
        }
    }
}

impl<C: Into<Connector>> From<KindOld<C>> for ElementKind {
    fn from(k: KindOld<C>) -> Self {
        match k {
            KindOld::Shape(s) => ElementKind::Shape(s),
            KindOld::Connector(c) => ElementKind::Connector(Box::new((*c).into())),
            KindOld::Ink(i) => ElementKind::Ink(i),
            KindOld::Frame(f) => ElementKind::Frame(f),
            KindOld::Image(i) => ElementKind::Image(i),
            KindOld::Embed(e) => ElementKind::Embed(e),
        }
    }
}

/// O elemento dos formatos 11–14.
#[derive(Deserialize)]
pub(super) struct ElementOld<C> {
    id: ElementId,
    kind: KindOld<C>,
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

pub(super) type ElementV13 = ElementOld<ConnectorV13>;

pub(super) fn element_v13(e: ElementV13) -> Element {
    element_old(e)
}

fn element_old<C: Into<Connector>>(e: ElementOld<C>) -> Element {
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
struct BoardOld<C> {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementOld<C>>,
    sketch: bool,
    team: Team,
    files: Files,
}

#[derive(Deserialize)]
struct SetOld<C> {
    _version: u32,
    boards: Vec<BoardOld<C>>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 13, já no formato actual.
pub(crate) fn read_v13(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ConnectorV13>(bytes)
}

/// Os bytes de um `BoardSet` no formato 14, já no formato actual.
pub(crate) fn read_v14(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ConnectorV14>(bytes)
}

fn read_old<C: for<'de> Deserialize<'de> + Into<Connector>>(
    bytes: &[u8],
) -> Result<BoardSet, String> {
    let old: SetOld<C> = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &element_old::<C>);
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
