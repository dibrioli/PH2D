//! **A receita de uma forma viva em passo com a geometria** (`VecShape::Param`, centrada no local 0 com o centro no
//! `Transform`): a porta do tamanho, e o círculo que o solver de CAD moveu ou re-escalou (W4.4,
//! `docs/BlenderConstraints/04_W4_cad.md` §3.2).

use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_vec_scene::cad::circulo_de;
use ph2d_vec_scene::{ShapeKind, VecPath, VecPathId, VecScene, Xform};

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

/// ⭐ **Onde a geometria de uma forma viva está em relação à receita** — o desvio local entre os pontos reais e os
/// mesmos pontos `cozida` (centrados no local 0). A receita diz a FORMA; os pontos dizem o SÍTIO: um *Set Center*,
/// um W/H a partir do canto e um arrasto da caixa deixam a forma fora da origem, e é assim que tem de ficar. Quem
/// lê a receita para pôr algo em cima da forma (as alças) ou para a re-cozinhar passa por aqui. Pelo centro da
/// caixa das âncoras: um `[0, 0]` EXACTO quando os pontos são os cozidos (o re-cook fica idempotente, ao bit).
#[must_use]
pub fn desvio_da_receita(real: &VecPath, cozida: &VecPath) -> [f64; 2] {
    let centro = |p: &VecPath| {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for v in p.verts_all() {
            for i in 0..2 {
                lo[i] = lo[i].min(v.anchor[i]);
                hi[i] = hi[i].max(v.anchor[i]);
            }
        }
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    };
    let (a, b) = (centro(real), centro(cozida));
    let d = [a[0] - b[0], a[1] - b[1]];
    if d[0].is_finite() && d[1].is_finite() {
        d
    } else {
        [0.0, 0.0]
    }
}

/// A geometria (sem estilo) de uma forma viva, CENTRADA no local 0 — o pivô nasce no
/// centro (ADR-0112) e o re-cook é idempotente (o `Transform` fica intacto). `None` para
/// `Text` (que tem o cozimento dele, em [`ph2d_vec_text::glyph`]) e para um `kind`
/// desconhecido (save de uma versão futura: vira path cru, não pânico).
#[must_use]
pub fn recook_shape(shape: &VecShape) -> Option<VecPath> {
    let VecShape::Param {
        kind,
        w,
        h,
        ref values,
    } = *shape
    else {
        return None;
    };
    let k = ShapeKind::from_u16(kind)?;
    let (hw, hh) = (w.abs() * 0.5, h.abs() * 0.5);
    // A reta guarda a DIREÇÃO (w/h com sinal): centrada, vai de −d/2 a +d/2. As demais
    // são simétricas em torno do centro, então a caixa centrada dá o mesmo resultado.
    let (a, b) = if k == ShapeKind::Line {
        ([-w * 0.5, -h * 0.5], [w * 0.5, h * 0.5])
    } else {
        ([-hw, -hh], [hw, hh])
    };
    Some(ph2d_vec_scene::cook(k, a, b, values))
}

/// Substitui a GEOMETRIA do path `id` pela forma re-cozida (centrada), preservando id e
/// estilo (fill/stroke). `true` se re-cozinhou.
pub fn recook_into(scene: &mut VecScene, id: VecPathId, shape: &VecShape) -> bool {
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

/// ⭐ O re-cook de uma forma JÁ viva: troca a forma de `antes` por `depois` e deixa-a onde estava
/// ([`desvio_da_receita`], medido contra a receita ANTIGA: `depois` pode
/// ter outra caixa de âncoras, como uma estrela com mais pontas). Sem isto, a forma movida ou com o
/// centro mudado saltava de volta ao local 0 na primeira edição de um parâmetro.
pub fn recook_no_sitio(
    scene: &mut VecScene,
    id: VecPathId,
    antes: &VecShape,
    depois: &VecShape,
) -> bool {
    let d = match (scene.path(id), recook_shape(antes)) {
        (Some(real), Some(cozida)) => desvio_da_receita(real, &cozida),
        _ => [0.0, 0.0],
    };
    if !recook_into(scene, id, depois) {
        return false;
    }
    if d != [0.0, 0.0]
        && let Some(p) = scene.path_mut(id)
    {
        p.for_each_vert_mut(|v| {
            for q in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                *q = [q[0] + d[0], q[1] + d[1]];
            }
        });
    }
    true
}

/// ⭐ Depois de um redimensionamento (a receita já mede a caixa nova): os pontos passam a ser os da
/// receita, no sítio. Escalar os pontos achata o raio e a estrela, que a receita não achata: as alças e
/// o desenho discordavam até ao primeiro re-cook, que era quando a forma mudava de cara.
pub fn recook_depois_de_redimensionar(
    sim: &SimWorld,
    scene: &mut VecScene,
    entity: Entity,
    id: VecPathId,
) -> bool {
    match sim.world().get::<VecShape>(entity).cloned() {
        Some(s @ VecShape::Param { .. }) => recook_no_sitio(scene, id, &s, &s),
        _ => false,
    }
}

/// Folga do assentamento, relativa ao raio: acima do arredondamento `f32` do `Transform` (o centro anda por ele), muito
/// abaixo do que se vê. Sem ela o passe re-escreveria a forma a cada quadro e cada quadro viraria passo de undo.
const FOLGA: f64 = 1e-4;
/// O passo relativo do `f32` (`6e-8`) com margem: longe da origem o centro só assenta até ele.
const F32_REL: f64 = 1e-6;

/// Um círculo VIVO cujo diâmetro saiu da receita (só o solver de CAD o escreve assim — o Node congela a receita antes
/// de editar): o `Transform` vai para o centro e a receita mede `2r`. Sem isto, a primeira edição de um parâmetro
/// re-cozinharia o círculo no tamanho antigo. Com o MESMO diâmetro nada a fazer: fora do centro é o *Set Center* do
/// artista, e quem lê a receita mede o sítio ([`desvio_da_receita`]). `drawing` = paths em gesto.
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
        if (2.0 * r - d).abs() <= folga {
            continue;
        }
        resize_recipe(sim, e, 2.0 * r, 2.0 * r);
        crate::transform::move_origin_to(sim, scene, e, id, [alvo[0] as f32, alvo[1] as f32]);
    }
}

#[cfg(test)]
#[path = "receita_tests.rs"]
mod tests;
