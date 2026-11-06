//! **As paletas do artista** (P2): *Save Palette as Preset* / *as Default* e a lista que se carrega
//! como o *Load Palette*. Um `.gpl` por paleta numa pasta do utilizador; a de omissão é
//! `default.gpl` e é a que um Pixel Sprite NOVO recebe (e a que o *Load Default Palette* carrega —
//! a lei do Aseprite); sem ela, a DB32 de fábrica.

use std::path::Path;

use ph2d_pixel_core::palette;

use crate::PixelApp;

pub use ph2d_pixel_ponte::paletas::{DEFAULT_FILE, lista, pasta_do_utilizador};
use ph2d_pixel_ponte::paletas::{EXT, grava, le};

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
