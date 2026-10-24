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

use bevy_ecs::world::World;
pub use ph2d_ecs::canal_palavra::Palavra;

use crate::{PropKind, TrackView};

/// **Quanto vale a propriedade de cada row, neste quadro.** Chaveado pelo alvo opaco da track
/// ([`TrackView::target`]), que é único por row — `entity` sozinho não serve (um objecto com seis
/// tracks tem seis números diferentes).
///
/// Vazio é o normal: um documento sem tracks, ou um em que nenhuma delas nomeia um escalar do
/// mundo, publica nada e a dope-sheet desenha exactamente como sempre desenhou.
///
/// Um canal DISCRETO ([`ph2d_ecs::CampoInfo::discreto`]) — uma escolha, uma camada, um script, um
/// texto — leva também a PALAVRA de cada valor que a row mostra (o de agora e o de cada chave); o
/// painel traduz a chave e escreve «Tag», «Front», «bob.luau», nunca «3.00» (F27 F).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackValues {
    valores: BTreeMap<u64, f32>,
    palavras: BTreeMap<(u64, u32), Palavra>,
}

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
        self.valores.get(&target).copied()
    }

    /// ⭐ **A palavra que o valor `v` da row `target` mostra**, se a publicação a trouxe (`None` = o
    /// número é a resposta, ou um texto «nenhum»).
    #[must_use]
    pub fn palavra(&self, target: u64, v: f32) -> Option<&Palavra> {
        self.palavras.get(&(target, v.round().to_bits()))
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
        sample: impl FnMut(u64, PropKind) -> Option<f32>,
    ) {
        self.publish_com_mundo(tracks, None, sample);
    }

    /// [`Self::publish`] com o mundo: cada row discreta leva a palavra do seu valor e a de cada
    /// chave, pela porta [`ph2d_ecs::canal_palavra::palavra`] (o escopo continua a ser as ROWS).
    pub fn publish_com_mundo(
        &mut self,
        tracks: &[TrackView],
        mundo: Option<&World>,
        mut sample: impl FnMut(u64, PropKind) -> Option<f32>,
    ) {
        self.valores.clear();
        self.palavras.clear();
        for t in tracks {
            let agora = sample(t.entity, t.prop);
            if let Some(v) = agora {
                self.valores.insert(t.target.get(), v);
            }
            let (Some(mundo), PropKind::Field(k)) = (mundo, t.prop) else {
                continue;
            };
            for v in agora.into_iter().chain(t.keys.iter().map(|k| k.value)) {
                let chave = (t.target.get(), v.round().to_bits());
                if self.palavras.contains_key(&chave) {
                    continue;
                }
                if let Some(p) = ph2d_ecs::canal_palavra::palavra(mundo, t.entity, k, v) {
                    self.palavras.insert(chave, p);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "track_values_tests.rs"]
mod tests;

impl crate::TimelineViewSnapshot {
    /// O nome que a row de `prop` mostra no lugar do rótulo da família (uma propriedade que o
    /// script declara chama-se como o script a chamou); `None` = o rótulo de sempre.
    #[must_use]
    pub fn channel_name(&self, prop: crate::PropKind) -> Option<&str> {
        match prop {
            crate::PropKind::Field(k) => self.channel_names.get(&k).map(String::as_str),
            _ => None,
        }
    }
}
