//! `ChangeSet`: trocas SIMULTÂNEAS sobre o mesmo texto de partida, invertíveis.
//!
//! Lei: as faixas estão em coordenadas do texto ANTES da troca, ordenadas e sem sobreposição.
//! Aplicar vai do fim para o início, para que cada faixa ainda aponte o texto de partida.

use crate::text::{Point, Text};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub range: Range<usize>,
    pub insert: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChangeSet {
    changes: Vec<Change>,
}

/// Para que lado vai uma posição que cai exactamente num ponto de inserção.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc {
    Before,
    After,
}

/// A edição no formato do `tree_sitter::InputEdit` (bytes + pontos em bytes), em ordem de
/// aplicação SEQUENCIAL: do fim para o início, então os pontos de partida valem tal como estão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextEdit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start: Point,
    pub old_end: Point,
    pub new_end: Point,
}

impl ChangeSet {
    /// Ordena e funde inserções no mesmo ponto; `None` se duas faixas se sobrepõem.
    pub fn new(mut changes: Vec<Change>) -> Option<Self> {
        changes.retain(|c| !(c.range.is_empty() && c.insert.is_empty()));
        changes.sort_by_key(|c| (c.range.start, c.range.end));
        let mut out: Vec<Change> = Vec::with_capacity(changes.len());
        for c in changes {
            if let Some(last) = out.last_mut() {
                if c.range.start < last.range.end {
                    return None;
                }
                if c.range.start == last.range.end && c.range.is_empty() && last.range.is_empty() {
                    last.insert.push_str(&c.insert);
                    continue;
                }
            }
            out.push(c);
        }
        Some(Self { changes: out })
    }

    pub fn single(range: Range<usize>, insert: impl Into<String>) -> Self {
        Self::new(vec![Change {
            range,
            insert: insert.into(),
        }])
        .unwrap_or_default()
    }

    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// As edições para o tree-sitter, calculadas sobre o texto de PARTIDA.
    pub fn text_edits(&self, before: &Text) -> Vec<TextEdit> {
        self.changes
            .iter()
            .rev()
            .map(|c| {
                let start = before.point_of_byte(c.range.start);
                let old_end = before.point_of_byte(c.range.end);
                let new_end = match c.insert.rfind('\n') {
                    Some(i) => Point {
                        row: start.row + c.insert.matches('\n').count(),
                        col: c.insert.len() - i - 1,
                    },
                    None => Point {
                        row: start.row,
                        col: start.col + c.insert.len(),
                    },
                };
                TextEdit {
                    start_byte: c.range.start,
                    old_end_byte: c.range.end,
                    new_end_byte: c.range.start + c.insert.len(),
                    start,
                    old_end,
                    new_end,
                }
            })
            .collect()
    }

    pub(crate) fn apply(&self, text: &mut Text) {
        for c in self.changes.iter().rev() {
            text.replace(c.range.clone(), &c.insert);
        }
    }

    /// A troca que desfaz esta, em coordenadas do texto DEPOIS dela.
    pub fn invert(&self, before: &Text) -> ChangeSet {
        let mut delta: isize = 0;
        let changes = self
            .changes
            .iter()
            .map(|c| {
                let start = (c.range.start as isize + delta) as usize;
                delta += c.insert.len() as isize - c.range.len() as isize;
                Change {
                    range: start..start + c.insert.len(),
                    insert: before.slice(c.range.clone()),
                }
            })
            .collect();
        ChangeSet { changes }
    }

    /// Onde fica, depois da troca, a posição `pos` do texto de partida.
    pub fn map_pos(&self, pos: usize, assoc: Assoc) -> usize {
        let mut delta: isize = 0;
        for c in &self.changes {
            if pos < c.range.start
                || (pos == c.range.start && !c.range.is_empty() && assoc == Assoc::Before)
            {
                break;
            }
            if pos > c.range.end
                || (pos == c.range.end && !c.range.is_empty() && assoc == Assoc::After)
            {
                delta += c.insert.len() as isize - c.range.len() as isize;
                continue;
            }
            let start = (c.range.start as isize + delta) as usize;
            return match assoc {
                Assoc::Before => start,
                Assoc::After => start + c.insert.len(),
            };
        }
        (pos as isize + delta) as usize
    }
}
