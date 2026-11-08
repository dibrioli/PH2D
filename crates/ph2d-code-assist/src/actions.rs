//! ⭐ **I7 — as acções de contexto** (Alt+Enter; `docs/CodeEditor/03_onda2.md`). A janelinha da lista
//! (I4) mostra-as, com a PRÉVIA da seleccionada ao lado; Enter (ou o clique) aplica-a num só passo de
//! undo. Por ordem:
//! 1. **«Did you mean …?»** — o `self.<campo>` que ninguém dá ([`crate::lint`]); o global `1001` e o
//!    membro `1002`, com as candidatas da completion do PRÓPRIO servidor no início da palavra (sabe o
//!    âmbito; medido: 55 itens em `pirnt`, os 17 membros depois de `ph2d.`), pela lei da distância.
//! 2. **«Add type»** — a dica de tipo do servidor no fim do nome sob o cursor, se se pode inserir.
//! 3. **As `quickfix` do servidor** que ecoam um diagnóstico que a aba MOSTRA (os ganchos do motor,
//!    escondidos, não ganham «Prefix 'init' with '_'»).
//!
//! Medição: `ferramentas/medicoes_onda2/I7_acoes_e_voce_quis_dizer.md`. Uma edição, uma tecla de fora
//! ou um clique fora fecham as acções; as respostas que chegam depois não têm dono.

use crate::lint::did_you_mean;
use ph2d_code_buffer::{Text, word_at};
use ph2d_code_intel::{Answer, Diag, Edit, Req, Ticket};
use ph2d_code_viewmodel::PreviewLine;
use ph2d_i18n::tr_with;
use std::ops::Range;

/// O `code` do Luau para «Unknown global» e «Key not found in table» (I0c).
pub const UNKNOWN_GLOBAL: &str = "1001";
pub const UNKNOWN_KEY: &str = "1002";
/// O `kind` LSP de uma palavra reservada (a completion mistura-as com os nomes).
pub const KEYWORD: u32 = 14;
/// As palavras reservadas que são VALORES: `ture` → `true` (o Python também o sugere); as outras
/// (`if`, `for`…) no lugar de um nome partem a sintaxe.
pub const VALUE_KEYWORDS: [&str; 3] = ["true", "false", "nil"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub title: String,
    /// Em bytes do texto em que as acções abriram, por ordem, sem se tocarem.
    pub edits: Vec<Edit>,
    pub then: Then,
}

/// O que fica depois de aplicada (bytes do texto DEPOIS).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Then {
    /// O cursor no fim da 1.ª troca.
    #[default]
    EndOfFirst,
    /// O cursor aqui (a soltura escolhida, item 15).
    Caret(usize),
    /// O F2 sobre o nome que começa aqui (o número promovido, item 16).
    Rename(usize),
}

/// Para que serve uma pergunta ao servidor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purpose {
    Fixes,
    /// As candidatas para o nome errado nestes bytes.
    Candidates(Range<usize>),
    /// A dica de tipo no fim deste nome.
    TypeHints(Range<usize>),
}

#[derive(Debug)]
pub struct Actions {
    pub revision: u64,
    /// Onde a janelinha se alinha (o início da palavra no cursor).
    pub anchor: usize,
    pub wants: Vec<(Req, Purpose)>,
    pub asked: Vec<(Ticket, Purpose)>,
    /// «Você quis dizer», «adicionar tipo», as do servidor.
    pub groups: [Vec<Action>; 3],
    pub selected: usize,
    pub top: usize,
    /// Uma lista que não é de consertos (a escolha da soltura, item 15): a caixa ao lado diz isto
    /// no lugar da prévia.
    pub note: Option<String>,
}

/// Os nomes da completion que podem estar no lugar de um nome errado.
pub fn candidate_names(items: &[ph2d_code_intel::Completion]) -> Vec<&str> {
    items
        .iter()
        .filter(|i| i.kind != Some(KEYWORD) || VALUE_KEYWORDS.contains(&i.label.as_str()))
        .filter(|i| is_ident(&i.label))
        .map(|i| i.label.as_str())
        .collect()
}

pub fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_alphabetic() || f == '_')
        && c.all(|c| c.is_alphanumeric() || c == '_')
}

/// O nome errado que o servidor acusa: o global inteiro (`1001`), ou o membro depois do último `.`/`:`
/// cujo nome a mensagem cita entre aspas (`1002`: «Key 'proprety' not found in table …»).
pub fn typo_of(text: &Text, d: &Diag) -> Option<Range<usize>> {
    let b = d.bytes.start.min(text.len())..d.bytes.end.min(text.len());
    let s = text.slice(b.clone());
    let r = match d.code.as_deref()? {
        UNKNOWN_GLOBAL => b,
        UNKNOWN_KEY => {
            let key = d.message.split('\'').nth(1)?;
            let dot = s.rfind(['.', ':'])? + 1;
            (s[dot..] == *key).then(|| b.start + dot..b.end)?
        }
        _ => return None,
    };
    // Uma palavra inteira: o diagnóstico seguro pelas edições (I3) pode ter ficado dentro de outra.
    (is_ident(&text.slice(r.clone())) && word_at(text, r.start) == Some(r.clone())).then_some(r)
}

impl Actions {
    pub fn did_you_mean(&mut self, typo: &Range<usize>, names: Vec<String>) {
        for name in names {
            let title = tr_with("code.action.did_you_mean", &[("name", &name)]);
            if !self.groups[0].iter().any(|a| a.title == title) {
                self.groups[0].push(Action {
                    title,
                    edits: vec![(typo.clone(), name)],
                    then: Then::default(),
                });
            }
        }
    }

    /// Tudo respondido: a lista mostra-se.
    pub fn ready(&self) -> bool {
        self.wants.is_empty() && self.asked.is_empty()
    }

    pub fn list(&self) -> impl Iterator<Item = &Action> {
        self.groups.iter().flatten()
    }

    pub fn owns(&self, t: Ticket) -> bool {
        self.asked.iter().any(|(a, _)| *a == t)
    }

    /// Envia o que falta perguntar (o `sync` do assistente, sobre o texto que ele acabou de receber).
    pub fn send(&mut self, mut ask: impl FnMut(Req) -> Ticket) {
        for (req, why) in std::mem::take(&mut self.wants) {
            self.asked.push((ask(req), why));
        }
    }

    /// Sem servidor que responda: ficam as nossas.
    pub fn offline(&mut self) {
        self.wants.clear();
        self.asked.clear();
    }

    pub fn answer(&mut self, ticket: Ticket, answer: Answer, text: &Text) {
        let Some(i) = self.asked.iter().position(|(t, _)| *t == ticket) else {
            return;
        };
        let (_, why) = self.asked.remove(i);
        match (why, answer) {
            (Purpose::Fixes, Answer::Fixes(fixes)) => {
                for f in fixes {
                    if !self.groups[2].iter().any(|a| a.title == f.title) {
                        self.groups[2].push(Action {
                            title: f.title,
                            edits: f.edits,
                            then: Then::default(),
                        });
                    }
                }
            }
            (Purpose::Candidates(typo), Answer::Completion(items)) => {
                let near = did_you_mean(&text.slice(typo.clone()), candidate_names(&items));
                self.did_you_mean(&typo, near);
            }
            (Purpose::TypeHints(word), Answer::TypeHints(hints)) => {
                let typed = hints.into_iter().find_map(|h| {
                    let ty = h.label.trim_start().strip_prefix(':')?.trim().to_owned();
                    (h.at == word.end && insertable(&ty)).then_some((ty, h.edits))
                });
                if let Some((ty, edits)) = typed {
                    self.groups[1].push(Action {
                        title: tr_with("code.action.add_type", &[("type", &ty)]),
                        edits,
                        then: Then::default(),
                    });
                }
            }
            _ => {}
        }
    }

    /// As linhas da acção `i` como ficam, sem a indentação comum, e o que muda nelas.
    pub fn preview(&self, i: usize, text: &Text) -> Vec<PreviewLine> {
        let Some(a) = self.list().nth(i) else {
            return Vec::new();
        };
        let (Some(first), Some(last)) = (a.edits.first(), a.edits.last()) else {
            return Vec::new();
        };
        let l0 = text.line_of_byte(first.0.start);
        let l1 = text.line_of_byte(last.0.end);
        let from = text.byte_of_line(l0);
        let to = text.line_end(l1);
        let mut out = String::new();
        let mut marks = Vec::new();
        let mut at = from;
        for (r, s) in &a.edits {
            out.push_str(&text.slice(at..r.start));
            marks.push(out.len()..out.len() + s.len());
            out.push_str(s);
            at = r.end;
        }
        out.push_str(&text.slice(at..to));
        let mut lines: Vec<PreviewLine> = Vec::new();
        let mut start = 0;
        for line in out.split('\n') {
            let end = start + line.len();
            let changed = marks
                .iter()
                // Uma inserção que acaba no `\n` não toca na linha seguinte.
                .filter(|m| m.start <= end && (m.end > start || (m.is_empty() && m.start >= start)))
                .map(|m| m.start.max(start) - start..m.end.min(end) - start)
                .collect();
            lines.push(PreviewLine {
                text: line.to_owned(),
                changed,
            });
            start = end + 1;
        }
        let indent = lines
            .iter()
            .filter(|l| !l.text.trim().is_empty())
            .map(|l| l.text.len() - l.text.trim_start().len())
            .min()
            .unwrap_or(0);
        for l in &mut lines {
            let cut = indent.min(l.text.len() - l.text.trim_start().len());
            l.text.drain(..cut);
            for m in &mut l.changed {
                *m = m.start.saturating_sub(cut)..m.end.saturating_sub(cut);
            }
        }
        // Só as linhas que mudam: as que ficam iguais ENTRE duas edições (o «Make it a property»
        // escreve no topo e no número) viram um «…».
        let mut kept: Vec<PreviewLine> = Vec::new();
        for l in lines {
            if !l.changed.is_empty() {
                kept.push(l);
            } else if kept.last().is_some_and(|k| !k.changed.is_empty()) {
                kept.push(PreviewLine {
                    text: "\u{2026}".to_owned(),
                    changed: Vec::new(),
                });
            }
        }
        if kept.last().is_some_and(|k| k.changed.is_empty()) {
            kept.pop();
        }
        kept
    }
}

/// Uma dica de tipo que vale a pena inserir: nem `any` (não diz nada), nem truncada, nem um genérico
/// solto (`T`: o script não o declara — inserido, é um erro novo; medido no `dt` do `update`).
pub fn insertable(ty: &str) -> bool {
    let generic = ty.chars().next().is_some_and(char::is_uppercase)
        && ty.chars().skip(1).all(|c| c.is_ascii_digit());
    !(ty.is_empty() || ty == "any" || ty.contains("TRUNCATED") || generic)
}

/// Anda `delta` na lista de `len` (dá a volta nas pontas, como a da completação) e mantém a
/// seleccionada à vista.
pub fn step(a: &mut Actions, delta: isize, len: usize, rows: usize) {
    (a.selected, a.top) = step_in(a.selected, a.top, delta, len, rows);
}

/// A seta (dá a volta) ou a página (pára no fim) numa janelinha de `rows` linhas sobre `len`.
pub fn step_in(
    selected: usize,
    top: usize,
    delta: isize,
    len: usize,
    rows: usize,
) -> (usize, usize) {
    let n = len.max(1) as isize;
    let s = if delta.abs() == 1 {
        (selected as isize + delta).rem_euclid(n) as usize
    } else {
        (selected as isize + delta).clamp(0, n - 1) as usize
    };
    let t = if s < top {
        s
    } else if s >= top + rows {
        s + 1 - rows
    } else {
        top
    };
    (s, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_nome_errado_que_o_servidor_acusa() {
        let t = Text::new("ph2d.proprety(\"x\", 1)\npirnt(1)\n");
        let d = |bytes, code: &str, message: &str| Diag {
            bytes,
            severity: 1,
            code: Some(code.into()),
            message: message.into(),
        };
        let key = d(
            0..13,
            "1002",
            "TypeError: Key 'proprety' not found in table '{ … }'",
        );
        assert_eq!(typo_of(&t, &key), Some(5..13));
        let glob = d(22..27, "1001", "TypeError: Unknown global 'pirnt'");
        assert_eq!(typo_of(&t, &glob), Some(22..27));
        let outra = d(0..13, "1002", "TypeError: Key 'x' not found in table");
        assert_eq!(
            typo_of(&t, &outra),
            None,
            "a chave da mensagem não é o membro"
        );
        let t2 = Text::new("xpirnt(1)\n");
        let dentro = d(1..6, "1001", "TypeError: Unknown global 'pirnt'");
        assert_eq!(
            typo_of(&t2, &dentro),
            None,
            "metade de uma palavra não se troca"
        );
        assert!(insertable("number") && insertable("boolean") && insertable("{ number }"));
        let item = |label: &str, kind| ph2d_code_intel::Completion {
            label: label.into(),
            kind: Some(kind),
            detail: None,
            documentation: None,
            sort_text: None,
            filter: None,
            insert: label.into(),
            snippet: false,
            replace: None,
            command: None,
        };
        let items = [
            item("print", 3),
            item("if", 14),
            item("true", 14),
            item("dt", 6),
            item("a b", 6),
        ];
        assert_eq!(candidate_names(&items), ["print", "true", "dt"]);
        assert_eq!(did_you_mean("ture", candidate_names(&items)), ["true"]);
        assert!(!insertable("any") && !insertable("T") && !insertable("U2"));
    }
}
