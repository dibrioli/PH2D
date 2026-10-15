//! Seleção multicursor: N faixas `anchor → head`, ordenadas e sem sobreposição, uma primária.

use crate::change::{Assoc, ChangeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub anchor: usize,
    pub head: usize,
    /// Coluna de ecrã que as setas ↑/↓ tentam manter (o «goal column»).
    pub goal: Option<usize>,
}

impl Range {
    pub fn cursor(at: usize) -> Self {
        Self { anchor: at, head: at, goal: None }
    }

    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head, goal: None }
    }

    pub fn from(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn to(&self) -> usize {
        self.anchor.max(self.head)
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn span(&self) -> std::ops::Range<usize> {
        self.from()..self.to()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    ranges: Vec<Range>,
    primary: usize,
}

impl Default for Selection {
    fn default() -> Self {
        Self::single(Range::cursor(0))
    }
}

impl Selection {
    pub fn single(r: Range) -> Self {
        Self { ranges: vec![r], primary: 0 }
    }

    /// Ordena e funde faixas que se tocam por dentro; a primária segue a faixa que a contém.
    pub fn new(ranges: Vec<Range>, primary: usize) -> Self {
        assert!(!ranges.is_empty(), "uma seleção tem pelo menos um cursor");
        let p = ranges[primary.min(ranges.len() - 1)];
        let mut sorted = ranges;
        sorted.sort_by_key(|r| (r.from(), r.to()));
        let mut out: Vec<Range> = Vec::with_capacity(sorted.len());
        for r in sorted {
            if let Some(last) = out.last_mut() {
                if r.from() < last.to() || r.span() == last.span() {
                    let (from, to) = (last.from().min(r.from()), last.to().max(r.to()));
                    let forward = last.head >= last.anchor;
                    *last = if forward { Range::new(from, to) } else { Range::new(to, from) };
                    continue;
                }
            }
            out.push(r);
        }
        let primary = out
            .iter()
            .position(|r| r.from() <= p.from() && p.to() <= r.to())
            .unwrap_or(out.len() - 1);
        Self { ranges: out, primary }
    }

    pub fn ranges(&self) -> &[Range] {
        &self.ranges
    }

    pub fn primary(&self) -> Range {
        self.ranges[self.primary]
    }

    pub fn primary_index(&self) -> usize {
        self.primary
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn map(&self, cs: &ChangeSet) -> Self {
        let ranges = self
            .ranges
            .iter()
            .map(|r| {
                let a = if r.is_empty() { Assoc::After } else if r.anchor < r.head { Assoc::After } else { Assoc::Before };
                let h = if r.is_empty() { Assoc::After } else if r.anchor < r.head { Assoc::Before } else { Assoc::After };
                Range::new(cs.map_pos(r.anchor, a), cs.map_pos(r.head, h))
            })
            .collect();
        Self::new(ranges, self.primary)
    }

    pub fn with_ranges(&self, ranges: Vec<Range>) -> Self {
        Self::new(ranges, self.primary)
    }

    pub fn push(&self, r: Range) -> Self {
        let mut ranges = self.ranges.clone();
        ranges.push(r);
        let n = ranges.len() - 1;
        Self::new(ranges, n)
    }

    pub fn only_primary(&self) -> Self {
        Self::single(self.primary())
    }
}
