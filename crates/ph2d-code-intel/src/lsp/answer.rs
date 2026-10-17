//! As respostas do servidor, já em BYTES do texto da versão a que respondem.

use crate::pos::bytes_of;
use ph2d_code_buffer::Text;
use ph2d_code_lsp::Diagnostic;
use serde_json::Value;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub bytes: Range<usize>,
    /// 1 erro · 2 aviso · 3 informação · 4 dica (a do LSP).
    pub severity: u8,
    pub code: Option<String>,
    pub message: String,
}

impl Diag {
    pub(crate) fn of(text: &Text, d: &Diagnostic) -> Self {
        Self {
            bytes: bytes_of(text, d.range),
            severity: d.severity.unwrap_or(1),
            code: d.code.clone(),
            message: d.message.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    /// O texto a inserir (`insertText`, ou o `textEdit`), e a faixa que ele troca.
    pub insert: String,
    pub replace: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    pub text: String,
    pub bytes: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Completion(Vec<Completion>),
    Hover(Option<Hover>),
    /// O servidor recusou, morreu ou o pedido não pôde ir (o texto diz porquê).
    Failed(String),
}

fn str_of(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_owned)
}

/// `MarkupContent`, `MarkedString`, uma lista deles, ou texto.
pub(crate) fn markup(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        Value::Array(a) => a.iter().map(markup).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

pub(crate) fn range_json(text: &Text, v: &Value) -> Option<Range<usize>> {
    let p = |p: &Value| {
        let n = |k| p.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
        ph2d_code_lsp::Position::new(n("line"), n("character"))
    };
    let r = ph2d_code_lsp::Range {
        start: p(v.get("start")?),
        end: p(v.get("end")?),
    };
    Some(bytes_of(text, r))
}

pub(crate) fn completion(text: &Text, v: &Value) -> Vec<Completion> {
    let items = match v {
        Value::Array(a) => a.as_slice(),
        Value::Object(o) => o
            .get("items")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice),
        _ => &[],
    };
    items
        .iter()
        .map(|i| {
            let label = str_of(i, "label").unwrap_or_default();
            let edit = i.get("textEdit");
            let replace = edit.and_then(|e| range_json(text, e.get("range").or(e.get("replace"))?));
            let insert = edit
                .and_then(|e| str_of(e, "newText"))
                .or_else(|| str_of(i, "insertText"))
                .unwrap_or_else(|| label.clone());
            Completion {
                kind: i.get("kind").and_then(Value::as_u64).map(|k| k as u32),
                detail: str_of(i, "detail"),
                documentation: i.get("documentation").map(markup).filter(|s| !s.is_empty()),
                sort_text: str_of(i, "sortText"),
                insert,
                replace,
                label,
            }
        })
        .collect()
}

pub(crate) fn hover(text: &Text, v: &Value) -> Option<Hover> {
    let t = markup(v.get("contents")?);
    (!t.is_empty()).then(|| Hover {
        text: t,
        bytes: v.get("range").and_then(|r| range_json(text, r)),
    })
}
