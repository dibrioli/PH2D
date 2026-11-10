//! A pintura da aba: o chão, o seletor pela porta da escolha do cartão (`Widget::Choice`) e o cartão da vista.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::cards::{CardCtx, Seg, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::{Tooltip, paint_tooltip};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::doc::{InputMapDoc, View};
use crate::layout::lay_out;

pub fn paint_doc(
    doc: &mut InputMapDoc,
    area: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = lay_out(area, text, doc.play_running());
    doc.layout = l;
    let ink = CardInk::of_list(theme);
    fill_rounded_rect(scene, area, 0.0, ink.color(ColorToken::KanbanList));
    doc.hits = HitIndex::default();
    doc.census = crate::census::Census::default();
    let labels = View::ALL.map(|v| tr(v.label_key()));
    let segs: Vec<Seg<'_>> = View::ALL
        .iter()
        .zip(labels)
        .map(|(&v, text)| Seg {
            id: v.id(),
            icon: None,
            text,
            on: v == doc.view(),
        })
        .collect();
    let mut c = CardCtx::flutuante(
        scene,
        text,
        &doc.store,
        &mut doc.hits,
        ink,
        l.selector.x,
        l.selector.w,
    );
    c.full(None, Widget::Choice(&segs), l.selector.y);
    if let Some(r) = l.banner {
        paint_play_banner(doc, r, scene, text, ink);
    }
    // A A é a superfície MAIS FUNDA e as linhas sobem dela (maquete A: painel ≈ kanban-list, linhas ≈ kanban-card).
    let body = if doc.view() == View::List {
        ColorToken::KanbanList
    } else {
        ColorToken::KanbanCard
    };
    fill_rounded_rect(scene, l.body, m::CARD_RADIUS, ink.color(body));
    match doc.view() {
        View::List => crate::view_list::paint(doc, l.body, scene, text, theme),
        View::Keyboard => crate::view_keyboard::paint(doc, l.body, scene, text, theme),
        View::TryIt => crate::view_try::paint(doc, l.body, scene, text, theme),
    }
    paint_hover_tip(doc, scene, text, theme);
}

/// O alvo da faixa do Play (a dica).
pub const PLAY_BANNER: ph2d_a11y::NodeId =
    ph2d_editor_core::registry::hash_node_id("input_map_tab.play_banner");

/// A faixa do Play: o mapa da aba é provado ao vivo e o Stop devolve o documento inteiro, o mapa incluído.
fn paint_play_banner(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    ink: CardInk,
) {
    fill_rounded_rect(
        scene,
        r,
        m::FIELD_RADIUS,
        ink.color(ColorToken::InputMapWarningBg),
    );
    doc.census.filled.push((r, ColorToken::InputMapWarningBg));
    let inner = Rect::new(
        r.x + m::CARD_PAD_X,
        r.y,
        (r.w - m::CARD_PAD_X * 2.0).max(0.0),
        r.h,
    );
    crate::view_list::text::line(
        &mut doc.census,
        text,
        scene,
        tr("input_map_tab.play_banner"),
        inner,
        m::VALUE_PX,
        ink.color(ColorToken::InputMapWarning),
    );
    doc.hits.register(PLAY_BANNER, r);
}

/// A dica do segmento sob o ponteiro, por baixo dele.
fn paint_hover_tip(
    doc: &InputMapDoc,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let (id, msg) = match doc.hover {
        Some(v) => (v.id(), tr(v.tip_key())),
        None if doc.banner_hover => (PLAY_BANNER, tr("input_map_tab.play_banner_tip")),
        None if doc.view() == View::Keyboard => {
            let Some(t) = crate::view_keyboard::tip_here(doc) else {
                return;
            };
            t
        }
        None => {
            let Some(id) = doc
                .list
                .hover_target
                .filter(|_| doc.view() != View::Keyboard)
            else {
                return;
            };
            let tip = match doc.view() {
                View::TryIt => crate::view_try::tip_of(doc, id),
                _ => crate::view_list::tip_of(doc, id),
            };
            let Some(msg) = tip else {
                return;
            };
            (id, msg)
        }
    };
    let Some(seg) = doc.hits.rect_for(id) else {
        return;
    };
    let pad = Spacing::Sm.px();
    let w = text.prefix_width(msg, TypeToken::Sm.px()) + pad * 2.0;
    let area = doc.layout.area;
    let x = seg.x.min(area.x + area.w - w - pad).max(area.x);
    let r = Rect::new(x, seg.y + seg.h + pad, w, m::FIELD_H);
    paint_tooltip(&Tooltip::new(id, msg), r, scene, text, theme);
}
