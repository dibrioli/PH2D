//! Testes do [`crate::connector_live`] — módulo irmão (teto de 600 LOC por arquivo da shell).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **A linha SEGUE a forma** (`moving_a_shape_drags_the_connector_endpoint_with_it`) — a
//!    feature inteira numa asserção.
//! 2. **A ordem do frame** (`a_fresh_connector_never_enters_the_xform_map`) — o `settle` só
//!    pula o conector que ENXERGA.
//! 3. O conector nunca guarda pose, e apagar uma forma não mata a linha.

use super::*;
use ph2d_ecs::{VecPathRef, VecShape};
use ph2d_vec_scene::{VecScene, rectangle};

/// Uma cena com dois retângulos e um conector entre eles. Devolve
/// `(sim, scene, map, conn_id, [a, b])`.
#[allow(clippy::type_complexity)]
fn scene_with_connector() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [VecPathId; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // A: [0,0]-[2,1] (centro (1, 0.5)) · B: [8,0]-[10,1] (centro (9, 0.5)).
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([8.0, 0.0], [10.0, 1.0]));
    let c = scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(attach(&mut sim, &map, c, &VecConnector::between(a, b)));
    (sim, scene, map, c, [a, b])
}

fn xforms(sim: &SimWorld, map: &VecEntityMap) -> VecXforms {
    crate::vec_transform::build(sim, map)
}

/// O ponto está NA borda do retângulo de mundo `lo..hi`?
fn on_border(lo: [f64; 2], hi: [f64; 2], p: [f64; 2]) -> bool {
    const E: f64 = 1e-6;
    let inside_x = p[0] >= lo[0] - E && p[0] <= hi[0] + E;
    let inside_y = p[1] >= lo[1] - E && p[1] <= hi[1] + E;
    let on_vert = (p[0] - lo[0]).abs() < E || (p[0] - hi[0]).abs() < E;
    let on_horz = (p[1] - lo[1]).abs() < E || (p[1] - hi[1]).abs() < E;
    (on_vert && inside_y) || (on_horz && inside_x)
}

fn ends(scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    let p = scene.paths().iter().find(|p| p.id == id).expect("conector");
    assert!(p.verts.len() >= 2, "a rota nunca e vazia (funcao TOTAL)");
    (
        p.verts.first().expect("primeiro").anchor,
        p.verts.last().expect("ultimo").anchor,
    )
}

/// **O TESTE.** Move-se uma forma; a linha vai junto.
///
/// É a feature inteira numa asserção: a ponta presa à forma que se moveu se desloca
/// EXATAMENTE o mesmo delta, e a rota continua encostando nos dois contornos. Se o
/// re-cook não rodar, se ele ler a bbox local em vez da de mundo, se a saída não
/// re-escolher o lado, ou se a geometria for escrita com pose por cima — este teste fica
/// vermelho.
#[test]
fn moving_a_shape_drags_the_connector_endpoint_with_it() {
    let (mut sim, mut scene, map, conn, [a, b]) = scene_with_connector();
    let mut cache = SideCache::new();

    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let (start_before, end_before) = ends(&scene, conn);
    // A sai pela DIREITA (a caixa B está a leste), B sai pela ESQUERDA.
    assert!(
        on_border([0.0, 0.0], [2.0, 1.0], start_before),
        "a ponta A tem de encostar no contorno de A: {start_before:?}"
    );
    assert!(
        on_border([8.0, 0.0], [10.0, 1.0], end_before),
        "a ponta B tem de encostar no contorno de B: {end_before:?}"
    );

    // Move a forma B por d — o gizmo de sprite faria exatamente isto (ADR-0111).
    let d = [3.0_f32, 2.0_f32];
    let eb = Entity::from_bits(map[&b]);
    sim.world_mut()
        .get_mut::<Transform>(eb)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(d[0], d[1]);

    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let (start_after, end_after) = ends(&scene, conn);

    // 1. A ponta presa a B ANDOU COM B — o mesmo delta, exato.
    let want = [
        end_before[0] + f64::from(d[0]),
        end_before[1] + f64::from(d[1]),
    ];
    assert!(
        (end_after[0] - want[0]).abs() < 1e-6 && (end_after[1] - want[1]).abs() < 1e-6,
        "a ponta presa a B tinha de andar {d:?} com ela: {end_before:?} -> {end_after:?}"
    );
    // 2. E a rota continua ENCOSTANDO nos dois contornos (nas posições de AGORA).
    assert!(
        on_border([0.0, 0.0], [2.0, 1.0], start_after),
        "a ponta A soltou do contorno: {start_after:?}"
    );
    assert!(
        on_border([11.0, 2.0], [13.0, 3.0], end_after),
        "a ponta B soltou do contorno novo de B: {end_after:?}"
    );
    // 3. A ponta presa a A não se mexeu (A não se mexeu).
    assert!(
        (start_after[0] - start_before[0]).abs() < 1e-6
            && (start_after[1] - start_before[1]).abs() < 1e-6,
        "A parada, ponta parada: {start_before:?} -> {start_after:?}"
    );
    // 4. O conector NUNCA ganha pose — a geometria dele é mundo (detalhe 3).
    assert_eq!(
        sim.world()
            .get::<Transform>(Entity::from_bits(map[&conn]))
            .copied(),
        Some(Transform::IDENTITY)
    );
    // E a forma A segue no lugar (o re-cook não toca em quem não é conector).
    assert_eq!(scene.path_curve_bbox(a).map(|(lo, _)| lo), Some([0.0, 0.0]));
}

/// Um conector arrastado pelo gizmo **volta para a identidade**: a pose não quer dizer
/// nada nele, e somá-la à geometria de mundo deslocaria a rota do lugar.
#[test]
fn a_connector_never_keeps_a_pose() {
    let (mut sim, mut scene, map, conn, _) = scene_with_connector();
    let mut cache = SideCache::new();
    let e = Entity::from_bits(map[&conn]);
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(5.0, 5.0);

    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);

    assert_eq!(
        sim.world().get::<Transform>(e).copied(),
        Some(Transform::IDENTITY),
        "o gizmo mexeu no conector; o re-cook desfez — e por isso ele nao e arrastavel"
    );
    // A rota continua entre as duas formas (não foi para (5,5)).
    let (s, t) = ends(&scene, conn);
    assert!(on_border([0.0, 0.0], [2.0, 1.0], s), "{s:?}");
    assert!(on_border([8.0, 0.0], [10.0, 1.0], t), "{t:?}");
}

/// **Apagar uma forma não mata a linha.** A ponta órfã CONGELA onde estava (o
/// `freeze_missing` do componente, wirado aqui) — e a linha continua na tela, largada.
#[test]
fn deleting_a_shape_freezes_the_orphaned_end_and_the_line_stays() {
    let (mut sim, mut scene, mut map, conn, [_, b]) = scene_with_connector();
    let mut cache = SideCache::new();
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let (_, end_before) = ends(&scene, conn);

    // A Hierarquia apaga B ⇒ o `sync` leva o path junto.
    sim.world_mut().despawn(Entity::from_bits(map[&b]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(!scene.paths().iter().any(|p| p.id == b), "B sumiu");

    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);

    let c = sim
        .world()
        .get::<VecConnector>(Entity::from_bits(map[&conn]))
        .expect("o conector sobreviveu");
    assert_eq!(
        c.end,
        ConnectorEnd::Free { at: end_before },
        "a ponta orfa tem de CONGELAR onde estava"
    );
    let (_, end_after) = ends(&scene, conn);
    assert!(
        (end_after[0] - end_before[0]).abs() < 1e-6,
        "a linha continua na tela, largada: {end_after:?}"
    );
}

/// **A ordem do frame é load-bearing.** Roda a sequência EXATA do `render_loop` para um
/// conector recém-criado (o gesto empurrou o path e deixou o componente pendente):
///
/// ```text
/// vec_entities::sync → connector_live::upkeep → vec_transform::settle_origins
///                    → vec_transform::build   → connector_live::recook
/// ```
///
/// O `settle` PULA conectores — mas só pula o que ENXERGA. Com o `upkeep` depois dele, a
/// linha recém-empurrada seria assentada como um path comum (origem no centro dela,
/// geometria recuada), e o afim resultante entraria no `VecXforms` do frame — que é o que
/// o render usa. A geometria dela já é MUNDO ⇒ ela apareceria **deslocada em dobro**.
///
/// A asserção é a que pega isso: um conector **nunca entra no mapa de afins**.
#[test]
fn a_fresh_connector_never_enters_the_xform_map() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([8.0, 0.0], [10.0, 1.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    // O gesto: empurra a linha (longe da origem, como qualquer rota) e deixa o
    // componente pendente para a entidade que ainda vai nascer.
    let c = scene.push_path(ph2d_vec_scene::line([2.0, 0.5], [8.0, 0.5]));
    let mut pending = Some((c, VecConnector::between(a, b)));
    let mut cache = SideCache::new();

    // ── o frame, na ordem do `render_loop` ──
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, None, &mut pending);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);

    assert!(pending.is_none(), "a entidade nasceu: a fila esvaziou");
    assert_eq!(
        ph2d_vec_scene::xform_of(&xf, c),
        ph2d_vec_scene::Xform::IDENTITY,
        "o conector foi ASSENTADO antes de o `settle` poder enxerga-lo — o afim dele \
         entrou no mapa e o render o desenharia deslocado em dobro"
    );
    // E a rota saiu certa: encostando nas duas formas, onde elas estão.
    let (s, t) = ends(&scene, c);
    assert!(on_border([0.0, 0.0], [2.0, 1.0], s), "{s:?}");
    assert!(on_border([8.0, 0.0], [10.0, 1.0], t), "{t:?}");
}

/// Um conector é uma entidade como outra qualquer — mas **não é uma forma viva**: o
/// `VecShape` não entra nele, e o re-cook dele é este módulo, não o `vec_shape_live`.
#[test]
fn a_connector_is_not_a_live_shape() {
    let (sim, _, map, conn, _) = scene_with_connector();
    let e = Entity::from_bits(map[&conn]);
    assert!(sim.world().get::<VecShape>(e).is_none());
    assert!(sim.world().get::<VecPathRef>(e).is_some());
    assert!(sim.world().get::<VecConnector>(e).is_some());
}
