//! Os ids da galeria em cartões (doc `CartoesUI/08` §1.1) — literais, para o censo
//! `every_non_literal_hash_is_named` e o de colisões os lerem.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Os cartões do vocabulário (secções) ──
pub const CARD_FIELDS: NodeId = hash_node_id("gallery.card.fields");
pub const CARD_CHOICES: NodeId = hash_node_id("gallery.card.choices");
pub const CARD_COLOR: NodeId = hash_node_id("gallery.card.color");
pub const CARD_ACTIONS: NodeId = hash_node_id("gallery.card.actions");
pub const CARD_MARKS: NodeId = hash_node_id("gallery.card.marks");
pub const CARD_ORDERING: NodeId = hash_node_id("gallery.card.ordering");
pub const CARD_PHYSICS: NodeId = hash_node_id("gallery.card.physics");

// ── Campos numéricos ──
pub const POS_X: NodeId = hash_node_id("gallery.field.pos_x");
pub const POS_Y: NodeId = hash_node_id("gallery.field.pos_y");
pub const ROTATION: NodeId = hash_node_id("gallery.field.rotation");
pub const SIZE_W: NodeId = hash_node_id("gallery.field.size_w");
pub const SIZE_H: NodeId = hash_node_id("gallery.field.size_h");
pub const OPACITY: NodeId = hash_node_id("gallery.field.opacity");
pub const MARK_FREE: NodeId = hash_node_id("gallery.field.mark_free");
pub const MARK_ANIMATED: NodeId = hash_node_id("gallery.field.mark_animated");
pub const MARK_KEY: NodeId = hash_node_id("gallery.field.mark_key");
pub const MARK_CHANGED: NodeId = hash_node_id("gallery.field.mark_changed");
pub const Z_ORDER: NodeId = hash_node_id("gallery.field.z_order");
pub const MASS: NodeId = hash_node_id("gallery.field.mass");

/// O campo de texto do nome.
pub const NAME: NodeId = hash_node_id("gallery.text.name");

// ── Escolhas ──
pub const LEVEL_LOW: NodeId = hash_node_id("gallery.choice.low");
pub const LEVEL_MID: NodeId = hash_node_id("gallery.choice.mid");
pub const LEVEL_HIGH: NodeId = hash_node_id("gallery.choice.high");
pub const CAP_BUTT: NodeId = hash_node_id("gallery.choice.cap_butt");
pub const CAP_ROUND: NodeId = hash_node_id("gallery.choice.cap_round");
pub const CAP_SQUARE: NodeId = hash_node_id("gallery.choice.cap_square");
pub const BARE_VISIBLE: NodeId = hash_node_id("gallery.bare.visible");
pub const BARE_LOCK: NodeId = hash_node_id("gallery.bare.lock");
pub const SNAP: NodeId = hash_node_id("gallery.check.snap");
pub const BLEND: NodeId = hash_node_id("gallery.dropdown.blend");
pub const BLEND_NORMAL: NodeId = hash_node_id("gallery.dropdown.blend_normal");
pub const BLEND_MULTIPLY: NodeId = hash_node_id("gallery.dropdown.blend_multiply");
pub const BLEND_SCREEN: NodeId = hash_node_id("gallery.dropdown.blend_screen");

// ── Cor ──
pub const FILL_SWATCH: NodeId = hash_node_id("gallery.color.swatch");
pub const FILL_EYE: NodeId = hash_node_id("gallery.color.eye");
pub const FILL_REMOVE: NodeId = hash_node_id("gallery.color.remove");
/// O «+» do cartão Color sem Fill (devolve-o).
pub const FILL_ADD: NodeId = hash_node_id("gallery.color.add");

// ── Ações ──
pub const PRIMARY: NodeId = hash_node_id("gallery.action.primary");
pub const SECONDARY: NodeId = hash_node_id("gallery.action.secondary");
pub const RESET: NodeId = hash_node_id("gallery.action.reset");
pub const PHYSICS_ADD: NodeId = hash_node_id("gallery.action.physics_add");
pub const PHYSICS_REMOVE: NodeId = hash_node_id("gallery.action.physics_remove");

/// `(id, valor de fábrica, passo, arrasto por px)` de cada campo numérico — a tabela que o
/// `populate` regista, o `paint` marca e o ↺ repõe.
pub const FIELDS: [(NodeId, f64, f64, f64); 12] = [
    (POS_X, 120.0, 1.0, 1.0), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (POS_Y, 80.0, 1.0, 1.0), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (ROTATION, 0.0, 1.0, 1.0),
    (SIZE_W, 64.0, 1.0, 1.0), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (SIZE_H, 48.0, 1.0, 1.0), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (OPACITY, 100.0, 1.0, 1.0), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (MARK_FREE, 0.0, 1.0, 1.0),
    (MARK_ANIMATED, 0.0, 1.0, 1.0),
    (MARK_KEY, 0.0, 1.0, 1.0),
    (MARK_CHANGED, 0.0, 1.0, 1.0),
    (Z_ORDER, 3.0, 1.0, 0.1), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
    (MASS, 1.0, 0.1, 0.01), // LITERAL-PX-OK: valor de fábrica do exemplo (dado do documento, não geometria de UI)
];

/// Os segmentos, ícones e opções — registados como `Button` pela mesma tabela.
pub const BUTTONS: [NodeId; 20] = [
    LEVEL_LOW,
    LEVEL_MID,
    LEVEL_HIGH,
    CAP_BUTT,
    CAP_ROUND,
    CAP_SQUARE,
    BARE_VISIBLE,
    BARE_LOCK,
    SNAP,
    BLEND,
    BLEND_NORMAL,
    BLEND_MULTIPLY,
    BLEND_SCREEN,
    FILL_SWATCH,
    FILL_EYE,
    FILL_REMOVE,
    FILL_ADD,
    PRIMARY,
    SECONDARY,
    RESET,
];

/// Os dois do cartão ausente (o «+» e o «−»), registados com os outros.
pub const PHYSICS_BUTTONS: [NodeId; 2] = [PHYSICS_ADD, PHYSICS_REMOVE];
