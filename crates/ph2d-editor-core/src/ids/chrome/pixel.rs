//! **Os ids do Pixel Art que o chrome lê** (ADR-0192): o rect do painel PIXEL, o abridor dele no menu
//! *Window* e os controlos do diálogo *New Sprite* (o chrome pinta-o e despacha-o).
//!
//! ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use super::super::hash_node_id;
use crate::pixel_panel::{NewSpriteBackground, NewSpriteMode};
use ph2d_a11y::NodeId;

pub const PIXEL_PANEL: NodeId = hash_node_id("pixel.panel");

/// A linha *Pixel* do menu *Window*.
pub const TOPBAR_PIXEL: NodeId = hash_node_id("topbar_pixel");

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
