//! A coluna lateral do verso: «Add to card» e «Actions» (a maquete, com o Trello por régua).

use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{BackHit, ROW, Side};
use ph2d_editor_core::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};

/// O rótulo, o ícone e se está «ligado» de cada botão.
fn button(cx: &Ctx<'_>, s: Side) -> (String, IconId, bool) {
    let card = cx.card;
    let t = |k: &str| tr(k).to_owned();
    match s {
        Side::Members => (t("kanban.back.side.members"), IconId::Group, false),
        Side::Labels => (t("kanban.back.side.labels"), IconId::Tag, false),
        Side::Checklist => (t("kanban.back.side.checklist"), IconId::Check, false),
        Side::Dates => (t("kanban.back.side.dates"), IconId::Timer, false),
        Side::Attachment => (t("kanban.back.side.attachment"), IconId::Link, false),
        Side::Cover => (t("kanban.back.side.cover"), IconId::Image, false),
        Side::Fields => (t("kanban.back.side.fields"), IconId::Menu, false),
        Side::Move => (t("kanban.back.side.move"), IconId::ChevronRight, false),
        Side::Copy => (t("kanban.back.side.copy"), IconId::Copy, false),
        Side::Mirror => (t("kanban.w6.side.mirror"), IconId::Clone, false),
        Side::Template if card.template => (t("kanban.w6.side.template_on"), IconId::File, true),
        Side::Template => (t("kanban.back.side.template"), IconId::File, false),
        Side::FromTemplate => (t("kanban.w6.side.from_template"), IconId::Plus, false),
        Side::Archive if card.archived => (t("kanban.back.side.restore"), IconId::Undo, false),
        Side::Archive => (t("kanban.back.side.archive"), IconId::Folder, false),
        Side::Delete => (t("kanban.back.side.delete"), IconId::Trash, false),
        Side::Share => (t("kanban.back.side.share"), IconId::Export, false),
        Side::Watch => {
            let on = card.watchers.contains(cx.me);
            let k = if on {
                "kanban.back.side.watching"
            } else {
                "kanban.back.side.watch"
            };
            (t(k), IconId::Eye, on)
        }
        Side::SceneObject => (t("kanban.links.add"), IconId::Cube, false),
        Side::AddButton => (t("kanban.auto.ui.add_button"), IconId::Plus, false),
        Side::ToNote => (t("kanban.links.to_note"), IconId::MessageSquare, false),
        Side::Vote => {
            let on = card.votes.contains(cx.me);
            let n = card.votes.len();
            let s = if n > 0 {
                tr_with("kanban.back.side.votes", &[("n", &n)])
            } else {
                t("kanban.back.side.vote")
            };
            (s, IconId::Vote, on)
        }
    }
}

const ADD: [Side; 8] = [
    Side::Members,
    Side::Labels,
    Side::Checklist,
    Side::Dates,
    Side::Attachment,
    Side::Cover,
    Side::Fields,
    Side::SceneObject,
];

const ACTIONS: [Side; 9] = [
    Side::Move,
    Side::Copy,
    Side::Mirror,
    Side::Template,
    Side::Archive,
    Side::Delete,
    Side::Share,
    Side::Watch,
    Side::Vote,
];

/// Pinta a coluna a partir de `y`; devolve o fundo.
pub(crate) fn paint(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32) -> f32 {
    let mut y = y;
    // Um modelo oferece «Create card from template» à cabeça das ações (o Trello).
    let mut actions = ACTIONS.to_vec();
    if cx.card.template {
        actions.insert(0, Side::FromTemplate);
    }
    if cx.on_board {
        actions.push(Side::ToNote);
    }
    for (caption, list) in [
        ("kanban.back.add_to_card", &ADD[..]),
        ("kanban.back.actions", &actions[..]),
    ] {
        ui.caption(tr(caption), x, y, u::SIDE_W);
        y += u::META_LINE;
        for s in list {
            let (label, icon, on) = button(cx, *s);
            ui.button(
                Rect::new(x, y, u::SIDE_W, ROW),
                &label,
                Some(icon),
                BackHit::Side(*s),
                on,
            );
            y += ROW + u::SMALL_GAP + 2.0;
        }
        y += u::GAP + u::SMALL_GAP;
    }
    buttons(ui, cx, x, y)
}

/// J2 — «Buttons» (o Butler, como no Trello): cada botão de cartão ligado, e «Add button».
fn buttons(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32) -> f32 {
    use crate::auto::buttons::{enabled, is_card_button};
    let mut y = y;
    ui.caption(tr("kanban.auto.ui.buttons"), x, y, u::SIDE_W);
    y += u::META_LINE;
    for (a, s) in enabled(cx.k, is_card_button) {
        let r = Rect::new(x, y, u::SIDE_W, ROW);
        ui.button(
            r,
            &s.name,
            Some(IconId::Bolt),
            BackHit::CardButton(a),
            false,
        );
        y += ROW + u::SMALL_GAP + u::HAIR;
    }
    let (label, icon, on) = button(cx, Side::AddButton);
    let r = Rect::new(x, y, u::SIDE_W, ROW);
    ui.button(r, &label, Some(icon), BackHit::Side(Side::AddButton), on);
    y + ROW + u::GAP
}
