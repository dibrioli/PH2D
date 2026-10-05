//! Os elementos de um quadro e o documento que os guarda.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::FracKey;

/// Identidade de um elemento dentro do seu quadro. Nunca reusada (as lápides guardam-na).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElementId(pub u64);

/// Cor do DOCUMENTO — dado do artista, não da UI (a paleta por omissão vem dos tokens).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

/// O que um elemento é. ⚠️ postcard é posicional: variante NOVA só no fim, e campo novo numa
/// variante existente sobe o [`crate::FORMAT_VERSION`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ElementKind {
    /// Rectângulo cheio — a primeira forma (W0: a régua de desenho).
    Rect { fill: Rgba },
}

/// Um elemento. Os quatro últimos campos são a gramática da colaboração (Etapa 2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub id: ElementId,
    pub kind: ElementKind,
    /// Canto superior esquerdo e tamanho, em unidades do mundo do quadro.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// Ordem em z.
    pub z: FracKey,
    /// Sobe a cada mudança deste elemento.
    pub version: u64,
    /// Desempate entre duas mudanças com a mesma `version` (Etapa 2).
    pub nonce: u32,
    /// Lápide: apagado continua no mapa, para não ressuscitar numa fusão.
    pub deleted: bool,
}

/// O conteúdo de um quadro. `BTreeMap` por lei (determinismo).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BoardDoc {
    pub(crate) elements: BTreeMap<ElementId, Element>,
    pub(crate) next_id: u64,
}

impl BoardDoc {
    /// Um id ainda não usado neste quadro.
    pub fn mint_id(&mut self) -> ElementId {
        self.next_id += 1;
        ElementId(self.next_id)
    }

    /// O elemento vivo (não apagado) com este id.
    #[must_use]
    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.elements.get(&id).filter(|e| !e.deleted)
    }

    /// Os elementos vivos, de trás para a frente (ordem de desenho).
    #[must_use]
    pub fn live_in_z_order(&self) -> Vec<&Element> {
        let mut v: Vec<&Element> = self.elements.values().filter(|e| !e.deleted).collect();
        v.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)));
        v
    }

    /// Quantos elementos vivos.
    #[must_use]
    pub fn live_len(&self) -> usize {
        self.elements.values().filter(|e| !e.deleted).count()
    }

    /// A chave de z para pôr um elemento novo à frente de todos.
    #[must_use]
    pub fn z_on_top(&self) -> FracKey {
        let top = self.elements.values().map(|e| &e.z).max();
        FracKey::between(top, None)
    }
}
