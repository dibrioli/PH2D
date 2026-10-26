//! **A receita de uma forma viva em passo com a geometria** (`VecShape::Param`, centrada no local 0 com o centro no
//! `Transform`): a porta do tamanho, e o círculo que o solver de CAD moveu ou re-escalou (W4.4,
//! `docs/BlenderConstraints/04_W4_cad.md` §3.2).

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_vec_scene::cad::circulo_de;
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, Xform};

use crate::entity_map::VecEntityMap;

/// **A caixa AUTORADA de uma forma viva passa a medir `w × h`** — a porta que mantém a RECEITA
/// em passo com a geometria. `false` se a entidade não tem receita paramétrica (um path cru, um
/// texto): aí não há nada a manter em passo, e isso não é erro.
///
/// ⚠️ **Sem ela, redimensionar apaga-se sozinho.** A geometria de uma forma viva é DERIVADA
/// (`recook_shape` cozinha de `w`/`h`), e todo redimensionamento do documento escreve os `verts`
/// e mais nada — o `w`/`h` do painel desde sempre, e a alça da moldura desde esta wave. A receita
/// fica a descrever a caixa ANTIGA, e a primeira edição de qualquer parâmetro re-cozinha a partir
/// dela: medido, `100 → 50 → 100`, **o redimensionamento evapora em silêncio**.
///
/// ⚠️ **O sinal é preservado**, e não é detalhe: a reta guarda a DIREÇÃO em `w`/`h`
/// (`recook_shape`), então tomar o módulo aqui viraria uma reta ao contrário no primeiro
/// re-cook.
pub fn resize_recipe(sim: &mut SimWorld, entity: Entity, w: f64, h: f64) -> bool {
    let Some(VecShape::Param {
        kind,
        w: w0,
        h: h0,
        values,
    }) = sim.world().get::<VecShape>(entity).cloned()
    else {
        return false;
    };
    let next = VecShape::Param {
        kind,
        w: w0.signum() * w.abs(),
        h: h0.signum() * h.abs(),
        values,
    };
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        e.insert(next);
        return true;
    }
    false
}

/// Folga do assentamento, relativa ao raio: acima do arredondamento `f32` do `Transform` (o centro anda por ele), muito
/// abaixo do que se vê. Sem ela o passe re-escreveria a forma a cada quadro e cada quadro viraria passo de undo.
const FOLGA: f64 = 1e-4;
/// O passo relativo do `f32` (`6e-8`) com margem: longe da origem o centro só assenta até ele.
const F32_REL: f64 = 1e-6;

/// Um círculo VIVO cuja geometria saiu do centro ou do diâmetro da receita (só o solver de CAD a escreve assim — o
/// Node congela a receita antes de editar): o `Transform` vai para o centro e a receita mede `2r`. Sem isto, a
/// primeira edição de um parâmetro re-cozinharia o círculo no sítio e no tamanho antigos. `drawing` = paths em gesto.
pub(crate) fn assenta_circulos_vivos(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    drawing: &[VecPathId],
) {
    let vivos: Vec<(VecPathId, Entity, f64)> = map
        .iter()
        .filter(|(id, _)| !drawing.contains(id))
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            match sim.world().get_entity(e).ok()?.get::<VecShape>()? {
                VecShape::Param { kind, w, h, .. }
                    if *kind == ShapeKind::Ellipse as u16 && w.abs() == h.abs() =>
                {
                    Some((id, e, w.abs()))
                }
                _ => None,
            }
        })
        .collect();
    for (id, e, d) in vivos {
        let Some((c, r)) = scene.path(id).and_then(|p| circulo_de(p, Xform::IDENTITY)) else {
            continue;
        };
        let mundo = crate::transform::world_transform(sim, e);
        let alvo = crate::transform::xform_of_transform(mundo).apply(c);
        let folga = FOLGA * r.max(1.0) + F32_REL * alvo[0].hypot(alvo[1]);
        if c[0].hypot(c[1]) <= folga && (2.0 * r - d).abs() <= folga {
            continue;
        }
        resize_recipe(sim, e, 2.0 * r, 2.0 * r);
        crate::transform::move_origin_to(sim, scene, e, id, [alvo[0] as f32, alvo[1] as f32]);
    }
}

#[cfg(test)]
#[path = "receita_tests.rs"]
mod tests;
