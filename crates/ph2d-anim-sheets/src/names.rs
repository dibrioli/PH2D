//! ⭐ **O nome de cada folha, uma fonte só** ([`ph2d_ecs::SheetNames`]): o nome de biblioteca de cada imagem que um
//! Animated usa como folha (a `0` e as extra). O editor de animações e o cartão Image leem-no; a shell publica-o uma
//! vez por quadro, com a biblioteca da sessão, só quando as imagens mudam.

use ph2d_asset::AssetId;
use ph2d_ecs::{Entity, SheetNames, SpriteAnimations, SpriteSheets, World};
use std::collections::BTreeMap;

/// As imagens das folhas de todos os Animated, se mudaram desde a última publicação; `None` = o recurso está em dia.
#[must_use]
pub fn stale_ids(world: &mut World, atlas: &BTreeMap<u32, AssetId>) -> Option<Vec<AssetId>> {
    let mut q = world.query_filtered::<Entity, bevy_ecs::query::Or<(
        bevy_ecs::query::With<SpriteSheets>,
        bevy_ecs::query::With<SpriteAnimations>,
    )>>();
    let es: Vec<Entity> = q.iter(world).collect();
    let mut ids: Vec<AssetId> = es
        .iter()
        .flat_map(|&e| {
            ph2d_render::texture_of(world.get(e), world.get(e), atlas)
                .into_iter()
                .chain(ph2d_ecs::sheet_assets(world, e))
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let em_dia = world
        .get_resource::<SheetNames>()
        .is_some_and(|n| n.ids == ids);
    (!em_dia).then_some(ids)
}

/// Publica os nomes de `ids` que a biblioteca `lib` conhece.
pub fn publish(world: &mut World, ids: Vec<AssetId>, lib: &BTreeMap<AssetId, String>) {
    let names = ids
        .iter()
        .filter_map(|id| lib.get(id).map(|n| (*id, n.clone())))
        .collect();
    world.insert_resource(SheetNames { ids, names });
}
