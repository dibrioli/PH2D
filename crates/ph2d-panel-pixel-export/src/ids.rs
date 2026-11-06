//! **Os ids das janelas de Export e Import.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const EXPORT_DRAG: NodeId = hash_node_id("pixel_export.drag");
pub const EXPORT_RESET: NodeId = hash_node_id("pixel_export.reset");
pub const CANCEL: NodeId = hash_node_id("pixel_export.cancel");
pub const OK: NodeId = hash_node_id("pixel_export.ok");

pub const SEC_FILE: NodeId = hash_node_id("pixel_export.sec.file");
pub const SEC_CONTENT: NodeId = hash_node_id("pixel_export.sec.content");
pub const SEC_PREVIEW: NodeId = hash_node_id("pixel_export.sec.preview");
pub const SEC_LAYOUT: NodeId = hash_node_id("pixel_export.sec.layout");
pub const SEC_SPRITE: NodeId = hash_node_id("pixel_export.sec.sprite");
pub const SEC_BORDERS: NodeId = hash_node_id("pixel_export.sec.borders");
pub const SEC_OUTPUT: NodeId = hash_node_id("pixel_export.sec.output");
pub const SEC_IMAGE: NodeId = hash_node_id("pixel_export.sec.image");
pub const SEC_GRID: NodeId = hash_node_id("pixel_export.sec.grid");

/// Os formatos do Export As, na ordem de `comum::FORMATOS`.
pub const FORMATS: [NodeId; 5] = [
    hash_node_id("pixel_export.format.png"),
    hash_node_id("pixel_export.format.gif"),
    hash_node_id("pixel_export.format.apng"),
    hash_node_id("pixel_export.format.webp"),
    hash_node_id("pixel_export.format.ase"),
];

// As listas suspensas: a caixa.
pub const LAYERS: NodeId = hash_node_id("pixel_export.layers");
pub const FRAMES: NodeId = hash_node_id("pixel_export.frames");
pub const DIRECTION: NodeId = hash_node_id("pixel_export.direction");
pub const SHEET_TYPE: NodeId = hash_node_id("pixel_export.sheet_type");
pub const LIMIT: NodeId = hash_node_id("pixel_export.limit");
pub const BOXES: [NodeId; 5] = [LAYERS, FRAMES, DIRECTION, SHEET_TYPE, LIMIT];

// As linhas fixas das listas.
pub const LAYERS_VISIBLE: NodeId = hash_node_id("pixel_export.layers.visible");
pub const LAYERS_ALL: NodeId = hash_node_id("pixel_export.layers.all");
pub const FRAMES_ALL: NodeId = hash_node_id("pixel_export.frames.all");
pub const FRAMES_RANGE: NodeId = hash_node_id("pixel_export.frames.range");
pub const DIRECTIONS: [NodeId; 4] = [
    hash_node_id("pixel_export.direction.forward"),
    hash_node_id("pixel_export.direction.reverse"),
    hash_node_id("pixel_export.direction.pingpong"),
    hash_node_id("pixel_export.direction.pingpong_reverse"),
];
pub const SHEET_TYPES: [NodeId; 5] = [
    hash_node_id("pixel_export.type.horizontal"),
    hash_node_id("pixel_export.type.vertical"),
    hash_node_id("pixel_export.type.rows"),
    hash_node_id("pixel_export.type.columns"),
    hash_node_id("pixel_export.type.packed"),
];
pub const LIMIT_NONE: NodeId = hash_node_id("pixel_export.limit.none");
pub const LIMIT_COLUMNS: NodeId = hash_node_id("pixel_export.limit.columns");
pub const LIMIT_ROWS: NodeId = hash_node_id("pixel_export.limit.rows");
pub const LIMIT_WIDTH: NodeId = hash_node_id("pixel_export.limit.width");
pub const LIMIT_HEIGHT: NodeId = hash_node_id("pixel_export.limit.height");
pub const LIMIT_OPTIONS: [NodeId; 5] = [
    LIMIT_NONE,
    LIMIT_COLUMNS,
    LIMIT_ROWS,
    LIMIT_WIDTH,
    LIMIT_HEIGHT,
];

const LAYER_BASE: NodeId = hash_node_id("pixel_export.layer");
const TAG_BASE: NodeId = hash_node_id("pixel_export.tag");

/// A linha da camada `i` (quantas forem: o sprite decide).
#[must_use]
pub const fn layer_option(i: usize) -> NodeId {
    NodeId(LAYER_BASE.0.wrapping_add(i as u64))
}

/// A linha da tag `i`.
#[must_use]
pub const fn tag_option(i: usize) -> NodeId {
    NodeId(TAG_BASE.0.wrapping_add(i as u64))
}

// Os números.
pub const FROM: NodeId = hash_node_id("pixel_export.from");
pub const TO: NodeId = hash_node_id("pixel_export.to");
pub const SCALE: NodeId = hash_node_id("pixel_export.scale");
pub const LOOPS: NodeId = hash_node_id("pixel_export.loops");
pub const LIMIT_N: NodeId = hash_node_id("pixel_export.limit_n");
pub const BORDER: NodeId = hash_node_id("pixel_export.border");
pub const SHAPE: NodeId = hash_node_id("pixel_export.shape");
pub const INNER: NodeId = hash_node_id("pixel_export.inner");
pub const EXTRUDE: NodeId = hash_node_id("pixel_export.extrude");
pub const GRID_X: NodeId = hash_node_id("pixel_export.grid_x");
pub const GRID_Y: NodeId = hash_node_id("pixel_export.grid_y");
pub const GRID_W: NodeId = hash_node_id("pixel_export.grid_w");
pub const GRID_H: NodeId = hash_node_id("pixel_export.grid_h");
pub const GAP_X: NodeId = hash_node_id("pixel_export.gap_x");
pub const GAP_Y: NodeId = hash_node_id("pixel_export.gap_y");

/// O maior lado de um documento: nenhum destes números passa dele.
const LADO: f64 = ph2d_pixel_core::MAX_SIDE as f64;
/// Escala em %: o maior lado de um documento vindo de um sprite de 1 célula.
const ESCALA_MAX: f64 = LADO * 100.0; // LITERAL-PX-OK: de razão para %, não píxeis

/// `(id, mínimo, máximo)`. O passo é 1.
pub const NUMEROS: [(NodeId, f64, f64); 15] = [
    (FROM, 1.0, LADO),
    (TO, 1.0, LADO),
    (SCALE, 1.0, ESCALA_MAX),
    // O contador de laços do GIF é um u16; 0 = para sempre.
    (LOOPS, 0.0, LADO),
    (LIMIT_N, 1.0, LADO),
    (BORDER, 0.0, LADO),
    (SHAPE, 0.0, LADO),
    (INNER, 0.0, LADO),
    (EXTRUDE, 0.0, LADO),
    (GRID_X, 0.0, LADO),
    (GRID_Y, 0.0, LADO),
    (GRID_W, 1.0, LADO),
    (GRID_H, 1.0, LADO),
    (GAP_X, 0.0, LADO),
    (GAP_Y, 0.0, LADO),
];

// As caixas de marcar.
pub const MERGE: NodeId = hash_node_id("pixel_export.merge");
pub const IGNORE_EMPTY: NodeId = hash_node_id("pixel_export.ignore_empty");
pub const POW2: NodeId = hash_node_id("pixel_export.pow2");
pub const SPLIT_LAYERS: NodeId = hash_node_id("pixel_export.split_layers");
pub const SPLIT_TAGS: NodeId = hash_node_id("pixel_export.split_tags");
pub const TRIM_CELS: NodeId = hash_node_id("pixel_export.trim_cels");
pub const TRIM_SPRITE: NodeId = hash_node_id("pixel_export.trim_sprite");
pub const JSON: NodeId = hash_node_id("pixel_export.json");
pub const JSON_TAGS: NodeId = hash_node_id("pixel_export.json_tags");
pub const JSON_LAYERS: NodeId = hash_node_id("pixel_export.json_layers");
pub const JSON_SLICES: NodeId = hash_node_id("pixel_export.json_slices");
pub const CHECKS: [NodeId; 11] = [
    MERGE,
    IGNORE_EMPTY,
    POW2,
    SPLIT_LAYERS,
    SPLIT_TAGS,
    TRIM_CELS,
    TRIM_SPRITE,
    JSON,
    JSON_TAGS,
    JSON_LAYERS,
    JSON_SLICES,
];

pub const JSON_HASH: NodeId = hash_node_id("pixel_export.json_hash");
pub const JSON_ARRAY: NodeId = hash_node_id("pixel_export.json_array");
pub const AS_PIXEL: NodeId = hash_node_id("pixel_export.as_pixel");
pub const AS_IMAGE: NodeId = hash_node_id("pixel_export.as_image");
pub const BY_ROWS: NodeId = hash_node_id("pixel_export.by_rows");
pub const BY_COLUMNS: NodeId = hash_node_id("pixel_export.by_columns");
/// O formato do nome de cada quadro no JSON.
pub const ITEM_NAME: NodeId = hash_node_id("pixel_export.item_name");
