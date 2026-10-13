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

/// O replay de um scrub, tique a tique: o estado da fita no tique corrente.
pub struct Replay {
    estado: Option<All>,
    /// Já pôs a sua camada? (a de um scrub anterior pode estar lá, e é de outro tique.)
    posto: bool,
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
        self.pulsos.agora = contagens(self.cook.visible_externals());
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

    /// Começa um replay em `anchor`.
    #[must_use]
    pub fn comeca_o_replay(&self, anchor: u64) -> Replay {
        Replay {
            estado: self.fita.estado_em(anchor),
            posto: false,
        }
    }

    /// Mostra no cozimento as entradas de `t` — as da fita onde ela cobre (POR CIMA do publicado,
    /// que fica intacto), as publicadas fora dela. Só troca quando muda.
    pub fn entradas_do_replay(&mut self, replay: &mut Replay, t: u64) {
        self.historia = self.fita.historia_em(t);
        self.tique_das_entradas = Some(t);
        if self.fita.cobre(t) {
            let mudou = match replay.estado.as_mut() {
                Some(e) => self.fita.avanca(t, e),
                None => {
                    replay.estado = self.fita.estado_em(t);
                    true
                }
            };
            if mudou || !replay.posto {
                self.cook.set_externals_replay(replay.estado.clone());
                replay.posto = true;
            }
        } else if self.cook.externals_replaying() || !replay.posto {
            self.cook.set_externals_replay(None);
            replay.posto = true;
        }
    }

    /// ⭐ **A regra de cada tique** (doc 128 §3.3), a MESMA nas duas rotas: um tique antes do último
    /// cozido, ou — com o relógio PARADO — um tique já gravado que não é o último (arrastar a fita
    /// para a frente) REPETE a fita; o resto (tocar, e editar no mesmo tique) GRAVA o publicado,
    /// e um ramo novo esquece o futuro. Devolve a gravação, se gravou.
    pub fn entradas_do_tique(
        &mut self,
        replay: &mut Replay,
        tick: u64,
        ultimo: Option<u64>,
    ) -> Option<Gravacao> {
        let repete = ultimo.is_some_and(|u| tick < u)
            || (!self.a_tocar && ultimo != Some(tick) && self.fita.cobre(tick));
        if repete {
            self.entradas_do_replay(replay, tick);
            self.pulsos_depois_do_scrub(tick);
            return None;
        }
        self.cook.set_externals_replay(None);
        let g = self.grava_entradas(tick);
        self.historia = self.fita.historia_em(tick);
        self.tique_das_entradas = Some(tick);
        Some(g)
    }

    /// **O último tique cujas entradas se mostraram** — o `ultimo` de [`Self::entradas_do_tique`]
    /// para uma rota que não guarda o último cozido (a placa num grafo sem estado): sem ele, um
    /// tique antes do primeiro gravado GRAVAVA, era um ramo e esquecia a fita inteira (medido no
    /// gate do cartão, `motion_bridge_historia_tests`), onde a CPU repete.
    #[must_use]
    pub fn tique_das_entradas(&self) -> Option<u64> {
        self.tique_das_entradas
    }

    /// ⭐ **O que a fita disse do último tique cozido** (doc 132 §2) — `Dobrada` quando o replay
    /// mostrou a base no lugar da história, `NaoGravada` quando mostrou o publicado agora; o cartão
    /// das entradas lê-o.
    #[must_use]
    pub fn historia_mostrada(&self) -> Historia {
        self.historia
    }

    /// Põe a fita num razão próprio (os gates e a cena de smoke dobram em segundos); esquece-a.
    pub fn define_o_razao_da_fita(&mut self, razao: std::sync::Arc<crate::Razao>) {
        self.fita = Fita::no_razao(razao);
        self.historia = Historia::Verdade;
    }

    /// O relógio está a TOCAR? (o shell diz-o a cada quadro; parado, arrastar a fita repete).
    pub fn define_a_tocar(&mut self, a_tocar: bool) {
        self.a_tocar = a_tocar;
    }
}
