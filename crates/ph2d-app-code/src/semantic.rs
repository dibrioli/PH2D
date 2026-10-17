//! ⭐ **I11 — as cores do assistente sobre as do tree-sitter** (`docs/CodeEditor/03_onda2.md`).
//!
//! - **Quando:** numa PAUSA da escrita — a do `idle_parse_delay` do Godot 4.7.2 (1,5 s; 0,5 s com
//!   erros: `idle_parse_delay_with_errors_found`, lidos de fábrica no I0b, `I0b_bruto.json`
//!   `godot.valores_de_fabrica`) —, sobre o texto que o servidor acabou de receber. Nunca na tecla.
//! - **Edições:** os tokens seguem-nas; um token que uma edição TOCA larga-se (o nome mudou), e a
//!   resposta em voo também (é sobre um texto que já não há).
//! - **O que pintam:** só NOMES (o servidor não recolore palavras reservadas, strings nem comentários).
//! - **O global que não existe** não tem token (medido no `luau-lsp` 1.70.1, `I6_I11_bruto.json`, e a
//!   legenda não tem tipo «não resolvido»): a cor de alerta vem do diagnóstico `1001` que a aba já
//!   segura (I3), e segue-o.

use crate::problems::Held;
use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_intel::{Answer, Diag, Sem, SemToken, Ticket};
use ph2d_code_syntax::Syntax;
use ph2d_panel_code::Ink;
use std::ops::Range;

/// A pausa da escrita antes de pedir as cores (`idle_parse_delay`).
pub(crate) const IDLE_NS: u128 = 1_500_000_000;
/// A mesma, com um erro na aba (`idle_parse_delay_with_errors_found`).
pub(crate) const IDLE_ERRORS_NS: u128 = 500_000_000;
/// O `code` do Luau para «Unknown global» (I0c).
const UNKNOWN_GLOBAL: &str = "1001";

/// As cores do assistente que a aba segura.
#[derive(Default)]
pub(crate) struct SemHeld {
    /// Por ordem do texto (a do servidor; as edições não a trocam).
    pub list: Vec<SemToken>,
    /// O (arranque do servidor, versão) da última pergunta: uma por versão de cada arranque.
    pub asked: Option<(u64, u64)>,
    pub ticket: Option<Ticket>,
}

impl SemHeld {
    pub fn edited(&mut self, edits: &[TextEdit]) {
        if edits.is_empty() {
            return;
        }
        self.ticket = None;
        for e in edits {
            self.list.retain_mut(|t| {
                if e.old_end_byte < t.bytes.start {
                    let moved = |b: usize| b - e.old_end_byte + e.new_end_byte;
                    t.bytes = moved(t.bytes.start)..moved(t.bytes.end);
                    true
                } else {
                    e.start_byte > t.bytes.end
                }
            });
        }
    }

    /// A resposta `ticket`: as cores novas; um erro deixa as que havia.
    pub fn answer(&mut self, ticket: Ticket, answer: Answer) {
        if self.ticket != Some(ticket) {
            return;
        }
        self.ticket = None;
        if let Answer::Tokens(list) = answer {
            self.list = list;
        }
    }

    /// A pausa que esta aba espera depois da última mudança.
    pub fn idle_ns(assist: &Held) -> u128 {
        if assist.list.iter().any(|d| d.severity == 1) {
            IDLE_ERRORS_NS
        } else {
            IDLE_NS
        }
    }
}

fn ink_of(s: Sem) -> Ink {
    match s {
        Sem::Library => Ink::Builtin,
        Sem::Parameter => Ink::Parameter,
        Sem::Property => Ink::Property,
        Sem::Function => Ink::Function,
        Sem::Variable => Ink::Variable,
        Sem::Type => Ink::Type,
    }
}

/// O realce de `range`: o do tree-sitter, os tokens do assistente por cima, e por cima de tudo o
/// global que não existe. O assistente só recolore NOMES: onde o tree-sitter vê outra coisa (uma
/// palavra reservada, uma string, um comentário — um token velho que a pausa ainda não refez), fica
/// a cor da gramática.
pub(crate) fn spans(
    syntax: &Syntax,
    text: &Text,
    sem: &SemHeld,
    assist: &[Diag],
    range: Range<usize>,
) -> Vec<(Range<usize>, Ink)> {
    let mut out: Vec<(Range<usize>, Ink)> = syntax
        .highlights(text, range.clone())
        .into_iter()
        .map(|h| (h.range, crate::problems::ink_of(h.kind)))
        .collect();
    let first = sem.list.partition_point(|t| t.bytes.end <= range.start);
    let tokens = sem.list[first..]
        .iter()
        .take_while(|t| t.bytes.start < range.end)
        .map(|t| (t.bytes.clone(), ink_of(t.sem)));
    let unresolved = assist
        .iter()
        .filter(|d| d.code.as_deref() == Some(UNKNOWN_GLOBAL))
        .filter(|d| d.bytes.start < range.end && range.start < d.bytes.end)
        .map(|d| (d.bytes.clone(), Ink::Unresolved));
    for (r, ink) in tokens.chain(unresolved) {
        let r = r.start.max(range.start)..r.end.min(range.end);
        if r.start < r.end && names_only(&out, &r) {
            out = paint_over(out, r, ink);
        }
    }
    out
}

/// Tudo o que o tree-sitter pinta em `r` é um nome.
fn names_only(spans: &[(Range<usize>, Ink)], r: &Range<usize>) -> bool {
    spans
        .iter()
        .filter(|(s, _)| s.start < r.end && r.start < s.end)
        .all(|(_, k)| {
            matches!(
                k,
                Ink::Plain
                    | Ink::Variable
                    | Ink::Parameter
                    | Ink::Property
                    | Ink::Function
                    | Ink::Builtin
                    | Ink::Type
                    | Ink::Unresolved
            )
        })
}

/// `spans` (ordenados, sem sobreposição) com `r` pintado de `ink` por cima.
fn paint_over(
    spans: Vec<(Range<usize>, Ink)>,
    r: Range<usize>,
    ink: Ink,
) -> Vec<(Range<usize>, Ink)> {
    let mut out = Vec::with_capacity(spans.len() + 2);
    let mut placed = false;
    for (s, k) in spans {
        if s.end <= r.start {
            out.push((s, k));
            continue;
        }
        if !placed {
            if s.start < r.start {
                out.push((s.start..r.start, k));
            }
            out.push((r.clone(), ink));
            placed = true;
        }
        if s.end > r.end {
            out.push((s.start.max(r.end)..s.end, k));
        }
    }
    if !placed {
        out.push((r, ink));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(bytes: Range<usize>) -> SemToken {
        SemToken {
            bytes,
            sem: Sem::Library,
        }
    }

    fn edit(start: usize, old_end: usize, new_end: usize) -> TextEdit {
        TextEdit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: new_end,
            start: ph2d_code_buffer::Point::default(),
            old_end: ph2d_code_buffer::Point::default(),
            new_end: ph2d_code_buffer::Point::default(),
        }
    }

    /// Antes do nome desloca; depois fica; a tocar (dentro, colado ao início ou ao fim) larga-o.
    #[test]
    fn um_token_segue_as_edicoes_e_larga_se_quando_o_nome_muda() {
        let mut h = SemHeld {
            list: vec![tok(10..14), tok(20..23)],
            ..SemHeld::default()
        };
        h.edited(&[edit(2, 2, 5)]);
        assert_eq!(h.list, [tok(13..17), tok(23..26)]);
        h.edited(&[edit(30, 30, 31)]);
        assert_eq!(h.list, [tok(13..17), tok(23..26)]);
        h.edited(&[edit(17, 17, 18)]);
        assert_eq!(h.list, [tok(24..27)], "escrever colado ao fim muda o nome");
        h.edited(&[edit(24, 24, 25)]);
        assert!(h.list.is_empty(), "escrever colado ao início muda o nome");
    }

    #[test]
    fn pintar_por_cima_parte_o_trecho_de_baixo() {
        let base = vec![(0..4, Ink::Keyword), (5..15, Ink::Variable), (16..18, Ink::Number)];
        let out = paint_over(base.clone(), 7..10, Ink::Builtin);
        assert_eq!(
            out,
            [
                (0..4, Ink::Keyword),
                (5..7, Ink::Variable),
                (7..10, Ink::Builtin),
                (10..15, Ink::Variable),
                (16..18, Ink::Number)
            ]
        );
        // Num buraco entre trechos, e no fim.
        assert_eq!(paint_over(base.clone(), 15..16, Ink::Unresolved)[2], (15..16, Ink::Unresolved));
        assert_eq!(paint_over(base, 20..22, Ink::Unresolved)[3], (20..22, Ink::Unresolved));
    }
}
