//! ⭐⭐ **A condução dos campos de família pela curva** — o livro do autorado que as famílias
//! VETOR, FLIP e FISICA partilham (`ConducaoVetor`, `ConducaoFlip`, `ConducaoFisica`): o que a
//! curva escreve a tocar é pré-visualização, o undo e o save fotografam o AUTORADO.
//!
//! A lei de esquecer (auditoria F31): enquanto a timeline TOCA, todo campo que a curva escreveu
//! neste quadro continua conduzido, mesmo que o valor não tenha mudado (o relógio repete o instante
//! quando o quadro de ecrã é mais curto que o da animação). Parada (pausa, scrub parado), só conta o
//! que MUDOU — a curva que já não muda nada vira documento.

use std::collections::{BTreeMap, BTreeSet};

use crate::FieldKey;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoDosCanais {
    autorado: BTreeMap<(u64, FieldKey), f32>,
    neste_quadro: BTreeSet<(u64, FieldKey)>,
    tocando: bool,
}

impl ConducaoDosCanais {
    /// A timeline está a tocar neste quadro? Quem drena a timeline di-lo antes do `aplicar`.
    pub fn a_tocar(&mut self, tocando: bool) {
        self.tocando = tocando;
    }

    /// A curva escreveu `(e, kk)`: o documento valia `antes`, ficou `ficou`. Devolve se mudou.
    pub fn escreveu(&mut self, e: u64, kk: FieldKey, antes: f32, ficou: f32) -> bool {
        let mudou = antes != ficou;
        if mudou || self.tocando {
            self.conduziu(e, kk, antes);
        }
        mudou
    }

    /// A curva conduz `(e, kk)`; `antes` era o valor do documento (o primeiro de uma corrida fica).
    pub fn conduziu(&mut self, e: u64, kk: FieldKey, antes: f32) {
        self.autorado.entry((e, kk)).or_insert(antes);
        self.neste_quadro.insert((e, kk));
    }

    /// Esquece quem não foi conduzido neste quadro (chamado onde o `PreviewDrive::settle` é).
    pub fn settle(&mut self) {
        let neste = std::mem::take(&mut self.neste_quadro);
        self.autorado.retain(|k, _| neste.contains(k));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.autorado.is_empty()
    }

    /// Os valores AUTORADOS, por `(entidade, campo)`.
    pub fn autorado(&self) -> impl Iterator<Item = (u64, FieldKey, f32)> + '_ {
        self.autorado.iter().map(|(&(e, kk), &v)| (e, kk, v))
    }

    /// Os campos conduzidos, por `(entidade, campo)`.
    pub fn campos(&self) -> impl Iterator<Item = (u64, FieldKey)> + '_ {
        self.autorado.keys().copied()
    }

    /// Quantos campos estão conduzidos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.autorado.len()
    }
}
