//! ⭐ **Acabamento — o que o Ctrl+clique alcança** (item 4 de `docs/CodeEditor/03_onda2.md`; oráculo
//! em `docs/CodeEditor/ferramentas/oraculo_acabamento/`). Com o Ctrl (ou ⌘) carregado, o nome sob o
//! rato que o Ctrl+clique leva a algum sítio sublinha e o ponteiro vira a mão.
//!
//! A validade é a do [`crate::goto`], pelas MESMAS portas (`goto_word`, `destination`): um sítio
//! neste script · `self.<propriedade>` · a documentação — e esta só existe se o balão a é
//! (`documents`: o luau-lsp dá um tipo a quase tudo, até a `if` e a `1`). Pergunta-se uma vez por
//! nome e revisão do texto.

use crate::doc::CodeDoc;
use ph2d_code_intel::{Answer, Ticket};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_panel_code::Hit;
use std::ops::Range;

/// Onde está a pergunta sobre um nome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// À espera de onde ele está definido.
    Definition,
    /// Nenhum sítio no script: à espera do balão (a documentação).
    Documentation,
    Reaches,
    Nowhere,
}

#[derive(Debug)]
pub(crate) struct Asked {
    pub word: Range<usize>,
    pub revision: u64,
    pub ticket: Option<Ticket>,
    pub verdict: Verdict,
}

impl Asked {
    /// Falta mandar o pedido do passo em que está.
    pub fn unsent(&self) -> bool {
        self.ticket.is_none() && matches!(self.verdict, Verdict::Definition | Verdict::Documentation)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Reach {
    /// Ctrl (ou ⌘) carregado no último quadro.
    pub held: bool,
    /// O rato sobre a aba; `None` = fora dela.
    pub mouse: Option<(f32, f32)>,
    /// O último nome perguntado: fica, e voltar a ele (ou ao Ctrl) não pergunta outra vez.
    pub asked: Option<Asked>,
}

impl CodeDoc {
    /// O nome de código desenhado sob `(x, y)`: o que o Ctrl+clique ali perguntaria.
    fn word_under(&self, x: f32, y: f32) -> Option<Range<usize>> {
        let Hit::Text(b) = self.view.hit(&self.buffer, x, y) else {
            return None;
        };
        self.goto_word(b).filter(|w| self.drawn_over(w, x, y))
    }

    /// Um quadro com o Ctrl `held`: o nome sob o rato pergunta-se ao assistente (uma vez por nome e
    /// revisão; a pergunta segue o caminho do Ctrl+clique, [`crate::intel`]).
    pub(crate) fn reach_tick(&mut self, held: bool) {
        self.reach.held = held;
        let Some(word) = held
            .then_some(self.reach.mouse)
            .flatten()
            .and_then(|(x, y)| self.word_under(x, y))
        else {
            return;
        };
        let revision = self.buffer.revision();
        let same = |a: &Asked| a.word == word && a.revision == revision;
        if !self.reach.asked.as_ref().is_some_and(same) {
            self.reach.asked = Some(Asked {
                word,
                revision,
                ticket: None,
                verdict: Verdict::Definition,
            });
        }
    }

    /// O nome sublinhado agora: o Ctrl carregado, o rato sobre ele, e o assistente disse que o
    /// Ctrl+clique o leva a algum sítio (nesta revisão).
    pub(crate) fn reach_link(&self) -> Option<Range<usize>> {
        let (x, y) = self.reach.mouse.filter(|_| self.reach.held)?;
        let a = self.reach.asked.as_ref().filter(|a| {
            a.verdict == Verdict::Reaches && a.revision == self.buffer.revision()
        })?;
        (self.word_under(x, y)? == a.word).then(|| a.word.clone())
    }

    pub(crate) fn reach_owns(&self, ticket: Ticket) -> bool {
        self.reach
            .asked
            .as_ref()
            .is_some_and(|a| a.ticket == Some(ticket))
    }

    /// A resposta à pergunta em curso: a definição decide pela regra do Ctrl+clique; sem sítio no
    /// script, pergunta-se o balão; qualquer outra resposta (recusa, servidor que caiu) = nada.
    pub(crate) fn reach_answer(&mut self, uri: &str, answer: Answer) {
        let Some(a) = self.reach.asked.take() else {
            return;
        };
        let verdict = match (a.verdict, answer) {
            (Verdict::Definition, Answer::Definition(targets)) => {
                if self.destination(uri, &a.word, &targets).is_some() {
                    Verdict::Reaches
                } else {
                    Verdict::Documentation
                }
            }
            (Verdict::Documentation, Answer::Hover(Some(h))) if Self::documents(&h.text) => {
                Verdict::Reaches
            }
            _ => Verdict::Nowhere,
        };
        self.reach.asked = Some(Asked {
            ticket: None,
            verdict,
            ..a
        });
    }

    /// Sem assistente que responda: nenhum Ctrl+clique leva a lado nenhum.
    pub(crate) fn reach_offline(&mut self) {
        if let Some(a) = self.reach.asked.as_mut().filter(|a| a.unsent()) {
            a.verdict = Verdict::Nowhere;
        }
    }

    /// O rato mexeu-se para `(x, y)`: por cima de algo que tapa o texto, ele não aponta nenhum nome.
    pub(crate) fn reach_mouse(&mut self, x: f32, y: f32) {
        self.reach.mouse = self.overlay_look(x, y).is_none().then_some((x, y));
    }

    /// O ponteiro sobre o que tapa o texto, a mesma ordem do clique (`press`): a caixinha do F2, o
    /// balão, a lista, a consola e a barra de busca são seta, menos os campos de texto; `None` = nada.
    fn overlay_look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if (self.rename.is_some() && self.view.rename_contains(x, y))
            || self.view.hint_hover_contains(x, y)
            || self.view.completion_hit(x, y).is_some()
            || self.console_view.hit(x, y).is_some()
        {
            return Some(PointerLook::Arrow);
        }
        let field = self.find_view.over(x, y).filter(|_| self.find.open)?;
        Some(if field {
            PointerLook::Text
        } else {
            PointerLook::Arrow
        })
    }

    /// O ponteiro em `(x, y)` da aba: o que tapa o texto ([`Self::overlay_look`]); o texto é o I; o
    /// nome sublinhado ([`Self::reach_link`]) é a mão.
    pub(crate) fn pointer_look(&self, x: f32, y: f32) -> PointerLook {
        if let Some(look) = self.overlay_look(x, y) {
            return look;
        }
        match self.view.hit(&self.buffer, x, y) {
            Hit::Text(_) if self.reach_link().is_some_and(|w| self.word_under(x, y) == Some(w)) => {
                PointerLook::Hand
            }
            Hit::Text(_) => PointerLook::Text,
            _ => PointerLook::Arrow,
        }
    }
}
