//! O teclado que rola na horizontal (dono 10/11: «a partir do limite o scroll precisa aparecer»): abaixo do
//! [`super::keys_fit::PITCH_FLOOR`] o desenho fica no passo mínimo, recortado ao campo, e esta barra discreta por
//! baixo dele rola-o — arrasto do polegar, Shift+roda, os dois dedos do trackpad, e o foco que traz a tecla à vista.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

use super::state::KbHit;
use crate::doc::InputMapDoc;
use crate::interact::{Control, ControlRole};

/// A barra da última pintura: o recorte do desenho, a vista (a largura útil), a largura do desenho e o polegar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HBar {
    pub clip: Rect,
    pub view: Rect,
    pub total: f32,
    pub thumb: Rect,
}

impl HBar {
    /// O máximo da rolagem.
    #[must_use]
    pub fn max(&self) -> f32 {
        (self.total - self.view.w).max(0.0)
    }
}

/// O polegar fino (o `BAR_TRACK` dos outros), sem trilho, na faixa `band` por baixo das teclas; desvanece quente.
pub(super) fn paint_bar(
    doc: &mut InputMapDoc,
    (clip, view, band): (Rect, Rect, Rect),
    total: f32,
    scene: &mut VectorScene,
    ink: CardInk,
) {
    let total = total.max(view.w);
    let w = (view.w * view.w / total).max(m::FIELD_H).min(view.w);
    let max = (total - view.w).max(f32::EPSILON);
    let x = view.x + (view.w - w) * (doc.kb.hscroll / max).clamp(0.0, 1.0);
    let thumb = Rect::new(x, band.y, w, band.h);
    let pill = Rect::new(x, band.y + (band.h - m::BAR_TRACK) * 0.5, w, m::BAR_TRACK);
    fill_rounded_rect(
        scene,
        pill,
        m::BAR_TRACK * 0.5,
        ink.color(ColorToken::BorderEmph),
    );
    let fade = doc.thumb_fade(KbHit::HThumb.id());
    if fade > 0.0 {
        let hot = ink.color(ColorToken::Text2).multiply_alpha(fade);
        fill_rounded_rect(scene, pill, m::BAR_TRACK * 0.5, hot);
    }
    doc.census.scrollbars.push((thumb, view, band));
    doc.kb.hbar = Some(HBar {
        clip,
        view,
        total,
        thumb,
    });
    doc.kb.targets.push(KbHit::HThumb);
    let label = tr("input_map_keyboard.hscrollbar");
    doc.control(Control::new(
        KbHit::HThumb.id(),
        thumb,
        ControlRole::ScrollThumb,
        label,
    ));
}

/// O Down no polegar: guarda (x do Down, rolagem no Down).
pub(crate) fn grab(doc: &mut InputMapDoc, x: f32) {
    doc.kb.hdrag = Some((x, doc.kb.hscroll));
}

/// O polegar agarrado segue o ponteiro. `true` = rolou.
pub(crate) fn drag(doc: &mut InputMapDoc, x: f32) -> bool {
    let (Some((x0, s0)), Some(b)) = (doc.kb.hdrag, doc.kb.hbar) else {
        return false;
    };
    let span = (b.view.w - b.thumb.w).max(1.0);
    let s = (s0 + (x - x0) * b.max() / span).clamp(0.0, b.max());
    let moved = (s - doc.kb.hscroll).abs() > f32::EPSILON;
    doc.kb.hscroll = s;
    moved
}

/// A roda horizontal (Shift+roda, os dois dedos de lado): só quando o teclado rola. `true` = rolou.
pub(crate) fn wheel(doc: &mut InputMapDoc, d: f32) -> bool {
    let Some(b) = doc.kb.hbar else {
        return false;
    };
    let before = doc.kb.hscroll;
    doc.kb.hscroll = (doc.kb.hscroll - d).clamp(0.0, b.max());
    (doc.kb.hscroll - before).abs() > f32::EPSILON
}

/// O foco pousou numa tecla do desenho que rola: a rolagem traz a tecla `r` inteira para a vista.
pub(crate) fn reveal(doc: &mut InputMapDoc, id: ph2d_a11y::NodeId, r: Rect) {
    let Some(b) = doc.kb.hbar else {
        return;
    };
    if !doc.kb.hkeys.contains(&id) {
        return;
    }
    let (lo, hi) = (b.view.x, b.view.x + b.view.w);
    let d = if r.x < lo {
        r.x - lo
    } else {
        (r.x + r.w - hi).max(0.0)
    };
    doc.kb.hscroll = (doc.kb.hscroll + d).clamp(0.0, b.max());
}
