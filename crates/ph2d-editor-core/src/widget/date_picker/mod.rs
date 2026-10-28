//! [`DatePicker`] — o painel de datas do verso do cartão (o do Trello): o calendário, as caixas
//! «início» e «entrega» (cada uma com a sua marca de activar), a hora da entrega, o lembrete e
//! os botões guardar/remover.
//!
//! Como o [`super::Calendar`], não faz contas de datas nem formata texto: a grelha, o texto de
//! cada data e a leitura do que o artista escreveu são do chamador. Ele guarda o estado do
//! painel e devolve [`DatePickerEvent`]s; o chamador responde com [`DatePicker::set_date`].

mod field;
mod paint;

pub use field::{FieldKey, LineField};
pub use paint::{DatePickerHit, DatePickerLayout, paint_date_picker};

use super::calendar::{Calendar, CalendarDate, CalendarEvent};
use crate::doc_surface::KeyInput;
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};

/// Qual das duas datas.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum DateSlot {
    Start,
    #[default]
    Due,
}

/// As três caixas de texto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateField {
    Start,
    Due,
    Time,
}

/// O lembrete da entrega (as nove escolhas do Trello).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Reminder {
    #[default]
    None,
    AtTime,
    Min5,
    Min10,
    Min15,
    Hour1,
    Hour2,
    Day1,
    Day2,
}

impl Reminder {
    pub const ALL: [Self; 9] = [
        Self::None,
        Self::AtTime,
        Self::Min5,
        Self::Min10,
        Self::Min15,
        Self::Hour1,
        Self::Hour2,
        Self::Day1,
        Self::Day2,
    ];

    /// Minutos antes da entrega (`None` = sem lembrete).
    #[must_use]
    pub const fn minutes_before(self) -> Option<u32> {
        match self {
            Self::None => None,
            Self::AtTime => Some(0),
            Self::Min5 => Some(5),
            Self::Min10 => Some(10),
            Self::Min15 => Some(15),
            Self::Hour1 => Some(60),
            Self::Hour2 => Some(120),
            Self::Day1 => Some(1440),
            Self::Day2 => Some(2880),
        }
    }

    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|r| *r == self).unwrap_or_default()
    }
}

/// Todo o texto do painel, já traduzido pelo chamador.
#[derive(Clone, Debug, Default)]
pub struct DatePickerLabels {
    pub title: String,
    pub close: String,
    pub start: String,
    pub due: String,
    pub time: String,
    pub reminder: String,
    /// Pela ordem de [`Reminder::ALL`].
    pub reminder_options: [String; 9],
    pub save: String,
    pub remove: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatePickerEvent {
    /// Mudar de mês / voltar a hoje — o chamador entrega a grelha nova ao `calendar`.
    Month(CalendarEvent),
    /// Um dia escolhido para `slot`; o chamador responde com [`DatePicker::set_date`].
    Pick { slot: DateSlot, date: CalendarDate },
    Toggled { slot: DateSlot, on: bool },
    /// O artista escreveu numa caixa e confirmou (Enter ou saiu dela); o chamador lê o texto.
    TextCommitted { field: DateField, text: String },
    Reminder(Reminder),
    Save,
    Remove,
    Close,
}

#[derive(Clone, Debug)]
pub struct DatePicker {
    pub id: NodeId,
    pub calendar: Calendar,
    pub labels: DatePickerLabels,
    pub start_on: bool,
    pub due_on: bool,
    pub start: Option<CalendarDate>,
    pub due: Option<CalendarDate>,
    /// O texto de cada caixa, pela ordem de [`DateField`].
    pub start_text: LineField,
    pub due_text: LineField,
    pub time_text: LineField,
    /// A caixa em edição (o teclado vai para ela).
    pub editing: Option<DateField>,
    /// A data que um clique no calendário escolhe.
    pub active: DateSlot,
    pub reminder: Reminder,
    pub reminder_open: bool,
    pub hover: Option<DatePickerHit>,
}

impl DatePicker {
    pub fn new(id: NodeId, calendar: Calendar, labels: DatePickerLabels) -> Self {
        let mut p = Self {
            id,
            calendar,
            labels,
            start_on: false,
            due_on: true,
            start: None,
            due: None,
            start_text: LineField::default(),
            due_text: LineField::default(),
            time_text: LineField::default(),
            editing: None,
            active: DateSlot::Due,
            reminder: Reminder::None,
            reminder_open: false,
            hover: None,
        };
        p.sync();
        p
    }

    fn field_mut(&mut self, f: DateField) -> &mut LineField {
        match f {
            DateField::Start => &mut self.start_text,
            DateField::Due => &mut self.due_text,
            DateField::Time => &mut self.time_text,
        }
    }

    #[must_use]
    pub fn field(&self, f: DateField) -> &LineField {
        match f {
            DateField::Start => &self.start_text,
            DateField::Due => &self.due_text,
            DateField::Time => &self.time_text,
        }
    }

    /// A data de `slot` e o texto com que o chamador a escreve.
    pub fn set_date(&mut self, slot: DateSlot, date: Option<CalendarDate>, text: impl Into<String>) {
        match slot {
            DateSlot::Start => {
                self.start = date;
                self.start_text = LineField::new(text);
            }
            DateSlot::Due => {
                self.due = date;
                self.due_text = LineField::new(text);
            }
        }
        self.sync();
    }

    pub fn set_time(&mut self, text: impl Into<String>) {
        self.time_text = LineField::new(text);
    }

    /// O calendário mostra a data activa e o intervalo início → entrega.
    pub fn sync(&mut self) {
        let start = self.start.filter(|_| self.start_on);
        let due = self.due.filter(|_| self.due_on);
        self.calendar.selected = match self.active {
            DateSlot::Start => start,
            DateSlot::Due => due,
        };
        self.calendar.range = start.zip(due);
    }

    fn toggle(&mut self, slot: DateSlot) -> DatePickerEvent {
        let on = match slot {
            DateSlot::Start => {
                self.start_on = !self.start_on;
                self.start_on
            }
            DateSlot::Due => {
                self.due_on = !self.due_on;
                self.due_on
            }
        };
        if on {
            self.active = slot;
        } else if self.active == slot {
            self.active = if slot == DateSlot::Start {
                DateSlot::Due
            } else {
                DateSlot::Start
            };
        }
        self.sync();
        DatePickerEvent::Toggled { slot, on }
    }

    /// Sai da caixa em edição: a saída confirma (como no Trello).
    fn leave_field(&mut self) -> Option<DatePickerEvent> {
        let f = self.editing.take()?;
        Some(DatePickerEvent::TextCommitted {
            field: f,
            text: self.field(f).text.clone(),
        })
    }

    fn edit(&mut self, f: DateField) -> Option<DatePickerEvent> {
        let left = if self.editing == Some(f) {
            None
        } else {
            self.leave_field()
        };
        self.editing = Some(f);
        let t = self.field_mut(f);
        t.caret = t.text.len();
        match f {
            DateField::Start => {
                self.start_on = true;
                self.active = DateSlot::Start;
            }
            DateField::Due | DateField::Time => {
                self.due_on = true;
                self.active = DateSlot::Due;
            }
        }
        self.calendar.focused = false;
        self.sync();
        left
    }

    /// Um clique em `(x, y)` dentro de `rect`. Um clique que sai de uma caixa em edição confirma-a
    /// primeiro (esse evento vem antes, e o do clique perde-se só se for o mesmo sítio).
    pub fn pointer_down(&mut self, rect: Rect, x: f32, y: f32) -> Vec<DatePickerEvent> {
        let l = DatePickerLayout::of(rect);
        let mut out = Vec::new();
        if self.reminder_open {
            self.reminder_open = false;
            if let Some(i) = l.reminder_option_at(x, y) {
                self.reminder = Reminder::ALL[i];
                out.push(DatePickerEvent::Reminder(self.reminder));
            }
            return out;
        }
        let Some(hit) = l.hit(x, y) else {
            out.extend(self.leave_field());
            return out;
        };
        match hit {
            DatePickerHit::Field(f) => out.extend(self.edit(f)),
            other => {
                out.extend(self.leave_field());
                match other {
                    DatePickerHit::Calendar => {
                        self.calendar.focused = true;
                        match self.calendar.pointer_down(l.calendar, x, y) {
                            Some(CalendarEvent::Pick(d)) => out.push(self.pick(d)),
                            Some(ev) => out.push(DatePickerEvent::Month(ev)),
                            None => {}
                        }
                    }
                    DatePickerHit::Check(slot) => out.push(self.toggle(slot)),
                    DatePickerHit::Reminder => self.reminder_open = true,
                    DatePickerHit::Save => out.push(DatePickerEvent::Save),
                    DatePickerHit::Remove => out.push(DatePickerEvent::Remove),
                    DatePickerHit::Close => out.push(DatePickerEvent::Close),
                    DatePickerHit::Field(_) => {}
                }
            }
        }
        out
    }

    fn pick(&mut self, d: CalendarDate) -> DatePickerEvent {
        let slot = if self.active == DateSlot::Start && self.start_on {
            DateSlot::Start
        } else {
            self.due_on = true;
            DateSlot::Due
        };
        match slot {
            DateSlot::Start => self.start = Some(d),
            DateSlot::Due => self.due = Some(d),
        }
        self.sync();
        DatePickerEvent::Pick { slot, date: d }
    }

    pub fn pointer_move(&mut self, rect: Rect, x: f32, y: f32) -> bool {
        let l = DatePickerLayout::of(rect);
        let h = l.hit(x, y);
        let cal = self.calendar.pointer_move(l.calendar, x, y);
        let changed = h != self.hover;
        self.hover = h;
        changed || cal
    }

    /// Uma tecla. `(tomou a tecla, evento)`.
    pub fn key(&mut self, input: &KeyInput<'_>) -> (bool, Option<DatePickerEvent>) {
        use crate::doc_surface::Key;
        if !input.pressed {
            return (self.editing.is_some(), None);
        }
        if let Some(f) = self.editing {
            let text = input.paste.as_deref().or(input.text);
            return match self.field_mut(f).key(input.key, text) {
                FieldKey::Commit => (true, self.leave_field()),
                FieldKey::Cancel => {
                    self.editing = None;
                    (true, None)
                }
                FieldKey::Edited | FieldKey::Moved => (true, None),
                FieldKey::Ignored => (false, None),
            };
        }
        if self.reminder_open && input.key == Key::Escape {
            self.reminder_open = false;
            return (true, None);
        }
        if input.key == Key::Escape {
            return (true, Some(DatePickerEvent::Close));
        }
        self.calendar.focused = true;
        match self.calendar.key(input.key, input.mods) {
            (taken, Some(CalendarEvent::Pick(d))) => (taken, Some(self.pick(d))),
            (taken, Some(ev)) => (taken, Some(DatePickerEvent::Month(ev))),
            (taken, None) => (taken, None),
        }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Dialog)
            .label(&self.labels.title)
            .bounds(x, y, w, h)
            .build()
    }
}
