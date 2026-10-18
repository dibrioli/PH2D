//! ⭐ **I12 — os esqueletos**, gerados da `api_table` (`docs/CodeEditor/03_onda2.md`): um gancho novo
//! em `HOOKS` = um esqueleto novo, sem código à mão.
//!
//! - **Ganchos** (`init`, `update`, `on_signal`): cada parâmetro é um buraco com o tipo escrito ao
//!   lado (`function update(${1:self}: ScriptSelf, ${2:dt}: number)`) e o corpo é o `$0`. Preenchido,
//!   passa limpo no `luau-lsp` normal e `--!strict` (medição I9_I12_I14 §I12). Propõem-se no TOPO do
//!   script, no início de uma instrução ou depois de `function ` (o idioma do Godot), só os que o
//!   script ainda não declara — o `luau-lsp` não os propõe (0 itens depois de `function `).
//! - **Chamadas da API** (`ph2d.property`, `ph2d.set`…): os parâmetros obrigatórios são buracos, e um
//!   de texto (`string`, ou uma união de literais como `PoseField`) entra ENTRE ASPAS, com o buraco
//!   dentro — o snippet do servidor não as põe.

use crate::complete::Item;
use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;
use ph2d_script::api_table::{API, API_TYPES, ApiEntry, ApiHook, HOOKS};

/// O `kind` LSP de um modelo (snippet).
const SNIPPET: u32 = 15;

/// Os parâmetros de um tipo de função Luau `(a: T, b: U?) -> R`: (nome, tipo, opcional).
pub(crate) fn params(luau_type: &str) -> Vec<(&str, &str, bool)> {
    let Some(inner) = luau_type.strip_prefix('(') else {
        return Vec::new();
    };
    // `->` não fecha nada: o `>` dele não conta.
    let shut =
        |prev: Option<char>, c: char| matches!(c, ')' | '}') || (c == '>' && prev != Some('-'));
    let mut depth = 0i32;
    let mut end = inner.len();
    let mut prev = None;
    for (i, c) in inner.char_indices() {
        if c == ')' && depth == 0 {
            end = i;
            break;
        }
        if matches!(c, '(' | '{' | '<') {
            depth += 1;
        } else if shut(prev, c) {
            depth -= 1;
        }
        prev = Some(c);
    }
    let list = &inner[..end];
    let mut out = Vec::new();
    let (mut depth, mut from, mut prev) = (0i32, 0usize, None);
    for (i, c) in list
        .char_indices()
        .chain(std::iter::once((list.len(), ',')))
    {
        if matches!(c, '(' | '{' | '<') {
            depth += 1;
        } else if shut(prev, c) {
            depth -= 1;
        } else if c == ',' && depth == 0 {
            let p = list[from..i].trim();
            from = i + 1;
            if let Some((name, ty)) = p.split_once(':') {
                let ty = ty.trim();
                out.push((name.trim(), ty.trim_end_matches('?'), ty.ends_with('?')));
            }
        }
        prev = Some(c);
    }
    out
}

/// O tipo é texto: `string`, ou um nome da `API_TYPES` cuja 1.ª alternativa é um literal de texto.
/// Devolve o texto do buraco (o nome do parâmetro, ou o 1.º literal).
fn textual(ty: &str, name: &str) -> Option<String> {
    let first = API_TYPES
        .iter()
        .find(|t| t.ident == ty)
        .map_or(ty, |t| t.definition)
        .split('|')
        .next()
        .unwrap_or("")
        .trim();
    if first == "string" {
        Some(name.to_owned())
    } else {
        first
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .map(str::to_owned)
    }
}

/// O esqueleto do gancho; `keyword` = com o `function ` (sem ele: depois de o artista o escrever).
pub(crate) fn hook_snippet(h: &ApiHook, keyword: bool) -> String {
    let args: Vec<String> = params(h.luau_type)
        .iter()
        .enumerate()
        .map(|(k, (name, ty, _))| format!("${{{}:{name}}}: {ty}", k + 1))
        .collect();
    format!(
        "{}{}({})\n\t$0\nend",
        if keyword { "function " } else { "" },
        h.ident,
        args.join(", ")
    )
}

/// A chamada `ident(…)` com um buraco por parâmetro obrigatório (os de texto entre aspas).
pub(crate) fn call_snippet(e: &ApiEntry) -> String {
    let args: Vec<String> = params(e.luau_type)
        .iter()
        .filter(|(_, _, optional)| !optional)
        .enumerate()
        .map(|(k, (name, ty, _))| match textual(ty, name) {
            Some(t) => format!("\"${{{}:{t}}}\"", k + 1),
            None => format!("${{{}:{name}}}", k + 1),
        })
        .collect();
    format!("{}({})$0", e.ident, args.join(", "))
}

/// A entrada da API `table.label`, se é uma.
pub(crate) fn api_entry(table: &str, label: &str) -> Option<&'static ApiEntry> {
    API.iter().find(|e| e.table == table && e.ident == label)
}

/// Os esqueletos que a palavra que começa em `start` aceita: os ganchos ainda por declarar e a
/// `ph2d.property`, no topo e no início de uma instrução (ou só os ganchos, depois de `function `).
pub(crate) fn skeletons(text: &Text, syntax: &Syntax, start: usize) -> Vec<Item> {
    let ls = text.byte_of_line(text.line_of_byte(start));
    let lead = text.slice(ls..start);
    let lead = lead.trim_start();
    let after_function = match lead.strip_prefix("function") {
        Some(rest) => !rest.is_empty() && rest.trim().is_empty(),
        None => false,
    };
    if !(lead.is_empty() || after_function) || !syntax.at_top_level(text, start) {
        return Vec::new();
    }
    let mut out: Vec<Item> = HOOKS
        .iter()
        .filter(|h| !syntax.declares_global_function(text, h.ident))
        .map(|h| {
            item(
                h.ident,
                "code.complete.hook",
                h.doc,
                hook_snippet(h, !after_function),
            )
        })
        .collect();
    if !after_function && let Some(p) = api_entry("ph2d", "property") {
        out.push(item(
            "ph2d.property",
            "code.complete.skeleton",
            p.doc,
            format!("ph2d.{}", call_snippet(p)),
        ));
    }
    out
}

/// Os esqueletos da sessão de completação `s` (um membro, `ph2d.x`, tem o `ph2d.` antes: nenhum).
pub(crate) fn local_items(text: &Text, syntax: &Syntax, s: &crate::complete::Session) -> Vec<Item> {
    skeletons(text, syntax, s.anchor)
}

fn item(label: &str, detail: &str, doc: &str, insert: String) -> Item {
    Item {
        label: label.to_owned(),
        kind: Some(SNIPPET),
        detail: Some(ph2d_i18n::tr(detail).to_owned()),
        documentation: Some(doc.to_owned()),
        sort_text: "1".to_owned(),
        filter: None,
        insert,
        snippet: true,
        signature: false,
        usage: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_parametros_de_um_tipo_luau() {
        assert_eq!(
            params("(name: string, default: PropertyDefault, options: PropertyOptions?) -> ()"),
            [
                ("name", "string", false),
                ("default", "PropertyDefault", false),
                ("options", "PropertyOptions", true)
            ]
        );
        assert_eq!(
            params("(f: (number, string) -> (), t: { a: number }) -> ()"),
            [
                ("f", "(number, string) -> ()", false),
                ("t", "{ a: number }", false)
            ]
        );
        assert_eq!(params("() -> number"), []);
    }

    #[test]
    fn os_esqueletos_saem_da_tabela() {
        let u = HOOKS.iter().find(|h| h.ident == "update").unwrap();
        assert_eq!(
            hook_snippet(u, true),
            "function update(${1:self}: ScriptSelf, ${2:dt}: number)\n\t$0\nend"
        );
        assert_eq!(
            call_snippet(api_entry("ph2d", "property").unwrap()),
            "property(\"${1:name}\", ${2:default})$0"
        );
        assert_eq!(
            call_snippet(api_entry("ph2d", "set").unwrap()),
            "set(${1:entity}, \"${2:x}\", ${3:value})$0",
            "o campo é uma união de literais: entre aspas, com o 1.º"
        );
    }
}
