//! O par de um parêntese, colchete ou chaveta — pela árvore, então `"("` dentro de string não conta.

use crate::Syntax;
use ph2d_code_buffer::Text;

const PAIRS: [(&str, &str); 3] = [("(", ")"), ("[", "]"), ("{", "}")];

impl Syntax {
    /// `byte` é o início do token-parêntese; devolve o início do seu par. `None` se ali não há
    /// parêntese, ou se o par está em FALTA (texto meio escrito).
    pub fn matching_bracket(&self, text: &Text, byte: usize) -> Option<usize> {
        if byte >= text.len() {
            return None;
        }
        let leaf = self
            .tree()
            .root_node()
            .descendant_for_byte_range(byte, byte + 1)?;
        if leaf.start_byte() != byte || leaf.child_count() != 0 || leaf.is_missing() {
            return None;
        }
        let kind = leaf.kind();
        let parent = leaf.parent()?;
        let mut cur = parent.walk();
        let siblings: Vec<_> = parent.children(&mut cur).collect();
        let at = siblings.iter().position(|s| s.id() == leaf.id())?;
        let partner = |want: &str, forward: bool| {
            let found = if forward {
                siblings[at + 1..].iter().find(|s| s.kind() == want)
            } else {
                siblings[..at].iter().rev().find(|s| s.kind() == want)
            };
            found.filter(|s| !s.is_missing()).map(|s| s.start_byte())
        };
        for (open, close) in PAIRS {
            if kind == open {
                return partner(close, true);
            }
            if kind == close {
                return partner(open, false);
            }
        }
        None
    }
}
