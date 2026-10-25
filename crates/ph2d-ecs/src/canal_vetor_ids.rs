//! ⭐ **O espelho dos ids das listas do Vetor no mundo** (F31 §A.4) — o gémeo de
//! [`crate::canal_lista::PosicoesDosItens`] para as entradas da pilha e os filtros da forma.
//!
//! A `VecScene` não está no `World`; quem precisa de traduzir posição ↔ id fora do `ph2d-app-vec`
//! (a migração das chaves v23 e o nome da linha, em `ph2d-timeline-persist`) lê-a aqui. Publicado
//! pelo `ph2d-app-vec` a cada quadro; uma entidade com algum id por atribuir NÃO se publica (a
//! migração espera por ela em vez de chavear uma entrada com `NONE`).

use std::collections::BTreeMap;

use crate::ItemId;
use crate::canal_vetor::Lista;

/// Os ids de cada lista de uma forma, na ordem da lista.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IdsDaForma {
    /// `p.paints` — a entrada `k` da pilha de pinturas.
    pub tintas: Vec<ItemId>,
    /// `p.effects` — o efeito `e`.
    pub efeitos: Vec<ItemId>,
    /// `VecFilter::ids()` — o id do filtro `op`.
    pub filtros: Vec<ItemId>,
}

impl IdsDaForma {
    /// A lista de `lista` (`None` = um campo sem lista).
    #[must_use]
    pub fn de(&self, lista: Lista) -> Option<&[ItemId]> {
        match lista {
            Lista::Nenhuma => None,
            Lista::Tinta => Some(&self.tintas),
            Lista::Efeito => Some(&self.efeitos),
            Lista::Filtro => Some(&self.filtros),
        }
    }

    /// A posição do id `item` na lista `lista`.
    #[must_use]
    pub fn posicao(&self, lista: Lista, item: ItemId) -> Option<usize> {
        self.de(lista)?.iter().position(|&x| x == item)
    }
}

/// Por entidade (os bits), os ids da forma dela — ver o cabeçalho do módulo.
#[derive(bevy_ecs::resource::Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct IdsDoVetor(pub BTreeMap<u64, IdsDaForma>);
