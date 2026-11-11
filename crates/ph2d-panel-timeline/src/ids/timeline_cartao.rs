//! A Timeline como cartão que se arrasta e redimensiona (dono 10/11): o título, as bordas e cantos, e
//! o «encaixar no lugar de origem» ao lado do «×».

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A faixa do título que arrasta o cartão (`BlenderHitKind::DragHandle`).
pub const TIMELINE_CARD_DRAG: NodeId = hash_node_id("timeline.card.drag");
/// «Snap back to its place»: o cartão volta ao encaixe de origem.
pub const TIMELINE_CARD_DOCK: NodeId = hash_node_id("timeline.card.dock");
/// Borda de baixo.
pub const TIMELINE_RESIZE_B: NodeId = hash_node_id("timeline.resize.b");
/// Borda da esquerda.
pub const TIMELINE_RESIZE_L: NodeId = hash_node_id("timeline.resize.l");
/// Borda da direita.
pub const TIMELINE_RESIZE_R: NodeId = hash_node_id("timeline.resize.r");
/// Canto de cima, à esquerda.
pub const TIMELINE_RESIZE_TL: NodeId = hash_node_id("timeline.resize.tl");
/// Canto de cima, à direita.
pub const TIMELINE_RESIZE_TR: NodeId = hash_node_id("timeline.resize.tr");
/// Canto de baixo, à esquerda.
pub const TIMELINE_RESIZE_BL: NodeId = hash_node_id("timeline.resize.bl");
/// Canto de baixo, à direita.
pub const TIMELINE_RESIZE_BR: NodeId = hash_node_id("timeline.resize.br");
/// O cartão «quem ouve» do marcador com sinal (doc 17 item 8): a lista própria que o clique de fora fecha.
pub const TIMELINE_SIGNAL_CARD: NodeId = hash_node_id("timeline.signal.card");
