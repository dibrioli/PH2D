//! ⭐ **I6 — ir para a definição** (Ctrl+clique / F12; decisão 5 do dono, `docs/CodeEditor/03_onda2.md`)
//! e, na API, **a página de ajuda** (`docs/CodeEditor/04_pagina_de_ajuda.md` §3.1).
//!
//! O que o `luau-lsp` 1.70.1 responde (I0b §5 e `I6_I11_bruto.json`): num local, num parâmetro e numa
//! `local function`, a declaração; em `self.t`, a escrita `self.t = …`; na API (`ph2d.set`, `print`,
//! `math.sin`) e em `self.<propriedade>`, `[]`. Então, por esta ordem:
//! - há um sítio neste script ⇒ o cursor vai para lá (um `local print` sombreia a global);
//! - `self.<nome>` de uma `ph2d.property("<nome>", …)` ⇒ a declaração dela (NOSSA: o servidor não liga
//!   o literal ao campo do `self`, I0b §1);
//! - a porta ÚNICA [`ph2d_code_help::resolve::resolve`] (com o balão do assistente para o sítio: só
//!   ele sabe o `m.cos` de um `local m = math`) ⇒ a página, aberta numa aba e rolada ao membro;
//! - senão nada. Palavras reservadas e números nem se perguntam.
//!
//! A definição e o balão perguntam-se JUNTOS; uma edição, um clique ou uma tecla antes das respostas
//! largam o pedido.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection, word_at};
use ph2d_code_help::Anchor;
use ph2d_code_help::resolve::{Query, resolve};
use ph2d_code_intel::{Answer, Target, Ticket};

#[derive(Debug, Default)]
pub(crate) struct Goto {
    /// O nome perguntado (bytes do buffer).
    pub word: std::ops::Range<usize>,
    /// Os pedidos da definição e do balão (mandados juntos).
    pub ticket: Option<Ticket>,
    pub hover_ticket: Option<Ticket>,
    /// As respostas: os sítios da definição; o balão (`Some(None)` = respondeu sem nada).
    pub targets: Option<Vec<Target>>,
    pub hover: Option<Option<String>>,
}

/// Para onde o Ctrl+clique num nome leva.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dest {
    /// Um byte deste script (a declaração, ou a `ph2d.property`).
    Here(usize),
    Help(Anchor),
    /// Um nome de objecto da cena (`ph2d.object("…")`, item 8): seleccioná-lo. O byte é o início do nome.
    Object(usize),
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

    /// Ctrl+clique ou F12 em `at`: o nome de CÓDIGO ali. Se a mão do Ctrl já sabe para onde ele
    /// leva (o mesmo nome, nesta revisão), vai já; senão pergunta a definição e o balão.
    pub(crate) fn go_to_definition(&mut self, at: usize) {
        if let Some(name) = self.object_word(at) {
            self.goto = None;
            self.arrive_at(Dest::Object(name.start));
            return;
        }
        let Some(word) = self.goto_word(at) else {
            self.goto = None;
            return;
        };
        if let Some(dest) = self.reach_dest(&word) {
            self.goto = None;
            self.arrive_at(dest);
            return;
        }
        self.goto = Some(Goto {
            word,
            ..Goto::default()
        });
    }

    /// A pergunta à porta única sobre o nome `word`: o qualificador e o `:` são o texto logo antes
    /// dele (`math.` · `s:` · um receptor literal `("x"):` ⇒ sem qualificador); `hover` = o balão.
    pub(crate) fn help_anchor(
        &self,
        word: &std::ops::Range<usize>,
        hover: Option<&str>,
    ) -> Option<Anchor> {
        let t = self.buffer.text();
        let name = t.slice(word.clone());
        let line_start = t.byte_of_line(t.line_of_byte(word.start));
        let before = t.slice(line_start..word.start);
        let sep = before.chars().last().filter(|c| matches!(c, '.' | ':'));
        let receiver = sep.map(|c| &before[..before.len() - c.len_utf8()]);
        let ident = receiver.map(|r| {
            let i = r
                .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map_or(0, |i| i + 1);
            &r[i..]
        });
        // `x().y` (um receptor que não é nome) não é a global `y`; `("x"):upper` pergunta pelo `:`.
        let qualifier = match (sep, ident) {
            (Some('.'), Some(q)) => Some(q),
            (Some(_), Some(q)) if !q.is_empty() => Some(q),
            _ => None,
        };
        resolve(&Query {
            word: &name,
            qualifier,
            colon: sep == Some(':'),
            hover,
        })
    }

    /// Vai: o cursor para o sítio neste script, ou o pedido da página (a família abre-a).
    pub(crate) fn arrive_at(&mut self, dest: Dest) {
        match dest {
            Dest::Here(b) => {
                self.buffer
                    .set_selection(Selection::single(Range::cursor(b)));
                self.buffer.seal_history();
                self.reveal();
            }
            Dest::Help(a) => self.help_request = Some(a),
            Dest::Object(b) => {
                let t = self.buffer.text();
                self.select_object = self
                    .syntax
                    .object_references(t)
                    .into_iter()
                    .find(|r| r.range.start == b)
                    .map(|r| r.name);
            }
        }
    }

    /// ⭐ O nome de objecto da CENA escrito em `at` (`ph2d.object`/`ph2d.find_by_name`, item 8): os
    /// bytes dentro das aspas, só se a cena tem um objecto com ele — o Ctrl+clique selecciona-o.
    pub(crate) fn object_word(&self, at: usize) -> Option<std::ops::Range<usize>> {
        use ph2d_code_syntax::ObjectCall;
        self.syntax
            .object_references(self.buffer.text())
            .into_iter()
            .find(|r| {
                r.range.start <= at
                    && at <= r.range.end
                    && r.call != ObjectCall::Spawn
                    && self.scene.objects.contains_key(&r.name)
            })
            .map(|r| r.range)
    }

    /// O pedido `ticket` é desta aba (o balão, a assinatura, as cores, a definição).
    pub(crate) fn owns(&self, ticket: Ticket) -> bool {
        self.hints.owns(ticket)
            || self.sem.ticket == Some(ticket)
            || self.inlay.ticket == Some(ticket)
            || self
                .goto
                .as_ref()
                .is_some_and(|g| g.ticket == Some(ticket) || g.hover_ticket == Some(ticket))
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
        } else if self
            .goto
            .as_ref()
            .is_some_and(|g| g.ticket == Some(ticket) || g.hover_ticket == Some(ticket))
        {
            self.goto_answer(ticket, uri, answer);
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

    /// Uma das duas respostas do Ctrl+clique/F12. Com as duas: a ordem do módulo. Outra resposta à
    /// definição (o servidor recusou ou foi-se) larga o pedido; ao balão, segue sem ele.
    fn goto_answer(&mut self, ticket: Ticket, uri: &str, answer: Answer) {
        let Some(g) = self.goto.as_mut() else { return };
        if g.ticket == Some(ticket) {
            match answer {
                Answer::Definition(t) => g.targets = Some(t),
                _ => {
                    self.goto = None;
                    return;
                }
            }
        } else {
            g.hover = Some(match answer {
                Answer::Hover(h) => h.map(|h| h.text),
                _ => None,
            });
        }
        let (Some(targets), Some(hover)) = (&g.targets, &g.hover) else {
            // Um sítio neste script não espera pelo balão.
            let here = g
                .targets
                .as_ref()
                .and_then(|t| t.iter().find(|t| t.uri == uri));
            if let Some(b) = here.map(|t| t.bytes.start) {
                self.goto = None;
                self.arrive_at(Dest::Here(b));
            }
            return;
        };
        let (targets, hover) = (targets.clone(), hover.clone());
        let word = g.word.clone();
        self.goto = None;
        let dest = self.dest_of(uri, &word, &targets, hover.as_deref());
        self.reach_learn(&word, dest);
        if let Some(d) = dest {
            self.arrive_at(d);
        }
    }

    /// A ordem do módulo sobre as duas respostas.
    pub(crate) fn dest_of(
        &self,
        uri: &str,
        word: &std::ops::Range<usize>,
        targets: &[Target],
        hover: Option<&str>,
    ) -> Option<Dest> {
        match self.destination(uri, word, targets) {
            Some(b) => Some(Dest::Here(b)),
            None => self.help_anchor(word, hover).map(Dest::Help),
        }
    }

    /// `self.<nome>` de uma `ph2d.property("<nome>", …)` do topo: onde o nome está declarado.
    pub(crate) fn property_of(&self, word: &std::ops::Range<usize>) -> Option<usize> {
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
