//! ⭐ **«This event · This and following events · All events»** — a pergunta do Google Calendar e
//! do Outlook quando se arrasta, edita ou apaga UMA ocorrência de um evento que se repete. A
//! resposta escolhe o lote ([`ph2d_kanban_model::change_occurrence`] /
//! [`ph2d_kanban_model::delete_occurrence`]); fechar sem responder não muda nada.

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided};
use ph2d_editor_core::widget::{Popover, paint_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{
    Event, EventId, KanbanId, KanbanSet, Moment, Plan, Scope, change_occurrence, delete_occurrence,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use crate::metrics as m;
use crate::ui::{c, text_top};

/// O que se quer fazer à ocorrência.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    /// O arrasto: a ocorrência passa a `[start, end)`.
    Move {
        start: Moment,
        end: Moment,
    },
    /// O popover: a ocorrência passa a ser `to`.
    Edit(Box<Event>),
    Delete,
}

/// Uma pergunta por responder: a série, a ocorrência (o início que a regra lhe dá) e a mudança.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub kanban: KanbanId,
    pub series: EventId,
    pub at: Moment,
    pub change: Change,
}

impl Ask {
    /// O lote da resposta `scope` (vazio = a série já não existe ou nada muda).
    #[must_use]
    pub fn plan(&self, set: &KanbanSet, scope: Scope) -> Plan {
        let mut plan = Plan::default();
        let Some(k) = set.get(self.kanban) else {
            return plan;
        };
        let Some(s) = k.events.get(&self.series).filter(|e| !e.meta.deleted) else {
            return plan;
        };
        let ops = match &self.change {
            Change::Move { start, end } => {
                let mut to = s.clone();
                (to.start, to.end) = (*start, *end);
                change_occurrence(k, s, self.at, &to, scope)
            }
            Change::Edit(to) => change_occurrence(k, s, self.at, to, scope),
            Change::Delete => delete_occurrence(k, s, self.at, scope),
        };
        for op in ops {
            plan.push(self.kanban, op);
        }
        plan
    }
}

/// Um sítio da pergunta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeHit {
    Pick(Scope),
    Cancel,
}

/// As três respostas, pela ordem do Google Calendar.
pub const SCOPES: [Scope; 3] = [Scope::This, Scope::Following, Scope::All];

fn scope_key(s: Scope) -> &'static str {
    match s {
        Scope::This => "planner.scope.this",
        Scope::Following => "planner.scope.following",
        Scope::All => "planner.scope.all",
    }
}

/// A pergunta aberta, junto de `anchor` (onde se largou ou o popover do evento).
#[derive(Clone, Debug)]
pub struct ScopePop {
    pub ask: Ask,
    pub anchor: (f32, f32),
    pub rect: Rect,
    pub hover: Option<ScopeHit>,
}

/// O que um gesto fez à pergunta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeDone {
    Stay,
    Cancel,
    Pick(Scope),
}

impl ScopePop {
    #[must_use]
    pub fn new(ask: Ask, anchor: (f32, f32)) -> Self {
        Self {
            ask,
            anchor,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            hover: None,
        }
    }

    /// A altura: o título, as três respostas e «Cancel».
    #[must_use]
    pub fn height() -> f32 {
        m::PAD + m::SECTION_H + 4.0 * (m::BUTTON_H + m::GAP) + m::PAD - m::GAP
    }

    /// Põe a pergunta junto da âncora, dentro de `area`.
    pub fn place(&mut self, area: Rect) {
        let h = Self::height();
        let x = (self.anchor.0 - m::POP_W / 2.0).clamp(
            area.x + m::PAD,
            (area.x + area.w - m::POP_W - m::PAD).max(area.x),
        );
        let y = self
            .anchor
            .1
            .clamp(area.y + m::TOP_H, (area.y + area.h - h).max(area.y));
        self.rect = Rect::new(x, y, m::POP_W, h);
    }

    /// Os sítios de clique.
    #[must_use]
    pub fn hits(&self) -> Vec<(ScopeHit, Rect)> {
        let r = self.rect;
        let x = r.x + m::PAD;
        let w = r.w - 2.0 * m::PAD;
        let mut y = r.y + m::PAD + m::SECTION_H;
        let mut v = Vec::new();
        for s in SCOPES {
            v.push((ScopeHit::Pick(s), Rect::new(x, y, w, m::BUTTON_H)));
            y += m::BUTTON_H + m::GAP;
        }
        v.push((ScopeHit::Cancel, Rect::new(x, y, w, m::BUTTON_H)));
        v
    }

    pub fn paint(&self, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
        paint_popover(&Popover::new(crate::POP_NODE), self.rect, scene, theme);
        let title = if self.ask.change == Change::Delete {
            "planner.scope.delete_title"
        } else {
            "planner.scope.edit_title"
        };
        let x = self.rect.x + m::PAD;
        let w = self.rect.w - 2.0 * m::PAD;
        let ty = text_top(ts, self.rect.y + m::PAD, m::SECTION_H, m::TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            tr(title),
            x,
            ty,
            m::TEXT_PX,
            w,
            c(theme, T::Text1),
        );
        for (h, r) in self.hits() {
            let (key, base) = match h {
                ScopeHit::Pick(s) => (scope_key(s), T::Bg2),
                ScopeHit::Cancel => ("planner.scope.cancel", T::Bg1),
            };
            let (bg, fg) = if self.hover == Some(h) {
                (T::Accent, T::AccentFg)
            } else {
                (base, T::Text1)
            };
            fill_rounded_rect(scene, r, m::RADIUS, c(theme, bg));
            let ty = text_top(ts, r.y, r.h, m::TEXT_PX);
            paint_text_elided(
                ts,
                scene,
                tr(key),
                r.x + m::BUTTON_PAD_X,
                ty,
                m::TEXT_PX,
                r.w - 2.0 * m::BUTTON_PAD_X,
                c(theme, fg),
            );
        }
    }

    /// Um clique: numa resposta escolhe-a; fora da pergunta (ou em «Cancel») desiste.
    #[must_use]
    pub fn click(&self, x: f32, y: f32) -> ScopeDone {
        match self
            .hits()
            .into_iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(h, _)| h)
        {
            Some(ScopeHit::Pick(s)) => ScopeDone::Pick(s),
            Some(ScopeHit::Cancel) => ScopeDone::Cancel,
            None if self.rect.contains(x, y) => ScopeDone::Stay,
            None => ScopeDone::Cancel,
        }
    }

    /// Enter = «This event» (a resposta por omissão do Google), Esc desiste.
    #[must_use]
    pub fn key(input: &KeyInput<'_>) -> ScopeDone {
        match (input.pressed, input.key) {
            (true, Key::Enter) => ScopeDone::Pick(Scope::This),
            (true, Key::Escape) => ScopeDone::Cancel,
            _ => ScopeDone::Stay,
        }
    }
}
