//! **Ler os formatos antigos** e trazê-los para o actual. O postcard é posicional: cada formato só
//! se lê com as structs DELE.
//!
//! - 1 (W0): rectângulos cheios, sem rotação — cada um vira uma [`Shape`] rectangular com o mesmo
//!   preenchimento, sem contorno nem texto, e ângulo zero.
//! - 2 (W1–W2): o texto das formas era uma `String` — vira um [`RichText`] sem trechos.
//! - 3 (W3): sem rascunho (`Style::sketch`, `Board::sketch`) nem traços da caneta — tudo final.
//! - 4 (W4): sem os campos de organizar (`frame`, `groups`, `locked`, `link`, `tags`) — tudo solto.
//! - 5 (W5): sem o nó de mapa mental (`Org::mind`, W6) — nada é mapa.
//! - 6 (W6): sem os votos nem os carimbos (`Org::votes`, `Org::stamps`, W7) — nenhum.
//! - 7 (W7): sem a equipa (`Board::team`: comentários, versões, frames fora da apresentação, W8).
//! - 8 (W8): sem o aspecto do Excalidraw (`Style::look`, `Org::seed`, W9) — o de nascença. Do 4 ao 8 o estilo
//!   lê-se com [`StyleV8`] (pelo [`KindV8`]).
//! - 9 (W9): mensagens e versões sem id (W11) — o leitor dá-lhos pelo contador ([`TeamV9`], que o 8
//!   também usa).
//! - 10 (W11–W12): os votos da última votação em cada elemento (`Org::votes`) e a equipa sem histórico
//!   (W13, ADR-0183): do 7 ao 10 os votos viram UMA sessão, mostrada ([`old_votes`]).
//!
//! Do 1 ao 3 o elemento, o documento e o quadro têm a mesma forma a menos do que o elemento É:
//! [`Old`] é genérico nisso.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    Board, BoardDoc, BoardId, BoardSet, Camera, Connector, Dash, Element, ElementId, ElementKind,
    End, FracKey, Head, Rgba, RichText, Route, Shape, ShapeType, Style, Team,
};

#[path = "legacy_v10.rs"]
mod v10;
#[path = "legacy_v8.rs"]
mod v8;
#[path = "legacy_v9.rs"]
mod v9;
pub(crate) use v8::read_v8;
use v8::{ElementV8, KindV8, element_v8};
use v9::TeamV9;
pub(crate) use v9::read_v9;
pub(crate) use v10::read_v10;

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
            look: crate::Look::default(),
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
pub(super) struct DocOld<E> {
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
            doc: doc_old(b.doc, &element),
            sketch: false,
            team: Team::default(),
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

/// Os votos (`Org::votes`, formatos 7–10) dos elementos vivos: `votes` diz de um elemento antigo se
/// está apagado e quantos votos tem.
pub(super) fn old_votes<E>(
    d: &DocOld<E>,
    votes: impl Fn(&E) -> (bool, u32),
) -> BTreeMap<ElementId, u32> {
    d.elements
        .iter()
        .filter_map(|(&id, e)| match votes(e) {
            (false, n) if n > 0 => Some((id, n)),
            _ => None,
        })
        .collect()
}

pub(super) fn doc_old<E>(d: DocOld<E>, element: &impl Fn(E) -> Element) -> BoardDoc {
    BoardDoc {
        elements: d
            .elements
            .into_iter()
            .map(|(id, e)| (id, element(e)))
            .collect(),
        next_id: d.next_id,
        top_z: d.top_z,
        rev: Default::default(),
        // Os formatos 1–4 não tinham frames.
        frames: Default::default(),
        minds: Default::default(),
        session: Default::default(),
    }
}

/// O quadro do formato 4: já com o rascunho (`sketch`).
#[derive(Deserialize)]
struct BoardV4 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementOld<KindV8>>,
    sketch: bool,
}

#[derive(Deserialize)]
struct SetV4 {
    _version: u32,
    boards: Vec<BoardV4>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 4 (W4), já no formato actual: o elemento era o de hoje sem
/// os campos de organizar (W5) — tudo nasce solto, sem grupo, destravado, sem link nem etiquetas.
/// ⚠️ O `ElementKind` lê-se com o enum ACTUAL: o `Frame` entrou no FIM, e os índices 0..=2 do
/// formato 4 continuam a ser `Shape`, `Connector`, `Ink`.
pub(crate) fn read_v4(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV4 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| Board {
            id: b.id,
            name: b.name,
            camera: b.camera,
            doc: doc_old(b.doc, &element_old),
            sketch: b.sketch,
            team: Team::default(),
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

/// O `Org` do formato 5: sem o nó de mapa mental.
#[derive(Deserialize)]
struct OrgV5 {
    groups: Vec<crate::GroupId>,
    locked: bool,
    link: Option<ElementId>,
    tags: Vec<crate::Tag>,
}

/// O elemento do formato 5: o de hoje com o `Org` antigo.
#[derive(Deserialize)]
struct ElementV5 {
    id: ElementId,
    kind: KindV8,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    frame: Option<ElementId>,
    org: Option<Box<OrgV5>>,
    version: u64,
    nonce: u32,
    deleted: bool,
}

#[derive(Deserialize)]
struct BoardV5 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV5>,
    sketch: bool,
}

#[derive(Deserialize)]
struct SetV5 {
    _version: u32,
    boards: Vec<BoardV5>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 5 (W5), já no formato actual: nenhum elemento é nó de mapa.
pub(crate) fn read_v5(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV5 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &element_v5);
            doc.reindex();
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team: Team::default(),
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

fn element_v5(e: ElementV5) -> Element {
    let mut el = Element::bare(e.id, e.z, e.kind.into(), [e.x, e.y, e.w, e.h]);
    el.angle = e.angle;
    el.frame = e.frame;
    el.org = e.org.map(|o| {
        Box::new(crate::Org {
            groups: o.groups,
            locked: o.locked,
            link: o.link,
            tags: o.tags,
            mind: None,
            stamps: Vec::new(),
            seed: None,
        })
    });
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
}

/// O `Org` do formato 6: sem os votos nem os carimbos.
#[derive(Deserialize)]
struct OrgV6 {
    groups: Vec<crate::GroupId>,
    locked: bool,
    link: Option<ElementId>,
    tags: Vec<crate::Tag>,
    mind: Option<crate::Mind>,
}

/// O elemento do formato 6: o de hoje com o `Org` antigo.
#[derive(Deserialize)]
struct ElementV6 {
    id: ElementId,
    kind: KindV8,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    frame: Option<ElementId>,
    org: Option<Box<OrgV6>>,
    version: u64,
    nonce: u32,
    deleted: bool,
}

#[derive(Deserialize)]
struct BoardV6 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV6>,
    sketch: bool,
}

#[derive(Deserialize)]
struct SetV6 {
    _version: u32,
    boards: Vec<BoardV6>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 6 (W6), já no formato actual: nenhum voto, nenhum carimbo.
pub(crate) fn read_v6(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV6 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &element_v6);
            doc.reindex();
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team: Team::default(),
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

fn element_v6(e: ElementV6) -> Element {
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
            stamps: Vec::new(),
            seed: None,
        })
    });
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
}

fn element_old<K: Into<ElementKind>>(e: ElementOld<K>) -> Element {
    let mut el = Element::bare(e.id, e.z, e.kind.into(), [e.x, e.y, e.w, e.h]);
    el.angle = e.angle;
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
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
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style,
        text: RichText::default(),
    };
    let mut el = Element::bare(e.id, e.z, ElementKind::Shape(shape), [e.x, e.y, e.w, e.h]);
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
}

#[derive(Deserialize)]
struct BoardV7 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV8>,
    sketch: bool,
}

#[derive(Deserialize)]
struct SetV7 {
    _version: u32,
    boards: Vec<BoardV7>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 7 (W7), já no formato actual: sem comentários nem versões.
pub(crate) fn read_v7(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV7 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let votes = v8::votes_v8(&b.doc);
            let mut doc = doc_old(b.doc, &element_v8);
            doc.reindex();
            let mut team = Team::default();
            team.adopt_old_votes(votes);
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team,
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
