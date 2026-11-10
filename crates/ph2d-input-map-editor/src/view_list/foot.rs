//! O pé da lista: UMA frase (a da vez, `rows::foot`), o destaque em SemiBold e o resto na MESMA linha, a seguir a
//! um espaço; o que não cabe parte pelas palavras nas linhas de baixo (a aba estreita, 1280×720).

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::{footer::Phrase, text};
use crate::doc::InputMapDoc;

/// A frase arrumada: as linhas do resto (a 1.ª ao lado do destaque; vazia = o resto começa por baixo).
pub(super) struct Laid {
    pub phrase: Phrase,
    pub lines: Vec<String>,
}

/// O passo entre as linhas partidas: o corpo e o respiro de um rótulo, mais junto que um campo por linha.
const LINE_STEP: f32 = m::VALUE_PX + m::LABEL_TOP;

impl Laid {
    pub fn h(&self) -> f32 {
        m::FIELD_H + LINE_STEP * self.lines.len().saturating_sub(1) as f32
    }
}

fn space(ts: &mut TextSystem) -> f32 {
    ts.prefix_width(" ", m::VALUE_PX)
}

/// Parte a frase para a largura `w` do pé.
pub(super) fn lay(ts: &mut TextSystem, phrase: Phrase, w: f32) -> Laid {
    let room = (w - m::SEG_INSET * 2.0).max(0.0);
    let first = room - text::title_width(ts, &phrase.lead, m::VALUE_PX) - space(ts);
    let lines = text::wrap_after(ts, &phrase.rest, m::VALUE_PX, first, room);
    Laid { phrase, lines }
}

/// Pinta a frase em `r` (a largura do pé; a altura é a de [`Laid::h`]).
pub(super) fn paint(
    doc: &mut InputMapDoc,
    laid: &Laid,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let p = &laid.phrase;
    let bg = Rect::new(r.x, r.y, r.w, laid.h());
    let (lead_c, rest_c) = if p.warn {
        fill_rounded_rect(
            scene,
            bg,
            m::FIELD_RADIUS,
            ink.color(ColorToken::InputMapWarningBg),
        );
        let stripe = Rect::new(bg.x, bg.y, m::BAR_TRACK, bg.h);
        fill_rounded_rect(
            scene,
            stripe,
            m::BAR_TRACK * 0.5,
            ink.color(ColorToken::InputMapWarning),
        );
        (ColorToken::InputMapWarning, ColorToken::Text1)
    } else {
        fill_rounded_rect(scene, bg, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
        (ColorToken::Text1, ColorToken::Text2)
    };
    let inner = Rect::new(
        r.x + m::SEG_INSET,
        r.y,
        r.w - m::SEG_INSET * 2.0,
        m::FIELD_H,
    );
    let lead_w = text::title(
        &mut doc.census,
        ts,
        scene,
        &p.lead,
        inner,
        m::VALUE_PX,
        ink.color(lead_c),
    );
    let after = inner.x + lead_w + space(ts);
    for (i, s) in laid.lines.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        let x = if i == 0 { after } else { inner.x };
        let y = inner.y + LINE_STEP * i as f32;
        let w = (inner.x + inner.w - x).max(0.0);
        text::line(
            &mut doc.census,
            ts,
            scene,
            s,
            Rect::new(x, y, w, m::FIELD_H),
            m::VALUE_PX,
            ink.color(rest_c),
        );
    }
    doc.census.foot.push(bg);
}
