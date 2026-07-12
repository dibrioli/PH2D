//! The **substrate signature** of the live-editable wash — the change detector that decides whether a
//! committed-but-still-wet pool must be re-rendered. Its own module (sibling of `paint.rs`, which sits at
//! the workspace LOC cap) so the next field added to it cannot push `paint.rs` over.

use crate::tool::PainterTool;

/// Everything the editable wash's re-render depends on that can move while the paper is still wet — the
/// change detector for [`PainterTool::rerender_editable_wash`]. Anything `apply_watercolor` reads from the
/// SUBSTRATE (the two texture slots, their depth/granulation knobs, and the pixel versions of their images)
/// belongs here; leaving a field out makes its slider silently inert on a committed wash.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct WetEditableSig {
    grain: ph2d_painter_brush::TextureSettings,
    paper: ph2d_painter_brush::TextureSettings,
    paper_depth: f32,
    granulation: f32,
    granulation_use_paper: bool,
    grain_image_version: u64,
    paper_image_version: u64,
}

impl PainterTool {
    /// The current substrate signature (see [`WetEditableSig`]).
    pub(super) fn wet_editable_sig(&self) -> WetEditableSig {
        let b = &self.paint.brush;
        WetEditableSig {
            grain: b.texture,
            paper: b.paper,
            paper_depth: b.paper_depth,
            granulation: b.effective_granulation(),
            granulation_use_paper: b.granulation_use_paper,
            grain_image_version: self.paint.texture_image_version,
            paper_image_version: self.paint.paper_image_version,
        }
    }
}
