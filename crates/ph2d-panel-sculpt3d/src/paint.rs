//! O corpo do painel. Percorre [`crate::rows::SECTIONS`] — a mesma tabela que o
//! `populate` e o `event` percorrem — para que o que é desenhado, o que é
//! registrado e o que despacha não possam discordar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_empty, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    SCULPT3D_SCROLLBAR_ID, paint_scrollbar, paint_slider_with_chip_layout_adaptive,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use crate::state::{self, Sculpt3dPanelState, set_last_content_h, set_last_visible_h};
use crate::{Sculpt3dPanel, rows};

mod body;
/// A cabeça e a cauda da seção do PINCEL — ver o doc do módulo.
mod brush;
/// As fileiras próprias de cada pincel — ver [`brush_fileiras`].
mod brush_fileiras;
/// **O QUE SE FAZ COM UMA MÁSCARA PINTADA** — irmão do [`body`], cortado por
/// assunto quando o transform cruzou o teto de LOC dele.
mod mask_tools;
/// A ferramenta e a referência que ela segue — ver o cabeçalho dele.
mod tool;
/// **OS WIDGETS** do painel. ⚠️ Ele subiu de dentro do [`body`] para cá quando
/// ganhou um SEGUNDO consumidor: o arquivo sempre morou em `paint/`, e a
/// declaração é que estava um nível abaixo do lugar dele.
mod widgets;

pub(crate) fn paint(_state: &mut Sculpt3dPanelState, ctx: &mut PaintCtx) {
    // ⚠️ **DUAS recusas, e nenhuma basta sozinha.** A primeira é a do artista (o
    // painel está fechado); a segunda é a da CENA — sem escultura viva, seis
    // seções de controles não alcançariam nada, e um painel que pinta sobre o
    // vazio é a forma de chrome morto que esta casa varre a cada wave.
    // ⭐ **UMA recusa, e não duas.** Só o artista cala este painel: fechado, ele limpa o rect e
    // sai — e é isso que faz o `panel_at` parar de o devolver no instante em que ele fecha.
    if !ctx.host.panel_visible(Sculpt3dPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::SCULPT3D_PANEL);
        return;
    }
    // ⛔⛔ **ABERTO e sem cena NÃO é motivo para silêncio — medido em 2026-09-09.** A segunda
    // recusa que vivia aqui dizia, com razão para o modelo antigo, que *«um painel que pinta
    // sobre o vazio é chrome morto»*. Com as ABAS o preço inverteu-se: quem está à frente esconde
    // os outros ocupantes, quem não pinta não publica rect, e uma coluna sem rects publicados
    // lê-se **livre** e fecha — levando consigo a fileira de abas e o vizinho VIVO. Medido com o
    // Inspector ao lado: `218` glifos no quadro (só o cromo de base) contra `269`, e nenhuma aba
    // para clicar de volta. ⇒ ele publica o rect e DIZ porquê está vazio.
    let Some(snapshot) = state::current() else {
        let rect: Rect = ctx.slot;
        let theme = ctx.host.theme();
        ctx.host
            .store_mut()
            .set_panel_rect(ids::SCULPT3D_PANEL, rect);
        paint_panel_empty(
            rect,
            Sculpt3dPanel::TITLE.tr(),
            crate::ids::SCULPT3D_CLOSE,
            tr("panel.sculpt3d.empty"),
            ctx.scene,
            ctx.text_system,
            ctx.host.hit_index_mut(),
            theme,
        );
        return;
    };

    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();

    // Publica o rect para o dispatch de roda/clique poder rotear para este painel.
    ctx.host
        .store_mut()
        .set_panel_rect(ids::SCULPT3D_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);

    // Alças de arraste + resize do slot do dock. Reusam os ids do Inspector
    // porque o slot da direita é compartilhado — o delta de resize deve
    // sobreviver à troca entre o Inspector e um painel de mundo.
    {}

    let title_size = paint_panel_title(
        rect,
        Sculpt3dPanel::TITLE.tr(),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        crate::ids::SCULPT3D_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::SCULPT3D_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = body::paint_sections(
        ctx,
        &snapshot,
        rect.x + PANEL_HEAD_PAD,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        body_top - scroll,
    );
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);
}

/// Uma row de slider+chip, da tabela.
pub(crate) fn paint_row(
    ctx: &mut PaintCtx,
    row: &rows::Row,
    value: f32,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // O valor publicado é a verdade; a pista guardada é só o que o arrasto
    // deixou. Semear a pista a partir do valor a cada frame é o que mantém o
    // slider honesto quando o estado muda de qualquer outro lugar — o teclado,
    // um clamp de viewport, um undo.
    let track = row.track_of(value);
    let display = f64::from(value);
    let text = format!("{display:.*}", row.decimals);
    let label = tr(row.label);

    paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        label,
        track,
        display,
        Some(&text),
        row.slider,
        row.chip,
        ph2d_editor_core::widget::property_label_col_w(x, w),
        ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    )
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host.store().scrollbar_visual(SCULPT3D_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        ctx.host
            .hit_index_mut()
            .register(SCULPT3D_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::SCULPT3D_PANEL, content_h);
    store.set_panel_visible_h(ids::SCULPT3D_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::SCULPT3D_PANEL) > max_scroll {
        store.set_panel_scroll(ids::SCULPT3D_PANEL, max_scroll);
    }
}

/// **A porta do READOUT para fora do módulo de pintura.**
///
/// ⚠️ O [`preview`](crate::preview) precisa dizer uma condição na mesma tipografia
/// e no mesmo ritmo das outras linhas de fato do painel, e re-implementá-la lá
/// seria a segunda resposta a *"como este painel escreve um fato?"* — divergindo
/// no dia em que o token do texto secundário mudasse.
pub(crate) fn readout_at(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    text: &str,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    body::readout_for(ctx, text, x, w, y)
}
