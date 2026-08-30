//! Re-export dos ids canónicos do editor-core. Eles vivem lá (e não aqui) pela mesma razão dos
//! irmãos: o `z-order walk` do chrome e o censo de colisões enumeram-nos lá, e movê-los para uma
//! crate de painel criaria um ciclo.

pub use ph2d_editor_core::ids::{
    ASSET_CATALOG_ALL, ASSET_CATALOG_COL, ASSET_CATALOG_NEW, ASSET_CATALOG_TOGGLE,
    ASSET_CATALOG_UNASSIGNED, ASSET_CLOSE, ASSET_DRAG_HANDLE, ASSET_KIND, ASSET_KIND_FILTERS,
    ASSET_PANEL, ASSET_RESIZE_HANDLE_BL, ASSET_SEARCH, ASSET_SIZE, ASSET_SORT, ASSET_SORT_MODES,
    MAX_ASSET_CELLS, MAX_CATALOG_ROWS, asset_cell_id, catalog_row_id,
};
