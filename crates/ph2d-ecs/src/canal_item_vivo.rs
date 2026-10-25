//! ⭐ **O item que uma chave nomeia ainda existe?** — a porta da timeline para a camada apagada
//! (decisão do dono 25/10): o binding de um item que saiu é «missing» como o de um objecto morto.
//! Responde pelos espelhos que as famílias publicam ([`IdsDoVetor`], [`PosicoesDosItens`]); a
//! timeline não conhece família nenhuma. Sem espelho da entidade não decide: vivo.

use crate::canal::familia::{LISTA, VETOR};
use crate::canal_lista::PosicoesDosItens;
use crate::canal_vetor::{Lista, lista};
use crate::canal_vetor_ids::IdsDoVetor;
use crate::{FieldKey, World};

/// `false` só quando a família publicou a entidade e o item de `k` não está lá.
#[must_use]
pub fn item_vivo(world: &World, entidade: u64, k: FieldKey) -> bool {
    match k.familia {
        VETOR => {
            let l = lista(k.campo);
            if k.item.is_none() || l == Lista::Nenhuma {
                return true;
            }
            world
                .get_resource::<IdsDoVetor>()
                .and_then(|m| m.0.get(&entidade))
                .is_none_or(|f| f.posicao(l, k.item).is_some())
        }
        LISTA => {
            let item = crate::canal_lista::item(k);
            world.get_resource::<PosicoesDosItens>().is_none_or(|p| {
                item.is_none() || !p.1.contains(&entidade) || p.0.contains_key(&(entidade, item.0))
            })
        }
        _ => true,
    }
}
