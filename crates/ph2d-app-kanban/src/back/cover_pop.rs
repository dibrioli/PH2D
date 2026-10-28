//! O popover da CAPA (o «Cover» do Trello): o tamanho, as dez cores, as imagens anexadas em
//! miniatura, o «Upload a cover image» e a dica de largar uma imagem ali.

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, Pop, PopHit, ROW};
use crate::metrics as m;
use crate::paint::label;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{CoverFill, Hue, LabelColor, Link, Shade};
use ph2d_tokens::{ColorToken as T, StrokeToken};

/// A capa: o tamanho (metade/inteira), as dez cores e as imagens dos anexos.
pub(super) fn cover(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let mut y = y;
    let cur = cx.card.cover;
    ui.caption(tr("kanban.back.cover_size"), x, y, w);
    y += u::META_LINE + u::SMALL_GAP;
    let half_w = (w - u::GAP) / 2.0;
    for (i, full) in [false, true].into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(x + i as f32 * (half_w + u::GAP), y, half_w, 2.0 * ROW);
        let on = cur.is_some_and(|c| c.full == full);
        // O tamanho escolhido leva o anel azul (o Trello), nunca o botão «ligado»: claro no tema
        // escuro, apagava a amostra.
        ui.button(r, "", None, BackHit::Pop(PopHit::CoverSize(full)), false);
        if on {
            let ring = ui.accent();
            stroke_rounded_rect(ui.scene, r, m::LABEL_RADIUS, StrokeToken::Default.px() * 2.0, ring);
        }
        let tint = cur.map_or(ui.c(T::KanbanCardMeta), |c| match c.fill {
            CoverFill::Color(hue) => {
                label(
                    ui.theme,
                    LabelColor {
                        hue,
                        shade: Shade::Normal,
                    },
                )
                .0
            }
            CoverFill::Image(_) => ui.c(T::KanbanCardMeta),
        });
        let inner = Rect::new(
            r.x + u::SMALL_GAP,
            r.y + u::SMALL_GAP,
            r.w - 2.0 * u::SMALL_GAP,
            r.h - 2.0 * u::SMALL_GAP,
        );
        let band = if full {
            inner
        } else {
            Rect::new(inner.x, inner.y, inner.w, inner.h / 2.0)
        };
        fill_rounded_rect(ui.scene, band, m::LABEL_RADIUS, tint);
    }
    y += 2.0 * ROW + u::GAP;
    if cur.is_some() {
        ui.button(
            Rect::new(x, y, w, ROW),
            tr("kanban.back.remove_cover"),
            None,
            BackHit::Pop(PopHit::Remove),
            false,
        );
        y += ROW + u::GAP;
    }
    ui.caption(tr("kanban.back.colors"), x, y, w);
    y += u::META_LINE + u::SMALL_GAP;
    let cols = 5usize;
    #[allow(clippy::cast_precision_loss)]
    let sw = (w - u::SMALL_GAP * (cols as f32 - 1.0)) / cols as f32;
    for (i, hue) in Hue::ALL.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x + (i % cols) as f32 * (sw + u::SMALL_GAP),
            y + (i / cols) as f32 * (ROW + u::SMALL_GAP),
            sw,
            ROW,
        );
        fill_rounded_rect(
            ui.scene,
            r,
            m::LABEL_RADIUS,
            label(
                ui.theme,
                LabelColor {
                    hue: *hue,
                    shade: Shade::Normal,
                },
            )
            .0,
        );
        if cur.is_some_and(|c| c.fill == CoverFill::Color(*hue)) {
            let fg = ui.fg();
            stroke_rounded_rect(
                ui.scene,
                r,
                m::LABEL_RADIUS,
                StrokeToken::Default.px() * 2.0,
                fg,
            );
        }
        #[allow(clippy::cast_possible_truncation)]
        ui.hit(r, BackHit::Pop(PopHit::CoverHue(i as u8)));
    }
    y += 2.0 * (ROW + u::SMALL_GAP);
    y = cover_images(ui, cx, x, y, w);
    ui.button(
        Rect::new(x, y, w, ROW),
        tr("kanban.back.upload_cover"),
        Some(IconId::Image),
        BackHit::Pop(PopHit::CoverUpload),
        false,
    );
    y += ROW + u::SMALL_GAP;
    let meta = ui.meta();
    // Explicação: quebra em linhas, nunca se elide.
    y += ui
        .text(tr("kanban.back.cover_tip"), x, y, u::SMALL_PX, w, meta)
        .max(u::META_LINE);
    y
}

/// «Attachments» da capa: as IMAGENS anexadas em miniatura (clicar = capa; a capa actual com
/// anel), ou o aviso de que ainda não há nenhuma. Devolve o `y` seguinte.
fn cover_images(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let mut y = y;
    let images: Vec<_> = super::lists::attachments_of(cx)
        .into_iter()
        .filter_map(|a| match a.link {
            Link::File(f) if cx.set.file(f).is_some_and(|k| k.size.is_some()) => Some((a.id, f)),
            _ => None,
        })
        .collect();
    ui.caption(tr("kanban.back.attachments"), x, y, w);
    y += u::META_LINE + u::SMALL_GAP;
    if images.is_empty() {
        let meta = ui.meta();
        y += ui
            .text(tr("kanban.back.no_images"), x, y, u::SMALL_PX, w, meta)
            .max(u::META_LINE)
            + u::GAP;
        return y;
    }
    let cols = 3usize;
    #[allow(clippy::cast_precision_loss)]
    let tw = (w - u::SMALL_GAP * (cols as f32 - 1.0)) / cols as f32;
    let th = 2.0 * ROW;
    let cur = cx.card.cover.map(|c| c.fill);
    for (i, (a, f)) in images.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            x + (i % cols) as f32 * (tw + u::SMALL_GAP),
            y + (i / cols) as f32 * (th + u::SMALL_GAP),
            tw,
            th,
        );
        let rad = m::LABEL_RADIUS;
        if !super::thumb::draw(ui.scene, cx.set, *f, r, (rad, rad, rad, rad), true) {
            let bg = ui.c(T::KanbanCardMeta);
            fill_rounded_rect(ui.scene, r, rad, bg);
        }
        if cur == Some(CoverFill::Image(*f)) {
            let fg = ui.fg();
            stroke_rounded_rect(ui.scene, r, rad, StrokeToken::Default.px() * 2.0, fg);
        }
        ui.hit(r, BackHit::Pop(PopHit::CoverImage(*a)));
    }
    #[allow(clippy::cast_precision_loss)]
    let rows = images.len().div_ceil(cols) as f32;
    y + rows * (th + u::SMALL_GAP) + u::GAP
}

/// O ponto `(x, y)` está sobre o popover da capa aberto (o último quadro pintado): largar uma
/// imagem ali é a capa.
pub(crate) fn over_cover(back: &Back, x: f32, y: f32) -> bool {
    matches!(back.pop, Some(Pop::Cover))
        && back
            .hits
            .iter()
            .any(|(r, h)| *h == BackHit::Pop(PopHit::Inside) && r.contains(x, y))
}
