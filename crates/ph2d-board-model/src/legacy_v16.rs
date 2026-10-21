//! **O formato 16** e as variantes do elemento dos formatos 9–16: o de hoje sem `Style::birth` nem
//! `Ink::clean` (formato 17, auditoria do Sketch, doc 17 §9). Cada struct que leva um `Style` tem
//! aqui a sua forma antiga; os leitores do 9 ao 16 lêem o elemento por elas (o [`super::v13::KindOld`]
//! e o `KindV12`). O 15 e o 16 diferem só na equipa (`Message::mentions`, B14).

use serde::Deserialize;

use crate::{
    BirthScale, Connector, Crop, Dash, Embed, End, FileId, Frame, Head, Image, ImageFilter, Ink,
    Look, Pen, Rgba, RichText, Route, Shape, ShapeType, Style, Team, TextTurn, Video,
};

/// O `Style` dos formatos 9–16 (sem a escala de nascença).
#[derive(Deserialize)]
pub(super) struct StyleV16 {
    fill: Option<Rgba>,
    stroke: Option<Rgba>,
    stroke_width: f64,
    dash: Dash,
    round: bool,
    opacity: u8,
    text_color: Rgba,
    font_size: f64,
    sketch: bool,
    #[serde(default)]
    look: Look,
}

impl From<StyleV16> for Style {
    fn from(s: StyleV16) -> Self {
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
            look: s.look,
            birth: BirthScale::default(),
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ShapeV16 {
    kind: ShapeType,
    style: StyleV16,
    text: RichText,
}

impl From<ShapeV16> for Shape {
    fn from(s: ShapeV16) -> Self {
        Shape {
            kind: s.kind,
            style: s.style.into(),
            text: s.text,
        }
    }
}

/// O traço dos formatos 9–16 (sem a geometria limpa: todo traço antigo é livre).
#[derive(Deserialize)]
pub(super) struct InkV16 {
    style: StyleV16,
    pen: Pen,
    points: Vec<[f32; 3]>,
    base: [f64; 2],
    pressure: bool,
}

impl From<InkV16> for Ink {
    fn from(i: InkV16) -> Self {
        Ink {
            style: i.style.into(),
            pen: i.pen,
            points: i.points,
            base: i.base,
            pressure: i.pressure,
            clean: false,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct FrameV16 {
    name: String,
    style: StyleV16,
}

impl From<FrameV16> for Frame {
    fn from(f: FrameV16) -> Self {
        Frame {
            name: f.name,
            style: f.style.into(),
        }
    }
}

/// A imagem dos formatos 13–16 (a do 12 é o `ImageV12`).
#[derive(Deserialize)]
pub(super) struct ImageV16 {
    file: FileId,
    style: StyleV16,
    crop: Option<Crop>,
    flip: [bool; 2],
    #[serde(default)]
    filter: ImageFilter,
}

impl From<ImageV16> for Image {
    fn from(i: ImageV16) -> Self {
        Image {
            file: i.file,
            style: i.style.into(),
            crop: i.crop,
            flip: i.flip,
            filter: i.filter,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct EmbedV16 {
    link: String,
    video: Video,
    style: StyleV16,
}

impl From<EmbedV16> for Embed {
    fn from(e: EmbedV16) -> Self {
        Embed {
            link: e.link,
            video: e.video,
            style: e.style.into(),
        }
    }
}

/// A seta dos formatos 15–16 (as do 9 ao 14 são a `ConnectorV13` e a `ConnectorV14`).
#[derive(Deserialize)]
pub(super) struct ConnectorV16 {
    start: End,
    end: End,
    route: Route,
    heads: [Head; 2],
    style: StyleV16,
    #[serde(default)]
    captions: Vec<crate::Caption>,
    #[serde(default)]
    turn: TextTurn,
    waypoints: Vec<[f64; 2]>,
    #[serde(default)]
    mids: Vec<Option<f64>>,
}

impl From<ConnectorV16> for Connector {
    fn from(c: ConnectorV16) -> Self {
        Connector {
            start: c.start,
            end: c.end,
            route: c.route,
            heads: c.heads,
            style: c.style.into(),
            captions: c.captions,
            turn: c.turn,
            waypoints: c.waypoints,
            mids: c.mids,
        }
    }
}

/// Os bytes de um `BoardSet` no formato 15, já no formato actual.
pub(crate) fn read_v15(bytes: &[u8]) -> Result<crate::BoardSet, String> {
    super::v13::read_old::<ConnectorV16, super::TeamV15>(bytes)
}

/// Os bytes de um `BoardSet` no formato 16, já no formato actual.
pub(crate) fn read_v16(bytes: &[u8]) -> Result<crate::BoardSet, String> {
    super::v13::read_old::<ConnectorV16, Team>(bytes)
}

/// Uma forma de hoje na forma dos formatos 9–16 (os testes dos leitores antigos).
#[cfg(test)]
pub(super) fn shape_v16(s: Shape) -> ShapeV16 {
    let t = s.style;
    let style = StyleV16 {
        fill: t.fill,
        stroke: t.stroke,
        stroke_width: t.stroke_width,
        dash: t.dash,
        round: t.round,
        opacity: t.opacity,
        text_color: t.text_color,
        font_size: t.font_size,
        sketch: t.sketch,
        look: t.look,
    };
    ShapeV16 {
        kind: s.kind,
        style,
        text: s.text,
    }
}

#[cfg(test)]
#[path = "legacy_v16_tests.rs"]
mod tests;
