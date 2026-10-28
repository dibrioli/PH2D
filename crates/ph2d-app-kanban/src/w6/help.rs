//! **`?` — os atalhos** (a janela de ajuda do Trello): todos os do plano §1.7, numa folha ao meio.

use super::Act;
use super::sheet::{self, Item, Sheet};
use crate::docs::KanbanDocs;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

/// Os atalhos, pela ordem do §1.7: `kanban.w6.key.<nome>` diz o que faz, `….keys` as teclas.
pub(crate) const KEYS: [&str; 26] = [
    "walk", "open", "close", "new", "quick", "title", "dates", "labels", "label_n", "members",
    "me", "archive", "move", "move_end", "watch", "vote", "filter", "mine", "clear", "menu",
    "collapse", "search", "complete", "copy", "save", "undo",
];

pub(crate) fn sheet() -> Sheet {
    let mut s = Sheet::new(sheet::WIDE);
    s.title("kanban.w6.help.title");
    s.push(
        Item::Note(tr("kanban.w6.help.note").to_owned()),
        Act::Inside,
    );
    for name in KEYS {
        let what = format!("kanban.w6.key.{name}");
        let keys = format!("{what}.keys");
        s.push(
            Item::Keys {
                keys: tr(&keys).to_owned(),
                what: tr(&what).to_owned(),
            },
            Act::Inside,
        );
    }
    s
}

/// A folha nasce ao meio da área, no topo.
pub(crate) fn anchor(area: Rect) -> Rect {
    let x = area.x + (area.w - sheet::WIDE).max(0.0) / 2.0;
    Rect::new(x, area.y + sheet::PAD, sheet::WIDE, 0.0)
}

pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if !input.pressed {
        return super::taken();
    }
    let close = input.key == Key::Escape || input.text == Some("?");
    if close {
        super::close_top(docs, id);
        return super::taken();
    }
    KeyOutcome::default()
}
