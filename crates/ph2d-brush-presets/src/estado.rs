//! O que é do artista e não do preset: favoritos, a política dos volumes, os volumes
//! lembrados por preset e o preset activo — `estado.json` na raiz do utilizador.

use crate::modelo::{Erro, Volumes};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// O nome do ficheiro de estado na raiz do utilizador.
pub const FICHEIRO_ESTADO: &str = "estado.json";

/// A versão do `estado.json`. `0` (ausente) = antes da Opacity do Wet Paint (10/10): o `opacity` lembrado
/// era a FORÇA partilhada, sem leitor no Wet Paint — ao ler, cai (fica a do preset). `1` = a Opacity do
/// Wet Paint.
pub const VERSAO_DO_ESTADO: u32 = 1;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EstadoDoUtilizador {
    #[serde(default)]
    pub favoritos: BTreeSet<String>,
    #[serde(default)]
    pub politica: Politica,
    #[serde(default)]
    pub lembrados: BTreeMap<String, Lembrado>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ativo: Option<String>,
    /// [`VERSAO_DO_ESTADO`] ao ler e ao gravar; `0` só num ficheiro de antes dela.
    #[serde(default)]
    pub versao: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Um estado NOVO nasce na versão de agora; só um ficheiro de antes dela, sem o campo, lê `0`.
impl Default for EstadoDoUtilizador {
    fn default() -> Self {
        Self {
            favoritos: BTreeSet::new(),
            politica: Politica::default(),
            lembrados: BTreeMap::new(),
            ativo: None,
            versao: VERSAO_DO_ESTADO,
            extra: BTreeMap::new(),
        }
    }
}

impl EstadoDoUtilizador {
    /// Lê `estado.json` da raiz. Ausente = vazio.
    ///
    /// # Errors
    /// [`Erro::Json`] se o ficheiro existe e não se lê (não se apaga o que o artista tinha).
    pub fn ler(raiz: &Path) -> Result<Self, Erro> {
        match std::fs::read(raiz.join(FICHEIRO_ESTADO)) {
            Ok(b) => {
                let mut e: Self =
                    serde_json::from_slice(&b).map_err(|e| Erro::Json(e.to_string()))?;
                if e.versao < 1 {
                    for l in e.lembrados.values_mut() {
                        l.volumes.opacity = None;
                    }
                }
                e.versao = VERSAO_DO_ESTADO;
                Ok(e)
            }
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
        let atual = Self {
            versao: VERSAO_DO_ESTADO,
            ..self.clone()
        };
        let texto = serde_json::to_string_pretty(&atual).map_err(|e| Erro::Json(e.to_string()))?;
        let tmp = raiz.join(format!("{FICHEIRO_ESTADO}.tmp"));
        std::fs::write(&tmp, texto).map_err(disco)?;
        std::fs::rename(&tmp, raiz.join(FICHEIRO_ESTADO)).map_err(disco)
    }

    /// O artista escolheu este preset: passa a activo. (Os «recentes» saíram a 09/10, pedido do dono: o
    /// menu repetia os nomes; um `estado.json` antigo com eles guarda-os em `extra` e nada os lê.)
    pub fn usar(&mut self, relativo: &str) {
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

#[cfg(test)]
mod tests_versao {
    use super::*;

    /// **Um `estado.json` de antes da Opacity do Wet Paint** (smoke do dono 10/10: o preview claro e a tela
    /// com a cor cheia): o `opacity` lembrado era a FORÇA partilhada (1,0), sem leitor no Wet Paint; lido
    /// como a Opacity nova, punha a cor cheia por cima do preset. Sem versão ⇒ esse campo cai (fica a do
    /// preset) e o estado sai com a versão de agora; com a versão, o `opacity` lembrado vale.
    #[test]
    fn um_estado_antigo_nao_traz_a_forca_como_opacity() {
        let raiz = std::env::temp_dir().join(format!("ph2d_estado_versao_{}", std::process::id()));
        std::fs::create_dir_all(&raiz).unwrap();
        let antigo = r#"{"politica":"volumes_and_mode","lembrados":{"a.png":{"volumes":{"size_px":60.0,"opacity":1.0,"water":0.5},"paint_mode":"Paint"}}}"#;
        std::fs::write(raiz.join(FICHEIRO_ESTADO), antigo).unwrap();
        let e = EstadoDoUtilizador::ler(&raiz).unwrap();
        let v = &e.lembrados["a.png"].volumes;
        assert_eq!(v.opacity, None, "a força antiga não entra como Opacity");
        assert_eq!(
            (v.size_px, v.water),
            (Some(60.0), Some(0.5)),
            "o resto fica"
        );
        assert_eq!(e.versao, VERSAO_DO_ESTADO);
        let mut novo = e.clone();
        novo.lembrados.get_mut("a.png").unwrap().volumes.opacity = Some(0.3);
        novo.gravar(&raiz).unwrap();
        let relido = EstadoDoUtilizador::ler(&raiz).unwrap();
        assert_eq!(
            relido.lembrados["a.png"].volumes.opacity,
            Some(0.3),
            "com versão, vale"
        );
        std::fs::remove_dir_all(&raiz).ok();
    }
}
