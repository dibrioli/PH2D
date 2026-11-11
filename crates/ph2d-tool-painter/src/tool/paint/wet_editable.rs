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
    /// O pincel de agora com o papel à escala do documento (B34.1, [`PainterTool::papel_na_escala`]) — para quem
    /// guarda o pincel inteiro: o estilo do traço da aguada e o alcance do Paper Edge.
    #[must_use]
    pub(super) fn pincel_com_papel_na_escala(&self) -> ph2d_painter_brush::BrushSpec {
        let mut b = self.paint.brush;
        b.paper = self.papel_na_escala(b.paper);
        b
    }

    /// The current substrate signature (see [`WetEditableSig`]).
    pub(super) fn wet_editable_sig(&self) -> WetEditableSig {
        let b = &self.paint.brush;
        WetEditableSig {
            grain: b.texture,
            paper: self.papel_na_escala(b.paper),
            paper_depth: b.paper_depth,
            granulation: b.effective_granulation(),
            granulation_use_paper: b.granulation_use_paper,
            grain_image_version: self.paint.texture_image_version,
            paper_image_version: self.paint.paper_image_version,
        }
    }
}

#[cfg(test)]
#[path = "fator_papel_tests.rs"]
mod fator_papel_tests;
