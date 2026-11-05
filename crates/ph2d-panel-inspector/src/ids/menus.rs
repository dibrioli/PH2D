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

/// ⭐ O seletor **Use** sob a linha Image (planos GEF 15 D1 e 16 D1): o grupo e as quatro opções, pela ordem de
/// `InspectorImageUse::ALL`.
pub const INSP_IMAGE_USE_GROUP: NodeId = hash_node_id("insp_image_use_group");
pub const INSP_IMAGE_USE: [NodeId; 4] = [
    hash_node_id("insp_image_use_still"),
    hash_node_id("insp_image_use_repeated"),
    hash_node_id("insp_image_use_animated"),
    hash_node_id("insp_image_use_region"),
];

/// **Replace** do cartão Image: o mesmo pedido do clique na miniatura.
pub const INSP_IMAGE_REPLACE: NodeId = hash_node_id("insp_image_replace");
/// A caixa «Tint each corner» do cartão Advanced.
pub const INSP_SPRITE_CORNERS_ON: NodeId = hash_node_id("insp_sprite_corners_on");

/// **Edit in Pixel** e **Edit in Painter** do cartão Image.
pub const INSP_IMAGE_EDIT_PIXEL: NodeId = hash_node_id("insp_image_edit_pixel");
pub const INSP_IMAGE_EDIT_PAINTER: NodeId = hash_node_id("insp_image_edit_painter");

/// **Keep old size** do recibo do Replace.
pub const INSP_IMAGE_KEEP_SIZE: NodeId = hash_node_id("insp_image_keep_size");
/// **Paint a new image…** do convite do cartão vazio.
pub const INSP_IMAGE_PAINT_NEW: NodeId = hash_node_id("insp_image_paint_new");
/// As imagens recentes do convite.
pub const INSP_IMAGE_RECENT: [NodeId; 3] = [
    hash_node_id("insp_image_recent_0"),
    hash_node_id("insp_image_recent_1"),
    hash_node_id("insp_image_recent_2"),
];

/// Os dois verbos do aviso de folha.
pub const INSP_IMAGE_SHEET_ANIMATED: NodeId = hash_node_id("insp_image_sheet_animated");
pub const INSP_IMAGE_SHEET_REGION: NodeId = hash_node_id("insp_image_sheet_region");

/// Os nove pontos do Pivot, por linhas, a de cima primeiro.
pub const INSP_SPRITE_PIVOT: [NodeId; 9] = [
    hash_node_id("insp_sprite_pivot_0"),
    hash_node_id("insp_sprite_pivot_1"),
    hash_node_id("insp_sprite_pivot_2"),
    hash_node_id("insp_sprite_pivot_3"),
    hash_node_id("insp_sprite_pivot_4"),
    hash_node_id("insp_sprite_pivot_5"),
    hash_node_id("insp_sprite_pivot_6"),
    hash_node_id("insp_sprite_pivot_7"),
    hash_node_id("insp_sprite_pivot_8"),
];
/// O nome do ponto: o clique abre os campos X e Y.
pub const INSP_SPRITE_PIVOT_NOME: NodeId = hash_node_id("insp_sprite_pivot_nome");
/// Os campos do Pivot em Custom.
pub const INSP_SPRITE_PIVOT_XY: [NodeId; 2] = [
    hash_node_id("insp_sprite_pivot_x"),
    hash_node_id("insp_sprite_pivot_y"),
];

/// **Edit Region…** — abre o editor da área de uma sprite em Region (plano GEF 16 D2).
pub const INSP_IMAGE_EDIT_REGION: NodeId = hash_node_id("insp_image_edit_region");

/// «‹ ›» do quadro de uma imagem parada numa folha (plano GEF 15, smoke do dono 27/10).
pub const INSP_IMAGE_FRAME_STEP: [NodeId; 2] = [
    hash_node_id("insp_image_frame_prev"),
    hash_node_id("insp_image_frame_next"),
];

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
