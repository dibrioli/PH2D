//! Os popovers do verso (mover, membros, etiquetas, checklist, datas, anexar, capa, campos, apagar,
//! partilhar, modelo): ancorados por baixo do botão que os abriu, presos à área da aba. O conteúdo
//! pinta-se numa cena à parte para o fundo saber a altura.

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, Pop, PopHit, ROW};
use crate::metrics as m;
use crate::paint::label;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::widget::{DatePickerLayout, LineField, paint_date_picker};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{CoverFill, Hue, LabelColor, Link, Shade};
use ph2d_tokens::{ColorToken as T, StrokeToken};
use ph2d_vector::VectorScene;

/// Os tipos de campo, pela ordem dos botões.
pub(crate) const FIELD_KINDS: [&str; 5] = [
    "kanban.back.kind.text",
    "kanban.back.kind.number",
    "kanban.back.kind.date",
    "kanban.back.kind.checkbox",
    "kanban.back.kind.dropdown",
];

/// O link interno do cartão (o «Share» do Trello).
pub(crate) fn share_link(cx: &Ctx<'_>) -> String {
    card_link(cx.k, cx.card)
}

/// O link interno de um cartão: o que o campo do «Share» mostra e o que o «Copy link» e o `Ctrl+C`
/// copiam.
pub(crate) fn card_link(k: &ph2d_kanban_model::Kanban, c: &ph2d_kanban_model::Card) -> String {
    format!("ph2d://kanban/{}/card/{}", k.id.0, c.id.0)
}

fn place(back: &Back, anchor: Option<Rect>, w: f32, h: f32) -> Rect {
    let area = back.area;
    let a = anchor.unwrap_or(Rect::new(
        area.x + (area.w - w) / 2.0,
        area.y + u::MODAL_TOP,
        w,
        0.0,
    ));
    let x = a.x.min(area.x + area.w - w - u::GAP).max(area.x + u::GAP);
    let below = a.y + a.h + u::SMALL_GAP;
    let y = if below + h <= area.y + area.h - u::GAP {
        below
    } else {
        (area.y + area.h - h - u::GAP).max(area.y + u::GAP)
    };
    Rect::new(x, y, w, h)
}

/// ⭐ Pinta o popover aberto (se há).
pub(crate) fn paint(ui: &mut Ui<'_>, back: &mut Back, cx: &Ctx<'_>) {
    let Some(pop) = back.pop.as_ref() else {
        return;
    };
    let anchor = ui
        .hits
        .iter()
        .rev()
        .find(|(_, h)| *h == pop.anchor())
        .map(|(r, _)| *r);
    if let Some(Pop::Dates(_, picker)) = back.pop.as_ref() {
        let r = place(back, anchor, u::POP_W, DatePickerLayout::preferred_height());
        paint_date_picker(&picker.widget, r, ui.scene, ui.ts, ui.theme);
        ui.hit(r, BackHit::Pop(PopHit::Picker));
        return;
    }
    // O conteúdo mede-se numa passagem à parte (sem alvos); a 2.ª pinta no sítio.
    let mut scratch = VectorScene::new();
    let mut none = Vec::new();
    let h = {
        let mut s = Ui {
            scene: &mut scratch,
            ts: ui.ts,
            theme: ui.theme,
            hits: &mut none,
            hover: None,
        };
        content(&mut s, back, cx, 0.0, 0.0)
    };
    let r = place(back, anchor, u::POP_W, h);
    crate::paint::shadow(
        ui.scene,
        r,
        m::MENU_RADIUS,
        ui.c(T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    let bg = ui.c(T::KanbanCard);
    fill_rounded_rect(ui.scene, r, m::MENU_RADIUS, bg);
    let ring = ui.c(T::KanbanSlot);
    stroke_rounded_rect(ui.scene, r, m::MENU_RADIUS, StrokeToken::Default.px(), ring);
    ui.hit(r, BackHit::Pop(PopHit::Inside));
    content(ui, back, cx, r.x, r.y);
}

fn title_key(pop: &Pop) -> &'static str {
    match pop {
        Pop::Move => "kanban.back.pop.move",
        Pop::Members(_) => "kanban.back.pop.members",
        Pop::Labels => "kanban.back.pop.labels",
        Pop::Label(None, ..) => "kanban.back.pop.label_new",
        Pop::Label(Some(_), ..) => "kanban.back.pop.label_edit",
        Pop::Checklist(_) => "kanban.back.pop.checklist",
        Pop::Dates(..) => "kanban.back.pop.dates",
        Pop::Attach(..) => "kanban.back.pop.attach",
        Pop::Cover => "kanban.back.pop.cover",
        Pop::Fields(..) => "kanban.back.pop.fields",
        Pop::Delete => "kanban.back.pop.delete",
        Pop::Share => "kanban.back.pop.share",
        Pop::Scene(_) => "kanban.links.add",
    }
}

/// O conteúdo do popover a partir de `(x, y)`; devolve a altura total.
fn content(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, x: f32, y: f32) -> f32 {
    let Some(pop) = back.pop.as_ref() else {
        return 0.0;
    };
    let p = u::POP_PAD;
    let iw = u::POP_W - 2.0 * p;
    let ix = x + p;
    // O título ao centro e o fechar.
    let t = tr(title_key(pop));
    let tw = ui.width(t, u::BODY_PX);
    let meta = ui.meta();
    ui.strong(
        t,
        x + (u::POP_W - tw) / 2.0,
        y + p / 2.0,
        ROW,
        u::BODY_PX,
        u::POP_W,
        meta,
    );
    let bg = ui.c(T::KanbanCard);
    ui.icon_button(
        Rect::new(x + u::POP_W - p / 2.0 - ROW, y + p / 2.0, ROW, ROW),
        IconId::Close,
        BackHit::Pop(PopHit::Close),
        false,
        bg,
    );
    let mut yy = y + p / 2.0 + ROW + u::GAP;
    let focus2 = back.pop_focus;
    match pop {
        Pop::Move => {
            for l in cx.k.lists_in_order() {
                let here = l.id == cx.card.list;
                ui.button(
                    Rect::new(ix, yy, iw, ROW),
                    &l.title,
                    here.then_some(IconId::Check),
                    BackHit::Pop(PopHit::List(l.id)),
                    here,
                );
                yy += ROW + u::SMALL_GAP;
            }
        }
        Pop::Members(item) => {
            let current: Vec<_> = match item {
                Some(i) => super::lists::item_of(cx, *i)
                    .and_then(|i| i.member.clone())
                    .into_iter()
                    .collect(),
                None => cx.card.members.clone(),
            };
            ui.caption(tr("kanban.back.card_members"), ix, yy, iw);
            yy += u::META_LINE + u::SMALL_GAP;
            for (i, who) in cx.people.iter().enumerate() {
                let row = Rect::new(ix, yy, iw, ROW + u::SMALL_GAP);
                #[allow(clippy::cast_possible_truncation)]
                let h = BackHit::Pop(PopHit::Member(i as u16));
                ui.button(row, "", None, h, false);
                ui.avatar(ix + u::SMALL_GAP, yy + u::SMALL_GAP / 2.0, ROW, who);
                let fg = ui.fg();
                ui.line(
                    &who.0,
                    ix + ROW + u::GAP + u::SMALL_GAP,
                    yy,
                    row.h,
                    u::BODY_PX,
                    iw - 2.0 * ROW,
                    fg,
                );
                if current.contains(who) {
                    crate::paint::icon(
                        ui.scene,
                        IconId::Check,
                        Rect::new(ix + iw - ROW, yy, ROW, row.h),
                        fg,
                    );
                }
                yy += row.h + u::SMALL_GAP;
            }
        }
        Pop::Labels => {
            let mut labels: Vec<_> = cx.k.labels.values().filter(|l| !l.meta.deleted).collect();
            labels.sort_by_key(|l| l.id);
            for l in labels {
                let on = cx.card.labels.contains(&l.id);
                let b = Rect::new(ix, yy + (ROW - u::CHECK) / 2.0, u::CHECK, u::CHECK);
                ui.checkbox(b, on);
                let cx0 = ix + u::CHECK + u::GAP;
                let cw = iw - u::CHECK - u::GAP - ROW - u::SMALL_GAP;
                let (fill, ink) = l
                    .color
                    .map_or((ui.c(T::KanbanSlot), ui.fg()), |c| label(ui.theme, c));
                let chip = Rect::new(cx0, yy, cw, ROW);
                fill_rounded_rect(ui.scene, chip, m::LABEL_RADIUS, fill);
                ui.strong(
                    &l.name,
                    cx0 + m::CARD_PAD_X,
                    yy,
                    ROW,
                    u::BODY_PX,
                    cw - m::CARD_PAD_X,
                    ink,
                );
                ui.hit(
                    Rect::new(ix, yy, u::CHECK + u::GAP + cw, ROW),
                    BackHit::Pop(PopHit::Label(l.id)),
                );
                let bg = ui.c(T::KanbanCard);
                ui.icon_button(
                    Rect::new(ix + iw - ROW, yy, ROW, ROW),
                    IconId::Modify,
                    BackHit::Pop(PopHit::LabelEdit(l.id)),
                    false,
                    bg,
                );
                yy += ROW + u::SMALL_GAP;
            }
            yy += u::GAP;
            ui.button(
                Rect::new(ix, yy, iw, ROW),
                tr("kanban.back.create_label"),
                None,
                BackHit::Pop(PopHit::LabelCreate),
                false,
            );
            yy += ROW;
        }
        Pop::Label(id, name, color) => {
            let (fill, ink) = color.map_or((ui.c(T::KanbanSlot), ui.fg()), |c| label(ui.theme, c));
            let preview = Rect::new(ix, yy, iw, ROW + u::GAP);
            fill_rounded_rect(ui.scene, preview, m::LABEL_RADIUS, fill);
            ui.strong(
                &name.text,
                ix + m::CARD_PAD_X,
                yy,
                preview.h,
                u::BODY_PX,
                iw,
                ink,
            );
            yy += preview.h + u::GAP;
            yy = input(ui, ix, yy, iw, "kanban.back.title", name, PopHit::Input);
            ui.caption(tr("kanban.back.select_color"), ix, yy, iw);
            yy += u::META_LINE + u::SMALL_GAP;
            let cols = 5usize;
            #[allow(clippy::cast_precision_loss)]
            let sw = (iw - u::SMALL_GAP * (cols as f32 - 1.0)) / cols as f32;
            let all = LabelColor::all();
            for (i, col) in all.iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let r = Rect::new(
                    ix + (i % cols) as f32 * (sw + u::SMALL_GAP),
                    yy + (i / cols) as f32 * (ROW + u::SMALL_GAP),
                    sw,
                    ROW,
                );
                fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, label(ui.theme, *col).0);
                if *color == Some(*col) {
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
                ui.hit(r, BackHit::Pop(PopHit::Color(i as u8)));
            }
            #[allow(clippy::cast_precision_loss)]
            let rows = all.len().div_ceil(cols) as f32;
            yy += rows * (ROW + u::SMALL_GAP) + u::SMALL_GAP;
            ui.button(
                Rect::new(ix, yy, iw, ROW),
                tr("kanban.back.remove_color"),
                Some(IconId::Close),
                BackHit::Pop(PopHit::NoColor),
                false,
            );
            yy += ROW + u::GAP;
            let s = tr(if id.is_some() {
                "kanban.back.save"
            } else {
                "kanban.back.create"
            });
            let sw = ui.button_w(s, false);
            ui.primary(Rect::new(ix, yy, sw, ROW), s, BackHit::Pop(PopHit::Save));
            if id.is_some() {
                let d = tr("kanban.back.delete");
                let dw = ui.button_w(d, false);
                danger(ui, Rect::new(ix + iw - dw, yy, dw, ROW), d, PopHit::Remove);
            }
            yy += ROW;
        }
        Pop::Checklist(name) => {
            yy = input(ui, ix, yy, iw, "kanban.back.title", name, PopHit::Input);
            let s = tr("kanban.back.add");
            let sw = ui.button_w(s, false);
            ui.primary(Rect::new(ix, yy, sw, ROW), s, BackHit::Pop(PopHit::Save));
            yy += ROW;
        }
        Pop::Attach(url, name) => {
            ui.caption(tr("kanban.back.attach_computer"), ix, yy, iw);
            yy += u::META_LINE + u::SMALL_GAP;
            let meta = ui.meta();
            yy += ui.text(tr("kanban.back.drop_hint"), ix, yy, u::SMALL_PX, iw, meta);
            yy += u::SMALL_GAP;
            let r = Rect::new(ix, yy, iw, ROW);
            ui.button(
                r,
                tr("kanban.back.choose_file"),
                None,
                BackHit::Pop(PopHit::PickFile),
                false,
            );
            yy += ROW + u::GAP;
            yy = input_focus(
                ui,
                ix,
                yy,
                iw,
                "kanban.back.attach_url",
                url,
                PopHit::Input,
                !focus2,
            );
            yy = input_focus(
                ui,
                ix,
                yy,
                iw,
                "kanban.back.attach_name",
                name,
                PopHit::Input2,
                focus2,
            );
            let s = tr("kanban.back.insert");
            let sw = ui.button_w(s, false);
            ui.primary(Rect::new(ix, yy, sw, ROW), s, BackHit::Pop(PopHit::Save));
            yy += ROW;
        }
        Pop::Cover => yy = cover(ui, cx, ix, yy, iw),
        Pop::Fields(name, kind, opts) => {
            let mut defs: Vec<_> = cx.k.fields.values().filter(|f| !f.meta.deleted).collect();
            defs.sort_by(|a, b| a.pos.cmp(&b.pos));
            for f in defs {
                let fg = ui.fg();
                ui.strong(&f.name, ix, yy, ROW, u::BODY_PX, iw - 2.0 * ROW, fg);
                let b = Rect::new(
                    ix + iw - 2.0 * ROW,
                    yy + (ROW - u::CHECK) / 2.0,
                    u::CHECK,
                    u::CHECK,
                );
                ui.checkbox(b, f.on_front);
                ui.hit(
                    Rect::new(b.x - u::SMALL_GAP, yy, ROW, ROW),
                    BackHit::Pop(PopHit::FieldFront(f.id)),
                );
                let bg = ui.c(T::KanbanCard);
                ui.icon_button(
                    Rect::new(ix + iw - ROW, yy, ROW, ROW),
                    IconId::Trash,
                    BackHit::Pop(PopHit::FieldDelete(f.id)),
                    false,
                    bg,
                );
                yy += ROW + u::SMALL_GAP;
            }
            let meta = ui.meta();
            // Explicação: quebra em linhas, nunca se elide.
            yy += ui.text(tr("kanban.back.on_front_hint"), ix, yy, u::SMALL_PX, iw, meta).max(u::META_LINE) + u::GAP;
            yy = input_focus(
                ui,
                ix,
                yy,
                iw,
                "kanban.back.field_name",
                name,
                PopHit::Input,
                !focus2,
            );
            #[allow(clippy::cast_precision_loss)]
            let kw = (iw - u::SMALL_GAP * 2.0) / 3.0;
            for (i, key) in FIELD_KINDS.iter().enumerate() {
                #[allow(clippy::cast_precision_loss)]
                let r = Rect::new(
                    ix + (i % 3) as f32 * (kw + u::SMALL_GAP),
                    yy + (i / 3) as f32 * (ROW + u::SMALL_GAP),
                    kw,
                    ROW,
                );
                #[allow(clippy::cast_possible_truncation)]
                ui.button(
                    r,
                    tr(key),
                    None,
                    BackHit::Pop(PopHit::FieldKind(i as u8)),
                    usize::from(*kind) == i,
                );
            }
            yy += 2.0 * (ROW + u::SMALL_GAP);
            if usize::from(*kind) == FIELD_KINDS.len() - 1 {
                yy = input_focus(
                    ui,
                    ix,
                    yy,
                    iw,
                    "kanban.back.field_options",
                    opts,
                    PopHit::Input2,
                    focus2,
                );
            }
            let s = tr("kanban.back.create");
            let sw = ui.button_w(s, false);
            ui.primary(Rect::new(ix, yy, sw, ROW), s, BackHit::Pop(PopHit::Save));
            yy += ROW;
        }
        Pop::Delete => {
            let fg = ui.fg();
            yy += ui.text(tr("kanban.back.delete_warning"), ix, yy, u::BODY_PX, iw, fg) + u::GAP;
            danger(
                ui,
                Rect::new(ix, yy, iw, ROW),
                tr("kanban.back.delete"),
                PopHit::Save,
            );
            yy += ROW;
        }
        Pop::Share => {
            ui.caption(tr("kanban.back.share_link"), ix, yy, iw);
            yy += u::META_LINE + u::SMALL_GAP;
            let r = Rect::new(ix, yy, iw, ROW);
            let mut f = LineField::new(share_link(cx));
            f.caret = f.text.len();
            ui.field(r, &f, "");
            ui.hit(r, BackHit::Pop(PopHit::Input));
            yy += ROW + u::SMALL_GAP;
            let s = tr("kanban.back.copy_link");
            let sw = ui.button_w(s, false);
            ui.primary(
                Rect::new(ix, yy, sw, ROW),
                s,
                BackHit::Pop(PopHit::CopyLink),
            );
            yy += ROW + u::SMALL_GAP;
            let meta = ui.meta();
            // Explicação: quebra em linhas, nunca se elide.
            yy += ui.text(tr("kanban.back.share_hint"), ix, yy, u::SMALL_PX, iw, meta).max(u::META_LINE);
        }
        Pop::Dates(..) => {}
        Pop::Scene(f) => yy = super::scene::pop(ui, back, cx, f, ix, yy, iw),
    }
    yy + p - y
}

fn danger(ui: &mut Ui<'_>, r: Rect, s: &str, h: PopHit) {
    let (bg, ink) = (ui.c(T::KanbanDueOver), ui.c(T::KanbanDueOverFg));
    fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, bg);
    let w = ui.width(s, u::BODY_PX);
    ui.strong(s, r.x + (r.w - w) / 2.0, r.y, r.h, u::BODY_PX, r.w, ink);
    ui.hit(r, BackHit::Pop(h));
}

#[allow(clippy::too_many_arguments)]
fn input(ui: &mut Ui<'_>, x: f32, y: f32, w: f32, caption: &str, f: &LineField, h: PopHit) -> f32 {
    input_focus(ui, x, y, w, caption, f, h, true)
}

#[allow(clippy::too_many_arguments)]
fn input_focus(
    ui: &mut Ui<'_>,
    x: f32,
    y: f32,
    w: f32,
    caption: &str,
    f: &LineField,
    h: PopHit,
    focused: bool,
) -> f32 {
    ui.caption(tr(caption), x, y, w);
    let r = Rect::new(x, y + u::META_LINE + u::SMALL_GAP, w, ROW);
    if focused {
        ui.field(r, f, "");
    } else {
        let bg = ui.c(T::KanbanCardRaised);
        fill_rounded_rect(ui.scene, r, m::FIELD_RADIUS, bg);
        let ring = ui.c(T::KanbanSlot);
        stroke_rounded_rect(
            ui.scene,
            r,
            m::FIELD_RADIUS,
            StrokeToken::Default.px(),
            ring,
        );
        let fg = ui.fg();
        ui.line(
            &f.text,
            r.x + m::FIELD_PAD_X,
            r.y,
            r.h,
            u::BODY_PX,
            r.w - 2.0 * m::FIELD_PAD_X,
            fg,
        );
    }
    ui.hit(r, BackHit::Pop(h));
    r.y + r.h + u::GAP
}

/// A capa: o tamanho (metade/inteira), as dez cores e as imagens dos anexos.
fn cover(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let mut y = y;
    let cur = cx.card.cover;
    ui.caption(tr("kanban.back.cover_size"), x, y, w);
    y += u::META_LINE + u::SMALL_GAP;
    let half_w = (w - u::GAP) / 2.0;
    for (i, full) in [false, true].into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(x + i as f32 * (half_w + u::GAP), y, half_w, 2.0 * ROW);
        let on = cur.is_some_and(|c| c.full == full);
        ui.button(r, "", None, BackHit::Pop(PopHit::CoverSize(full)), on);
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
    let images: Vec<_> = super::lists::attachments_of(cx)
        .into_iter()
        .filter(|a| matches!(a.link, Link::File(_)))
        .collect();
    ui.caption(tr("kanban.back.attachments"), x, y, w);
    y += u::META_LINE + u::SMALL_GAP;
    if images.is_empty() {
        let meta = ui.meta();
        ui.line(
            tr("kanban.back.no_images"),
            x,
            y,
            u::META_LINE,
            u::SMALL_PX,
            w,
            meta,
        );
        y += u::META_LINE;
    }
    for a in images {
        ui.button(
            Rect::new(x, y, w, ROW),
            &a.name,
            Some(IconId::Image),
            BackHit::Pop(PopHit::CoverImage(a.id)),
            false,
        );
        y += ROW + u::SMALL_GAP;
    }
    y
}
