//! O I7 (`docs/CodeEditor/03_onda2.md`): as correcções do servidor (`textDocument/codeAction`) e as
//! dicas de tipo (`textDocument/inlayHint`), já em BYTES do texto da versão do pedido.
//!
//! Medido no `luau-lsp` 1.70.1 (`ferramentas/medicoes_onda2/I7_acoes_e_voce_quis_dizer.md` §1–2): as
//! únicas que corrigem são `quickfix` com edição; o resto (Extract, Inline, Sort) é refactor ou do
//! Roblox. A quickfix do `FunctionUnused` vem mesmo sem o diagnóstico no `context`, mas o campo
//! `diagnostics` dela só ECOA o que se mandou: guarda-se só a que ecoa.

use super::answer::Diag;
use crate::pos::{bytes_of, position_of, range_of};
use ph2d_code_buffer::Text;
use ph2d_code_lsp::{Position, Range as LspRange};
use serde_json::{Value, json};
use std::ops::Range;

/// Uma troca de texto: a faixa e o que fica no lugar dela.
pub type Edit = (Range<usize>, String);

/// Uma correcção do servidor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<Edit>,
}

/// Uma dica de tipo que se pode inserir (`: number` depois de um nome).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeHint {
    /// Onde a dica está (o fim do nome).
    pub at: usize,
    /// O rótulo inteiro (`: number`).
    pub label: String,
    pub edits: Vec<Edit>,
}

fn pos_json(p: Position) -> Value {
    json!({ "line": p.line, "character": p.character })
}

pub(crate) fn range_json(text: &Text, bytes: Range<usize>) -> Value {
    let r = range_of(text, bytes);
    json!({ "start": pos_json(r.start), "end": pos_json(r.end) })
}

fn lsp_range(v: &Value) -> Option<LspRange> {
    let p = |p: &Value| {
        let n = |k| p.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
        Position::new(n("line"), n("character"))
    };
    Some(LspRange {
        start: p(v.get("start")?),
        end: p(v.get("end")?),
    })
}

/// O pedido de `codeAction` na faixa `bytes`, com os diagnósticos que a aba mostra ali.
pub(crate) fn params(uri: &str, text: &Text, bytes: Range<usize>, diags: &[Diag]) -> Value {
    let diags: Vec<Value> = diags
        .iter()
        .map(|d| {
            let code = d.code.as_deref().map(|c| {
                c.parse::<i64>()
                    .map_or_else(|_| Value::from(c), Value::from)
            });
            json!({ "range": range_json(text, d.bytes.clone()), "severity": d.severity,
                    "code": code, "message": d.message, "source": "Luau" })
        })
        .collect();
    json!({ "textDocument": { "uri": uri }, "range": range_json(text, bytes),
            "context": { "diagnostics": diags, "triggerKind": 1 } })
}

/// As trocas de `edit` (`WorkspaceEdit`) no documento `uri`; `None` = toca noutro documento.
fn edits_of(uri: &str, text: &Text, edit: &Value) -> Option<Vec<Edit>> {
    let mut out = Vec::new();
    let mut put = |list: &Value| {
        for e in list.as_array().into_iter().flatten() {
            let r = e.get("range").and_then(lsp_range);
            let t = e.get("newText").and_then(Value::as_str);
            if let (Some(r), Some(t)) = (r, t) {
                out.push((bytes_of(text, r), t.to_owned()));
            }
        }
    };
    for (u, list) in edit
        .get("changes")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if u != uri {
            return None;
        }
        put(list);
    }
    for dc in edit
        .get("documentChanges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let u = dc
            .get("textDocument")
            .and_then(|d| d.get("uri"))
            .and_then(Value::as_str);
        if u != Some(uri) {
            return None;
        }
        put(dc.get("edits").unwrap_or(&Value::Null));
    }
    out.sort_by_key(|(r, _)| r.start);
    let disjoint = out.windows(2).all(|w| w[0].0.end <= w[1].0.start);
    (!out.is_empty() && disjoint).then_some(out)
}

pub(crate) fn fixes(uri: &str, text: &Text, v: &Value) -> Vec<Fix> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|a| a.get("kind").and_then(Value::as_str) == Some("quickfix"))
        .filter(|a| {
            a.get("diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|d| !d.is_empty())
        })
        .filter_map(|a| {
            Some(Fix {
                title: a.get("title")?.as_str()?.to_owned(),
                edits: edits_of(uri, text, a.get("edit")?)?,
            })
        })
        .collect()
}

/// O pedido de `inlayHint` do documento inteiro (o servidor ignora a faixa, I0b §6).
pub(crate) fn hint_params(uri: &str, text: &Text) -> Value {
    let end = position_of(text, text.len());
    json!({ "textDocument": { "uri": uri },
            "range": { "start": { "line": 0, "character": 0 }, "end": pos_json(end) } })
}

pub(crate) fn type_hints(text: &Text, v: &Value) -> Vec<TypeHint> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|h| h.get("kind").and_then(Value::as_u64) == Some(1))
        .filter_map(|h| {
            let p = h.get("position")?;
            let at = bytes_of(text, lsp_range(&json!({ "start": p, "end": p }))?).start;
            let label = match h.get("label")? {
                Value::String(s) => s.clone(),
                Value::Array(parts) => parts
                    .iter()
                    .filter_map(|p| p.get("value").and_then(Value::as_str))
                    .collect(),
                _ => return None,
            };
            let mut edits = Vec::new();
            for e in h
                .get("textEdits")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                edits.push((
                    bytes_of(text, lsp_range(e.get("range")?)?),
                    e.get("newText")?.as_str()?.to_owned(),
                ));
            }
            (!edits.is_empty()).then_some(TypeHint { at, label, edits })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// As respostas do `luau-lsp` 1.70.1 (`I7_bruto.json`, fase `cliente`), encurtadas.
    #[test]
    fn so_a_quickfix_que_ecoa_um_diagnostico() {
        let t = Text::new("function init(self)\nend\nlocal function sobra() end\n");
        let uri = "file:///ph2d-project/1/a.luau";
        let edit = |l: u32, c: u32, s: &str| {
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_owned(),
                json!([{ "range": { "start": { "line": l, "character": c },
                         "end": { "line": l, "character": c } }, "newText": s }]),
            );
            json!({ "changes": changes })
        };
        let v = json!([
            { "title": "Prefix 'init' with '_' to silence", "kind": "quickfix", "diagnostics": [],
              "edit": edit(0, 9, "_") },
            { "title": "Prefix 'sobra' with '_' to silence", "kind": "quickfix",
              "diagnostics": [{ "code": 8 }], "edit": edit(2, 15, "_") },
            { "title": "Sort requires", "kind": "source.organizeImports", "diagnostics": [{ "code": 8 }],
              "edit": edit(0, 0, "") },
            { "title": "Extract to function", "kind": "refactor.extract", "diagnostics": [{ "code": 8 }] },
            { "title": "Noutro ficheiro", "kind": "quickfix", "diagnostics": [{ "code": 8 }],
              "edit": { "changes": { "file:///outro.luau": [] } } }
        ]);
        let f = fixes(uri, &t, &v);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].title, "Prefix 'sobra' with '_' to silence");
        let b = "function init(self)\nend\nlocal function ".len();
        assert_eq!(f[0].edits, [(b..b, "_".to_owned())]);
    }

    #[test]
    fn as_dicas_de_tipo_com_edicao() {
        let t = Text::new("local s\nlocal ação = 1\n");
        let v = json!([
            { "kind": 1, "position": { "line": 0, "character": 7 }, "label": [{ "value": ": " }, { "value": "number" }],
              "textEdits": [{ "newText": ": number", "range": { "start": { "line": 0, "character": 7 },
                              "end": { "line": 0, "character": 7 } } }] },
            { "kind": 1, "position": { "line": 1, "character": 10 }, "label": ": number", "textEdits": [] },
            { "kind": 2, "position": { "line": 1, "character": 0 }, "label": "x:",
              "textEdits": [{ "newText": "x", "range": { "start": { "line": 1, "character": 0 },
                              "end": { "line": 1, "character": 0 } } }] }
        ]);
        let h = type_hints(&t, &v);
        assert_eq!(
            h,
            [TypeHint {
                at: 7,
                label: ": number".into(),
                edits: vec![(7..7, ": number".into())]
            }]
        );
    }
}
