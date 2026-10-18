//! ⭐ **I14 — `--!strict` por ficheiro**: o `luau-lsp` já o lê por ficheiro (provado pelo caminho do
//! app em `oraculo_lsp::o_assistente_fixado_sublinha_o_que_se_escreve`); aqui o que se VÊ e se FAZ — a
//! barra de estado diz o modo, e o comando «Strict Types» põe ou tira a linha, num passo de undo.

use ph2d_code_buffer::Text;

/// A diretiva do modo estrito do Luau.
pub(crate) const STRICT: &str = "--!strict";

/// A linha da diretiva, se há: os comentários de topo (`--!` são as diretivas) antes do 1.º código.
pub(crate) fn directive_line(text: &Text) -> Option<usize> {
    for line in 0..text.line_count() {
        let l = text.slice(text.byte_of_line(line)..text.line_end(line));
        let l = l.trim();
        if l == STRICT {
            return Some(line);
        }
        if !(l.is_empty() || l.starts_with("--")) {
            return None;
        }
    }
    None
}

/// O script tem `--!strict`.
pub(crate) fn is_strict(text: &Text) -> bool {
    directive_line(text).is_some()
}

impl crate::doc::CodeDoc {
    /// Põe `--!strict` na 1.ª linha, ou tira a linha dele (um passo de undo; o cursor fica no código).
    pub(crate) fn toggle_strict(&mut self) {
        use ph2d_code_buffer::{Change, ChangeSet, EditKind};
        let t = self.buffer.text();
        let change = match directive_line(t) {
            Some(line) => Change {
                range: t.byte_of_line(line)..t.byte_of_line(line + 1).max(t.line_end(line)),
                insert: String::new(),
            },
            None => Change {
                range: 0..0,
                insert: format!("{STRICT}\n"),
            },
        };
        let Some(cs) = ChangeSet::new(vec![change]) else {
            return;
        };
        let edits = self.buffer.apply(cs, None, EditKind::Other);
        self.edited(&edits, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diretiva_so_conta_antes_do_codigo() {
        let s = |t: &str| is_strict(&Text::new(t));
        assert!(s("--!strict\nlocal x = 1\n"));
        assert!(s("-- bob\n\n--!native\n--!strict\nlocal x = 1\n"));
        assert!(!s("local x = 1\n--!strict\n"));
        assert!(!s("--!nonstrict\nlocal x = 1\n"));
        assert!(!s("-- --!strict\n"));
    }
}
