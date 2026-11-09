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

/// A forma que guarda a exceção de `e`: a [`crate::forma_para_editar`] de um corpo (opção B; a mesma
/// que a escrita alveja, olho aberto ou não), ou `e` (peça, corpo antigo).
#[must_use]
pub fn forma_da_excecao(world: &World, e: Entity) -> Entity {
    if world.get::<RigidBody>(e).is_some() {
        crate::forma_para_editar(world, e).unwrap_or(e)
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

/// **A seleção diante do «Collides with»**.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EstadoDaSelecao {
    /// A linha efetiva da primária.
    pub linha: u32,
    /// Os bits em que alguma forma da seleção discorda da primária.
    pub misto: u32,
    /// Alguma forma da seleção tem exceção.
    pub difere: bool,
    /// Os bits que alguma forma da seleção fixa na exceção (`include | exclude`): os quadros a destacar.
    pub fixos: u32,
}

/// O [`EstadoDaSelecao`] de `primaria` com `selecao`. Quem não tem `Collider` não conta.
#[must_use]
pub fn estado_da_selecao(world: &World, primaria: Entity, selecao: &[Entity]) -> EstadoDaSelecao {
    let fixos_de = |e| {
        let [i, x] = mascaras(world, forma_da_excecao(world, e));
        i | x
    };
    let (linha, difere) = linha_efetiva(world, primaria).unwrap_or_default();
    let mut s = EstadoDaSelecao {
        linha,
        difere,
        fixos: fixos_de(primaria),
        ..EstadoDaSelecao::default()
    };
    for &e in selecao {
        if let Some((r, d)) = linha_efetiva(world, e) {
            s.misto |= r ^ linha;
            s.difere |= d;
            s.fixos |= fixos_de(e);
        }
    }
    s
}
