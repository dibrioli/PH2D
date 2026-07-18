//! Os gates do [`crate::envelope_live`] — o Envelope vivo como **container** (ADR-0129, Fatia 3).
//!
//! O que eles medem é o comportamento do HOST; a **correção** da deformação (a curva sobrevive, não
//! só os pontos de controle) é da crate `ph2d-vec-envelope` e já está gateada lá. Aqui: em repouso
//! nada muda · a gaiola puxada deforma **pelo motor**, uma só gaiola para TODOS os filhos · `create`
//! reparenta os filhos na identidade sob o container · a pose vive no `Transform` do CONTAINER e não
//! vaza para a geometria dos filhos (é ela que o gizmo do Select move — Fatia 2) · a fonte autorada
//! de cada filho sobrevive · gaiola degenerada congela · assar a pose de mundo do filho na fonte.

use super::*;
use ph2d_ecs::{ChildOf, Transform, VecPathRef};
use ph2d_vec_scene::{ShapeKind, cook};

fn ellipse(c: [f64; 2], r: f64) -> VecPath {
    cook(
        ShapeKind::Ellipse,
        [c[0] - r, c[1] - r],
        [c[0] + r, c[1] + r],
        &[],
    )
}

/// Cria um envelope sobre `shapes` (formas em MUNDO/identidade), com o `sync` que dá entidade a cada
/// uma. Devolve `(sim, scene, map, container_bits, child_ids)`.
fn envelope_over(shapes: Vec<VecPath>) -> (SimWorld, VecScene, VecEntityMap, u64, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = shapes.into_iter().map(|s| scene.push_path(s)).collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let container = create(&mut sim, &mut scene, &map, &ids).expect("create");
    (sim, scene, map, container, ids)
}

/// O path da cena após um recook.
fn frame(sim: &mut SimWorld, scene: &mut VecScene, id: VecPathId) -> VecPath {
    recook(sim, scene);
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o path")
        .clone()
}

/// O `VecEnvelope` do container `bits`.
fn env_of(sim: &SimWorld, bits: u64) -> VecEnvelope {
    sim.world()
        .get::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope no container")
        .clone()
}

/// Sobrescreve os cantos do container.
fn set_corners(sim: &mut SimWorld, bits: u64, corners: [[f64; 2]; 4]) {
    sim.world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(bits))
        .expect("VecEnvelope")
        .corners = corners;
}

/// A fonte autorada de um filho (a cozida, desserializada dos bytes do componente).
fn source_of(child: &VecEnvelopeChild) -> VecPath {
    postcard::from_bytes(&child.source).expect("source")
}

/// **EM REPOUSO A FORMA NÃO MUDA.** Gaiola = cantos da bbox-união ⇒ identidade ⇒ a forma volta igual.
///
/// É a metade "presença" do par: um motor que não escrevesse nada também deixaria a forma "igual"
/// (vazia). Este teste afirma o contrário — a forma continua **cheia** e com o mesmo bbox.
#[test]
fn at_rest_the_envelope_leaves_the_child_unchanged() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (b0, b1) = super::union_control_bbox([&src]).unwrap();
    let (mut sim, mut scene, _map, _c, ids) = envelope_over(vec![src]);

    let out = frame(&mut sim, &mut scene, ids[0]);
    assert!(!out.verts.is_empty(), "a forma sumiu em repouso");
    let (o2, s2) = super::union_control_bbox([&out]).expect("bbox");
    let d = (o2[0] - b0[0]).abs()
        + (o2[1] - b0[1]).abs()
        + (s2[0] - b1[0]).abs()
        + (s2[1] - b1[1]).abs();
    assert!(
        d < 1e-6,
        "repouso deslocou o bbox em {d:.3e} (devia ser identidade)"
    );
}

/// **A GAIOLA PUXADA DEFORMA — e escreve a saída do MOTOR, não a ingênua.** A forma **mudou**, e o
/// que foi escrito é **exatamente** `warp_path(fonte, QuadWarp)` — o host liga o motor, não uma 2ª
/// porta. A correção do motor (curva sobrevive) é gateada na crate; aqui prova-se o **fio**.
#[test]
fn a_pulled_cage_deforms_the_child_through_the_engine() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![src]);
    let (origin, size) =
        super::union_control_bbox([&source_of(&env_of(&sim, container).children[0])]).unwrap();
    // Topo estreitado: trapézio convexo forte (perspectiva).
    let [bl, br, ..] = bbox_corners(origin, size);
    let pulled = [
        bl,
        br,
        [origin[0] + size[0] * 0.7, origin[1] + size[1]],
        [origin[0] + size[0] * 0.3, origin[1] + size[1]],
    ];
    assert!(QuadWarp::is_convex(&pulled), "o fixture devia ser convexo");
    set_corners(&mut sim, container, pulled);

    let source = source_of(&env_of(&sim, container).children[0]);
    let out = frame(&mut sim, &mut scene, ids[0]);
    assert_ne!(
        out.verts, source.verts,
        "a gaiola foi puxada e a forma não mudou"
    );

    let warp = QuadWarp::new(origin, size, pulled).unwrap();
    let expected = warp_path(&source, &warp, accuracy(size));
    assert_eq!(
        out.verts, expected.verts,
        "o recook não escreveu a saída do motor — 2ª porta para a mesma pergunta"
    );
}

/// **UMA gaiola deforma DOIS filhos** — o *warp group*. As duas formas passam pelo MESMO `QuadWarp`
/// sobre a bbox-UNIÃO, byte a byte; nenhuma fica em repouso. É o que separa um container de dois
/// envelopes soltos: um só mapa, um só gesto.
#[test]
fn a_two_shape_envelope_deforms_both_by_the_one_cage() {
    let (mut sim, mut scene, _map, container, _ids) =
        envelope_over(vec![ellipse([-3.0, 0.0], 1.5), ellipse([3.0, 0.0], 1.5)]);

    let env = env_of(&sim, container);
    let sources: Vec<VecPath> = env.children.iter().map(source_of).collect();
    // A bbox-união abrange as DUAS (de x≈-4.5 a x≈4.5).
    let (origin, size) = super::union_control_bbox(sources.iter()).unwrap();
    assert!(
        origin[0] < -4.0 && origin[0] + size[0] > 4.0,
        "a união devia abranger ambas"
    );

    // Puxa o topo — uma gaiola só.
    let [bl, br, ..] = bbox_corners(origin, size);
    let pulled = [
        bl,
        br,
        [origin[0] + size[0] * 0.65, origin[1] + size[1]],
        [origin[0] + size[0] * 0.35, origin[1] + size[1]],
    ];
    assert!(QuadWarp::is_convex(&pulled));
    set_corners(&mut sim, container, pulled);
    recook(&mut sim, &mut scene);

    let warp = QuadWarp::new(origin, size, pulled).unwrap();
    let acc = accuracy(size);
    for (child, src) in env.children.iter().zip(&sources) {
        let out = scene.paths().iter().find(|p| p.id == child.path).unwrap();
        let expected = warp_path(src, &warp, acc);
        assert_eq!(
            out.verts, expected.verts,
            "o filho {} não passou pela gaiola-união (ou passou por outra)",
            child.path
        );
        assert_ne!(
            out.verts, src.verts,
            "o filho {} ficou em repouso",
            child.path
        );
    }
}

/// **`create` REPARENTA os filhos na IDENTIDADE sob o container.** Estrutural: o container carrega o
/// `VecEnvelope` e NÃO tem `VecPathRef` (é grupo); cada filho é `ChildOf(container)` e está em
/// `Transform::IDENTITY` (a pose dele foi assada na fonte — passo 1 do `create`). Sem a identidade, a
/// pose do filho se aplicaria DUAS vezes (própria + herdada do container).
#[test]
fn create_reparents_children_at_identity_under_the_container() {
    let (sim, _scene, map, container, ids) =
        envelope_over(vec![ellipse([-3.0, 0.0], 1.5), ellipse([3.0, 0.0], 1.5)]);
    let ce = Entity::from_bits(container);
    assert!(
        sim.world().get::<VecEnvelope>(ce).is_some(),
        "container sem VecEnvelope"
    );
    assert!(
        sim.world().get::<VecPathRef>(ce).is_none(),
        "o container não é um path"
    );
    for id in &ids {
        let child = Entity::from_bits(map[id]);
        assert_eq!(
            sim.world().get::<ChildOf>(child).map(|c| c.parent()),
            Some(ce),
            "o filho {id} não foi reparentado sob o container"
        );
        assert_eq!(
            sim.world().get::<Transform>(child).copied(),
            Some(Transform::IDENTITY),
            "o filho {id} não ficou na identidade — a pose se aplicaria duas vezes"
        );
    }
}

/// **`create` ASSA a pose de MUNDO do filho na fonte.** Uma forma assentada (`settle`) tem a
/// geometria LOCAL centrada na origem e a pose no `Transform`. Depois do `create`, a fonte do filho
/// tem de estar em MUNDO (na posição original), não local-centrada — senão o envelope nasceria na
/// origem, longe da forma. É o que permite o filho ir para a identidade.
#[test]
fn create_bakes_the_child_world_pose_into_the_source() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Elipse longe da origem: o settle lhe dá pose ≈(40,20) e geometria local centrada em 0.
    let id = scene.push_path(ellipse([40.0, 20.0], 2.0));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);
    // A geometria local já está centrada perto da origem (o settle a moveu).
    let local_bbox = scene.path_curve_bbox(id).unwrap();
    assert!(
        local_bbox.0[0].abs() < 5.0,
        "o settle devia ter centrado a geometria local"
    );

    let container = create(&mut sim, &mut scene, &map, &[id]).expect("create");
    // A fonte do filho volta a estar em MUNDO (≈40,20): a pose foi assada.
    let src = source_of(&env_of(&sim, container).children[0]);
    let (o, _) = super::union_control_bbox([&src]).unwrap();
    assert!(
        o[0] > 35.0,
        "a pose de mundo NÃO foi assada na fonte (bbox em x={:.1}, esperado ≈38)",
        o[0]
    );
    // E a geometria do path na cena também (container na identidade ⇒ local == mundo).
    let (wo, _) = scene.path_curve_bbox(id).unwrap();
    assert!(
        wo[0] > 35.0,
        "o path da cena devia render em MUNDO após o create"
    );
}

/// **A POSE vive no CONTAINER e não vaza para a geometria dos filhos (Fatia 2).** Mover o `Transform`
/// do container move o envelope inteiro; o recook não o toca (escreve só geometria local). Duas
/// afirmações: (1) a pose do container sobrevive ao recook, (2) ela NÃO entrou nos vértices do filho
/// (a geometria local segue perto da fonte). Container-pose + filho-geometria-local = o gizmo não dobra.
#[test]
fn the_pose_lives_on_the_container_and_stays_out_of_child_geometry() {
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    let ce = Entity::from_bits(container);
    let pose = ph2d_core::Vec2::new(100.0, -50.0);
    sim.world_mut()
        .get_mut::<Transform>(ce)
        .expect("Transform")
        .translation = pose;

    let out = frame(&mut sim, &mut scene, ids[0]);

    assert_eq!(
        sim.world()
            .get::<Transform>(ce)
            .expect("Transform")
            .translation,
        pose,
        "o recook apagou a pose do container — o gizmo do Select seria inócuo"
    );
    let (o, _) = super::union_control_bbox([&out]).expect("bbox");
    assert!(
        o[0] < 50.0,
        "a pose vazou para a geometria do filho (bbox em x={:.1}, esperado perto da fonte ~3)",
        o[0]
    );
}

/// **A FONTE AUTORADA SOBREVIVE em cada filho.** O recook sobrescreveu os paths da cena com as
/// deformadas — mas a fonte afiada continua nos bytes de cada `VecEnvelopeChild`, e é ela que o
/// undo/save recuperam. Sem isto, o 1º frame varreria o desenho sem retorno (*"funciona e esquece"*).
#[test]
fn the_authored_source_survives_in_each_child() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (mut sim, mut scene, _map, container, _ids) = envelope_over(vec![src.clone()]);
    let (origin, size) =
        super::union_control_bbox([&source_of(&env_of(&sim, container).children[0])]).unwrap();
    let [bl, br, tr, tl] = bbox_corners(origin, size);
    set_corners(
        &mut sim,
        container,
        [bl, br, [tr[0] - size[0] * 0.3, tr[1]], tl],
    );

    recook(&mut sim, &mut scene);

    let kept = source_of(&env_of(&sim, container).children[0]);
    let cooked_src = src.cooked().into_owned();
    assert_eq!(
        kept.verts, cooked_src.verts,
        "a fonte autorada foi corrompida — o undo/save recuperariam a forma errada"
    );
}

/// **UMA GAIOLA DEGENERADA CONGELA as formas, não as apaga.** 3 cantos colineares ⇒ `QuadWarp::new`
/// devolve `None` ⇒ o recook pula, e os paths ficam onde estavam. É a mesma escolha do morph com uma
/// fonte perdida.
#[test]
fn a_degenerate_cage_freezes_the_shapes() {
    let (mut sim, mut scene, _map, container, ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    let (origin, size) =
        super::union_control_bbox([&source_of(&env_of(&sim, container).children[0])]).unwrap();
    // Cantos 1,2,3 (BR/TR/TL) colineares — a singularidade da homografia de Heckbert.
    let degenerate = [
        [origin[0], origin[1]],
        [origin[0] + size[0], origin[1]],
        [origin[0] + size[0], origin[1] + size[1] * 0.5],
        [origin[0] + size[0], origin[1] + size[1]],
    ];
    assert!(
        QuadWarp::new(origin, size, degenerate).is_none(),
        "o fixture devia ser degenerado"
    );
    set_corners(&mut sim, container, degenerate);

    let before = scene
        .paths()
        .iter()
        .find(|p| p.id == ids[0])
        .unwrap()
        .clone();
    let after = frame(&mut sim, &mut scene, ids[0]);
    assert_eq!(
        after.verts, before.verts,
        "gaiola degenerada mexeu na forma — ela tinha de CONGELAR onde estava"
    );
}

/// Um pen com `paths` selecionados — o que o `dissolve` lê para achar os envelopes tocados.
fn pen_with(paths: &[VecPathId]) -> ph2d_vec_edit::PenTool {
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(paths);
    pen
}

/// Deforma a gaiola do container num trapézio (o que o artista faz no Node).
fn pull_cage(sim: &mut SimWorld, container: u64) {
    let env = env_of(sim, container);
    let (origin, size) = super::union_control_bbox(
        env.children
            .iter()
            .map(source_of)
            .collect::<Vec<_>>()
            .iter(),
    )
    .expect("domínio");
    let [bl, br, ..] = bbox_corners(origin, size);
    set_corners(
        sim,
        container,
        [
            bl,
            br,
            [origin[0] + size[0] * 0.7, origin[1] + size[1]],
            [origin[0] + size[0] * 0.3, origin[1] + size[1]],
        ],
    );
}

/// **RELEASE ressuscita a fonte autorada e LIBERTA os filhos.** A deformação é desfeita, o filho
/// volta a ser RAIZ (sem pai, com ordem própria, na identidade — o estado de uma forma
/// recém-desenhada) e o container morre. Sem isto, envolver seria porta de mão única.
#[test]
fn release_restores_the_authored_source_and_frees_the_children() {
    let (mut sim, mut scene, map, container, ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    pull_cage(&mut sim, container);
    recook(&mut sim, &mut scene);
    let source = source_of(&env_of(&sim, container).children[0]);
    let deformed = scene
        .paths()
        .iter()
        .find(|p| p.id == ids[0])
        .unwrap()
        .clone();
    assert_ne!(
        deformed.verts, source.verts,
        "o fixture devia estar deformado"
    );

    let mut pen = pen_with(&ids);
    assert!(dissolve(
        &mut sim,
        &mut scene,
        &map,
        &mut pen,
        Keep::Authored
    ));

    // (1) a geometria voltou a ser a AUTORADA.
    let after = scene.paths().iter().find(|p| p.id == ids[0]).unwrap();
    assert_eq!(
        after.verts, source.verts,
        "o Release não ressuscitou a fonte autorada"
    );
    // (2) o filho é RAIZ de novo, na identidade.
    let child = Entity::from_bits(map[&ids[0]]);
    assert!(
        sim.world().get::<ChildOf>(child).is_none(),
        "o filho continuou pendurado no container"
    );
    assert!(
        sim.world().get::<ph2d_ecs::RootOrder>(child).is_some(),
        "sem RootOrder = empate"
    );
    assert_eq!(
        sim.world().get::<Transform>(child).copied(),
        Some(Transform::IDENTITY)
    );
    // (3) o container morreu, e a seleção passou às formas libertadas.
    assert!(
        sim.world()
            .get_entity(Entity::from_bits(container))
            .is_err(),
        "o container sobreviveu ao dissolve"
    );
    assert_eq!(pen.selected_paths(), ids.as_slice());
}

/// **EXPAND materializa a deformada** — o oposto do Release: a geometria que está na tela vira o
/// desenho definitivo. Mesma dissolução, outra resposta a *"qual geometria fica?"*.
#[test]
fn expand_keeps_the_deformed_geometry() {
    let (mut sim, mut scene, map, container, ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    pull_cage(&mut sim, container);
    recook(&mut sim, &mut scene);
    let source = source_of(&env_of(&sim, container).children[0]);
    let deformed = scene
        .paths()
        .iter()
        .find(|p| p.id == ids[0])
        .unwrap()
        .clone();

    let mut pen = pen_with(&ids);
    assert!(dissolve(
        &mut sim,
        &mut scene,
        &map,
        &mut pen,
        Keep::Deformed
    ));

    let after = scene.paths().iter().find(|p| p.id == ids[0]).unwrap();
    assert_eq!(after.verts, deformed.verts, "o Expand perdeu a deformação");
    assert_ne!(
        after.verts, source.verts,
        "o Expand devolveu a fonte (é o Release)"
    );
    assert!(
        sim.world()
            .get_entity(Entity::from_bits(container))
            .is_err()
    );
}

/// **A POSE do container é ASSADA na geometria — a arte NÃO se move ao dissolver.** É o inverso
/// exato do que o `create` fez na entrada. Sem isto, mover o envelope pelo gizmo e depois expandir
/// teleportaria a arte de volta para onde ela estava antes de ser movida.
#[test]
fn dissolving_bakes_the_container_pose_so_the_art_does_not_move() {
    let (mut sim, mut scene, map, container, ids) = envelope_over(vec![ellipse([5.0, 3.0], 2.0)]);
    // O gizmo do Select move/escala o envelope (translação + escala uniforme: assim bbox e afim
    // comutam e o oráculo é exato).
    let ce = Entity::from_bits(container);
    {
        let mut t = sim.world_mut().get_mut::<Transform>(ce).expect("Transform");
        t.translation = ph2d_core::Vec2::new(100.0, -50.0);
        t.scale = ph2d_core::Vec2::new(2.0, 2.0);
    }
    // Onde a arte está no MUNDO antes de dissolver (geometria local × pose do container).
    let xf =
        crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(&sim, ce));
    let (lo, hi) = scene.path_curve_bbox(ids[0]).unwrap();
    let (want_lo, want_hi) = (xf.apply(lo), xf.apply(hi));

    let mut pen = pen_with(&ids);
    assert!(dissolve(
        &mut sim,
        &mut scene,
        &map,
        &mut pen,
        Keep::Deformed
    ));

    // Depois: o filho está na identidade, então a bbox da CENA já é a de MUNDO.
    let (got_lo, got_hi) = scene.path_curve_bbox(ids[0]).unwrap();
    let d = (got_lo[0] - want_lo[0]).abs()
        + (got_lo[1] - want_lo[1]).abs()
        + (got_hi[0] - want_hi[0]).abs()
        + (got_hi[1] - want_hi[1]).abs();
    assert!(
        d < 1e-6,
        "a arte se moveu ao dissolver ({d:.3e}): a pose do container não foi assada na geometria"
    );
}

/// **Sem envelope na seleção o dissolve é no-op** — uma forma comum não é dissolvida, e o `false`
/// é o que impede o shell de imprimir que fez algo.
#[test]
fn dissolve_without_an_envelope_is_a_no_op() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ellipse([0.0, 0.0], 2.0));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let before = scene.paths().iter().find(|p| p.id == id).unwrap().clone();

    let mut pen = pen_with(&[id]);
    assert!(!dissolve(
        &mut sim,
        &mut scene,
        &map,
        &mut pen,
        Keep::Authored
    ));
    assert!(!dissolve(
        &mut sim,
        &mut scene,
        &map,
        &mut pen,
        Keep::Deformed
    ));
    let after = scene.paths().iter().find(|p| p.id == id).unwrap();
    assert_eq!(after.verts, before.verts, "uma forma comum foi mexida");
}

/// **`create` sobre nada dá `None`** — ids que não resolvem (formas inexistentes) não criam um
/// container órfão. É a guarda que impede um envelope vazio.
#[test]
fn create_over_no_live_shapes_is_none() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let map = VecEntityMap::new();
    // Ids que nunca entraram no mapa.
    assert!(create(&mut sim, &mut scene, &map, &[999, 1000]).is_none());
    assert!(create(&mut sim, &mut scene, &map, &[]).is_none());
    // E nenhum container foi deixado para trás.
    let mut q = sim.world_mut().query::<&VecEnvelope>();
    assert_eq!(
        q.iter(sim.world()).count(),
        0,
        "create deixou um container órfão"
    );
}
