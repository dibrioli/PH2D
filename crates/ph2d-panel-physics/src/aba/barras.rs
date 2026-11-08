//! ⭐ As barras de rolagem da grade (onda 9f): a calha e o polegar na língua das barras do app (`Bg3` na calha, a cor
//! do polegar pela porta `thumb_color`: repouso, sob o ponteiro, a arrastar). A geometria é a da [`super::Geometria`].

use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas};
use ph2d_editor_core::widget::{SCROLLBAR_W, ScrollbarState, scrollbar_thumb_color as thumb_color};
use ph2d_tokens::ColorToken;

use super::Geometria;

/// Pinta as barras que a grade precisa (nenhuma, se tudo cabe).
pub(super) fn pinta(ctx: &mut PaintCtx, g: &Geometria) {
    let theme = ctx.host.theme();
    let ponteiro = ctx.host.store().last_pointer();
    let arrasto = super::arrastando();
    let barras = [
        (g.barra_vertical(), g.polegar_vertical(), true),
        (g.barra_horizontal(), g.polegar_horizontal(), false),
    ];
    camada_das_listas::por_cima(ctx.scene, |cena| {
        for (calha, polegar, vertical) in barras {
            let (Some(calha), Some(polegar)) = (calha, polegar) else {
                continue;
            };
            let raio = SCROLLBAR_W * 0.5;
            fill_rounded_rect(cena, calha, raio, resolve(ColorToken::Bg3, theme));
            let sob = ponteiro.is_some_and(|[x, y]| polegar.contains(x, y));
            let estado = match arrasto {
                Some(v) if v == vertical => ScrollbarState::Dragging,
                _ if sob => ScrollbarState::Hovered,
                _ => ScrollbarState::Normal,
            };
            fill_rounded_rect(
                cena,
                polegar,
                raio,
                thumb_color((estado, f32::from(u8::from(sob))), theme),
            );
        }
    });
}
