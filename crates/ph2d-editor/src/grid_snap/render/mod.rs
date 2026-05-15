//! Per-kind render adapters that emit `ph2d_vector::VectorScene`
//! strokes consuming `ph2d_grid` math against the editor's
//! [`crate::grid::GridView`] (camera + canvas-clip params).
//!
//! Single entry point [`paint`] dispatches by [`super::state::GridKind`].
//! The Coordenador's planned refactor of `crates/ph2d-editor/src/grid.rs`
//! will call this from the host render loop in place of its current
//! hardcoded-square overlay.

pub mod chunks;
pub mod hex;
pub mod iso;
pub mod quadtree;
pub mod square;
pub mod staggered;
pub mod tri;
pub mod util;
pub mod voronoi;

use crate::grid::GridView;
use crate::grid_snap::state::{GridKind, GridSnapState};
use ph2d_vector::{Color, VectorScene};

/// Compose the active-kind grid overlay into `scene`, clipped to
/// `view.canvas`. No-op when overlay display is disabled.
pub fn paint(scene: &mut VectorScene, view: &GridView, state: &GridSnapState) {
    if !state.show_overlay {
        return;
    }
    let color = color_with_opacity(state.color_rgba, state.opacity);

    scene.push_clip(&crate::paint::rect_to_vello(view.canvas));
    match state.kind {
        GridKind::Square => square::paint(scene, view, color, &state.square_cfg),
        GridKind::Hex => hex::paint(scene, view, color, &state.hex_cfg),
        GridKind::Iso => iso::paint(scene, view, color, &state.iso_cfg),
        GridKind::StaggeredSquare => {
            staggered::paint_square(scene, view, color, &state.staggered_square_cfg)
        }
        GridKind::StaggeredHex => {
            staggered::paint_hex(scene, view, color, &state.staggered_hex_cfg)
        }
        GridKind::Tri => tri::paint(scene, view, color, &state.tri_cfg),
        GridKind::Quadtree => quadtree::paint(scene, view, color, &state.quadtree_cfg),
        GridKind::Voronoi => voronoi::paint(scene, view, color, &state.voronoi_cfg),
        GridKind::Chunks => chunks::paint(scene, view, color, &state.chunks_cfg),
    }
    scene.pop_layer();
}

fn color_with_opacity(rgba: [u8; 4], opacity: f32) -> Color {
    let alpha = (rgba[3] as f32 * opacity.clamp(0.0, 1.0)) as u8;
    Color::from_rgba8(rgba[0], rgba[1], rgba[2], alpha)
}
