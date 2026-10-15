//! As três queries compiladas UMA vez por processo, e o mapa captura → [`HighlightKind`].

use crate::highlight::HighlightKind;
use ph2d_code_buffer::Text;
use std::sync::OnceLock;
use tree_sitter::{Language, Node, Query};

pub const HIGHLIGHTS_SCM: &str = include_str!("../queries/highlights.scm");
pub const FOLDS_SCM: &str = include_str!("../queries/folds.scm");
pub const INDENTS_SCM: &str = include_str!("../queries/indents.scm");

/// O que o realce faz com uma captura da query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureUse {
    Paint(HighlightKind),
    /// Captura auxiliar que não pinta (`@spell` do nvim; `@_x` só serve a um predicado).
    Ignore,
}

/// A tabela COMPLETA: toda captura de `highlights.scm` tem de estar aqui (teste de cegueira).
const CAPTURES: &[(&str, CaptureUse)] = {
    use CaptureUse::{Ignore, Paint};
    use HighlightKind as K;
    &[
        ("keyword", Paint(K::Keyword)),
        ("keyword.directive", Paint(K::Keyword)),
        ("keyword.return", Paint(K::Keyword)),
        ("keyword.repeat", Paint(K::Keyword)),
        ("keyword.conditional", Paint(K::Keyword)),
        ("keyword.function", Paint(K::Keyword)),
        ("keyword.operator", Paint(K::Keyword)),
        ("keyword.coroutine", Paint(K::Builtin)),
        ("operator", Paint(K::Operator)),
        ("variable", Paint(K::Variable)),
        ("variable.builtin", Paint(K::Builtin)),
        ("variable.member", Paint(K::Property)),
        ("variable.parameter", Paint(K::Parameter)),
        ("type", Paint(K::Type)),
        ("type.builtin", Paint(K::Type)),
        ("type.definition", Paint(K::Type)),
        ("constant", Paint(K::Constant)),
        ("constant.builtin", Paint(K::Builtin)),
        ("module.builtin", Paint(K::Builtin)),
        ("function", Paint(K::Function)),
        ("function.call", Paint(K::FunctionCall)),
        ("function.method.call", Paint(K::FunctionCall)),
        ("function.builtin", Paint(K::Builtin)),
        ("constructor", Paint(K::Punctuation)),
        ("number", Paint(K::Number)),
        ("string", Paint(K::String)),
        ("string.regexp", Paint(K::String)),
        ("boolean", Paint(K::Boolean)),
        ("punctuation.delimiter", Paint(K::Punctuation)),
        ("punctuation.bracket", Paint(K::Punctuation)),
        ("attribute", Paint(K::Attribute)),
        ("comment", Paint(K::Comment)),
        ("comment.documentation", Paint(K::Comment)),
        ("spell", Ignore),
        ("_method", Ignore),
    ]
};

/// O texto de um nó para os predicados (`#eq?`, `#match?`), lido da rope por pedaços.
pub(crate) fn source<'a>(text: &'a Text) -> impl FnMut(Node) -> crop::iter::Chunks<'a> + 'a {
    let rope = text.rope();
    move |n: Node| rope.byte_slice(n.byte_range()).chunks()
}

pub fn capture_table() -> &'static [(&'static str, CaptureUse)] {
    CAPTURES
}

pub(crate) struct Queries {
    pub language: Language,
    pub highlights: Query,
    /// Índice da captura → o que ela pinta (`None` = ignorada ou desconhecida).
    pub paint: Vec<Option<HighlightKind>>,
    pub folds: Query,
    pub indents: Query,
    pub indent_begin: u32,
    pub indent_end: u32,
    pub indent_branch: u32,
    pub indent_open: u32,
}

/// Uma query embutida que não compila é defeito do PROGRAMA, nunca do texto do utilizador —
/// os testes compilam as três antes de qualquer outra coisa.
fn compile(language: &Language, src: &str, name: &str) -> Query {
    Query::new(language, src).unwrap_or_else(|e| panic!("query embutida `{name}` não compila: {e}"))
}

fn capture(q: &Query, name: &str) -> u32 {
    q.capture_index_for_name(name)
        .unwrap_or_else(|| panic!("indents.scm sem a captura @{name}"))
}

pub(crate) fn get() -> &'static Queries {
    static Q: OnceLock<Queries> = OnceLock::new();
    Q.get_or_init(|| {
        let language: Language = tree_sitter_luau::LANGUAGE.into();
        let highlights = compile(&language, HIGHLIGHTS_SCM, "highlights.scm");
        let paint = highlights
            .capture_names()
            .iter()
            .map(|n| match CAPTURES.iter().find(|(c, _)| c == n) {
                Some((_, CaptureUse::Paint(k))) => Some(*k),
                _ => None,
            })
            .collect();
        let folds = compile(&language, FOLDS_SCM, "folds.scm");
        let indents = compile(&language, INDENTS_SCM, "indents.scm");
        let (indent_begin, indent_end, indent_branch, indent_open) = (
            capture(&indents, "indent.begin"),
            capture(&indents, "indent.end"),
            capture(&indents, "indent.branch"),
            capture(&indents, "indent.open"),
        );
        Queries {
            language,
            highlights,
            paint,
            folds,
            indents,
            indent_begin,
            indent_end,
            indent_branch,
            indent_open,
        }
    })
}

/// As capturas e os predicados que a query de realce REALMENTE tem (para os testes de cegueira).
pub fn highlight_captures() -> Vec<String> {
    get()
        .highlights
        .capture_names()
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Predicados que o tree-sitter puro NÃO avalia (passariam calados): tem de ser vazio.
pub fn unevaluated_predicates() -> Vec<String> {
    let q = get();
    [&q.highlights, &q.folds, &q.indents]
        .iter()
        .flat_map(|query| {
            (0..query.pattern_count()).flat_map(move |i| {
                query
                    .general_predicates(i)
                    .iter()
                    .map(|p| p.operator.to_string())
            })
        })
        .collect()
}
