//! **O componente DERIVADO** — irmão do [`super::registry`] por CAP de LOC, cortado pela responsabilidade:
//! o `serialize` de cada tipo e o que NÃO se grava numa entidade, e porquê ([P14 §4 A](../../../../docs/PixelArt/P14_plano.md)).

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::Serialize;

use super::registry::{ComponentRegistry, RegistryError};

/// O `serialize` de sempre: postcard do `T`, `None` sem ele.
pub(super) fn serialize_of<T: Component + Serialize>(
    world: &World,
    entity: Entity,
) -> Result<Option<Vec<u8>>, RegistryError> {
    let e = world
        .get_entity(entity)
        .map_err(|_| RegistryError::EntityMissing(entity))?;
    match e.get::<T>() {
        Some(v) => Ok(Some(
            postcard::to_allocvec(v).map_err(RegistryError::Encode)?,
        )),
        None => Ok(None),
    }
}

/// O `serialize` de um `C` derivado na entidade com `M` ([`ComponentRegistry::derivado_quando`]).
pub(super) fn serialize_derivado<C: Component + Serialize, M: Component>(
    world: &World,
    entity: Entity,
) -> Result<Option<Vec<u8>>, RegistryError> {
    let e = world
        .get_entity(entity)
        .map_err(|_| RegistryError::EntityMissing(entity))?;
    if e.contains::<M>() {
        return Ok(None);
    }
    serialize_of::<C>(world, entity)
}

/// Um componente com campos DERIVADOS ao lado de campos do artista (o `Collider` de uma forma que
/// segue uma slice: a forma e o offset saem do documento; densidade, atrito, camada são do artista).
pub trait CamposDerivados: Component + Serialize {
    /// O valor com os campos derivados no ponto neutro: é o que se grava.
    #[must_use]
    fn sem_derivados(&self) -> Self;
}

/// O `serialize` de um `C` com campos derivados na entidade com `M`
/// ([`ComponentRegistry::campos_derivados_quando`]).
pub(super) fn serialize_sem_derivados<C: CamposDerivados, M: Component>(
    world: &World,
    entity: Entity,
) -> Result<Option<Vec<u8>>, RegistryError> {
    let e = world
        .get_entity(entity)
        .map_err(|_| RegistryError::EntityMissing(entity))?;
    if !e.contains::<M>() {
        return serialize_of::<C>(world, entity);
    }
    match e.get::<C>() {
        Some(v) => Ok(Some(
            postcard::to_allocvec(&v.sem_derivados()).map_err(RegistryError::Encode)?,
        )),
        None => Ok(None),
    }
}

impl ComponentRegistry {
    fn grava_de<C: 'static>(&mut self, quem: &str) -> &mut super::registry::SerializeFn {
        let entry = self
            .by_rust
            .get(&std::any::TypeId::of::<C>())
            .and_then(|id| self.by_id.get_mut(id))
            .unwrap_or_else(|| {
                panic!(
                    "ComponentRegistry::{quem}: '{}' is not registered — register it first",
                    std::any::type_name::<C>()
                )
            });
        &mut entry.grava
    }

    /// **Os campos derivados de `C` na entidade que tem `M`**: ali grava-se
    /// [`CamposDerivados::sem_derivados`], logo mudar SÓ um campo derivado não marca o diff do
    /// undo nem muda o ficheiro; o resto de `C` grava-se como sempre. Quem reescreve os derivados é
    /// a família de `M`. Porquê: [P14 §4 C](../../../../docs/PixelArt/P14_plano.md).
    ///
    /// Panics se `C` não estiver registado.
    pub fn campos_derivados_quando<C, M>(&mut self)
    where
        C: CamposDerivados + 'static,
        M: Component,
    {
        *self.grava_de::<C>("campos_derivados_quando") = serialize_sem_derivados::<C, M>;
    }

    /// **`C` é derivado na entidade que tem `M`**: ali o `grava` dele devolve `Ok(None)`, logo `C`
    /// não entra no ficheiro, no diff do undo nem na foto do Play (os três leem o `grava`). O
    /// `serialize` não muda: Inspector, cópias e Simular veem o derivado. Noutra entidade, `C`
    /// grava-se como sempre. Quem reescreve o derivado é a família de `M` (o Stop e o undo fazem
    /// respawn sem ele).
    /// Porquê: [P14 §4 A](../../../../docs/PixelArt/P14_plano.md).
    ///
    /// Panics se `C` não estiver registado — chame depois do `register` dele.
    pub fn derivado_quando<C, M>(&mut self)
    where
        C: Component + Serialize + 'static,
        M: Component,
    {
        *self.grava_de::<C>("derivado_quando") = serialize_derivado::<C, M>;
    }
}
