//! O que é do artista e não do preset: favoritos, recentes, a política dos volumes, os volumes
//! lembrados por preset e o preset activo — `estado.json` na raiz do utilizador.

use crate::modelo::{Erro, Volumes};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Quantos recentes a lista guarda. Recurso: as linhas do navegador (a secção «Recentes» mostra-os
/// todos sem rolar dentro da coluna do painel).
pub const MAX_RECENTES: usize = 8;

/// O nome do ficheiro de estado na raiz do utilizador.
pub const FICHEIRO_ESTADO: &str = "estado.json";

/// Ao trocar de preset, o que se guarda do que o artista mexeu (a política do Rebelle, 02 §5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Politica {
    /// Os volumes e o modo de pintura ficam lembrados no preset que se deixa.
    #[default]
    VolumesAndMode,
    /// Só os volumes.
    Volumes,
    /// Nada: voltar a um preset é voltar ao que o ficheiro diz.
    None,
}

impl Politica {
    pub const TODAS: [Politica; 3] = [Politica::VolumesAndMode, Politica::Volumes, Politica::None];
}

/// O que um preset lembra do artista.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Lembrado {
    #[serde(default)]
    pub volumes: Volumes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint_mode: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EstadoDoUtilizador {
    #[serde(default)]
    pub favoritos: BTreeSet<String>,
    /// O mais recente primeiro, sem repetidos, no máximo [`MAX_RECENTES`].
    #[serde(default)]
    pub recentes: Vec<String>,
    #[serde(default)]
    pub politica: Politica,
    #[serde(default)]
    pub lembrados: BTreeMap<String, Lembrado>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ativo: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl EstadoDoUtilizador {
    /// Lê `estado.json` da raiz. Ausente = vazio.
    ///
    /// # Errors
    /// [`Erro::Json`] se o ficheiro existe e não se lê (não se apaga o que o artista tinha).
    pub fn ler(raiz: &Path) -> Result<Self, Erro> {
        match std::fs::read(raiz.join(FICHEIRO_ESTADO)) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| Erro::Json(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Erro::Disco(e.to_string())),
        }
    }

    /// Grava `estado.json` na raiz.
    ///
    /// # Errors
    /// [`Erro::Disco`] se o disco recusar.
    pub fn gravar(&self, raiz: &Path) -> Result<(), Erro> {
        let disco = |e: std::io::Error| Erro::Disco(e.to_string());
        std::fs::create_dir_all(raiz).map_err(disco)?;
        let texto = serde_json::to_string_pretty(self).map_err(|e| Erro::Json(e.to_string()))?;
        let tmp = raiz.join(format!("{FICHEIRO_ESTADO}.tmp"));
        std::fs::write(&tmp, texto).map_err(disco)?;
        std::fs::rename(&tmp, raiz.join(FICHEIRO_ESTADO)).map_err(disco)
    }

    /// O artista escolheu este preset: passa a activo e ao topo dos recentes.
    pub fn usar(&mut self, relativo: &str) {
        self.recentes.retain(|r| r != relativo);
        self.recentes.insert(0, relativo.to_owned());
        self.recentes.truncate(MAX_RECENTES);
        self.ativo = Some(relativo.to_owned());
    }

    pub fn alternar_favorito(&mut self, relativo: &str) {
        if !self.favoritos.remove(relativo) {
            self.favoritos.insert(relativo.to_owned());
        }
    }

    /// Ao deixar `relativo`: guarda o que a política manda (e esquece o que ela já não manda).
    pub fn lembrar(&mut self, relativo: &str, volumes: Volumes, paint_mode: Option<String>) {
        match self.politica {
            Politica::None => {
                self.lembrados.remove(relativo);
            }
            Politica::Volumes => {
                self.lembrados.insert(
                    relativo.to_owned(),
                    Lembrado {
                        volumes,
                        paint_mode: None,
                    },
                );
            }
            Politica::VolumesAndMode => {
                self.lembrados.insert(
                    relativo.to_owned(),
                    Lembrado {
                        volumes,
                        paint_mode,
                    },
                );
            }
        }
    }

    /// O preset deixou de existir no caminho (apagado) ou voltou à fábrica: esquece o lembrado.
    pub fn esquecer(&mut self, relativo: &str) {
        self.lembrados.remove(relativo);
    }
}
