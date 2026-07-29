#![forbid(unsafe_code)]
//! `ph2d-flip-render` — o pipeline **wgpu dedicado** que rasteriza os traços do
//! Flip (ADR-0114, W1). Mesmo caminho para o editor e para o runtime do jogo.
//!
//! Por que crate separada (e não dentro de `ph2d-render`): o `ph2d-render`
//! deliberadamente **não depende de crates de documento** (o editor constrói a
//! cena e a passa adiante — mesmo padrão do `ph2d-vec-render`). Aqui é igual: este
//! passe recebe `wgpu::Device/Queue` + o alvo do shell, lê o `FlipDrawing`, e
//! rasteriza. O compositing de camada (T1.7) reusa o compositor 22-modos do
//! Painter na orquestração do shell, não daqui.
//!
//! **Chave 2D-ortográfica:** a matemática 3D do Grease Pencil COLAPSA — sem
//! divisão perspectiva, `thickness_px = raio_mundo · pixels_por_mundo` (o zoom da
//! `Camera2d`); o plano do traço É o plano da tela.
//!
//! W1 incremental: **T1.1 = packing** (este módulo, headless). Vertex/fragment
//! shader + passe entram a seguir.

mod binning;
mod composite;
mod fill;
mod fill_holes;
mod neighbors;
mod pack;
mod pipeline;
mod tau;
mod walk_gpu;

pub use binning::{
    BinSeg, DEFAULT_TILE, MIN_WIDTH_PX, ScreenSpace, TileBins, bin_segments, walk_pixel,
};
pub use composite::{FlipCompose, SLICE_FORMAT};
pub use fill::{FillVertex, triangulate};
pub use pack::{
    FLAG_CLOSED, FLAG_END_FLAT, FLAG_START_FLAT, FlipGpuData, GpuPoint, GpuSegRef, GpuStroke,
    pack_drawing, pack_drawings,
};
pub use pipeline::{CameraRaw, FlipRenderer};
pub use tau::{PAINTER_SPACING, SUB, dab_weight, f_of};
pub use walk_gpu::{WalkJob, WalkPass};
