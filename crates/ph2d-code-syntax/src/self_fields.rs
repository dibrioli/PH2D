//! O `self` dos ganchos do motor (CodeEditor Onda 2, I7): que campos o script DÁ ao `self` e que campos
//! LÊ. O `luau-lsp` infere o `self` pelo uso e aceita qualquer campo, nem em `--!strict` acusa um
//! (`docs/CodeEditor/ferramentas/medicoes_onda2/I7_acoes_e_voce_quis_dizer.md` §3): o campo lido que
//! ninguém dá é um lint NOSSO.

use crate::Syntax;
use ph2d_code_buffer::Text;
use std::ops::Range;
use tree_sitter::Node;

/// Os campos do `self` dos ganchos `function <gancho>(self, …)` do topo.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SelfFields {
    /// Os nomes dados (`self.x = …`, `self.x += …`, `self["x"] = …`, `function self.x()`), por ordem.
    pub assigned: Vec<String>,
    /// Cada leitura `self.<nome>`: o nome e os bytes dele.
    pub reads: Vec<(String, Range<usize>)>,
}

impl SelfFields {
    fn give(&mut self, name: String) {
        if !self.assigned.contains(&name) {
            self.assigned.push(name);
        }
    }
}

impl Syntax {
    /// Os campos do `self` dos `hooks` declarados globais no topo. `None` = não se sabe: o texto tem um
    /// erro de sintaxe, ou o `self` FOGE (passado a uma função, guardado, devolvido, indexado por uma
    /// chave que não é um literal, redeclarado) — qualquer campo pode então ser dado noutro sítio.
    #[must_use]
    pub fn self_fields(&self, text: &Text, hooks: &[&str]) -> Option<SelfFields> {
        let root = self.tree().root_node();
        if root.has_error() {
            return None;
        }
        let mut out = SelfFields::default();
        for i in 0..root.child_count() {
            let Some(f) = root.child(i as u32) else {
                continue;
            };
            let local = root.field_name_for_child(i as u32) == Some("local_declaration");
            let hook = f.kind() == "function_declaration"
                && !local
                && f.child_by_field_name("name").is_some_and(|n| {
                    n.kind() == "identifier" && hooks.contains(&&*text.slice(n.byte_range()))
                });
            if !hook || !first_param_is_self(f, text) {
                continue;
            }
            if let Some(body) = f.child_by_field_name("body") {
                walk(body, text, &mut out)?;
            }
        }
        Some(out)
    }
}

fn is_self(n: Node, text: &Text) -> bool {
    n.kind() == "identifier" && text.slice(n.byte_range()) == "self"
}

fn params(f: Node) -> Vec<Node> {
    let Some(ps) = f.child_by_field_name("parameters") else {
        return Vec::new();
    };
    let mut c = ps.walk();
    ps.named_children(&mut c)
        .filter_map(|p| p.named_child(0))
        .collect()
}

fn first_param_is_self(f: Node, text: &Text) -> bool {
    params(f).first().is_some_and(|p| is_self(*p, text))
}

fn walk(n: Node, text: &Text, out: &mut SelfFields) -> Option<()> {
    if matches!(n.kind(), "function_definition" | "function_declaration") {
        let name = n.child_by_field_name("name");
        // `function self.x()` dá o campo; `function self:x()` também, e o `self` de dentro é OUTRO.
        let method = name.is_some_and(|m| m.kind() == "method_index_expression");
        if let Some(m) = name
            && m.child_by_field_name("table")
                .is_some_and(|t| is_self(t, text))
        {
            let field = m.child_by_field_name(if method { "method" } else { "field" })?;
            out.give(text.slice(field.byte_range()));
        }
        if method || params(n).iter().any(|p| is_self(*p, text)) {
            return Some(());
        }
        return match n.child_by_field_name("body") {
            Some(b) => walk(b, text, out),
            None => Some(()),
        };
    }
    if is_self(n, text) {
        return use_of_self(n, text, out);
    }
    let mut c = n.walk();
    for ch in n.named_children(&mut c).collect::<Vec<_>>() {
        walk(ch, text, out)?;
    }
    Some(())
}

/// O índice `p` é um ALVO de `=` ou `+=` (`self.x = …`).
fn is_target(p: Node) -> bool {
    p.parent().is_some_and(|l| {
        l.kind() == "variable_list"
            && l.parent()
                .is_some_and(|s| matches!(s.kind(), "assignment_statement" | "update_statement"))
    })
}

/// Um uso do identificador `self`: só indexado por um nome ou por um literal ele não foge.
fn use_of_self(n: Node, text: &Text, out: &mut SelfFields) -> Option<()> {
    let p = n.parent()?;
    if p.child_by_field_name("table") != Some(n) {
        return None;
    }
    match p.kind() {
        "dot_index_expression" => {
            let f = p.child_by_field_name("field")?;
            let name = text.slice(f.byte_range());
            if is_target(p) {
                out.give(name);
            } else {
                out.reads.push((name, f.byte_range()));
            }
            Some(())
        }
        "bracket_index_expression" => {
            let f = p
                .child_by_field_name("field")
                .filter(|f| f.kind() == "string")?;
            let content = f.child_by_field_name("content")?;
            if is_target(p) {
                out.give(text.slice(content.byte_range()));
            }
            Some(())
        }
        // `self:m()`: o método pode vir de uma metatable — nem leitura nem fuga.
        "method_index_expression" => Some(()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOOKS: &[&str] = &["init", "update", "on_signal"];

    fn fields(src: &str) -> Option<SelfFields> {
        let t = Text::new(src);
        Syntax::new(&t).self_fields(&t, HOOKS)
    }

    fn reads(f: &SelfFields) -> Vec<&str> {
        f.reads.iter().map(|(n, _)| n.as_str()).collect()
    }

    #[test]
    fn os_campos_dados_e_lidos_dos_ganchos() {
        let src = "function init(self)\n  self.t = 0\n  self[\"k\"] = 1\nend\nfunction update(self, dt)\n  self.t += dt\n  self.a, self.b = self.amplitude, self.tint.r\n  local f = function() return self.w end\n  local g = function(self) return self.z end\n  function self.cb() end\n  self:metodo()\nend\n";
        let f = fields(src).unwrap();
        assert_eq!(f.assigned, ["t", "k", "a", "b", "cb"]);
        assert_eq!(reads(&f), ["amplitude", "tint", "w"]);
        let (_, r) = &f.reads[0];
        assert_eq!(&src[r.clone()], "amplitude");
    }

    #[test]
    fn so_os_ganchos_globais_com_self() {
        let f = fields("local function update(self) return self.a end\nfunction outra(self) return self.b end\nfunction init(me) return me.c end\nfunction M:h() return self.d end\nprint(self.e)\n").unwrap();
        assert_eq!(f, SelfFields::default());
    }

    #[test]
    fn um_self_que_foge_nao_se_sabe() {
        for corpo in [
            "ajuda(self)",
            "local s = self",
            "return self",
            "self[k] = 1",
            "local self = {}",
            "print(self == nil)",
            "for self = 1, 2 do end",
        ] {
            let src = format!("function update(self)\n  {corpo}\n  print(self.x)\nend\n");
            assert_eq!(fields(&src), None, "{corpo}");
        }
        assert_eq!(fields("function update(self)\n  print(self.x\nend\n"), None);
    }
}
