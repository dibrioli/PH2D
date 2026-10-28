//! As checklists (progresso, itens com responsável e data, esconder marcados, item → cartão) e os
//! anexos (miniatura/ícone, abrir, capa, apagar) do verso.

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, EditAt, ROW};
use crate::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::{Attachment, CheckItem, CheckItemId, Checklist, CoverFill, Link};
use ph2d_tokens::ColorToken as T;

/// As checklists vivas do cartão, pela ordem.
pub(crate) fn of_card<'a>(cx: &Ctx<'a>) -> Vec<&'a Checklist> {
    let mut v: Vec<&Checklist> =
        cx.k.checklists
            .values()
            .filter(|c| c.card == cx.card.id && !c.meta.deleted)
            .collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v
}

/// Os itens vivos de uma checklist, pela ordem.
pub(crate) fn items_of<'a>(cx: &Ctx<'a>, cl: &Checklist) -> Vec<&'a CheckItem> {
    let mut v: Vec<&CheckItem> =
        cx.k.items
            .values()
            .filter(|i| i.checklist == cl.id && !i.meta.deleted)
            .collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v
}

/// Os anexos vivos do cartão, pela ordem.
pub(crate) fn attachments_of<'a>(cx: &Ctx<'a>) -> Vec<&'a Attachment> {
    let mut v: Vec<&Attachment> =
        cx.k.attachments
            .values()
            .filter(|a| a.card == cx.card.id && !a.meta.deleted)
            .collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v
}

fn editing<'b>(back: &'b Back, at: &EditAt) -> Option<&'b super::Edit> {
    back.edit.as_ref().filter(|e| e.at == *at)
}

/// Um campo aberto no lugar com «Save»/«Cancel» por baixo; devolve o fundo.
fn open_field(ui: &mut Ui<'_>, e: &super::Edit, x: f32, y: f32, w: f32, save: &str) -> f32 {
    let r = Rect::new(x, y, w, ROW);
    ui.field(r, &e.field, "");
    ui.hit(r, BackHit::EditField);
    let y = y + ROW + u::GAP;
    let sw = ui.button_w(save, false);
    ui.primary(Rect::new(x, y, sw, ROW), save, BackHit::EditSave);
    let cancel = tr("kanban.back.cancel");
    let cw = ui.button_w(cancel, false);
    ui.button(
        Rect::new(x + sw + u::GAP, y, cw, ROW),
        cancel,
        None,
        BackHit::EditCancel,
        false,
    );
    y + ROW
}

pub(crate) fn checklists(
    ui: &mut Ui<'_>,
    back: &Back,
    cx: &Ctx<'_>,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    let mut y = y;
    for cl in of_card(cx) {
        y = checklist(ui, back, cx, cl, x, y, w) + u::SECTION_GAP;
    }
    y
}

#[allow(clippy::too_many_lines)]
fn checklist(
    ui: &mut Ui<'_>,
    back: &Back,
    cx: &Ctx<'_>,
    cl: &Checklist,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    let items = items_of(cx, cl);
    let done = items.iter().filter(|i| i.done).count();
    let meta = ui.meta();
    crate::paint::icon(
        ui.scene,
        IconId::Check,
        Rect::new(x - u::MAIN_INDENT, y, ROW, ROW),
        meta,
    );
    let del = tr("kanban.back.delete");
    let dw = ui.button_w(del, false);
    let hide = if cl.hide_checked {
        tr_with("kanban.back.show_checked", &[("n", &done)])
    } else {
        tr("kanban.back.hide_checked").to_owned()
    };
    let hw = if done > 0 {
        ui.button_w(&hide, false)
    } else {
        0.0
    };
    let title_w = (w - dw - hw - 2.0 * u::GAP).max(0.0);
    if let Some(e) = editing(back, &EditAt::ClTitle(cl.id)) {
        let bottom = open_field(ui, e, x, y, title_w, tr("kanban.back.save"));
        return checklist_body(ui, back, cx, cl, &items, done, x, bottom + u::GAP, w);
    }
    ui.heading(&cl.title, x, y, title_w);
    ui.hit(Rect::new(x, y, title_w, ROW), BackHit::ClTitle(cl.id));
    let mut bx = x + w - dw;
    ui.button(
        Rect::new(bx, y, dw, ROW),
        del,
        None,
        BackHit::ClDelete(cl.id),
        false,
    );
    if done > 0 {
        bx -= hw + u::GAP;
        ui.button(
            Rect::new(bx, y, hw, ROW),
            &hide,
            None,
            BackHit::ClHide(cl.id),
            false,
        );
    }
    checklist_body(ui, back, cx, cl, &items, done, x, y + ROW + u::GAP, w)
}

#[allow(clippy::too_many_arguments)]
fn checklist_body(
    ui: &mut Ui<'_>,
    back: &Back,
    cx: &Ctx<'_>,
    cl: &Checklist,
    items: &[&CheckItem],
    done: usize,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let frac = if items.is_empty() {
        0.0
    } else {
        done as f32 / items.len() as f32
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pct = (frac * 100.0).round() as u32;
    let meta = ui.meta();
    ui.line(
        &tr_with("kanban.back.percent", &[("n", &pct)]),
        x - u::MAIN_INDENT,
        y,
        u::META_LINE,
        u::SMALL_PX,
        u::PERCENT_W,
        meta,
    );
    let bar = Rect::new(x, y + (u::META_LINE - u::BAR_H) / 2.0, w, u::BAR_H);
    let slot = ui.c(T::KanbanSlot);
    fill_rounded_rect(ui.scene, bar, u::BAR_H / 2.0, slot);
    if frac > 0.0 {
        let fill = if done == items.len() {
            ui.c(T::KanbanDueDone)
        } else {
            ui.accent()
        };
        fill_rounded_rect(
            ui.scene,
            Rect::new(bar.x, bar.y, bar.w * frac, bar.h),
            u::BAR_H / 2.0,
            fill,
        );
    }
    let mut y = y + u::META_LINE + u::SMALL_GAP;
    for it in items.iter().filter(|i| !(cl.hide_checked && i.done)) {
        y = item(ui, back, cx, it, x, y, w);
    }
    y += u::SMALL_GAP;
    if let Some(e) = editing(back, &EditAt::NewItem(cl.id)) {
        return open_field(ui, e, x, y, w, tr("kanban.back.add"));
    }
    let add = tr("kanban.back.add_item");
    let aw = ui.button_w(add, false);
    ui.button(
        Rect::new(x, y, aw, ROW),
        add,
        None,
        BackHit::AddItem(cl.id),
        false,
    );
    y + ROW
}

fn item(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, it: &CheckItem, x: f32, y: f32, w: f32) -> f32 {
    let id = it.id;
    let row = Rect::new(x - u::MAIN_INDENT, y, w + u::MAIN_INDENT, u::ITEM_H);
    let hovered = matches!(
        back.hover,
        Some(
            BackHit::ItemBox(i)
            | BackHit::ItemText(i)
            | BackHit::ItemMember(i)
            | BackHit::ItemDue(i)
            | BackHit::ItemConvert(i)
            | BackHit::ItemDelete(i)
        ) if i == id
    );
    if hovered {
        let slot = ui.c(T::KanbanSlot);
        fill_rounded_rect(ui.scene, row, m::BUTTON_RADIUS, slot);
    }
    let bx = x - u::MAIN_INDENT + (u::MAIN_INDENT - u::CHECK) / 2.0;
    let b = Rect::new(bx, y + (u::ITEM_H - u::CHECK) / 2.0, u::CHECK, u::CHECK);
    ui.checkbox(b, it.done);
    ui.hit(
        Rect::new(row.x, y, u::MAIN_INDENT, u::ITEM_H),
        BackHit::ItemBox(id),
    );
    if let Some(e) = editing(back, &EditAt::Item(id)) {
        return open_field(
            ui,
            e,
            x,
            y + (u::ITEM_H - ROW) / 2.0,
            w,
            tr("kanban.back.save"),
        ) + u::GAP;
    }
    // A direita: a data, o responsável e, a pairar, as ações.
    let mut rx = x + w;
    let mut slot = |ui: &mut Ui<'_>, wd: f32| {
        rx -= wd;
        let r = Rect::new(rx, y + (u::ITEM_H - ROW) / 2.0, wd, ROW);
        rx -= u::SMALL_GAP;
        let _ = ui;
        r
    };
    let mut icons: Vec<(IconId, BackHit)> = Vec::new();
    if hovered {
        icons.push((IconId::Trash, BackHit::ItemDelete(id)));
        icons.push((IconId::ConvertCurves, BackHit::ItemConvert(id)));
        if it.due.is_none() {
            icons.push((IconId::Timer, BackHit::ItemDue(id)));
        }
        if it.member.is_none() {
            icons.push((IconId::Add, BackHit::ItemMember(id)));
        }
    }
    for (icon, h) in icons {
        let r = slot(ui, ROW);
        let fill = ui.c(T::KanbanCard);
        ui.icon_button(r, icon, h, false, fill);
    }
    if let Some(who) = &it.member {
        let r = slot(ui, u::AVATAR_SMALL);
        ui.avatar(
            r.x,
            y + (u::ITEM_H - u::AVATAR_SMALL) / 2.0,
            u::AVATAR_SMALL,
            who,
        );
        ui.hit(r, BackHit::ItemMember(id));
    }
    if let Some(d) = it.due {
        let t = super::dates::short(d, cx.now);
        let tw = ui.width(&t, u::SMALL_PX) + 2.0 * u::GAP;
        let r = slot(ui, tw);
        let chip = Rect::new(r.x, y + (u::ITEM_H - u::META_LINE) / 2.0, r.w, u::META_LINE);
        let slot_c = ui.c(T::KanbanSlot);
        fill_rounded_rect(ui.scene, chip, m::LABEL_RADIUS - u::HAIR / 2.0, slot_c);
        let fg = ui.fg();
        ui.line(&t, chip.x + u::GAP, chip.y, chip.h, u::SMALL_PX, tw, fg);
        ui.hit(chip, BackHit::ItemDue(id));
    }
    let tw = (rx - x).max(0.0);
    let ink = if it.done { ui.meta() } else { ui.fg() };
    let h = ui.measure(&it.text, u::BODY_PX, tw).max(u::META_LINE);
    let ty = y + ((u::ITEM_H - h) / 2.0).max(u::GAP);
    ui.text(&it.text, x, ty, u::BODY_PX, tw, ink);
    if it.done {
        let lw = ui.width(&it.text, u::BODY_PX).min(tw);
        let line = Rect::new(
            x,
            ty + u::META_LINE / 2.0,
            lw,
            ph2d_tokens::StrokeToken::Default.px(),
        );
        ui.scene.fill_rect(rect_to_vello(line), ink);
    }
    ui.hit(Rect::new(x, y, tw, u::ITEM_H.max(h)), BackHit::ItemText(id));
    y + u::ITEM_H.max(h + 2.0 * u::GAP)
}

/// O ícone de um anexo pelo tipo.
fn kind_icon(a: &Attachment) -> IconId {
    match a.link {
        Link::File(_) => IconId::File,
        Link::Url(_) => IconId::Link,
        Link::Card(..) => IconId::Clone,
        Link::Kanban(_) => IconId::LayoutGrid,
    }
}

pub(crate) fn attachments(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let list = attachments_of(cx);
    if list.is_empty() {
        return y;
    }
    let meta = ui.meta();
    crate::paint::icon(
        ui.scene,
        IconId::Link,
        Rect::new(x - u::MAIN_INDENT, y, ROW, ROW),
        meta,
    );
    ui.heading(tr("kanban.back.attachments"), x, y, w);
    let add = tr("kanban.back.add");
    let aw = ui.button_w(add, false);
    ui.button(
        Rect::new(x + w - aw, y, aw, ROW),
        add,
        None,
        BackHit::AttAdd,
        false,
    );
    let mut y = y + ROW + u::GAP;
    for a in list {
        let thumb = Rect::new(x, y, u::THUMB_W, u::THUMB_H);
        let slot = ui.c(T::KanbanSlot);
        fill_rounded_rect(ui.scene, thumb, m::LABEL_RADIUS, slot);
        let r = m::LABEL_RADIUS;
        let pictured = match a.link {
            Link::File(f) => super::thumb::draw(ui.scene, cx.set, f, thumb, (r, r, r, r), true),
            _ => false,
        };
        if !pictured {
            let kind = match a.link {
                Link::File(f) => cx.set.file(f).and_then(|_| super::attach::kind_of(&a.name)),
                _ => None,
            };
            match kind {
                Some(k) => {
                    let kw = ui.width(&k, u::SMALL_PX);
                    let ky = thumb.y + (thumb.h - u::META_LINE) / 2.0;
                    let kx = thumb.x + ((thumb.w - kw) / 2.0).max(0.0);
                    ui.strong(&k, kx, ky, u::META_LINE, u::SMALL_PX, thumb.w, meta);
                }
                None => crate::paint::icon(ui.scene, kind_icon(a), thumb, meta),
            }
        }
        ui.hit(thumb, BackHit::AttOpen(a.id));
        let tx = x + u::THUMB_W + u::GAP;
        let tw = (w - u::THUMB_W - u::GAP).max(0.0);
        let fg = ui.fg();
        ui.strong(&a.name, tx, y, u::META_LINE, u::BODY_PX, tw, fg);
        let nw = ui.width(&a.name, u::BODY_PX).min(tw);
        ui.hit(Rect::new(tx, y, nw, u::META_LINE), BackHit::AttOpen(a.id));
        let mut added = tr_with(
            "kanban.back.added",
            &[("when", &super::dates::ago(a.at, cx.now))],
        );
        if let Link::File(f) = a.link
            && let Some(file) = cx.set.file(f)
        {
            let size = super::attach::human_size(file.bytes.0.len());
            added = tr_with("kanban.back.file_meta", &[("added", &added), ("size", &size)]);
        }
        ui.line(
            &added,
            tx,
            y + u::META_LINE,
            u::META_LINE,
            u::SMALL_PX,
            tw,
            meta,
        );
        let mut bx = tx;
        let by = y + 2.0 * u::META_LINE;
        if let Link::File(f) = a.link {
            let is_cover = matches!(cx.card.cover, Some(c) if c.fill == CoverFill::Image(f));
            let s = tr(if is_cover {
                "kanban.back.remove_cover"
            } else {
                "kanban.back.make_cover"
            });
            let sw = ui.width(s, u::SMALL_PX);
            ui.line(s, bx, by, u::META_LINE, u::SMALL_PX, sw, meta);
            ui.hit(Rect::new(bx, by, sw, u::META_LINE), BackHit::AttCover(a.id));
            bx += sw + u::GAP;
        }
        let s = tr("kanban.back.delete");
        let sw = ui.width(s, u::SMALL_PX);
        ui.line(s, bx, by, u::META_LINE, u::SMALL_PX, sw, meta);
        ui.hit(
            Rect::new(bx, by, sw, u::META_LINE),
            BackHit::AttDelete(a.id),
        );
        y += u::THUMB_H.max(3.0 * u::META_LINE) + u::GAP;
    }
    y + u::SECTION_GAP - u::GAP
}

/// O item `id` (para quem só tem o id).
pub(crate) fn item_of<'a>(cx: &Ctx<'a>, id: CheckItemId) -> Option<&'a CheckItem> {
    cx.k.items.get(&id).filter(|i| !i.meta.deleted)
}
