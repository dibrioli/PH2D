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

/// Quanto um balão À VISTA espera o rato que saiu dele antes de fechar: o caminho do ponto da calha
/// (ou da palavra) até aos botões atravessa o que não é nem um nem o outro (smoke do dono, 18/10). É
/// o `editor.hover.hidingDelay` do VS Code 1.142 (300 ms, o padrão gravado no produto).
pub(crate) const HOVER_HIDE_NS: u128 = 300_000_000;

/// Para onde o rato foi depois de sair do balão à vista, e quando isso vale.
type Target = Option<(Range<usize>, Option<Note>)>;

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
    /// O balão de um problema (I8, `crate::note`), além da palavra.
    pub note: Option<Note>,
    /// A pausa passou (o balão de um problema abre sem esperar o assistente).
    pub ripe: bool,
    /// As correcções do problema (I8b): as do Alt+Enter naquele sítio, um botão cada.
    pub fixes: Option<crate::actions::Actions>,
    /// O botão sob o rato.
    pub hot: Option<usize>,
    /// Quanto do realce de cada botão está presente (o eixo do hover do app, `crate::note`).
    pub live: Vec<f32>,
    /// O rato saiu do balão à vista: `(quando vale, o alvo novo, quando o rato lá parou)` — o alvo
    /// vale passado [`HOVER_HIDE_NS`], e a pausa dele conta desde que o rato lá parou.
    pub leaving: Option<(u128, Target, u128)>,
}

/// O que o balão mostra de um problema (I8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Note {
    /// O rato no ponto da calha da linha: as explicações dos problemas dela. Não pergunta ao assistente.
    Gutter(usize),
    /// O rato sobre o culpado sublinhado (a palavra do balão): a mensagem crua.
    Culprit,
}

impl Hovering {
    /// Pergunta ao assistente (a calha não tem palavra).
    pub fn asks(&self) -> bool {
        !matches!(self.note, Some(Note::Gutter(_)))
    }
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
    /// O rato mexeu-se sobre `word` (`None` = sobre nada que tenha balão), talvez um problema
    /// (`note`), no instante `now_ns`. `open` = o balão está à vista: um alvo novo espera
    /// [`HOVER_HIDE_NS`] ([`Self::settle`]), e voltar a ele antes disso não muda nada.
    pub fn pointer(&mut self, at: Target, now_ns: u128, open: bool) {
        if let Some(h) = self.hover.as_mut() {
            let same = at
                .as_ref()
                .is_some_and(|(w, n)| *w == h.word && *n == h.note);
            if same {
                h.leaving = None;
            } else if open {
                let due = h.leaving.as_ref().map_or(now_ns + HOVER_HIDE_NS, |l| l.0);
                h.leaving = Some((due, at, now_ns));
                return;
            }
        }
        let Some((word, note)) = at else {
            self.hover = None;
            return;
        };
        match &mut self.hover {
            Some(h) if h.word == word && h.note == note => {
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
                    note,
                    ripe: false,
                    fixes: None,
                    hot: None,
                    live: Vec::new(),
                    leaving: None,
                });
            }
        }
    }

    /// O rato voltou ao balão à vista: fica.
    pub fn stay(&mut self) {
        if let Some(h) = self.hover.as_mut() {
            h.leaving = None;
        }
    }

    /// Um quadro no instante `now_ns`: o rato que saiu do balão há [`HOVER_HIDE_NS`] leva-o para o
    /// alvo novo (cuja pausa conta desde que o rato lá parou) ou fecha-o.
    pub fn settle(&mut self, now_ns: u128) {
        let due = self
            .hover
            .as_ref()
            .and_then(|h| h.leaving.as_ref())
            .is_some_and(|l| l.0 <= now_ns);
        if due && let Some((_, to, since)) = self.hover.as_mut().and_then(|h| h.leaving.take()) {
            self.pointer(to, since, false);
        }
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
        self.hover.as_ref().is_some_and(|h| {
            h.ticket == Some(ticket) || h.fixes.as_ref().is_some_and(|f| f.owns(ticket))
        }) || self.sig.as_ref().is_some_and(|s| s.ticket == Some(ticket))
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

    /// Sem assistente: fecha o que é dele (o balão de um problema fica, sem a palavra).
    pub fn close(&mut self) {
        match self.hover.as_mut() {
            Some(h) if h.note.is_some() => {
                h.shown = None;
                if let Some(f) = h.fixes.as_mut() {
                    f.offline();
                }
            }
            _ => self.hover = None,
        }
        self.sig = None;
    }
}

impl CodeDoc {
    /// O rato sem botão em `(x, y)`: sobre o balão fica; sobre o ponto da calha de uma linha com
    /// problemas, sobre um culpado sublinhado ou sobre uma palavra de código (dentro do desenho dela,
    /// nunca no vazio depois do fim da linha) conta a pausa; no resto fecha-o.
    pub(crate) fn hover_at(&mut self, x: f32, y: f32, now_ns: u128) {
        if self.view.hint_hover_contains(x, y) {
            let hot = self.view.hint_button_at(x, y);
            self.hints.stay();
            if let Some(h) = self.hints.hover.as_mut() {
                h.hot = hot;
            }
            return;
        }
        let at = match self.view.hit(&self.buffer, x, y) {
            ph2d_panel_code::Hit::Gutter(line) => self
                .gutter_note(line)
                .map(|(anchor, note)| (anchor, Some(note))),
            ph2d_panel_code::Hit::Text(b) => match self.culprit_at(b, x, y) {
                Some(c) => Some((c, Some(Note::Culprit))),
                None => word_at(self.buffer.text(), b)
                    .filter(|w| self.drawn_over(w, x, y))
                    .filter(|w| {
                        self.hints.hover.as_ref().is_some_and(|h| &h.word == w)
                            || self.syntax.in_code(self.buffer.text(), w.start)
                    })
                    .map(|w| (w, None)),
            },
            _ => None,
        };
        let open = self.view.hints_painted().0.is_some();
        self.hints.pointer(at, now_ns, open);
        if let Some(h) = self.hints.hover.as_mut() {
            h.hot = None;
        }
    }

    /// O clique no botão `index` do balão: aplica a correcção (como o Enter na janelinha do
    /// Alt+Enter) e o balão fecha.
    pub(crate) fn apply_fix(&mut self, index: usize) {
        if let Some(a) = self.hints.hover.take().and_then(|h| h.fixes) {
            self.apply(&a, index);
        }
    }
}
