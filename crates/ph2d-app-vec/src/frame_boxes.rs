//! ⭐ **As CAIXAS que o enquadramento encaixa** (F8, `ph2d_render::camera_frame`): o que a cena
//! DESENHA, família a família — a imagem como é desenhada (âncora, base, malha posada), a forma pela
//! curva nas `VecXforms`, e o que a shell souber das famílias que esta crate não conhece (o Flip).
//!
//! ⛔ Antes da F8 o *Frame All* só contava imagens (`scene_sprites_bbox_world`): numa cena de formas
//! ele caía na «cena vazia». O escondido não conta (não é desenhado); o trancado conta (os dois
//! oráculos — `oraculo_enquadrar/`).

use ph2d_ecs::{ChildOf, Entity, SimWorld, VecPathRef};
use ph2d_render::WorldBbox;
use ph2d_render::camera_frame::union;
use ph2d_vec_scene::{VecScene, VecXforms};

/// A caixa de MUNDO da curva de uma forma (`None` = forma sem geometria).
#[must_use]
pub fn path_box(scene: &VecScene, xf: &VecXforms, id: u64) -> Option<WorldBbox> {
    let (lo, hi) = scene.path_world_curve_bbox(xf, id)?;
    #[allow(clippy::cast_possible_truncation)]
    Some(WorldBbox {
        min: [lo[0] as f32, lo[1] as f32],
        max: [hi[0] as f32, hi[1] as f32],
    })
}

/// Junta `b` em `acc`.
pub fn grow(acc: &mut Option<WorldBbox>, b: Option<WorldBbox>) {
    if let Some(b) = b {
        *acc = Some(union(*acc, b));
    }
}

/// ⭐ **O que a cena DESENHA**: as imagens do mundo de apresentação (já sem as escondidas — o
/// extract não as emite), as formas que entraram na ORDEM do quadro (`drawn_paths`, a verdade do
/// que o presente pinta) e as caixas de `others`.
#[must_use]
pub fn scene_box(
    present: &mut ph2d_ecs::World,
    scene: &VecScene,
    xf: &VecXforms,
    drawn_paths: impl IntoIterator<Item = u64>,
    others: impl IntoIterator<Item = WorldBbox>,
) -> Option<WorldBbox> {
    let mut acc = ph2d_render::scene_sprites_bbox_world(present);
    for id in drawn_paths {
        grow(&mut acc, path_box(scene, xf, id));
    }
    for b in others {
        grow(&mut acc, Some(b));
    }
    acc
}

/// ⭐ **A caixa de um objecto SELECIONADO** e dos seus descendentes desenhados — a imagem, a forma,
/// ou o que `other` responder (o Flip); um grupo é a união dos filhos não escondidos. `None` = nada
/// desenhado debaixo dele (o objecto vazio fica de fora: não tem tamanho para encaixar).
pub fn object_box(
    sim: &SimWorld,
    present: &mut ph2d_ecs::World,
    scene: &VecScene,
    xf: &VecXforms,
    root: Entity,
    other: &mut dyn FnMut(Entity) -> Option<WorldBbox>,
) -> Option<WorldBbox> {
    let w = sim.world();
    let mut acc = None;
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        grow(
            &mut acc,
            ph2d_render::selection_bbox_world(present, e.to_bits()),
        );
        if let Some(r) = w.get::<VecPathRef>(e) {
            grow(&mut acc, path_box(scene, xf, r.0));
        }
        grow(&mut acc, other(e));
        stack.extend(w.iter_entities().filter_map(|c| {
            let kid = c.id();
            let mine = w.get::<ChildOf>(kid).is_some_and(|p| p.parent() == e);
            (mine && !ph2d_ecs::is_hidden_in_tree(w, kid)).then_some(kid)
        }));
    }
    acc
}

#[cfg(test)]
#[path = "frame_boxes_tests.rs"]
mod tests;
