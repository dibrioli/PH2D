//! Re-export dos ids canónicos do editor-core. Eles vivem lá (e não aqui) pela mesma razão dos
//! irmãos: o `z-order walk` do chrome e o censo de colisões enumeram-nos lá, e movê-los para uma
//! crate de painel criaria um ciclo.

pub use ph2d_editor_core::ids::{
    ASSET_CATALOG_COL, ASSET_PANEL, MAX_CATALOG_ROWS, asset_cell_id, catalog_row_id,
    catalog_row_index,
};

mod asset_browser;
pub use asset_browser::*;
