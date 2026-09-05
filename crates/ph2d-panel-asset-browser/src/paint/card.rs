//! **UM cartão da grade de assets** — filho cortado do `paint.rs` por responsabilidade quando a
//! porta da moldura do tema (wave 4 do redesenho, 2026-09-05) o fez passar o teto de LOC do
//! painel. O pai orquestra a grade; este sabe pintar uma célula.

use super::*;

/// **Um cartão.** Quadrado + nome + detalhe.
///
/// ⚠️ **O que fica por baixo da miniatura é o FUNDO DO CANVAS**, e a cor dominante do asset (A2)
/// só aparece enquanto não há miniatura — a lei, com o porquê das duas metades, vive no
/// [`crate::card_backdrop`].
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    cell: f32,
    name: &str,
    detail: &str,
    swatch: [u8; 4],
    thumb_img: Option<&ph2d_asset_index::Thumb>,
    key: ph2d_asset_index::AssetRef,
) {
    let hot = ctx.host.store().button_visual(id).0 != ph2d_editor_core::widget::ButtonState::Normal;
    let thumb = Rect::new(rect.x, rect.y, cell, cell);
    fill_rounded_rect(
        ctx.scene,
        thumb,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let inset = Spacing::Xs.px();
    fill_rounded_rect(
        ctx.scene,
        Rect::new(
            thumb.x + inset,
            thumb.y + inset,
            (thumb.w - inset * 2.0).max(0.0),
            (thumb.h - inset * 2.0).max(0.0),
        ),
        Radius::Sm.px(),
        ph2d_editor_core::paint::token_to_vello(crate::card_backdrop::card_backdrop(
            theme,
            swatch,
            thumb_img.is_some(),
        )),
    );
    // ⭐⭐ **A miniatura por CIMA do fundo** (wave A6): uma imagem com alfa deixa ver o que está
    // atrás dela, e o que está atrás é o mesmo fundo em que o objecto pousa na tela.
    if let Some(t) = thumb_img {
        crate::paint_thumb::paint_thumb(ctx, key, t, thumb, inset);
    }
    // ⭐ Pela porta do TEMA: a miniatura é plana num tema moderno (sem moldura em repouso).
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        thumb,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        theme,
        if hot {
            ph2d_tokens::visuals::Feel::Hovered
        } else {
            ph2d_tokens::visuals::Feel::Rest
        },
        StrokeToken::Default.px(),
        resolve(
            if hot {
                ColorToken::Accent
            } else {
                ColorToken::Border
            },
            theme,
        ),
    );
    // ⚠️ **As duas linhas do rótulo saem de TOKENS, não de múltiplos do tamanho da fonte.** A 1.ª
    // versão usava `font * 1.2` para o avanço e `font * 0.85` para o detalhe, e o gate dos números
    // mágicos apanhou-a: um múltiplo escolhido à mão não segue a escala tipográfica quando ela
    // mudar.
    let name_y = thumb.y + thumb.h + Spacing::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        name,
        rect.x,
        name_y,
        TypeToken::Sm.px(),
        rect.w,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        detail,
        rect.x,
        name_y + TypeToken::Sm.px() + Spacing::Xxs.px(),
        TypeToken::Xs.px(),
        rect.w,
        resolve(ColorToken::Text2, theme),
    );
    // ⚠️ **O rectângulo de gesto é o CARTÃO INTEIRO**, não só a miniatura — o nome faz parte do
    // alvo, que é o que todo navegador de ficheiros faz.
    ctx.host.hit_index_mut().register(id, rect);
}
