//! **A fita das entradas** (ciclo 19, [doc 128 §3.3](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md)):
//! o que estava publicado em cada tique cozido para a frente, para o scrub o repetir.
//!
//! O anel de checkpoints guarda o estado do GRAFO; os externals vêm de fora (o jogo, o editor) e
//! não voltam atrás com o editor. Sem a fita, o replay coze cada tique passado com o valor de
//! AGORA (medido: doc 128 §5.1).
//!
//! Só se gravam MUDANÇAS, por revisão: uma entrada parada não custa nada por tique, e as colunas
//! são partilhadas (`Arc`) com o que já estava publicado.

use ph2d_nodegraph::external::{All, External};
use std::collections::BTreeMap;

/// `(nome, Some(novo) | None = deixou de estar publicado)`.
type Mudancas = Vec<(String, Option<External>)>;

/// O que a gravação de um tique fez.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gravacao {
    /// O tique já estava gravado com as mesmas revisões (a tocar sobre chão já gravado).
    Igual,
    /// Um tique novo, para lá do último gravado.
    Nova,
    /// O tique já estava gravado com OUTRAS entradas: o futuro velho foi esquecido a partir
    /// daqui — quem guarda estado desse futuro (o anel) tem de o esquecer também.
    Ramo,
}

/// O orçamento da fita (a medição por tique: doc 128 §5).
pub const FITA_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
pub struct Fita {
    /// O estado antes da primeira marca (e, depois de [`Self::dobra`], o de `base_ate`).
    base: All,
    base_ate: Option<u64>,
    marcas: BTreeMap<u64, Mudancas>,
    /// `[primeiro, ultimo]` = os tiques gravados.
    primeiro: Option<u64>,
    ultimo: Option<u64>,
    bytes: usize,
    orcamento: usize,
}

fn peso(m: &Mudancas) -> usize {
    m.iter()
        .map(|(n, e)| n.len() + e.as_ref().map_or(0, |e| e.value.approx_bytes() + 16))
        .sum()
}

fn mesmas_revisoes(a: &All, b: &All) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|((na, ea), (nb, eb))| na == nb && ea.rev == eb.rev)
}

fn aplica(estado: &mut All, mudancas: &Mudancas) {
    for (nome, e) in mudancas {
        match e {
            Some(e) => {
                estado.insert(nome.clone(), e.clone());
            }
            None => {
                estado.remove(nome);
            }
        }
    }
}

fn diferenca(de: &All, para: &All) -> Mudancas {
    let mut out = Mudancas::new();
    for (nome, e) in para {
        if de.get(nome).is_none_or(|d| d.rev != e.rev) {
            out.push((nome.clone(), Some(e.clone())));
        }
    }
    for nome in de.keys() {
        if !para.contains_key(nome) {
            out.push((nome.clone(), None));
        }
    }
    out
}

impl Fita {
    #[must_use]
    pub fn new() -> Self {
        Self {
            orcamento: FITA_BYTES,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn cobre(&self, t: u64) -> bool {
        matches!((self.primeiro, self.ultimo), (Some(p), Some(u)) if p <= t && t <= u)
    }

    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn set_orcamento(&mut self, bytes: usize) {
        self.orcamento = bytes;
    }

    /// O que estava publicado em `t`, ou `None` fora do que foi gravado (o replay usa então o
    /// publicado agora).
    #[must_use]
    pub fn estado_em(&self, t: u64) -> Option<All> {
        self.cobre(t).then(|| self.estado_ate(t + 1))
    }

    /// Avança `estado` (o de `t − 1`) para `t`; `true` se mudou — o replay anda tique a tique sem
    /// refazer a soma desde a base.
    pub fn avanca(&self, t: u64, estado: &mut All) -> bool {
        match self.marcas.get(&t) {
            Some(m) => {
                aplica(estado, m);
                true
            }
            None => false,
        }
    }

    /// Grava o que está publicado AGORA como o estado de `t`.
    pub fn grava(&mut self, t: u64, agora: &All) -> Gravacao {
        let resultado = if self.cobre(t) {
            if self
                .estado_em(t)
                .is_some_and(|s| mesmas_revisoes(&s, agora))
            {
                return Gravacao::Igual;
            }
            Gravacao::Ramo
        } else if self.primeiro.is_some_and(|p| t < p) {
            Gravacao::Ramo
        } else {
            Gravacao::Nova
        };
        if resultado == Gravacao::Ramo {
            self.esquece_de(t);
        }
        if self.base_ate.is_some_and(|b| t <= b) {
            // Dentro do que já foi dobrado: o tique passa a ser a nova base.
            self.base = agora.clone();
            self.base_ate = Some(t);
        } else {
            let m = diferenca(&self.estado_ate(t), agora);
            if !m.is_empty() {
                self.bytes += peso(&m);
                self.marcas.insert(t, m);
            }
        }
        self.primeiro = self.primeiro.or(Some(t));
        self.ultimo = Some(t);
        while self.bytes > self.orcamento && !self.marcas.is_empty() {
            self.dobra();
        }
        resultado
    }

    /// O estado no fim do que está gravado ANTES de `t`.
    fn estado_ate(&self, t: u64) -> All {
        let mut estado = self.base.clone();
        for m in self.marcas.range(..t).map(|(_, m)| m) {
            aplica(&mut estado, m);
        }
        estado
    }

    /// Esquece `t` e tudo depois; antes do primeiro tique gravado, esquece tudo.
    fn esquece_de(&mut self, t: u64) {
        if self.primeiro.is_none_or(|p| t <= p) {
            self.clear_keeping_budget();
            return;
        }
        for m in self.marcas.split_off(&t).values() {
            self.bytes -= peso(m);
        }
        self.ultimo = Some(t - 1);
    }

    /// ⚠️ **Acima do orçamento, a marca mais velha DOBRA-SE na base**: um scrub para antes dela
    /// repete o estado da base em vez do que se publicou na época — o limite nomeado (doc 128 §5).
    fn dobra(&mut self) {
        let Some((t, m)) = self.marcas.pop_first() else {
            return;
        };
        self.bytes -= peso(&m);
        aplica(&mut self.base, &m);
        self.base_ate = Some(t);
    }

    fn clear_keeping_budget(&mut self) {
        let orcamento = self.orcamento;
        *self = Self {
            orcamento,
            ..Self::default()
        };
    }

    pub fn clear(&mut self) {
        self.clear_keeping_budget();
    }
}

#[cfg(test)]
#[path = "fita_tests.rs"]
mod tests;
