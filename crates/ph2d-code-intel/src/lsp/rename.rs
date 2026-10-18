//! O I10 (`docs/CodeEditor/03_onda2.md`): renomear (`textDocument/rename`) e as referências
//! (`textDocument/references`), em BYTES do texto da versão do pedido.
//!
//! Medido no `luau-lsp` 1.70.1 (`ferramentas/medicoes_onda2/I10_rename_e_undo.md`): a resposta é
//! sempre `changes` e sempre só deste documento; o servidor aceita palavras reservadas e nomes que já
//! existem (a validação é de quem pergunta) e MORRE sobre um membro (`ph2d.set`, `math.sin`) — quem
//! pergunta nunca manda um membro.

use super::fix::{Edit, edits_of};
use ph2d_code_buffer::Text;
use serde_json::{Value, json};
use std::ops::Range;

pub(crate) fn rename_params(uri: &str, text: &Text, byte: usize, name: &str) -> Value {
    let mut p = super::worker::params_at(uri, text, byte);
    p["newName"] = Value::from(name);
    p
}

pub(crate) fn references_params(uri: &str, text: &Text, byte: usize) -> Value {
    let mut p = super::worker::params_at(uri, text, byte);
    p["context"] = json!({ "includeDeclaration": true });
    p
}

/// As trocas do rename neste documento; `None` = nenhuma, ou alguma noutro documento.
pub(crate) fn rename(uri: &str, text: &Text, v: &Value) -> Option<Vec<Edit>> {
    edits_of(uri, text, v)
}

/// Os sítios neste documento, ordenados e sem repetidos.
pub(crate) fn references(uri: &str, text: &Text, v: &Value) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = v
        .as_array()
        .into_iter()
        .flatten()
        .filter(|l| l.get("uri").and_then(Value::as_str) == Some(uri))
        .filter_map(|l| super::answer::range_json(text, l.get("range")?))
        .collect();
    out.sort_by_key(|r| (r.start, r.end));
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_referencias_ficam_neste_documento_em_bytes_depois_de_um_emoji() {
        let text = Text::new("local tag = \"🎈\"; local d = 1\nprint(d)\n");
        let uri = "file:///ph2d-project/1/a.luau";
        let v = json!([
            { "uri": uri, "range": { "start": { "line": 1, "character": 6 }, "end": { "line": 1, "character": 7 } } },
            { "uri": uri, "range": { "start": { "line": 0, "character": 24 }, "end": { "line": 0, "character": 25 } } },
            { "uri": "file:///ph2d-project/2/b.luau", "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } } }
        ]);
        let r = references(uri, &text, &v);
        let s = text.to_string();
        assert_eq!(r.len(), 2);
        assert!(r.iter().all(|r| &s[r.clone()] == "d"), "{r:?}");
        assert!(r[0].start < r[1].start);
    }

    #[test]
    fn um_rename_que_toca_outro_documento_nao_se_aplica() {
        let text = Text::new("local a = 1\n");
        let uri = "file:///ph2d-project/1/a.luau";
        let r = |u: &str| json!({ "changes": { u: [ { "newText": "b", "range": { "start": { "line": 0, "character": 6 }, "end": { "line": 0, "character": 7 } } } ] } });
        assert_eq!(
            rename(uri, &text, &r(uri)),
            Some(vec![(6..7, "b".to_owned())])
        );
        assert_eq!(rename(uri, &text, &r("file:///outro")), None);
    }
}
