//! ⭐ **«Make 220 a property»** (CodeEditor 13, Onda C item 16): um número no corpo de uma função com
//! `self` vira um campo do Inspector — `ph2d.property("<nome>", 220)` no topo e `self.<nome>` no lugar
//! do número. A lei pura; a aba aplica-a num passo de undo e abre o F2 sobre o nome.

use crate::Syntax;
use crate::names::is_identifier;
use ph2d_code_buffer::Text;
use std::ops::Range;
use tree_sitter::Node;

/// As duas trocas, por ordem do texto, e onde fica o nome novo depois delas (o `self.<nome>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promotion {
    pub name: String,
    pub edits: Vec<(Range<usize>, String)>,
    /// O byte do INÍCIO do nome em `self.<nome>`, no texto DEPOIS das trocas.
    pub name_at: usize,
}

fn is_self(n: Node, text: &Text) -> bool {
    n.kind() == "identifier" && text.slice(n.byte_range()) == "self"
}

/// A função tem `self`: o 1.º parâmetro, ou um método (`function M:f()`).
fn has_self(f: Node, text: &Text) -> bool {
    if f.child_by_field_name("name")
        .is_some_and(|m| m.kind() == "method_index_expression")
    {
        return true;
    }
    let Some(ps) = f.child_by_field_name("parameters") else {
        return false;
    };
    let mut c = ps.walk();
    let first = ps.named_children(&mut c).next();
    first.is_some_and(|p| is_self(p, text) || p.named_child(0).is_some_and(|i| is_self(i, text)))
}

/// O nome que o contexto dá: o alvo da atribuição (`local speed = 220` → `speed`, `self.x = 220` →
/// `x`); sem alvo, `value`. O artista escreve o verdadeiro no F2 que se abre logo a seguir.
fn context_name(n: Node, text: &Text) -> String {
    let mut p = n.parent();
    while let Some(s) = p {
        if matches!(s.kind(), "function_declaration" | "function_definition") {
            break;
        }
        if s.kind() == "assignment_statement"
            && let Some(list) = s.named_child(0).filter(|l| l.kind() == "variable_list")
            && let Some(target) = list.named_child(0)
        {
            let leaf = match target.kind() {
                "dot_index_expression" => target.child_by_field_name("field"),
                "identifier" => Some(target),
                _ => None,
            };
            if let Some(name) = leaf
                .map(|l| text.slice(l.byte_range()))
                .filter(|s| is_identifier(s))
            {
                return name;
            }
        }
        p = s.parent();
    }
    "value".to_owned()
}

impl Syntax {
    /// O número sob `byte` (ou que acaba nele) a promover a propriedade, se o contexto o deixa: dentro
    /// de uma função com `self`, fora de uma chamada `ph2d.property(…)`.
    #[must_use]
    pub fn promote_number(&self, text: &Text, byte: usize) -> Option<Promotion> {
        let byte = byte.min(text.len());
        let root = self.tree().root_node();
        let at = |b: usize| {
            root.descendant_for_byte_range(b, b)
                .filter(|n| n.kind() == "number")
        };
        let num = at(byte).or_else(|| at(byte.checked_sub(1)?))?;
        let mut func = None;
        let mut p = num.parent();
        while let Some(s) = p {
            if s.kind() == "function_call"
                && s.child_by_field_name("name")
                    .is_some_and(|m| text.slice(m.byte_range()) == "ph2d.property")
            {
                return None;
            }
            if func.is_none() && matches!(s.kind(), "function_declaration" | "function_definition")
            {
                func = Some(s);
            }
            p = s.parent();
        }
        if !has_self(func?, text) {
            return None;
        }
        let declared = self.property_declarations(text);
        let base = context_name(num, text);
        let taken = |n: &str| {
            declared.iter().any(|(d, _)| d == n) || text.to_string().contains(&format!("self.{n}"))
        };
        let name = std::iter::once(base.clone())
            .chain((2..).map(|i| format!("{base}_{i}")))
            .find(|n| !taken(n))?;
        let decl_at = self.property_insert_byte(text);
        let decl = format!(
            "ph2d.property({}, {})\n",
            crate::dropping::luau_string(&name),
            text.slice(num.byte_range())
        );
        let field = format!("self.{name}");
        let name_at = num.start_byte() + decl.len() + "self.".len();
        Some(Promotion {
            name,
            edits: vec![(decl_at..decl_at, decl), (num.byte_range(), field)],
            name_at,
        })
    }

    /// Onde entra uma `ph2d.property` nova: a linha a seguir à última do topo; sem nenhuma, depois dos
    /// comentários do início.
    fn property_insert_byte(&self, text: &Text) -> usize {
        if let Some((_, r)) = self.property_declarations(text).last() {
            return text.byte_of_line(text.line_of_byte(r.start) + 1);
        }
        let line = (0..text.line_count())
            .find(|&l| !text.line_string(l).trim_start().starts_with("--"))
            .unwrap_or(text.line_count());
        text.byte_of_line(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn promote(src: &str, needle: &str) -> Option<(String, String)> {
        let t = Text::new(src);
        let s = Syntax::new(&t);
        let p = s.promote_number(&t, src.find(needle)?)?;
        let mut out = src.to_owned();
        for (r, ins) in p.edits.iter().rev() {
            out.replace_range(r.clone(), ins);
        }
        assert_eq!(&out[p.name_at..p.name_at + p.name.len()], p.name);
        Some((p.name, out))
    }

    #[test]
    fn o_numero_no_corpo_vira_propriedade_e_campo_do_self() {
        let src = "-- door\nph2d.property(\"a\", 1)\nfunction update(self, dt)\n  ph2d.set(self.id, \"x\", x + 220 * dt)\nend\n";
        let (name, out) = promote(src, "220").expect("oferecida");
        assert_eq!(name, "value");
        assert_eq!(
            out,
            "-- door\nph2d.property(\"a\", 1)\nph2d.property(\"value\", 220)\nfunction update(self, dt)\n  ph2d.set(self.id, \"x\", x + self.value * dt)\nend\n"
        );
        let src = "-- a\n-- b\nfunction update(self, dt)\n  local speed = 2.5\nend\n";
        let (name, out) = promote(src, "2.5").expect("oferecida");
        assert_eq!(name, "speed");
        assert!(
            out.starts_with("-- a\n-- b\nph2d.property(\"speed\", 2.5)\nfunction"),
            "{out}"
        );
    }

    #[test]
    fn nao_no_topo_nem_dentro_de_ph2d_property_nem_sem_self() {
        let top = "local n = 220\nfunction update(self)\nend\n";
        assert_eq!(promote(top, "220"), None);
        let decl = "ph2d.property(\"speed\", 220)\nfunction update(self)\nend\n";
        assert_eq!(promote(decl, "220"), None);
        let no_self = "function f(a)\n  return a + 220\nend\n";
        assert_eq!(promote(no_self, "220"), None);
        let nome = "function update(self)\n  local x220 = 1\nend\n";
        assert_eq!(promote(nome, "220"), None);
    }
}
