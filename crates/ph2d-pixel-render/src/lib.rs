//! **Pixel Art — a vista** (ADR-0196).
//!
//! O sprite desenha-se como UMA imagem composta, amostrada por vizinho mais próximo
//! (`ImageQuality::Low`), pelo afim `pixel da imagem → ecrã` que a família calcula com a câmera e o
//! `Transform` do objecto. Por cima, no modo Edit: o xadrez por baixo, a grelha de pixels, a borda
//! do canvas e o cursor de célula. As linhas saem em coordenadas de ECRÃ com `Affine::IDENTITY` (o
//! transform de um `stroke` do Vello multiplica a espessura).

mod checker;
pub mod desenhados;
pub mod guias;
mod lint_marks;
mod overlay;
pub mod prefs_canal;
pub mod prefs_view;
mod view;
pub mod zoom;

pub use checker::{CHECKER_CELLS, Checker, draw_checker};
pub use lint_marks::draw_lint_marks;
pub use overlay::{
    ANTS_DASH_PX, SliceMark, draw_marching_ants, draw_slice, mask_edges, rect_edges,
};
pub use view::{
    CURSOR_CORE, SpriteImage, cell_at, draw_border, draw_cell_cursor, draw_pixel_grid,
    draw_polygons, draw_symmetry_axes, draw_tile_grid, image_to_world, px_per_cell,
};
