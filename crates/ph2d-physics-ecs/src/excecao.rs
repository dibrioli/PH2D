//! **O «Collides with» de uma forma** (GEF02 B9, dono 08/11): a régua (a matriz VIVA no mundo) e o estado de uma
//! seleção — a linha efetiva da primeira, os bits que divergem entre elas e se alguma tem exceção.

use crate::{CollisionOverride, LayerMatrix, MAX_LAYERS, RigidBody};
use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

/// A matriz que o mundo corre agora (`PhysicsBridge::settings_vivos`), posta no mundo por quem publica as camadas.
/// Ausente = tudo colide, o default da matriz.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveLayerRows(pub [u32; MAX_LAYERS]);

impl LiveLayerRows {
    /// A linha da camada `layer`.
    #[must_use]
    pub fn row(world: &World, layer: u8) -> u32 {
        let rows = world
            .get_resource::<Self>()
            .map_or_else(|| LayerMatrix::all().rows(), |r| r.0);
        rows[usize::from(layer).min(MAX_LAYERS - 1)]
    }

    /// Põe as linhas no mundo, se mudaram.
    pub fn install(world: &mut World, rows: [u32; MAX_LAYERS]) {
        if world.get_resource::<Self>().map(|r| r.0) != Some(rows) {
            world.insert_resource(Self(rows));
        }
    }
}

/// A forma que guarda a exceção de `e`: a principal de um corpo (opção B), ou `e` (peça, corpo antigo).
#[must_use]
pub fn forma_da_excecao(world: &World, e: Entity) -> Entity {
    if world.get::<RigidBody>(e).is_some() {
        crate::main_shape(world, e).unwrap_or(e)
    } else {
        e
    }
}

/// `[include, exclude]` da exceção da forma `forma`; `[0, 0]` sem ela.
#[must_use]
pub fn mascaras(world: &World, forma: Entity) -> [u32; 2] {
    world
        .get::<CollisionOverride>(forma)
        .map_or([0; 2], |o| [o.include, o.exclude])
}

/// A linha efetiva da forma de `e`, e se ela tem exceção. `None` sem `Collider`.
#[must_use]
pub fn linha_efetiva(world: &World, e: Entity) -> Option<(u32, bool)> {
    let forma = forma_da_excecao(world, e);
    let col = world.get::<crate::Collider>(forma)?;
    let o = world.get::<CollisionOverride>(forma).copied();
    let row = LiveLayerRows::row(world, col.layer);
    Some((CollisionOverride::effective_row(row, o), o.is_some()))
}

/// **A seleção diante do «Collides with»**: `(linha da primária, misto, difere)`. `misto` = os bits em que alguma
/// forma da seleção discorda da primária; `difere` = alguma tem exceção. Quem não tem `Collider` não conta.
#[must_use]
pub fn estado_da_selecao(world: &World, primaria: Entity, selecao: &[Entity]) -> (u32, u32, bool) {
    let (row, mut difere) = linha_efetiva(world, primaria).unwrap_or_default();
    let mut misto = 0;
    for &e in selecao {
        if let Some((r, d)) = linha_efetiva(world, e) {
            misto |= r ^ row;
            difere |= d;
        }
    }
    (row, misto, difere)
}
