//! Quais quinas ganham alça de raio neste frame — a política, num ponto testável.
//!
//! O motor (`ph2d_vec_scene::corner_live`) sabe onde há quina; o `PenTool` sabe onde a
//! alça é desenhada e agarrada. O que **só a shell** sabe é se aquele path é uma **forma
//! viva** — e essa é a pergunta que decide se a alça pode existir.
//!
//! # Por que uma Live Shape NÃO tem alça de raio (e o bug que isto evita)
//!
//! Uma forma paramétrica guarda uma RECEITA no `VecShape::Param` (kind + w/h + valores), e
//! a geometria dela é **derivada**: `vec_shape_live::recook_into` reescreve `path.verts`
//! INTEIRO a cada mudança de parâmetro. O `corner_radius` mora dentro do vértice — logo,
//! um raio autorado numa Live Shape sobrevive até o instante em que o usuário encosta num
//! slider do painel, e some **sem erro nenhum**.
//!
//! Isso é pior que não funcionar: funciona, o usuário confia, e um arrasto de slider
//! desfaz o trabalho dele em silêncio. (E não dá para "consertar" preservando os raios no
//! recook: a CONTAGEM de vértices é função dos parâmetros — mexer no slider de lados de um
//! polígono muda quantas quinas existem. Não há para onde levar o raio da quina que
//! deixou de existir.)
//!
//! O raio de uma forma viva é um **campo dela** (o `Radius` do painel, que o polígono, a
//! estrela e o round-rect já têm — inclusive por-canto no round-rect). O raio por-vértice é
//! para caminho **desenhado**. É a mesma divisão do Illustrator: uma live shape tem as
//! propriedades dela; Live Corners é o que se ganha depois de expandir. Aqui "expandir" é
//! **Convert to Curves**, que já existe e é literalmente descartar o `VecShape`.

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_vec_scene::VecScene;
use ph2d_vec_scene::corner_live::{CORNER_HANDLE_PARK_PX, CornerHandleView};

use crate::vec_entities::VecEntityMap;

/// Este path é uma forma **viva** (geometria derivada de parâmetros)?
#[must_use]
pub(crate) fn is_live_shape(
    sim: &SimWorld,
    map: &VecEntityMap,
    id: ph2d_vec_scene::VecPathId,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    sim.world()
        .get::<VecShape>(Entity::from_bits(bits))
        .is_some()
}

/// As alças de raio a desenhar neste frame: as do path selecionado, **se ele for um
/// caminho desenhado**. Uma forma viva devolve vazio (ver o módulo).
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: Option<ph2d_vec_scene::VecPathId>,
    xforms: &ph2d_vec_scene::VecXforms,
    px_to_world: f64,
) -> Vec<CornerHandleView> {
    let Some(id) = selected else {
        return Vec::new();
    };
    if is_live_shape(sim, map, id) {
        return Vec::new();
    }
    ph2d_vec_edit::corner_handle::view(
        scene,
        id,
        &ph2d_vec_scene::xform_of(xforms, id),
        CORNER_HANDLE_PARK_PX * px_to_world,
    )
}

#[cfg(test)]
#[path = "corner_handles_tests.rs"]
mod tests;
