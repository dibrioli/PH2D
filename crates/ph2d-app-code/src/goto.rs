//! ⭐ **I6 — ir para a definição** (Ctrl+clique / F12; decisão 5 do dono, `docs/CodeEditor/03_onda2.md`).
//!
//! O que o `luau-lsp` 1.70.1 responde (I0b §5 e `I6_I11_bruto.json`): num local, num parâmetro e numa
//! `local function`, a declaração; em `self.t`, a escrita `self.t = …`; na API (`ph2d.set`, `print`,
//! `math.sin`) e em `self.<propriedade>`, `[]`. Então:
//! - há um sítio neste script ⇒ o cursor vai para lá;
//! - `self.<nome>` de uma `ph2d.property("<nome>", …)` ⇒ a declaração dela (NOSSA: o servidor não liga
//!   o literal ao campo do `self`, I0b §1);
//! - o resto ⇒ a documentação, no balão do I5, aberto já e preso ao nome (o rato não o leva) — só
//!   se o balão o é ([`CodeDoc::documents`]); palavras reservadas e números nem se perguntam.
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
    /// O NOME de código em `at` (nem palavra reservada nem número): o que o Ctrl+clique pergunta (e o
    /// Ctrl+rato, [`crate::reach`]).
    pub(crate) fn goto_word(&self, at: usize) -> Option<std::ops::Range<usize>> {
        let t = self.buffer.text();
        word_at(t, at).filter(|w| {
            self.syntax.in_code(t, w.start) && ph2d_code_syntax::is_identifier(&t.slice(w.clone()))
        })
    }

    /// Ctrl+clique ou F12 em `at`: pergunta onde está definido o nome de CÓDIGO ali.
    pub(crate) fn go_to_definition(&mut self, at: usize) {
        self.goto = self.goto_word(at).map(|word| Goto { word, ticket: None });
    }

    /// O pedido `ticket` é desta aba (o balão, a assinatura, as cores, a definição).
    pub(crate) fn owns(&self, ticket: Ticket) -> bool {
        self.hints.owns(ticket)
            || self.sem.ticket == Some(ticket)
            || self.inlay.ticket == Some(ticket)
            || self.goto.as_ref().is_some_and(|g| g.ticket == Some(ticket))
            || self.actions.as_ref().is_some_and(|a| a.owns(ticket))
            || self.occ.ticket == Some(ticket)
            || self.reach_owns(ticket)
            || self
                .rename
                .as_ref()
                .and_then(|r| r.sent.as_ref())
                .is_some_and(|s| s.1 == Some(ticket))
    }

    /// A resposta `ticket` a um pedido sobre o documento `uri` (este).
    pub(crate) fn answer(&mut self, ticket: Ticket, uri: &str, answer: Answer) {
        if self.sem.ticket == Some(ticket) {
            self.sem.answer(ticket, answer);
        } else if self.inlay.ticket == Some(ticket) {
            self.inlay.answer(ticket, answer);
        } else if self.occ.ticket == Some(ticket) {
            self.occurrences_answer(answer);
        } else if self.reach_owns(ticket) {
            self.reach_answer(uri, answer);
        } else if self
            .rename
            .as_ref()
            .and_then(|r| r.sent.as_ref())
            .is_some_and(|s| s.1 == Some(ticket))
        {
            self.rename_answer(answer);
        } else if self.actions.as_ref().is_some_and(|a| a.owns(ticket)) {
            self.actions_answer(ticket, answer);
        } else if let Some(f) = self
            .hints
            .hover
            .as_mut()
            .and_then(|h| h.fixes.as_mut())
            .filter(|f| f.owns(ticket))
        {
            f.answer(ticket, answer, self.buffer.text());
        } else if let Some(g) = self.goto.take_if(|g| g.ticket == Some(ticket)) {
            // Outra resposta = o servidor recusou ou foi-se: nada a mostrar.
            if let Answer::Definition(targets) = answer {
                self.arrive(uri, &g.word, &targets);
            }
        } else {
            self.hints.answer(ticket, answer);
        }
    }

    /// Para onde a resposta `targets` leva `word`: `Some(byte)` = um sítio neste script (`uri`) ou a
    /// declaração da propriedade; `None` = a documentação (o balão, se o assistente tem o que dizer).
    pub(crate) fn destination(
        &self,
        uri: &str,
        word: &std::ops::Range<usize>,
        targets: &[Target],
    ) -> Option<usize> {
        let here = targets.iter().find(|t| t.uri == uri).map(|t| t.bytes.start);
        here.or_else(|| self.property_of(word))
    }

    fn arrive(&mut self, uri: &str, word: &std::ops::Range<usize>, targets: &[Target]) {
        match self.destination(uri, word, targets) {
            Some(b) => {
                self.buffer
                    .set_selection(Selection::single(Range::cursor(b)));
                self.buffer.seal_history();
                self.reveal();
            }
            None => self.hints.pin(word.clone()),
        }
    }

    /// O balão `text` do assistente é a DOCUMENTAÇÃO de um nome — declara-o (`function math.sin(…)`,
    /// `type ph2d = {…}`, `local brilho: number`) — e não só o tipo de uma expressão (`number`, `T`)
    /// nem um nome que não existe (`*error-type*`). Sondado no luau-lsp 1.70.1:
    /// `docs/CodeEditor/ferramentas/medicoes_onda2/acabamento_bruto.json`.
    pub(crate) fn documents(text: &str) -> bool {
        let first = text.lines().find(|l| !l.starts_with("```")).unwrap_or("");
        !text.contains("*error-type*")
            && ["function ", "type ", "local "]
                .iter()
                .any(|p| first.starts_with(p))
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
