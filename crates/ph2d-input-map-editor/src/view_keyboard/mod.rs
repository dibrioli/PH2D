//! **B · Keyboard** (plano `docs/InputMap/01_plano.md` §3, maquete B, no molde da aba do Animated Sprite): à
//! esquerda as acções por quem as lê, em cima o teclado (ou o comando) com uma barra por acção em cada tecla, em
//! baixo a acção aberta com as ligações em ladrilhos, a escuta no lugar e — com manche — a barra do curso.

mod geometry;
mod input;
mod keys;
mod list;
mod pad;
pub mod state;
mod tiles;
mod travel;

pub use input::tip_of;
pub(crate) use input::{key, motion, press, raw_key};

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::readers::Reader;
use ph2d_input::{ActionId, Binding};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;

/// Os três grupos da lista, por quem lê a acção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    PlatformPlayer,
    TopDownPlayer,
    Yours,
}

/// O grupo de uma acção: lida pelos dois movers vai para o Platform Player (como a maquete).
#[must_use]
pub fn group_of(doc: &InputMapDoc, name: &str) -> Group {
    let rs = crate::view_list::footer::readers_of(&doc.snap, name);
    if rs.contains(&Reader::PlatformPlayer) {
        Group::PlatformPlayer
    } else if rs.contains(&Reader::TopDownPlayer) {
        Group::TopDownPlayer
    } else {
        Group::Yours
    }
}

/// A acção aberta: a escolhida se ainda existe, senão a primeira.
#[must_use]
pub fn open(doc: &InputMapDoc) -> Option<ActionId> {
    let map = &doc.snap.map;
    doc.kb
        .open
        .filter(|a| map.get(*a).is_some())
        .or_else(|| map.actions().first().map(|a| a.id))
}

/// A ligação `b` é a que o alvo desenhado `part` mostra? O gatilho é o mesmo como botão e como eixo.
pub(crate) fn same_input(part: Binding, b: Binding) -> bool {
    use ph2d_input::{GamepadAxis as A, GamepadButton as P};
    part == b
        || matches!(
            (part, b),
            (
                Binding::PadButton(P::LeftTrigger),
                Binding::PadAxis {
                    axis: A::LeftTrigger,
                    ..
                }
            ) | (
                Binding::PadButton(P::RightTrigger),
                Binding::PadAxis {
                    axis: A::RightTrigger,
                    ..
                }
            )
        )
}

/// As acções que usam `b`, pela ordem do mapa.
pub(crate) fn users(doc: &InputMapDoc, b: Binding) -> Vec<ActionId> {
    doc.snap
        .map
        .actions()
        .iter()
        .filter(|a| a.bindings.iter().any(|x| same_input(b, *x)))
        .map(|a| a.id)
        .collect()
}

/// Pinta a vista dentro do cartão `body` (o cartão já está pintado).
pub fn paint(
    doc: &mut InputMapDoc,
    body: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let ink = CardInk::of_list(theme);
    doc.kb.targets.clear();
    doc.kb.bar = None;
    if let Some(n) = doc.kb.select_name.clone()
        && let Some(id) = doc.snap.map.id(&n)
    {
        doc.kb.open = Some(id);
        doc.kb.select_name = None;
    }
    doc.kb.open = open(doc);
    let inner = Rect::new(
        body.x + m::CARD_PAD_X,
        body.y + m::CARD_PAD_TOP,
        (body.w - m::CARD_PAD_X * 2.0).max(0.0),
        (body.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM).max(0.0),
    );
    let list_w = list::width(doc, ts).min(inner.w * 0.5);
    let list_r = Rect::new(inner.x, inner.y, list_w, inner.h);
    fill_rounded_rect(
        scene,
        list_r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanList),
    );
    list::paint(doc, list_r, scene, ts, ink);
    let rx = list_r.x + list_r.w + m::CARD_GAP;
    let rw = (inner.x + inner.w - rx).max(0.0);
    let top_h = keys::top_height(doc, rw, inner.h, ts);
    let top = Rect::new(rx, inner.y, rw, top_h);
    fill_rounded_rect(
        scene,
        top,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanList),
    );
    keys::paint_top(doc, top, scene, ts, ink);
    let by = top.y + top.h + m::CARD_GAP;
    let bottom = Rect::new(rx, by, rw, (inner.y + inner.h - by).max(0.0));
    fill_rounded_rect(
        scene,
        bottom,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanList),
    );
    tiles::paint(doc, bottom, scene, ts, ink);
}

/// Regista um alvo da vista: no índice da pintura e na lista que devolve o `KbHit`.
pub(crate) fn target(doc: &mut InputMapDoc, h: state::KbHit, r: Rect) {
    doc.hits.register(h.id(), r);
    doc.kb.targets.push(h);
}
