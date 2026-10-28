//! ⭐ **Os botões do Butler** — premir um botão de cartão (no verso) ou de quadro (no cabeçalho)
//! corre o executor puro (`press_card_button`/`press_board_button`) e aplica o plano em nome do
//! BUTLER (a atividade diz «Butler»), um passo de desfazer em cada Kanban tocado. E a fiada de
//! botões de quadro que o cabeçalho põe ao lado do «…» ([`paint_board_buttons`]).

use crate::docs::KanbanDocs;
use crate::metrics as m;
use crate::paint::{c, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::documents::undo_tick;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_auto::{Kind, Outcome, Spec, press_board_button, press_card_button, specs};
use ph2d_kanban_model::{Actor, AutomationId, CardId, Kanban, Member};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// Os botões ligados do tipo de `want` (cartão ou quadro), pela ordem: o id e o nome.
pub(crate) fn enabled(k: &Kanban, want: fn(&Kind) -> bool) -> Vec<(AutomationId, Spec)> {
    specs(k)
        .into_iter()
        .filter(|(_, s)| s.enabled && want(&s.kind))
        .collect()
}

pub(crate) fn is_card_button(k: &Kind) -> bool {
    matches!(k, Kind::CardButton)
}

#[allow(dead_code)] // o cabeçalho (J1) chama [`paint_board_buttons`]
pub(crate) fn is_board_button(k: &Kind) -> bool {
    matches!(k, Kind::BoardButton { .. })
}

/// Aplica o que o Butler fez (com a hora de agora, o autor Butler). `true` = escreveu.
fn apply(docs: &mut KanbanDocs, out: Outcome) -> bool {
    if out.plan.is_empty() {
        return false;
    }
    let actor = Actor {
        by: Member::butler(),
        at: crate::actions::now(),
    };
    docs.set
        .borrow_mut()
        .apply_plan(out.plan, &mut docs.histories, undo_tick(), &actor);
    true
}

/// ⭐ Premir o botão de cartão `a` no cartão `card` da vista `id`.
pub(crate) fn press_card(docs: &mut KanbanDocs, id: u64, a: AutomationId, card: CardId) -> bool {
    let Some(kid) = docs.kanban_of(id) else {
        return false;
    };
    let zone = crate::back::dates::zone();
    let out = press_card_button(
        &docs.set.borrow(),
        kid,
        a,
        card,
        &docs.me,
        crate::actions::now(),
        &zone,
    );
    apply(docs, out)
}

/// ⭐ Premir o botão de quadro `a` na vista `id` (o cabeçalho, ou «Run» no painel).
pub(crate) fn press_board(docs: &mut KanbanDocs, id: u64, a: AutomationId) -> bool {
    let Some(kid) = docs.kanban_of(id) else {
        return false;
    };
    let zone = crate::back::dates::zone();
    let out = press_board_button(&docs.set.borrow(), kid, a, &docs.me, crate::actions::now(), &zone);
    apply(docs, out)
}

/// ⭐ A fiada dos botões de quadro para o CABEÇALHO: pinta-os da direita para a esquerda a acabar em
/// `right` (o cabeçalho põe-na à esquerda do «…»), na faixa `y..y+h`; devolve o rect de cada um
/// (para o clique: [`press_board`]) e onde a fiada começa.
#[allow(clippy::too_many_arguments, dead_code)] // o cabeçalho (J1) liga-o
pub(crate) fn paint_board_buttons(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    right: f32,
    y: f32,
    h: f32,
    hover: Option<AutomationId>,
) -> (Vec<(Rect, AutomationId)>, f32) {
    let fg = c(theme, T::KanbanHeaderFg);
    let mut x = right;
    let mut out = Vec::new();
    for (a, s) in enabled(k, is_board_button).into_iter().rev() {
        let w = ts.prefix_width(&s.name, m::BUTTON_TEXT_PX).ceil()
            + m::ICON_PX
            + m::HEADER_GAP
            + 2.0 * m::BUTTON_PAD_X;
        x -= w;
        let r = Rect::new(x, y, w, h);
        let fill = if hover == Some(a) {
            T::KanbanHeaderBtnOn
        } else {
            T::KanbanHeaderBtn
        };
        fill_rounded_rect(scene, r, m::BUTTON_RADIUS, c(theme, fill));
        let ir = Rect::new(r.x + m::BUTTON_PAD_X, r.y + (h - m::ICON_PX) / 2.0, m::ICON_PX, m::ICON_PX);
        paint_icon(scene, IconId::Play, ir, fg, StrokeToken::Default.px());
        let tx = ir.x + m::ICON_PX + m::HEADER_GAP;
        let ty = text_top(ts, r.y, h, m::BUTTON_TEXT_PX);
        paint_text_elided(ts, scene, &s.name, tx, ty, m::BUTTON_TEXT_PX, r.x + w - tx, fg);
        out.push((r, a));
        x -= m::HEADER_GAP;
    }
    out.reverse();
    (out, x)
}
