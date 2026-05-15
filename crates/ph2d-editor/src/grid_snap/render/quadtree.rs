//! Quadtree render adapter — stub for Stage 8 (lands in Stage 9).

use crate::grid::GridView;
use crate::grid_snap::state::QuadtreeCfg;
use ph2d_vector::{Color, VectorScene};

pub fn paint(_scene: &mut VectorScene, _view: &GridView, _color: Color, _cfg: &QuadtreeCfg) {
    // Stage 9 fills in: build Quadtree, insert demo points, recurse
    // leaf AABBs into BezPath outlines, stroke.
}
