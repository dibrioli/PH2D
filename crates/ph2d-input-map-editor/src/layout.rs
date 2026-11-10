//! A disposição da aba: o seletor compacto à esquerda no topo e o corpo da vista por baixo.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;

use crate::doc::View;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    pub area: Rect,
    /// O segmentado List · Keyboard · Try it, com a largura natural das três palavras.
    pub selector: Rect,
    /// O cartão da vista.
    pub body: Rect,
}

#[must_use]
pub fn lay_out(area: Rect, text: &mut TextSystem) -> Layout {
    let gap = m::CARD_GAP;
    let segs: f32 = View::ALL
        .iter()
        .map(|v| text.prefix_width(ph2d_i18n::tr(v.label_key()), m::SEG_TEXT_PX) + m::SEG_INSET * 2.0)
        .sum();
    let n = View::ALL.len() as f32;
    let w = (segs + m::SEG_PAD * (n + 1.0)).min(area.w - gap * 2.0);
    let selector = Rect::new(area.x + gap, area.y + gap, w.max(0.0), m::FIELD_H);
    let top = selector.y + selector.h + gap;
    let body = Rect::new(
        area.x + gap,
        top,
        (area.w - gap * 2.0).max(0.0),
        (area.y + area.h - gap - top).max(0.0),
    );
    Layout {
        area,
        selector,
        body,
    }
}
