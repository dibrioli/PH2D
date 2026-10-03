//! ⭐ **Os céus que vêm com o produto** — os oito do Blender (Poly Haven, CC0; `ceus/LICENSE-CC0.txt`),
//! a `1K`, `1,8 MB` ao todo. Embarcados no binário: abrem no celular e na web sem sistema de ficheiros.

use crate::Panorama;

/// Um céu embarcado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Embarcado {
    Cidade,
    Patio,
    Floresta,
    Interior,
    Noite,
    Estudio,
    Nascer,
    Por,
}

impl Embarcado {
    /// Todos, na ordem em que o painel os oferece.
    pub const TODOS: [Self; 8] = [
        Self::Estudio,
        Self::Interior,
        Self::Cidade,
        Self::Patio,
        Self::Floresta,
        Self::Nascer,
        Self::Por,
        Self::Noite,
    ];

    /// O id estável (o nome do ficheiro) — é o que o i18n e quem grava a escolha usam.
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Cidade => "city",
            Self::Patio => "courtyard",
            Self::Floresta => "forest",
            Self::Interior => "interior",
            Self::Noite => "night",
            Self::Estudio => "studio",
            Self::Nascer => "sunrise",
            Self::Por => "sunset",
        }
    }

    /// Os bytes do EXR.
    #[must_use]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Cidade => include_bytes!("../ceus/city.exr"),
            Self::Patio => include_bytes!("../ceus/courtyard.exr"),
            Self::Floresta => include_bytes!("../ceus/forest.exr"),
            Self::Interior => include_bytes!("../ceus/interior.exr"),
            Self::Noite => include_bytes!("../ceus/night.exr"),
            Self::Estudio => include_bytes!("../ceus/studio.exr"),
            Self::Nascer => include_bytes!("../ceus/sunrise.exr"),
            Self::Por => include_bytes!("../ceus/sunset.exr"),
        }
    }

    /// O panorama decodificado.
    ///
    /// # Panics
    /// Nunca com os ficheiros embarcados (há gate: `os_embarcados_decodificam`).
    #[must_use]
    pub fn panorama(self) -> Panorama {
        Panorama::de_exr(self.bytes()).expect("o céu embarcado decodifica")
    }
}
