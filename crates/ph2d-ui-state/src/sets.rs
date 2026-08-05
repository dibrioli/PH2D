//! **ONDE OS ESTADOS MORAM** — a tabela que viaja no documento.

use crate::pose::UiState;
use ph2d_vec_scene::VecPathId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Os estados de cada HOSPEDEIRO, indexados pelo [`VecPathId`] dele.
///
/// ⚠️ **A chave é o `VecPathId`, nunca a entidade.** Bits de entidade são id de ALOCAÇÃO e o undo
/// global respawna tudo com bits novos — uma tabela chaveada por eles perderia os estados no
/// primeiro Ctrl+Z. É a mesma lição que a timeline pagou nas bindings e a física nos joints.
///
/// ⚠️ **`BTreeMap` e não `HashMap`:** esta tabela é serializada, e a ordem de iteração de um
/// `HashMap` faria dois saves do mesmo documento diferirem — e, pior, faria o **diff do undo**
/// registrar um passo espúrio sobre um estado que ninguém tocou.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StateSets {
    by_host: BTreeMap<VecPathId, Vec<UiState>>,
}

impl StateSets {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_host.is_empty()
    }

    /// Os estados de `host`, ou uma fatia vazia.
    #[must_use]
    pub fn get(&self, host: VecPathId) -> &[UiState] {
        self.by_host.get(&host).map_or(&[], Vec::as_slice)
    }

    /// Quem tem estados. Ordem determinista (é um `BTreeMap`).
    pub fn hosts(&self) -> impl Iterator<Item = VecPathId> + '_ {
        self.by_host.keys().copied()
    }

    /// **Grava um estado novo** no fim da lista de `host`, e devolve o índice dele.
    pub fn push(&mut self, host: VecPathId, state: UiState) -> usize {
        let v = self.by_host.entry(host).or_default();
        v.push(state);
        v.len() - 1
    }

    /// Substitui o estado `index` de `host` — o *"Update State"* do artista, que re-grava a pose
    /// atual por cima de uma que ele já tinha nomeado.
    ///
    /// ⚠️ Ele **preserva o NOME**: o artista está a corrigir a pose, não a re-baptizar o estado, e
    /// perder o nome aqui faria a lista dele mudar debaixo do dedo.
    pub fn replace_pose(&mut self, host: VecPathId, index: usize, mut state: UiState) -> bool {
        let Some(slot) = self.by_host.get_mut(&host).and_then(|v| v.get_mut(index)) else {
            return false;
        };
        state.name.clone_from(&slot.name);
        *slot = state;
        true
    }

    pub fn rename(&mut self, host: VecPathId, index: usize, name: impl Into<String>) -> bool {
        match self.by_host.get_mut(&host).and_then(|v| v.get_mut(index)) {
            Some(s) => {
                s.name = name.into();
                true
            }
            None => false,
        }
    }

    /// Apaga o estado `index`. **O hospedeiro sem estado nenhum sai da tabela** — um documento não
    /// carrega uma entrada vazia, e é isso que mantém o `is_empty` honesto e o save enxuto.
    pub fn remove(&mut self, host: VecPathId, index: usize) -> bool {
        let Some(v) = self.by_host.get_mut(&host) else {
            return false;
        };
        if index >= v.len() {
            return false;
        }
        v.remove(index);
        if v.is_empty() {
            self.by_host.remove(&host);
        }
        true
    }

    /// **Esquece um hospedeiro que já não existe.** Chamado quando uma forma é apagada: sem isto a
    /// tabela acumularia estados de objetos que ninguém vê, e eles viajariam no arquivo para
    /// sempre.
    pub fn retain_hosts(&mut self, alive: impl Fn(VecPathId) -> bool) {
        self.by_host.retain(|id, _| alive(*id));
    }
}
