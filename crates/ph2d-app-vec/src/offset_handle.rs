//! ⭐ **A alça do Offset Path na tela** (dono 02/11: «faça tudo como no inspector») — arrastar a borda
//! cresce ou encolhe a forma ao vivo, como o *Dynamic Offset* do Inkscape e o *Contour* do Affinity.
//!
//! A alça senta na borda DIREITA da fonte, a um quarto da altura (a meia altura é da pega de escala
//! do gizmo, que ela roubaria), deslocada pela distância viva; uma haste liga-a à borda. Arrastá-la
//! escreve a mesma relação que o campo `Distance (px)` ([`crate::offset_live::arm`]) — campo e alça
//! são duas mãos sobre o mesmo número.

use ph2d_ecs::SimWorld;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

/// A alça agarrada: a forma e a borda de onde a distância se mede (congelada no press).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grab {
    pub path: VecPathId,
    pub edge: [f64; 2],
}

/// A alça só aparece com a secção **Offset Path** à vista (ela e o grupo Advanced abertos) ou com
/// um offset VIVO na forma — fora disso ela seria um ponto a mais em toda forma selecionada.
#[must_use]
pub fn shown(
    store: &ph2d_editor_core::interaction::WidgetStore,
    sim: &SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
) -> bool {
    let aberta = !store.is_collapsed(ph2d_tool_vector::ids::VECTOR_SECTION_EXPAND)
        && !store.is_collapsed(ph2d_panel_vector::ids::VECTOR_SECTION_ADVANCED);
    aberta || crate::offset_live::spec_of(sim, map, id).is_some()
}

/// `(borda, alça)` em mundo — `None` sem caixa (forma vazia).
#[must_use]
pub fn handle(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    id: VecPathId,
) -> Option<([f64; 2], [f64; 2])> {
    let (lo, hi) = scene.path_world_curve_bbox(xforms, id)?;
    let edge = [hi[0], lo[1] + (hi[1] - lo[1]) * 0.25];
    let d = crate::offset_live::spec_of(sim, map, id).map_or(0.0, |o| o.d);
    Some((edge, [edge[0] + d, edge[1]]))
}

/// O press: agarra a alça se `world` cai a `hit_r` dela.
#[must_use]
pub fn press_at(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    id: VecPathId,
    world: [f64; 2],
    hit_r: f64,
) -> Option<Grab> {
    let (edge, at) = handle(sim, scene, map, xforms, id)?;
    let perto = (world[0] - at[0]).hypot(world[1] - at[1]) <= hit_r;
    perto.then_some(Grab { path: id, edge })
}

/// O arrasto: a distância é o quanto o dedo passou da borda (à esquerda dela, encolhe), com os
/// knobs do painel. De volta à borda = sem offset (a lei do [`crate::offset_live::arm`]).
pub fn drag(sim: &mut SimWorld, map: &VecEntityMap, grab: Grab, world: [f64; 2]) {
    let d = world[0] - grab.edge[0];
    let knobs = crate::expand::expand_knobs();
    crate::offset_live::arm(sim, map, &[grab.path], crate::expand::spec_with(knobs, d));
}

#[cfg(test)]
#[path = "offset_handle_tests.rs"]
mod tests;
