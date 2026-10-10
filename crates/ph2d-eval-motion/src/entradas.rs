//! A bomba e a [`Fita`] das entradas (doc 128 §3.3): gravar ao cozer para a frente, repor no
//! replay do scrub. As duas rotas (CPU e placa) passam por aqui.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::external::{All, INPUT_COUNT, INPUT_PREFIX, INPUT_VALUE};
use std::collections::BTreeMap;

/// As contagens dos gatilhos: as de ANTES do tique corrente e as de agora. Derivar duas vezes no
/// mesmo tique (as sub-passadas da placa, um re-cozimento) dá os MESMOS pulsos.
#[derive(Default)]
pub struct Pulsos {
    antes: BTreeMap<String, Vec<f32>>,
    agora: BTreeMap<String, Vec<f32>>,
    tique: Option<u64>,
}

fn contagens(all: &All) -> BTreeMap<String, Vec<f32>> {
    all.iter()
        .filter(|(nome, _)| nome.starts_with(INPUT_PREFIX))
        .filter_map(|(nome, e)| match e.value.get(INPUT_COUNT) {
            Some(Column::Scalar(n)) => Some((nome.clone(), n.clone())),
            _ => None,
        })
        .collect()
}

/// O que o replay de um scrub põe no cozimento, tique a tique.
pub struct Replay {
    vivas: All,
    estado: Option<All>,
    na_fita: Option<bool>,
}

impl MotionCookPump {
    /// Grava o publicado AGORA como as entradas de `tick`. Um RAMO (o tique já estava gravado com
    /// outras entradas) esquece o futuro do anel também: os checkpoints dele são de outra história.
    pub fn grava_entradas(&mut self, tick: u64) -> Gravacao {
        self.deriva_pulsos(tick);
        let g = self.fita.grava(tick, self.cook.externals());
        if g == Gravacao::Ramo {
            self.ring.esquece_depois_de(tick);
        }
        g
    }

    /// ⭐ **O PULSO de cada gatilho, neste tique** (doc 128 §3.2): `1` na linha cuja contagem
    /// mudou desde o fim do tique anterior. Quem escreve só conta — o pulso é da bomba, que é quem
    /// sabe quantos tiques um quadro coze (dois tiques num quadro: o pulso fica no primeiro; um
    /// quadro sem tique: fica para o próximo que cozer).
    fn deriva_pulsos(&mut self, tick: u64) {
        if self.pulsos.tique != Some(tick) {
            self.pulsos.antes = self.pulsos.agora.clone();
            self.pulsos.tique = Some(tick);
        }
        for (nome, n) in contagens(self.cook.externals()) {
            let antes = self.pulsos.antes.get(&nome);
            let v: Vec<f32> = n
                .iter()
                .enumerate()
                .map(|(i, &c)| f32::from(antes.and_then(|a| a.get(i)).copied().unwrap_or(0.0) != c))
                .collect();
            let s = Stream::new(n.len())
                .with(INPUT_COUNT, Column::Scalar(n.clone()))
                .with(INPUT_VALUE, Column::Scalar(v));
            self.cook.set_external(nome.clone(), s);
            self.pulsos.agora.insert(nome, n);
        }
    }

    /// Depois de um scrub para `alvo`: as contagens voltam às da época (as de `alvo` e as do tique
    /// antes dele), para o próximo tique pulsar contra a história certa.
    pub(crate) fn pulsos_depois_do_scrub(&mut self, alvo: u64) {
        self.pulsos.agora = contagens(self.cook.externals());
        self.pulsos.antes = alvo
            .checked_sub(1)
            .and_then(|t| self.fita.estado_em(t))
            .map(|e| contagens(&e))
            .unwrap_or_default();
        self.pulsos.tique = Some(alvo);
    }

    /// A fita, só para leitura (os testes e a medição do custo por tique).
    #[must_use]
    pub fn fita(&self) -> &Fita {
        &self.fita
    }

    /// Começa um replay: guarda o publicado agora (o que vale para lá do que foi gravado).
    #[must_use]
    pub fn comeca_o_replay(&self, anchor: u64) -> Replay {
        Replay {
            vivas: self.cook.externals().clone(),
            estado: self.fita.estado_em(anchor),
            na_fita: None,
        }
    }

    /// Põe no cozimento as entradas de `t` — as da fita onde ela cobre, as vivas fora dela. Só
    /// troca o conjunto quando ele muda (uma entrada parada não custa nada no replay).
    pub fn entradas_do_replay(&mut self, replay: &mut Replay, t: u64) {
        if self.fita.cobre(t) {
            let mudou = match replay.estado.as_mut() {
                Some(e) => self.fita.avanca(t, e),
                None => {
                    replay.estado = self.fita.estado_em(t);
                    true
                }
            };
            if mudou || replay.na_fita != Some(true) {
                if let Some(e) = &replay.estado {
                    self.cook.replace_externals(e.clone());
                }
                replay.na_fita = Some(true);
            }
        } else if replay.na_fita != Some(false) {
            self.cook.replace_externals(replay.vivas.clone());
            replay.na_fita = Some(false);
        }
    }
}
