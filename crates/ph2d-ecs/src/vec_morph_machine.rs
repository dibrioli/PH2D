//! **O GRAFO AUTORADO da máquina de estados do Morph** — o irmão de documento do
//! [`crate::VecMorph`] (plano [`docs/Vector Module/32`]).
//!
//! # ⭐ Dois componentes na MESMA entidade, e a divisão é entre autorado e vivo
//!
//! * [`crate::VecMorph`] — **o que a cena mostra agora**: o par de fontes e o `t`. Ele já existia,
//!   e **não muda uma linha**.
//! * [`VecMorphMachine`] — **o que o artista desenhou**: as setas e as condições.
//!
//! ⚠️ **O ESTADO VIVO da máquina não está aqui, e não pode estar.** *Onde a forma está agora* é
//! `ph2d_morph_machine::MorphMachine`, que vive numa tabela runtime-only da shell — a mesma lei,
//! palavra por palavra, das `UiMachines` da ponte de estados de UI: *gravá-la faria um projecto
//! reabrir a meio de uma transição*.
//!
//! # ⚠️ Sem este componente, o `VecMorph` é **byte-idêntico** ao que sempre foi
//!
//! Quem conduz o `t` de um morph sem máquina continua a ser a timeline (`PropKind::Morph`). O
//! componente é a porta que diz *"agora quem manda é o grafo"* — e a ausência dele é a resposta
//! honesta de que ninguém desenhou seta nenhuma.

use bevy_ecs::prelude::Component;
use ph2d_morph_machine::MorphGraph;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// As setas que o artista desenhou no canvas, para o morph desta entidade.
///
/// ⚠️ **Ele é `SimComponent`** pela mesma razão do [`crate::VecMorph`]: o replay tem de reproduzir
/// o que o grafo fez, e um componente fora do passo fixo divergiria entre a gravação e a
/// reprodução.
/// ⛔ **Sem `Default`, e a ausência é a decisão.** Ela põe o componente na família `intrinsic` do
/// catálogo (`g(...)`, o mesmo do `VecMorph`): ele chega com o **gesto** de desenhar a primeira
/// seta, e **não** pela paleta do Inspector. Uma máquina anexada a frio nasceria com `start = 0` —
/// uma forma que não existe —, que é estado inalcançável a fingir-se de vazio.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecMorphMachine {
    pub graph: MorphGraph,
}

impl SimComponent for VecMorphMachine {}

impl VecMorphMachine {
    /// Uma máquina nova sobre as formas `shapes`, **sem tecla nenhuma atribuída**.
    ///
    /// A **primeira** é onde ela nasce (o `start` é derivado da lista — ver [`MorphGraph::start`]).
    ///
    /// ⚠️ **Sem teclas é o estado honesto de nascimento**, e não um vazio a corrigir: o artista
    /// acabou de dizer *"estas são as formas"* e ainda não disse o que leva a cada uma. A máquina
    /// mostra a primeira e fica parada — que é exactamente o que ele pediu até agora.
    #[must_use]
    pub fn new(shapes: &[u64]) -> Self {
        Self {
            graph: MorphGraph {
                states: shapes
                    .iter()
                    .map(|&s| ph2d_morph_machine::MorphState::new(s))
                    .collect(),
            },
        }
    }
}
