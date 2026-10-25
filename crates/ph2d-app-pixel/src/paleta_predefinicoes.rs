//! **As paletas do artista** (P2): *Save Palette as Preset* / *as Default* e a lista que se carrega
//! como o *Load Palette*. Um `.gpl` por paleta numa pasta do utilizador; a de omissão é
//! `default.gpl` e é a que um Pixel Sprite NOVO recebe (e a que o *Load Default Palette* carrega —
//! a lei do Aseprite); sem ela, a DB32 de fábrica.

use std::path::{Path, PathBuf};

use ph2d_pixel_core::palette;
use ph2d_pixel_io::palette_file::{self, PaletteFormat};

use crate::PixelApp;

/// O ficheiro da paleta de omissão, dentro da pasta.
pub const DEFAULT_FILE: &str = "default.gpl";
const EXT: &str = "gpl";

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

fn le(caminho: &Path) -> Result<palette_file::PaletteFile, String> {
    let bytes = std::fs::read(caminho).map_err(|e| e.to_string())?;
    palette_file::read(&bytes, Some(EXT)).map_err(|e| format!("{e:?}"))
}

fn grava(caminho: &Path, nome: &str, colors: &[[u8; 4]], t: Option<u8>) -> Result<(), String> {
    if let Some(p) = caminho.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let bytes = palette_file::write(PaletteFormat::Gpl, nome, colors, t);
    std::fs::write(caminho, bytes).map_err(|e| e.to_string())
}

impl PixelApp {
    /// **A paleta com que nasce um Pixel Sprite**: a de omissão do artista, ou a DB32.
    #[must_use]
    pub fn paleta_de_omissao(&self) -> Vec<[u8; 4]> {
        self.paletas
            .as_deref()
            .and_then(|p| le(&p.join(DEFAULT_FILE)).ok())
            .map(|f| f.colors)
            .filter(|c| !c.is_empty() && c.len() <= palette::MAX_COLORS)
            .unwrap_or_else(|| palette::DEFAULT.to_vec())
    }

    /// As predefinições gravadas (lidas da pasta na primeira pergunta e depois de cada gravação).
    pub fn predefinicoes(&mut self) -> &[String] {
        if self.predefinicoes.is_none() {
            self.predefinicoes = Some(self.paletas.as_deref().map(lista).unwrap_or_default());
        }
        self.predefinicoes.as_deref().unwrap_or_default()
    }

    /// A paleta do sprite `key` e o índice transparente que o ficheiro guarda (Indexed).
    fn paleta_do(&self, key: u32) -> Option<(Vec<[u8; 4]>, Option<u8>)> {
        let s = &self.docs.get(&key)?.sprite;
        let t = (s.mode() == ph2d_pixel_core::ColorMode::Indexed).then(|| s.transparent_index());
        Some((s.palette().to_vec(), t))
    }

    /// *Save Palette as Preset* para o ficheiro `caminho` (o diálogo abre na pasta).
    pub fn grava_predefinicao(&mut self, key: u32, caminho: &Path) {
        let Some((colors, t)) = self.paleta_do(key) else {
            return;
        };
        let nome = caminho
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        match grava(caminho, &nome, &colors, t) {
            Ok(()) => {
                self.predefinicoes = None;
                self.aviso(
                    false,
                    ph2d_i18n::tr_with("pixel.palette.preset_saved", &[("name", &nome)]),
                );
            }
            Err(e) => self.aviso(
                true,
                ph2d_i18n::tr_with("pixel.palette.save_failed", &[("e", &e)]),
            ),
        }
    }

    pub(crate) fn grava_predefinicao_pelo_dialogo(&mut self, key: u32) {
        let Some(pasta) = self.paletas.clone() else {
            self.aviso(true, ph2d_i18n::tr("pixel.palette.no_folder"));
            return;
        };
        let _ = std::fs::create_dir_all(&pasta);
        let dialogo = rfd::FileDialog::new()
            .add_filter(ph2d_i18n::tr("pixel.palette.file_kind"), &[EXT])
            .set_directory(&pasta)
            .set_file_name(format!("palette.{EXT}"));
        if let Some(caminho) = ph2d_app_host::modal::save_file(dialogo) {
            self.grava_predefinicao(key, &caminho);
        }
    }

    /// *Save Palette as Default*: a paleta do sprite `key` passa a ser a de um sprite novo.
    pub fn grava_omissao(&mut self, key: u32) {
        let Some(pasta) = self.paletas.clone() else {
            self.aviso(true, ph2d_i18n::tr("pixel.palette.no_folder"));
            return;
        };
        let Some((colors, t)) = self.paleta_do(key) else {
            return;
        };
        match grava(&pasta.join(DEFAULT_FILE), "", &colors, t) {
            Ok(()) => self.aviso(false, ph2d_i18n::tr("pixel.palette.default_saved")),
            Err(e) => self.aviso(
                true,
                ph2d_i18n::tr_with("pixel.palette.save_failed", &[("e", &e)]),
            ),
        }
    }

    /// Carrega a predefinição `nome` (UM passo do desfazer, como o *Load Palette*).
    pub(crate) fn carrega_predefinicao(&mut self, key: u32, nome: &str) {
        let lida = self
            .paletas
            .as_deref()
            .map(|p| p.join(format!("{nome}.{EXT}")))
            .ok_or_else(String::new)
            .and_then(|c| le(&c));
        match lida {
            Ok(f) => self.poe_paleta(key, f.colors, f.transparent),
            Err(_) => {
                self.predefinicoes = None;
                self.aviso(
                    true,
                    ph2d_i18n::tr_with("pixel.palette.preset_missing", &[("name", &nome)]),
                );
            }
        }
    }
}
