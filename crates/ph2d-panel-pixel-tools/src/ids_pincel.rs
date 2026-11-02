//! Os ids da P7b no painel PIXEL: o pincel de padrão, os pincéis guardados e a Dynamics.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::{
    PIXEL_MAX_BRUSH, PixelGradientDither, PixelPatternAlign, PixelPatternTurn, PixelSensor,
};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// Os três alinhamentos do padrão (segmentos) e a dica de cada um.
pub const PATTERN_ALIGNS: [(PixelPatternAlign, NodeId, &str); 3] = [
    (
        PixelPatternAlign::Source,
        hash_node_id("pixel.brush.align.source"),
        "panel.pixel.pattern.source_tip",
    ),
    (
        PixelPatternAlign::Destination,
        hash_node_id("pixel.brush.align.destination"),
        "panel.pixel.pattern.destination_tip",
    ),
    (
        PixelPatternAlign::Paint,
        hash_node_id("pixel.brush.align.paint"),
        "panel.pixel.pattern.paint_tip",
    ),
];

/// Virar e girar o padrão (na ordem dos botões).
pub const TURNS: [(PixelPatternTurn, NodeId); 4] = [
    (PixelPatternTurn::FlipH, hash_node_id("pixel.brush.flip_h")),
    (PixelPatternTurn::FlipV, hash_node_id("pixel.brush.flip_v")),
    (
        PixelPatternTurn::RotateLeft,
        hash_node_id("pixel.brush.rotate_left"),
    ),
    (
        PixelPatternTurn::RotateRight,
        hash_node_id("pixel.brush.rotate_right"),
    ),
];

pub const SAVE_HERE: NodeId = hash_node_id("pixel.brush.save_here");

/// O espaço `n` (de 0) da lista: recuperar, trancar, apagar.
#[must_use]
pub fn brush_slot_id(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.brush.slot.{n}"))
}

#[must_use]
pub fn brush_lock_id(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.brush.slot.{n}.lock"))
}

#[must_use]
pub fn brush_delete_id(n: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.brush.slot.{n}.delete"))
}

/// O botão que abre e fecha a secção Dynamics.
pub const DYNAMICS: NodeId = hash_node_id("pixel.dyn.open");
pub const DYN_STABILIZER: NodeId = hash_node_id("pixel.dyn.stabilizer");

/// A linha da grelha Size · Angle · Gradient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynRow {
    Size,
    Angle,
    Gradient,
}

impl DynRow {
    pub const ALL: [Self; 3] = [Self::Size, Self::Angle, Self::Gradient];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Size => "panel.pixel.dynamics.size",
            Self::Angle => "panel.pixel.dynamics.angle",
            Self::Gradient => "panel.pixel.dynamics.gradient",
        }
    }
}

/// Os sensores de uma linha como segmentos (uma só entrada por linha, como no Trial).
pub const SENSORS: [(PixelSensor, &str); 3] = [
    (PixelSensor::Off, "panel.pixel.dynamics.off"),
    (PixelSensor::Pressure, "panel.pixel.dynamics.pressure"),
    (PixelSensor::Velocity, "panel.pixel.dynamics.velocity"),
];

/// O segmento do sensor `k` ([`SENSORS`]) na linha `row`.
#[must_use]
pub fn sensor_id(row: DynRow, k: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.dyn.sensor.{row:?}.{k}"))
}

/// Todos os segmentos dos sensores: `(linha, sensor, id)`.
pub fn sensor_ids() -> impl Iterator<Item = (DynRow, PixelSensor, NodeId)> {
    DynRow::ALL.into_iter().flat_map(|r| {
        SENSORS
            .iter()
            .enumerate()
            .map(move |(k, (s, _))| (r, *s, sensor_id(r, k)))
    })
}

/// A corda do Stabilizer, o Min do Size e o Min do Angle: id, faixa `(min, max, passo)`.
pub const DYN_STABILIZER_LEN: NodeId = hash_node_id("pixel.dyn.stabilizer_len");
pub const DYN_MIN_SIZE: NodeId = hash_node_id("pixel.dyn.min_size");
pub const DYN_MIN_ANGLE: NodeId = hash_node_id("pixel.dyn.min_angle");
/// A barra do Stabilizer vai a 64: a foto 55 do Trial mostra 16 a ¼ da barra.
pub const DYN_NUMBERS: [(NodeId, (f64, f64, f64)); 3] = [
    (DYN_STABILIZER_LEN, (1.0, 64.0, 1.0)), // LITERAL-PX-OK: faixa do valor no Aseprite (domínio do campo, não medida de desenho)
    (DYN_MIN_SIZE, (1.0, PIXEL_MAX_BRUSH as f64, 1.0)),
    (DYN_MIN_ANGLE, (-180.0, 180.0, 1.0)), // LITERAL-PX-OK: faixa do valor no Aseprite (domínio do campo, não medida de desenho)
];

/// A matriz do Gradient (segmentos).
pub const DITHERS: [(PixelGradientDither, NodeId); 4] = [
    (
        PixelGradientDither::None,
        hash_node_id("pixel.dyn.dither.none"),
    ),
    (
        PixelGradientDither::Bayer2,
        hash_node_id("pixel.dyn.dither.bayer2"),
    ),
    (
        PixelGradientDither::Bayer4,
        hash_node_id("pixel.dyn.dither.bayer4"),
    ),
    (
        PixelGradientDither::Bayer8,
        hash_node_id("pixel.dyn.dither.bayer8"),
    ),
];

/// As barras de duas pegas: a da pressão e a da velocidade.
pub const THRESHOLDS: [(PixelSensor, NodeId, &str); 2] = [
    (
        PixelSensor::Pressure,
        hash_node_id("pixel.dyn.range.pressure"),
        "panel.pixel.dynamics.range_pressure",
    ),
    (
        PixelSensor::Velocity,
        hash_node_id("pixel.dyn.range.velocity"),
        "panel.pixel.dynamics.range_velocity",
    ),
];
