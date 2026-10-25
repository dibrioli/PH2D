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

use crate::scene::registry::stable_type_id;
use crate::{ItemId, StableIdCounter};

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

/// O passe de UM componente com itens.
#[derive(Clone, Copy)]
pub struct Passe {
    /// O nome canónico do componente no registo.
    pub canonical: &'static str,
    ler: fn(&mut World) -> Vec<(Entity, Vec<ItemId>)>,
    escrever: fn(&mut World, Entity, &[ItemId]),
    /// Os bytes v194 (sem `item`) na forma de agora.
    legado: fn(&[u8]) -> Legado,
}

impl Passe {
    /// O passe de `T`, com `L` = a forma gravada até ao `PROJECT_SCHEMA` 194.
    #[must_use]
    pub fn de<T: ComItens, L: DeserializeOwned + Into<T>>(canonical: &'static str) -> Self {
        Self {
            canonical,
            ler: ler::<T>,
            escrever: escrever::<T>,
            legado: legado::<T, L>,
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

/// O que um blob é para o degrau `194 → 195`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Legado {
    /// Lido na forma v194 (até ao último byte), e aqui na de agora.
    Migrado(Vec<u8>),
    /// Já na forma de agora: uma migração mais antiga (a do `v128`) reescreveu-o com o tipo vivo.
    JaNovo,
    Ilegivel,
}

/// `Some` só se `b` se lê como `X` até ao último byte (o postcard ignora a cauda: sem isto um blob
/// de uma forma passaria por outra).
fn inteiro<X: DeserializeOwned>(b: &[u8]) -> Option<X> {
    match postcard::take_from_bytes::<X>(b) {
        Ok((x, resto)) if resto.is_empty() => Some(x),
        _ => None,
    }
}

fn legado<T: ComItens, L: DeserializeOwned + Into<T>>(b: &[u8]) -> Legado {
    if let Some(l) = inteiro::<L>(b) {
        let t: T = l.into();
        return postcard::to_allocvec(&t).map_or(Legado::Ilegivel, Legado::Migrado);
    }
    if inteiro::<T>(b).is_some() {
        Legado::JaNovo
    } else {
        Legado::Ilegivel
    }
}

/// Os componentes do `ph2d-ecs` com itens.
#[must_use]
pub fn desta_crate() -> [Passe; DESTA_CRATE] {
    use crate::item_id_v194 as v;
    [
        Passe::de::<crate::Timers, v::TimersV194>("ph2d::ecs::Timers"),
        Passe::de::<crate::Tweens, v::TweensV194>("ph2d::ecs::Tweens"),
        Passe::de::<crate::StateMachine, v::StateMachineV194>("ph2d::ecs::StateMachine"),
        Passe::de::<crate::CounterWatch, v::CounterWatchV194>("ph2d::ecs::CounterWatch"),
        Passe::de::<crate::SignalActions, v::SignalActionsV194>("ph2d::ecs::SignalActions"),
        Passe::de::<crate::SignalOnAction, v::SignalOnActionV194>("ph2d::ecs::SignalOnAction"),
        Passe::de::<crate::ShakeEmitter, v::ShakeEmitterV194>("ph2d::ecs::ShakeEmitter"),
        Passe::de::<crate::NamedAnchorList, v::NamedAnchorListV194>("ph2d::ecs::NamedAnchorList"),
    ]
}

/// Quantos componentes do `ph2d-ecs` têm itens.
pub const DESTA_CRATE: usize = 8;

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
                *id = contador.take_item();
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
    mudou
}

/// O que a migração dos itens fez.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MigracaoV194 {
    pub migrados: usize,
    pub ilegiveis: usize,
}

/// ⭐ **O degrau `194 → 195`**: os blobs dos componentes com itens passam à forma com `item`
/// (`0`); o passe dá-lhes ids na ordem da lista no primeiro quadro (ou no `stable_id_for_name`).
pub fn migra_v194(world: &mut crate::scene::WorldSnapshot) -> MigracaoV194 {
    let passes: Vec<(u64, Passe)> = (passes().into_iter())
        .map(|p| (stable_type_id(p.canonical), p))
        .collect();
    let mut m = MigracaoV194::default();
    for row in &mut world.entities {
        for slot in 0..row.components.len() {
            let id = row.components[slot].type_id;
            let Some((_, p)) = passes.iter().find(|(t, _)| *t == id) else {
                continue;
            };
            match (p.legado)(&row.components[slot].data) {
                Legado::Migrado(novo) => {
                    std::sync::Arc::make_mut(row).components[slot].data = novo;
                    m.migrados += 1;
                }
                Legado::JaNovo => {}
                Legado::Ilegivel => m.ilegiveis += 1,
            }
        }
    }
    m
}

#[cfg(test)]
#[path = "item_id_tests.rs"]
mod tests;
