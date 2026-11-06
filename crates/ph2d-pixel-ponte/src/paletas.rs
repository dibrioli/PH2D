//! As paletas do artista no disco: um `.gpl` por paleta numa pasta do utilizador.

use std::path::{Path, PathBuf};

use ph2d_pixel_io::palette_file::{self, PaletteFormat};

/// O ficheiro da paleta de omissão, dentro da pasta.
pub const DEFAULT_FILE: &str = "default.gpl";
pub const EXT: &str = "gpl";

/// A pasta: `PH2D_PALETTES_DIR` · `$XDG_DATA_HOME/ph2d/palettes` · `$HOME/.local/share/ph2d/palettes`
/// (o padrão dos pincéis e das macros).
#[must_use]
pub fn pasta_do_utilizador() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("PH2D_PALETTES_DIR") {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ph2d").join("palettes"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/ph2d/palettes"))
}

/// Os nomes das predefinições da pasta (sem a de omissão), por ordem.
#[must_use]
pub fn lista(pasta: &Path) -> Vec<String> {
    let Ok(dir) = std::fs::read_dir(pasta) else {
        return Vec::new();
    };
    let mut nomes: Vec<String> = dir
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(EXT))
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some(DEFAULT_FILE))
        .filter_map(|p| p.file_stem().and_then(|n| n.to_str()).map(str::to_owned))
        .collect();
    nomes.sort();
    nomes
}

pub fn le(caminho: &Path) -> Result<palette_file::PaletteFile, String> {
    let bytes = std::fs::read(caminho).map_err(|e| e.to_string())?;
    palette_file::read(&bytes, Some(EXT)).map_err(|e| format!("{e:?}"))
}

pub fn grava(caminho: &Path, nome: &str, colors: &[[u8; 4]], t: Option<u8>) -> Result<(), String> {
    if let Some(p) = caminho.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let bytes = palette_file::write(PaletteFormat::Gpl, nome, colors, t);
    std::fs::write(caminho, bytes).map_err(|e| e.to_string())
}
