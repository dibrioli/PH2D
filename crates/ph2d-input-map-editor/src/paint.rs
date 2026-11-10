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
    let l = lay_out(area, text);
    doc.layout = l;
    let ink = CardInk::of_list(theme);
    fill_rounded_rect(scene, area, 0.0, ink.color(ColorToken::KanbanList));
    doc.hits = HitIndex::default();
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
    fill_rounded_rect(scene, l.body, m::CARD_RADIUS, ink.color(ColorToken::KanbanCard));
    match doc.view() {
        View::List => crate::view_list::paint(doc, l.body, scene, text, theme),
        View::Keyboard => crate::view_keyboard::paint(doc, l.body, scene, text, theme),
        View::TryIt => crate::view_try::paint(doc, l.body, scene, text, theme),
    }
    paint_hover_tip(doc, scene, text, theme);
}

/// A dica do segmento sob o ponteiro, por baixo dele.
fn paint_hover_tip(doc: &InputMapDoc, scene: &mut VectorScene, text: &mut TextSystem, theme: Theme) {
    let Some(v) = doc.hover else {
        return;
    };
    let Some(seg) = doc.hits.rect_for(v.id()) else {
        return;
    };
    let msg = tr(v.tip_key());
    let pad = Spacing::Sm.px();
    let w = text.prefix_width(msg, TypeToken::Sm.px()) + pad * 2.0;
    let r = Rect::new(seg.x, seg.y + seg.h + pad, w, m::FIELD_H);
    paint_tooltip(&Tooltip::new(v.id(), msg), r, scene, text, theme);
}
