//! O que a família pergunta à árvore sobre o script (CodeEditor Onda 2, I4): as propriedades
//! `ph2d.property` declaradas no topo (o `self.` da lista de completação — o `luau-lsp` não as liga ao
//! `self`) e se um byte está em CÓDIGO (a lista não abre a escrever numa string nem num comentário).

use crate::{HighlightKind, Syntax};
use ph2d_code_buffer::Text;

impl Syntax {
    /// Os nomes das `ph2d.property("<nome>", …)` do topo, pela ordem.
    #[must_use]
    pub fn declared_properties(&self, text: &Text) -> Vec<String> {
        let root = self.tree().root_node();
        let mut c = root.walk();
        root.named_children(&mut c)
            .filter(|n| n.kind() == "function_call")
            .filter_map(|n| {
                let name = n.child_by_field_name("name")?;
                if text.slice(name.byte_range()) != "ph2d.property" {
                    return None;
                }
                let args = n.child_by_field_name("arguments")?;
                let first = args.named_child(0).filter(|a| a.kind() == "string")?;
                let content = first.child_by_field_name("content")?;
                Some(text.slice(content.byte_range()))
            })
            .collect()
    }

    /// O carácter antes de `byte` é código (nem string nem comentário).
    #[must_use]
    pub fn in_code(&self, text: &Text, byte: usize) -> bool {
        let byte = byte.min(text.len());
        if byte == 0 {
            return true;
        }
        let prev = text.prev_grapheme(byte);
        !self
            .highlights(text, prev..byte)
            .iter()
            .any(|h| matches!(h.kind, HighlightKind::String | HighlightKind::Comment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_propriedades_do_topo_e_so_elas() {
        let t = Text::new(
            "ph2d.property(\"amplitude\", 1.0)\nph2d.property('speed', 2)\nlocal x = ph2d.property(\"nao\", 1)\nfunction init(self)\n  ph2d.property(\"dentro\", 1)\nend\n",
        );
        let s = Syntax::new(&t);
        assert_eq!(s.declared_properties(&t), ["amplitude", "speed"]);
    }

    #[test]
    fn strings_e_comentarios_nao_sao_codigo() {
        let t = Text::new("local a = \"ab\" -- cd\nlocal b\n");
        let s = Syntax::new(&t);
        let at = |p: &str| t.slice(0..t.len()).find(p).unwrap() + p.len();
        assert!(s.in_code(&t, at("local a")));
        assert!(!s.in_code(&t, at("\"ab")));
        assert!(!s.in_code(&t, at("-- cd")));
        assert!(s.in_code(&t, at("local b")));
    }
}
