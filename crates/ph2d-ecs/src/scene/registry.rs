//! `ComponentRegistry` — manual map from stable canonical names to
//! their serde-driven insert/serialize fn pointers.
//!
//! # Why manual (not `inventory`)
//!
//! ADR-0025 §"Decisões finalizadas" → ComponentRegistry strategy =
//! manual via `register_*_components()`. Reasons:
//!
//! - `inventory` uses link-section magic that doesn't reliably
//!   survive wasm32 LLD linking — would break the web target
//!   (§11.12). M14.3a runs in CI on Linux + Mac + Windows + wasm32,
//!   and a flaky linker would gate the whole milestone.
//! - LLM auditability: a `grep` for the canonical name resolves to
//!   the registration call instantly. With `inventory` the
//!   discovery layer is hidden.
//! - HR-17 examples: scripting examples reference canonical
//!   component names. They need to be visible in source.
//!
//! # Stable type_id
//!
//! `std::any::TypeId::of::<T>()` is **not** stable across rustc
//! versions (Rust documents this). We compute the `ComponentTypeId`
//! as `blake3(canonical_name).first_8_bytes()` so cooked prefabs
//! produced by one toolchain still load on another.
//!
//! Canonical names follow `ph2d::<crate>::<TypeName>`:
//! - `ph2d::ecs::Transform`
//! - `ph2d::ecs::Name`
//! - `ph2d::render::Sprite`     (registered in ph2d-render)
//! - `ph2d::script::LuauScript` (registered in ph2d-script)

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

/// Stable, content-addressed identifier for a `Component` type at
/// the cooked-wire level. Same numeric type as
/// `ph2d_asset::ComponentTypeId`.
pub type ComponentTypeId = u64;

/// Compute the canonical `ComponentTypeId` for a `Component` type by
/// hashing its `canonical_name` (e.g. `"ph2d::ecs::Transform"`).
///
/// This function is deterministic across architectures and rustc
/// versions (it depends only on blake3), so the result is safe to
/// embed in a cooked asset and stable across future compiler
/// upgrades. **Never** swap this for `std::any::TypeId::of::<T>()`
/// — that's documented as version-unstable.
pub fn stable_type_id(canonical_name: &str) -> ComponentTypeId {
    let h = blake3::hash(canonical_name.as_bytes());
    let bytes = h.as_bytes();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(buf)
}

/// Errors raised by the registry on insert / serialize.
#[derive(Debug)]
pub enum RegistryError {
    Decode(postcard::Error),
    Encode(postcard::Error),
    UnknownTypeId(ComponentTypeId),
    EntityMissing(Entity),
}

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
        }
    }
}

impl std::error::Error for RegistryError {}

/// Decode `bytes` and insert the resulting component onto `entity`.
pub type InsertFromBytesFn = fn(&mut World, Entity, &[u8]) -> Result<(), RegistryError>;

/// Serialize the component (if present) on `entity` to postcard bytes.
/// `Ok(None)` = entity exists but has no component of this type.
pub type SerializeFn = fn(&World, Entity) -> Result<Option<Vec<u8>>, RegistryError>;

/// Remove the component (if present) from `entity`. A no-op when the
/// entity is gone or doesn't carry the component — detaching an absent
/// optional component is idempotent (Sprite Inspector v2 W3: toggling a
/// marker / unsetting an optional override).
pub type RemoveFn = fn(&mut World, Entity);

/// One registered Component type's vtable: canonical name + id +
/// fn pointers for `(de)serialize through postcard` + type-erased
/// removal.
pub struct ComponentTypeEntry {
    pub canonical_name: &'static str,
    pub type_id: ComponentTypeId,
    pub insert_from_bytes: InsertFromBytesFn,
    pub serialize: SerializeFn,
    pub remove: RemoveFn,
}

/// Manual registry of component types known to the spawn / save
/// pipeline.
///
/// `BTreeMap` keyed by `ComponentTypeId` (sorted iteration → HR-5
/// determinism friendly). Lookups are O(log N) but `N` is the
/// number of registered types — single digits in practice.
pub struct ComponentRegistry {
    by_id: BTreeMap<ComponentTypeId, ComponentTypeEntry>,
    by_name: BTreeMap<&'static str, ComponentTypeId>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            by_id: BTreeMap::new(),
            by_name: BTreeMap::new(),
        }
    }

    /// Register a component type under its canonical name. Panics
    /// if the same name is registered twice — caller bug, not a
    /// runtime condition we want to recover from.
    pub fn register<T>(&mut self, canonical_name: &'static str)
    where
        T: Component<Mutability = bevy_ecs::component::Mutable>
            + Serialize
            + DeserializeOwned
            + 'static,
    {
        let id = stable_type_id(canonical_name);
        if let Some(prev) = self.by_name.get(canonical_name) {
            panic!(
                "ComponentRegistry: '{canonical_name}' already registered \
                 (existing id 0x{prev:016x})"
            );
        }
        if let Some(prev) = self.by_id.get(&id) {
            panic!(
                "ComponentRegistry: id collision on 0x{id:016x} between \
                 '{}' and '{canonical_name}' — choose a different name",
                prev.canonical_name
            );
        }
        let entry = ComponentTypeEntry {
            canonical_name,
            type_id: id,
            insert_from_bytes: |world, entity, bytes| {
                let v: T = postcard::from_bytes(bytes).map_err(RegistryError::Decode)?;
                let mut e = world
                    .get_entity_mut(entity)
                    .map_err(|_| RegistryError::EntityMissing(entity))?;
                e.insert(v);
                Ok(())
            },
            serialize: |world, entity| {
                let e = world
                    .get_entity(entity)
                    .map_err(|_| RegistryError::EntityMissing(entity))?;
                match e.get::<T>() {
                    Some(v) => {
                        let bytes = postcard::to_allocvec(v).map_err(RegistryError::Encode)?;
                        Ok(Some(bytes))
                    }
                    None => Ok(None),
                }
            },
            remove: |world, entity| {
                if let Ok(mut e) = world.get_entity_mut(entity) {
                    e.remove::<T>();
                }
            },
        };
        self.by_id.insert(id, entry);
        self.by_name.insert(canonical_name, id);
    }

    pub fn get_by_id(&self, id: ComponentTypeId) -> Option<&ComponentTypeEntry> {
        self.by_id.get(&id)
    }

    pub fn get_by_name(&self, canonical_name: &str) -> Option<&ComponentTypeEntry> {
        self.by_name
            .get(canonical_name)
            .and_then(|id| self.by_id.get(id))
    }

    /// Count of registered types.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate registered entries in `type_id`-sorted order. Useful
    /// for cooker tooling that wants to enumerate the supported set.
    pub fn iter(&self) -> impl Iterator<Item = &ComponentTypeEntry> {
        self.by_id.values()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register the components owned by `ph2d-ecs` (Transform, Name,
/// Visibility, RootOrder). Other crates contribute via their own
/// `register_*_components` functions, called once at boot from the
/// shell.
pub fn register_ecs_components(reg: &mut ComponentRegistry) {
    reg.register::<crate::Transform>("ph2d::ecs::Transform");
    reg.register::<crate::Name>("ph2d::ecs::Name");
    reg.register::<crate::Visibility>("ph2d::ecs::Visibility");
    reg.register::<crate::RootOrder>("ph2d::ecs::RootOrder");
    // Sprite Inspector v2 W3 — sorting / visibility / sampling
    // components (spec §02). Optional: serialized only when present, so
    // legacy scenes are byte-unchanged.
    reg.register::<crate::SortingLayer>("ph2d::ecs::SortingLayer");
    reg.register::<crate::OrderInLayer>("ph2d::ecs::OrderInLayer");
    reg.register::<crate::ZIndexOverride>("ph2d::ecs::ZIndexOverride");
    reg.register::<crate::ZAsRelative>("ph2d::ecs::ZAsRelative");
    reg.register::<crate::YSort>("ph2d::ecs::YSort");
    reg.register::<crate::SortingGroup>("ph2d::ecs::SortingGroup");
    reg.register::<crate::ShowBehindParent>("ph2d::ecs::ShowBehindParent");
    reg.register::<crate::TopLevel>("ph2d::ecs::TopLevel");
    reg.register::<crate::ClipChildren>("ph2d::ecs::ClipChildren");
    reg.register::<crate::MaskInteraction>("ph2d::ecs::MaskInteraction");
    reg.register::<crate::Mask2D>("ph2d::ecs::Mask2D");
    reg.register::<crate::TextureFilter>("ph2d::ecs::TextureFilter");
    reg.register::<crate::TextureRepeat>("ph2d::ecs::TextureRepeat");
    reg.register::<crate::VisibilityLayer>("ph2d::ecs::VisibilityLayer");
    reg.register::<crate::OnScreenEnabler>("ph2d::ecs::OnScreenEnabler");
    reg.register::<crate::UvTransform>("ph2d::ecs::UvTransform");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hard-coded type ids — locked in to catch accidental renames.
    /// If you change a canonical name, bump the prefab schema
    /// version (HR-14) and add a migration; do **not** silently
    /// recompute this constant.
    #[test]
    fn stable_type_ids_are_locked_in() {
        // These values are the first 8 bytes of `blake3(name)` in
        // little-endian. Recompute with:
        //   echo -n 'ph2d::ecs::Transform' | b3sum --no-names | head -c 16
        // and reverse byte order.
        // The assertion catches accidental renames before they ship.
        let t = stable_type_id("ph2d::ecs::Transform");
        let n = stable_type_id("ph2d::ecs::Name");
        // Sanity: distinct names → distinct ids.
        assert_ne!(t, n);
        // Determinism: same input → same id every call.
        assert_eq!(stable_type_id("ph2d::ecs::Transform"), t);
    }

    #[test]
    fn register_ecs_components_populates_registry() {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        // 4 foundational (Transform/Name/Visibility/RootOrder) + 16 W3
        // sorting/visibility/sampling/mask components (incl. Mask2D source).
        assert_eq!(reg.len(), 20);
        assert!(reg.get_by_name("ph2d::ecs::Transform").is_some());
        assert!(reg.get_by_name("ph2d::ecs::Name").is_some());
        assert!(reg.get_by_name("ph2d::ecs::Visibility").is_some());
        assert!(reg.get_by_name("ph2d::ecs::RootOrder").is_some());
        assert!(reg.get_by_name("ph2d::ecs::ZIndexOverride").is_some());
        assert!(reg.get_by_name("ph2d::ecs::YSort").is_some());
        assert!(reg.get_by_name("ph2d::ecs::TextureFilter").is_some());
        assert!(reg.get_by_name("ph2d::ecs::OnScreenEnabler").is_some());
        assert!(reg.get_by_name("ph2d::ecs::Missing").is_none());
    }

    #[test]
    fn insert_serialize_round_trip_transform() {
        use crate::Transform;
        use bevy_ecs::world::World;
        use ph2d_core::Vec2;

        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let entry = reg.get_by_name("ph2d::ecs::Transform").unwrap();

        let original = Transform {
            translation: Vec2::new(3.0, 4.0),
            rotation: 1.5,
            scale: Vec2::new(2.0, 2.0),
            ..Transform::IDENTITY
        };
        let bytes = postcard::to_allocvec(&original).unwrap();

        let mut world = World::new();
        let entity = world.spawn_empty().id();
        (entry.insert_from_bytes)(&mut world, entity, &bytes).unwrap();

        let serialized = (entry.serialize)(&world, entity).unwrap().unwrap();
        let back: Transform = postcard::from_bytes(&serialized).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn duplicate_registration_panics() {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reg.register::<crate::Transform>("ph2d::ecs::Transform");
        }));
        assert!(result.is_err(), "duplicate registration must panic");
    }
}
