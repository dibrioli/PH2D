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

/// A menor distancia entre duas polilinhas (amostrada). E a medida do que o olho ve: duas
/// linhas que se tocam sao, para o usuario, UMA linha.
fn min_gap(p: &[[f64; 2]], q: &[[f64; 2]]) -> f64 {
    let sample = |v: &[[f64; 2]]| -> Vec<[f64; 2]> {
        v.windows(2)
            .flat_map(|w| {
                (0..=40).map(move |k| {
                    let t = f64::from(k) / 40.0;
                    [
                        w[0][0] + (w[1][0] - w[0][0]) * t,
                        w[0][1] + (w[1][1] - w[0][1]) * t,
                    ]
                })
            })
            .collect()
    };
    let (sp, sq) = (sample(p), sample(q));
    sp.iter()
        .flat_map(|a| sq.iter().map(move |b| (a[0] - b[0]).hypot(a[1] - b[1])))
        .fold(f64::INFINITY, f64::min)
}

fn poly(scene: &VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("conector")
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect()
}

/// **O desvio de obstaculo, no produto.** Uma terceira forma plantada entre as duas caixas
/// ligadas: a linha tem de contorna-la.
///
/// O teste PROVA que morde: primeiro roda a cena SEM a parede e verifica que a rota passa
/// exatamente por onde a parede vai ficar (ela e uma reta de A a B). So entao planta a parede e
/// exige o desvio. Sem essa primeira metade, o verde do final nao distinguiria "a shell
/// alimenta os obstaculos" de "a rota nunca passou por ali, de sorte".
#[test]
fn a_shape_planted_between_two_boxes_pushes_the_line_around_it() {
    let (mut sim, mut scene, mut map, conn, _) = scene_with_connector();
    let mut cache = SideCache::new();

    // Sem a parede: a rota e a reta de A a B, na altura y = 0.5.
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let straight = poly(&scene, conn);

    // Onde a parede VAI ficar. A reta a atravessa — e e isso que da sentido ao teste.
    let (lo, hi) = ([4.0, -2.0], [6.0, 3.0]);
    let inside = |p: [f64; 2]| p[0] > lo[0] && p[0] < hi[0] && p[1] > lo[1] && p[1] < hi[1];
    let crossed = straight.windows(2).any(|w| {
        (1..40).any(|k| {
            let t = f64::from(k) / 40.0;
            inside([
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ])
        })
    });
    assert!(
        crossed,
        "teste VACUO: a rota ja nao passava por onde a parede vai entrar ({straight:?}), entao \
         o desvio abaixo nao provaria nada"
    );

    // Planta a parede e re-cozinha.
    scene.push_path(rectangle(lo, hi));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    let around = poly(&scene, conn);

    for w in around.windows(2) {
        for k in 0..=40 {
            let t = f64::from(k) / 40.0;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ];
            assert!(
                !inside(p),
                "a linha ATRAVESSOU a forma plantada em {p:?} — a shell nao esta alimentando o \
                 roteador com as outras formas: {around:?}"
            );
        }
    }
    assert!(
        around.len() > 2,
        "a linha tinha de DOBRAR para contornar: {around:?}"
    );
}

/// **Dois conectores no mesmo par de formas nunca se sobrepoem** — e este e o gate que a
/// primeira versao do spread nao tinha, e por isso ela era um placebo.
///
/// As caixas aqui estao EMPILHADAS de proposito. Era o caso que expunha o bug: a rota entre
/// elas tem quatro pontos (`p0, s0, s1, p1`), logo nao tem vertice "do meio" nenhum — e o
/// deslocamento antigo, que mexia so nos vertices do meio, nao mexia em nada. Os dois
/// conectores saiam identicos, um exatamente por baixo do outro, e o usuario juraria que o
/// segundo nao foi criado.
///
/// A separacao acontece no PONTO DE SAIDA: as duas linhas encostam em pontos diferentes da
/// face. Este teste mede o que o olho ve — a menor distancia entre as duas polilinhas.
#[test]
fn two_connectors_between_the_same_pair_never_overlap() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Empilhadas: A embaixo, B em cima. A rota direta e uma reta vertical.
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let b = scene.push_path(rectangle([0.0, 4.0], [2.0, 5.0]));
    let c0 = scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0]));
    let c1 = scene.push_path(ph2d_vec_scene::line([0.0, 0.0], [0.0, 0.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    let first = VecConnector::between(a, b);
    let mut second = VecConnector::between(a, b);
    second.parallel_index = 1; // o gesto o numera assim ao criar o segundo do par
    assert!(attach(&mut sim, &map, c0, &first));
    assert!(attach(&mut sim, &map, c1, &second));

    let mut cache = SideCache::new();
    let xf = xforms(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut cache);

    let (p0, p1) = (poly(&scene, c0), poly(&scene, c1));
    let gap = min_gap(&p0, &p1);
    assert!(
        gap > 0.1,
        "os dois conectores se SOBREPOEM (distancia minima {gap:.4}): o segundo desaparece por \
         baixo do primeiro e o usuario jura que ele nao foi criado.\n  1o: {p0:?}\n  2o: {p1:?}"
    );
    // E as duas continuam encostando nas duas formas — separar nao pode soltar a linha.
    for (id, p) in [(c0, &p0), (c1, &p1)] {
        let (s, t) = ends(&scene, id);
        assert!(
            on_border([0.0, 0.0], [2.0, 1.0], s),
            "{p:?} soltou de A: {s:?}"
        );
        assert!(
            on_border([0.0, 4.0], [2.0, 5.0], t),
            "{p:?} soltou de B: {t:?}"
        );
    }
}

/// O valor FIXADO no painel vence o automatico — e continua valendo no frame seguinte (o
/// re-cook nao pode re-derivar por cima do que o usuario escolheu).
#[test]
fn a_pinned_jetty_survives_the_recook() {
    let (mut sim, mut scene, map, conn, _) = scene_with_connector();
    let mut cache = SideCache::new();
    let e = Entity::from_bits(map[&conn]);

    let mut c = VecConnector::between(scene.paths()[0].id, scene.paths()[1].id);
    c.jetty = Some(2.5);
    assert!(attach(&mut sim, &map, conn, &c));

    for _ in 0..3 {
        let xf = xforms(&sim, &map);
        recook(&mut sim, &mut scene, &map, &xf, &mut cache);
    }
    assert_eq!(
        sim.world().get::<VecConnector>(e).expect("conector").jetty,
        Some(2.5),
        "o re-cook re-derivou o jetty por cima do valor que o usuario fixou"
    );
}
