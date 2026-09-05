//! ⭐⭐⭐ **O VALOR VIVO de cada row** — o que a propriedade animada VALE agora, para a dope-sheet
//! o mostrar ao lado do nome (report do Enio, 2026-09-04: *"o painel não mostra as propriedades
//! animadas — os números não mudam em tempo real com a animação"*).
//!
//! # Por que ele NÃO é amostrado da curva
//!
//! O painel já sabe amostrar a curva desta row (é o que desenha o gráfico), e ler dali seria de
//! graça. Seria também **um espelho**: uma curva que diz `0` sobre uma forma que continua opaca
//! escreveria `0,00` na row e o artista não veria defeito nenhum. Foi exactamente esse o par de
//! reports de 2026-09-04 — *a curva estava certa e o desenho não* —, e um readout que repete a
//! curva é o instrumento que **não** teria acusado.
//!
//! ⇒ O número vem do **MUNDO**, pela mesma porta que a tecla K usa para capturar uma pose
//! (`sample_prop_value` na shell). O que a row mostra é o que o objecto TEM.
//!
//! # Por que a shell preenche, e não a `rebuild`
//!
//! Terceiro campo com a forma do [`crate::TimelineViewSnapshot::object_names`], e pela mesma razão:
//! o documento guarda a CURVA e aponta para os objectos por `wire_id`; quem sabe o que uma
//! propriedade vale depois de todos os motores escreverem é a shell. Ela preenche depois da
//! `rebuild`.
//!
//! ⚠️ **A publicação é uma PORTA só** ([`TrackValues::publish`]) — ela limpa e re-preenche a partir
//! das rows que o snapshot tem. Um `set` avulso deixaria entradas de um objecto que saiu das
//! tracks, e o `bevy` recicla bits: a row seguinte mostraria o número de outro objecto.

use std::collections::BTreeMap;

use crate::{PropKind, TrackView};

/// **Quanto vale a propriedade de cada row, neste quadro.** Chaveado pelo alvo opaco da track
/// ([`TrackView::target`]), que é único por row — `entity` sozinho não serve (um objecto com seis
/// tracks tem seis números diferentes).
///
/// Vazio é o normal: um documento sem tracks, ou um em que nenhuma delas nomeia um escalar do
/// mundo, publica nada e a dope-sheet desenha exactamente como sempre desenhou.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackValues(BTreeMap<u64, f32>);

impl TrackValues {
    /// **Quanto vale esta row agora**, ou `None` — a porta única de leitura.
    ///
    /// `None` tem três causas, todas legítimas e indistinguíveis de propósito (o painel faz o
    /// mesmo com as três: não desenha número nenhum): a entidade morreu, o componente que carrega
    /// a propriedade não está nela, ou o canal **não é um escalar do mundo** — `TimeRemap` é um
    /// relógio e `Position` é uma distância ao longo de uma trajectória, e as duas recusam na
    /// porta de amostragem, com o motivo escrito lá.
    #[must_use]
    pub fn get(&self, target: u64) -> Option<f32> {
        self.0.get(&target).copied()
    }

    /// **Re-publica os valores das rows que este snapshot tem** — limpa e preenche numa passagem.
    ///
    /// `sample` é a porta do mundo (na shell, `sample_prop_value`): ela responde `None` quando não
    /// há número a mostrar, e esse `None` viaja até ao pintor como ausência de entrada.
    ///
    /// ⚠️ **O escopo são as ROWS, nunca a cena** — a pergunta é sobre o que vai ser pintado, e uma
    /// varredura do mundo publicaria centenas de números que ninguém lê. É a mesma lei que o
    /// publicador de nomes já segue.
    pub fn publish(
        &mut self,
        tracks: &[TrackView],
        mut sample: impl FnMut(u64, PropKind) -> Option<f32>,
    ) {
        self.0.clear();
        for t in tracks {
            if let Some(v) = sample(t.entity, t.prop) {
                self.0.insert(t.target.get(), v);
            }
        }
    }
}

#[cfg(test)]
#[path = "track_values_tests.rs"]
mod tests;
