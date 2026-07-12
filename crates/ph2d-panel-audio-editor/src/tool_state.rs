//! Which **tool** the pointer is holding over the waveform, and what the clip has been cut into.
//!
//! The Edit section's top row is a tool group — Select, Move, Scale — exactly one of which is
//! armed. The panel owns the choice; the **shell** owns the waveform overlay and the pointer, so
//! it reads the armed tool to decide what a press means: drag a range, drag a piece, or drag a
//! piece's edge.
//!
//! Same split as `spectral_state`: the panel never touches the clip, and the shell publishes back
//! the facts the panel needs in order to dim a button honestly (`set_pieces`).
//!
//! Thread-local: panel and bridge both run on the main thread.

use std::cell::Cell;

/// What a press on the waveform does. Exactly one is armed at a time — a pointer cannot mean two
/// things at once, and a tool group that lets you arm none is a tool group with a dead state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditTool {
    /// Drag a time **range** — the selection. The default, and what the waveform has always done.
    #[default]
    Select,
    /// Drag a **piece** onto another seam: the parts of a split recording, rearranged.
    Move,
    /// Drag a piece's right **edge**: time-stretch it (same pitch, new length) — shorten a take
    /// without cutting anything out of it.
    Scale,
}

thread_local! {
    /// Panel → shell: the armed tool.
    static TOOL: Cell<EditTool> = const { Cell::new(EditTool::Select) };
    /// Shell → panel: how many pieces the clip is in. `1` = uncut. Move needs at least two
    /// (there is nowhere to drop the only piece), and Clear Cuts needs at least one cut to clear.
    static PIECES: Cell<usize> = const { Cell::new(1) };
}

/// Arm a tool. Public because the shell re-arms **Select** when a new clip is loaded: a Move tool
/// left armed over a clip with no cuts is a pointer that does nothing.
pub fn set_tool(t: EditTool) {
    TOOL.with(|c| c.set(t));
}

/// Shell + panel: the armed tool.
pub fn tool() -> EditTool {
    TOOL.with(Cell::get)
}

/// Shell: publish how many pieces the clip is currently in.
///
/// **Disarms Move when the clip stops having pieces to trade.** Refusing to *arm* it on an uncut
/// clip is not enough: arm Move with three pieces, then click Clear Cuts, and the tool is still
/// held — over a clip where dragging can do nothing. A pointer with no legal gesture reads as a
/// broken editor, not an empty one. This is the choke point every path runs through (Clear Cuts,
/// Load, an undo that removes the last cut), so there is one place to get it right instead of
/// three places to forget.
pub fn set_pieces(n: usize) {
    let n = n.max(1);
    PIECES.with(|c| c.set(n));
    if n < 2 && tool() == EditTool::Move {
        set_tool(EditTool::Select);
    }
}

/// How many pieces the clip is in (`1` = uncut).
pub fn pieces() -> usize {
    PIECES.with(Cell::get)
}

/// Is the clip cut at all? What lights Clear Cuts and Export Pieces.
pub(crate) fn has_cuts() -> bool {
    pieces() > 1
}
