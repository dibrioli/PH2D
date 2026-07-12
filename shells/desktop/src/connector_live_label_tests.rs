//! **O rótulo NÃO é alvo de uma ponta** — módulo filho do [`super`] (teto de 600 LOC por arquivo da
//! shell). Herda `scene_with_connector`, `on_border` e `ends` pelo `use super::*`.
//!
//! A segunda aresta do laço rótulo↔rota. A primeira (o rótulo como PAREDE) vive em
//! `label_live_tests::loops`; esta é a que o gesto abre: o `shape_under_cursor` pega a forma do TOPO
//! sob o cursor e filtra os conectores — **mas não os rótulos**. E um rótulo nasce CENTRADO no
//! hospedeiro: ele cobre a caixa em que o usuário está mirando.

use super::*;

/// **A SEGUNDA aresta do laço: uma ponta presa no PRÓPRIO rótulo.** E esta não oscila — ela FOGE.
///
/// O gesto que prende a ponta (`connector_gesture::shape_under_cursor`) pega a forma do TOPO sob o
/// cursor e filtra os **conectores** — mas não os rótulos. E um rótulo nasce **centrado no
/// hospedeiro**: ele cobre exatamente a caixa em que o usuário está mirando. Prender a linha ao
/// TEXTO em vez de à FORMA é o caminho comum, não o exótico.
///
/// Quando o texto em que a ponta se prendeu é o rótulo do **próprio** conector, o laço fecha: a
/// rota passa a depender da bbox do rótulo, e o rótulo se põe no meio da rota. Medido antes do
/// fix, frame a frame, a ponta corria para o meio e o comprimento **caía pela metade a cada
/// frame** — 4.0, 2.0, 1.0, 0.5, 0.25, … para sempre. A linha e o texto pulando sem parar.
///
/// O `connector_live::end_box` resolve o alvo pelo `walls::anchor_target`: um rótulo não é objeto a
/// que se prender — a linha se prende ao que ele ROTULA. Aqui o hospedeiro é o próprio conector
/// (ligar linha em linha realimentaria a rota), então não sobra alvo: a ponta **congela** onde
/// estava, pelo mesmo caminho de um alvo apagado.
///
/// O gate é o do período: rota e pose **param**, e param no 1º frame.
#[test]
fn an_end_bound_to_its_own_label_freezes_instead_of_chasing_it() {
    use ph2d_ecs::{VecLabel, VecTextParams};

    let (mut sim, mut scene, mut map, conn, [_, b]) = scene_with_connector();
    let mut cache = SideCache::new();
    let frame = |sim: &mut SimWorld, scene: &mut VecScene, map: &mut VecEntityMap, c: &mut _| {
        crate::vec_entities::sync(sim, scene, map);
        crate::vec_transform::settle_origins(sim, scene, map, &[]);
        let xf = crate::vec_transform::build(sim, map);
        recook(sim, scene, map, &xf, c);
    };
    frame(&mut sim, &mut scene, &mut map, &mut cache);

    // O rótulo do conector: um texto em cima da rota, vinculado a ela.
    let label = scene.push_path(rectangle([4.0, 0.2], [6.0, 0.8]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let le = Entity::from_bits(map[&label]);
    sim.world_mut().entity_mut(le).insert((
        VecShape::Text(VecTextParams {
            text: "L".to_owned(),
            origin: [0.0, 0.0],
            family: None,
            size: 1.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: 0,
            axes: Vec::new(),
        }),
        VecLabel::on(conn),
    ));

    // ... e o gesto religa a ponta B **no rótulo** (era o texto que estava sob o cursor).
    let ce = Entity::from_bits(map[&conn]);
    if let Some(mut c) = sim.world_mut().get_mut::<VecConnector>(ce) {
        c.end = ConnectorEnd::Bound {
            target: label,
            anchor: Anchor::Floating,
        };
    }

    let mut seen: Vec<([f64; 2], [f64; 2])> = Vec::new();
    for f in 0..32 {
        frame(&mut sim, &mut scene, &mut map, &mut cache);
        let e = ends(&scene, conn);
        assert!(
            e.1[0].is_finite() && e.1[1].is_finite(),
            "frame {f}: a ponta fugiu para o infinito"
        );
        seen.push(e);
    }
    for f in 1..seen.len() {
        assert_eq!(
            seen[f],
            seen[f - 1],
            "frame {f}: a ponta continua PERSEGUINDO o proprio rotulo — o laco nao foi cortado \
             (comprimento caindo pela metade a cada frame)"
        );
    }
    // E a ponta congelou ONDE ESTAVA — encostada em B, não em cima do texto. A bbox tem de ser a
    // de MUNDO: toda forma assentada tem a bbox LOCAL centrada em 0 (ADR-0112), e comparar ali
    // diria que a ponta está fora de uma caixa que na verdade nem é a de B.
    let xf = crate::vec_transform::build(&sim, &map);
    let (lo, hi) = scene.path_world_curve_bbox(&xf, b).expect("a bbox de B");
    let (_, end) = seen[0];
    assert!(
        on_border(lo, hi, end),
        "a ponta tinha de CONGELAR na borda de B ({lo:?}..{hi:?}), nao voar para o rotulo: {end:?}"
    );
}

/// **Uma ponta largada no RÓTULO de uma forma se prende à FORMA.** O caso comum da mesma aresta.
///
/// Um rótulo nasce CENTRADO no hospedeiro — ele cobre a caixa. Mirar no meio de uma caixa rotulada
/// e soltar a linha ali é o gesto normal, e o `shape_under_cursor` devolve o TEXTO (ele filtra
/// conectores, não rótulos). Sem a resolução no `end_box`, a linha encosta na legenda em vez da
/// caixa: a seta aponta para o meio do texto, e a rota some por baixo dele.
///
/// Aqui o hospedeiro do rótulo está VIVO, então o `alive`/`freeze` não entra — quem tem de
/// consertar é o `end_box`, e é ele que este gate mede.
#[test]
fn an_end_dropped_on_a_shapes_label_anchors_to_the_shape() {
    use ph2d_ecs::{VecLabel, VecTextParams};

    let (mut sim, mut scene, mut map, conn, [_, b]) = scene_with_connector();
    let mut cache = SideCache::new();

    // O rótulo de B: um texto pequeno, centrado nela (é onde ele nasce).
    let label = scene.push_path(rectangle([8.6, 0.3], [9.4, 0.7]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let le = Entity::from_bits(map[&label]);
    sim.world_mut().entity_mut(le).insert((
        VecShape::Text(VecTextParams {
            text: "B".to_owned(),
            origin: [0.0, 0.0],
            family: None,
            size: 1.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: 0,
            axes: Vec::new(),
        }),
        VecLabel::on(b),
    ));
    // O gesto mirou no meio de B e pegou o TEXTO dela.
    let ce = Entity::from_bits(map[&conn]);
    if let Some(mut c) = sim.world_mut().get_mut::<VecConnector>(ce) {
        c.end = ConnectorEnd::Bound {
            target: label,
            anchor: Anchor::Floating,
        };
    }

    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);

    let (lo, hi) = scene.path_world_curve_bbox(&xf, b).expect("a bbox de B");
    let (_, end) = ends(&scene, conn);
    assert!(
        on_border(lo, hi, end),
        "a linha tem de encostar na FORMA ({lo:?}..{hi:?}), nao na legenda dela: {end:?}"
    );
    // E o vínculo NÃO foi reescrito: quem resolve é a leitura, não o documento — um save antigo
    // (ou um undo) traz o alvo velho de volta, e ele tem de continuar resolvendo.
    let c = sim
        .world()
        .get::<VecConnector>(ce)
        .expect("o conector vivo");
    assert_eq!(
        c.end,
        ConnectorEnd::Bound {
            target: label,
            anchor: Anchor::Floating
        },
        "o alvo gravado fica como esta; a resolucao e do end_box, a cada leitura"
    );
}
