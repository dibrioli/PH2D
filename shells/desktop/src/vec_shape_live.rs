//! Live Shapes das formas paramétricas (rect/ellipse/polygon/star/rrect/spiral/line/
//! arc) — o mesmo tratamento do texto ([`crate::vec_text`]).
//!
//! Uma forma desenhada **nasce viva**: ao soltar o gesto, a geometria é RE-COZIDA
//! CENTRADA no local 0 (o pivô nasce no centro dela), o `Transform` recebe o centro do
//! retângulo autorado, e a entidade ganha o componente [`VecShape`] com os parâmetros
//! (`w`/`h` + sides/points/radius/turns/degrees). A partir daí a geometria é uma função
//! pura dos parâmetros — mudá-los re-cozinha a forma.
//!
//! **"Convert to Curves"** de uma forma paramétrica é só **descartar o `VecShape`**: a
//! geometria já assada na cena vira um path cru, editável com a caneta. (O texto é o
//! único que também explode — num grupo de paths por-letra.)
//!
//! O `w`/`h` vem do retângulo AUTORADO do gesto (`ShapeTool::bounds`), não da bbox da
//! geometria: num polígono/estrela elas diferem (a bbox de um triângulo é menor que a
//! elipse que o circunscreve), e é o retângulo que o usuário desenhou.

use ph2d_ecs::{Entity, SimWorld, Transform, VecShape};
use ph2d_vec_edit::{ShapeKind, ShapeParams, ShapeTool};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// Os parâmetros da forma VIVA a partir do gesto: `w`/`h` COM SINAL (o `Line` precisa
/// da direção; as demais usam o módulo) e o centro implícito no `Transform`.
#[must_use]
fn shape_from_drag(
    kind: ShapeKind,
    params: ShapeParams,
    start: [f64; 2],
    cur: [f64; 2],
    px_to_world: f64,
) -> VecShape {
    let w = cur[0] - start[0];
    let h = cur[1] - start[1];
    match kind {
        ShapeKind::Rectangle => VecShape::Rectangle { w, h },
        ShapeKind::Ellipse => VecShape::Ellipse { w, h },
        ShapeKind::Polygon => VecShape::Polygon {
            w,
            h,
            sides: params.sides,
        },
        ShapeKind::Star => VecShape::Star {
            w,
            h,
            points: params.star_points,
            inner_ratio: params.star_inner_ratio,
        },
        ShapeKind::RoundRect => VecShape::RoundRect {
            w,
            h,
            // O raio é autorado em PIXELS; a forma guarda mundo (a geometria é mundo).
            radius: params.corner_radius_px * px_to_world,
        },
        ShapeKind::Spiral => VecShape::Spiral {
            w,
            h,
            turns: params.spiral_turns,
        },
        ShapeKind::Line => VecShape::Line { w, h },
        ShapeKind::Arc => VecShape::Arc {
            w,
            h,
            degrees: params.arc_degrees,
        },
    }
}

/// A geometria (sem estilo) de uma forma viva, CENTRADA no local 0 — o pivô nasce no
/// centro (ADR-0112) e o re-cook é idempotente (o `Transform` fica intacto). `None`
/// para `Text` (o texto tem o cozimento dele em [`crate::vec_glyph`]).
#[must_use]
pub(crate) fn recook_shape(shape: &VecShape) -> Option<VecPath> {
    let half = |v: f64| v.abs() * 0.5;
    Some(match *shape {
        VecShape::Rectangle { w, h } => {
            ph2d_vec_scene::rectangle([-half(w), -half(h)], [half(w), half(h)])
        }
        VecShape::RoundRect { w, h, radius } => {
            ph2d_vec_scene::rounded_rect([-half(w), -half(h)], [half(w), half(h)], radius)
        }
        VecShape::Ellipse { w, h } => ph2d_vec_scene::ellipse([0.0, 0.0], half(w), half(h)),
        VecShape::Polygon { w, h, sides } => {
            ph2d_vec_scene::regular_polygon([0.0, 0.0], half(w), half(h), sides)
        }
        VecShape::Star {
            w,
            h,
            points,
            inner_ratio,
        } => ph2d_vec_scene::star([0.0, 0.0], half(w), half(h), points, inner_ratio),
        VecShape::Spiral { w, h, turns } => {
            ph2d_vec_scene::spiral([0.0, 0.0], half(w), half(h), turns)
        }
        // A reta guarda a DIREÇÃO (w/h com sinal): centrada, vai de −d/2 a +d/2.
        VecShape::Line { w, h } => ph2d_vec_scene::line([-w * 0.5, -h * 0.5], [w * 0.5, h * 0.5]),
        VecShape::Arc { w, h, degrees } => {
            ph2d_vec_scene::arc([0.0, 0.0], half(w), half(h), degrees)
        }
        VecShape::Text(_) => return None,
    })
}

/// Substitui a GEOMETRIA do path `id` pela forma re-cozida (centrada), preservando id
/// e estilo (fill/stroke). `true` se re-cozinhou.
pub(crate) fn recook_into(scene: &mut VecScene, id: VecPathId, shape: &VecShape) -> bool {
    let Some(geom) = recook_shape(shape) else {
        return false;
    };
    let Some(p) = scene.path_mut(id) else {
        return false;
    };
    p.verts = geom.verts;
    p.closed = geom.closed;
    p.subpaths = geom.subpaths;
    p.fill_rule = geom.fill_rule;
    true
}

/// A forma recém-desenhada (a última commitada pelo `ShapeTool`) NASCE VIVA: re-cozinha
/// a geometria centrada no local 0, põe o centro do gesto no `Transform` e pendura o
/// `VecShape`. Idempotente — uma vez viva (componente presente), não faz nada. Roda
/// pós-`sync` (a entidade existe) e ANTES do `settle` (que pula formas vivas).
pub(crate) fn make_committed_shape_live(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    tool: &ShapeTool,
) {
    let Some(id) = tool.selected() else { return };
    let Some(&bits) = map.get(&id) else { return };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecShape>(entity).is_some() {
        return; // já é viva
    }
    let (start, cur) = tool.bounds();
    let shape = shape_from_drag(tool.kind(), tool.params(), start, cur, tool.px_to_world());
    if !recook_into(scene, id, &shape) {
        return;
    }
    // O centro do retângulo autorado é a pose; a geometria já está centrada nele.
    let cx = ((start[0] + cur[0]) * 0.5) as f32;
    let cy = ((start[1] + cur[1]) * 0.5) as f32;
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(cx, cy);
        }
        e.insert(shape);
    }
}

/// "Convert to Curves" de formas paramétricas (NÃO-texto): descarta o `VecShape` — a
/// geometria já assada vira um path cru, editável com a caneta. Devolve quantas
/// converteu.
pub(crate) fn drop_shape_params(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> usize {
    let mut n = 0;
    for id in selection {
        let Some(&bits) = map.get(id) else { continue };
        let entity = Entity::from_bits(bits);
        // O texto já foi explodido antes; aqui só as paramétricas.
        if matches!(sim.world().get::<VecShape>(entity), Some(VecShape::Text(_))) {
            continue;
        }
        if sim.world().get::<VecShape>(entity).is_some()
            && let Ok(mut e) = sim.world_mut().get_entity_mut(entity)
        {
            e.remove::<VecShape>();
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Toda forma viva nasce CENTRADA no local 0 — o pivô é o centro do GERADOR (num
    /// polígono ímpar isso não é a bbox, é o circunscrito: o centro natural da forma).
    /// Invariante robusto: toda âncora cabe na caixa centrada `±w/2 × ±h/2`.
    #[test]
    fn every_live_shape_cooks_centered_on_the_local_origin() {
        let shapes = [
            VecShape::Rectangle { w: 4.0, h: 2.0 },
            VecShape::Ellipse { w: 4.0, h: 2.0 },
            VecShape::Polygon {
                w: 4.0,
                h: 4.0,
                sides: 5, // ímpar de propósito
            },
            VecShape::Star {
                w: 4.0,
                h: 4.0,
                points: 5,
                inner_ratio: 0.5,
            },
            VecShape::RoundRect {
                w: 4.0,
                h: 2.0,
                radius: 0.3,
            },
            VecShape::Spiral {
                w: 4.0,
                h: 4.0,
                turns: 3,
            },
            VecShape::Line { w: 4.0, h: 2.0 },
            VecShape::Arc {
                w: 4.0,
                h: 4.0,
                degrees: 180.0,
            },
        ];
        for s in shapes {
            let p = recook_shape(&s).expect("forma paramétrica cozinha");
            let (hw, hh) = match s {
                VecShape::Rectangle { w, h }
                | VecShape::Ellipse { w, h }
                | VecShape::Polygon { w, h, .. }
                | VecShape::Star { w, h, .. }
                | VecShape::RoundRect { w, h, .. }
                | VecShape::Spiral { w, h, .. }
                | VecShape::Line { w, h }
                | VecShape::Arc { w, h, .. } => (w.abs() * 0.5, h.abs() * 0.5),
                VecShape::Text(_) => unreachable!(),
            };
            for v in p.verts_all() {
                assert!(
                    v.anchor[0].abs() <= hw + 1e-6 && v.anchor[1].abs() <= hh + 1e-6,
                    "{s:?}: âncora {:?} fora da caixa centrada ±{hw}×±{hh}",
                    v.anchor
                );
            }
        }
    }

    /// O texto não passa pelo cozimento paramétrico (tem o dele, em `vec_glyph`).
    #[test]
    fn text_is_not_cooked_here() {
        let t = VecShape::Text(ph2d_ecs::VecTextParams {
            text: "a".into(),
            origin: [0.0, 0.0],
            family: None,
            size: 1.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: 0,
            axes: Vec::new(),
        });
        assert!(recook_shape(&t).is_none());
    }
}
