//! **Quando o worker serve uma imagem** (filho de [`super`] por LOC): a regra pura, com o relógio injectado, que o
//! [`super::DoWorker::serve`] consulta num passo fechado. O gate do agendamento corre-a sobre um relógio SIMULADO.

use super::STEP_S;

/// O tecto da parede do worker que a cópia do espelho pode ocupar quando a água está atrasada.
pub(super) const FRACCAO_DA_COPIA: f64 = 0.2;

/// O que o worker sabe no instante da pergunta.
#[derive(Clone, Copy, Debug)]
pub(super) struct Momento {
    /// O relógio do worker, s.
    pub(super) agora_s: f64,
    /// A dívida do laço, s (`acc` do `worker_loop`, presa em `MAX_BEHIND_S`).
    pub(super) acc: f64,
    pub(super) sim_corre: bool,
}

/// A última cópia: o que custou e quando acabou.
#[derive(Default)]
pub(super) struct Agenda {
    ultima_ms: f64,
    fim_s: Option<f64>,
}

impl Agenda {
    /// A imagem pedida corre agora? Sim se a água não anda, se a cópia cabe na FOLGA do passo (`STEP_S − acc`, só
    /// há folga com a água em dia) ou, com a água atrasada, se desde a última cópia passou o bastante para a cópia
    /// ocupar ≤ [`FRACCAO_DA_COPIA`] da parede do worker. Sem média que envelhece: a última cópia é a previsão, e
    /// uma cópia cara (a 1.ª é a tela inteira) só adia a seguinte `4×` o seu custo (Bug #60, DF-52).
    pub(super) fn deve_servir(&self, m: Momento) -> bool {
        let copia_s = self.ultima_ms / 1000.0;
        if !m.sim_corre || m.acc + copia_s <= STEP_S {
            return true;
        }
        match self.fim_s {
            None => true,
            Some(fim) => m.agora_s - fim >= copia_s * (1.0 / FRACCAO_DA_COPIA - 1.0),
        }
    }

    /// Uma cópia de `ms` acabou em `fim_s` (o relógio do [`Momento`]).
    pub(super) fn copiou(&mut self, ms: f64, fim_s: f64) {
        self.ultima_ms = ms;
        self.fim_s = Some(fim_s);
    }
}

#[cfg(test)]
#[path = "offthread_agenda_tests.rs"]
mod tests;
