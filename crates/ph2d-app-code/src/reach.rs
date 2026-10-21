//! ⭐ **Acabamento — o que o Ctrl+clique alcança** (item 4 de `docs/CodeEditor/03_onda2.md`; oráculo
//! em `docs/CodeEditor/ferramentas/oraculo_acabamento/`). Com o Ctrl (ou ⌘) carregado, o nome sob o
//! rato que o Ctrl+clique leva a algum sítio sublinha e o ponteiro vira a mão.
//!
//! A validade é a do [`crate::goto`], pelas MESMAS portas (`goto_word`, `dest_of`): um sítio neste
//! script · `self.<propriedade>` · a página de ajuda que a porta única
//! ([`ph2d_code_help::resolve::resolve`]) dá (o luau-lsp dá um tipo a quase tudo, até a `if` e a
//! `1`: o tipo sozinho não é página). Pergunta-se uma vez por nome e revisão do texto; o
//! Ctrl+clique sobre um nome já respondido vai sem perguntar outra vez.

use crate::doc::CodeDoc;
use crate::goto::Dest;
use ph2d_code_intel::{Answer, Ticket};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_panel_code::Hit;
use std::ops::Range;

/// As cores `spans` (ordenadas, sem sobreposição) com o nome `link` na cor do link: os trechos que o
/// tocam partem-se à volta dele.
pub(crate) fn tint(
    spans: Vec<(Range<usize>, ph2d_panel_code::Ink)>,
    link: &Range<usize>,
) -> Vec<(Range<usize>, ph2d_panel_code::Ink)> {
    let mut out = Vec::with_capacity(spans.len() + 2);
    for (r, ink) in spans {
        if r.end <= link.start || r.start >= link.end {
            out.push((r, ink));
            continue;
        }
        if r.start < link.start {
            out.push((r.start..link.start, ink));
        }
        if r.end > link.end {
            out.push((link.end..r.end, ink));
        }
    }
    out.push((link.clone(), ph2d_panel_code::Ink::Link));
    out.sort_by_key(|(r, _)| r.start);
    out
}

/// Onde está a pergunta sobre um nome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// À espera de onde ele está definido.
    Definition,
    /// Nenhum sítio no script: à espera do balão (a página de ajuda).
    Documentation,
    Reaches(Dest),
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
        self.ticket.is_none()
            && matches!(self.verdict, Verdict::Definition | Verdict::Documentation)
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
        self.goto_word(b)
            .or_else(|| self.object_word(b))
            .filter(|w| self.drawn_over(w, x, y))
    }

    /// Um quadro com o Ctrl `held`: o nome sob o rato pergunta-se ao assistente (uma vez por nome e
    /// revisão; a pergunta segue o caminho do Ctrl+clique, [`crate::intel`]).
    ///
    /// Carregar no Ctrl fecha o balão do rato (o Godot 4.7.2 fá-lo; a mão e o sublinhado dizem o que o
    /// clique fará).
    pub(crate) fn reach_tick(&mut self, held: bool) {
        if held && !self.reach.held {
            self.hints.hover = None;
        }
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
            // Um nome de objecto da cena (item 8) alcança sem perguntar ao assistente.
            let verdict = if self.object_word(word.start).as_ref() == Some(&word) {
                Verdict::Reaches(Dest::Object(word.start))
            } else {
                Verdict::Definition
            };
            self.reach.asked = Some(Asked {
                word,
                revision,
                ticket: None,
                verdict,
            });
        }
    }

    /// O nome sublinhado agora: o Ctrl carregado, o rato sobre ele, e o assistente disse que o
    /// Ctrl+clique o leva a algum sítio (nesta revisão).
    pub(crate) fn reach_link(&self) -> Option<Range<usize>> {
        let (x, y) = self.reach.mouse.filter(|_| self.reach.held)?;
        let a = self.reach.asked.as_ref().filter(|a| {
            matches!(a.verdict, Verdict::Reaches(_)) && a.revision == self.buffer.revision()
        })?;
        (self.word_under(x, y)? == a.word).then(|| a.word.clone())
    }

    /// A pergunta sobre o nome sob o rato já tem resposta (nesta revisão): `Some(true)` = o
    /// Ctrl+clique leva-o a algum sítio; `None` = sem pergunta, ou à espera.
    pub(crate) fn reach_settled(&self) -> Option<bool> {
        let (x, y) = self.reach.mouse.filter(|_| self.reach.held)?;
        let a = self.reach.asked.as_ref()?;
        let current = a.revision == self.buffer.revision() && self.word_under(x, y)? == a.word;
        match a.verdict {
            Verdict::Reaches(_) if current => Some(true),
            Verdict::Nowhere if current => Some(false),
            _ => None,
        }
    }

    pub(crate) fn reach_owns(&self, ticket: Ticket) -> bool {
        self.reach
            .asked
            .as_ref()
            .is_some_and(|a| a.ticket == Some(ticket))
    }

    /// A resposta à pergunta em curso: a definição decide pela regra do Ctrl+clique; sem sítio no
    /// script, pergunta-se o balão e decide a porta única; outra resposta à definição (recusa,
    /// servidor que caiu) = nada; ao balão, a porta decide sem ele.
    pub(crate) fn reach_answer(&mut self, uri: &str, answer: Answer) {
        let Some(a) = self.reach.asked.take() else {
            return;
        };
        let verdict = match (a.verdict, answer) {
            (Verdict::Definition, Answer::Definition(targets)) => {
                match self.destination(uri, &a.word, &targets) {
                    Some(b) => Verdict::Reaches(Dest::Here(b)),
                    None => Verdict::Documentation,
                }
            }
            (Verdict::Documentation, answer) => {
                let hover = match answer {
                    Answer::Hover(h) => h.map(|h| h.text),
                    _ => None,
                };
                match self.help_anchor(&a.word, hover.as_deref()) {
                    Some(anchor) => Verdict::Reaches(Dest::Help(anchor)),
                    None => Verdict::Nowhere,
                }
            }
            _ => Verdict::Nowhere,
        };
        self.reach.asked = Some(Asked {
            ticket: None,
            verdict,
            ..a
        });
    }

    /// Para onde o Ctrl+clique em `word` leva, se a mão já o sabe nesta revisão (`None` = por saber,
    /// ou a lado nenhum: o clique pergunta).
    pub(crate) fn reach_dest(&self, word: &Range<usize>) -> Option<Dest> {
        let a = self.reach.asked.as_ref()?;
        match a.verdict {
            Verdict::Reaches(d) if a.word == *word && a.revision == self.buffer.revision() => {
                Some(d)
            }
            _ => None,
        }
    }

    /// O Ctrl+clique respondeu sobre `word`: a mão fica a sabê-lo (as perguntas dela em voo
    /// cancelaram-se ao mandar as do clique).
    pub(crate) fn reach_learn(&mut self, word: &Range<usize>, dest: Option<Dest>) {
        self.reach.asked = Some(Asked {
            word: word.clone(),
            revision: self.buffer.revision(),
            ticket: None,
            verdict: dest.map_or(Verdict::Nowhere, Verdict::Reaches),
        });
    }

    /// Sem assistente que responda: nenhum Ctrl+clique leva a lado nenhum.
    pub(crate) fn reach_offline(&mut self) {
        if let Some(a) = self.reach.asked.as_mut().filter(|a| a.unsent()) {
            a.verdict = Verdict::Nowhere;
        }
    }

    /// O rato mexeu-se para `(x, y)` (`inside` = dentro da aba): fora dela, ou por cima de algo que
    /// tapa o texto, ele não aponta nenhum nome.
    pub(crate) fn reach_mouse(&mut self, inside: bool, x: f32, y: f32) {
        self.reach.mouse = (inside && self.overlay_look(x, y).is_none()).then_some((x, y));
    }

    /// O ponteiro sobre o que tapa o texto, a mesma ordem do clique (`press`): a caixinha do F2, o
    /// balão, a lista e a barra de busca são seta, menos os campos de texto; na faixa de baixo e no
    /// contador da barra, o que um clique leva a algum sítio é a mão; `None` = nada.
    fn overlay_look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if (self.rename.is_some() && self.view.rename_contains(x, y))
            || self.view.hint_hover_contains(x, y)
            || self.view.completion_hit(x, y).is_some()
        {
            return Some(PointerLook::Arrow);
        }
        if let Some(h) = self.console_view.hit(x, y) {
            return Some(if self.console_view.clickable(h) {
                PointerLook::Hand
            } else {
                PointerLook::Arrow
            });
        }
        if self.view.counter_contains(x, y) {
            return Some(PointerLook::Hand);
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
            Hit::Text(_)
                if self
                    .reach_link()
                    .is_some_and(|w| self.word_under(x, y) == Some(w)) =>
            {
                PointerLook::Hand
            }
            Hit::Text(_) => PointerLook::Text,
            _ => PointerLook::Arrow,
        }
    }
}

impl crate::docs::CodeDocs {
    /// O ponteiro em `(x, y)` sobre a aba `id` ([`CodeDoc::pointer_look`]); `None` fora da aba.
    pub(crate) fn tab_pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        if !self.area.contains(x, y) {
            return None;
        }
        let d = self.docs.iter().find(|d| d.script.0 == id)?;
        Some(d.pointer_look(x, y))
    }
}
