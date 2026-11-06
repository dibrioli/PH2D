//! **Os ids da tela dos tilemaps.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

pub const TILES_DRAG: NodeId = hash_node_id("pixel_tiles.drag");
pub const TILES_CANCEL: NodeId = hash_node_id("pixel_tiles.cancel");
pub const TILES_OK: NodeId = hash_node_id("pixel_tiles.ok");

pub const SEC_DRAW: NodeId = hash_node_id("pixel_tiles.sec.draw");
pub const SEC_TILESET: NodeId = hash_node_id("pixel_tiles.sec.tileset");
pub const TILES_SEC_GRID: NodeId = hash_node_id("pixel_tiles.sec.grid");

pub const TARGET_PIXELS: NodeId = hash_node_id("pixel_tiles.target.pixels");
pub const TARGET_TILES: NodeId = hash_node_id("pixel_tiles.target.tiles");
pub const MODE_MANUAL: NodeId = hash_node_id("pixel_tiles.mode.manual");
pub const MODE_AUTO: NodeId = hash_node_id("pixel_tiles.mode.auto");
pub const MODE_STACK: NodeId = hash_node_id("pixel_tiles.mode.stack");

pub const FLIP_X: NodeId = hash_node_id("pixel_tiles.flip.x");
pub const FLIP_Y: NodeId = hash_node_id("pixel_tiles.flip.y");
pub const FLIP_D: NodeId = hash_node_id("pixel_tiles.flip.d");
pub const ADD_TILE: NodeId = hash_node_id("pixel_tiles.add");
pub const DELETE_TILE: NodeId = hash_node_id("pixel_tiles.delete");

pub const TILE_W: NodeId = hash_node_id("pixel_tiles.tile_w");
pub const TILE_H: NodeId = hash_node_id("pixel_tiles.tile_h");
pub const MATCH_FLIPS: NodeId = hash_node_id("pixel_tiles.match_flips");
pub const NEW_SET: NodeId = hash_node_id("pixel_tiles.new_set");

/// O lado máximo de um tile no diálogo. Recurso: o `.ase` e o nosso formato guardam-no em 16 bits.
pub const TILE_SIDE_MAX: f64 = u16::MAX as f64;

pub const BOTOES: [NodeId; 13] = [
    TILES_CANCEL,
    TILES_OK,
    TARGET_PIXELS,
    TARGET_TILES,
    MODE_MANUAL,
    MODE_AUTO,
    MODE_STACK,
    FLIP_X,
    FLIP_Y,
    FLIP_D,
    ADD_TILE,
    DELETE_TILE,
    NEW_SET,
];

/// A linha do tileset `i` na janela de criar ou converter.
#[must_use]
pub fn conjunto(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_tiles.set.{i}"))
}

/// A célula do tile `i` no cartão Tileset.
#[must_use]
pub fn tile(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel_tiles.tile.{i}"))
}
