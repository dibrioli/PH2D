//! **Os formatos 13 e 14** (W15–W23): o de hoje com a seta de um rótulo único a meio — no 13 sem o
//! segmento do meio do cotovelo (`Connector::mids`, W23 B5(1)), no 14 com ele; o 15 trocou o rótulo
//! pelos textos ao longo da linha (`Connector::captions`, W23 B5(2)). As structs do elemento são
//! genéricas na seta, e as variantes lêem-se pelas do formato 16 (`legacy_v16.rs`); o quadro, na
//! equipa. Os leitores do 9 ao 12 lêem a seta pela [`ConnectorV13`] (antes do 13 ela já era esta) e
//! o elemento pelo [`KindV13`] (as variantes novas só se acrescentam no fim); o 15 e o 16, pelo
//! [`read_old`] com a seta do 16.

use serde::Deserialize;

use super::v16::{EmbedV16, FrameV16, ImageV16, InkV16, ShapeV16, StyleV16};
use super::{DocOld, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, Connector, Element, ElementId, ElementKind, End, Files,
    FracKey, Head, Org, Route, Team, TextTurn,
};

/// A seta dos formatos 9–13.
#[derive(Deserialize)]
pub(super) struct ConnectorV13 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: StyleV16,
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
    style: StyleV16,
    label: String,
    waypoints: Vec<[f64; 2]>,
    mids: Vec<Option<f64>>,
}

/// O que um elemento É, nos formatos 9–16 (o 12 lê a imagem pela dele), com a seta `C` do formato.
#[derive(Deserialize)]
pub(super) enum KindOld<C> {
    Shape(ShapeV16),
    Connector(Box<C>),
    Ink(Box<InkV16>),
    Frame(FrameV16),
    Image(Box<ImageV16>),
    Embed(Box<EmbedV16>),
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
            style: c.style.into(),
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
            KindOld::Shape(s) => ElementKind::Shape(s.into()),
            KindOld::Connector(c) => ElementKind::Connector(Box::new((*c).into())),
            KindOld::Ink(i) => ElementKind::Ink(Box::new((*i).into())),
            KindOld::Frame(f) => ElementKind::Frame(f.into()),
            KindOld::Image(i) => ElementKind::Image(Box::new((*i).into())),
            KindOld::Embed(e) => ElementKind::Embed(Box::new((*e).into())),
        }
    }
}

/// O elemento dos formatos 11–16.
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

/// O quadro dos formatos 12–16, com a seta `C` e a equipa `T` do formato.
#[derive(Deserialize)]
struct BoardOld<C, T> {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementOld<C>>,
    sketch: bool,
    team: T,
    files: Files,
}

#[derive(Deserialize)]
struct SetOld<C, T> {
    _version: u32,
    boards: Vec<BoardOld<C, T>>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 13, já no formato actual.
pub(crate) fn read_v13(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ConnectorV13, super::TeamV15>(bytes)
}

/// Os bytes de um `BoardSet` no formato 14, já no formato actual.
pub(crate) fn read_v14(bytes: &[u8]) -> Result<BoardSet, String> {
    read_old::<ConnectorV14, super::TeamV15>(bytes)
}

/// Os bytes de um `BoardSet` dos formatos 13–16, já no formato actual.
pub(super) fn read_old<C, T>(bytes: &[u8]) -> Result<BoardSet, String>
where
    C: for<'de> Deserialize<'de> + Into<Connector>,
    T: for<'de> Deserialize<'de> + Into<Team>,
{
    let old: SetOld<C, T> = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
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
                team: b.team.into(),
                files: b.files,
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
