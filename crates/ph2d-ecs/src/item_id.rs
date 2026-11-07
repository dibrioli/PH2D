//! ⭐ **O passe que dá identidade aos ITENS das listas** (F30 §A1, `docs/UI_New_and_Simple/F30_plano.md`
//! §A1) — o gémeo do [`crate::assign_missing_stable_ids`], que o chama no fim.
//!
//! Idempotente. Por ordem: (a) o [`StableIdCounter`] reconcilia-se contra todo id de item vivo; (b)
//! um item com `0` recebe um id novo (na ordem da lista: determinístico); (c) um id REPETIDO dentro
//! da MESMA entidade (um item duplicado) é refeito — fica o primeiro na ordem da lista.
//!
//! ⚠️ A unicidade é por ENTIDADE, não por projecto: a chave de animação já leva a entidade (o
//! binding), e a cópia de um objecto (`deep_copy_subtree`) e o sync mestre→instância (que compara
//! bytes a cada quadro) copiam os itens com os ids — refazê-los brigaria com o sync em todo quadro.
//! O contador é do projecto e nunca desce: um id dado nunca volta a nascer noutro item.
//!
//! Cada componente com lista regista um [`Passe`] (os do `ph2d-ecs` estão em [`DESTA_CRATE`]; os
//! de outra crate entram por [`regista`] no registo de componentes dela).

use std::collections::BTreeSet;
use std::sync::Mutex;

use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::prelude::{Entity, World};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{
    CounterWatch, ItemId, NamedAnchorList, ShakeEmitter, SignalActions, SignalOnAction,
    StableIdCounter, StateMachine, Timers, Tweens,
};

/// Um componente com itens identificados, por ordem (as listas dele em sequência).
pub trait ComItens: Component<Mutability = Mutable> + Serialize + DeserializeOwned {
    /// Os ids, na ordem das listas.
    fn ids(&self) -> Vec<ItemId>;
    /// Escreve `ids` na mesma ordem de [`Self::ids`].
    fn com_ids(&mut self, ids: &[ItemId]);
}

/// Os ids de uma lista de itens com um campo `item`.
#[macro_export]
macro_rules! com_itens {
    ($t:ty: $($lista:tt),+) => {
        impl $crate::item_id::ComItens for $t {
            fn ids(&self) -> Vec<$crate::ItemId> {
                let mut v = Vec::new();
                $( v.extend(self.$lista.iter().map(|x| x.item)); )+
                v
            }
            fn com_ids(&mut self, ids: &[$crate::ItemId]) {
                let mut ids = ids.iter();
                $( for (x, id) in self.$lista.iter_mut().zip(ids.by_ref()) { x.item = *id; } )+
            }
        }
    };
}

/// Os itens de cada entidade.
type ItensPorEntidade = Vec<(Entity, Vec<ItemId>)>;

/// O passe de UM componente com itens.
#[derive(Clone, Copy)]
pub struct Passe {
    /// O nome canónico do componente no registo.
    pub canonical: &'static str,
    ler: fn(&mut World) -> ItensPorEntidade,
    escrever: fn(&mut World, Entity, &[ItemId]),
}

impl Passe {
    /// O passe de `T`.
    #[must_use]
    pub fn de<T: ComItens>(canonical: &'static str) -> Self {
        Self {
            canonical,
            ler: ler::<T>,
            escrever: escrever::<T>,
        }
    }
}

fn ler<T: ComItens>(w: &mut World) -> Vec<(Entity, Vec<ItemId>)> {
    let mut q = w.query::<(Entity, &T)>();
    q.iter(w).map(|(e, c)| (e, c.ids())).collect()
}

fn escrever<T: ComItens>(w: &mut World, e: Entity, ids: &[ItemId]) {
    if let Some(mut c) = w.get_mut::<T>(e) {
        c.com_ids(ids);
    }
}

crate::com_itens!(Timers: 0);
crate::com_itens!(Tweens: 0);
crate::com_itens!(StateMachine: states, transitions);
crate::com_itens!(CounterWatch: 0);
crate::com_itens!(SignalActions: 0);
crate::com_itens!(SignalOnAction: 0);
crate::com_itens!(ShakeEmitter: 0);
crate::com_itens!(NamedAnchorList: 0);

/// Os componentes do `ph2d-ecs` com itens.
#[must_use]
pub fn desta_crate() -> [Passe; DESTA_CRATE] {
    [
        Passe::de::<crate::Timers>("ph2d::ecs::Timers"),
        Passe::de::<crate::Tweens>("ph2d::ecs::Tweens"),
        Passe::de::<crate::StateMachine>("ph2d::ecs::StateMachine"),
        Passe::de::<crate::CounterWatch>("ph2d::ecs::CounterWatch"),
        Passe::de::<crate::SignalActions>("ph2d::ecs::SignalActions"),
        Passe::de::<crate::SignalOnAction>("ph2d::ecs::SignalOnAction"),
        Passe::de::<crate::ShakeEmitter>("ph2d::ecs::ShakeEmitter"),
        Passe::de::<crate::NamedAnchorList>("ph2d::ecs::NamedAnchorList"),
        Passe::de::<crate::VecFilter>("ph2d::ecs::VecFilter"),
    ]
}

/// Quantos componentes do `ph2d-ecs` têm itens.
pub const DESTA_CRATE: usize = 9;

static DE_FORA: Mutex<Vec<Passe>> = Mutex::new(Vec::new());

/// Regista o passe de um componente de outra crate (idempotente, pelo nome canónico).
pub fn regista(p: Passe) {
    let mut v = DE_FORA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !v.iter().any(|x| x.canonical == p.canonical) {
        v.push(p);
    }
}

/// Todos os passes conhecidos: os desta crate e os registados.
#[must_use]
pub fn passes() -> Vec<Passe> {
    let mut v = desta_crate().to_vec();
    let fora = DE_FORA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for p in fora.iter() {
        if !v.iter().any(|x| x.canonical == p.canonical) {
            v.push(*p);
        }
    }
    v
}

/// ⭐ **Dá um id a todo item sem um, e refaz os repetidos na mesma entidade.** Idempotente: devolve
/// `false` quando nada mudou. Ver o cabeçalho do módulo.
pub fn assign_missing_item_ids(world: &mut World) -> bool {
    let passes = passes();
    let mut listas: Vec<(Entity, usize, Vec<ItemId>)> = Vec::new();
    for (k, p) in passes.iter().enumerate() {
        listas.extend((p.ler)(world).into_iter().map(|(e, ids)| (e, k, ids)));
    }
    let maior = listas
        .iter()
        .flat_map(|(_, _, ids)| ids.iter().map(|i| u64::from(i.0)))
        .max()
        .unwrap_or(0);
    let mut contador = world
        .remove_resource::<StableIdCounter>()
        .unwrap_or_default();
    contador.reconcile_at_least(maior.saturating_add(1));
    // A entidade e, dentro dela, a ordem dos passes: a ordem das listas.
    listas.sort_by_key(|&(e, k, _)| (e.index(), k));
    let mut mudou = false;
    let mut vistos: (Option<Entity>, BTreeSet<u32>) = (None, BTreeSet::new());
    for (e, k, ids) in &mut listas {
        if vistos.0 != Some(*e) {
            vistos = (Some(*e), BTreeSet::new());
        }
        let mut esta = false;
        for id in ids.iter_mut() {
            if id.is_none() || !vistos.1.insert(id.0) {
                *id = contador.aloca_item();
                vistos.1.insert(id.0);
                esta = true;
            }
        }
        if esta {
            (passes[*k].escrever)(world, *e, ids);
            mudou = true;
        }
    }
    world.insert_resource(contador);
    // Os ids que acabaram de nascer fixam o relógio dos tweens no mesmo passe (F32 §4).
    let tweens = crate::tween::adopta_relogios(world);
    crate::path_follow::adopta_relogios(world) || tweens || mudou
}

#[cfg(test)]
#[path = "item_id_tests.rs"]
mod tests;
