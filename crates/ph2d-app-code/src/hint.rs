//! ⭐ **I5 — o balão do rato e a ajuda de assinatura** (`docs/CodeEditor/03_onda2.md`).
//!
//! - **Balão**: o rato PARADO [`HOVER_PAUSE_NS`] sobre uma palavra de código pergunta ao assistente.
//!   Cada movimento recomeça a pausa (o `symbol_tooltip` do `CodeEdit` do Godot), por isso o rato a
//!   andar nunca pergunta. Sai quando o rato deixa a palavra e o balão, com uma tecla, um clique, a
//!   roda ou uma edição.
//! - **Assinatura**: abre com `(` ou `,` escritos em código, ou ao aceitar uma função da lista (o
//!   `command` [`PARAMETER_HINTS`] do item). Aberta, segue o cursor: um texto ou um cursor novo
//!   pergunta outra vez, e quando o assistente diz que o cursor já não está numa chamada, fecha. Esc
//!   fecha-a.

use crate::doc::CodeDoc;
use ph2d_code_buffer::word_at;
use ph2d_code_intel::{Hover, Signature, Ticket};
use std::ops::Range;

/// A pausa do rato antes do balão: o `gui/timers/tooltip_delay_sec` do Godot 4.7.2 (0,5 s; o tooltip
/// de símbolo do `CodeEdit` mediu 510 ms, I0b §7).
pub(crate) const HOVER_PAUSE_NS: u128 = 500_000_000;

/// O comando com que o `luau-lsp` marca cada função da lista (sonda do I4, 16/10).
pub(crate) const PARAMETER_HINTS: &str = "editor.action.triggerParameterHints";

#[derive(Debug)]
pub(crate) struct Hovering {
    /// A palavra sob o rato (bytes do buffer).
    pub word: Range<usize>,
    pub due_ns: u128,
    /// Perguntado (uma vez por palavra).
    pub ticket: Option<Ticket>,
    pub shown: Option<Hover>,
    /// Aberto por um comando (a definição na API, I6): o rato não o leva.
    pub pinned: bool,
}

#[derive(Debug, Default)]
pub(crate) struct SigHelp {
    /// A (revisão, cursor) da última pergunta: outra ⇒ pergunta outra vez.
    pub asked: Option<(u64, usize)>,
    pub ticket: Option<Ticket>,
    pub shown: Option<Signature>,
}

/// O balão e a assinatura de uma aba.
#[derive(Debug, Default)]
pub(crate) struct Hints {
    pub hover: Option<Hovering>,
    pub sig: Option<SigHelp>,
}

impl Hints {
    /// O rato mexeu-se sobre `word` (`None` = sobre nada que tenha balão) no instante `now_ns`.
    pub fn pointer(&mut self, word: Option<Range<usize>>, now_ns: u128) {
        if self.hover.as_ref().is_some_and(|h| h.pinned) {
            return;
        }
        let Some(word) = word else {
            self.hover = None;
            return;
        };
        match &mut self.hover {
            Some(h) if h.word == word => {
                if h.ticket.is_none() {
                    h.due_ns = now_ns + HOVER_PAUSE_NS;
                }
            }
            _ => {
                self.hover = Some(Hovering {
                    word,
                    due_ns: now_ns + HOVER_PAUSE_NS,
                    ticket: None,
                    shown: None,
                    pinned: false,
                });
            }
        }
    }

    /// O balão de `word` já (sem a pausa do rato), preso a ela.
    pub fn pin(&mut self, word: Range<usize>) {
        self.hover = Some(Hovering {
            word,
            due_ns: 0,
            ticket: None,
            shown: None,
            pinned: true,
        });
    }

    /// Abre a assinatura (ou pergunta outra vez, se já está aberta).
    pub fn open_signature(&mut self) {
        self.sig.get_or_insert_with(SigHelp::default).asked = None;
    }

    /// Escreveu-se `ch` (`code` = fora de strings e comentários).
    pub fn typed(&mut self, ch: char, code: bool) {
        if code && matches!(ch, '(' | ',') {
            self.open_signature();
        }
    }

    /// O pedido `ticket` é desta aba.
    pub fn owns(&self, ticket: Ticket) -> bool {
        self.hover
            .as_ref()
            .is_some_and(|h| h.ticket == Some(ticket))
            || self.sig.as_ref().is_some_and(|s| s.ticket == Some(ticket))
    }

    /// A resposta `ticket` do assistente. Um balão vazio fica sem pintar (e não pergunta outra vez
    /// sobre a mesma palavra); uma chamada que acabou, ou um erro, fecha a assinatura.
    pub fn answer(&mut self, ticket: Ticket, answer: ph2d_code_intel::Answer) {
        use ph2d_code_intel::Answer;
        if let Some(h) = self.hover.as_mut().filter(|h| h.ticket == Some(ticket)) {
            if let Answer::Hover(a) = answer {
                h.shown = a;
            }
            return;
        }
        if let Some(s) = self.sig.as_mut().filter(|s| s.ticket == Some(ticket)) {
            match answer {
                Answer::Signature(Some(sig)) => {
                    s.shown = Some(sig);
                    s.ticket = None;
                }
                _ => self.sig = None,
            }
        }
    }

    pub fn close(&mut self) {
        self.hover = None;
        self.sig = None;
    }
}

impl CodeDoc {
    /// O rato sem botão em `(x, y)`: sobre o balão fica; sobre uma palavra de código (dentro do
    /// desenho dela, nunca no vazio depois do fim da linha) conta a pausa; no resto fecha-o.
    pub(crate) fn hover_at(&mut self, x: f32, y: f32, now_ns: u128) {
        if self.view.hint_hover_contains(x, y) {
            return;
        }
        let word = match self.view.hit(&self.buffer, x, y) {
            ph2d_panel_code::Hit::Text(b) => word_at(self.buffer.text(), b).filter(|w| {
                let (a, z) = (
                    self.view.caret_rect(&self.buffer, w.start),
                    self.view.caret_rect(&self.buffer, w.end),
                );
                a.zip(z)
                    .is_some_and(|(a, z)| a.x <= x && x <= z.x && a.y <= y && y < a.y + a.h)
            }),
            _ => None,
        };
        let word = word.filter(|w| {
            self.hints.hover.as_ref().is_some_and(|h| &h.word == w)
                || self.syntax.in_code(self.buffer.text(), w.start)
        });
        self.hints.pointer(word, now_ns);
    }
}
