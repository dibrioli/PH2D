//! **Pixel Art — a vista** (ADR-0192).
//!
//! O sprite desenha-se como UMA imagem composta, amostrada por vizinho mais próximo
//! (`ImageQuality::Low`), pelo afim `pixel da imagem → ecrã` que a família calcula com a câmera e o
//! `Transform` do objecto. Por cima, no modo Edit: o xadrez por baixo, a grelha de pixels, a borda
//! do canvas e o cursor de célula. As linhas saem em coordenadas de ECRÃ com `Affine::IDENTITY` (o
//! transform de um `stroke` do Vello multiplica a espessura).

mod view;
pub mod zoom;

pub use view::{
    CHECKER_CELLS, CURSOR_CORE, SpriteImage, cell_at, draw_border, draw_cell_cursor, draw_checker,
    draw_pixel_grid, draw_symmetry_axes, image_to_world, px_per_cell,
};
