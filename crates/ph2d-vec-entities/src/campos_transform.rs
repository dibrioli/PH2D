//! **Os campos e os verbos do cartão Transform do Vector, e quem anda com eles.** Doc:
//! `docs/Vector Module/BUGS_vector.md` (o gizmo da forma que ficava para trás, 04/11).
//!
//! ⭐ **Mover um objecto é mover a POSE** — a do gizmo: o pivô, os filhos e tudo o que se mede a partir
//! da origem andam junto. Assar o deslocamento nos pontos deixava-os para trás. Só um caminho cuja
//! geometria é reescrita em mundo a cada quadro (conector, blend, morph, envelope, pele) continua nos
//! pontos ([`crate::transform::anda_pela_pose`]).
//!
//! ⭐ **Girar e espelhar uma forma VIVA também é pose:** a geometria dela é derivada da receita, e o que
//! se assa nos pontos evapora no re-cook. W/H continuam a escrever a caixa, com a receita em passo.

use ph2d_ecs::{Entity, SimWorld, Transform, VecShape};
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{FlipAxis, Rotate90, VecPathId, VecScene, VecXforms};

use crate::entity_map::VecEntityMap;
use crate::transform::{
    anda_pela_pose, build, translate_shape_world, world_transform, xform_of_transform,
};

/// Qual campo: X/Y = canto mínimo da caixa de MUNDO; W/H = o tamanho dela.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VecTransformField {
    X,
    Y,
    W,
    H,
}

fn entidade(map: &VecEntityMap, id: VecPathId) -> Option<Entity> {
    map.get(&id).map(|&bits| Entity::from_bits(bits))
}

/// ⭐ **A porta de mover um objecto por um delta de MUNDO.** `false` se o caminho sumiu.
pub fn mover_no_mundo(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    id: VecPathId,
    delta: [f64; 2],
) -> bool {
    if let Some(e) = entidade(map, id)
        && anda_pela_pose(sim, e)
        && translate_shape_world(sim, e, delta)
    {
        return true;
    }
    scene.translate_path_world(xforms, id, delta[0], delta[1])
}

/// A seta do teclado: nós escolhidos andam pela porta da caneta; sem nós, o objecto inteiro.
pub fn seta(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    pen: &mut PenTool,
    map: &VecEntityMap,
    delta: [f64; 2],
) -> bool {
    if !pen.selected_verts().is_empty() {
        return pen.nudge(scene, delta[0], delta[1]);
    }
    let xf = build(sim, map);
    let mut moveu = false;
    for &id in pen.selected_paths() {
        moveu |= mover_no_mundo(sim, scene, map, &xf, id, delta);
    }
    moveu
}

/// Um campo do Transform escreve na forma `sel`. O undo é o do diff do quadro.
pub fn apply_vec_transform(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    scene: &mut VecScene,
    sel: Option<VecPathId>,
    xforms: &VecXforms,
    field: VecTransformField,
    target: f64,
) {
    let Some(sel) = sel else {
        return;
    };
    let Some((lo, hi)) = scene.path_world_curve_bbox(xforms, sel) else {
        return;
    };
    // O escalonamento acontece na geometria local, em torno do canto local que corresponde ao `lo`
    // de mundo: a razão é a mesma nos dois espaços.
    let local_lo = scene.path_curve_bbox(sel).map_or([0.0, 0.0], |(l, _)| l);
    let escala = |sim: &mut SimWorld, scene: &mut VecScene, sx: f64, sy: f64| {
        if scene.scale_path(sel, sx, sy, local_lo) {
            keep_recipe_in_step(sim, scene, map, sel, sx, sy);
        }
    };
    match field {
        VecTransformField::X | VecTransformField::Y => {
            let delta = if field == VecTransformField::X {
                [target - lo[0], 0.0]
            } else {
                [0.0, target - lo[1]]
            };
            if delta[0].abs() > 1e-9 || delta[1].abs() > 1e-9 {
                mover_no_mundo(sim, scene, map, xforms, sel, delta);
            }
        }
        VecTransformField::W => {
            let w = hi[0] - lo[0];
            if w > 1e-6 && (target.max(1e-4) / w - 1.0).abs() > 1e-9 {
                escala(sim, scene, target.max(1e-4) / w, 1.0);
            }
        }
        VecTransformField::H => {
            let h = hi[1] - lo[1];
            if h > 1e-6 && (target.max(1e-4) / h - 1.0).abs() > 1e-9 {
                escala(sim, scene, 1.0, target.max(1e-4) / h);
            }
        }
    }
}

/// A receita da forma VIVA passa a medir a caixa escalada: sem isto o redimensionamento evapora no
/// primeiro re-cook (medido, `100 → 50 → 100`). Um caminho cru não tem receita, e isso não é falha.
fn keep_recipe_in_step(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    sx: f64,
    sy: f64,
) {
    if let Some(e) = entidade(map, id)
        && let Some(VecShape::Param { w, h, .. }) = sim.world().get::<VecShape>(e).cloned()
    {
        crate::receita::resize_recipe(sim, e, w * sx, h * sy);
        crate::receita::recook_depois_de_redimensionar(sim, scene, e, id);
    }
}

/// A entidade da forma VIVA `id` que se move pela pose; `None` = o verbo assa nos pontos.
fn viva(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<Entity> {
    let e = entidade(map, id)?;
    (sim.world().get::<VecShape>(e).is_some() && anda_pela_pose(sim, e)).then_some(e)
}

/// Muda a pose com o centro da forma parado no mundo (os verbos giram e espelham em torno dele).
fn pose_em_torno_do_centro(
    sim: &mut SimWorld,
    scene: &VecScene,
    e: Entity,
    id: VecPathId,
    muda: impl FnOnce(&mut Transform),
) -> bool {
    let Some((lo, hi)) = scene.path_bbox(id) else {
        return false;
    };
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let onde = |sim: &SimWorld| xform_of_transform(world_transform(sim, e)).apply(c);
    let antes = onde(sim);
    {
        let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) else {
            return false;
        };
        muda(&mut t);
    }
    let depois = onde(sim);
    translate_shape_world(sim, e, [antes[0] - depois[0], antes[1] - depois[1]])
}

/// O campo Rotation (delta em graus, positivo = anti-horário no mundo).
pub fn apply_vec_rotate_by(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    sel: Option<VecPathId>,
    degrees: f64,
) {
    let Some(sel) = sel else {
        return;
    };
    if degrees.abs() < 1e-9 {
        return;
    }
    if let Some(e) = viva(sim, map, sel) {
        #[expect(clippy::cast_possible_truncation, reason = "a pose guarda f32")]
        let rad = degrees.to_radians() as f32;
        pose_em_torno_do_centro(sim, scene, e, sel, |t| t.rotation += rad);
        return;
    }
    let Some((lo, hi)) = scene.path_bbox(sel) else {
        return;
    };
    let pivot = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    scene.rotate_path_by(sel, degrees.to_radians(), pivot);
}

/// Os botões Flip do Transform.
pub fn espelhar(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    axis: FlipAxis,
) {
    if let Some(e) = viva(sim, map, id) {
        pose_em_torno_do_centro(sim, scene, e, id, |t| match axis {
            FlipAxis::Horizontal => t.scale.x = -t.scale.x,
            FlipAxis::Vertical => t.scale.y = -t.scale.y,
        });
    } else {
        scene.flip_path(id, axis);
    }
}

/// Os botões de quarto de volta do Transform (horário e anti-horário são palavras sobre o ECRÃ).
pub fn quarto_de_volta(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    id: VecPathId,
    dir: Rotate90,
) {
    if let Some(e) = viva(sim, map, id) {
        let q = match dir {
            Rotate90::Cw => -std::f32::consts::FRAC_PI_2,
            Rotate90::Ccw => std::f32::consts::FRAC_PI_2,
        };
        pose_em_torno_do_centro(sim, scene, e, id, |t| t.rotation += q);
    } else {
        scene.rotate_path(id, dir);
    }
}
