//! ⭐ **E7 — os problemas da aba**: o erro que o Luau deu ao texto salvo (com a LINHA) e os erros vivos
//! do tree-sitter enquanto se escreve, sem duplicar; o símbolo culpado vem SEMPRE da árvore (o lugar
//! exacto, conferido contra o `luau-lsp` como oráculo em `tests/it/oraculo_lsp.rs`).

use ph2d_code_buffer::Text;
use ph2d_code_syntax::{HighlightKind, SyntaxError};
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::{Ink, Problem, Severity};
use ph2d_script::ScriptDiagnostic;

/// A tinta de cada tipo de realce (16 → 14: chamada = função, booleano = constante, atributo = palavra
/// reservada).
#[must_use]
pub fn ink_of(k: HighlightKind) -> Ink {
    match k {
        HighlightKind::Keyword | HighlightKind::Attribute => Ink::Keyword,
        HighlightKind::Function | HighlightKind::FunctionCall => Ink::Function,
        HighlightKind::Variable => Ink::Variable,
        HighlightKind::Parameter => Ink::Parameter,
        HighlightKind::Property => Ink::Property,
        HighlightKind::Type => Ink::Type,
        HighlightKind::String => Ink::String,
        HighlightKind::Number => Ink::Number,
        HighlightKind::Boolean | HighlightKind::Constant => Ink::Constant,
        HighlightKind::Builtin => Ink::Builtin,
        HighlightKind::Comment => Ink::Comment,
        HighlightKind::Operator => Ink::Operator,
        HighlightKind::Punctuation => Ink::Punctuation,
    }
}

/// O culpado da árvore numa linha (0-based): o primeiro erro que começa nela.
#[must_use]
pub fn culprit_on_line(
    text: &Text,
    errors: &[SyntaxError],
    line: usize,
) -> Option<std::ops::Range<usize>> {
    errors
        .iter()
        .find(|e| text.line_of_byte(e.range.start) == line)
        .map(|e| e.range.clone())
}

/// Os problemas da aba. `saved` = o diagnóstico do Luau para o texto salvo e a linha (0-based) onde
/// ele está AGORA no buffer — quem chama segue-a pelas edições, ou não o passa (ver
/// `CodeDoc::problems`).
#[must_use]
pub fn problems(
    text: &Text,
    errors: &[SyntaxError],
    saved: Option<(&ScriptDiagnostic, usize)>,
) -> Vec<Problem> {
    let mut out: Vec<Problem> = errors
        .iter()
        .map(|e| Problem {
            line: text.line_of_byte(e.range.start),
            severity: Severity::Error,
            culprit: Some(e.range.clone()),
            message: match &e.missing {
                Some(sym) => tr_with("code.problem.missing", &[("sym", sym)]),
                None => tr("code.problem.syntax").to_owned(),
            },
        })
        .collect();
    // Um erro por linha: o do tree-sitter que começa primeiro.
    out.dedup_by_key(|p| p.line);
    if let Some((d, line)) = saved {
        let line = line.min(text.line_count().saturating_sub(1));
        match out.iter_mut().find(|p| p.line == line) {
            Some(p) => d.message.clone_into(&mut p.message),
            None => out.push(Problem {
                line,
                severity: Severity::Error,
                culprit: culprit_on_line(text, errors, line),
                message: d.message.clone(),
            }),
        }
        out.sort_by_key(|p| p.line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada tipo de realce tem tinta — um tipo novo sem braço não compila; e os três que juntam vão
    /// para onde a pintura os espera.
    #[test]
    fn todo_tipo_de_realce_tem_tinta() {
        for k in HighlightKind::ALL {
            let _ = ink_of(k);
        }
        assert_eq!(ink_of(HighlightKind::FunctionCall), Ink::Function);
        assert_eq!(ink_of(HighlightKind::Boolean), Ink::Constant);
        assert_eq!(ink_of(HighlightKind::Attribute), Ink::Keyword);
        assert_eq!(ink_of(HighlightKind::String), Ink::String);
    }
}
