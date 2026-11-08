//! Sprite ▸ Properties… (P12 U, O17): o botão e o Run Command abrem/fecham a secção do painel PIXEL;
//! o que ela mostra chega por `PixelPanelInfo` (tamanho, modo, frames, camadas, `user.sprite`) e o
//! cartão User Data volta por `PixelRequest::UserData(PixelOwner::Sprite, _)`: um passo «User Data».

pub(crate) fn pede() {
    ph2d_editor_core::pixel_panel::sprite_props::toggle();
}
