//! **O valor de um componente por NOME de campo** — o irmão do [`super::registry`] pelo tecto de
//! LOC: a vtable ganha `value_of`/`set_value`, que passam o componente inteiro por
//! `serde_json::Value`, e quem lê ou troca um campo fá-lo pela chave serde do
//! [`ph2d_component_desc::FieldDesc`] (doc CodeEditor 08 §3.3).
//!
//! ⚠️ O caminho gravado continua o postcard (posicional); o `Value` é só a porta por nome, e não
//! viaja para o ficheiro.

use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Errors raised by the registry on insert / serialize.
#[derive(Debug)]
pub enum RegistryError {
    Decode(postcard::Error),
    Encode(postcard::Error),
    UnknownTypeId(ComponentTypeId),
    EntityMissing(Entity),
    /// O valor por nome não cabe no tipo (campo de tipo errado, número fora do inteiro…).
    Value(String),
}

use super::registry::ComponentTypeId;

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "ComponentRegistry decode error: {e}"),
            Self::Encode(e) => write!(f, "ComponentRegistry encode error: {e}"),
            Self::UnknownTypeId(id) => {
                write!(f, "ComponentRegistry: unknown type_id 0x{id:016x}")
            }
            Self::EntityMissing(e) => {
                write!(f, "ComponentRegistry: entity {:?} not in world", e)
            }
            Self::Value(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for RegistryError {}

/// O componente (se presente) como `serde_json::Value`. `Ok(None)` = a entidade não o tem.
pub type ValueOfFn = fn(&World, Entity) -> Result<Option<serde_json::Value>, RegistryError>;

/// Reconstrói o componente a partir de um `Value` e substitui-o na entidade.
pub type SetValueFn = fn(&mut World, Entity, serde_json::Value) -> Result<(), RegistryError>;

pub(super) fn value_of<T: Component + Serialize>(
    world: &World,
    entity: Entity,
) -> Result<Option<serde_json::Value>, RegistryError> {
    let e = world
        .get_entity(entity)
        .map_err(|_| RegistryError::EntityMissing(entity))?;
    e.get::<T>()
        .map(|v| serde_json::to_value(v).map_err(|e| RegistryError::Value(e.to_string())))
        .transpose()
}

pub(super) fn set_value<T: Component<Mutability = Mutable> + DeserializeOwned>(
    world: &mut World,
    entity: Entity,
    value: serde_json::Value,
) -> Result<(), RegistryError> {
    let v: T = serde_json::from_value(value).map_err(|e| RegistryError::Value(e.to_string()))?;
    let mut e = world
        .get_entity_mut(entity)
        .map_err(|_| RegistryError::EntityMissing(entity))?;
    e.insert(v);
    Ok(())
}
