//! **«Merge by distance»** do cartão Shape: a lei pura é [`ph2d_shape_handles::fusao`]; aqui a seleção do canvas,
//! o relatório e o `ColliderOutline` que a fila grava (um passo de undo, «Custom»).

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor_core::OutlineEdit;
use ph2d_physics_ecs::{ColliderOutline, OutlineRing};
use ph2d_shape_handles::pontos::PontosNoCanvas;
use std::sync::Arc;

/// O último «Merge» (sessão: fora do gravar e do undo). O cartão só o mostra enquanto os anéis são os dele.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct MergeReport {
    pub shape: Entity,
    pub rings: Arc<[OutlineRing]>,
    pub merged: u32,
    pub kept: u32,
}

/// O `ColliderOutline` que a edição `e` do cartão pede para `shape`; `None` = nada a gravar.
pub fn outline_edited(world: &mut World, shape: Entity, e: OutlineEdit) -> Option<ColliderOutline> {
    let cur = world
        .get::<ColliderOutline>(shape)
        .cloned()
        .unwrap_or_default();
    let OutlineEdit::Merge(px) = e else {
        return Some(crate::asked_with(&cur, e));
    };
    let no_canvas = ph2d_shape_handles::editando(world) == Some(shape.to_bits());
    let sel = match world.get_resource::<PontosNoCanvas>() {
        Some(p) if no_canvas => p.selecionados.clone(),
        _ => Vec::new(),
    };
    let f = ph2d_shape_handles::fusao::fundidos(&cur.rings, &sel, px * 16.0)
        .filter(|f| f.fundidos > 0 || f.retidos > 0);
    let next = f
        .as_ref()
        .filter(|f| f.fundidos > 0)
        .map(|f| ColliderOutline {
            rings: Arc::from(f.rings.clone()),
            custom: true,
            ..cur.clone()
        });
    if let (Some(f), Some(_), Some(mut p)) = (
        &f,
        &next,
        world
            .get_resource_mut::<PontosNoCanvas>()
            .filter(|_| no_canvas),
    ) {
        p.selecionados.clone_from(&f.selecionados);
    }
    world.insert_resource(MergeReport {
        shape,
        rings: next
            .as_ref()
            .map_or_else(|| cur.rings.clone(), |n| n.rings.clone()),
        merged: f.as_ref().map_or(0, |f| f.fundidos) as u32,
        kept: f.as_ref().map_or(0, |f| f.retidos) as u32,
    });
    next
}

/// O relatório do último «Merge» de `shape`, se os anéis ainda são os que ele deixou.
#[must_use]
pub fn merged(world: &World, shape: Entity, o: &ColliderOutline) -> Option<(u32, u32)> {
    world
        .get_resource::<MergeReport>()
        .filter(|r| r.shape == shape && r.rings == o.rings)
        .map(|r| (r.merged, r.kept))
}
