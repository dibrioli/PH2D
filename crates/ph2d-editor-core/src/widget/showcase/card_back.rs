//! Os widgets do VERSO do cartão (Kanban W3): o [`Calendar`], o [`DatePicker`] e o
//! [`RichEditor`], com dados de amostra — outubro de 2026, o intervalo 20 → 30, e uma descrição
//! com todos os tipos de bloco e de marca. Pintados por baixo do cartão da secção «Card».

use super::*;
use crate::widget::{
    CALENDAR_CELLS, Calendar, CalendarCell, CalendarDate, CalendarLayout, DatePicker,
    DatePickerLabels, DatePickerLayout, DateSlot, Reminder, RichEditor, RichEditorLabels,
    paint_calendar, paint_date_picker, paint_rich_editor,
};
use ph2d_richdoc::{Block, BlockKind, Inline, Mark, RichDoc};
use std::cell::RefCell;

/// A grelha de outubro de 2026 com a semana a começar à segunda (1/10 é quinta), hoje = 28.
#[must_use]
pub fn demo_month() -> [CalendarCell; CALENDAR_CELLS] {
    let mut cells = [CalendarCell::default(); CALENDAR_CELLS];
    for (i, c) in cells.iter_mut().enumerate() {
        // `i - 2` é o dia de outubro (as três primeiras são 28–30 de setembro).
        let n = i as i32 - 2;
        *c = if n < 1 {
            CalendarCell {
                year: 2026,
                month: 9,
                day: (30 + n) as u8,
                ..Default::default()
            }
        } else if n > 31 {
            CalendarCell {
                year: 2026,
                month: 11,
                day: (n - 31) as u8,
                ..Default::default()
            }
        } else {
            CalendarCell {
                year: 2026,
                month: 10,
                day: n as u8,
                in_month: true,
                is_today: n == 28,
            }
        };
    }
    cells
}

#[must_use]
pub fn demo_calendar(id: NodeId) -> Calendar {
    let days = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"].map(String::from);
    let mut c = Calendar::new(id).labels(days, "Previous month", "Next month", "Today");
    c.set_month("October 2026", demo_month());
    c.selected = Some(CalendarDate::new(2026, 10, 14));
    c
}

#[must_use]
pub fn demo_date_picker(id: NodeId) -> DatePicker {
    let labels = DatePickerLabels {
        title: String::from("Dates"),
        close: String::from("Close"),
        start: String::from("Start date"),
        due: String::from("Due date"),
        time: String::from("Time"),
        reminder: String::from("Set due date reminder"),
        reminder_options: [
            "None",
            "At time of due date",
            "5 Minutes before",
            "10 Minutes before",
            "15 Minutes before",
            "1 Hour before",
            "2 Hours before",
            "1 Day before",
            "2 Days before",
        ]
        .map(String::from),
        save: String::from("Save"),
        remove: String::from("Remove"),
    };
    let mut p = DatePicker::new(id, demo_calendar(ph2d_a11y::NodeId(id.0.wrapping_add(64))), labels);
    p.start_on = true;
    p.set_date(DateSlot::Start, Some(CalendarDate::new(2026, 10, 20)), "10/20/2026");
    p.set_date(DateSlot::Due, Some(CalendarDate::new(2026, 10, 30)), "10/30/2026");
    p.set_time("10:00 AM");
    p.reminder = Reminder::Day1;
    p
}

#[must_use]
pub fn demo_doc() -> RichDoc {
    let mut rich = Inline::default();
    rich.push("Ship the ", &[]);
    rich.push("card back", &[Mark::Bold]);
    rich.push(" with ", &[]);
    rich.push("rich", &[Mark::Italic]);
    rich.push(" text, ", &[]);
    rich.push("inline code", &[Mark::Code]);
    rich.push(", a ", &[]);
    rich.push("link", &[Mark::Link(String::from("https://trello.com"))]);
    rich.push(" and ", &[]);
    rich.push("strike", &[Mark::Strike]);
    rich.push(".", &[]);
    RichDoc::from_blocks(vec![
        Block::new(BlockKind::Heading(1), Inline::plain("Card back")),
        Block::new(BlockKind::Paragraph, rich),
        Block::new(BlockKind::Bullet, Inline::plain("Calendar and dates")),
        Block::new(BlockKind::Bullet, Inline::plain("Nested item")).with_indent(1),
        Block::new(BlockKind::Numbered, Inline::plain("First step")),
        Block::new(BlockKind::Numbered, Inline::plain("Second step")),
        Block::new(BlockKind::Check(true), Inline::plain("Design approved")),
        Block::new(BlockKind::Check(false), Inline::plain("Write the tests")),
        Block::new(BlockKind::Quote, Inline::plain("Make it work like Trello.")),
        Block::new(
            BlockKind::Code(String::from("rust")),
            Inline::plain("let due = picker.due;"),
        ),
        Block::new(BlockKind::Rule, Inline::default()),
        Block::paragraph("The end."),
    ])
}

#[must_use]
pub fn demo_rich_editor(id: NodeId) -> RichEditor {
    let labels = RichEditorLabels {
        placeholder: String::from("Add a more detailed description…"),
        tools: [
            "Aa", "B", "I", "S", "</>", "•", "1.", "[x]", "\u{201c}", "{ }", "\u{2197}", "\u{2014}",
        ]
        .map(String::from),
        styles: ["Normal text", "Heading 1", "Heading 2", "Heading 3"].map(String::from),
    };
    let mut e = RichEditor::new(id, demo_doc(), labels);
    e.editing = true;
    e
}

thread_local! {
    /// O editor de amostra vive entre quadros (guarda a disposição moldada).
    static EDITOR: RefCell<Option<RichEditor>> = const { RefCell::new(None) };
}

/// Pinta os três widgets a partir de `y`; devolve o fundo.
pub(super) fn paint_card_back_widgets(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Lg.px();
    let mut y = y + gap;
    let cal = demo_calendar(ids::INSP_SECTION_CARD);
    let ch = CalendarLayout::preferred_height();
    paint_calendar(&cal, Rect::new(x, y, w, ch), scene, text_system, theme);
    y += ch + gap;

    let picker = demo_date_picker(ids::INSP_SECTION_CARD);
    let ph = DatePickerLayout::preferred_height();
    paint_date_picker(&picker, Rect::new(x, y, w, ph), scene, text_system, theme);
    y += ph + gap;

    EDITOR.with(|cell| {
        let mut cell = cell.borrow_mut();
        let ed = cell.get_or_insert_with(|| demo_rich_editor(ids::INSP_SECTION_CARD));
        let eh = ed.preferred_height(text_system, w);
        paint_rich_editor(ed, Rect::new(x, y, w, eh), scene, text_system, theme);
        y + eh
    })
}
