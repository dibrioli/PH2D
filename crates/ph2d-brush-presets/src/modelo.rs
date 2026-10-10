//! O que um preset diz: [`Parametros`] (o JSON) + a [`Miniatura`] + a [`Origem`] + as texturas.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// A versão do formato. Sobe SÓ numa mudança que um leitor velho leria errado; campo novo não a sobe.
pub const SCHEMA: u32 = 1;

/// O lado da miniatura (px). A mesma do Rebelle e do Krita: cabe na grelha do navegador a 2× de escala.
pub const LADO_MINIATURA: u32 = 256;

/// De onde o ficheiro veio (`tEXt` `ph2d-origin`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origem {
    #[default]
    Fabrica,
    Utilizador,
    Importado,
}

impl Origem {
    pub(crate) fn texto(self) -> &'static str {
        match self {
            Origem::Fabrica => "factory",
            Origem::Utilizador => "user",
            Origem::Importado => "imported",
        }
    }

    pub(crate) fn de_texto(s: &str) -> Option<Self> {
        match s {
            "factory" => Some(Origem::Fabrica),
            "user" => Some(Origem::Utilizador),
            "imported" => Some(Origem::Importado),
            _ => None,
        }
    }
}

/// Os cinco volumes do topo do painel (02 §5). `None` = o preset não os fixa: fica o do meio.
/// `size_px` é o DIÂMETRO que se vê (o anel do cursor), nunca o raio do motor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Volumes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_px: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// Volumes de ondas futuras: lidos e regravados intactos.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// O JSON do `iTXt` `ph2d-brush`. Os mapas são por NOME (a chave do `KNOB_DEFS`, o nome do campo do
/// pincel), nunca por índice: reordenar a tabela do motor não estraga um preset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parametros {
    pub schema: u32,
    /// O meio-alvo (`"WetPaint"`).
    pub media: String,
    pub name: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub volumes: Volumes,
    /// O modo de pintura (a ferramenta do meio: `"Paint"`, `"Smear"`…). `None` = não o fixa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint_mode: Option<String>,
    /// Os knobs do motor do meio, pela chave do registo.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub engine: BTreeMap<String, f64>,
    /// O sub-conjunto do pincel que o meio lê, pelo nome do campo.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub brush: BTreeMap<String, Value>,
    /// A lei medida raio do motor → largura que se vê, `[raio, largura]` (05 §1.1); regera-se com o motor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub width_table: Vec<[f64; 2]>,
    /// Campos de ondas futuras: lidos e regravados intactos.
    /// **A inclinação da tela que o pincel traz** (W1e, F4 — dono 09/10: o preset guarda o Tilt, o
    /// artista continua a mudá-lo). `None` = um preset de antes dela: aplicá-lo NÃO mexe na tela (o
    /// formato não sobe de versão — o campo é opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tilt: Option<Tilt>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Parametros {
    /// Um preset vazio de um meio: nada fixado além do nome.
    #[must_use]
    pub fn novo(media: &str, group: &str, name: &str) -> Self {
        Self {
            schema: SCHEMA,
            media: media.to_owned(),
            name: name.to_owned(),
            group: group.to_owned(),
            volumes: Volumes::default(),
            paint_mode: None,
            engine: BTreeMap::new(),
            brush: BTreeMap::new(),
            width_table: Vec::new(),
            tilt: None,
            extra: BTreeMap::new(),
        }
    }
}

/// A inclinação da tela: ligada, o anel do dial (`0..=8`, a força) e o raio (`0..12`, a direcção, 3 =
/// para baixo) — os três números do dial Tilt do Wet Paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tilt {
    pub on: bool,
    pub ring: u8,
    pub spoke: u8,
}

/// A imagem do ficheiro: RGBA8 não pré-multiplicado, linha a linha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Miniatura {
    pub largura: u32,
    pub altura: u32,
    pub rgba: Vec<u8>,
}

/// Um ficheiro de preset inteiro.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub parametros: Parametros,
    pub origem: Origem,
    pub miniatura: Miniatura,
    /// Texturas NOSSAS, por id (o PNG da textura, intacto).
    pub texturas: BTreeMap<String, Vec<u8>>,
}

/// Porque um ficheiro não se leu. Cada variante diz o que o artista tem de saber.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Erro {
    /// Não começa pela assinatura PNG.
    NaoEPng,
    /// Um chunk corta-se a meio ou o CRC não confere (o ficheiro foi truncado ou alterado).
    ChunkCorrompido(String),
    /// É um PNG, mas sem o `iTXt` `ph2d-brush` (um editor de imagem regravou-o e deitou o texto fora).
    SemParametros,
    /// Feito por uma versão mais nova do PH2D.
    SchemaFuturo(u32),
    /// O JSON não se lê.
    Json(String),
    /// A imagem usa uma codificação que este leitor não cobre (só RGB/RGBA de 8 bits, sem entrelaçado).
    MiniaturaIlegivel(String),
    /// Ler ou gravar no disco falhou.
    Disco(String),
}

impl std::fmt::Display for Erro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erro::NaoEPng => write!(f, "não é um PNG"),
            Erro::ChunkCorrompido(c) => write!(f, "chunk {c} corrompido"),
            Erro::SemParametros => write!(f, "PNG sem os parâmetros do pincel (ph2d-brush)"),
            Erro::SchemaFuturo(s) => {
                write!(f, "schema {s} é de uma versão mais nova (esta lê {SCHEMA})")
            }
            Erro::Json(e) => write!(f, "parâmetros ilegíveis: {e}"),
            Erro::MiniaturaIlegivel(e) => write!(f, "miniatura ilegível: {e}"),
            Erro::Disco(e) => write!(f, "disco: {e}"),
        }
    }
}

impl std::error::Error for Erro {}
