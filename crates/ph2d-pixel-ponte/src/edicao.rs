//! **Os menus Edit e Sprite** do chrome para o núcleo (saiu de `ph2d-app-pixel::edicao`, P14).

use ph2d_editor_core::pixel_panel::{
    PixelEditAction as E, PixelModify, PixelRequest, PixelSizeMethod,
};
use ph2d_pixel_core::canvas::SizeMethod;
use ph2d_pixel_core::selection::Modify;

/// O *Modify* da selecção no vocabulário do núcleo.
#[must_use]
pub fn modify_in(k: PixelModify) -> Modify {
    match k {
        PixelModify::Expand => Modify::Expand,
        PixelModify::Contract => Modify::Contract,
        PixelModify::Border => Modify::Border,
    }
}

/// O método de reamostragem do chrome no vocabulário do núcleo.
#[must_use]
pub fn method_in(m: PixelSizeMethod) -> SizeMethod {
    match m {
        PixelSizeMethod::Nearest => SizeMethod::Nearest,
        PixelSizeMethod::Bilinear => SizeMethod::Bilinear,
        PixelSizeMethod::RotSprite => SizeMethod::RotSprite,
    }
}

/// O pedido larga a moldura solta antes (o Aseprite assenta as células levantadas antes de outro
/// comando): tudo o que muda o documento, salvo a própria moldura.
#[must_use]
pub fn assenta_antes(r: &PixelRequest) -> bool {
    !matches!(
        r,
        PixelRequest::Tool(_)
            | PixelRequest::Group(_)
            | PixelRequest::Options(_)
            | PixelRequest::Fg(_)
            | PixelRequest::Bg(_)
            | PixelRequest::Swap
            | PixelRequest::Swatch { .. }
            | PixelRequest::SortKey(_)
            | PixelRequest::TransformField { .. }
            | PixelRequest::Edit(E::Commit | E::Cancel | E::Transform | E::Copy | E::CopyMerged)
    )
}
