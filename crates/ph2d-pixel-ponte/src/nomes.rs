//! **O nome de cada passo na Undo History** (P12 H, Aseprite O11): o comando como o artista o
//! chama. A fonte da palavra é a do comando nos atalhos (`pixel.keys.*`) ou o título do diálogo
//! quando existem; `pixel.history.*` só para o que não tem nome noutro sítio.

use ph2d_editor_core::pixel_panel::PixelLayerEffect;
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelEditAction as E, PixelFrameAction as F, PixelLayerAction, PixelRequest,
    PixelSpriteAction as Sp,
};

/// O nome de um passo do menu Edit: o do comando nos atalhos (`pixel.keys.*`) quando lá está.
pub const fn nome_edit(a: E) -> &'static str {
    match a {
        E::FlipH => "pixel.keys.edit.flip_h",
        E::FlipV => "pixel.keys.edit.flip_v",
        E::Rotate180 | E::Rotate90Cw | E::Rotate90Ccw => "pixel.history.rotate",
        E::ShiftLeft | E::ShiftRight | E::ShiftUp | E::ShiftDown => "pixel.history.shift",
        E::Cut => "pixel.keys.edit.cut",
        E::Fill => "pixel.keys.edit.fill",
        E::Stroke => "pixel.keys.edit.stroke",
        E::Guides(_) => "pixel.history.guides",
        _ => "pixel.keys.edit.clear",
    }
}

pub const fn nome_sprite(a: Sp) -> &'static str {
    match a {
        Sp::CanvasSize => "pixel.keys.sprite.canvas_size",
        Sp::SpriteSize => "pixel.keys.sprite.sprite_size",
        Sp::RotateCanvas180 | Sp::RotateCanvas90Cw | Sp::RotateCanvas90Ccw => {
            "pixel.history.rotate"
        }
        Sp::FlipCanvasH => "pixel.keys.edit.flip_h",
        Sp::FlipCanvasV => "pixel.keys.edit.flip_v",
        Sp::Crop => "pixel.history.crop",
        Sp::Trim => "pixel.history.trim",
        Sp::Properties => "pixel.keys.sprite.properties",
    }
}

/// O nome do OK de um diálogo: o título da janela do filtro, ou o do comando.
pub const fn nome_dialogo(c: PixelDialogChoice) -> &'static str {
    match c {
        PixelDialogChoice::CanvasSize { .. } => "pixel.keys.sprite.canvas_size",
        PixelDialogChoice::SpriteSize { .. } => "pixel.keys.sprite.sprite_size",
        PixelDialogChoice::Tween { .. } => "pixel.tween.title",
        PixelDialogChoice::Invert { .. } => "pixel.filter_dialog.invert.title",
        PixelDialogChoice::Despeckle { .. } => "pixel.history.despeckle",
        PixelDialogChoice::Outline { .. } => "pixel.filter_dialog.outline.title",
        PixelDialogChoice::BrightnessContrast { .. } => {
            "pixel.filter_dialog.brightness_contrast.title"
        }
        PixelDialogChoice::HueSaturation { .. } => "pixel.filter_dialog.hue_saturation.title",
        PixelDialogChoice::ColorCurve { .. } => "pixel.filter_dialog.curve.title",
        PixelDialogChoice::Convolution { .. } => "pixel.filter_dialog.convolution.title",
        PixelDialogChoice::ColorRange { .. } | PixelDialogChoice::Modify { .. } => {
            "pixel.history.select"
        }
    }
}

/// O nome do passo de um pedido de camada: o do comando nos atalhos (`pixel.keys.*`) quando lá está.
pub const fn nome_camada(r: &PixelRequest) -> &'static str {
    use PixelLayerAction as A;
    match r {
        PixelRequest::LayerVisible(_) | PixelRequest::LayerSolo(_) => "pixel.keys.layer.visibility",
        PixelRequest::LayerRename { .. } => "pixel.keys.layer.rename",
        PixelRequest::LayerNormal(_) => "pixel.history.layer_normal",
        PixelRequest::LayerEffect(_, PixelLayerEffect::Apply) => {
            "pixel.history.layer_effects_apply"
        }
        PixelRequest::LayerEffect(..) => "pixel.history.layer_effects",
        PixelRequest::LayerAction(a) => match a {
            A::NewLayer => "pixel.keys.layer.new",
            A::NewGroup => "pixel.keys.layer.new_group",
            A::Delete => "pixel.history.delete_layer",
            A::Duplicate => "pixel.history.duplicate_layer",
            A::MoveUp | A::MoveDown => "pixel.history.move_layer",
            A::MergeDown => "pixel.history.merge_down",
            A::Flatten => "pixel.history.flatten",
            A::FlattenVisible => "pixel.history.flatten_visible",
            A::ViaCopy => "pixel.keys.layer.via_copy",
            A::ViaCut => "pixel.keys.layer.via_cut",
            A::PasteAsLayer => "pixel.history.paste_as_layer",
            A::PasteAsReference => "pixel.history.paste_as_reference",
            A::NewTilemap => "pixel.keys.layer.new_tilemap",
            A::ConvertToTilemap => "pixel.history.convert_to_tilemap",
            A::ConvertToImage => "pixel.history.convert_to_image",
            A::PasteAsSprite => "pixel.history.layers",
        },
        _ => "pixel.history.layer_properties",
    }
}

/// O nome do passo de uma acção do menu Frame: o do comando nos atalhos quando lá está.
pub const fn nome_quadro(a: F) -> &'static str {
    match a {
        F::NewFrame => "pixel.keys.frame.new",
        F::NewEmptyFrame => "pixel.keys.frame.new_empty",
        F::DuplicateLinked => "pixel.keys.frame.duplicate_linked",
        F::Delete => "pixel.keys.frame.delete",
        F::Reverse => "pixel.keys.frame.reverse",
        F::LinkPrevious => "pixel.history.link_cels",
        F::Unlink => "pixel.history.unlink_cels",
        F::NewTag => "pixel.history.new_tag",
        F::LoopSection => "pixel.keys.frame.loop_section",
        F::Tween => "pixel.tween.title",
        _ => "pixel.history.move_frame",
    }
}
