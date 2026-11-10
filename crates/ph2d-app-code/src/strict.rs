//! ⭐ **I14 — `--!strict` por ficheiro**: o `luau-lsp` já o lê por ficheiro (provado pelo caminho do
//! app em `oraculo_lsp::o_assistente_fixado_sublinha_o_que_se_escreve`); aqui o que se VÊ e se FAZ — a
//! barra de estado diz o modo, e o comando «Strict Types» põe ou tira a linha, num passo de undo.

pub(crate) use ph2d_code_assist::strict::{STRICT, directive_line, is_strict};

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
