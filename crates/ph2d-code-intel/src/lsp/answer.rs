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
    /// O texto pelo qual se filtra (`filterText`; `None` = o rótulo).
    pub filter: Option<String>,
    /// O texto a inserir (`insertText`, ou o `textEdit`), e a faixa que ele troca.
    pub insert: String,
    /// `insert` é um snippet (`insertTextFormat` 2: `set(${1:entity})$0`).
    pub snippet: bool,
    pub replace: Option<Range<usize>>,
    /// O comando que o editor corre depois de aceitar (`command.command`; o `luau-lsp` manda
    /// `editor.action.triggerParameterHints` em cada função).
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    pub text: String,
    pub bytes: Option<Range<usize>>,
}

/// A assinatura activa da chamada no cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub label: String,
    /// O parâmetro activo, em bytes do `label`.
    pub active: Option<Range<usize>>,
    pub param_doc: Option<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Completion(Vec<Completion>),
    Hover(Option<Hover>),
    /// `None` = o cursor não está numa chamada.
    Signature(Option<Signature>),
    /// Vazio = o servidor não sabe onde (a API: o `d.luau` não é navegável, I0b §5).
    Definition(Vec<super::Target>),
    Tokens(Vec<super::SemToken>),
    /// As `quickfix` com edição que ecoam um diagnóstico mandado (I7).
    Fixes(Vec<super::Fix>),
    TypeHints(Vec<super::TypeHint>),
    /// As trocas do rename neste documento; `None` = nenhuma, ou alguma fora dele (I10).
    Rename(Option<Vec<super::Edit>>),
    /// Os sítios do nome neste documento, ordenados (I10).
    References(Vec<Range<usize>>),
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
                filter: str_of(i, "filterText"),
                insert,
                snippet: i.get("insertTextFormat").and_then(Value::as_u64) == Some(2),
                replace,
                command: i
                    .get("command")
                    .and_then(|c| c.get("command"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                label,
            }
        })
        .collect()
}

/// O byte de `label` a `units` unidades UTF-16 do início (além do fim = o fim).
fn utf16_byte(label: &str, units: u64) -> usize {
    let mut u = 0;
    for (b, c) in label.char_indices() {
        if u >= units {
            return b;
        }
        u += c.len_utf16() as u64;
    }
    label.len()
}

/// `SignatureHelp`: a assinatura activa e o parâmetro activo dela (o da assinatura manda sobre o do
/// topo). O rótulo de um parâmetro é `[início, fim]` em UTF-16 dentro do rótulo, ou o texto dele.
pub(crate) fn signature(v: &Value) -> Option<Signature> {
    let sigs = v.get("signatures")?.as_array()?;
    let n = |o: &Value, k| o.get(k).and_then(Value::as_u64);
    let sig = sigs.get(n(v, "activeSignature").unwrap_or(0) as usize)?;
    let label = str_of(sig, "label")?;
    let active = n(sig, "activeParameter").or_else(|| n(v, "activeParameter"));
    let param = active.and_then(|i| sig.get("parameters")?.as_array()?.get(i as usize));
    let range = param.and_then(|p| match p.get("label")? {
        Value::Array(a) => {
            let (s, e) = (a.first()?.as_u64()?, a.get(1)?.as_u64()?);
            Some(utf16_byte(&label, s)..utf16_byte(&label, e))
        }
        Value::String(s) => label.find(s.as_str()).map(|b| b..b + s.len()),
        _ => None,
    });
    let doc_of = |o: &Value| o.get("documentation").map(markup).filter(|s| !s.is_empty());
    Some(Signature {
        active: range.filter(|r| r.start < r.end),
        param_doc: param.and_then(doc_of),
        doc: doc_of(sig),
        label,
    })
}

pub(crate) fn hover(text: &Text, v: &Value) -> Option<Hover> {
    let t = markup(v.get("contents")?);
    (!t.is_empty()).then(|| Hover {
        text: t,
        bytes: v.get("range").and_then(|r| range_json(text, r)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A resposta do `luau-lsp` 1.70.1 a `ph2d.set(self.id, |)` com a base `--docs` (I0b §4,
    /// `I0b_bruto.json`): o parâmetro activo é o 2.º, em offsets UTF-16 do rótulo.
    #[test]
    fn a_assinatura_marca_o_parametro_activo_em_bytes_do_rotulo() {
        let v: Value = serde_json::from_str(r#"{"activeParameter": 1, "activeSignature": 0,
            "signatures": [{"activeParameter": 1,
              "documentation": {"kind": "markdown", "value": "escreve um campo da pose."},
              "label": "function ph2d.set(entity: number, field: \"rotation\" | \"scale_x\" | \"scale_y\" | \"x\" | \"y\", value: number): ()",
              "parameters": [
                {"documentation": {"kind": "markdown", "value": "o objeto"}, "label": [18, 32]},
                {"documentation": {"kind": "markdown", "value": "o campo"}, "label": [34, 87]},
                {"documentation": {"kind": "markdown", "value": ""}, "label": [89, 102]}]}]}"#)
        .unwrap();
        let s = signature(&v).unwrap();
        let r = s.active.clone().unwrap();
        assert!(s.label[r].starts_with("field: \"rotation\""), "{s:?}");
        assert_eq!(s.param_doc.as_deref(), Some("o campo"));
        assert_eq!(s.doc.as_deref(), Some("escreve um campo da pose."));
        // Um rótulo com letras fora do ASCII: o offset conta unidades UTF-16, não bytes.
        let v = serde_json::json!({ "signatures": [{ "label": "f(ação: 🙂, b: n)",
            "parameters": [{ "label": [2, 10] }, { "label": [12, 16] }] }], "activeParameter": 1 });
        let s = signature(&v).unwrap();
        assert_eq!(&s.label[s.active.unwrap()], "b: n");
        assert_eq!(signature(&serde_json::json!({ "signatures": [] })), None);
    }
}
