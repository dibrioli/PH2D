//! **Os ids dos diálogos de filtro do Pixel P8** (Invert, Despeckle, Outline) — no mesmo
//! `ContextMenuKind::PixelEditDialog` da P6; pintar, registar e despachar leem estas tabelas.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use crate::pixel_panel::{PixelHueMode, PixelOutlineBg, PixelOutlineMatrix, PixelOutlinePlace};
use ph2d_a11y::NodeId;

pub const PIXEL_FILTER_PREVIEW: NodeId = hash_node_id("pixel.filter_dialog.preview");
pub const PIXEL_FILTER_TILED: NodeId = hash_node_id("pixel.filter_dialog.tiled");
/// Despeckle: a janela da mediana.
pub const PIXEL_FILTER_MEDIAN_W: NodeId = hash_node_id("pixel.filter_dialog.median_w");
pub const PIXEL_FILTER_MEDIAN_H: NodeId = hash_node_id("pixel.filter_dialog.median_h");

/// O *Target*: `R G B A` (índice no `PixelChannels`, botão, rótulo).
pub const PIXEL_FILTER_CHANNELS: [(usize, NodeId, &str); 4] = [
    (
        0,
        hash_node_id("pixel.filter_dialog.channel.r"),
        "pixel.filter_dialog.channel.r",
    ),
    (
        1,
        hash_node_id("pixel.filter_dialog.channel.g"),
        "pixel.filter_dialog.channel.g",
    ),
    (
        2,
        hash_node_id("pixel.filter_dialog.channel.b"),
        "pixel.filter_dialog.channel.b",
    ),
    (
        3,
        hash_node_id("pixel.filter_dialog.channel.a"),
        "pixel.filter_dialog.channel.a",
    ),
];

pub const PIXEL_FILTER_PLACES: [(PixelOutlinePlace, NodeId, &str); 2] = [
    (
        PixelOutlinePlace::Outside,
        hash_node_id("pixel.filter_dialog.place.outside"),
        "pixel.filter_dialog.place.outside",
    ),
    (
        PixelOutlinePlace::Inside,
        hash_node_id("pixel.filter_dialog.place.inside"),
        "pixel.filter_dialog.place.inside",
    ),
];

pub const PIXEL_FILTER_MATRICES: [(PixelOutlineMatrix, NodeId, &str); 4] = [
    (
        PixelOutlineMatrix::Circle,
        hash_node_id("pixel.filter_dialog.matrix.circle"),
        "pixel.filter_dialog.matrix.circle",
    ),
    (
        PixelOutlineMatrix::Square,
        hash_node_id("pixel.filter_dialog.matrix.square"),
        "pixel.filter_dialog.matrix.square",
    ),
    (
        PixelOutlineMatrix::Horizontal,
        hash_node_id("pixel.filter_dialog.matrix.horizontal"),
        "pixel.filter_dialog.matrix.horizontal",
    ),
    (
        PixelOutlineMatrix::Vertical,
        hash_node_id("pixel.filter_dialog.matrix.vertical"),
        "pixel.filter_dialog.matrix.vertical",
    ),
];

pub const PIXEL_FILTER_BACKGROUNDS: [(PixelOutlineBg, NodeId, &str); 2] = [
    (
        PixelOutlineBg::Transparent,
        hash_node_id("pixel.filter_dialog.bg.transparent"),
        "pixel.filter_dialog.bg.transparent",
    ),
    (
        PixelOutlineBg::BackgroundColor,
        hash_node_id("pixel.filter_dialog.bg.color"),
        "pixel.filter_dialog.bg.color",
    ),
];

/// Adjustments: os campos `(índice em `PixelFilterDialog::adjust`, id, mín, máx)`.
pub const PIXEL_FILTER_BRIGHTNESS: NodeId = hash_node_id("pixel.filter_dialog.brightness");
pub const PIXEL_FILTER_CONTRAST: NodeId = hash_node_id("pixel.filter_dialog.contrast");
pub const PIXEL_FILTER_HUE: NodeId = hash_node_id("pixel.filter_dialog.hue");
pub const PIXEL_FILTER_SATURATION: NodeId = hash_node_id("pixel.filter_dialog.saturation");
pub const PIXEL_FILTER_LIGHTNESS: NodeId = hash_node_id("pixel.filter_dialog.lightness");
pub const PIXEL_FILTER_ADJUST_FIELDS: [(usize, NodeId, i32, i32); 5] = [
    (0, PIXEL_FILTER_BRIGHTNESS, -100, 100),
    (1, PIXEL_FILTER_CONTRAST, -100, 100),
    (2, PIXEL_FILTER_HUE, -180, 180),
    (3, PIXEL_FILTER_SATURATION, -100, 100),
    (4, PIXEL_FILTER_LIGHTNESS, -100, 100),
];

/// Color Curve: a caixa do editor de curva (`Plain`: premir agarra/acrescenta, duplo-clique tira).
pub const PIXEL_FILTER_CURVE: NodeId = hash_node_id("pixel.filter_dialog.curve");

/// Hue/Saturation: os quatro modelos, com os rótulos do Aseprite.
pub const PIXEL_FILTER_HUE_MODES: [(PixelHueMode, NodeId, &str); 4] = [
    (
        PixelHueMode::Hsl,
        hash_node_id("pixel.filter_dialog.mode.hsl"),
        "pixel.filter_dialog.mode.hsl",
    ),
    (
        PixelHueMode::Hsv,
        hash_node_id("pixel.filter_dialog.mode.hsv"),
        "pixel.filter_dialog.mode.hsv",
    ),
    (
        PixelHueMode::HslPlus,
        hash_node_id("pixel.filter_dialog.mode.hsl_plus"),
        "pixel.filter_dialog.mode.hsl_plus",
    ),
    (
        PixelHueMode::HsvPlus,
        hash_node_id("pixel.filter_dialog.mode.hsv_plus"),
        "pixel.filter_dialog.mode.hsv_plus",
    ),
];

/// Os botões de escolha dos diálogos de filtro (registo e posse).
pub fn pixel_filter_buttons() -> impl Iterator<Item = NodeId> {
    [PIXEL_FILTER_PREVIEW, PIXEL_FILTER_TILED]
        .into_iter()
        .chain(PIXEL_FILTER_CHANNELS.iter().map(|(_, id, _)| *id))
        .chain(PIXEL_FILTER_PLACES.iter().map(|(_, id, _)| *id))
        .chain(PIXEL_FILTER_MATRICES.iter().map(|(_, id, _)| *id))
        .chain(PIXEL_FILTER_BACKGROUNDS.iter().map(|(_, id, _)| *id))
        .chain(PIXEL_FILTER_HUE_MODES.iter().map(|(_, id, _)| *id))
}
