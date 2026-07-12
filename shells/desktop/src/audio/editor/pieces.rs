//! **The piece gesture** — dragging a part of a split clip, and dragging its edge.
//!
//! The document is a buffer with cuts (`ph2d_audio_edit::pieces`), so both gestures end in one
//! call and one undo step. What lives here is everything *between* the press and that call: the
//! drag in flight.
//!
//! ## Why the drag is not committed frame by frame
//!
//! Move could be: a reorder is a memcpy, and re-running it per mouse-move would be affordable.
//! Scale could not — WSOLA over a 3-minute piece is not a per-frame cost, and a time-stretch that
//! ran sixty times a second would spend the whole drag re-grinding audio the user is still
//! deciding about. So **neither** commits until release, and both draw an outline instead. That
//! is also what an NLE does, and for the same reason: what you are dragging is an *intent*, and it
//! is not audio until you let go.
//!
//! The gesture therefore lives on the runtime (not the `App`), because the **overlay** has to draw
//! it and the overlay is handed the `AudioSystem`, not the shell's input state.

use crate::audio::AudioSystem;

/// A piece drag in flight. Nothing is committed while one of these exists.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PieceDrag {
    /// Dragging piece `piece` around; `to` is the boundary index it would drop onto right now.
    Move { piece: usize, to: usize },
    /// Dragging piece `piece`'s right edge; `new_len` is the length it would become (frames).
    Scale { piece: usize, new_len: usize },
}

/// The shortest a piece may be dragged down to. A piece of a handful of frames is not a take, and
/// WSOLA has nothing to lock onto inside one — the grain is 1024 frames wide.
const MIN_PIECE_FRAMES: usize = 256;

impl AudioSystem {
    /// The clip's cut positions (frames) — what the overlay draws as seams.
    pub(crate) fn editor_cuts(&self) -> Vec<usize> {
        self.editor
            .clip
            .as_ref()
            .map(|c| c.cuts().to_vec())
            .unwrap_or_default()
    }

    /// How many pieces the clip is in (`1` = uncut) — what dims Move, Clear Cuts, Export Pieces.
    pub(crate) fn editor_piece_count(&self) -> usize {
        self.editor.clip.as_ref().map_or(1, |c| c.pieces().len())
    }

    /// The drag in flight, for the overlay to outline.
    pub(crate) fn editor_piece_drag(&self) -> Option<PieceDrag> {
        self.editor.piece_drag
    }

    /// Press at `frame` with the **Move** tool: grab the piece under the cursor and select it, so
    /// the thing you are dragging is the thing that is highlighted. `false` if there is nothing to
    /// grab (no clip, or a single uncut piece with nowhere to go).
    pub(crate) fn editor_piece_grab(&mut self, frame: usize) -> bool {
        let Some(clip) = self.editor.clip.as_mut() else {
            return false;
        };
        let pieces = clip.pieces();
        if pieces.len() < 2 {
            return false;
        }
        let piece = clip.piece_at(frame);
        clip.set_selection(pieces.get(piece).cloned());
        self.editor.piece_drag = Some(PieceDrag::Move { piece, to: piece });
        true
    }

    /// Press at `frame` with the **Scale** tool: grab the piece under the cursor by its right edge.
    /// Works on an uncut clip too — one piece is the whole take, and shortening it is the point.
    pub(crate) fn editor_piece_scale_grab(&mut self, frame: usize) -> bool {
        let Some(clip) = self.editor.clip.as_mut() else {
            return false;
        };
        let pieces = clip.pieces();
        let piece = clip.piece_at(frame);
        let Some(r) = pieces.get(piece) else {
            return false;
        };
        clip.set_selection(Some(r.clone()));
        self.editor.piece_drag = Some(PieceDrag::Scale {
            piece,
            new_len: r.len(),
        });
        true
    }

    /// The cursor moved to `frame` while a piece drag is live. Updates the outline; commits nothing.
    pub(crate) fn editor_piece_drag_to(&mut self, frame: usize) -> bool {
        let (Some(clip), Some(drag)) = (self.editor.clip.as_ref(), self.editor.piece_drag) else {
            return false;
        };
        self.editor.piece_drag = Some(match drag {
            PieceDrag::Move { piece, .. } => PieceDrag::Move {
                piece,
                to: clip.nearest_boundary(frame),
            },
            PieceDrag::Scale { piece, .. } => {
                // The edge follows the cursor: the new length is from the piece's START to here.
                // Clamped through the SAME function the commit uses, so the outline cannot promise
                // a length the release then refuses to make.
                let start = clip.pieces().get(piece).map_or(0, |r| r.start);
                let want = frame.saturating_sub(start).max(MIN_PIECE_FRAMES);
                PieceDrag::Scale {
                    piece,
                    new_len: clip.clamp_stretch(piece, want),
                }
            }
        });
        true
    }

    /// Let go. **This** is where the one undo step lands.
    pub(crate) fn editor_piece_release(&mut self) -> bool {
        let Some(drag) = self.editor.piece_drag.take() else {
            return false;
        };
        // The rack auditions over the CURRENT clip. Moving the audio out from under it would leave
        // a preview of a buffer that no longer exists — the same reasoning `editor_apply` uses for
        // every other edit, and the same answer.
        if self.editor.fx_audition.is_some() {
            self.editor_fx_cancel();
        }
        let Some(clip) = self.editor.clip.as_mut() else {
            return true;
        };
        match drag {
            PieceDrag::Move { piece, to } => {
                clip.move_piece(piece, to);
            }
            PieceDrag::Scale { piece, new_len } => {
                clip.stretch_piece(piece, new_len);
            }
        }
        // The clip is a different buffer now. Hot-swap it into the sounding preview, or the
        // rearranged take keeps playing in its old order — the kind of "it works, but the sound is
        // wrong" that a listener catches instantly and a test never does.
        self.editor_hot_swap();
        true
    }
}
