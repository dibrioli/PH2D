//! A árvore que segue o texto: parse inteiro uma vez, depois reparse INCREMENTAL por edição.
//!
//! Lei medida (`tests/it/incremental.rs`): com o texto VÁLIDO a árvore incremental é igual à do
//! parse do zero (forma e faixas). Com ERROR, a recuperação de erro do tree-sitter depende da
//! história de edições: ambas veem o erro, mas a forma dentro dele pode diferir (medido: 0 a 17
//! passos em 600 de traços aleatórios longos), e a diferença nunca sobrevive à volta a um texto
//! válido. Re-parse do zero quando há erro foi RECUSADO: 17 ms contra 1,9 ms por tecla a 7 500
//! linhas (`reparse_por_tecla`), e texto quebrado é o estado normal de quem escreve.

use crate::queries;
use ph2d_code_buffer::{Point, Text, TextEdit};
use tree_sitter::{InputEdit, Parser, Tree};

pub struct Syntax {
    parser: Parser,
    tree: Tree,
}

fn ts_point(p: Point) -> tree_sitter::Point {
    tree_sitter::Point {
        row: p.row,
        column: p.col,
    }
}

/// Lê a rope por pedaços: o tree-sitter pede «o texto a partir do byte b», a rope dá o resto do
/// pedaço onde `b` cai — nunca uma `String` do texto inteiro.
fn parse(parser: &mut Parser, text: &Text, old: Option<&Tree>) -> Tree {
    let rope = text.rope();
    let len = rope.byte_len();
    let mut read = |byte: usize, _: tree_sitter::Point| -> &[u8] {
        if byte >= len {
            return &[];
        }
        rope.byte_slice(byte..len)
            .chunks()
            .next()
            .map_or(&[][..], str::as_bytes)
    };
    // `None` só com cancelamento ou sem linguagem; nenhum dos dois existe aqui (texto quebrado
    // dá árvore com ERROR, nunca `None`).
    parser
        .parse_with_options(&mut read, old, None)
        .expect("tree-sitter sem cancelamento devolve sempre árvore")
}

impl Syntax {
    pub fn new(text: &Text) -> Self {
        let mut parser = Parser::new();
        parser.set_language(&queries::get().language).expect(
            "ABI da gramática Luau fora da faixa do tree-sitter (teste `abi_da_gramatica`)",
        );
        let tree = parse(&mut parser, text, None);
        Self { parser, tree }
    }

    /// Aplica as edições à árvore (na ordem dada: a do `Buffer`, do fim para o início) e reparse
    /// incremental contra `text_after`, o texto JÁ editado.
    pub fn edit(&mut self, edits: &[TextEdit], text_after: &Text) {
        if edits.is_empty() {
            return;
        }
        for e in edits {
            self.tree.edit(&InputEdit {
                start_byte: e.start_byte,
                old_end_byte: e.old_end_byte,
                new_end_byte: e.new_end_byte,
                start_position: ts_point(e.start),
                old_end_position: ts_point(e.old_end),
                new_end_position: ts_point(e.new_end),
            });
        }
        self.tree = parse(&mut self.parser, text_after, Some(&self.tree));
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn sexp(&self) -> String {
        self.tree.root_node().to_sexp()
    }
}
