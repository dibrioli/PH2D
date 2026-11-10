//! Os assets de som do sprite, decodificados uma vez cada (fora do Play), com os picos da onda.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use ph2d_audio::SampleData;

use crate::picos::{Pico, picos};

/// Um asset pronto a tocar e a desenhar.
pub struct Som {
    pub dados: SampleData,
    pub picos: Arc<[Pico]>,
}

impl Som {
    /// A duração em ms da fonte.
    #[must_use]
    pub fn dur_ms(&self) -> u64 {
        let rate = u64::from(self.dados.format().sample_rate.max(1));
        self.dados.frame_count() as u64 * 1000 / rate
    }
}

/// Os sons por asset (a referência do `FaixaDeAudio::asset`).
#[derive(Default)]
pub struct Sons {
    mapa: BTreeMap<String, Som>,
    /// Os assets que não se leram: não se tenta de novo a cada quadro.
    falhou: BTreeSet<String>,
}

impl Sons {
    /// Guarda `dados` sob `asset` e calcula os picos (uma vez).
    pub fn insere(&mut self, asset: &str, dados: SampleData) {
        let picos = picos(&dados).into();
        self.falhou.remove(asset);
        self.mapa.insert(asset.to_owned(), Som { dados, picos });
    }

    /// Decodifica os bytes de um ficheiro de som (WAV, FLAC, Ogg, MP3, Opus) sob `asset`.
    pub fn carrega_bytes(&mut self, asset: &str, bytes: &[u8]) -> Result<(), String> {
        let d = ph2d_audio_decode::decode_any::decode(bytes)?;
        self.insere(asset, d);
        Ok(())
    }

    /// Lê e decodifica `asset`, relativo a `base` (a pasta do projeto) ou absoluto. Já carregado:
    /// não lê de novo.
    pub fn carrega(&mut self, base: Option<&Path>, asset: &str) -> Result<(), String> {
        if self.mapa.contains_key(asset) {
            return Ok(());
        }
        if self.falhou.contains(asset) {
            return Err(asset.to_owned());
        }
        let p = Path::new(asset);
        let p = match base {
            Some(b) if p.is_relative() => b.join(p),
            _ => p.to_path_buf(),
        };
        let r = std::fs::read(&p)
            .map_err(|e| format!("{}: {e}", p.display()))
            .and_then(|b| self.carrega_bytes(asset, &b));
        if r.is_err() {
            self.falhou.insert(asset.to_owned());
        }
        r
    }

    /// O som de `asset`, se carregado.
    #[must_use]
    pub fn som(&self, asset: &str) -> Option<&Som> {
        self.mapa.get(asset)
    }
}
