//! As páginas NOSSAS, geradas da `api_table` (a fonte única do `ph2d.d.luau`) com o texto do
//! `api_help`: `ph2d`, `motion`, `types` (os tipos exportados) e `hooks` (o que o motor chama).

use crate::{Kind, Member, Page, Param};
use ph2d_i18n::tr;
use ph2d_script::api_help::api_help;
use ph2d_script::api_table::{API, API_GLOBALS, API_TYPES, HOOKS, luau_params};

pub(crate) const TYPES: &str = "types";
pub(crate) const HOOKS_PAGE: &str = "hooks";

/// `(a: T, b: U) -> R` (tipo de função) → `(a: T, b: U): R` (declaração, como o balão do `luau-lsp`);
/// `-> ()` não diz nada.
#[must_use]
pub(crate) fn declaration(luau_type: &str) -> String {
    let mut depth = 0usize;
    let bytes = luau_type.as_bytes();
    for (i, c) in luau_type.char_indices() {
        match c {
            '(' | '{' | '[' | '<' => depth += 1,
            ')' | '}' | ']' | '>' if !(c == '>' && i > 0 && bytes[i - 1] == b'-') => {
                depth = depth.saturating_sub(1);
            }
            '-' if depth == 0 && luau_type[i..].starts_with("->") => {
                let (args, ret) = (luau_type[..i].trim(), luau_type[i + 2..].trim());
                return if ret == "()" {
                    args.to_owned()
                } else {
                    format!("{args}: {ret}")
                };
            }
            _ => {}
        }
    }
    luau_type.to_owned()
}

fn member(
    key: &str,
    name: &'static str,
    kind: Kind,
    sig: String,
    doc: &str,
    luau_type: &str,
) -> Member {
    let help = api_help(key);
    let mut text = doc.to_owned();
    if let Some(h) = help.filter(|h| !h.more.is_empty()) {
        text.push_str("\n\n");
        text.push_str(h.more);
    }
    let params = luau_params(luau_type)
        .into_iter()
        .map(|(n, _)| Param {
            name: n.to_owned(),
            text: help
                .and_then(|h| h.params.iter().find(|(p, _)| *p == n))
                .map(|(_, t)| (*t).to_owned())
                .unwrap_or_default(),
        })
        .collect();
    Member {
        name,
        kind,
        sigs: vec![sig],
        text,
        params,
        returns: help
            .filter(|h| !h.returns.is_empty())
            .map(|h| h.returns.to_owned()),
        example: help.map(|h| h.example).filter(|e| !e.is_empty()),
    }
}

pub(crate) fn pages() -> Vec<Page> {
    let mut pages: Vec<Page> = API_GLOBALS
        .iter()
        .map(|g| Page {
            id: g.ident,
            title: g.ident.to_owned(),
            summary: g.doc.to_owned(),
            about: String::new(),
            members: API
                .iter()
                .filter(|e| e.table == g.ident)
                .map(|e| {
                    member(
                        &format!("{}.{}", e.table, e.ident),
                        e.ident,
                        Kind::Function,
                        declaration(e.luau_type),
                        e.doc,
                        e.luau_type,
                    )
                })
                .collect(),
        })
        .collect();
    for p in &mut pages {
        p.members.sort_by_key(|m| m.name);
    }
    let mut types: Vec<Member> = API_TYPES
        .iter()
        .map(|t| {
            member(
                &format!("type.{}", t.ident),
                t.ident,
                Kind::Type,
                t.definition.to_owned(),
                t.doc,
                "",
            )
        })
        .collect();
    types.sort_by_key(|m| m.name);
    pages.push(Page {
        id: TYPES,
        title: tr("code.help.types.title").to_owned(),
        summary: tr("code.help.types").to_owned(),
        about: tr("code.help.types.about").to_owned(),
        members: types,
    });
    pages.push(Page {
        id: HOOKS_PAGE,
        title: tr("code.help.hooks.title").to_owned(),
        summary: tr("code.help.hooks").to_owned(),
        about: tr("code.help.hooks.about").to_owned(),
        members: HOOKS
            .iter()
            .map(|h| {
                member(
                    &format!("hook.{}", h.ident),
                    h.ident,
                    Kind::Hook,
                    declaration(h.luau_type),
                    h.doc,
                    h.luau_type,
                )
            })
            .collect(),
    });
    pages
}

#[cfg(test)]
mod tests {
    use super::declaration;

    #[test]
    fn o_tipo_de_funcao_vira_declaracao() {
        assert_eq!(
            declaration("(n: number) -> number?"),
            "(n: number): number?"
        );
        assert_eq!(
            declaration("(entity: Entity, field: PoseField, value: number) -> ()"),
            "(entity: Entity, field: PoseField, value: number)"
        );
        assert_eq!(
            declaration("(f: (number) -> string) -> { string }"),
            "(f: (number) -> string): { string }"
        );
    }
}
