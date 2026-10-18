//! ⭐ **I14 — os sítios do script para a paleta**: `@` os símbolos (da árvore: funções, propriedades,
//! locais do topo) e `:` as linhas; a ida até eles põe o cursor no início do nome, como o VS Code.

use ph2d_code_buffer::{Range, Selection};
use ph2d_code_syntax::SymbolKind;
use ph2d_editor_core::doc_surface::{DocPlaces, DocSymbol, DocSymbolKind};

impl crate::docs::CodeDocs {
    /// Os sítios da aba `id` (a árvore exacta: a paleta abre-se raramente).
    pub(crate) fn tab_places(&mut self, id: u64) -> Option<DocPlaces> {
        let d = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let t = d.buffer.text();
        d.syntax.fresh(t);
        let symbols = d
            .syntax
            .symbols(t)
            .into_iter()
            .map(|s| {
                let line = t.line_of_byte(s.byte);
                DocSymbol {
                    name: s.name,
                    container: None,
                    kind: match s.kind {
                        SymbolKind::Function => DocSymbolKind::Function,
                        SymbolKind::Property => DocSymbolKind::Property,
                        SymbolKind::Variable => DocSymbolKind::Variable,
                    },
                    line,
                    col: t.slice(t.byte_of_line(line)..s.byte).chars().count(),
                }
            })
            .collect();
        Some(DocPlaces {
            symbols,
            lines: t.line_count(),
        })
    }

    /// O cursor da aba `id` na linha e coluna (em caracteres, como o `:40:5`; presa ao fim da linha).
    pub(crate) fn tab_go_to(&mut self, id: u64, line: usize, col: usize) -> bool {
        let Some(d) = self.docs.iter_mut().find(|d| d.script.0 == id) else {
            return false;
        };
        let t = d.buffer.text();
        let line = line.min(t.line_count().saturating_sub(1));
        let start = t.byte_of_line(line);
        let text = t.slice(start..t.line_end(line));
        let at = start + text.char_indices().nth(col).map_or(text.len(), |(b, _)| b);
        d.buffer.set_selection(Selection::single(Range::cursor(at)));
        d.buffer.seal_history();
        d.reveal();
        true
    }
}
