//! **O IMPORT de um `.ase`/`.aseprite`**: mora em [`ph2d_app_sprite_anim::ase_import`] (P14e, a
//! shell só compõe); aqui só o nome que o roteador e a sonda usam.

pub(crate) use ph2d_app_sprite_anim::ase_import::{
    ASE_EXTENSIONS, AseImportResult, import_ase, is_ase_file,
};
