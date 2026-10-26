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
    decidido(world, entidade, k).unwrap_or(true)
}

/// A chave `k` endereça um item de lista (tinta, efeito, filtro, item de componente)?
#[must_use]
pub fn nomeia_item(k: FieldKey) -> bool {
    match k.familia {
        VETOR => !k.item.is_none() && lista(k.campo) != Lista::Nenhuma,
        LISTA => !crate::canal_lista::item(k).is_none(),
        _ => false,
    }
}

/// O item de `k` está lá? `None` = não há que decidir: a chave não nomeia item, ou a família ainda
/// não publicou a entidade (o load: espera pelo espelho, F32 §1).
#[must_use]
pub fn decidido(world: &World, entidade: u64, k: FieldKey) -> Option<bool> {
    match k.familia {
        VETOR => {
            let l = lista(k.campo);
            if k.item.is_none() || l == Lista::Nenhuma {
                return None;
            }
            let f = world.get_resource::<IdsDoVetor>()?.0.get(&entidade)?;
            Some(f.posicao(l, k.item).is_some())
        }
        LISTA => {
            let item = crate::canal_lista::item(k);
            let p = world.get_resource::<PosicoesDosItens>()?;
            (!item.is_none() && p.1.contains(&entidade))
                .then(|| p.0.contains_key(&(entidade, item.0)))
        }
        _ => None,
    }
}
