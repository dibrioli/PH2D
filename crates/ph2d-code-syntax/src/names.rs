//! Os NOMES do script para renomear e realçar (CodeEditor Onda 2, I10): o que está sob o cursor, se um
//! nome novo é válido e se já existe. O `luau-lsp` aceita palavras reservadas e nomes que já existem, e
//! morre sobre um membro (`docs/CodeEditor/ferramentas/medicoes_onda2/I10_rename_e_undo.md` §1): estas
//! perguntas são nossas, antes de perguntar ao servidor.

use crate::Syntax;
use ph2d_code_buffer::Text;
use std::ops::Range;
use tree_sitter::Node;

/// As palavras reservadas do Luau (`continue`, `type` e `export` são contextuais: são nomes).
pub const LUAU_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

/// Um nome que se pode escrever onde está `old`: identificador ASCII do Luau, não reservado.
#[must_use]
pub fn is_identifier(s: &str) -> bool {
    let mut c = s.chars();
    c.next()
        .is_some_and(|f| f.is_ascii_alphabetic() || f == '_')
        && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
        && !LUAU_KEYWORDS.contains(&s)
}

/// O que está sob o cursor, para renomear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameAt {
    /// Um nome do código (local, parâmetro, função): é o servidor que o segue.
    Name(Range<usize>),
    /// Um campo do `self` (`self.x`, `self["x"]`) ou o nome de uma `ph2d.property("x")` do topo.
    SelfField(String, Range<usize>),
    /// O nome de um gancho do motor declarado no topo (`function update`): o motor chama-o por ele.
    Hook(Range<usize>),
    /// O membro de outra tabela (`ph2d.set`, `math.sin`, `v.x`).
    Member(Range<usize>),
}

impl Syntax {
    /// O nome em `byte` (dentro dele ou logo a seguir ao fim). `None` = nenhum nome ali.
    #[must_use]
    pub fn name_at(&self, text: &Text, byte: usize, hooks: &[&str]) -> Option<NameAt> {
        let root = self.tree().root_node();
        let at = |b: usize| {
            let n = root.descendant_for_byte_range(b, b)?;
            matches!(n.kind(), "identifier" | "string_content").then_some(n)
        };
        let n = at(byte).or_else(|| at(byte.checked_sub(1)?))?;
        let r = n.byte_range();
        let p = n.parent()?;
        if n.kind() == "string_content" {
            let s = p.parent()?;
            let self_key = s.kind() == "bracket_index_expression"
                && s.child_by_field_name("table")
                    .is_some_and(|t| is_self(t, text))
                && s.child_by_field_name("field") == Some(p);
            let decl = self
                .property_declarations(text)
                .iter()
                .any(|(_, d)| *d == r);
            return (self_key || decl).then(|| NameAt::SelfField(text.slice(r.clone()), r));
        }
        let member = match p.kind() {
            "dot_index_expression" => p.child_by_field_name("field") == Some(n),
            "method_index_expression" => p.child_by_field_name("method") == Some(n),
            _ => false,
        };
        if member {
            let table = p.child_by_field_name("table")?;
            return Some(
                if is_self(table, text) && p.kind() == "dot_index_expression" {
                    NameAt::SelfField(text.slice(r.clone()), r)
                } else {
                    NameAt::Member(r)
                },
            );
        }
        let word = text.slice(r.clone());
        if hooks.contains(&word.as_str()) && self.declares_global_function(text, &word) {
            return Some(NameAt::Hook(r));
        }
        Some(NameAt::Name(r))
    }

    /// Um nome do código (não um membro, não um campo) com este texto existe no script.
    #[must_use]
    pub fn name_in_use(&self, text: &Text, name: &str) -> bool {
        fn walk(n: Node, text: &Text, name: &str) -> bool {
            if n.kind() == "identifier" && text.slice(n.byte_range()) == name {
                let member = n.parent().is_some_and(|p| {
                    (p.kind() == "dot_index_expression"
                        && p.child_by_field_name("field") == Some(n))
                        || (p.kind() == "method_index_expression"
                            && p.child_by_field_name("method") == Some(n))
                });
                if !member {
                    return true;
                }
            }
            let mut c = n.walk();
            n.named_children(&mut c).any(|ch| walk(ch, text, name))
        }
        walk(self.tree().root_node(), text, name)
    }

    /// O script declara a função GLOBAL `name` no topo (um gancho, se `name` é um).
    #[must_use]
    pub fn declares_global_function(&self, text: &Text, name: &str) -> bool {
        let root = self.tree().root_node();
        (0..root.child_count()).any(|i| {
            root.field_name_for_child(i) != Some("local_declaration")
                && root.child(i).is_some_and(|f| {
                    f.kind() == "function_declaration"
                        && f.child_by_field_name("name").is_some_and(|n| {
                            n.kind() == "identifier" && text.slice(n.byte_range()) == name
                        })
                })
        })
    }
}

fn is_self(n: Node, text: &Text) -> bool {
    n.kind() == "identifier" && text.slice(n.byte_range()) == "self"
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOOKS: &[&str] = &["init", "update", "on_signal"];

    fn at(src: &str, needle: &str, off: usize) -> Option<NameAt> {
        let t = Text::new(src);
        Syntax::new(&t).name_at(&t, src.find(needle).unwrap() + off, HOOKS)
    }

    #[test]
    fn o_que_esta_sob_o_cursor() {
        let src = "ph2d.property(\"amplitude\", 1)\nlocal function ajuda(v) return v end\nfunction update(self, dt)\n  local s = self.amplitude + self[\"amplitude\"]\n  ph2d.set(self.id, \"x\", math.sin(s))\n  self:metodo()\nend\n";
        let f = |n: &str| src.find(n).unwrap();
        assert_eq!(
            at(src, "local s", 6),
            Some(NameAt::Name(f("s = ")..f("s = ") + 1))
        );
        assert_eq!(
            at(src, "local s", 7),
            Some(NameAt::Name(f("s = ")..f("s = ") + 1)),
            "logo a seguir ao fim"
        );
        assert_eq!(
            at(src, "dt)", 0),
            Some(NameAt::Name(f("dt)")..f("dt)") + 2))
        );
        let a = f("self.amplitude") + 5;
        assert_eq!(
            at(src, "self.amplitude", 7),
            Some(NameAt::SelfField("amplitude".into(), a..a + 9))
        );
        let b = f("self[\"amplitude\"]") + 6;
        assert_eq!(
            at(src, "self[\"amplitude\"]", 8),
            Some(NameAt::SelfField("amplitude".into(), b..b + 9))
        );
        assert_eq!(
            at(src, "\"amplitude\", 1", 3),
            Some(NameAt::SelfField("amplitude".into(), 15..24))
        );
        assert!(matches!(at(src, "ph2d.set", 6), Some(NameAt::Member(_))));
        assert!(matches!(at(src, "math.sin", 6), Some(NameAt::Member(_))));
        assert!(
            matches!(at(src, "self:metodo", 7), Some(NameAt::Member(_))),
            "o método não é um campo"
        );
        assert!(matches!(
            at(src, "function update", 10),
            Some(NameAt::Hook(_))
        ));
        assert!(
            matches!(at(src, "ajuda(v)", 1), Some(NameAt::Name(_))),
            "local function não é gancho"
        );
        assert_eq!(
            at(src, "\"x\"", 1),
            None,
            "uma string qualquer não é um nome"
        );
        assert_eq!(at(src, "return v", 2), None, "palavra reservada");
    }

    #[test]
    fn nomes_validos_e_em_uso() {
        for ok in ["a", "_x", "speed2", "continue", "type"] {
            assert!(is_identifier(ok), "{ok}");
        }
        for bad in ["", "1x", "x-y", "end", "nil", "função", "a b"] {
            assert!(!is_identifier(bad), "{bad}");
        }
        let t = Text::new("local brilho = 1\nprint(v.cor, m:pinta())\n");
        let s = Syntax::new(&t);
        assert!(s.name_in_use(&t, "brilho"));
        assert!(s.name_in_use(&t, "print"));
        assert!(
            !s.name_in_use(&t, "cor"),
            "um membro não é um nome do script"
        );
        assert!(!s.name_in_use(&t, "pinta"));
        assert!(!s.name_in_use(&t, "outro"));
    }
}
