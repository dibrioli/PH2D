//! Os quatro cartões da vista C: Start from · Try it · The stick · Controls (C1–C4 da maquete).

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::ActionId;
use ph2d_input::recipes::Recipe;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::state::TryHit;
use super::{line_h, para, para_h, target, wrap};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;

pub(super) const PAD: f32 = m::CARD_PAD_X;
pub(super) const ROW: f32 = m::FIELD_H;
pub(super) const GAP: f32 = m::FIELD_ROW_GAP;

/// Um texto numa linha de altura `ROW`, na largura natural (até `max_w`); devolve a largura usada.
pub(super) fn word(
    doc: &mut InputMapDoc,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    (x, y, max_w): (f32, f32, f32),
    px: f32,
    color: ph2d_vector::Color,
) -> f32 {
    let w = ts.prefix_width(s, px).min(max_w);
    line(
        &mut doc.census,
        ts,
        scene,
        s,
        Rect::new(x, y, w, ROW),
        px,
        color,
    )
}

pub(super) fn inner_w(w: f32) -> f32 {
    (w - PAD * 2.0).max(0.0)
}

fn recipe_cols(w: f32) -> usize {
    if inner_w(w) / 4.0 >= ROW * 6.0 { 4 } else { 2 }
}

fn names_of(r: Recipe) -> String {
    let names: Vec<&str> = r.actions().iter().map(|a| a.name).collect();
    if names.is_empty() {
        tr(r.blurb_key()).to_owned()
    } else {
        names.join(" \u{00B7} ")
    }
}

fn tile_w(w: f32) -> f32 {
    let n = recipe_cols(w) as f32;
    (inner_w(w) - m::FIELD_GAP * (n - 1.0)) / n
}

fn tile_h(ts: &mut TextSystem, w: f32) -> f32 {
    let tw = tile_w(w) - m::SEG_INSET * 2.0;
    let lines = Recipe::ALL
        .iter()
        .map(|r| wrap(ts, &names_of(*r), m::LABEL_PX, tw).len())
        .max()
        .unwrap_or(1);
    line_h(m::VALUE_PX) + lines as f32 * line_h(m::LABEL_PX) + m::SEG_INSET
}

fn confirm_text(doc: &InputMapDoc, r: Recipe) -> String {
    let n = doc.snap.map.len();
    let k = if n == 1 {
        "input_map_try.confirm_one"
    } else {
        "input_map_try.confirm_many"
    };
    tr(k)
        .replace("{n}", &n.to_string())
        .replace("{recipe}", tr(r.label_key()))
}

pub(super) fn recipes_h(doc: &mut InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
    let rows = (Recipe::ALL.len() / recipe_cols(w)) as f32;
    let iw = inner_w(w) - m::SEG_INSET * 2.0;
    let cap = para_h(ts, tr("input_map_try.recipe_caption"), m::LABEL_PX, iw);
    let confirm = doc.tryit.confirm.map_or(0.0, |r| {
        para_h(ts, &confirm_text(doc, r), m::VALUE_PX, iw) + ROW + GAP * 2.0
    });
    m::CARD_PAD_TOP
        + line_h(m::LABEL_PX)
        + (tile_h(ts, w) + GAP) * rows
        + cap
        + GAP
        + confirm
        + m::CARD_PAD_BOTTOM
}

pub(super) fn recipes(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let (x, w) = (r.x + PAD, inner_w(r.w));
    let mut y = r.y + m::CARD_PAD_TOP;
    let label = Rect::new(x, y, w, line_h(m::LABEL_PX));
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_try.start_from"),
        label,
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += line_h(m::LABEL_PX);
    let (cols, tw, th) = (recipe_cols(r.w), tile_w(r.w), tile_h(ts, r.w));
    for (i, rc) in Recipe::ALL.iter().enumerate() {
        let tx = x + (tw + m::FIELD_GAP) * (i % cols) as f32;
        let ty = y + (th + GAP) * (i / cols) as f32;
        let tile = Rect::new(tx, ty, tw, th);
        let on = doc.tryit.confirm == Some(*rc);
        let bg = if on {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(scene, tile, m::FIELD_RADIUS, ink.color(bg));
        let ix = tx + m::SEG_INSET;
        let iw = tw - m::SEG_INSET * 2.0;
        let title = Rect::new(ix, ty, iw, line_h(m::VALUE_PX));
        crate::view_list::text::title(
            &mut doc.census,
            ts,
            scene,
            tr(rc.label_key()),
            title,
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        para(
            doc,
            ts,
            scene,
            &names_of(*rc),
            (ix, ty + line_h(m::VALUE_PX), iw),
            m::LABEL_PX,
            ink.color(ColorToken::Text2),
        );
        target(doc, TryHit::Recipe(*rc), tile);
    }
    y += (th + GAP) * (Recipe::ALL.len() / cols) as f32;
    if let Some(rc) = doc.tryit.confirm {
        let q = confirm_text(doc, rc);
        y = para(
            doc,
            ts,
            scene,
            &q,
            (x, y, w),
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        ) + GAP;
        let bw = (w - m::FIELD_GAP) * 0.5;
        for (i, (h, k, primary)) in [
            (TryHit::Replace, "input_map_try.replace", true),
            (TryHit::Cancel, "input_map_try.cancel", false),
        ]
        .into_iter()
        .enumerate()
        {
            let bx = x + (bw + m::FIELD_GAP) * i as f32;
            let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, bx, bw);
            c.full(
                None,
                Widget::Button {
                    id: h.id(),
                    text: tr(k),
                    primary,
                },
                y,
            );
            doc.tryit.targets.push(h);
        }
        y += ROW + GAP;
    }
    let cap = tr("input_map_try.recipe_caption");
    let ch = para_h(ts, cap, m::LABEL_PX, w - m::SEG_INSET * 2.0);
    fill_rounded_rect(
        scene,
        Rect::new(x, y, w, ch),
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    para(
        doc,
        ts,
        scene,
        cap,
        (x + m::SEG_INSET, y, w - m::SEG_INSET * 2.0),
        m::LABEL_PX,
        ink.color(ColorToken::Text2),
    );
}

fn bar_cols(w: f32) -> usize {
    ((inner_w(w) / (ROW * 8.0)) as usize).clamp(1, 3)
}

/// O pé do Try it: «You are pressing Up Arrow: jump and move_up are on. …».
pub(crate) fn footer(doc: &InputMapDoc) -> String {
    if doc.snap.play_running {
        return tr("input_map_try.idle_running").to_owned();
    }
    let keys = doc.tryit.held_keys();
    if keys.is_empty() {
        return tr("input_map_try.idle").to_owned();
    }
    let names: Vec<String> = keys.iter().map(|k| k.label()).collect();
    let lead =
        tr("input_map_try.pressing").replace("{keys}", &names.join(tr("input_map_try.list_sep")));
    let on: Vec<&str> = doc
        .snap
        .map
        .actions()
        .iter()
        .filter(|a| doc.tryit.actions.sample(a.id).strength > 0.0)
        .map(|a| a.name.as_str())
        .collect();
    let rest = match on.len() {
        0 => tr("input_map_try.on_none").to_owned(),
        1 => tr("input_map_try.on_one").replace("{actions}", on[0]),
        _ => tr("input_map_try.on_many").replace("{actions}", &on.join(tr("input_map_try.and"))),
    };
    format!("{lead} {rest}")
}

pub(super) fn live_h(doc: &mut InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
    let n = doc.snap.map.len().div_ceil(bar_cols(w)) as f32;
    let foot = para_h(
        ts,
        &footer(doc),
        m::VALUE_PX,
        inner_w(w) - m::SEG_INSET * 2.0,
    );
    m::CARD_PAD_TOP
        + ROW
        + crate::view_keyboard::keys::keyboard_h(inner_w(w))
        + GAP
        + n * line_h(m::LABEL_PX)
        + GAP
        + foot
        + m::CARD_PAD_BOTTOM
}

pub(super) fn live(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let (x, w) = (r.x + PAD, inner_w(r.w));
    let mut y = r.y + m::CARD_PAD_TOP;
    let t = tr("input_map_try.try_it");
    let tw = word(
        doc,
        ts,
        scene,
        t,
        (x, y, w),
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    let lv = tr("input_map_try.live");
    let lx = x + tw + m::SEG_ICON_GAP;
    let lw = word(
        doc,
        ts,
        scene,
        lv,
        (lx, y, w),
        m::LABEL_PX,
        ink.color(ColorToken::Success),
    );
    let state = tr(if doc.snap.play_running {
        "input_map_try.running"
    } else {
        "input_map_try.stopped"
    });
    let sw = ts
        .prefix_width(state, m::LABEL_PX)
        .min((w - tw - lw).max(0.0));
    line(
        &mut doc.census,
        ts,
        scene,
        state,
        Rect::new(x + w - sw, y, sw, ROW),
        m::LABEL_PX,
        ink.color(ColorToken::Text2),
    );
    y += ROW;
    let dh = crate::view_keyboard::keys::keyboard_h(w);
    let draw = Rect::new(x, y, w, dh);
    fill_rounded_rect(
        scene,
        draw,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanCard),
    );
    let held = doc.tryit.held_keys();
    crate::view_keyboard::keys::paint_keys(doc, draw, scene, ts, ink, &|k| held.contains(&k));
    y += dh + GAP;
    let cols = bar_cols(r.w);
    let cw = (w - m::FIELD_GAP * (cols as f32 - 1.0)) / cols as f32;
    let actions: Vec<(ActionId, String)> = doc
        .snap
        .map
        .actions()
        .iter()
        .map(|a| (a.id, a.name.clone()))
        .collect();
    for (i, (id, name)) in actions.iter().enumerate() {
        let cx = x + (cw + m::FIELD_GAP) * (i % cols) as f32;
        let cy = y + line_h(m::LABEL_PX) * (i / cols) as f32;
        bar_row(
            doc,
            (*id, name),
            Rect::new(cx, cy, cw, line_h(m::LABEL_PX)),
            scene,
            ts,
            ink,
        );
    }
    y += line_h(m::LABEL_PX) * actions.len().div_ceil(cols) as f32 + GAP;
    let foot = footer(doc);
    let fh = para_h(ts, &foot, m::VALUE_PX, w - m::SEG_INSET * 2.0);
    fill_rounded_rect(
        scene,
        Rect::new(x, y, w, fh),
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    para(
        doc,
        ts,
        scene,
        &foot,
        (x + m::SEG_INSET, y, w - m::SEG_INSET * 2.0),
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
}

/// Uma acção: a amostra da cor, o nome, a barra da força 0.00–1.00 e o número.
fn bar_row(
    doc: &mut InputMapDoc,
    (id, name): (ActionId, &str),
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let s = doc.tryit.actions.sample(id).strength;
    let color = ink.color(crate::palette::of_action(id));
    let sq = m::SEG_PAD * 2.0;
    fill_rounded_rect(
        scene,
        Rect::new(r.x, r.y + (r.h - sq) * 0.5, sq, sq),
        m::FIELD_BORDER,
        color,
    );
    let nx = r.x + sq + m::SEG_ICON_GAP;
    let nw = ts.prefix_width(name, m::LABEL_PX);
    let fg = ink.color(if s > 0.0 {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    });
    line(
        &mut doc.census,
        ts,
        scene,
        name,
        Rect::new(nx, r.y, nw, r.h),
        m::LABEL_PX,
        fg,
    );
    let v = format!("{s:.2}");
    let vw = ts.prefix_width(&v, m::LABEL_PX);
    line(
        &mut doc.census,
        ts,
        scene,
        &v,
        Rect::new(r.x + r.w - vw, r.y, vw, r.h),
        m::LABEL_PX,
        fg,
    );
    let bx = nx + nw + m::SEG_ICON_GAP;
    let bw = (r.x + r.w - vw - m::SEG_ICON_GAP - bx).max(0.0);
    let bar = Rect::new(bx, r.y + (r.h - m::SEG_PAD) * 0.5, bw, m::SEG_PAD);
    fill_rounded_rect(
        scene,
        bar,
        m::SEG_PAD * 0.5,
        ink.color(ColorToken::CardField),
    );
    fill_rounded_rect(
        scene,
        Rect::new(bx, bar.y, bw * s, bar.h),
        m::SEG_PAD * 0.5,
        color,
    );
}
