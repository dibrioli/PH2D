//! O quadro da linha do tempo — porta de visibilidade (+ limpeza do rect) · moldura e título · a
//! barra (navegar, tocar, papel vegetal, o menu Frame) · a grelha ([`crate::paint_grid`]) e as
//! propriedades ([`crate::paint_props`]) lado a lado, tudo recortado à banda.
//!
//! ⭐ O rect É `ctx.slot` — a banda do encaixe `Bottom` (a mesma lei da tira do Flip).

use crate::state::current_pixel;
use crate::{PixelTimelinePanel, PixelTimelinePanelState, event, ids};
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, rect_for_label, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelPanelInfo};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title, paint_segmented_button_in_group,
    panel_close_button_rect,
};
use ph2d_editor_core::widget::{
    Button, GroupCell, GroupPos, IconButtonStyle, IconGlyph, paint_icon_button_in_group,
    segment_rects_for,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

pub(crate) fn paint(st: &mut PixelTimelinePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PixelTimelinePanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PIXEL_TIMELINE_PANEL);
        st.direction_open = None;
        return;
    }
    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PIXEL_TIMELINE_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        PixelTimelinePanel::TITLE.tr(),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(rect, ids::CLOSE, ctx.host.hit_index_mut(), ctx.scene, theme);

    let info = current_pixel();
    event::register_frame(ctx.host.store_mut(), &info);
    event::seed_renames(st, ctx.host.store_mut());
    espelha_campos(ctx, &info);

    // ⭐ A barra mora na linha do TÍTULO, à direita dele (a banda de fábrica tem 240 px: cada linha
    // de barra a menos é uma camada a mais à vista); só o que não cabe desce para a linha seguinte.
    let title_bottom = rect.y + PANEL_TITLE_BASELINE + title_size;
    let inner = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        rect.y,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        // Em baixo só o vão pequeno: a banda de fábrica (240 px) mostra assim o cabeçalho, as tags
        // e QUATRO camadas (medido na cena 8 a 1930×1040: as filas começam 102 px abaixo do topo
        // do painel e o passo é 25 px; com o PANEL_HEAD_PAD em baixo cabiam três e meia).
        (rect.h - Spacing::Sm.px()).max(0.0),
    );
    ctx.scene.push_clip(&rect_to_vello(inner));
    ctx.host.hit_index_mut().push_clip(inner);
    st.direction_open = None;
    if info.editing {
        let titulo_w = ctx
            .text_system
            .prefix_width(PixelTimelinePanel::TITLE.tr(), title_size);
        let linha = Rect::new(
            inner.x + titulo_w + Spacing::Lg.px(),
            rect.y + PANEL_TITLE_BASELINE + (title_size - ROW_H_PX) * 0.5,
            (inner.w - titulo_w - Spacing::Lg.px() - PANEL_HEADER_CLOSE_RESERVE).max(0.0),
            ROW_H_PX,
        );
        let fundo = barra(ctx, &info, linha, inner);
        let y = fundo.max(title_bottom) + Spacing::Sm.px();
        let body = Rect::new(inner.x, y, inner.w, (inner.y + inner.h - y).max(0.0));
        let gap = Spacing::Md.px();
        // As propriedades: até três colunas de painel (Frame · Cel · Tag) na metade da banda.
        let n = crate::paint_props::colunas_em(body.w * 0.5);
        let props_w = crate::paint_props::largura(n).min(body.w * 0.5);
        let grade = Rect::new(body.x, body.y, (body.w - props_w - gap).max(0.0), body.h);
        crate::paint_grid::paint(st, ctx, &info, grade);
        let props = Rect::new(body.x + body.w - props_w, body.y, props_w, body.h);
        crate::paint_props::paint(st, ctx, &info, props, n);
    } else {
        paint_text(
            ctx.text_system,
            ctx.scene,
            tr("panel.pixel.timeline.empty"),
            inner.x,
            title_bottom + Spacing::Sm.px() + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            inner.w.max(1.0),
            resolve(ColorToken::Text3, theme),
        );
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();

    // O menu da direcção, por cima de tudo (fora do recorte).
    if let Some((chip, cur)) = st.direction_open {
        let dd = ph2d_editor_core::widget::Dropdown::new(
            ids::TAG_DIRECTION,
            "",
            crate::paint_props::direction_options(),
        )
        .selected(cur)
        .open(true);
        let viewport = ctx.viewport;
        ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, viewport, theme, None);
    }
    // Re-regista o X depois do corpo (o último registo vence).
    ctx.host
        .hit_index_mut()
        .register(ids::CLOSE, panel_close_button_rect(rect));
}

/// Os campos mostram o MODELO (o instantâneo); a edição em curso de um campo com foco fica.
fn espelha_campos(ctx: &mut PaintCtx, info: &PixelPanelInfo) {
    let pos = event::activo(info);
    let dur = pos.map(|p| info.frames[p].duration_ms);
    let cel = crate::paint_props::cel_activo(info);
    let tag = event::tag_activa(info).map(|t| info.tags[t].repeat);
    let valores = [
        (ids::FRAME_DURATION, dur.map(f64::from)),
        (ids::CEL_OPACITY, cel.map(|c| f64::from(c.opacity))),
        (ids::CEL_Z, cel.map(|c| f64::from(c.z))),
        (ids::TAG_REPEAT, tag.map(f64::from)),
    ];
    let store = ctx.host.store_mut();
    let foco = store.focus_id();
    for (id, v) in valores {
        if let Some(v) = v
            && foco != Some(id)
            && store.number_value(id) != Some(v)
        {
            store.set_number_value(id, v);
        }
    }
}

/// **A barra**: navegar e tocar, o menu Frame (ícones com a dica do nome) e o papel vegetal (dois
/// interruptores com palavra) — cada grupo encostado, os grupos separados pelo vão. Começa em
/// `linha` (a linha do título); um grupo que não cabe desce para uma linha inteira de `inner`
/// (nunca transborda). Devolve o fundo da última linha.
fn barra(ctx: &mut PaintCtx, info: &PixelPanelInfo, linha: Rect, inner: Rect) -> f32 {
    let theme = ctx.host.theme();
    let (h, gap) = (ROW_H_PX, Spacing::Md.px());
    let (mut x, mut y, mut fim) = (linha.x, linha.y, linha.x + linha.w);
    let mut quebra = |x: &mut f32, y: &mut f32, w: f32| {
        if *x + w > fim {
            *x = inner.x;
            *y += ph2d_tokens::row_pitch_px();
            fim = inner.x + inner.w;
        }
    };
    let transporte = ids::TRANSPORT.iter().map(|(a, id, icon, _)| {
        let icon = if *a == PixelFrameAction::Play && info.playing {
            IconId::Stop
        } else {
            *icon
        };
        (*id, icon)
    });
    let mut grupos: Vec<Vec<(ph2d_a11y::NodeId, IconId)>> = vec![transporte.collect()];
    for g in ids::ACTION_GROUPS {
        grupos.push(
            g.iter()
                .map(|&i| (ids::ACTIONS[i].1, ids::ACTIONS[i].2))
                .collect(),
        );
    }
    for grupo in grupos {
        let n = grupo.len();
        #[expect(clippy::cast_precision_loss, reason = "≤ cinco botões")]
        let w = n as f32 * h;
        quebra(&mut x, &mut y, w);
        // Os ícones são quadrados (a altura da linha), encostados como um grupo.
        for (i, (id, icon)) in grupo.into_iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ cinco botões")]
            let r = Rect::new(x + i as f32 * h, y, h, h);
            let cell = GroupCell {
                col: GroupPos::of(i, n),
                row: GroupPos::Only,
            };
            let visual = ctx.host.store().button_visual(id);
            let glyph = IconGlyph::Builtin(icon);
            paint_icon_button_in_group(
                r,
                glyph,
                IconButtonStyle::Compact,
                visual,
                cell,
                ctx.scene,
                theme,
            );
            ctx.host.hit_index_mut().register(id, r);
        }
        x += w + gap;
    }

    let font = Button::label_font_px();
    let labels: Vec<&str> = ids::TOGGLES.iter().map(|(.., k)| tr(k)).collect();
    let w = largura(ctx, &labels, font);
    quebra(&mut x, &mut y, w);
    let fila = Rect::new(x, y, w, h);
    let ligados = [info.onion, info.onion_tint];
    let cells = segment_rects_for(fila, &labels, font, ctx.text_system);
    for (((_, id, _), (r, cell)), (label, on)) in ids::TOGGLES
        .iter()
        .zip(cells)
        .zip(labels.iter().zip(ligados))
    {
        let visual = ctx.host.store().button_visual(*id);
        paint_segmented_button_in_group(
            r,
            label,
            on,
            visual,
            ctx.scene,
            ctx.text_system,
            theme,
            cell,
        );
        ctx.host.hit_index_mut().register(*id, r);
    }
    y + h
}

/// A largura natural de uma fileira de rótulos (a soma das larguras que a porta lhes daria).
fn largura(ctx: &mut PaintCtx, labels: &[&str], font: f32) -> f32 {
    labels
        .iter()
        .map(|l| rect_for_label(ctx.text_system.prefix_width(l, font)))
        .sum()
}
