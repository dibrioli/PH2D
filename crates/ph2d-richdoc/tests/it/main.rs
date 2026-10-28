mod edit;
mod inline;
mod shortcuts;

use ph2d_richdoc::{Block, BlockKind, Caret, RichDoc, Selection};

pub fn doc(blocks: &[(BlockKind, &str)]) -> RichDoc {
    RichDoc::from_blocks(
        blocks
            .iter()
            .map(|(k, t)| Block::paragraph(t).kind(k))
            .collect(),
    )
}

pub trait KindExt {
    fn kind(self, k: &BlockKind) -> Block;
}

impl KindExt for Block {
    fn kind(mut self, k: &BlockKind) -> Block {
        self.kind = k.clone();
        self
    }
}

pub fn at(block: usize, byte: usize) -> Selection {
    Selection::caret(Caret::new(block, byte))
}

pub fn texts(d: &RichDoc) -> Vec<(BlockKind, String)> {
    d.blocks()
        .iter()
        .map(|b| (b.kind.clone(), b.inline.text().to_owned()))
        .collect()
}
