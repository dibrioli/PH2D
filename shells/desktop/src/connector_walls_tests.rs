//! Testes do [`super`] — as duas arestas do laço rótulo↔rota, cortadas.
//!
//! O laço inteiro (a linha e o texto pulando sem parar) vive nos gates de integração — que rodam a
//! ordem REAL do frame, em `label_live_tests` e `connector_live_tests`. Aqui ficam as regras
//! isoladas: *quem é anotação* e *a que uma ponta pode se prender*.

use super::*;
use ph2d_ecs::{Transform, VecPathRef, VecTextParams};
use ph2d_vec_scene::rectangle;

fn text_params() -> VecTextParams {
    VecTextParams {
        text: "L".to_owned(),
        origin: [0.0, 0.0],
        family: None,
        size: 1.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: 0,
        axes: Vec::new(),
    }
}

/// Um documento com uma forma, um conector, um texto SOLTO e um RÓTULO da forma.
fn doc() -> (SimWorld, VecScene, VecEntityMap, [VecPathId; 4]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let shape = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let conn = scene.push_path(ph2d_vec_scene::line([2.0, 0.5], [8.0, 0.5]));
    let loose = scene.push_path(rectangle([4.0, 0.2], [6.0, 0.8]));
    let label = scene.push_path(rectangle([0.5, 0.2], [1.5, 0.8]));
    for (i, id) in [shape, conn, loose, label].into_iter().enumerate() {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());
        // O conector é um conector; os dois textos são `VecShape::Text`; só o rótulo tem vínculo.
        if i == 1 {
            sim.world_mut()
                .entity_mut(e)
                .insert(VecConnector::between(shape, 99));
        }
        if i >= 2 {
            sim.world_mut()
                .entity_mut(e)
                .insert(VecShape::Text(text_params()));
        }
        if i == 3 {
            sim.world_mut().entity_mut(e).insert(VecLabel::on(shape));
        }
    }
    (sim, scene, map, [shape, conn, loose, label])
}

/// **Anotação = conector + TEXTO** (com vínculo ou sem). O texto SOLTO é o caso que o `VecLabel`
/// sozinho não pegava — e ele existe: é o texto que o usuário criou com a ferramenta T e largou em
/// cima da linha. Uma forma comum continua sendo parede.
#[test]
fn text_is_annotation_whether_or_not_it_is_a_label() {
    let (sim, _, map, [shape, conn, loose, label]) = doc();
    assert!(!is_annotation(&sim, &map, shape), "uma FORMA e estrutura");
    assert!(is_annotation(&sim, &map, conn), "conector nao e parede");
    assert!(
        is_annotation(&sim, &map, loose),
        "texto SOLTO tambem nao e parede — anotacao nao e estrutura"
    );
    assert!(is_annotation(&sim, &map, label), "rotulo, idem");
}

/// **O rótulo não entra nas paredes; a forma entra.** É a aresta "obstáculo" do laço, isolada.
#[test]
fn the_walls_of_the_diagram_are_the_shapes_and_only_the_shapes() {
    let (sim, scene, map, [shape, ..]) = doc();
    let xf = VecXforms::new();
    let boxes = shape_boxes(&sim, &scene, &xf, &map);
    assert_eq!(boxes.len(), 1, "so a FORMA e parede: {boxes:?}");
    let (lo, hi) = scene.path_world_curve_bbox(&xf, shape).expect("bbox");
    assert!((boxes[0].min[0] - lo[0]).abs() < 1e-9 && (boxes[0].max[1] - hi[1]).abs() < 1e-9);
}

/// **Uma ponta presa num RÓTULO se prende ao que ele rotula.** É a aresta "alvo" do laço.
///
/// O gesto (`shape_under_cursor`) pega a forma do TOPO sob o cursor e filtra os conectores — mas
/// não os rótulos. E o rótulo nasce CENTRADO no hospedeiro: ele cobre a caixa em que o usuário
/// está mirando. Sem esta resolução, a linha se prende ao texto.
#[test]
fn an_end_bound_to_a_label_anchors_to_what_the_label_labels() {
    let (sim, scene, map, [shape, _, _, label]) = doc();
    assert_eq!(
        anchor_target(&sim, &scene, &map, label),
        Some(shape),
        "o alvo real e a FORMA, nao a legenda dela"
    );
    assert_eq!(
        anchor_target(&sim, &scene, &map, shape),
        Some(shape),
        "uma forma resolve para si mesma"
    );
}

/// **Anotação nua não é alvo.** Um texto solto e um conector não ancoram ponta nenhuma — e um
/// alvo apagado também não. Nos três o chamador congela a ponta (o mesmo caminho do `freeze`).
#[test]
fn a_loose_text_a_connector_and_a_dead_id_are_not_anchors() {
    let (sim, scene, map, [_, conn, loose, _]) = doc();
    assert_eq!(anchor_target(&sim, &scene, &map, loose), None);
    assert_eq!(
        anchor_target(&sim, &scene, &map, conn),
        None,
        "ligar linha em linha realimentaria a rota"
    );
    assert_eq!(anchor_target(&sim, &scene, &map, 4242), None, "id morto");
}

/// **A resolução é TOTAL.** Um ciclo de vínculos (um rótulo cujo hospedeiro é ele mesmo, ou dois
/// que se apontam) não pode travar o frame: o teto de saltos garante a parada, e o resultado é
/// "sem alvo" — que é a resposta certa, não um `loop {}`.
#[test]
fn a_cycle_of_labels_terminates_instead_of_hanging_the_frame() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    let ea = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(a)))
        .id();
    let eb = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(b)))
        .id();
    map.insert(a, ea.to_bits());
    map.insert(b, eb.to_bits());
    // Dois rótulos que se rotulam. Absurdo — e alcançável, porque o `path_at` do duplo-clique
    // pega o que está no TOPO, e o topo pode ser outro rótulo.
    sim.world_mut()
        .entity_mut(ea)
        .insert((VecShape::Text(text_params()), VecLabel::on(b)));
    sim.world_mut()
        .entity_mut(eb)
        .insert((VecShape::Text(text_params()), VecLabel::on(a)));

    assert_eq!(
        anchor_target(&sim, &scene, &map, a),
        None,
        "o ciclo termina e nao ancora nada"
    );
}
