//! **O ficheiro dos pincéis guardados** (saído de `ph2d-app-pixel::pinceis`, 09/11): a pasta do
//! artista, a leitura da lista e a tradução da ponta painel ↔ núcleo. O estado vive na família.

use std::path::{Path, PathBuf};

use ph2d_editor_core::pixel_panel::PixelBrushShape;
use ph2d_pixel_core::brush::BrushShape;
use ph2d_pixel_io::brush_file::{self, BrushSlot};

/// O ficheiro da lista, dentro da pasta.
pub const FILE: &str = "brushes.txt";

/// A pasta dos pincéis do artista (`PH2D_PIXEL_BRUSHES_DIR` manda; senão a dos dados do utilizador).
#[must_use]
pub fn pasta_do_utilizador() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("PH2D_PIXEL_BRUSHES_DIR") {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ph2d").join("pixel_brushes"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/ph2d/pixel_brushes"))
}

/// A lista da pasta (ficheiro ausente = lista vazia).
///
/// # Errors
/// O ficheiro existe e não se lê, ou não se interpreta (`brushes.txt:<linha> <o quê>`).
pub fn le(pasta: &Path) -> Result<Vec<BrushSlot>, String> {
    match std::fs::read_to_string(pasta.join(FILE)) {
        Ok(t) => brush_file::read(&t).map_err(|e| format!("{FILE}:{} {}", e.line, e.what)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.to_string()),
    }
}

/// A ponta do painel no núcleo.
#[must_use]
pub fn shape_in(s: PixelBrushShape) -> BrushShape {
    match s {
        PixelBrushShape::Circle => BrushShape::Circle,
        PixelBrushShape::Square => BrushShape::Square,
        PixelBrushShape::Line => BrushShape::Line,
    }
}

/// A ponta do núcleo no painel.
#[must_use]
pub fn shape_out(s: BrushShape) -> PixelBrushShape {
    match s {
        BrushShape::Circle => PixelBrushShape::Circle,
        BrushShape::Square => PixelBrushShape::Square,
        BrushShape::Line => PixelBrushShape::Line,
    }
}
