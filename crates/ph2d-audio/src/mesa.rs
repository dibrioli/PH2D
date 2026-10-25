//! **A MESA DE SOM como documento** (F30 §B2): o que o artista autora na mesa
//! (`ph2d-panel-audio-mixer`) — a faixa master, os efeitos do master, o ducking e os sub-barramentos.
//! Viaja no `.ph2dproj` como blob com versão própria ([`Mesa::to_bytes`]); os medidores e o
//! *Play Test* são da sessão e ficam fora.
//!
//! Os valores são os que o motor recebe: ganho linear, Hz, pan `-1..1`, o resto `0..1`.

use serde::{Deserialize, Serialize};

use crate::SUB_BUS_COUNT;

/// A versão do blob ([`Mesa::to_bytes`]) — sobe quando a forma de [`Mesa`] muda.
pub const MESA_VERSAO: u32 = 1;

/// Um sub-barramento da mesa (índice = `BusId::SUB_BUSES`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BusMesa {
    pub ganho: f32,
    pub mudo: bool,
    pub solo: bool,
    pub pan: f32,
    /// O passa-baixo (o *Tone*), em Hz.
    pub tom_hz: f32,
    /// O passa-alto (o *Low Cut*), em Hz; 20 Hz = desligado.
    pub corte_baixo_hz: f32,
    pub envio_reverb: f32,
    pub envio_delay: f32,
    pub comp: f32,
}

impl Default for BusMesa {
    fn default() -> Self {
        Self {
            ganho: 1.0,
            mudo: false,
            solo: false,
            pan: 0.0,
            tom_hz: 20_000.0,
            corte_baixo_hz: 20.0,
            envio_reverb: 0.0,
            envio_delay: 0.0,
            comp: 0.0,
        }
    }
}

/// ⭐ **A mesa inteira, como o artista a deixou.**
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mesa {
    pub ganho: f32,
    pub corte_alto_hz: f32,
    pub corte_baixo_hz: f32,
    pub mudo: bool,
    pub pan: f32,
    pub limitador: bool,
    pub reverb: bool,
    pub reverb_tamanho: f32,
    pub reverb_retorno: f32,
    /// As três bandas do EQ (`0.5` = plano), `[grave, médio, agudo]`.
    pub eq: [f32; 3],
    pub delay: bool,
    /// Segundos (`0..1`).
    pub delay_tempo: f32,
    pub delay_feedback: f32,
    pub delay_retorno: f32,
    pub ducking: bool,
    pub ducking_profundidade: f32,
    /// O sub-barramento-chave: todos os outros baixam debaixo dele.
    pub ducking_chave: u8,
    pub bus: [BusMesa; SUB_BUS_COUNT],
}

impl Default for Mesa {
    fn default() -> Self {
        Self {
            ganho: 1.0,
            corte_alto_hz: 20_000.0,
            corte_baixo_hz: 20.0,
            mudo: false,
            pan: 0.0,
            limitador: false,
            reverb: false,
            reverb_tamanho: 0.5,
            reverb_retorno: 0.3,
            eq: [0.5; 3],
            delay: false,
            delay_tempo: 0.25,
            delay_feedback: 0.3,
            delay_retorno: 0.3,
            ducking: false,
            ducking_profundidade: 0.7,
            #[expect(clippy::cast_possible_truncation, reason = "SUB_BUS_COUNT = 4")]
            ducking_chave: (SUB_BUS_COUNT - 1) as u8,
            bus: [BusMesa::default(); SUB_BUS_COUNT],
        }
    }
}

/// O blob da mesa não se lê.
#[derive(Debug, thiserror::Error)]
pub enum MesaIlegivel {
    #[error("versão {0} da mesa desconhecida")]
    Versao(u32),
    #[error("mesa ilegível: {0}")]
    Bytes(#[from] postcard::Error),
}

impl Mesa {
    /// O blob do ficheiro: `(MESA_VERSAO, Mesa)`; a mesa de fábrica grava VAZIO.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        if *self == Self::default() {
            return Vec::new();
        }
        postcard::to_allocvec(&(MESA_VERSAO, self)).unwrap_or_default()
    }

    /// O inverso de [`Self::to_bytes`]: vazio = a mesa de fábrica (um projecto anterior à mesa).
    ///
    /// # Errors
    /// Uma versão desconhecida ou bytes que não são uma mesa.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MesaIlegivel> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let (versao, resto) = postcard::take_from_bytes::<u32>(bytes)?;
        if versao != MESA_VERSAO {
            return Err(MesaIlegivel::Versao(versao));
        }
        Ok(postcard::from_bytes(resto)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mesa_atravessa_o_blob() {
        let mut m = Mesa::default();
        assert!(m.to_bytes().is_empty(), "a de fábrica não grava nada");
        m.bus[2].pan = -0.5;
        m.ducking_chave = 0;
        m.reverb = true;
        let b = m.to_bytes();
        assert_eq!(Mesa::from_bytes(&b).expect("lê"), m);
        assert_eq!(Mesa::from_bytes(&[]).expect("vazio"), Mesa::default());
        let mut outra = b.clone();
        outra[0] = 9;
        assert!(
            Mesa::from_bytes(&outra).is_err(),
            "versão desconhecida recusa"
        );
    }
}
