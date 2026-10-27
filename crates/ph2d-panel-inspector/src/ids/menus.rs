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

/// ⭐ O seletor **Use** sob a linha Image (plano GEF 15 D1): o grupo e as três opções, pela ordem de
/// `InspectorImageUse::ALL`.
pub const INSP_IMAGE_USE_GROUP: NodeId = hash_node_id("insp_image_use_group");
pub const INSP_IMAGE_USE: [NodeId; 3] = [
    hash_node_id("insp_image_use_still"),
    hash_node_id("insp_image_use_repeated"),
    hash_node_id("insp_image_use_animated"),
];

/// A célula `i` da miniatura da folha (plano GEF 15 D2), derivada da semente; o recurso é a grelha da imagem.
#[must_use]
pub const fn insp_image_cell(i: u32) -> NodeId {
    let mut z = hash_node_id("insp_image_cell").0 ^ (i as u64).wrapping_add(1);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    NodeId(z ^ (z >> 31))
}
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
