//! Os gates do [`crate::envelope_live`] — o Envelope vivo (ADR-0129, Fatia B).
//!
//! O que eles medem é o comportamento do HOST; a **correção** da deformação (curva sobrevive, não
//! só os pontos de controle) é da crate `ph2d-vec-envelope` e já está gateada lá (invariância à
//! subdivisão). Aqui: em repouso não muda nada · a gaiola puxada deforma **pelo motor** · a **pose
//! sobrevive ao recook** (é ela que o gizmo do Select move — Fatia 2) e não vaza para a geometria
//! local · a fonte autorada sobrevive no componente · gaiola degenerada congela.

use super::*;
use ph2d_ecs::Transform;
use ph2d_vec_scene::{ShapeKind, cook};

/// Uma cena com um envelope sobre uma forma. Devolve `(sim, scene, map, id, componente)`.
///
/// Os `corners` são passados já puxados (ou em repouso) — `create` nasce em repouso, e o teste
/// sobrescreve antes de pendurar, que é o que a UI da fatia seguinte fará.
fn scene_with_envelope(
    shape: VecPath,
    corners: [[f64; 2]; 4],
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, VecEnvelope) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(shape);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let (eid, mut env) = create(&scene, id).expect("create");
    env.corners = corners;
    assert!(attach(&mut sim, &map, eid, &env));
    (sim, scene, map, id, env)
}

/// Roda um frame de recook e devolve o path da cena.
fn frame(sim: &mut SimWorld, scene: &mut VecScene, map: &VecEntityMap, id: VecPathId) -> VecPath {
    recook(sim, scene, map);
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o path")
        .clone()
}

fn ellipse(c: [f64; 2], r: f64) -> VecPath {
    cook(
        ShapeKind::Ellipse,
        [c[0] - r, c[1] - r],
        [c[0] + r, c[1] + r],
        &[],
    )
}

/// A fonte que o recook usa (a cozida, desserializada dos bytes do componente).
fn source_of(env: &VecEnvelope) -> VecPath {
    postcard::from_bytes(&env.source).expect("source")
}

/// **EM REPOUSO A FORMA NÃO MUDA.** Gaiola = cantos do bbox ⇒ identidade ⇒ a forma volta igual.
///
/// É a metade "presença" do par: um motor que não escrevesse nada também deixaria a forma "igual"
/// (vazia). Este teste afirma o contrário — a forma continua **cheia** e com o mesmo bbox.
#[test]
fn at_rest_the_envelope_leaves_the_shape_unchanged() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (origin, size) = control_bbox(&src).unwrap();
    let (mut sim, mut scene, map, id, _) =
        scene_with_envelope(src.clone(), bbox_corners(origin, size));

    let out = frame(&mut sim, &mut scene, &map, id);
    assert!(!out.verts.is_empty(), "a forma sumiu em repouso");
    let (o2, s2) = control_bbox(&out).expect("bbox");
    let d = (o2[0] - origin[0]).abs()
        + (o2[1] - origin[1]).abs()
        + (s2[0] - size[0]).abs()
        + (s2[1] - size[1]).abs();
    assert!(
        d < 1e-6,
        "repouso deslocou o bbox em {d:.3e} (devia ser identidade)"
    );
}

/// **A GAIOLA PUXADA DEFORMA — e escreve a saída do MOTOR, não a ingênua.**
///
/// Duas afirmações num teste: (a) a forma **mudou** (o bbox saiu do lugar), e (b) o que foi escrito
/// é **exatamente** `warp_path(fonte, QuadWarp)` — o host liga o motor, não uma 2ª porta. A correção
/// do motor (curva sobrevive) é gateada na crate; aqui prova-se o **fio**.
#[test]
fn a_pulled_cage_deforms_the_shape_through_the_engine() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (origin, size) = control_bbox(&src).unwrap();
    // Topo estreitado: trapézio convexo forte (perspectiva).
    let [bl, br, _tr, _tl] = bbox_corners(origin, size);
    let pulled = [
        bl,
        br,
        [origin[0] + size[0] * 0.7, origin[1] + size[1]],
        [origin[0] + size[0] * 0.3, origin[1] + size[1]],
    ];
    assert!(QuadWarp::is_convex(&pulled), "o fixture devia ser convexo");

    let (mut sim, mut scene, map, id, env) = scene_with_envelope(src, pulled);
    let out = frame(&mut sim, &mut scene, &map, id);

    // (a) mudou
    let source = source_of(&env);
    assert_ne!(
        out.verts, source.verts,
        "a gaiola foi puxada e a forma não mudou"
    );

    // (b) é a saída do motor, byte a byte
    let (o, s) = control_bbox(&source).unwrap();
    let warp = QuadWarp::new(o, s, pulled).unwrap();
    let expected = warp_path(&source, &warp, accuracy(s));
    assert_eq!(
        out.verts, expected.verts,
        "o recook não escreveu a saída do motor — 2ª porta para a mesma pergunta"
    );
}

/// **A SEQUÊNCIA DO SMOKE deforma** — e é a que difere: no smoke o `create` roda **depois** de o
/// `settle_origins` ter centrado a forma (geometria vira LOCAL, `Transform` ganha a pose). O teste
/// acima cria antes de qualquer settle (forma em MUNDO, identidade), e por isso não pegava um bug
/// de reconstrução mundo↔local. Este reproduz o frame real: push → sync → **settle** → build →
/// create → pull → attach → recook.
#[test]
fn the_smoke_sequence_deforms_the_shape() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ellipse([0.0, 0.0], 2.0));

    // Frame N: sync + settle (o que o smoke deixa acontecer antes de criar o envelope).
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    // Frame N+1 prólogo: create sobre a forma JÁ assentada.
    let (eid, mut env) = create(&scene, id).expect("create pós-settle");

    // Puxa como o smoke: topo a 35% da base.
    let [bl, br, tr, tl] = env.corners;
    let cx = (tl[0] + tr[0]) * 0.5;
    let k = 0.35;
    env.corners = [
        bl,
        br,
        [cx + (tr[0] - cx) * k, tr[1]],
        [cx + (tl[0] - cx) * k, tl[1]],
    ];
    assert!(attach(&mut sim, &map, eid, &env));

    let out = frame(&mut sim, &mut scene, &map, id);

    // A fonte que o recook desta sequência de facto usou, e a saída que o motor produziria dela.
    let e = Entity::from_bits(map[&id]);
    let env_now = sim
        .world()
        .get::<VecEnvelope>(e)
        .expect("componente")
        .clone();
    let source = source_of(&env_now);
    let (o, s) = control_bbox(&source).expect("bbox");
    let expected = warp_path(
        &source,
        &QuadWarp::new(o, s, env_now.corners).unwrap(),
        accuracy(s),
    );

    // (1) a DEFORMAÇÃO aconteceu — a saída difere da fonte em repouso.
    assert_ne!(
        out.verts, source.verts,
        "a sequência do smoke NÃO deformou: o recook devolveu a fonte intacta"
    );
    // (2) e é a saída do motor (o fio está certo mesmo pós-settle).
    assert_eq!(
        out.verts, expected.verts,
        "pós-settle o recook divergiu do motor — a reconstrução mundo<->local corrompeu a fonte"
    );
}

/// **O caminho REAL do app: `upkeep` drena o pending e ANEXA** (não `attach` direto). Os outros
/// gates chamam `attach` na mão; o app enfileira em `vec_envelope_pending` e o `upkeep` do frame
/// seguinte é quem anexa. Se o `upkeep` não anexar (ex.: a forma "some" da cena por um frame, ou o
/// mapa não tem a entidade), o componente nunca chega, o recook não acha envelope, e a forma fica
/// **sem deformar e sem gaiola** — exatamente o sintoma de "não vejo canto onde puxar".
#[test]
fn the_upkeep_path_attaches_and_deforms() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(ellipse([0.0, 0.0], 2.0));

    // Frame N: sync + settle (o que o app faz antes de o smoke criar o envelope).
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    // Frame N+1 prólogo (o smoke): create + pull, e ENFILEIRA (não anexa na mão).
    let (eid, mut env) = create(&scene, id).expect("create");
    let [bl, br, tr, tl] = env.corners;
    let cx = (tl[0] + tr[0]) * 0.5;
    env.corners = [
        bl,
        br,
        [cx + (tr[0] - cx) * 0.35, tr[1]],
        [cx + (tl[0] - cx) * 0.35, tl[1]],
    ];
    let mut pending = Some((eid, env));

    // Frame N+1 corpo: sync + upkeep (a porta do app) + recook.
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    assert!(pending.is_none(), "o upkeep não drenou o pending");

    // O componente TEM de estar anexado — senão o recook não acha envelope nenhum.
    let e = Entity::from_bits(map[&id]);
    assert!(
        sim.world().get::<VecEnvelope>(e).is_some(),
        "o upkeep não anexou o VecEnvelope — nem deforma nem desenha a gaiola"
    );

    let source = frame(&mut sim, &mut scene, &map, id);
    // (`frame` roda o recook e devolve o path.) A forma tem de ter deformado.
    let cooked_src = ellipse([0.0, 0.0], 2.0).cooked().into_owned();
    assert_ne!(
        source.verts, cooked_src.verts,
        "o upkeep anexou mas a forma não deformou"
    );
}

/// **A POSE sobrevive ao recook — e NÃO vaza para a geometria local (Fatia 2).** No modelo LOCAL a
/// geometria escrita é local e o `Transform` é a pose; o recook **não** a força a identidade (o
/// modelo antigo forçava, e por isso o gizmo do Select era inócuo). Duas afirmações: (1) a pose
/// continua lá após o recook, (2) ela NÃO entrou nos vértices (o bbox local segue perto da fonte,
/// não deslocado pela pose) — geometria-local + pose-separada é o que impede o gizmo de dobrar.
#[test]
fn the_pose_survives_recook_and_stays_out_of_the_local_geometry() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (origin, size) = control_bbox(&src).unwrap();
    let (mut sim, mut scene, map, id, _) = scene_with_envelope(src, bbox_corners(origin, size));

    let e = Entity::from_bits(map[&id]);
    let pose = ph2d_core::Vec2::new(100.0, -50.0);
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .translation = pose;

    let out = frame(&mut sim, &mut scene, &map, id);

    // (1) a pose sobrevive — é ela que o gizmo do Select move.
    assert_eq!(
        sim.world()
            .get::<Transform>(e)
            .expect("Transform")
            .translation,
        pose,
        "o recook apagou a pose (o modelo antigo forçava identidade) — o gizmo seria inócuo"
    );
    // (2) e a pose NÃO vazou para a geometria: a fonte vive perto de x=5; se a translação de +100
    // tivesse entrado nos vértices, o bbox local estaria lá longe. Local + pose separados.
    let (o, _) = control_bbox(&out).expect("bbox");
    assert!(
        o[0] < 50.0,
        "a pose vazou para a geometria local (bbox em x={:.1}, esperado perto da fonte ~3)",
        o[0]
    );
}

/// **A FONTE AUTORADA SOBREVIVE no componente.** O recook sobrescreveu o path da cena com a
/// deformada — mas a fonte afiada continua nos bytes do `VecEnvelope`, e é ela que o undo/save
/// recuperam. Sem isto, o 1º frame varreria o desenho sem retorno (o *"funciona e depois esquece"*).
#[test]
fn the_authored_source_survives_in_the_component() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (origin, size) = control_bbox(&src).unwrap();
    let [bl, br, tr, tl] = bbox_corners(origin, size);
    let pulled = [bl, br, [tr[0] - size[0] * 0.3, tr[1]], tl];

    let (mut sim, mut scene, map, id, _) = scene_with_envelope(src.clone(), pulled);
    let scene_before = scene.paths().iter().find(|p| p.id == id).unwrap().clone();
    let out = frame(&mut sim, &mut scene, &map, id);
    assert_ne!(
        out.verts, scene_before.verts,
        "o recook devia ter deformado"
    );

    // A fonte no componente continua a ser a original (cozida), intacta.
    let e = Entity::from_bits(map[&id]);
    let env = sim.world().get::<VecEnvelope>(e).expect("componente");
    let kept = source_of(env);
    let cooked_src = src.cooked().into_owned();
    assert_eq!(
        kept.verts, cooked_src.verts,
        "a fonte autorada foi corrompida — o undo/save recuperariam a forma errada"
    );
}

/// **UMA GAIOLA DEGENERADA CONGELA a forma, não a apaga.** 3 cantos colineares ⇒ `QuadWarp::new`
/// devolve `None` ⇒ o recook pula, e o path da cena fica onde estava (não some, não vira lixo). É a
/// mesma escolha do morph com uma fonte perdida.
#[test]
fn a_degenerate_cage_freezes_the_shape() {
    let src = ellipse([5.0, 3.0], 2.0);
    let (origin, size) = control_bbox(&src).unwrap();
    // Cantos 1,2,3 (BR/TR/TL) colineares — todos em x = ox+w. É essa a singularidade da homografia
    // de Heckbert (o denominador de Cramer sobre q1,q2,q3 some), não "três cantos quaisquer".
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

    let (mut sim, mut scene, map, id, _) = scene_with_envelope(src, degenerate);
    let before = scene.paths().iter().find(|p| p.id == id).unwrap().clone();
    let after = frame(&mut sim, &mut scene, &map, id);
    assert_eq!(
        after.verts, before.verts,
        "gaiola degenerada mexeu na forma — ela tinha de CONGELAR onde estava"
    );
}
