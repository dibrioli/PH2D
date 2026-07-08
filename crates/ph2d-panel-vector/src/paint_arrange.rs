//! Path-reshape subsection painter for the Vector Style panel: the Smooth /
//! Sharpen / Simplify / Subdivide buttons that act on ALL vertices of the
//! selected path. Split from `paint_sections` to keep that file under the
//! 600-LOC panel cap; it's an `impl BodyCtx` block over there.

use crate::paint_sections::BodyCtx;
use crate::{ids, state};

impl BodyCtx<'_> {
    /// "Path" section — reshape the whole selected path. Smooth / Sharpen are a
    /// 2-col row; Simplify (fewer points) / Subdivide (more points) are the
    /// point-density pair on a second 2-col row; then a full-width Close/Open
    /// toggle (label from the published `closed` flag). `w`/`gap` are the shared
    /// Arrange column metrics; returns the advanced `y`.
    pub(crate) fn path_section(&mut self, w: f32, gap: f32, mut y: f32) -> f32 {
        y = self.section_label("Path", y);
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SMOOTH, "Smooth"),
                (ids::VECTOR_PATH_SHARPEN, "Sharpen"),
            ],
            y,
        );
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SIMPLIFY, "Simplify"),
                (ids::VECTOR_PATH_SUBDIVIDE, "Subdivide"),
            ],
            y,
        );
        // Close/Open toggle — label reflects the current state (default "Close"
        // when no path is selected / not yet closed).
        let label = if state::current_path_closed() == Some(true) {
            "Open Path"
        } else {
            "Close Path"
        };
        self.action_button(ids::VECTOR_PATH_CLOSE, label, y)
    }
}
