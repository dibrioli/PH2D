//! A atividade do verso: a caixa de comentário (editor rico compacto, `@` completa membros) e a
//! lista intercalada pela hora de comentários (editar, apagar, reacções) e do histórico (o que as
//! `KanbanOp` registaram; «Show details» mostra tudo).

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, REACTIONS, ROW};
use crate::metrics as m;
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::widget::{RichEditor, paint_rich_editor};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::{Activity, Comment, Happened, Kanban, Member, Moment, Subject};
use ph2d_richdoc::{Caret, Selection};
use ph2d_tokens::{ColorToken as T, StrokeToken};

/// O `@nome` a meio de escrever antes do cursor: `(byte do @, o que já se escreveu)`.
pub(crate) fn mention_query(ed: &RichEditor) -> Option<(usize, String)> {
    if !ed.editing {
        return None;
    }
    let sel = ed.selection();
    if !sel.is_collapsed() {
        return None;
    }
    let text = ed.doc().block(sel.head.block)?.inline.text();
    let before = text.get(..sel.head.byte)?;
    let at = before.rfind('@')?;
    let q = &before[at + 1..];
    let opens = before[..at]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace);
    (opens && !q.contains(char::is_whitespace)).then(|| (at, q.to_owned()))
}

/// Quem casa com o que se escreveu depois do `@`.
pub(crate) fn candidates(people: &[Member], q: &str) -> Vec<Member> {
    let q = q.to_lowercase();
    people
        .iter()
        .filter(|p| p.0.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

/// Troca o `@…` em curso pela menção de `who`.
pub(crate) fn pick_mention(ed: &mut RichEditor, who: &Member) {
    let Some((at, _)) = mention_query(ed) else {
        return;
    };
    let head = ed.selection().head;
    let e = ed.editor_mut();
    e.set_selection(Selection::new(Caret::new(head.block, at), head));
    e.insert_mention(&who.0);
}

/// O que um acontecimento diz («moved this card from To Do to Doing»).
pub(crate) fn happened_text(k: &Kanban, w: &Happened) -> Option<String> {
    let list = |l| k.lists.get(&l).map_or(String::new(), |l| l.title.clone());
    let card = |c| k.cards.get(&c).map_or(String::new(), |c| c.title.clone());
    Some(match w {
        Happened::Created { list: Some(l) } => {
            tr_with("kanban.back.h.created", &[("list", &list(*l))])
        }
        Happened::Created { list: None } => return None,
        Happened::Converted { from } => {
            tr_with("kanban.back.h.converted", &[("card", &card(*from))])
        }
        Happened::Moved { from, to } => tr_with(
            "kanban.back.h.moved",
            &[("from", &list(*from)), ("to", &list(*to))],
        ),
        Happened::Renamed { from } => tr_with("kanban.back.h.renamed", &[("from", from)]),
        Happened::DescriptionChanged => tr("kanban.back.h.description").to_owned(),
        Happened::StartSet(Some(_)) => tr("kanban.back.h.start").to_owned(),
        Happened::StartSet(None) => tr("kanban.back.h.start_removed").to_owned(),
        Happened::DueSet(Some(_)) => tr("kanban.back.h.due").to_owned(),
        Happened::DueSet(None) => tr("kanban.back.h.due_removed").to_owned(),
        Happened::DueDone(true) => tr("kanban.back.h.done").to_owned(),
        Happened::DueDone(false) => tr("kanban.back.h.undone").to_owned(),
        Happened::MemberAdded(m) => tr_with("kanban.back.h.member_added", &[("m", &m.0)]),
        Happened::MemberRemoved(m) => tr_with("kanban.back.h.member_removed", &[("m", &m.0)]),
        Happened::Archived => tr("kanban.back.h.archived").to_owned(),
        Happened::Unarchived => tr("kanban.back.h.unarchived").to_owned(),
        Happened::Deleted => tr("kanban.back.h.deleted").to_owned(),
        Happened::Revived => tr("kanban.back.h.revived").to_owned(),
        Happened::ChecklistAdded { title, .. } => {
            tr_with("kanban.back.h.checklist_added", &[("t", title)])
        }
        Happened::ChecklistRemoved { title, .. } => {
            tr_with("kanban.back.h.checklist_removed", &[("t", title)])
        }
        Happened::ItemChecked {
            text, done: true, ..
        } => tr_with("kanban.back.h.item_done", &[("t", text)]),
        Happened::ItemChecked {
            text, done: false, ..
        } => tr_with("kanban.back.h.item_undone", &[("t", text)]),
        Happened::Attached { name, .. } => tr_with("kanban.back.h.attached", &[("t", name)]),
        Happened::Detached { name, .. } => tr_with("kanban.back.h.detached", &[("t", name)]),
    })
}

/// O que se vê sem «Show details»: nascer, mudar de lista, converter (o resto é detalhe).
fn headline(w: &Happened) -> bool {
    matches!(
        w,
        Happened::Created { .. } | Happened::Moved { .. } | Happened::Converted { .. }
    )
}

enum Entry<'a> {
    Comment(&'a Comment),
    Done(&'a Activity),
}

impl Entry<'_> {
    fn at(&self) -> Moment {
        match self {
            Entry::Comment(c) => c.at,
            Entry::Done(a) => a.at,
        }
    }
}

/// A secção «Activity»: a caixa e a lista, da mais nova para a mais antiga.
pub(crate) fn feed(ui: &mut Ui<'_>, back: &mut Back, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let meta = ui.meta();
    crate::paint::icon(
        ui.scene,
        ph2d_editor_core::IconId::History,
        Rect::new(x - u::MAIN_INDENT, y, ROW, ROW),
        meta,
    );
    ui.heading(tr("kanban.back.activity"), x, y, w);
    let s = tr(if back.details {
        "kanban.back.hide_details"
    } else {
        "kanban.back.show_details"
    });
    let bw = ui.button_w(s, false);
    ui.button(
        Rect::new(x + w - bw, y, bw, ROW),
        s,
        None,
        BackHit::Details,
        false,
    );
    let mut y = y + ROW + u::GAP + u::HAIR;
    ui.avatar(x, y, u::AVATAR, cx.me);
    let bx = x + u::AVATAR + u::GAP + u::HAIR;
    let bw = (w - u::AVATAR - u::GAP - u::HAIR).max(0.0);
    if back.comment_edit.is_none() {
        y = comment_box(ui, back, bx, y, bw);
    } else {
        y += u::ITEM_H;
    }
    y += u::POP_PAD;
    let mut entries: Vec<Entry<'_>> =
        cx.k.comments
            .values()
            .filter(|c| c.card == cx.card.id && !c.meta.deleted)
            .map(Entry::Comment)
            .collect();
    entries.extend(
        cx.k.activity_of(Subject::Card(cx.card.id))
            .into_iter()
            .filter(|a| back.details || headline(&a.what))
            .map(Entry::Done),
    );
    entries.sort_by_key(|e| std::cmp::Reverse(e.at()));
    back.feed.clear();
    for e in entries {
        y = match e {
            Entry::Comment(c) => comment(ui, back, cx, c, x, y, w),
            Entry::Done(a) => done(ui, back, cx, a, x, y, w),
        } + u::POP_PAD;
    }
    y
}

/// A caixa do comentário (nova ou a editar um): o editor e, em edição, «Save»/«Cancel».
fn comment_box(ui: &mut Ui<'_>, back: &mut Back, x: f32, y: f32, w: f32) -> f32 {
    let h = if back.comment.editing {
        back.comment.preferred_height(ui.ts, w).max(3.0 * ROW)
    } else {
        u::ITEM_H
    };
    let r = Rect::new(x, y, w, h);
    if !back.comment.editing {
        let (bg, ring) = (ui.c(T::KanbanCardRaised), ui.c(T::KanbanSlot));
        fill_rounded_rect(ui.scene, r, m::CARD_RADIUS, bg);
        stroke_rounded_rect(ui.scene, r, m::CARD_RADIUS, StrokeToken::Default.px(), ring);
    }
    paint_rich_editor(&mut back.comment, r, ui.scene, ui.ts, ui.theme);
    ui.hit(r, BackHit::CommentArea);
    let mut y = y + h;
    if back.comment.editing {
        y += u::GAP;
        let save = tr("kanban.back.save");
        let sw = ui.button_w(save, false);
        ui.primary(Rect::new(x, y, sw, ROW), save, BackHit::CommentSave);
        let cancel = tr("kanban.back.cancel");
        let cw = ui.button_w(cancel, false);
        ui.button(
            Rect::new(x + sw + u::GAP, y, cw, ROW),
            cancel,
            None,
            BackHit::CommentCancel,
            false,
        );
        y += ROW;
    }
    y
}

fn comment(
    ui: &mut Ui<'_>,
    back: &mut Back,
    cx: &Ctx<'_>,
    c: &Comment,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    back.feed.push(format!("comment:{}:{}", c.author.0, c.text));
    ui.avatar(x, y, u::AVATAR, &c.author);
    let bx = x + u::AVATAR + u::GAP + u::HAIR;
    let bw = (w - u::AVATAR - u::GAP - u::HAIR).max(0.0);
    let fg = ui.fg();
    let nw = ui.width(&c.author.0, u::BODY_PX);
    ui.strong(&c.author.0, bx, y, u::META_LINE, u::BODY_PX, bw, fg);
    let mut when = super::dates::ago(c.at, cx.now);
    if c.edited.is_some() {
        when = tr_with("kanban.back.when_edited", &[("when", &when)]);
    }
    let meta = ui.meta();
    ui.line(
        &tr_with("kanban.back.dot", &[("t", &when)]),
        bx + nw + u::SMALL_GAP,
        y,
        u::META_LINE,
        u::SMALL_PX,
        bw - nw,
        meta,
    );
    let mut y = y + u::META_LINE + u::SMALL_GAP;
    if back.comment_edit == Some(c.id) {
        return comment_box(ui, back, bx, y, bw);
    }
    let body = back.bodies.entry(c.id).or_insert_with(|| {
        let ed = RichEditor::new(
            super::node(c.card, 100 + c.id.0),
            ph2d_kanban_md::parse(&c.text),
            super::rich_labels(""),
        );
        (c.text.clone(), ed)
    });
    if body.0 != c.text {
        body.1.set_doc(ph2d_kanban_md::parse(&c.text));
        body.0.clone_from(&c.text);
    }
    let ed = &mut body.1;
    let h = ed.preferred_height(ui.ts, bw);
    let r = Rect::new(bx, y, bw, h);
    let (bg, ring) = (ui.c(T::KanbanCardRaised), ui.c(T::KanbanSlot));
    fill_rounded_rect(ui.scene, r, m::CARD_RADIUS, bg);
    stroke_rounded_rect(ui.scene, r, m::CARD_RADIUS, StrokeToken::Default.px(), ring);
    paint_rich_editor(ed, r, ui.scene, ui.ts, ui.theme);
    y += h + u::SMALL_GAP + u::HAIR;
    // As reacções (com a contagem) e, para quem escreveu, editar e apagar.
    let mut rx = bx;
    let chip_h = u::AVATAR_SMALL;
    for (i, e) in REACTIONS.iter().enumerate() {
        let n = c
            .reactions
            .iter()
            .find(|r| r.emoji == *e)
            .map_or(0, |r| r.by.len());
        let mine = c
            .reactions
            .iter()
            .any(|r| r.emoji == *e && r.by.contains(cx.me));
        let label = if n > 0 {
            format!("{e} {n}")
        } else {
            (*e).to_owned()
        };
        let cw = ui.width(&label, u::SMALL_PX) + 2.0 * u::GAP;
        let r = Rect::new(rx, y, cw, chip_h);
        let slot = ui.c(T::KanbanSlot);
        fill_rounded_rect(ui.scene, r, chip_h / 2.0, slot);
        if mine {
            let accent = ui.accent();
            stroke_rounded_rect(ui.scene, r, chip_h / 2.0, StrokeToken::Default.px(), accent);
        }
        ui.line(&label, rx + u::GAP, y, chip_h, u::SMALL_PX, cw, fg);
        #[allow(clippy::cast_possible_truncation)]
        ui.hit(r, BackHit::React(c.id, i as u8));
        rx += cw + u::SMALL_GAP;
    }
    if c.author == *cx.me {
        rx += u::GAP;
        for (key, h) in [
            ("kanban.back.edit", BackHit::CommentEdit(c.id)),
            ("kanban.back.delete", BackHit::CommentDelete(c.id)),
        ] {
            let s = tr(key);
            let sw = ui.width(s, u::SMALL_PX);
            ui.line(s, rx, y, chip_h, u::SMALL_PX, sw, meta);
            ui.hit(Rect::new(rx, y, sw, chip_h), h);
            rx += sw + u::GAP;
        }
    }
    y + chip_h
}

fn done(
    ui: &mut Ui<'_>,
    back: &mut Back,
    cx: &Ctx<'_>,
    a: &Activity,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    let Some(what) = happened_text(cx.k, &a.what) else {
        return y - u::POP_PAD;
    };
    let line = tr_with("kanban.back.who_did", &[("who", &a.by.0), ("what", &what)]);
    back.feed.push(format!("activity:{line}"));
    ui.avatar(x, y, u::AVATAR, &a.by);
    let bx = x + u::AVATAR + u::GAP + u::HAIR;
    let bw = (w - u::AVATAR - u::GAP - u::HAIR).max(0.0);
    let fg = ui.fg();
    let h = ui.text(&line, bx, y, u::BODY_PX, bw, fg);
    let meta = ui.meta();
    ui.line(
        &super::dates::ago(a.at, cx.now),
        bx,
        y + h,
        u::META_LINE,
        u::SMALL_PX,
        bw,
        meta,
    );
    y + (h + u::META_LINE).max(u::AVATAR)
}

/// A lista do `@` sob o cursor da caixa de comentário (por cima de tudo).
pub(crate) fn mentions(ui: &mut Ui<'_>, back: &mut Back, cx: &Ctx<'_>) {
    let Some((_, q)) = mention_query(&back.comment) else {
        return;
    };
    let list = candidates(&cx.people, &q);
    if list.is_empty() {
        return;
    }
    back.mention_pick = back.mention_pick.min(list.len() - 1);
    let Some(area) = ui
        .hits
        .iter()
        .rev()
        .find(|h| h.1 == BackHit::CommentArea)
        .map(|h| h.0)
    else {
        return;
    };
    let head = back.comment.selection().head;
    let caret = back.comment.layout(ui.ts, area).caret_rect(head);
    #[allow(clippy::cast_precision_loss)]
    let h = list.len() as f32 * ROW + 2.0 * u::SMALL_GAP;
    let r = Rect::new(caret.x, caret.y + caret.h + u::SMALL_GAP, u::MENTION_W, h);
    let bg = ui.c(T::KanbanCard);
    crate::paint::shadow(
        ui.scene,
        r,
        m::MENU_RADIUS,
        ui.c(T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    fill_rounded_rect(ui.scene, r, m::MENU_RADIUS, bg);
    ui.hit(r, BackHit::Inside);
    for (i, who) in list.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let row = Rect::new(r.x, r.y + u::SMALL_GAP + i as f32 * ROW, r.w, ROW);
        if i == back.mention_pick {
            let slot = ui.c(T::KanbanSlot);
            fill_rounded_rect(ui.scene, row, 0.0, slot);
        }
        ui.avatar(
            row.x + u::GAP,
            row.y + (ROW - u::AVATAR_SMALL) / 2.0,
            u::AVATAR_SMALL,
            who,
        );
        let fg = ui.fg();
        let tx = row.x + 2.0 * u::GAP + u::AVATAR_SMALL;
        ui.line(&who.0, tx, row.y, ROW, u::BODY_PX, row.w - tx + row.x, fg);
        #[allow(clippy::cast_possible_truncation)]
        ui.hit(row, BackHit::Mention(i as u8));
    }
}
