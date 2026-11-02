use super::*;

fn cena() -> (VecScene, SimWorld, VecEntityMap, VecXforms, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    let xf = ph2d_vec_entities::transform::build(&sim, &map);
    (scene, sim, map, xf, id)
}

/// ⭐ **Arrastar a alça escreve a distância; a alça segue-a.** Para fora cresce, para dentro
/// encolhe, de volta à borda desarma. *Mutação: `d = edge − world` ⇒ vermelho.*
#[test]
fn dragging_the_handle_grows_and_shrinks() {
    let (scene, mut sim, map, xf, id) = cena();
    let (edge, at) = handle(&sim, &scene, &map, &xf, id).expect("a alça");
    assert_eq!(
        edge,
        [1.0, -0.5],
        "a um quarto: a meia altura é da pega de escala"
    );
    assert_eq!(at, edge, "sem offset a alça está na borda");
    let g = press_at(&sim, &scene, &map, &xf, id, [1.05, -0.5], 0.1).expect("agarra");
    drag(&mut sim, &map, g, [1.5, 0.3]);
    let d = crate::offset_live::spec_of(&sim, &map, id)
        .expect("armou")
        .d;
    assert!(
        (d - 0.5).abs() < 1e-12,
        "cresce o que o dedo passou da borda: {d}"
    );
    assert_eq!(
        handle(&sim, &scene, &map, &xf, id).map(|h| h.1),
        Some([1.5, -0.5])
    );
    drag(&mut sim, &map, g, [0.7, 0.0]);
    assert!(crate::offset_live::spec_of(&sim, &map, id).expect("vivo").d < 0.0);
    drag(&mut sim, &map, g, [1.0, 0.0]);
    assert!(
        crate::offset_live::spec_of(&sim, &map, id).is_none(),
        "na borda desarma"
    );
}

/// ⛔ CONTROLO: um press longe da alça não a agarra.
#[test]
fn a_press_away_from_the_handle_does_not_grab() {
    let (scene, sim, map, xf, id) = cena();
    assert!(press_at(&sim, &scene, &map, &xf, id, [0.0, 0.0], 0.1).is_none());
}
