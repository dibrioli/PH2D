use super::*;
use ph2d_ecs::VecPathRef;

/// O palco da `=36` sem a janela: o retângulo (livre, sem a ferramenta), a forma composta, uma
/// entidade por caminho e a Timeline.
fn palco() -> (VecScene, SimWorld, TimelineState, VecPathId, u64, u64) {
    let mut scene = VecScene::new();
    let camadas = scene.push_path(VecPath {
        verts: quadrado(CAMADAS_X, 0.0, LADO),
        closed: true,
        ..VecPath::default()
    });
    let compostos = scene.push_path(contornos());
    let mut sim = SimWorld::new();
    let a = sim.world_mut().spawn(VecPathRef(camadas)).id().to_bits();
    let b = sim.world_mut().spawn(VecPathRef(compostos)).id().to_bits();
    (scene, sim, TimelineState::default(), camadas, a, b)
}

fn n_chaves(tl: &TimelineState, bits: u64, kk: FieldKey) -> usize {
    tl.doc
        .binding_for(bits, PropKind::Field(kk))
        .and_then(|b| tl.doc.active_clip().track(b.target))
        .map_or(0, |t| t.keys().len())
}

/// ⭐ A `=36` monta o que o smoke ensina: as chaves vão pelo ID nas três listas (duas tintas, dois
/// filtros) e por contorno nos dois contornos; apagar a tinta e o filtro de baixo deixa os vizinhos
/// a responder pela MESMA chave.
#[test]
fn a_cena_36_chaveia_as_camadas_pelo_id_e_os_dois_contornos() {
    let (mut scene, mut sim, mut tl, camadas, a, b) = palco();
    let compostos = scene.paths()[1].id;
    let log = montar(&mut scene, &mut sim, &mut tl, (camadas, a), (compostos, b));
    assert!(log.contains("=36"), "{log}");

    let p = scene.path(camadas).unwrap();
    let (fill, stroke) = (p.paints[0].id, p.paints[1].id);
    assert!(!fill.is_none() && !stroke.is_none() && fill != stroke);
    let ent = Entity::from_bits(a);
    let ids = sim.world().get::<VecFilter>(ent).unwrap().ids().to_vec();
    assert!(ids.len() == 2 && ids.iter().all(|i| !i.is_none()) && ids[0] != ids[1]);

    let verde = FieldKey::do_item(VETOR, c::TINTA_PREENCH + c::OFF_SOLIDA + 1, fill, 0);
    let largura = FieldKey::do_item(VETOR, c::TRACO_LARGURA, stroke, 0);
    let raio = FieldKey::do_item(VETOR, c::FILTRO_RAIO, ids[0], 0);
    let longe = FieldKey::do_item(VETOR, c::FILTRO_DESLOCA_X, ids[1], 0);
    for kk in [verde, largura, raio, longe] {
        assert_eq!(n_chaves(&tl, a, kk), 3, "{kk:?}");
    }
    let todos = crate::canais_campos::campos(p, None);
    assert!(todos.contains(&verde) && todos.contains(&largura));
    let filtros = crate::canais_filtros::campos(sim.world().get::<VecFilter>(ent).unwrap());
    assert!(filtros.contains(&raio) && filtros.contains(&longe));
    for (i, j) in [(0, 0), (0, 3), (1, 0), (1, 3)] {
        let kk = FieldKey::new(VETOR, c::ANCORA_Y, i, j);
        assert_eq!(n_chaves(&tl, b, kk), 3, "contorno {i} ponto {j}");
    }
    assert_eq!(n_chaves(&tl, b, FieldKey::new(VETOR, c::ANCORA_X, 1, 2)), 3);
    assert_eq!(scene.path(scene.paths()[1].id).unwrap().contour_count(), 2);

    // O gesto do dono: o lixo da 1.ª tinta e o ✕ do 1.º filtro.
    let largura_antes = crate::canais_campos::ler(p, None, largura);
    crate::paint_stack::remove(&mut scene, &[camadas], 0);
    let p = scene.path(camadas).unwrap();
    assert_eq!(crate::canais_campos::ler(p, None, largura), largura_antes);
    assert_eq!(crate::canais_campos::ler(p, None, verde), None);
    let mut f = sim.world_mut().get_mut::<VecFilter>(ent).unwrap();
    let longe_antes = crate::canais_filtros::ler(&f, longe);
    f.remove(0);
    assert_eq!(crate::canais_filtros::ler(&f, longe), longe_antes);
    assert_eq!(crate::canais_filtros::ler(&f, raio), None);
}
