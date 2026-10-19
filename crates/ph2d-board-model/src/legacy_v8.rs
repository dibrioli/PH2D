//! **Os formatos 4–8 do ESTILO e o 8 inteiro** (cortado do `legacy.rs` no fecho da W9, tecto de LOC):
//! o `Style` sem o `look` e o `Org` sem a semente da W9 — os tipos que os leitores 4–8 usam.

use serde::Deserialize;

use super::{DocOld, TeamV9, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, Connector, Dash, Element, ElementId, ElementKind, End,
    FracKey, Head, Rgba, RichText, Route, Shape, ShapeType, Style,
};

/// O `Style` dos formatos 4–8 (sem o `look` da W9).
#[derive(Deserialize)]
pub(super) struct StyleV8 {
    fill: Option<Rgba>,
    stroke: Option<Rgba>,
    stroke_width: f64,
    dash: Dash,
    round: bool,
    opacity: u8,
    text_color: Rgba,
    font_size: f64,
    sketch: bool,
}

impl From<StyleV8> for Style {
    fn from(s: StyleV8) -> Self {
        Style {
            fill: s.fill,
            stroke: s.stroke,
            stroke_width: s.stroke_width,
            dash: s.dash,
            round: s.round,
            opacity: s.opacity,
            text_color: s.text_color,
            font_size: s.font_size,
            sketch: s.sketch,
            look: crate::Look::default(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ShapeV8 {
    kind: ShapeType,
    style: StyleV8,
    text: RichText,
}

#[derive(Deserialize)]
pub(super) struct ConnectorV8 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: StyleV8,
    label: String,
    waypoints: Vec<[f64; 2]>,
}

#[derive(Deserialize)]
pub(super) struct InkV8 {
    style: StyleV8,
    pen: crate::Pen,
    points: Vec<[f32; 3]>,
    base: [f64; 2],
    pressure: bool,
}

#[derive(Deserialize)]
pub(super) struct FrameV8 {
    name: String,
    style: StyleV8,
}

/// O `ElementKind` dos formatos 4–8 (o `Frame` entrou no FIM na W5: os índices 0..=2 do formato 4
/// continuam a ser `Shape`, `Connector`, `Ink`).
#[derive(Deserialize)]
pub(super) enum KindV8 {
    Shape(ShapeV8),
    Connector(Box<ConnectorV8>),
    Ink(Box<InkV8>),
    Frame(FrameV8),
}

impl From<KindV8> for ElementKind {
    fn from(k: KindV8) -> Self {
        match k {
            KindV8::Shape(s) => ElementKind::Shape(Shape {
                kind: s.kind,
                style: s.style.into(),
                text: s.text,
            }),
            KindV8::Connector(c) => {
                let c = *c;
                ElementKind::Connector(Box::new(Connector {
                    start: c.start,
                    end: c.end,
                    route: c.route,
                    heads: c.heads,
                    style: c.style.into(),
                    captions: Connector::captions_of(c.label),
                    turn: crate::TextTurn::Level,
                    waypoints: c.waypoints,
                    mids: Vec::new(),
                }))
            }
            KindV8::Ink(i) => {
                let i = *i;
                ElementKind::Ink(Box::new(crate::Ink {
                    style: i.style.into(),
                    pen: i.pen,
                    points: i.points,
                    base: i.base,
                    pressure: i.pressure,
                }))
            }
            KindV8::Frame(f) => ElementKind::Frame(crate::Frame {
                name: f.name,
                style: f.style.into(),
            }),
        }
    }
}

/// O `Org` dos formatos 7–8: sem a semente importada (W9).
#[derive(Deserialize)]
pub(super) struct OrgV8 {
    groups: Vec<crate::GroupId>,
    locked: bool,
    link: Option<ElementId>,
    tags: Vec<crate::Tag>,
    mind: Option<crate::Mind>,
    votes: u32,
    stamps: Vec<crate::StampCount>,
}

/// O elemento dos formatos 7–8: o de hoje com o estilo e o `Org` antigos.
#[derive(Deserialize)]
pub(super) struct ElementV8 {
    id: ElementId,
    kind: KindV8,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    frame: Option<ElementId>,
    org: Option<Box<OrgV8>>,
    version: u64,
    nonce: u32,
    deleted: bool,
}

/// Os votos dos elementos vivos de um documento dos formatos 7–8.
pub(super) fn votes_v8(d: &DocOld<ElementV8>) -> std::collections::BTreeMap<ElementId, u32> {
    super::old_votes(d, |e| (e.deleted, e.org.as_ref().map_or(0, |o| o.votes)))
}

pub(super) fn element_v8(e: ElementV8) -> Element {
    let mut el = Element::bare(e.id, e.z, e.kind.into(), [e.x, e.y, e.w, e.h]);
    el.angle = e.angle;
    el.frame = e.frame;
    el.org = e.org.map(|o| {
        Box::new(crate::Org {
            groups: o.groups,
            locked: o.locked,
            link: o.link,
            tags: o.tags,
            mind: o.mind,
            stamps: o.stamps,
            seed: None,
        })
    });
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
}

#[derive(Deserialize)]
struct BoardV8 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV8>,
    sketch: bool,
    team: TeamV9,
}

#[derive(Deserialize)]
struct SetV8 {
    _version: u32,
    boards: Vec<BoardV8>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 8 (W8), já no formato actual: o aspecto de nascença.
pub(crate) fn read_v8(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV8 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let votes = votes_v8(&b.doc);
            let mut doc = doc_old(b.doc, &element_v8);
            doc.reindex();
            let mut team: crate::Team = b.team.into();
            team.adopt_old_votes(votes);
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team,
                files: crate::Files::new(),
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
