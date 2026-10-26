//! **Os ids dos quatro diálogos do Pixel P6** (Color Range, Modify, Canvas Size, Sprite Size) — um
//! só `ContextMenuKind::PixelEditDialog`; pintar, registar e despachar leem estas tabelas.
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use crate::pixel_panel::{PixelModifyBrush, PixelSizeMethod};
use ph2d_a11y::NodeId;

pub const PIXEL_EDIT_OK: NodeId = hash_node_id("pixel.edit_dialog.ok");
pub const PIXEL_EDIT_CANCEL: NodeId = hash_node_id("pixel.edit_dialog.cancel");

/// Color Range: a tolerância `0..=255`.
pub const PIXEL_EDIT_TOLERANCE: NodeId = hash_node_id("pixel.edit_dialog.tolerance");
/// Modify: os píxeis.
pub const PIXEL_EDIT_MODIFY_N: NodeId = hash_node_id("pixel.edit_dialog.modify_n");

/// Canvas Size: o tamanho novo e as quatro bordas (em sincronia).
pub const PIXEL_EDIT_CANVAS_W: NodeId = hash_node_id("pixel.edit_dialog.canvas_w");
pub const PIXEL_EDIT_CANVAS_H: NodeId = hash_node_id("pixel.edit_dialog.canvas_h");
pub const PIXEL_EDIT_LEFT: NodeId = hash_node_id("pixel.edit_dialog.left");
pub const PIXEL_EDIT_TOP: NodeId = hash_node_id("pixel.edit_dialog.top");
pub const PIXEL_EDIT_RIGHT: NodeId = hash_node_id("pixel.edit_dialog.right");
pub const PIXEL_EDIT_BOTTOM: NodeId = hash_node_id("pixel.edit_dialog.bottom");

/// Sprite Size: píxeis e percentagens (em sincronia) e o Lock Ratio.
pub const PIXEL_EDIT_SPRITE_W: NodeId = hash_node_id("pixel.edit_dialog.sprite_w");
pub const PIXEL_EDIT_SPRITE_H: NodeId = hash_node_id("pixel.edit_dialog.sprite_h");
pub const PIXEL_EDIT_SPRITE_WPCT: NodeId = hash_node_id("pixel.edit_dialog.sprite_wpct");
pub const PIXEL_EDIT_SPRITE_HPCT: NodeId = hash_node_id("pixel.edit_dialog.sprite_hpct");
pub const PIXEL_EDIT_LOCK: NodeId = hash_node_id("pixel.edit_dialog.lock_ratio");

/// As bordas na ordem da choice: esquerda, cima, direita, baixo.
pub const PIXEL_EDIT_BORDERS: [NodeId; 4] = [
    PIXEL_EDIT_LEFT,
    PIXEL_EDIT_TOP,
    PIXEL_EDIT_RIGHT,
    PIXEL_EDIT_BOTTOM,
];

/// Todos os campos numéricos dos quatro diálogos.
pub const PIXEL_EDIT_FIELDS: [NodeId; 12] = [
    PIXEL_EDIT_TOLERANCE,
    PIXEL_EDIT_MODIFY_N,
    PIXEL_EDIT_CANVAS_W,
    PIXEL_EDIT_CANVAS_H,
    PIXEL_EDIT_LEFT,
    PIXEL_EDIT_TOP,
    PIXEL_EDIT_RIGHT,
    PIXEL_EDIT_BOTTOM,
    PIXEL_EDIT_SPRITE_W,
    PIXEL_EDIT_SPRITE_H,
    PIXEL_EDIT_SPRITE_WPCT,
    PIXEL_EDIT_SPRITE_HPCT,
];

/// A grelha 3×3 da âncora do Canvas Size: `(coluna, linha)`, o botão e o rótulo, linha a linha.
pub const PIXEL_EDIT_ANCHORS: [((u8, u8), NodeId, &str); 9] = [
    (
        (0, 0),
        hash_node_id("pixel.edit_dialog.anchor.nw"),
        "pixel.edit_dialog.anchor.nw",
    ),
    (
        (1, 0),
        hash_node_id("pixel.edit_dialog.anchor.n"),
        "pixel.edit_dialog.anchor.n",
    ),
    (
        (2, 0),
        hash_node_id("pixel.edit_dialog.anchor.ne"),
        "pixel.edit_dialog.anchor.ne",
    ),
    (
        (0, 1),
        hash_node_id("pixel.edit_dialog.anchor.w"),
        "pixel.edit_dialog.anchor.w",
    ),
    (
        (1, 1),
        hash_node_id("pixel.edit_dialog.anchor.c"),
        "pixel.edit_dialog.anchor.c",
    ),
    (
        (2, 1),
        hash_node_id("pixel.edit_dialog.anchor.e"),
        "pixel.edit_dialog.anchor.e",
    ),
    (
        (0, 2),
        hash_node_id("pixel.edit_dialog.anchor.sw"),
        "pixel.edit_dialog.anchor.sw",
    ),
    (
        (1, 2),
        hash_node_id("pixel.edit_dialog.anchor.s"),
        "pixel.edit_dialog.anchor.s",
    ),
    (
        (2, 2),
        hash_node_id("pixel.edit_dialog.anchor.se"),
        "pixel.edit_dialog.anchor.se",
    ),
];

/// O pincel do Modify.
pub const PIXEL_EDIT_BRUSHES: [(PixelModifyBrush, NodeId, &str); 2] = [
    (
        PixelModifyBrush::Square,
        hash_node_id("pixel.edit_dialog.brush.square"),
        "pixel.edit_dialog.brush.square",
    ),
    (
        PixelModifyBrush::Circle,
        hash_node_id("pixel.edit_dialog.brush.circle"),
        "pixel.edit_dialog.brush.circle",
    ),
];

/// A interpolação do Sprite Size.
pub const PIXEL_EDIT_METHODS: [(PixelSizeMethod, NodeId, &str); 3] = [
    (
        PixelSizeMethod::Nearest,
        hash_node_id("pixel.edit_dialog.method.nearest"),
        "pixel.edit_dialog.method.nearest",
    ),
    (
        PixelSizeMethod::Bilinear,
        hash_node_id("pixel.edit_dialog.method.bilinear"),
        "pixel.edit_dialog.method.bilinear",
    ),
    (
        PixelSizeMethod::RotSprite,
        hash_node_id("pixel.edit_dialog.method.rotsprite"),
        "pixel.edit_dialog.method.rotsprite",
    ),
];

/// O `id` é um controlo de um dos diálogos do Pixel P6? (Um clique nele não fecha o diálogo.)
#[must_use]
pub fn pixel_edit_dialog_owns(id: NodeId) -> bool {
    id == PIXEL_EDIT_OK
        || id == PIXEL_EDIT_CANCEL
        || id == PIXEL_EDIT_LOCK
        || PIXEL_EDIT_FIELDS.contains(&id)
        || PIXEL_EDIT_ANCHORS.iter().any(|(_, b, _)| *b == id)
        || PIXEL_EDIT_BRUSHES.iter().any(|(_, b, _)| *b == id)
        || PIXEL_EDIT_METHODS.iter().any(|(_, b, _)| *b == id)
}
