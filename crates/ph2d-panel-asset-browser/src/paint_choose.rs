//! ⭐ **O SELETOR DE IMAGEM** ([plano GEF 12](../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)) — o
//! *object picker* da Unity / *Quick Load* da Godot. ⛔ Não é uma segunda grelha: é este painel com um ALVO
//! (o objecto cuja ranhura de imagem se clicou). Com alvo a grelha só mostra imagens, a busca toma o foco, e
//! um clique numa imagem escolhe-a e fecha; o `✕` da faixa (ou fechar o painel) desiste.

use crate::ids;
use crate::paint::pad;
use crate::state::{AssetBrowserState, Choosing};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Radius, Spacing, TypeToken};
use std::cell::RefCell;

thread_local! {
    /// O pedido do shell à espera do próximo `paint` (o painel é dono do estado; o shell só pede).
    static REQUEST: RefCell<Option<Choosing>> = const { RefCell::new(None) };
}

/// ⭐ **O shell pede o seletor para a sprite `entity_bits`** (cujo nome a faixa mostra).
pub fn choose_image_for(entity_bits: u64, name: String) {
    REQUEST.with(|r| *r.borrow_mut() = Some(Choosing { entity_bits, name }));
}

/// O pedido pendente passa para o estado (chamado no início do `paint` do painel aberto).
pub(crate) fn take_request(state: &mut AssetBrowserState, ctx: &mut PaintCtx) {
    if let Some(c) = REQUEST.with(|r| r.borrow_mut().take()) {
        state.choosing = Some(c);
        state.related = None;
        ctx.host.store_mut().set_focus(Some(ids::ASSET_SEARCH));
        crate::event::reset_scroll(ctx.host);
    }
}

/// Pinta a faixa *«Choose the image for …»* com o `✕` e devolve o `y` de baixo.
pub(crate) fn paint(
    state: &AssetBrowserState,
    ctx: &mut PaintCtx,
    rect: Rect,
    y: f32,
    col_w: f32,
) -> f32 {
    let Some(choosing) = &state.choosing else {
        return y;
    };
    let theme = ctx.host.theme();
    let x = rect.x + col_w + pad();
    let w = (rect.x + rect.w - pad() - x).max(0.0);
    let row_h = Density::Compact.row_h_px();
    if w <= row_h {
        return y;
    }
    let band = Rect::new(x, y, w, row_h);
    fill_rounded_rect(
        ctx.scene,
        band,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::AccentSoft, theme),
    );
    let cancel = Rect::new(band.x + band.w - row_h, band.y, row_h, row_h);
    paint_icon_button(
        cancel,
        IconGlyph::Builtin(ph2d_editor_core::icons::IconId::Close),
        IconButtonStyle::Plain,
        ctx.host.store().button_visual(ids::ASSET_CHOOSE_CANCEL),
        ctx.scene,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_CHOOSE_CANCEL, cancel);
    let label = ph2d_i18n::tr_with(
        "panel.asset_browser.pick.choose_for",
        &[("name", &choosing.name)],
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        band.x + Spacing::Xs.px(),
        band.y + (row_h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (band.w - row_h - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    y + row_h + ph2d_tokens::control_gap_px()
}
