//! A lista «Actions» por grupo (PLATFORM PLAYER · TOP-DOWN PLAYER · YOURS): ponto da cor, nome, quantas
//! ligações; a aberta acesa; «+ Add» (abre o campo do nome no lugar) e «Remove» em baixo, como em Animations.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::state::KbHit;
use super::{Group, group_of};
use crate::doc::InputMapDoc;
use crate::ids;
use crate::interact::{Control, ControlRole};
use crate::view_list::text::line;

const ROW_H: f32 = m::FIELD_H - m::SEG_PAD;
/// O marcador da cor da acção: um quadrado de cantos redondos (maquete B1).
const MARK: f32 = m::KEY_MARK_BOX - m::SEG_PAD;

const GROUPS: [(Group, &str); 3] = [
    (
        Group::PlatformPlayer,
        "input_map_tab.reader.platform_player",
    ),
    (Group::TopDownPlayer, "input_map_tab.reader.top_down_player"),
    (Group::Yours, "input_map_keyboard.yours"),
];

fn count_text(n: usize) -> String {
    if n == 0 {
        tr("input_map_keyboard.none").to_owned()
    } else {
        n.to_string()
    }
}

/// A largura natural da coluna: o nome mais largo, a contagem, e os dois botões.
pub(super) fn width(doc: &InputMapDoc, ts: &mut TextSystem) -> f32 {
    let count_w = ts.prefix_width(tr("input_map_keyboard.none"), m::VALUE_PX);
    let names = doc
        .snap
        .map
        .actions()
        .iter()
        .map(|a| ts.prefix_width(&a.name, m::VALUE_PX))
        .fold(0.0_f32, f32::max);
    let heads = GROUPS
        .iter()
        .map(|(_, k)| ts.prefix_width(&tr(k).to_uppercase(), m::LABEL_PX))
        .fold(0.0_f32, f32::max);
    let row = MARK + m::SEG_INSET * 4.0 + names + count_w;
    let buttons = [
        tr("input_map_keyboard.add"),
        tr("input_map_keyboard.remove"),
    ]
    .iter()
    .map(|t| ts.prefix_width(t, m::VALUE_PX) + m::BUTTON_TEXT_PAD * 2.0)
    .sum::<f32>()
        + m::FIELD_GAP;
    row.max(heads + m::SEG_INSET * 2.0).max(buttons) + m::LIST_PAD_X * 2.0
}

pub(super) fn paint(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let x = r.x + m::LIST_PAD_X;
    let w = (r.w - m::LIST_PAD_X * 2.0).max(0.0);
    let mut y = r.y + m::LIST_PAD_X;
    let head = Rect::new(x + m::SEG_INSET, y, w, ROW_H);
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_keyboard.actions"),
        head,
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += ROW_H;
    let open = doc.kb.open;
    for (g, key) in GROUPS {
        let rows: Vec<(ph2d_input::ActionId, String, usize)> = doc
            .snap
            .map
            .actions()
            .iter()
            .filter(|a| group_of(doc, &a.name) == g)
            .map(|a| (a.id, a.name.clone(), a.bindings.len()))
            .collect();
        if rows.is_empty() {
            continue;
        }
        let gh = Rect::new(x + m::SEG_INSET, y, w - m::SEG_INSET, ROW_H);
        line(
            &mut doc.census,
            ts,
            scene,
            &tr(key).to_uppercase(),
            gh,
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
        y += ROW_H;
        for (id, name, n) in rows {
            let row = Rect::new(x, y, w, ROW_H);
            y += ROW_H;
            let rid = KbHit::Row(id).id();
            let hover = doc.is_hovered(rid);
            if doc.is_pressed(rid) {
                fill_rounded_rect(scene, row, m::SEG_RADIUS, ink.color(ColorToken::AccentSoft));
                crate::view_list::ring(scene, row, ink);
            } else if open == Some(id) {
                fill_rounded_rect(scene, row, m::SEG_RADIUS, ink.color(ColorToken::AccentSoft));
            } else if hover {
                fill_rounded_rect(
                    scene,
                    row,
                    m::SEG_RADIUS,
                    ink.color(ColorToken::CardFieldHover),
                );
            }
            let dot = MARK;
            let dr = Rect::new(row.x + m::SEG_INSET, row.y + (row.h - dot) * 0.5, dot, dot);
            fill_rounded_rect(
                scene,
                dr,
                m::SEG_PAD,
                ink.color(crate::palette::of_action(id)),
            );
            let count = count_text(n);
            let cw = ts.prefix_width(&count, m::VALUE_PX);
            let cr = Rect::new(row.x + row.w - m::SEG_INSET - cw, row.y, cw, row.h);
            let nx = dr.x + dot + m::SEG_INSET;
            let nr = Rect::new(nx, row.y, (cr.x - nx - m::SEG_INSET).max(0.0), row.h);
            line(
                &mut doc.census,
                ts,
                scene,
                &name,
                nr,
                m::VALUE_PX,
                ink.color(ColorToken::Text1),
            );
            line(
                &mut doc.census,
                ts,
                scene,
                &count,
                cr,
                m::VALUE_PX,
                ink.color(ColorToken::Text3),
            );
            doc.kb.targets.push(KbHit::Row(id));
            doc.control(Control::new(rid, row, ControlRole::ListItem, name));
        }
    }
    // Logo abaixo do último grupo (maquete B1), nunca presos ao fundo de um cartão alto.
    y += m::FIELD_ROW_GAP;
    let (add, remove, name) = (
        tr("input_map_keyboard.add"),
        tr("input_map_keyboard.remove"),
        tr("input_map_list.new_name"),
    );
    if doc.kb.adding {
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, x, w);
        y = c.full(
            None,
            Widget::Text {
                id: ids::NEW_NAME,
                placeholder: name,
            },
            y,
        ) + m::FIELD_ROW_GAP;
    }
    // «+ Add» e «Remove» compactos, na largura do texto (maquete B1), encostados à esquerda.
    let (mut bx, mut end) = (x, y);
    let remove_off = open.is_none().then_some("input_map_keyboard.remove_off_none");
    if let Some(ph2d_editor_core::interaction::InteractiveState::Button { state }) =
        doc.store.get_mut(KbHit::Remove.id())
    {
        *state = if remove_off.is_some() {
            ph2d_editor_core::widget::ButtonState::Disabled
        } else {
            ph2d_editor_core::widget::ButtonState::Normal
        };
    }
    for (h, text) in [(KbHit::Add, add), (KbHit::Remove, remove)] {
        let bw = button_w(ts, text).min((x + w - bx).max(0.0));
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, bx, bw);
        let widget = Widget::Button {
            id: h.id(),
            text,
            primary: false,
        };
        end = c.full(None, widget, y);
        bx += bw + m::FIELD_GAP;
        doc.kb.targets.push(h);
        if let Some(r) = doc.hits.rect_for(h.id()) {
            if doc.is_pressed(h.id()) {
                crate::view_list::ring(scene, r, ink);
            }
            let off = if h == KbHit::Remove { remove_off } else { None };
            doc.control(Control::new(h.id(), r, ControlRole::Button, text).disabled(off));
        }
    }
    end + m::LIST_PAD_BOTTOM
}

/// A largura de um botão compacto: o texto e o recuo do cartão dos dois lados.
#[must_use]
pub fn button_w(ts: &mut TextSystem, text: &str) -> f32 {
    ts.prefix_width(text, m::VALUE_PX) + m::CARD_PAD_X * 2.0
}

/// O polegar discreto da lista que rola (o da A e da C): sem trilho, na margem direita; desvanece quente.
pub(super) fn scrollbar(doc: &mut InputMapDoc, view: Rect, total: f32, scene: &mut VectorScene, ink: CardInk) {
    use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
    let lane = Rect::new(view.x, view.y, view.w - m::SEG_INSET, view.h);
    let thumb = scrollbar_thumb_rect(scrollbar_track_rect(lane), doc.kb.scroll, total, view.h);
    let pill = Rect::new(thumb.x + thumb.w - m::BAR_TRACK, thumb.y, m::BAR_TRACK, thumb.h);
    fill_rounded_rect(scene, pill, m::BAR_TRACK * 0.5, ink.color(ColorToken::BorderEmph));
    let fade = doc.thumb_fade(KbHit::Thumb.id());
    if fade > 0.0 {
        let hot = ink.color(ColorToken::Text2).multiply_alpha(fade);
        fill_rounded_rect(scene, pill, m::BAR_TRACK * 0.5, hot);
    }
    doc.kb.targets.push(KbHit::Thumb);
    let label = tr("input_map_keyboard.scrollbar");
    doc.control(Control::new(KbHit::Thumb.id(), thumb, ControlRole::ScrollThumb, label));
}
