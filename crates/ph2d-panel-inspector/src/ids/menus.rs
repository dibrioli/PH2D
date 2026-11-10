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

/// A caixa «Tint each corner» do cartão Advanced.
pub const INSP_SPRITE_CORNERS_ON: NodeId = hash_node_id("insp_sprite_corners_on");

/// **Edit in Painter** do cartão Image.
pub const INSP_IMAGE_EDIT_PAINTER: NodeId = hash_node_id("insp_image_edit_painter");
/// **Pixel** do cartão de um Pixel Sprite.
pub const INSP_IMAGE_EDIT_PIXEL: NodeId = hash_node_id("insp_image_edit_pixel");

/// **Keep old size** do recibo do Replace.
pub const INSP_IMAGE_KEEP_SIZE: NodeId = hash_node_id("insp_image_keep_size");
/// **Paint a new image…** do convite do cartão vazio.
pub const INSP_IMAGE_PAINT_NEW: NodeId = hash_node_id("insp_image_paint_new");

/// Os dois verbos do aviso de folha.
pub const INSP_IMAGE_SHEET_ANIMATED: NodeId = hash_node_id("insp_image_sheet_animated");
pub const INSP_IMAGE_SHEET_REGION: NodeId = hash_node_id("insp_image_sheet_region");
/// «Cut again», no aviso de imagem trocada.
pub const INSP_IMAGE_CUT_AGAIN: NodeId = hash_node_id("insp_image_cut_again");
/// «Keep the cut», no aviso de imagem trocada.
pub const INSP_IMAGE_KEEP_CUT: NodeId = hash_node_id("insp_image_keep_cut");
/// A ranhura **Normal Map** do cartão Image (abre o seletor) e o **None** que a tira (P14).
pub const INSP_IMAGE_NORMAL_SLOT: NodeId = hash_node_id("insp_image_normal_slot");
pub const INSP_IMAGE_NORMAL_NONE: NodeId = hash_node_id("insp_image_normal_none");

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
/// O botão ao lado da grade: abre os campos X e Y.
pub const INSP_SPRITE_PIVOT_NOME: NodeId = hash_node_id("insp_sprite_pivot_nome");
/// O campo Pivot de uma Animated Sprite (GEF 29): abre o editor de animações com o Align ligado.
pub const INSP_SPRITE_PIVOT_EDITOR: NodeId = hash_node_id("insp_sprite_pivot_editor");
/// **Keep in place**: o Mirror object espelha tudo, e a imagem fica onde está.
pub const INSP_SPRITE_MIRROR_KEEP: NodeId = hash_node_id("insp_sprite_mirror_keep");
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

/// ⭐ As folhas de um Animated Sprite (plano GEF 26 §3.3). A miniatura, com duas ou mais: abre o dropdown.
pub const INSP_IMAGE_SHEET_PICK: NodeId = hash_node_id("insp_image_sheet_pick");
/// **+ Load sheet** sob a moldura.
pub const INSP_IMAGE_SHEET_LOAD: NodeId = hash_node_id("insp_image_sheet_load");
/// **Delete sheet** sob a moldura (só com duas ou mais).
pub const INSP_IMAGE_SHEET_DELETE: NodeId = hash_node_id("insp_image_sheet_delete");
/// O véu do dropdown aberto: o clique fora fecha-o sem chegar a ninguém.
pub const INSP_IMAGE_SHEET_SCRIM: NodeId = hash_node_id("insp_image_sheet_scrim");
/// **Quantas folhas têm item no dropdown.** O recurso é o índice `u8` do `ShowSheet`, menos o `255` do quadro vazio
/// (`AnimFrameRef::EMPTY_SHEET`): toda folha que o modelo exprime tem item.
pub const MAX_SHEET_ITEMS: usize = 255;
/// O item da folha `k` no dropdown.
pub const INSP_IMAGE_SHEET_ITEM: [NodeId; MAX_SHEET_ITEMS] =
    super::inspector_audio::tabela_de_linha(hash_node_id("insp_image_sheet_item"));
/// **Remove empty frames** no cartão Animation.
pub const INSP_ANIM_REMOVE_EMPTY_FRAMES: NodeId = hash_node_id("insp_anim_remove_empty_frames");
/// **Delete empty animations** no cartão Animation.
pub const INSP_ANIM_DELETE_EMPTY_ANIMS: NodeId = hash_node_id("insp_anim_delete_empty_anims");
/// O selo da tinta do Painter a caminho da prévia do cartão Animation (só a dica: não se clica).
pub const INSP_ANIM_PREVIEW_SYNC: NodeId = hash_node_id("insp_anim_preview_sync");
/// P13 L: a *Light* do cartão Pixel Sprite (*Smooth* · *Palette*).
pub const INSP_PIXEL_LIGHT_SMOOTH: NodeId = hash_node_id("insp_pixel_light_smooth");
pub const INSP_PIXEL_LIGHT_PALETTE: NodeId = hash_node_id("insp_pixel_light_palette");

/// P14 §6: o *Pivot* do cartão Pixel Sprite — a lista e as linhas (0 = *Center*, depois as slices
/// com pivô por nome; 64 slices, a tabela da lista de animações).
pub const INSP_PIXEL_PIVOT_DD: NodeId = hash_node_id("insp_pixel_pivot_dd");
pub const INSP_PIXEL_PIVOT_OPT: [NodeId; 65] = {
    let base = hash_node_id("insp_pixel_pivot_opt");
    let mut t = [base; 65];
    let mut i = 0;
    while i < 65 {
        t[i] = NodeId(base.0.wrapping_add(i as u64));
        i += 1;
    }
    t
};

/// **O Look da forma de colisão** (GEF02 onda 8): os nove pontos do Pivot (por linhas, a de cima primeiro), o botão
/// que abre os campos, os campos X e Y (metros a partir do centro da forma) e a Opacity (barra 0..1 e o chip em %).
pub const INSP_SHAPE_PIVOT: [NodeId; 9] = [
    hash_node_id("insp_shape_pivot_0"),
    hash_node_id("insp_shape_pivot_1"),
    hash_node_id("insp_shape_pivot_2"),
    hash_node_id("insp_shape_pivot_3"),
    hash_node_id("insp_shape_pivot_4"),
    hash_node_id("insp_shape_pivot_5"),
    hash_node_id("insp_shape_pivot_6"),
    hash_node_id("insp_shape_pivot_7"),
    hash_node_id("insp_shape_pivot_8"),
];
pub const INSP_SHAPE_PIVOT_NOME: NodeId = hash_node_id("insp_shape_pivot_nome");
pub const INSP_SHAPE_PIVOT_XY: [NodeId; 2] = [
    hash_node_id("insp_shape_pivot_x"),
    hash_node_id("insp_shape_pivot_y"),
];
pub const INSP_SHAPE_OPACITY: NodeId = hash_node_id("insp_shape_opacity");
/// A amostra Color (abre o seletor de cor; a cor vive na tabela `widget_colors`).
pub const INSP_SHAPE_TINT_SWATCH: NodeId = hash_node_id("insp_shape_tint_swatch");
pub const INSP_SHAPE_OPACITY_CHIP: NodeId = hash_node_id("insp_shape_opacity_chip");
