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
use std::collections::BTreeMap;

use ph2d_morph_machine::MorphKey;
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
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecMorphMachine {
    /// ⭐⭐⭐ **A tecla e o ritmo de cada forma, indexados por `VecPathId`.**
    ///
    /// # ⛔⛔ A LISTA de formas NÃO mora aqui — ela são os FILHOS (W11)
    ///
    /// Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se
    /// for arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente
    /// passa a fazer parte do sistema."*
    ///
    /// Até 2026-08-25 este componente guardava a lista (`graph.states`) **e** o
    /// `morph_set::upkeep` escrevia `ChildOf` — duas respostas para *«que formas estão neste
    /// conjunto»*, que já podiam discordar (apagar um filho deixava a lista a nomear uma forma
    /// inexistente). O arrastar-para-dentro torna a discordância um **gesto**, e a cura não é
    /// reconciliar: é **não a poder exprimir**.
    ///
    /// ⇒ o grafo é **DERIVADO** dos `Children` a cada quadro (`morph_set::graph_of`), como o
    /// `FieldDoc` do módulo 3D Modeling é cozido da hierarquia. Arrastar para dentro **é** entrar;
    /// arrastar para fora **é** sair. Nenhum código reage ao gesto porque não há gesto a que
    /// reagir.
    ///
    /// ⚠️ **Uma forma sem entrada usa os valores de partida** ([`MorphKey::default`]) — é assim
    /// que um filho recém-arrastado já participa sem que ninguém escreva nada por ele.
    ///
    /// ⚠️ **`BTreeMap`, nunca `HashMap`** — a espinha do determinismo deste repo, e aqui ela é
    /// load-bearing duas vezes: a ordem entra no `deterministic_hash` e no diff do undo.
    ///
    /// ⚠️ **As chaves de formas que saíram FICAM.** Arrastar uma forma para fora e voltar a
    /// arrastá-la para dentro devolve-lhe a tecla que ela tinha — perder o trabalho do artista
    /// por um gesto reversível seria a pior leitura possível de *"desconectar"*.
    pub keys: BTreeMap<u64, MorphKey>,
}

impl SimComponent for VecMorphMachine {}

impl VecMorphMachine {
    /// Uma máquina nova **sem tecla nenhuma atribuída**.
    ///
    /// ⚠️ **Ela não recebe as formas, e a ausência é o desenho:** as formas são os FILHOS, e quem
    /// os pendura é o [`crate::ChildOf`]. Passá-las aqui recriaria a segunda lista que a W11
    /// apagou.
    ///
    /// ⚠️ **Sem teclas é o estado honesto de nascimento**: o artista acabou de dizer *"estas são
    /// as formas"* e ainda não disse o que leva a cada uma. A máquina mostra a primeira e fica
    /// parada — que é exactamente o que ele pediu até agora.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A chave desta forma, ou os valores de partida — **a única porta**.
    ///
    /// ⚠️ Um `keys.get(&id).cloned().unwrap_or_default()` escrito à mão noutro sítio seria a
    /// segunda lei de *«o que é uma forma sem chave»*, e as duas divergiriam no dia em que o
    /// default mudasse.
    #[must_use]
    pub fn key_of(&self, shape: u64) -> MorphKey {
        self.keys.get(&shape).cloned().unwrap_or_default()
    }
}
