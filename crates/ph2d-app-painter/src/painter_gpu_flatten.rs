//! Flatten a painter `LayerStack` → `Vec<LayerOp>` for the GPU compositor.
//!
//! ⭐ The law lives in [`ph2d_painter_layer_ops::flatten_for_gpu`] (moved
//! there when the 3D piece's layer stack became its second consumer,
//! `docs/3D/30` §13); this is the Painter's name for the same door.

pub use ph2d_painter_layer_ops::flatten_for_gpu;
