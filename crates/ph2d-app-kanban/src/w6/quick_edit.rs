//! ⭐ **A edição rápida** (`E`, o lápis, o botão direito — o idioma do Trello): o cartão abre-se NO
//! LUGAR, com o título a escrever sobre ele e o resto do quadro escurecido; ao lado, o menu
//! «Open card · Edit labels · Change members · Change cover · Edit dates · Move · Copy · Mirror ·
//! Archive». `Enter` ou «Save» grava; `Esc` desiste; clicar fora grava (o Trello).

use super::{Act, Quick};
use crate::docs::KanbanDocs;
use crate::edit::{Field, Target};
use crate::metrics as m;
use crate::paint::{c, text_top};
use crate::tab::Tab;
use ph2d_editor_core::IconId;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{CardId, Kanban};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// O menu ao lado do cartão: a largura, o espaço entre botões e a caixa mínima do título.
const SIDE_W: f32 = 184.0;
const SIDE_GAP: f32 = 4.0;
const MIN_H: f32 = 88.0;
const SAVE_W: f32 = 88.0;

/// As linhas do menu, pela ordem do Trello.
pub(crate) const ROWS: [(Quick, &str, IconId); 9] = [
    (Quick::Open, "kanban.w6.quick.open", IconId::Open),
    (Quick::Labels, "kanban.w6.quick.labels", IconId::Tag),
    (Quick::Members, "kanban.w6.quick.members", IconId::Group),
    (Quick::Cover, "kanban.w6.quick.cover", IconId::Image),
    (Quick::Dates, "kanban.w6.quick.dates", IconId::Timer),
    (Quick::Move, "kanban.w6.quick.move", IconId::ChevronRight),
    (Quick::Copy, "kanban.w6.quick.copy", IconId::Copy),
    (Quick::Mirror, "kanban.w6.quick.mirror", IconId::Clone),
    (Quick::Archive, "kanban.w6.quick.archive", IconId::Folder),
];

/// W11 — a linha a mais no canvas do Quadro: «Convert to sticky note».
const TO_NOTE: (Quick, &str, IconId) =
    (Quick::ToNote, "kanban.hooks.to_note", IconId::MessageSquare);

/// As linhas da vista `id`: as do Trello, e no Quadro também a da nota.
fn rows(id: u64) -> impl Iterator<Item = (Quick, &'static str, IconId)> {
    ROWS.into_iter()
        .chain(crate::canvas::on_board(id).then_some(TO_NOTE))
}

/// ⭐ Abre a edição rápida da linha `row`. Um espelho abre o verso do original (o título é dele).
pub(crate) fn open(docs: &mut KanbanDocs, id: u64, row: CardId) {
    let mirror = docs
        .tabs
        .iter()
        .find(|t| t.id() == id)
        .is_some_and(|t| t.rows.mirrors.contains_key(&row));
    if mirror {
        super::gesture::open_back(docs, id, row);
        return;
    }
    let title = docs.kanban_of(id).and_then(|kid| {
        docs.set
            .borrow()
            .get(kid)?
            .cards
            .get(&row)
            .map(|c| c.title.clone())
    });
    let (Some(title), Some(t)) = (title, docs.tab_mut(id)) else {
        return;
    };
    t.w6.close_top();
    t.menu = None;
    t.focus = Some(row);
    t.field = Some(Field::open(Target::CardTitle(row), &title));
    t.w6.quick = Some(row);
}

/// A caixa do título sobre o cartão (a disposição já o cresce para o texto).
fn card_rect(tab: &Tab, row: CardId) -> Option<Rect> {
    let (_, cb) = tab.layout.card(row)?;
    Some(Rect::new(
        cb.rect.x,
        cb.rect.y,
        cb.rect.w,
        cb.rect.h.max(MIN_H),
    ))
}

pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
) {
    let Some(row) = tab.w6.quick else {
        return;
    };
    let Some(r) = card_rect(tab, row) else {
        return;
    };
    let area = tab.layout.area;
    let shadow = c(theme, T::KanbanShadowFlight);
    scene.fill_rect(rect_to_vello(area), shadow);
    // Os botões são do cabeçalho (translúcidos sobre o fundo): por baixo deles, o fundo do quadro e a
    // sombra, opacos — a lista que estiver ali não se vê através do texto.
    let under = crate::paint::board_bg(theme, k.header.background);
    // O chão dos botões (o fundo sob a sombra) e o véu que o texto branco pede sobre ele.
    let floor = over(shadow, under);
    let veil = ph2d_board_render::kanban_header_veil_over(theme, floor);
    let mut hits = vec![(r, Act::Inside)];
    if let Some(f) = tab
        .field
        .as_ref()
        .filter(|f| f.target == Target::CardTitle(row))
    {
        crate::paint::editing_card(scene, ts, theme, f, r);
    }
    let save = Rect::new(r.x, r.y + r.h + m::HEADER_GAP, SAVE_W, m::BUTTON_H);
    let t = if tab.w6.hover == Some(Act::QuickSave) {
        T::AccentHover
    } else {
        T::Accent
    };
    fill_rounded_rect(scene, save, m::BUTTON_RADIUS, c(theme, t));
    let label = tr("kanban.w6.quick.save");
    let w = crate::docs::title_width(ts, label, m::BUTTON_TEXT_PX);
    let ty = text_top(ts, save.y, save.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        label,
        save.x + (save.w - w) / 2.0,
        ty,
        m::BUTTON_TEXT_PX,
        save.w,
        c(theme, T::AccentFg),
    );
    hits.push((save, Act::QuickSave));
    let right = r.x + r.w + m::HEADER_GAP;
    let x = if right + SIDE_W <= area.x + area.w {
        right
    } else {
        r.x - m::HEADER_GAP - SIDE_W
    };
    let fg = c(theme, T::KanbanHeaderFg);
    let mut y = r.y;
    for (q, key, icon) in rows(tab.id()) {
        let b = Rect::new(x, y, SIDE_W, m::BUTTON_H);
        let on = tab.w6.hover == Some(Act::Quick(q));
        let ink = if on { c(theme, T::KanbanBtnOnFg) } else { fg };
        fill_rounded_rect(scene, b, m::BUTTON_RADIUS, floor);
        let fill = if on {
            c(theme, T::KanbanHeaderBtnOn)
        } else {
            veil
        };
        fill_rounded_rect(scene, b, m::BUTTON_RADIUS, fill);
        let ir = Rect::new(b.x + m::BUTTON_PAD_X, b.y, m::ICON_PX, b.h);
        crate::paint::icon(scene, icon, ir, ink);
        let tx = ir.x + m::ICON_PX + m::LIST_ICON_GAP;
        let ty = text_top(ts, b.y, b.h, m::BUTTON_TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            tr(key),
            tx,
            ty,
            m::BUTTON_TEXT_PX,
            b.x + b.w - tx,
            ink,
        );
        hits.push((b, Act::Quick(q)));
        y += m::BUTTON_H + SIDE_GAP;
    }
    tab.w6.hits.borrow_mut().extend(hits);
}

/// Fecha a edição rápida, gravando o título.
fn done(docs: &mut KanbanDocs, id: u64) -> Option<CardId> {
    let row = docs.tab_mut(id)?.w6.quick.take();
    crate::pointer::commit_field(docs, id);
    row
}

/// O rect que o último quadro pintou para a ação `a` (a âncora de um diálogo).
pub(crate) fn rect_of(docs: &KanbanDocs, id: u64, a: Act) -> Option<Rect> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    let hits = t.w6.hits.borrow();
    hits.iter().rev().find(|(_, x)| *x == a).map(|(r, _)| *r)
}

/// ⭐ Um botão da edição rápida.
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, a: Act) {
    let Some(row) = docs.tab_mut(id).and_then(|t| t.w6.quick) else {
        return;
    };
    let side = match a {
        Act::Quick(Quick::Labels) => Some(crate::back::Side::Labels),
        Act::Quick(Quick::Members) => Some(crate::back::Side::Members),
        Act::Quick(Quick::Cover) => Some(crate::back::Side::Cover),
        Act::Quick(Quick::Dates) => Some(crate::back::Side::Dates),
        _ => None,
    };
    match a {
        Act::QuickSave => {
            done(docs, id);
        }
        Act::Quick(Quick::Open) => {
            done(docs, id);
            super::gesture::open_back(docs, id, row);
        }
        Act::Quick(Quick::Archive) => {
            done(docs, id);
            super::gesture::archive_row(docs, id, row);
        }
        Act::Quick(Quick::ToNote) => {
            done(docs, id);
            if let Some(card) = super::gesture::shows(docs, id, row) {
                ph2d_editor_core::kanban_links::request_card_to_note(card);
            }
        }
        Act::Quick(q @ (Quick::Move | Quick::Copy | Quick::Mirror)) => {
            let anchor = rect_of(docs, id, a).unwrap_or_default();
            let Some(src) = super::gesture::shows(docs, id, row) else {
                return;
            };
            let kind = match q {
                Quick::Move => super::copy_move::move_kind(docs, id, row),
                Quick::Copy => Some(super::DialogKind::CopyCard(src)),
                _ => Some(super::DialogKind::MirrorCard(src)),
            };
            if let Some(kind) = kind {
                super::copy_move::open(docs, id, kind, anchor);
            }
        }
        _ => {
            if let Some(s) = side {
                done(docs, id);
                super::gesture::open_back(docs, id, row);
                crate::back::press_side(docs, id, s);
            }
        }
    }
}

/// O teclado: `Enter` (ou `Ctrl+Enter`) grava e fecha; `Esc` desiste. O resto escreve no título.
pub(crate) fn key(
    docs: &mut KanbanDocs,
    id: u64,
    _ts: &mut TextSystem,
    input: &KeyInput<'_>,
) -> Option<KeyOutcome> {
    if !input.pressed {
        return None;
    }
    match input.key {
        Key::Escape => {
            if let Some(t) = docs.tab_mut(id) {
                t.w6.quick = None;
                t.field = None;
            }
            Some(super::taken())
        }
        Key::Enter if !input.mods.shift => {
            done(docs, id);
            Some(super::taken())
        }
        _ => None,
    }
}

/// `top` (translúcida) composta sobre `bottom` (opaca).
fn over(top: ph2d_vector::Color, bottom: ph2d_vector::Color) -> ph2d_vector::Color {
    let a = top.components[3];
    let mut out = bottom;
    for i in 0..3 {
        out.components[i] = top.components[i] * a + bottom.components[i] * (1.0 - a);
    }
    out.components[3] = 1.0;
    out
}
