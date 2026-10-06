//! ⭐ doc 121 §9.19 (3) — **A PONTA E A JUNTA do traço**, as duas escolhas do cartão da forma.
//!
//! O `StrokeSpec` sempre as soube (`LineCap`, `LineJoin`) e as duas rotas de desenho — a placa e o Vello —
//! já as honravam; faltava o nó ter onde dizê-las: o produto tracejava sempre rente e em esquadria.
//! ⚠️ O índice é formato de arquivo (o valor gravado no `.ph2dproj`) — APPEND ONLY, e o `0` é o traço de
//! sempre: um projecto sem as linhas abre igual.

/// A ponta de cada traço e de cada pedaço do tracejado.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// A junta entre dois troços de um traço.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

impl StrokeCap {
    /// O valor do param (um índice; fora da faixa, ou `NaN`, é o de sempre).
    #[must_use]
    pub fn from_index(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Round,
            2 => Self::Square,
            _ => Self::Butt,
        }
    }
}

impl StrokeJoin {
    /// O valor do param (um índice; fora da faixa, ou `NaN`, é o de sempre).
    #[must_use]
    pub fn from_index(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Round,
            2 => Self::Bevel,
            _ => Self::Miter,
        }
    }
}

/// Os rótulos do `Cap`, pela ordem dos índices.
pub(crate) static CAP_LABELS: &[&str] = &[
    "node.opts.node_motion_shape.cap_labels.0",
    "node.opts.node_motion_shape.cap_labels.1",
    "node.opts.node_motion_shape.cap_labels.2",
];

/// Os rótulos do `Join`, pela ordem dos índices.
pub(crate) static JOIN_LABELS: &[&str] = &[
    "node.opts.node_motion_shape.join_labels.0",
    "node.opts.node_motion_shape.join_labels.1",
    "node.opts.node_motion_shape.join_labels.2",
];
