//! O I6 e o I11 (`docs/CodeEditor/03_onda2.md`): a definição (`textDocument/definition`) e os tokens
//! semânticos (`textDocument/semanticTokens/full`), já em BYTES do texto da versão do pedido.
//!
//! Medido no `luau-lsp` 1.70.1 (`ferramentas/medicoes_onda2/I6_I11_bruto.json`): só os NOMES têm
//! token, e nem todos — um local e um global que não existe ficam sem nenhum (a legenda não tem tipo
//! «não resolvido»). A biblioteca (`ph2d`, `ph2d.set`, `print`, `math.sin`, `self`) vem com o
//! modificador `defaultLibrary`.

use crate::pos::bytes_of;
use ph2d_code_buffer::Text;
use ph2d_code_lsp::{Position, Range as LspRange};
use serde_json::Value;
use std::ops::Range;

/// O que um nome é, segundo o servidor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sem {
    /// Da biblioteca (o modificador `defaultLibrary`), seja qual for o tipo.
    Library,
    Parameter,
    Property,
    Function,
    Variable,
    Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemToken {
    pub bytes: Range<usize>,
    pub sem: Sem,
}

/// Onde um nome está definido: o documento e a faixa do nome nele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub uri: String,
    pub bytes: Range<usize>,
}

/// A legenda do servidor (`semanticTokensProvider.legend` do `initialize`).
#[derive(Debug, Clone, Default)]
pub(crate) struct Legend {
    types: Vec<Option<Sem>>,
    /// O bit do modificador `defaultLibrary`.
    library: Option<u32>,
}

impl Legend {
    pub(crate) fn of(capabilities: &Value) -> Self {
        let legend = &capabilities["semanticTokensProvider"]["legend"];
        let names = |k: &str| -> Vec<String> {
            legend[k]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| v.as_str().unwrap_or_default().to_owned())
                        .collect()
                })
                .unwrap_or_default()
        };
        let types = names("tokenTypes")
            .iter()
            .map(|t| match t.as_str() {
                "parameter" => Some(Sem::Parameter),
                "property" | "enumMember" => Some(Sem::Property),
                "function" | "method" => Some(Sem::Function),
                "variable" => Some(Sem::Variable),
                "type" | "class" | "enum" | "interface" | "struct" | "typeParameter" => {
                    Some(Sem::Type)
                }
                // Palavras reservadas, strings, números, comentários: o tree-sitter já os sabe.
                _ => None,
            })
            .collect();
        let library = names("tokenModifiers")
            .iter()
            .position(|m| m == "defaultLibrary")
            .and_then(|i| u32::try_from(i).ok())
            .filter(|b| *b < 32);
        Self { types, library }
    }
}

/// `SemanticTokens.data`: cinco inteiros por token — a linha e o início RELATIVOS ao anterior (o
/// início só quando a linha é a mesma), o comprimento em UTF-16, o tipo e os modificadores.
pub(crate) fn tokens(text: &Text, legend: &Legend, v: &Value) -> Vec<SemToken> {
    let Some(data) = v.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    let n = |i: usize| {
        data.get(i)
            .and_then(Value::as_u64)
            .and_then(|x| u32::try_from(x).ok())
            .unwrap_or(0)
    };
    let (mut line, mut col) = (0u32, 0u32);
    let mut out = Vec::with_capacity(data.len() / 5);
    for i in (0..data.len() / 5).map(|k| k * 5) {
        let (dl, ds, len, ty, mods) = (n(i), n(i + 1), n(i + 2), n(i + 3), n(i + 4));
        if dl > 0 {
            line = line.saturating_add(dl);
            col = ds;
        } else {
            col = col.saturating_add(ds);
        }
        let sem = if legend.library.is_some_and(|b| (mods >> b) & 1 == 1) {
            Some(Sem::Library)
        } else {
            legend.types.get(ty as usize).copied().flatten()
        };
        let Some(sem) = sem else { continue };
        let bytes = bytes_of(
            text,
            LspRange {
                start: Position::new(line, col),
                end: Position::new(line, col.saturating_add(len)),
            },
        );
        if bytes.start < bytes.end {
            out.push(SemToken { bytes, sem });
        }
    }
    out
}

/// `Location`, `Location[]` ou `LocationLink[]` (desta, a faixa do NOME: `targetSelectionRange`).
/// `text_of` dá o texto de cada documento aberto no servidor (o do pedido: o da versão dele); um
/// documento que o worker não tem fica de fora.
pub(crate) fn definition(text_of: impl Fn(&str) -> Option<Text>, v: &Value) -> Vec<Target> {
    let one = |l: &Value| -> Option<Target> {
        let uri = l.get("uri").or_else(|| l.get("targetUri"))?.as_str()?;
        let range = l
            .get("range")
            .or_else(|| l.get("targetSelectionRange"))
            .or_else(|| l.get("targetRange"))?;
        let text = text_of(uri)?;
        Some(Target {
            uri: uri.to_owned(),
            bytes: super::answer::range_json(&text, range)?,
        })
    };
    match v {
        Value::Array(a) => a.iter().filter_map(one).collect(),
        Value::Object(_) => one(v).into_iter().collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn legend() -> Legend {
        let caps: Value = serde_json::from_str(include_str!(
            "../../../../docs/CodeEditor/ferramentas/medicoes_onda2/caps_luau_lsp_1.70.1.json"
        ))
        .unwrap();
        Legend::of(&caps["capabilities"])
    }

    /// A resposta real do `luau-lsp` 1.70.1 ao texto partido da sonda (`I6_I11_bruto.json`,
    /// `partido`): `ph2d` (variable · readonly, defaultLibrary) e `get` (function · os mesmos), com
    /// o início relativo na mesma linha. Um emoji antes: as colunas são UTF-16, os bytes não.
    #[test]
    fn os_tokens_relativos_viram_bytes_e_a_biblioteca_manda_sobre_o_tipo() {
        let l = legend();
        let t = Text::new("if self.active then\n  local z = ph2d.get(self.id, \"x\")\n");
        let v = json!({ "data": [1, 12, 4, 8, 516, 0, 5, 3, 12, 516] });
        let got = tokens(&t, &l, &v);
        let s = t.to_string();
        let words: Vec<(&str, Sem)> = got.iter().map(|k| (&s[k.bytes.clone()], k.sem)).collect();
        assert_eq!(words, [("ph2d", Sem::Library), ("get", Sem::Library)]);
        // Sem o modificador: o tipo da legenda (8 = variable, 7 = parameter); 15 = keyword fica de fora.
        let t = Text::new("-- 🙂 x\nf(dt, k)\n");
        let v = json!({ "data": [1, 2, 2, 7, 0, 0, 4, 1, 8, 0, 0, 2, 1, 15, 0] });
        let s = t.to_string();
        let got: Vec<(&str, Sem)> = tokens(&t, &l, &v)
            .iter()
            .map(|k| (&s[k.bytes.clone()], k.sem))
            .collect();
        assert_eq!(got, [("dt", Sem::Parameter), ("k", Sem::Variable)]);
        let t = Text::new("🙂🙂 ab\n");
        let v = json!({ "data": [0, 5, 2, 8, 0] });
        let got = tokens(&t, &l, &v);
        assert_eq!(&t.to_string()[got[0].bytes.clone()], "ab", "UTF-16, não bytes");
    }

    #[test]
    fn a_definicao_aceita_as_tres_formas_e_larga_o_documento_desconhecido() {
        let t = Text::new("local 🙂ab = 1\nprint(ab)\n");
        let at = |uri: &str| (uri == "file:///a").then(|| t.clone());
        let r = json!({ "start": { "line": 0, "character": 8 }, "end": { "line": 0, "character": 10 } });
        let ab = 10..12;
        let loc = json!({ "uri": "file:///a", "range": r });
        assert_eq!(definition(at, &loc)[0].bytes, ab);
        assert_eq!(definition(at, &json!([loc]))[0].bytes, ab);
        let link = json!([{ "targetUri": "file:///a", "targetSelectionRange": r,
            "targetRange": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 14 } } }]);
        assert_eq!(definition(at, &link)[0].bytes, ab, "o NOME, não a declaração inteira");
        assert!(definition(at, &json!([{ "uri": "file:///b", "range": r }])).is_empty());
        assert!(definition(at, &json!([])).is_empty());
        assert!(definition(at, &Value::Null).is_empty());
    }
}
