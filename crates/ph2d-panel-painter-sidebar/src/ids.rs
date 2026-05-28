//! Widget NodeIds para o Painter sidebar.
//!
//! IDs derivados de slugs estáveis via `ph2d_tool_registry::hash_node_id`
//! (FNV-1a 64-bit `const fn`). Adicionar novo widget = pick slug único +
//! adicionar const aqui. Collisions caught por `tests/architecture/
//! node_id_collisions.rs` em editor-core.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Slider de tamanho (size_px).
pub const SIZE_SLIDER: NodeId = hash_node_id("painter_sidebar.size_slider");
/// Chip numeric do size_slider (link via store.link_slider_number).
pub const SIZE_CHIP: NodeId = hash_node_id("painter_sidebar.size_chip");
/// Slider de opacidade.
pub const OPACITY_SLIDER: NodeId = hash_node_id("painter_sidebar.opacity_slider");
/// Chip numeric do opacity_slider.
pub const OPACITY_CHIP: NodeId = hash_node_id("painter_sidebar.opacity_chip");
/// Botão Undo (sidebar; gesture 2-finger tap também dispara).
pub const UNDO_BUTTON: NodeId = hash_node_id("painter_sidebar.undo_button");
/// Botão Redo (sidebar; gesture 3-finger tap também dispara).
pub const REDO_BUTTON: NodeId = hash_node_id("painter_sidebar.redo_button");
/// Modifier square (centro da sidebar) — W2 ship default = eyedropper-
/// while-held. Configurável em W10 Gesture Controls.
pub const MODIFIER_SQUARE: NodeId = hash_node_id("painter_sidebar.modifier_square");
