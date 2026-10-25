//! **Os ids do Pixel Art que o chrome lê** (ADR-0192): o rect do painel PIXEL, o abridor dele no menu
//! *Window* e os controlos do diálogo *New Sprite* (o chrome pinta-o e despacha-o).
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use crate::pixel_panel::{NewSpriteBackground, NewSpriteMode, PixelDither, PixelToGray};
use ph2d_a11y::NodeId;

pub const PIXEL_PANEL: NodeId = hash_node_id("pixel.panel");

/// A linha *Pixel* do menu *Window*.
pub const TOPBAR_PIXEL: NodeId = hash_node_id("topbar_pixel");

/// A amostra da cor de FRENTE (a coluna da esquerda em Draw): um Down abre o seletor partilhado, e
/// a família lê de volta a cor dele (`ph2d_app_pixel::panel`).
pub const PIXEL_FG: NodeId = hash_node_id("pixel.fg");
/// A amostra da cor de FUNDO, na mesma forma.
pub const PIXEL_BG: NodeId = hash_node_id("pixel.bg");

/// O campo da largura do *New Sprite*.
pub const NEW_SPRITE_WIDTH: NodeId = hash_node_id("pixel.new_sprite.width");
/// O campo da altura do *New Sprite*.
pub const NEW_SPRITE_HEIGHT: NodeId = hash_node_id("pixel.new_sprite.height");
pub const NEW_SPRITE_OK: NodeId = hash_node_id("pixel.new_sprite.ok");
pub const NEW_SPRITE_CANCEL: NodeId = hash_node_id("pixel.new_sprite.cancel");

/// Os modos de cor: o valor, o botão e o rótulo — pintar, registar e despachar leem a MESMA tabela.
pub const NEW_SPRITE_MODES: [(NewSpriteMode, NodeId, &str); 3] = [
    (
        NewSpriteMode::Rgba,
        hash_node_id("pixel.new_sprite.mode.rgba"),
        "pixel.new_sprite.mode.rgba",
    ),
    (
        NewSpriteMode::Gray,
        hash_node_id("pixel.new_sprite.mode.gray"),
        "pixel.new_sprite.mode.gray",
    ),
    (
        NewSpriteMode::Indexed,
        hash_node_id("pixel.new_sprite.mode.indexed"),
        "pixel.new_sprite.mode.indexed",
    ),
];

/// Os fundos, na mesma forma.
pub const NEW_SPRITE_BGS: [(NewSpriteBackground, NodeId, &str); 3] = [
    (
        NewSpriteBackground::Transparent,
        hash_node_id("pixel.new_sprite.bg.transparent"),
        "pixel.new_sprite.bg.transparent",
    ),
    (
        NewSpriteBackground::White,
        hash_node_id("pixel.new_sprite.bg.white"),
        "pixel.new_sprite.bg.white",
    ),
    (
        NewSpriteBackground::Black,
        hash_node_id("pixel.new_sprite.bg.black"),
        "pixel.new_sprite.bg.black",
    ),
];

/// Os dois campos numéricos.
pub const NEW_SPRITE_FIELDS: [NodeId; 2] = [NEW_SPRITE_WIDTH, NEW_SPRITE_HEIGHT];

/// O `id` é um controlo do diálogo *New Sprite*? (Um clique nele não fecha o diálogo.)
#[must_use]
pub fn new_sprite_dialog_owns(id: NodeId) -> bool {
    NEW_SPRITE_FIELDS.contains(&id)
        || id == NEW_SPRITE_OK
        || id == NEW_SPRITE_CANCEL
        || NEW_SPRITE_MODES.iter().any(|(_, b, _)| *b == id)
        || NEW_SPRITE_BGS.iter().any(|(_, b, _)| *b == id)
}

/// A amostra do *Edit color* do painel PIXEL: um Down abre o seletor partilhado sobre a cor da
/// amostra clicada da paleta, e a família lê de volta a cor dele (`ph2d_app_pixel::panel`).
pub const PIXEL_PAL_EDIT: NodeId = hash_node_id("pixel.palette.edit");

pub const COLOR_MODE_OK: NodeId = hash_node_id("pixel.color_mode.ok");
pub const COLOR_MODE_CANCEL: NodeId = hash_node_id("pixel.color_mode.cancel");

/// Os alvos do diálogo *Color Mode*: o valor, o botão e o rótulo (as chaves dos modos do New Sprite).
pub const COLOR_MODE_TARGETS: [(NewSpriteMode, NodeId, &str); 3] = [
    (
        NewSpriteMode::Rgba,
        hash_node_id("pixel.color_mode.target.rgba"),
        "pixel.new_sprite.mode.rgba",
    ),
    (
        NewSpriteMode::Gray,
        hash_node_id("pixel.color_mode.target.gray"),
        "pixel.new_sprite.mode.gray",
    ),
    (
        NewSpriteMode::Indexed,
        hash_node_id("pixel.color_mode.target.indexed"),
        "pixel.new_sprite.mode.indexed",
    ),
];

/// Os oito dithers, na ordem da lista do Aseprite.
pub const COLOR_MODE_DITHERS: [(PixelDither, NodeId, &str); 8] = [
    (
        PixelDither::None,
        hash_node_id("pixel.color_mode.dither.none"),
        "pixel.color_mode.dither.none",
    ),
    (
        PixelDither::Ordered2,
        hash_node_id("pixel.color_mode.dither.ordered2"),
        "pixel.color_mode.dither.size2",
    ),
    (
        PixelDither::Ordered4,
        hash_node_id("pixel.color_mode.dither.ordered4"),
        "pixel.color_mode.dither.size4",
    ),
    (
        PixelDither::Ordered8,
        hash_node_id("pixel.color_mode.dither.ordered8"),
        "pixel.color_mode.dither.size8",
    ),
    (
        PixelDither::Old2,
        hash_node_id("pixel.color_mode.dither.old2"),
        "pixel.color_mode.dither.size2",
    ),
    (
        PixelDither::Old4,
        hash_node_id("pixel.color_mode.dither.old4"),
        "pixel.color_mode.dither.size4",
    ),
    (
        PixelDither::Old8,
        hash_node_id("pixel.color_mode.dither.old8"),
        "pixel.color_mode.dither.size8",
    ),
    (
        PixelDither::FloydSteinberg,
        hash_node_id("pixel.color_mode.dither.floyd"),
        "pixel.color_mode.dither.floyd",
    ),
];

/// A arrumação dos dithers no diálogo: a 1.ª fila sem rótulo (os dois nomes longos), depois uma
/// fila por família com o rótulo dela à esquerda e os três tamanhos — os nomes nunca se cortam.
pub const COLOR_MODE_DITHER_WIDE: [PixelDither; 2] =
    [PixelDither::None, PixelDither::FloydSteinberg];
pub const COLOR_MODE_DITHER_ROWS: [(&str, [PixelDither; 3]); 2] = [
    (
        "pixel.color_mode.dither.ordered",
        [
            PixelDither::Ordered2,
            PixelDither::Ordered4,
            PixelDither::Ordered8,
        ],
    ),
    (
        "pixel.color_mode.dither.old",
        [PixelDither::Old2, PixelDither::Old4, PixelDither::Old8],
    ),
];

/// Os três métodos de cinza.
pub const COLOR_MODE_GRAYS: [(PixelToGray, NodeId, &str); 3] = [
    (
        PixelToGray::Luminance,
        hash_node_id("pixel.color_mode.gray.luminance"),
        "pixel.color_mode.gray.luminance",
    ),
    (
        PixelToGray::Hsv,
        hash_node_id("pixel.color_mode.gray.hsv"),
        "pixel.color_mode.gray.hsv",
    ),
    (
        PixelToGray::Hsl,
        hash_node_id("pixel.color_mode.gray.hsl"),
        "pixel.color_mode.gray.hsl",
    ),
];

/// O `id` é um controlo do diálogo *Color Mode*?
#[must_use]
pub fn color_mode_dialog_owns(id: NodeId) -> bool {
    id == COLOR_MODE_OK
        || id == COLOR_MODE_CANCEL
        || COLOR_MODE_TARGETS.iter().any(|(_, b, _)| *b == id)
        || COLOR_MODE_DITHERS.iter().any(|(_, b, _)| *b == id)
        || COLOR_MODE_GRAYS.iter().any(|(_, b, _)| *b == id)
}
