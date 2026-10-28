//! ⭐ **As listas pintadas** — a aberta (cabeça, cartões, pé), a recolhida, «Add another list» e os
//! botões dos compositores. O fundo, o cabeçalho, os campos e o que voa moram em [`crate::paint`].

use crate::layout::{ListBox, limit_text, shown_text};
use crate::list_title;
use crate::metrics as m;
use crate::paint::{c, editing_card, field_box, hover_fill, icon, list_colors, shadow, text_top};
use crate::paint_card::{CardLook, Ink};
use crate::tab::{Hit, Lift, Tab};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::Kanban;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke, VectorScene};

/// A pílula do limite e a contagem da recolhida: neutras abaixo do limite, a cor do «prazo perto»
/// quando a lista está cheia (ou passou dele).
fn limit_colors(theme: Theme, full: bool, fg: Color) -> (Color, Color) {
    if full {
        (c(theme, T::KanbanDueSoon), c(theme, T::KanbanDueSoonFg))
    } else {
        (c(theme, T::KanbanSlot), fg)
    }
}

/// A lista `lb` tem limite e está cheia?
fn is_full(k: &Kanban, lb: &ListBox) -> bool {
    k.lists
        .get(&lb.id)
        .and_then(|l| l.card_limit)
        .is_some_and(|n| lb.count >= n as usize)
}

/// O texto de uma pílula, centrado em `r` (`SemiBold`, como a maquete).
fn pill_text(scene: &mut VectorScene, ts: &mut TextSystem, r: Rect, text: &str, fg: Color) {
    let y = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_title_elided(
        ts,
        scene,
        text,
        r.x + m::COUNT_PAD_X,
        y,
        m::SMALL_PX,
        r.w - 2.0 * m::COUNT_PAD_X,
        fg,
    );
}

/// O glifo de recolher (`‹›` aberta, `›‹` recolhida), na caixa de ícone centrada em `around`.
fn collapse_glyph(scene: &mut VectorScene, around: Rect, open: bool, color: Color) {
    let pts = if open {
        m::COLLAPSE_OPEN
    } else {
        m::COLLAPSE_SHUT
    };
    let mut p = BezPath::new();
    for arrow in pts {
        p.move_to(arrow[0]);
        p.line_to(arrow[1]);
        p.line_to(arrow[2]);
    }
    let s = f64::from(m::ICON_PX) / m::ICON_GRID;
    let x = f64::from(around.x + (around.w - m::ICON_PX) / 2.0).round();
    let y = f64::from(around.y + (around.h - m::ICON_PX) / 2.0).round();
    let t = Affine::translate((x, y)) * Affine::scale(s);
    scene.stroke_path(&Stroke::new(m::GLYPH_STROKE), t, &Brush::Solid(color), &p);
}

/// Uma lista aberta. Devolve os cartões pintados.
#[allow(clippy::too_many_arguments)]
pub(crate) fn list(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
    lb: &ListBox,
    looks: &mut dyn FnMut(ph2d_kanban_model::CardId) -> Option<CardLook>,
) -> Vec<ph2d_kanban_model::CardId> {
    let color = k.lists.get(&lb.id).and_then(|l| l.color);
    let (bg, fg, meta) = list_colors(theme, color);
    shadow(
        scene,
        lb.rect,
        m::LIST_RADIUS,
        c(theme, T::KanbanListShadow),
        m::SHADOW_NEAR,
    );
    fill_rounded_rect(scene, lb.rect, m::LIST_RADIUS, bg);
    let title = k.lists.get(&lb.id).map_or("", |l| l.title.as_str());
    match tab
        .field
        .as_ref()
        .filter(|f| f.target == crate::edit::Target::ListTitle(lb.id))
    {
        Some(f) => field_box(scene, ts, theme, f, lb.title, list_title::PX),
        None => {
            let y = text_top(ts, lb.title.y, lb.title.h, list_title::PX);
            let shown = list_title::paint(ts, scene, title, lb.title.x, y, lb.title.w, fg);
            noted(tab, lb, shown);
        }
    }
    let limit = k.lists.get(&lb.id).and_then(|l| l.card_limit);
    if let Some(r) = lb.limit {
        // Com o filtro, a pílula diz quantas se vêem («1 of 3»); senão, o limite.
        let (text, full) = match (lb.total, limit) {
            (Some(t), _) => (shown_text(lb.count, t), false),
            (None, Some(n)) => (limit_text(lb.count, n), is_full(k, lb)),
            (None, None) => (String::new(), false),
        };
        let (pbg, pfg) = limit_colors(theme, full, fg);
        fill_rounded_rect(scene, r, m::LIMIT_RADIUS, pbg);
        pill_text(scene, ts, r, &text, pfg);
    }
    hover_fill(
        scene,
        lb.collapse,
        tab.hover == Some(Hit::Collapse(lb.id)),
        theme,
        m::BUTTON_RADIUS,
    );
    collapse_glyph(scene, lb.collapse, true, meta);
    hover_fill(
        scene,
        lb.menu,
        tab.hover == Some(Hit::ListMenu(lb.id)),
        theme,
        m::BUTTON_RADIUS,
    );
    icon(scene, IconId::MoreHorizontal, lb.menu, meta);
    // Os cartões, recortados ao corpo: só os que se vêem são pintados (milhares cabem).
    scene.push_clip(&rect_to_vello(lb.body));
    let ink = Ink::of(theme);
    let lifted = match tab.drag.map(|d| d.lift) {
        Some(Lift::Card(id)) => Some(id),
        _ => None,
    };
    if let Some(s) = lb.slot {
        fill_rounded_rect(scene, s, m::CARD_RADIUS, c(theme, T::KanbanSlot));
    }
    let mut painted = Vec::new();
    if lb.total.is_some() && lb.cards.is_empty() && lb.composer.is_none() {
        let y = text_top(ts, lb.body.y + m::LIST_BODY_TOP, m::LINE_H, m::SMALL_PX);
        let x = lb.body.x + m::LIST_BODY_X + m::CARD_PAD_X;
        paint_text_elided(
            ts,
            scene,
            tr("kanban.w6.no_match"),
            x,
            y,
            m::SMALL_PX,
            lb.body.w - 2.0 * (x - lb.body.x),
            meta,
        );
    }
    let aging = k.header.aging;
    for cb in &lb.cards {
        if cb.rect.y > lb.body.y + lb.body.h || cb.rect.y + cb.rect.h < lb.body.y {
            continue;
        }
        if Some(cb.id) == lifted {
            continue;
        }
        let Some(mut look) = looks(cb.id) else {
            continue;
        };
        look.hover = tab.hover == Some(Hit::Card(cb.id)) || tab.hover == Some(Hit::CardEdit(cb.id));
        look.focus = tab.focus == Some(cb.id);
        match tab
            .field
            .as_ref()
            .filter(|f| f.target == crate::edit::Target::CardTitle(cb.id))
        {
            Some(f) => editing_card(scene, ts, theme, f, cb.rect),
            None => crate::w6::aging::card(
                scene,
                ts,
                theme,
                &ink,
                &look,
                cb.rect,
                tab.label_text,
                (aging, look.age, cb.id.0),
            ),
        }
        painted.push(cb.id);
    }
    if let (Some(r), Some(f)) = (lb.composer, tab.field.as_ref()) {
        editing_card(scene, ts, theme, f, r);
    }
    scene.pop_layer();
    // O pé: «Add a card», ou os botões do compositor.
    match lb.close {
        Some(close) => composer_buttons(
            scene,
            ts,
            theme,
            lb.add,
            close,
            "kanban.add_card_button",
            meta,
            tab,
        ),
        None => {
            hover_fill(
                scene,
                lb.add,
                tab.hover == Some(Hit::AddCard(lb.id)),
                theme,
                m::ADD_RADIUS,
            );
            let ir = Rect::new(lb.add.x + m::ADD_PAD_X, lb.add.y, m::ICON_PX, lb.add.h);
            icon(scene, IconId::Plus, ir, meta);
            let x = ir.x + m::ICON_PX + m::LIST_ICON_GAP;
            let y = text_top(ts, lb.add.y, lb.add.h, m::BUTTON_TEXT_PX);
            paint_text_elided(
                ts,
                scene,
                tr("kanban.add_card"),
                x,
                y,
                m::BUTTON_TEXT_PX,
                lb.add.w - (x - lb.add.x),
                meta,
            );
            if let Some(r) = lb.template {
                hover_fill(
                    scene,
                    r,
                    tab.hover == Some(Hit::TemplateBtn(lb.id)),
                    theme,
                    m::ADD_RADIUS,
                );
                icon(scene, IconId::File, r, meta);
            }
        }
    }
    painted
}

/// O título pintado entra no registo do quadro (a sonda compara-o com a medida).
fn noted(tab: &Tab, lb: &ListBox, shown: String) {
    tab.painted.borrow_mut().titles.push((
        lb.id,
        shown,
        list_title::PX,
        list_title::WEIGHT.value(),
    ));
}

/// Uma lista recolhida: o botão, o título na vertical e a contagem.
pub(crate) fn collapsed(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &Tab,
    lb: &ListBox,
) {
    let color = k.lists.get(&lb.id).and_then(|l| l.color);
    let (bg, fg, meta) = list_colors(theme, color);
    shadow(
        scene,
        lb.rect,
        m::LIST_RADIUS,
        c(theme, T::KanbanListShadow),
        m::SHADOW_NEAR,
    );
    fill_rounded_rect(scene, lb.rect, m::LIST_RADIUS, bg);
    hover_fill(
        scene,
        lb.collapse,
        tab.hover == Some(Hit::Collapse(lb.id)),
        theme,
        m::BUTTON_RADIUS,
    );
    collapse_glyph(scene, lb.collapse, false, meta);
    let title = k.lists.get(&lb.id).map_or("", |l| l.title.as_str());
    let top = lb.collapse.y + lb.collapse.h + m::LIST_ICON_GAP;
    let room =
        (lb.rect.y + lb.rect.h - top - m::COUNT_H - m::LIST_HEAD_TOP - m::LIST_ICON_GAP).max(0.0);
    // O título deitado, rodado 90° no sentido do relógio (o `writing-mode: vertical-rl` da maquete).
    let mut s = VectorScene::new();
    let shown = list_title::paint(ts, &mut s, title, 0.0, 0.0, room, fg);
    noted(tab, lb, shown);
    let x = f64::from(lb.rect.x + (lb.rect.w + m::LINE_H) / 2.0);
    let t = Affine::translate((x, f64::from(top))) * Affine::rotate(std::f64::consts::FRAC_PI_2);
    scene.append_transformed(&s, t);
    let n = lb.count.to_string();
    let w = crate::docs::title_width(ts, &n, m::SMALL_PX) + 2.0 * m::COUNT_PAD_X;
    let pill = Rect::new(
        lb.rect.x + (lb.rect.w - w) / 2.0,
        top + room + m::LIST_ICON_GAP,
        w,
        m::COUNT_H,
    );
    let (pbg, pfg) = limit_colors(theme, is_full(k, lb), fg);
    fill_rounded_rect(scene, pill, m::COUNT_RADIUS, pbg);
    pill_text(scene, ts, pill, &n, pfg);
}

/// «Add another list» — ou o compositor de lista.
pub(crate) fn add_list(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, tab: &Tab) {
    let l = &tab.layout;
    let fg = c(theme, T::KanbanHeaderFg);
    match (l.add_list_field, l.add_list_close, tab.field.as_ref()) {
        (Some(fr), Some(close), Some(f)) => {
            let bottom = l.add_list.y + l.add_list.h + m::LIST_FOOT_BOTTOM;
            let back = Rect::new(
                fr.x - m::LIST_BODY_X,
                fr.y - m::LIST_BODY_X,
                m::LIST_W,
                bottom - fr.y + m::LIST_BODY_X,
            );
            fill_rounded_rect(scene, back, m::LIST_RADIUS, c(theme, T::KanbanList));
            field_box(scene, ts, theme, f, fr, list_title::PX);
            let meta = c(theme, T::KanbanListMeta);
            composer_buttons(
                scene,
                ts,
                theme,
                l.add_list,
                close,
                "kanban.add_list_button",
                meta,
                tab,
            );
        }
        _ => {
            let on = tab.hover == Some(Hit::AddList);
            let fill = if on {
                T::KanbanHeaderBtn
            } else {
                T::KanbanNewList
            };
            fill_rounded_rect(scene, l.add_list, m::LIST_RADIUS, c(theme, fill));
            let ir = Rect::new(
                l.add_list.x + m::NEW_LIST_PAD_X,
                l.add_list.y,
                m::ICON_PX,
                l.add_list.h,
            );
            icon(scene, IconId::Plus, ir, fg);
            let x = ir.x + m::ICON_PX + m::HEADER_GAP;
            let y = text_top(ts, l.add_list.y, l.add_list.h, m::BUTTON_TEXT_PX);
            paint_text_elided(
                ts,
                scene,
                tr("kanban.add_list"),
                x,
                y,
                m::BUTTON_TEXT_PX,
                l.add_list.w - (x - l.add_list.x),
                fg,
            );
        }
    }
}

/// O botão azul de um compositor («Add card» / «Add list») e o «×».
#[allow(clippy::too_many_arguments)]
fn composer_buttons(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    add: Rect,
    close: Rect,
    key: &str,
    meta: Color,
    tab: &Tab,
) {
    let fill = if tab.hover == Some(Hit::FieldAdd) {
        T::AccentHover
    } else {
        T::Accent
    };
    fill_rounded_rect(scene, add, m::BUTTON_RADIUS, c(theme, fill));
    let y = text_top(ts, add.y, add.h, m::BUTTON_TEXT_PX);
    let label = tr(key);
    let w = crate::docs::title_width(ts, label, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        label,
        add.x + ((add.w - w) / 2.0).max(m::BUTTON_PAD_X),
        y,
        m::BUTTON_TEXT_PX,
        add.w,
        c(theme, T::AccentFg),
    );
    hover_fill(
        scene,
        close,
        tab.hover == Some(Hit::FieldClose),
        theme,
        m::BUTTON_RADIUS,
    );
    icon(scene, IconId::Close, close, meta);
}
