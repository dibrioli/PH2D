//! Live Shapes — o nascimento e o re-cozimento de uma forma paramétrica.
//!
//! Uma forma desenhada **nasce viva**: ao soltar o gesto, a geometria é RE-COZIDA
//! CENTRADA no local 0 (o pivô nasce no centro dela), o `Transform` recebe o centro do
//! retângulo autorado, e a entidade ganha o componente `VecShape::Param` com o `kind` +
//! os parâmetros. A partir daí a geometria é uma função pura deles — mudá-los re-cozinha
//! a forma (é o que o painel faz, em [`crate::vec_shape_params`]).
//!
//! **"Convert to Curves"** de uma forma paramétrica é só **descartar o `VecShape`**: a
//! geometria já assada na cena vira um path cru, editável com a caneta. (O texto é o
//! único que também explode — num grupo de paths por-letra.)
//!
//! O `w`/`h` vem do retângulo AUTORADO do gesto (`ShapeTool::bounds`), não da bbox da
//! geometria: num polígono/estrela elas diferem (a bbox de um triângulo é menor que a
//! elipse que o circunscreve), e é o retângulo que o usuário desenhou.
//!
//! Nada aqui sabe QUE formas existem — o cozimento é o do catálogo
//! (`ph2d_vec_scene::cook`), a mesma porta que o preview do arrasto usa.

use ph2d_ecs::{Entity, MAX_SHAPE_VALUES, SimWorld, Transform, VecShape};
use ph2d_vec_edit::ShapeTool;
use ph2d_vec_scene::{MAX_SHAPE_FIELDS, ShapeKind, VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// O ECS guarda os valores num array primitivo próprio (ele não depende do vetor). Se os
/// dois tetos divergissem, o `VecShape` truncaria parâmetros em silêncio.
const _: () = assert!(MAX_SHAPE_VALUES == MAX_SHAPE_FIELDS);

/// A geometria (sem estilo) de uma forma viva, CENTRADA no local 0 — o pivô nasce no
/// centro (ADR-0112) e o re-cook é idempotente (o `Transform` fica intacto). `None` para
/// `Text` (que tem o cozimento dele, em [`crate::vec_glyph`]) e para um `kind`
/// desconhecido (save de uma versão futura: vira path cru, não pânico).
#[must_use]
pub(crate) fn recook_shape(shape: &VecShape) -> Option<VecPath> {
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

/// A forma recém-desenhada NASCE VIVA: re-cozinha a geometria centrada no local 0, põe o centro
/// do gesto no `Transform` e pendura o `VecShape`. Roda pós-`sync` (a entidade existe) e ANTES do
/// `settle` (que pula formas vivas).
///
/// ⚠️ **É um EVENTO de uma vez, não um invariante por-frame** — o pedido vem do
/// `ShapeTool::pending_live` e é CONSUMIDO quando o nascimento acontece.
///
/// A versão anterior lia o `selected()` (que vive para sempre) e se guardava só com *"o
/// componente está presente?"*. Essa pergunta tem DOIS motivos para dar "não": a forma ainda não
/// nasceu, **ou o artista congelou a receita de propósito** (Convert to Curves / o gesto de
/// quina). Lendo o 2º como o 1º, o frame seguinte ressuscitava a receita e chamava o
/// `recook_into` — que faz `p.verts = geom.verts` e **apaga todo `corner_radius` do caminho**.
/// Sobrevivia só o raio escrito depois do recook: *"a mesma ferramenta desfaz o que tinha feito
/// no outro ponto"* (Enio). [[feedback_a_condition_that_enumerates_its_readers_rots]]
pub(crate) fn make_committed_shape_live(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    tool: &mut ShapeTool,
) {
    let Some(id) = tool.pending_live() else {
        return;
    };
    // A entidade pode não existir no frame do commit; o pedido sobrevive para o próximo.
    let Some(&bits) = map.get(&id) else { return };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecShape>(entity).is_some() {
        tool.clear_pending_live(); // já é viva — o pedido está cumprido
        return;
    }
    let (start, cur) = tool.bounds();
    let shape = VecShape::Param {
        kind: tool.kind().as_u16(),
        w: cur[0] - start[0],
        h: cur[1] - start[1],
        values: tool.values(), // já em MUNDO (a shell converteu na fronteira)
    };
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
        // O nascimento ACONTECEU: consome o pedido. Daqui em diante, a ausência do `VecShape`
        // significa "o artista congelou", e ninguém a ressuscita.
        tool.clear_pending_live();
    }
}

/// "Convert to Curves" de formas paramétricas (NÃO-texto): descarta o `VecShape` — a
/// geometria já assada vira um path cru, editável com a caneta. Devolve quantas converteu.
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
    use ph2d_vec_scene::ALL_SHAPES;

    /// **Gate de cobertura:** TODA forma do catálogo re-cozinha CENTRADA no local 0 — o
    /// pivô é o centro (ADR-0112), e é o que faz o re-cook preservar a pose. Uma forma
    /// nova que cozinhasse fora do centro faria a forma pular ao editar um parâmetro.
    #[test]
    fn every_live_shape_cooks_centered_on_the_local_origin() {
        for &k in ALL_SHAPES {
            let shape = VecShape::Param {
                kind: k.as_u16(),
                w: 4.0,
                h: 2.0,
                values: k.defaults(),
            };
            let p = recook_shape(&shape).expect("forma paramétrica cozinha");
            for v in p.verts_all() {
                assert!(
                    v.anchor[0].abs() <= 2.0 + 1e-6 && v.anchor[1].abs() <= 1.0 + 1e-6,
                    "{k:?}: âncora {:?} fora da caixa centrada +-2 x +-1",
                    v.anchor
                );
            }
        }
    }

    /// O texto não passa pelo cozimento paramétrico (tem o dele, em `vec_glyph`), e um
    /// `kind` DESCONHECIDO (save de uma versão futura) vira path cru — nunca pânico.
    #[test]
    fn text_and_unknown_kinds_do_not_cook_here() {
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

        let future = VecShape::Param {
            kind: 9_999,
            w: 1.0,
            h: 1.0,
            values: [0.0; MAX_SHAPE_VALUES],
        };
        assert!(recook_shape(&future).is_none(), "forma futura != panic");
    }
}
