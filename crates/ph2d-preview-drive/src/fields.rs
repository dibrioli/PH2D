//! ⭐ **Os CAMPOS que um script escreve a correr** (doc CodeEditor 08 §3.6) — o irmão do memo por
//! `Driver`, mas com a chave `(entidade, componente, campo)` e o valor como `serde_json::Value`: um
//! script alcança qualquer campo nomeado (`health.max`), e uma variante de [`crate::Driven`] por
//! campo seria um enum de centenas.
//!
//! A lei é a do [`crate::Driver::ScriptPose`]: **a corrida não é documento** — a captura vê o
//! autorado, o rebobinar devolve-o — e **outra mão** (o Inspector na pausa) passa a ser o autorado.
//! A granularidade é o CAMPO, como a do memo: repor o componente inteiro engoliria o que outro motor
//! escreve nos campos vizinhos (o `frame` da §11 ao lado do `tint`).

use std::collections::BTreeMap;

use ph2d_ecs::scene::registry::{
    ComponentTypeEntry, InsertDefaultFn, RemoveFn, SetValueFn, ValueOfFn,
};

/// Um campo de um componente, com as portas da vtable que o lêem e escrevem.
#[derive(Clone, Copy, Debug)]
pub struct FieldRef {
    pub canonical_name: &'static str,
    pub key: &'static str,
    pub value_of: ValueOfFn,
    pub set_value: SetValueFn,
    /// Para tirar o componente que o SCRIPT criou (uma linha no neutro) e o voltar a pôr.
    pub remove: RemoveFn,
    pub insert_default: Option<InsertDefaultFn>,
}

impl FieldRef {
    #[must_use]
    pub fn of(entry: &ComponentTypeEntry, key: &'static str) -> Self {
        Self {
            canonical_name: entry.canonical_name,
            key,
            value_of: entry.value_of,
            set_value: entry.set_value,
            remove: entry.remove,
            insert_default: entry.insert_default,
        }
    }
}
use ph2d_ecs::{Entity, SimWorld};
use serde_json::Value;

#[derive(Clone)]
struct FieldEntry {
    authored: Value,
    last_written: Value,
    value_of: ValueOfFn,
    set_value: SetValueFn,
    /// O script criou o componente: o autorado é a AUSÊNCIA.
    born: Option<(RemoveFn, Option<InsertDefaultFn>)>,
}

/// `(bits da entidade, nome canónico, chave serde)`.
type Key = (u64, &'static str, &'static str);

/// O ledger dos campos escritos por script.
#[derive(Default)]
pub struct FieldDrive {
    memo: BTreeMap<Key, FieldEntry>,
}

/// O que a [`FieldDrive::substitute`] deslocou — o vivo, para o [`FieldDrive::restore`].
pub struct DisplacedFields(Vec<(Key, Value, FieldEntry)>);

fn read(value_of: ValueOfFn, sim: &SimWorld, entity: Entity, key: &str) -> Option<Value> {
    // ⚠️ Pela lei do nome (`slot_in`): um newtype (`"."`) e um elemento (`"axis[0]"`) também.
    ph2d_ecs::scene::property::slot_in(&mut value_of(sim.world(), entity).ok()??, key).cloned()
}

fn write(
    value_of: ValueOfFn,
    set_value: SetValueFn,
    sim: &mut SimWorld,
    entity: Entity,
    key: &str,
    v: &Value,
) {
    let Some(mut whole) = value_of(sim.world(), entity).ok().flatten() else {
        return;
    };
    match ph2d_ecs::scene::property::slot_in(&mut whole, key) {
        Some(slot) if *slot != *v => *slot = v.clone(),
        _ => return,
    }
    let _ = set_value(sim.world_mut(), entity, whole);
}

impl FieldDrive {
    /// Um script escreveu `key` do componente de `entry`: `before` é o que lá estava, `after` o que
    /// deixou; `born` = a escrita criou o componente.
    pub fn driven(&mut self, entity: Entity, field: FieldRef, before: Value, after: Value, born: bool) {
        let k = (entity.to_bits(), field.canonical_name, field.key);
        let entry = field;
        match self.memo.get_mut(&k) {
            None => {
                self.memo.insert(
                    k,
                    FieldEntry {
                        authored: before,
                        last_written: after,
                        value_of: entry.value_of,
                        set_value: entry.set_value,
                        born: born.then_some((entry.remove, entry.insert_default)),
                    },
                );
            }
            Some(e) => {
                if e.last_written != before {
                    e.authored = before;
                }
                e.last_written = after;
            }
        }
    }

    /// Uma vez por quadro: um campo que mudou sem o script foi **outra mão**, e passa a ser o
    /// autorado. Quem perdeu a entidade ou o componente sai do ledger.
    pub fn observe(&mut self, sim: &SimWorld) {
        self.memo.retain(|&(bits, _, key), e| {
            let Some(now) = read(e.value_of, sim, Entity::from_bits(bits), key) else {
                return false;
            };
            if now != e.last_written {
                e.authored = now.clone();
                e.last_written = now;
            }
            true
        });
    }

    /// Rebobinar: todo campo volta ao autorado e o ledger esvazia. Devolve quantos.
    pub fn release(&mut self, sim: &mut SimWorld) -> usize {
        let n = self.memo.len();
        for ((bits, _, key), e) in std::mem::take(&mut self.memo) {
            if let Some((remove, _)) = e.born {
                remove(sim.world_mut(), Entity::from_bits(bits));
                continue;
            }
            write(
                e.value_of,
                e.set_value,
                sim,
                Entity::from_bits(bits),
                key,
                &e.authored,
            );
        }
        n
    }

    /// O mundo passa ao autorado para a captura; devolve o vivo.
    #[must_use]
    pub fn substitute(&self, sim: &mut SimWorld) -> DisplacedFields {
        let mut live = Vec::with_capacity(self.memo.len());
        for (&k, e) in &self.memo {
            let entity = Entity::from_bits(k.0);
            let Some(now) = read(e.value_of, sim, entity, k.2) else {
                continue;
            };
            match e.born {
                Some((remove, _)) => remove(sim.world_mut(), entity),
                None => write(e.value_of, e.set_value, sim, entity, k.2, &e.authored),
            }
            live.push((k, now, e.clone()));
        }
        DisplacedFields(live)
    }

    /// Devolve ao mundo o que a [`Self::substitute`] deslocou.
    pub fn restore(sim: &mut SimWorld, live: &DisplacedFields) {
        for ((bits, _, key), v, e) in &live.0 {
            let entity = Entity::from_bits(*bits);
            if let Some((_, Some(insert))) = e.born
                && matches!((e.value_of)(sim.world(), entity), Ok(None))
            {
                let _ = insert(sim.world_mut(), entity);
            }
            write(e.value_of, e.set_value, sim, entity, key, v);
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.memo.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memo.is_empty()
    }
}
