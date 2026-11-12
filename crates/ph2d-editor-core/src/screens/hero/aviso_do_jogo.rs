//! ⭐ O aviso do editor parado enquanto o jogo corre na janela própria (dono 11/11): sem ele o
//! editor congelado, com a janela do jogo atrás, lia-se como o app pendurado.

use crate::paint::{paint_text_centered, rect_for_label, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A frase do aviso.
pub(crate) fn texto() -> &'static str {
    ph2d_i18n::tr("shell.janela_do_jogo.running")
}

/// O cartão do aviso, ao meio da janela `viewport`, com a largura da frase.
pub(crate) fn cartao(text_system: &mut TextSystem, viewport: Rect) -> Rect {
    let font = TypeToken::Md.px();
    let w = rect_for_label(text_system.prefix_width(texto(), font)) + 2.0 * Spacing::Xl.px();
    let w = w.min(viewport.w);
    let h = Spacing::Xl2.px() + Spacing::Xl.px();
    Rect::new(
        viewport.x + (viewport.w - w) / 2.0,
        viewport.y + (viewport.h - h) / 2.0,
        w,
        h,
    )
}

/// Escurece a última imagem do editor e pinta o cartão com a frase por cima de tudo.
pub(crate) fn paint(
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    scene.fill_rect(
        crate::paint::rect_to_vello(viewport),
        resolve(ColorToken::BgScrim, theme),
    );
    let r = cartao(text_system, viewport);
    crate::widget::lista_flutuante::Superficie::de(theme).moldura_de_painel(scene, r);
    let font = TypeToken::Md.px();
    paint_text_centered(
        text_system,
        scene,
        texto(),
        r,
        font,
        resolve(ColorToken::Text1, theme),
    );
}
