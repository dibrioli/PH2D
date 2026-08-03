//! **OS COMPONENTES** — o mestre, a instância, e o override esparso (plano UI/UX W5).
//!
//! Um MESTRE é um caminho como qualquer outro, marcado. Uma INSTÂNCIA é um caminho cuja
//! geometria é **derivada** do mestre por frame — não uma cópia no documento. É isso que faz
//! *"editar o mestre propaga"* ser verdade **por construção**, em vez de um passe de propagação
//! que alguém esquece de chamar.
//!
//! # O endereço é UM, e o plano tinha DOIS
//!
//! O plano previa `VecComponentMain { key: String }` **e** `VecInstance { main: VecPathId }` — o
//! mestre carregando um nome e a instância apontando por id. Duas respostas a *"qual mestre é
//! este?"* divergem no dia em que uma ganha um caso (renomear, duplicar, importar), então uma
//! morreu: **o id vence**, que é o precedente local literal — o [`crate::VecPatternPath::path`] e
//! o [`crate::VecTextPath::path`] apontam o caminho-guia por `u64` pela mesma razão. O nome de
//! exibição do mestre é o `Name` que a Hierarquia já mostra; um segundo nome dentro do componente
//! seria a coisa que fica velha quando o artista renomeia na árvore.
//!
//! ⚠️ **`u64` e não `VecPathId`** porque o ECS não depende do vetor — a razão está escrita nos
//! dois irmãos acima, e é a mesma que faz o [`crate::VecShape`] guardar primitivos.
//!
//! # O que a instância DESENHA
//!
//! O mestre é o caminho-raiz **mais toda a sub-árvore ECS dele**: uma moldura (W0) com filhos é,
//! sem nenhum código a mais, um componente com peças. A instância publica esse conjunto inteiro
//! na `LiveGeometry` dela, assado na pose DELA — o 9º produtor.
//!
//! # Os overrides são ESPARSOS, e a ordem deles é LEI
//!
//! Só o que difere viaja. E a lista é mantida **ordenada por `(sub, slot)`**, o que não é
//! arrumação: o undo deste editor compara os **BYTES** do componente (`canonicalize`), então duas
//! instâncias logicamente iguais com a lista em ordens diferentes comparariam DIFERENTE — e cada
//! frame viraria um passo de undo espúrio. É o mesmo mecanismo que fez a física endereçar corpos
//! por nome em vez de por bits de entidade.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Este caminho é um MESTRE de componente.**
///
/// Marcador: a PRESENÇA é o booleano — o idioma do [`crate::VecCutPath`] e dos marcadores da
/// física. Um `bool` teria dois estados para a mesma coisa (ausente e `false`), e alguém acabaria
/// a testar o errado.
///
/// ⚠️ Sem o registro no `ComponentRegistry` este marcador é **DESCARTADO pelo snapshot**: um
/// Ctrl+Z devolveria a arte com as instâncias apontando para um mestre que já não se declara
/// mestre — e elas ficariam órfãs em silêncio.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VecComponentMain;

impl SimComponent for VecComponentMain {}

/// **Em que uma instância pode DIFERIR do mestre.**
///
/// Vocabulário FECHADO e pequeno de propósito. O plano falava de um `PropPath` genérico
/// partilhado com a W4 (a tabela de tokens) — mas a W4b não foi construída, então esse tipo **não
/// existe**, e inventar uma enumeração geral de *"que propriedade de que objeto"* sem o consumidor
/// que a exige é desenhar no escuro. Quando a W4b chegar, ela estende **esta** lista ou a absorve;
/// o que não pode acontecer é nascerem duas.
///
/// Os dois de hoje são os dois que um catálogo de verdade usa: a peça muda de COR, ou a peça não
/// aparece nesta instância (o *boolean property* do Figma).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OverrideSlot {
    /// A cor de preenchimento desta peça, RGBA não-premultiplicado.
    ///
    /// ⚠️ **Sólida, e é uma decisão e não uma limitação de tipo:** o `Paint` do documento é um
    /// enum rico (gradientes com pontos em MUNDO), e o ECS não pode nomeá-lo. Mas um gradiente
    /// autorado em coordenadas de mundo **não sobrevive** a ser reposicionado noutra instância —
    /// ele descreveria a rampa onde o mestre está, não onde a cópia está. Uma cor sólida é a
    /// grandeza que viaja.
    Fill([u8; 4]),
    /// Esta peça do mestre **não aparece** nesta instância.
    Hidden,
}

impl OverrideSlot {
    /// A chave de ordenação **por espécie** — o que torna a lista canônica sem depender do valor.
    ///
    /// ⚠️ Ordenar pelo valor faria a posição de uma entrada mudar quando a cor muda, e duas
    /// instâncias iguais poderiam guardar a mesma coisa em ordens diferentes: exatamente o que a
    /// canonicidade existe para impedir.
    #[must_use]
    pub const fn kind(self) -> u8 {
        match self {
            Self::Fill(_) => 0,
            Self::Hidden => 1,
        }
    }
}

/// Um override: **que peça do mestre**, e **o que nela difere**.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceOverride {
    /// O `VecPathId` da peça **NO MESTRE** (como `u64`; ver o § do cabeçalho).
    ///
    /// ⚠️ Endereçar no mestre, e não na instância, é o que faz o override **sobreviver** a editar
    /// o mestre: a peça continua a mesma peça mesmo que a geometria dela mude por inteiro.
    pub sub: u64,
    /// O que difere.
    pub slot: OverrideSlot,
}

/// **Este caminho é uma INSTÂNCIA** do mestre `main`.
#[derive(Component, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VecInstance {
    /// O `VecPathId` do mestre (como `u64`).
    pub main: u64,
    /// O que difere do mestre — **esparso** e mantido em ordem canônica por [`Self::set`].
    pub overrides: Vec<InstanceOverride>,
}

impl VecInstance {
    /// Uma instância limpa do mestre `main`.
    #[must_use]
    pub fn new(main: u64) -> Self {
        Self {
            main,
            overrides: Vec::new(),
        }
    }

    /// **A porta ÚNICA de escrita de override** — grava (ou substitui) mantendo a ordem canônica.
    ///
    /// Substitui pela espécie: pedir `Fill` duas vezes na mesma peça deixa **uma** entrada, senão
    /// a lista cresceria a cada nudge de cor e o consumidor teria de decidir qual vale (a última?
    /// a primeira?) — pergunta que ninguém deve ter de responder.
    pub fn set(&mut self, sub: u64, slot: OverrideSlot) {
        let key = (sub, slot.kind());
        match self
            .overrides
            .binary_search_by_key(&key, |o| (o.sub, o.slot.kind()))
        {
            Ok(i) => self.overrides[i].slot = slot,
            Err(i) => self.overrides.insert(i, InstanceOverride { sub, slot }),
        }
    }

    /// O override desta peça nesta espécie, se houver.
    #[must_use]
    pub fn get(&self, sub: u64, kind: u8) -> Option<OverrideSlot> {
        self.overrides
            .binary_search_by_key(&(sub, kind), |o| (o.sub, o.slot.kind()))
            .ok()
            .map(|i| self.overrides[i].slot)
    }

    /// **Reset**: a instância volta a ser exatamente o mestre.
    pub fn reset(&mut self) {
        self.overrides.clear();
    }

    /// A lista está na ordem canônica? (O invariante que o `canonicalize` do undo exige.)
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        self.overrides
            .windows(2)
            .all(|w| (w[0].sub, w[0].slot.kind()) < (w[1].sub, w[1].slot.kind()))
    }
}

impl SimComponent for VecInstance {}

#[cfg(test)]
#[path = "vec_component_tests.rs"]
mod tests;
