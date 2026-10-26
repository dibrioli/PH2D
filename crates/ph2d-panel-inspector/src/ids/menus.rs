//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/menus.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// M14.5 inspector phase (6.4/§9): "Reimport at current px/m" button
/// shown in the Render Source section when the selected sprite has a
/// live atlas-backed source. Click → `HeroScreen.pending_reimport`
/// gets set with the entity bits the host then drains to recompute
/// `Sprite.size` against the current `ProjectSettings.pixels_per_meter`.
pub const INSP_RENDER_SOURCE_REIMPORT: NodeId = hash_node_id("insp_render_source_reimport");

/// ⭐ «Make it Animated» — o botão do aviso «imagem animada num bloco que repete» (plano GEF 13 D5).
pub const INSP_IMAGE_MAKE_ANIMATED: NodeId = hash_node_id("insp_image_make_animated");
/// A caixa **Tiled** de Texture & Atlas (plano GEF 14 D5).
pub const INSP_IMAGE_TILED: NodeId = hash_node_id("insp_image_tiled");
/// **Width / Height** (m) de uma sprite Tiled, `[x, y]`.
pub const INSP_IMAGE_TILED_SIZE: [NodeId; 2] = [
    hash_node_id("insp_image_tiled_w"),
    hash_node_id("insp_image_tiled_h"),
];

/// W2 Sprite Inspector v2: logical Flip H / Flip V checkboxes in the
/// Render Source section. Toggling dispatches an
/// `EditorAction::InspectorSpriteEdit` with `SpriteFieldEdit::FlipX/FlipY`.
pub const INSP_SPRITE_FLIP_X: NodeId = hash_node_id("insp_sprite_flip_x");

/// Vertical flip checkbox — see [`INSP_SPRITE_FLIP_X`].
pub const INSP_SPRITE_FLIP_Y: NodeId = hash_node_id("insp_sprite_flip_y");
