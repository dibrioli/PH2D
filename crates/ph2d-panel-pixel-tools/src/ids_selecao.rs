//! Os ids da P6 no painel PIXEL: as opções das ferramentas de selecção, a moldura solta e os
//! botões dos menus Select, Edit e Sprite. ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::{
    PixelEditAction as E, PixelModify, PixelRequest, PixelResample, PixelSelectAction as S,
    PixelSelectMode, PixelSpriteAction as Sp, PixelTransformField,
};
use ph2d_tool_registry::hash_node_id;

/// A varinha: tolerância, contígua, conectividade 4 · 8, amostra camada · visíveis.
pub const WAND_TOLERANCE: NodeId = hash_node_id("pixel.opt.wand_tolerance");
pub const WAND_CONTIGUOUS: NodeId = hash_node_id("pixel.opt.wand_contiguous");
pub const WAND_CONNECT_4: NodeId = hash_node_id("pixel.opt.wand_connect4");
pub const WAND_CONNECT_8: NodeId = hash_node_id("pixel.opt.wand_connect8");
pub const WAND_SAMPLE_LAYER: NodeId = hash_node_id("pixel.opt.wand_sample_layer");
pub const WAND_SAMPLE_VISIBLE: NodeId = hash_node_id("pixel.opt.wand_sample_visible");
/// O Transform: *Auto Select Layer*.
pub const AUTO_SELECT_LAYER: NodeId = hash_node_id("pixel.opt.auto_select_layer");

/// Os quatro modos de juntar (pintar, registar e despachar leem a MESMA tabela).
pub const SELECT_MODES: [(PixelSelectMode, NodeId); 4] = [
    (
        PixelSelectMode::Replace,
        hash_node_id("pixel.opt.select.replace"),
    ),
    (PixelSelectMode::Add, hash_node_id("pixel.opt.select.add")),
    (
        PixelSelectMode::Subtract,
        hash_node_id("pixel.opt.select.subtract"),
    ),
    (
        PixelSelectMode::Intersect,
        hash_node_id("pixel.opt.select.intersect"),
    ),
];

/// A amostragem da moldura solta.
pub const RESAMPLES: [(PixelResample, NodeId); 2] = [
    (PixelResample::Fast, hash_node_id("pixel.opt.resample.fast")),
    (
        PixelResample::RotSprite,
        hash_node_id("pixel.opt.resample.rotsprite"),
    ),
];

/// Os campos da moldura solta: o campo, o id e a faixa `(min, max)` do domínio.
pub const XFORM_FIELDS: [(PixelTransformField, NodeId, (f64, f64)); 6] = [
    (
        PixelTransformField::X,
        hash_node_id("pixel.xform.x"),
        (-65535.0, 65535.0), // LITERAL-PX-OK: domínio do campo (o lado máximo do sprite, u16)
    ),
    (
        PixelTransformField::Y,
        hash_node_id("pixel.xform.y"),
        (-65535.0, 65535.0), // LITERAL-PX-OK: domínio do campo (o lado máximo do sprite, u16)
    ),
    (
        PixelTransformField::W,
        hash_node_id("pixel.xform.w"),
        (1.0, 65535.0), // LITERAL-PX-OK: domínio do campo (o lado máximo do sprite, u16)
    ),
    (
        PixelTransformField::H,
        hash_node_id("pixel.xform.h"),
        (1.0, 65535.0), // LITERAL-PX-OK: domínio do campo (o lado máximo do sprite, u16)
    ),
    (
        PixelTransformField::Angle,
        hash_node_id("pixel.xform.angle"),
        (-180.0, 180.0), // LITERAL-PX-OK: domínio do ângulo em graus
    ),
    (
        PixelTransformField::Skew,
        hash_node_id("pixel.xform.skew"),
        (-89.0, 89.0), // LITERAL-PX-OK: o MAX_SKEW_DEG do ph2d-pixel-core (a 90° a tangente não existe)
    ),
];

/// Os botões do menu Select: o id, o pedido e o rótulo, na ordem em que se pintam.
pub const SELECT_ACTIONS: [(NodeId, PixelRequest, &str); 10] = [
    (
        hash_node_id("pixel.select.all"),
        PixelRequest::Select(S::All),
        "panel.pixel.select.all",
    ),
    (
        hash_node_id("pixel.select.deselect"),
        PixelRequest::Select(S::Deselect),
        "panel.pixel.select.deselect",
    ),
    (
        hash_node_id("pixel.select.reselect"),
        PixelRequest::Select(S::Reselect),
        "panel.pixel.select.reselect",
    ),
    (
        hash_node_id("pixel.select.inverse"),
        PixelRequest::Select(S::Inverse),
        "panel.pixel.select.inverse",
    ),
    (
        hash_node_id("pixel.select.color_range"),
        PixelRequest::Select(S::ColorRange),
        "panel.pixel.select.color_range",
    ),
    (
        hash_node_id("pixel.select.expand"),
        PixelRequest::Select(S::Modify(PixelModify::Expand)),
        "panel.pixel.select.expand",
    ),
    (
        hash_node_id("pixel.select.contract"),
        PixelRequest::Select(S::Modify(PixelModify::Contract)),
        "panel.pixel.select.contract",
    ),
    (
        hash_node_id("pixel.select.border"),
        PixelRequest::Select(S::Modify(PixelModify::Border)),
        "panel.pixel.select.border",
    ),
    (
        hash_node_id("pixel.select.load_msk"),
        PixelRequest::Select(S::LoadMsk),
        "panel.pixel.select.load_msk",
    ),
    (
        hash_node_id("pixel.select.save_msk"),
        PixelRequest::Select(S::SaveMsk),
        "panel.pixel.select.save_msk",
    ),
];

/// Os botões do menu Edit e da moldura (Apply = `Enter`, Cancel = `Esc`).
pub const EDIT_ACTIONS: [(NodeId, PixelRequest, &str); 17] = [
    (
        hash_node_id("pixel.edit.copy"),
        PixelRequest::Edit(E::Copy),
        "panel.pixel.edit.copy",
    ),
    (
        hash_node_id("pixel.edit.cut"),
        PixelRequest::Edit(E::Cut),
        "panel.pixel.edit.cut",
    ),
    (
        hash_node_id("pixel.edit.copy_merged"),
        PixelRequest::Edit(E::CopyMerged),
        "panel.pixel.edit.copy_merged",
    ),
    (
        hash_node_id("pixel.edit.paste"),
        PixelRequest::Edit(E::Paste),
        "panel.pixel.edit.paste",
    ),
    (
        hash_node_id("pixel.edit.clear"),
        PixelRequest::Edit(E::Clear),
        "panel.pixel.edit.clear",
    ),
    (
        hash_node_id("pixel.edit.transform"),
        PixelRequest::Edit(E::Transform),
        "panel.pixel.edit.transform",
    ),
    (
        hash_node_id("pixel.edit.flip_h"),
        PixelRequest::Edit(E::FlipH),
        "panel.pixel.edit.flip_h",
    ),
    (
        hash_node_id("pixel.edit.flip_v"),
        PixelRequest::Edit(E::FlipV),
        "panel.pixel.edit.flip_v",
    ),
    (
        hash_node_id("pixel.edit.rotate180"),
        PixelRequest::Edit(E::Rotate180),
        "panel.pixel.edit.rotate180",
    ),
    (
        hash_node_id("pixel.edit.rotate_cw"),
        PixelRequest::Edit(E::Rotate90Cw),
        "panel.pixel.edit.rotate_cw",
    ),
    (
        hash_node_id("pixel.edit.rotate_ccw"),
        PixelRequest::Edit(E::Rotate90Ccw),
        "panel.pixel.edit.rotate_ccw",
    ),
    (
        hash_node_id("pixel.edit.shift_left"),
        PixelRequest::Edit(E::ShiftLeft),
        "panel.pixel.edit.shift_left",
    ),
    (
        hash_node_id("pixel.edit.shift_right"),
        PixelRequest::Edit(E::ShiftRight),
        "panel.pixel.edit.shift_right",
    ),
    (
        hash_node_id("pixel.edit.shift_up"),
        PixelRequest::Edit(E::ShiftUp),
        "panel.pixel.edit.shift_up",
    ),
    (
        hash_node_id("pixel.edit.shift_down"),
        PixelRequest::Edit(E::ShiftDown),
        "panel.pixel.edit.shift_down",
    ),
    (
        hash_node_id("pixel.edit.apply"),
        PixelRequest::Edit(E::Commit),
        "panel.pixel.edit.apply",
    ),
    (
        hash_node_id("pixel.edit.cancel"),
        PixelRequest::Edit(E::Cancel),
        "panel.pixel.edit.cancel",
    ),
];

/// Os botões do menu Sprite.
pub const SPRITE_ACTIONS: [(NodeId, PixelRequest, &str); 9] = [
    (
        hash_node_id("pixel.sprite.canvas_size"),
        PixelRequest::Sprite(Sp::CanvasSize),
        "panel.pixel.sprite.canvas_size",
    ),
    (
        hash_node_id("pixel.sprite.sprite_size"),
        PixelRequest::Sprite(Sp::SpriteSize),
        "panel.pixel.sprite.sprite_size",
    ),
    (
        hash_node_id("pixel.sprite.rotate180"),
        PixelRequest::Sprite(Sp::RotateCanvas180),
        "panel.pixel.sprite.rotate180",
    ),
    (
        hash_node_id("pixel.sprite.rotate_cw"),
        PixelRequest::Sprite(Sp::RotateCanvas90Cw),
        "panel.pixel.sprite.rotate_cw",
    ),
    (
        hash_node_id("pixel.sprite.rotate_ccw"),
        PixelRequest::Sprite(Sp::RotateCanvas90Ccw),
        "panel.pixel.sprite.rotate_ccw",
    ),
    (
        hash_node_id("pixel.sprite.flip_h"),
        PixelRequest::Sprite(Sp::FlipCanvasH),
        "panel.pixel.sprite.flip_h",
    ),
    (
        hash_node_id("pixel.sprite.flip_v"),
        PixelRequest::Sprite(Sp::FlipCanvasV),
        "panel.pixel.sprite.flip_v",
    ),
    (
        hash_node_id("pixel.sprite.crop"),
        PixelRequest::Sprite(Sp::Crop),
        "panel.pixel.sprite.crop",
    ),
    (
        hash_node_id("pixel.sprite.trim"),
        PixelRequest::Sprite(Sp::Trim),
        "panel.pixel.sprite.trim",
    ),
];
