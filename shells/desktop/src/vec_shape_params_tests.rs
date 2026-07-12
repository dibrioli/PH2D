//! Testes de [`super`] (`vec_shape_params.rs`) — o alvo do painel, o campo genérico, a
//! fronteira de unidade (px ↔ mundo) e o re-cook in-place. Extraídos para um módulo irmão
//! (`#[path]`) sob o teto de LOC da shell (HR-18).
//!
//! Os testes são **sobre o catálogo**, não sobre formas nomeadas: uma forma nova entra na
//! tabela e já nasce coberta por eles.

use super::*;
use crate::vec_shape_live::recook_shape;
use ph2d_ecs::Transform;
use ph2d_tool_vector::shapes::FieldUnit;
use ph2d_vec_scene::ALL_SHAPES;

/// Uma forma viva no mundo: path na cena + entidade + `VecShape`, com uma POSE (o "move
/// do usuário") para provar que o re-cook não a perde.
fn live_shape(
    kind: ShapeKind,
    values: ShapeValues,
    pose: [f32; 2],
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let shape = VecShape::Param {
        kind: kind.as_u16(),
        w: 4.0,
        h: 4.0,
        values,
    };
    let id = scene.push_path(recook_shape(&shape).expect("forma paramétrica cozinha"));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let entity = Entity::from_bits(*map.get(&id).expect("a entidade do path"));
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(pose[0], pose[1]);
        }
        e.insert(shape);
    }
    (sim, scene, map, id)
}

fn anchors(scene: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o path")
        .verts_all()
        .map(|v| v.anchor)
        .collect()
}

fn field(i: usize) -> NodeId {
    ph2d_editor::ids::vector_shape_field_id(i)
}

/// O CORAÇÃO do ciclo paramétrico: mexer num campo de um polígono VIVO selecionado
/// re-cozinha a forma **no lugar** — a contagem de âncoras muda, mas o id do path, a pose
/// (`Transform`) e o centro da geometria ficam. Sem isso, "live shape" não existe: um
/// polígono de 5 lados nunca viraria de 7.
#[test]
fn a_shape_field_recooks_the_selected_live_shape_in_place() {
    let pose = [12.0, -7.0];
    let (mut sim, mut scene, map, id) =
        live_shape(ShapeKind::Polygon, ShapeKind::Polygon.defaults(), pose);
    let before = anchors(&scene, id).len();

    let edited = edit_selected_shape(&mut sim, &mut scene, &map, &[id], |k, v| {
        apply_shape_field(k, v, field(0), 7.0, 1.0) // 7 lados
    });

    assert!(edited, "havia forma viva selecionada");
    let (_, _, kind, values) =
        panel_shape_target(&sim, &map, &[id]).expect("segue sendo forma viva");
    assert_eq!(kind, ShapeKind::Polygon);
    assert!(
        (values[0] - 7.0).abs() < 1e-9,
        "o parâmetro foi para a entidade"
    );
    let after = anchors(&scene, id);
    assert_ne!(
        after.len(),
        before,
        "a geometria re-cozinhou (7 != 5 ancoras)"
    );
    assert_eq!(
        scene.paths().len(),
        1,
        "re-cook IN-PLACE: nada de path novo"
    );
    // A pose é do `Transform` — o re-cook nunca a toca (senão a forma pularia).
    let e = Entity::from_bits(*map.get(&id).expect("entidade"));
    let t = sim.world().get::<Transform>(e).expect("Transform");
    assert!(
        (t.translation.x - pose[0]).abs() < 1e-6 && (t.translation.y - pose[1]).abs() < 1e-6,
        "a pose (o move do usuario) sobreviveu ao re-cook"
    );
}

/// Um campo que não existe na forma selecionada não a toca (o painel nem o desenha, mas o
/// caminho recusa por construção).
#[test]
fn a_field_that_the_shape_does_not_have_leaves_it_alone() {
    // O retângulo não declara parâmetro nenhum: TODO campo tem de ser recusado.
    let (mut sim, mut scene, map, id) = live_shape(
        ShapeKind::Rectangle,
        ShapeKind::Rectangle.defaults(),
        [0.0, 0.0],
    );
    let before = anchors(&scene, id);
    let edited = edit_selected_shape(&mut sim, &mut scene, &map, &[id], |k, v| {
        apply_shape_field(k, v, field(0), 9.0, 1.0)
    });
    assert!(!edited, "o retangulo nao tem campo 0");
    assert_eq!(anchors(&scene, id), before, "a geometria nao foi re-cozida");
}

/// **Gate anti-campo-morto:** para TODA forma do catálogo, cada campo que ela declara move
/// de fato a geometria dela (dois valores distintos ⇒ formas distintas). Um campo novo que
/// o `cook` ignore fica editável, verde no CI, e morto na tela.
#[test]
fn every_declared_field_of_every_shape_moves_its_geometry() {
    for &k in ALL_SHAPES {
        let d = ph2d_tool_vector::shapes::desc(k);
        for (i, f) in d.fields.iter().enumerate() {
            let mut lo = k.defaults();
            let mut hi = k.defaults();
            assert!(
                apply_shape_field(k, &mut lo, field(i), f.min, 1.0)
                    && apply_shape_field(k, &mut hi, field(i), f.max, 1.0),
                "{k:?}.{}: o campo nao foi aceito",
                f.label
            );
            // A geometria são as âncoras E os handles: um parâmetro que só encurva (o
            // bico da gota, a cintura da chave) mexe nos handles sem mover uma âncora —
            // comparar só âncoras acusaria um campo vivo de estar morto.
            let cook = |v: ShapeValues| {
                let s = VecShape::Param {
                    kind: k.as_u16(),
                    w: 4.0,
                    h: 4.0,
                    values: v,
                };
                let p = recook_shape(&s).expect("cozinha");
                p.verts_all()
                    .map(|x| (x.anchor, x.in_handle, x.out_handle))
                    .collect::<Vec<_>>()
            };
            assert_ne!(
                cook(lo),
                cook(hi),
                "{k:?}.{}: os extremos do campo dao a MESMA geometria — o parametro nao faz nada",
                f.label
            );
        }
    }
}

/// A fronteira de unidade fecha para TODO campo `Px` de TODA forma: a caixa do painel fala
/// **pixels** (é o que o usuário digita — a unidade de mundo é pequena demais: a viewport
/// inteira tem ~10 unidades), a forma guarda **mundo**, e o ida-e-volta não pode mover o
/// número (senão ele saltaria de escala a cada clique).
#[test]
fn every_px_field_round_trips_across_the_unit_boundary() {
    const PTW: f64 = 0.01; // ~1000 px de tela = 10 unidades de mundo
    for &k in ALL_SHAPES {
        let d = ph2d_tool_vector::shapes::desc(k);
        for (i, f) in d.fields.iter().enumerate() {
            if f.unit != FieldUnit::Px {
                continue;
            }
            let mut world = k.defaults();
            assert!(apply_shape_field(k, &mut world, field(i), 30.0, PTW));
            assert!(
                (world[i] - 30.0 * PTW).abs() < 1e-9,
                "{k:?}.{}: guardado em MUNDO (px x px_to_world)",
                f.label
            );
            let ui = ui_values_of(k, &world, PTW);
            assert!(
                (ui[i] - 30.0).abs() < 1e-9,
                "{k:?}.{}: voltou a 30 px",
                f.label
            );
        }
    }
}

/// O ALVO dos campos de forma pula o TEXTO (que tem a seção própria) e ignora path cru.
#[test]
fn the_panel_target_is_the_live_parametric_shape_only() {
    let (sim, _scene, map, id) = live_shape(
        ShapeKind::Ellipse,
        ShapeKind::Ellipse.defaults(),
        [0.0, 0.0],
    );
    assert!(
        panel_shape_target(&sim, &map, &[id]).is_some(),
        "uma elipse viva e alvo"
    );
    assert!(
        panel_shape_target(&sim, &map, &[]).is_none(),
        "sem selecao, sem alvo"
    );
}
