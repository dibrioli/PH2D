//! ⭐ **I6 — ir para a definição** (Ctrl+clique / F12; decisão 5 do dono, `docs/CodeEditor/03_onda2.md`).
//!
//! O que o `luau-lsp` 1.70.1 responde (I0b §5 e `I6_I11_bruto.json`): num local, num parâmetro e numa
//! `local function`, a declaração; em `self.t`, a escrita `self.t = …`; na API (`ph2d.set`, `print`,
//! `math.sin`) e em `self.<propriedade>`, `[]`. Então:
//! - há um sítio neste script ⇒ o cursor vai para lá;
//! - `self.<nome>` de uma `ph2d.property("<nome>", …)` ⇒ a declaração dela (NOSSA: o servidor não liga
//!   o literal ao campo do `self`, I0b §1);
//! - o resto ⇒ a documentação, no balão do I5, aberto já e preso ao nome (o rato não o leva).
//!
//! Uma edição, um clique ou uma tecla antes da resposta largam o pedido.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection, word_at};
use ph2d_code_intel::{Answer, Target, Ticket};

#[derive(Debug)]
pub(crate) struct Goto {
    /// O nome perguntado (bytes do buffer).
    pub word: std::ops::Range<usize>,
    pub ticket: Option<Ticket>,
}

impl CodeDoc {
    /// Ctrl+clique ou F12 em `at`: pergunta onde está definido o nome de CÓDIGO ali.
    pub(crate) fn go_to_definition(&mut self, at: usize) {
        let t = self.buffer.text();
        self.goto = word_at(t, at)
            .filter(|w| self.syntax.in_code(t, w.start))
            .map(|word| Goto { word, ticket: None });
    }

    /// O pedido `ticket` é desta aba (o balão, a assinatura, as cores, a definição).
    pub(crate) fn owns(&self, ticket: Ticket) -> bool {
        self.hints.owns(ticket)
            || self.sem.ticket == Some(ticket)
            || self.goto.as_ref().is_some_and(|g| g.ticket == Some(ticket))
    }

    /// A resposta `ticket` a um pedido sobre o documento `uri` (este).
    pub(crate) fn answer(&mut self, ticket: Ticket, uri: &str, answer: Answer) {
        if self.sem.ticket == Some(ticket) {
            self.sem.answer(ticket, answer);
        } else if let Some(g) = self.goto.take_if(|g| g.ticket == Some(ticket)) {
            match answer {
                Answer::Definition(targets) => self.arrive(uri, &g.word, &targets),
                // O servidor recusou ou foi-se: nada a mostrar.
                _ => {}
            }
        } else {
            self.hints.answer(ticket, answer);
        }
    }

    fn arrive(&mut self, uri: &str, word: &std::ops::Range<usize>, targets: &[Target]) {
        let here = targets.iter().find(|t| t.uri == uri).map(|t| t.bytes.start);
        match here.or_else(|| self.property_of(word)) {
            Some(b) => {
                self.buffer
                    .set_selection(Selection::single(Range::cursor(b)));
                self.buffer.seal_history();
                self.reveal();
            }
            None => self.hints.pin(word.clone()),
        }
    }

    /// `self.<nome>` de uma `ph2d.property("<nome>", …)` do topo: onde o nome está declarado.
    fn property_of(&self, word: &std::ops::Range<usize>) -> Option<usize> {
        let t = self.buffer.text();
        let before = word.start.checked_sub("self.".len())?;
        if t.floor_char(before) != before || t.slice(before..word.start) != "self." {
            return None;
        }
        let name = t.slice(word.clone());
        self.syntax
            .property_declarations(t)
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, r)| r.start)
    }
}
