//! **Os pincéis guardados** (os espaços Alt+1, Alt+2… do Aseprite) — no COMPUTADOR, ao lado das
//! paletas do artista, valem em todo projecto (dono 02/11). Cada Ctrl+B cria um espaço
//! destrancado com *Brush Type* + *Image Color*; *Save Brush Here* cria um trancado com o pincel
//! actual inteiro; recuperar um espaço aplica só os parâmetros que ele guarda (Trial, §3 do handoff P7a).

use std::path::{Path, PathBuf};

use ph2d_editor_core::pixel_panel::PixelBrushShape;
use ph2d_pixel_core::brush::BrushShape;
use ph2d_pixel_core::pattern::PatternBrush;
use ph2d_pixel_io::brush_file::{self, BrushKind, BrushParams, BrushSlot};

use crate::PixelApp;

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

fn le(pasta: &Path) -> Result<Vec<BrushSlot>, String> {
    match std::fs::read_to_string(pasta.join(FILE)) {
        Ok(t) => brush_file::read(&t).map_err(|e| format!("{FILE}:{} {}", e.line, e.what)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.to_string()),
    }
}

fn shape_in(s: PixelBrushShape) -> BrushShape {
    match s {
        PixelBrushShape::Circle => BrushShape::Circle,
        PixelBrushShape::Square => BrushShape::Square,
        PixelBrushShape::Line => BrushShape::Line,
    }
}

fn shape_out(s: BrushShape) -> PixelBrushShape {
    match s {
        BrushShape::Circle => PixelBrushShape::Circle,
        BrushShape::Square => PixelBrushShape::Square,
        BrushShape::Line => PixelBrushShape::Line,
    }
}

impl PixelApp {
    /// A lista (lida da pasta na 1.ª pergunta; um ficheiro estragado avisa e começa vazia, sem o
    /// apagar até à próxima gravação).
    pub fn espacos(&mut self) -> &[BrushSlot] {
        self.espacos_mut();
        self.espacos.as_deref().unwrap_or_default()
    }

    fn espacos_mut(&mut self) -> &mut Vec<BrushSlot> {
        if self.espacos.is_none() {
            let lida = self.pinceis.as_deref().map(le).unwrap_or(Ok(Vec::new()));
            let lista = lida.unwrap_or_else(|e| {
                self.avisos.push((
                    true,
                    ph2d_i18n::tr_with("pixel.brush.read_failed", &[("e", &e)]),
                ));
                Vec::new()
            });
            self.espacos = Some(lista);
        }
        self.espacos.get_or_insert_with(Vec::new)
    }

    fn grava_espacos(&mut self) {
        let Some(pasta) = self.pinceis.clone() else {
            return;
        };
        let texto = brush_file::write(self.espacos.as_deref().unwrap_or_default());
        let r =
            std::fs::create_dir_all(&pasta).and_then(|()| std::fs::write(pasta.join(FILE), texto));
        if let Err(e) = r {
            self.avisos.push((
                true,
                ph2d_i18n::tr_with("pixel.brush.save_failed", &[("e", &e.to_string())]),
            ));
        }
    }

    /// O espaço do pincel actual: as opções e o padrão (lido no sprite `key`).
    fn espaco_actual(&self, key: u32, params: BrushParams, locked: bool) -> Option<BrushSlot> {
        let o = &self.options;
        let kind = match &self.pincel {
            Some(p) => {
                let s = &self.docs.get(&key)?.sprite;
                BrushKind::Pattern {
                    size: p.size(),
                    cells: p.to_rgba(s),
                    origin: p.origin,
                }
            }
            None => BrushKind::Shape(shape_in(o.shape)),
        };
        Some(BrushSlot {
            locked,
            params,
            kind,
            size: o.size,
            angle: o.angle,
            image_color: self.pincel_imagem,
            ink: crate::gesture::ink(o.ink),
            fg: self.fg,
            bg: self.bg,
            pixel_perfect: o.pixel_perfect,
        })
    }

    /// O Ctrl+B acabou de pôr um padrão na mão: um espaço novo, destrancado.
    pub(crate) fn espaco_do_ctrl_b(&mut self, key: u32) {
        if let Some(s) = self.espaco_actual(key, BrushParams::NEW_BRUSH, false) {
            self.espacos_mut().push(s);
            self.grava_espacos();
        }
    }

    /// **Save Brush Here**: o pincel actual inteiro num espaço TRANCADO.
    pub(crate) fn guarda_aqui(&mut self, key: u32) {
        if let Some(s) = self.espaco_actual(key, BrushParams::SAVE_HERE, true) {
            let n = {
                let l = self.espacos_mut();
                l.push(s);
                l.len()
            };
            self.grava_espacos();
            self.avisos.push((
                false,
                ph2d_i18n::tr_with("pixel.brush.saved", &[("n", &n.to_string())]),
            ));
        }
    }

    /// **Alt+n**: recupera o espaço `n` (de 0) sobre o sprite `key` — só os parâmetros que ele
    /// guarda; o padrão volta com as cores da imagem.
    pub(crate) fn recupera(&mut self, key: u32, n: usize) -> bool {
        let Some(s) = self.espacos_mut().get(n).cloned() else {
            return false;
        };
        let p = s.params;
        if p.kind {
            match &s.kind {
                BrushKind::Shape(sh) => {
                    self.pincel = None;
                    self.options.shape = shape_out(*sh);
                }
                BrushKind::Pattern {
                    size,
                    cells,
                    origin,
                } => {
                    let Some(doc) = self.docs.get(&key) else {
                        return false;
                    };
                    let Some(b) = PatternBrush::from_rgba(&doc.sprite, *size, cells, *origin)
                    else {
                        return false;
                    };
                    self.pincel = Some(std::sync::Arc::new(b));
                }
            }
        }
        if p.size {
            self.options.size = s
                .size
                .clamp(1, ph2d_editor_core::pixel_panel::PIXEL_MAX_BRUSH);
        }
        if p.angle {
            self.options.angle = s.angle;
        }
        if p.image_color {
            self.pincel_imagem = s.image_color;
        }
        if p.ink {
            self.options.ink = crate::gesture::ink_out(s.ink);
        }
        if p.fg {
            self.fg = s.fg;
        }
        if p.bg {
            self.bg = s.bg;
        }
        if p.pixel_perfect {
            self.options.pixel_perfect = s.pixel_perfect;
        }
        true
    }

    /// Apaga o espaço `n` (de 0).
    pub(crate) fn apaga_espaco(&mut self, n: usize) {
        let l = self.espacos_mut();
        if n < l.len() {
            l.remove(n);
            self.grava_espacos();
        }
    }

    /// Tranca ou destranca o espaço `n`.
    pub(crate) fn tranca_espaco(&mut self, n: usize) {
        if let Some(s) = self.espacos_mut().get_mut(n) {
            s.locked = !s.locked;
            self.grava_espacos();
        }
    }
}
