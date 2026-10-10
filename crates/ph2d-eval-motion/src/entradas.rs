//! A bomba e a [`Fita`] das entradas (doc 128 §3.3): gravar ao cozer para a frente, repor no
//! replay do scrub. As duas rotas (CPU e placa) passam por aqui.

use super::*;
use ph2d_nodegraph::external::All;

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
        let g = self.fita.grava(tick, self.cook.externals());
        if g == Gravacao::Ramo {
            self.ring.esquece_depois_de(tick);
        }
        g
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
