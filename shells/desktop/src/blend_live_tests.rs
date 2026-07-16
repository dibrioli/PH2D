//! Testes do [`crate::blend_live`] — módulo irmão (teto de 600 LOC por arquivo da shell).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **A transição SEGUE a forma** (`moving_a_source_reflows_the_blend`) — a feature inteira numa
//!    asserção: mover uma fonte re-coza o overlay, sem re-clicar "Blend".
//! 2. **Girar QUALQUER ponta refaz os passos** (`rotating_either_endpoint_reflows_the_blend`) — a
//!    simetria; o "a última não influencia" do smoke era o CÍRCULO (rotacionalmente simétrico).
//! 3. **A última forma fica POR CIMA** (`the_last_source_is_drawn_on_top`) — a pilha de z do
//!    Illustrator: fonte0 embaixo → passos → fonte1 em cima.
//! 4. A cadeia é pairwise; o blend vive na identidade; o `settle` o pula; um elo morto é pulado.
//!
//! # O `out` do recook é o OVERLAY ORDENADO (passos + fontes reempilhadas)
//!
//! Não é "os passos": é o que o passe de render desenha, em z. Para cada elo, os passos e **a
//! fonte de cima** (redesenhada por cima deles) — só a fonte0 fica no z da cena. Por isso o tamanho
//! é `(elos)·(passos+1)`, e o último item é sempre a última fonte.

use super::*;
use ph2d_vec_scene::{ShapeKind, VecScene, cook, rectangle};

/// Uma cena com `n` retângulos regularmente espaçados e um Blend Object vivo sobre eles.
/// Devolve `(sim, scene, map, spine_id, sources)`. `pub(super)` — o irmão `spine_tests` reusa.
#[allow(clippy::type_complexity)]
pub(super) fn scene_with_blend(
    n: usize,
    steps: u32,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Retângulos 2×2, centros em x = 0, 4, 8, … (bem separados: a correspondência importa).
    let sources: Vec<VecPathId> = (0..n)
        .map(|i| {
            let x = i as f64 * 4.0;
            scene.push_path(rectangle([x - 1.0, -1.0], [x + 1.0, 1.0]))
        })
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let xf = crate::vec_transform::build(&sim, &map);
    let (spine_id, blend) = create(&mut scene, &xf, &sources, steps).expect("create");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map); // dá entidade ao spine
    assert!(attach(&mut sim, &map, spine_id, &blend));
    (sim, scene, map, spine_id, sources)
}

/// O centro (média das âncoras) de um path de MUNDO. `pub(super)` — o irmão `spine_tests` reusa.
pub(super) fn centroid(p: &ph2d_vec_scene::VecPath) -> [f64; 2] {
    let n = p.verts.len().max(1) as f64;
    let (sx, sy) = p
        .verts
        .iter()
        .fold((0.0, 0.0), |(x, y), v| (x + v.anchor[0], y + v.anchor[1]));
    [sx / n, sy / n]
}

/// A geometria do overlay inteiro, achatada — muda se QUALQUER passo mudou (o oráculo do reflow).
fn fingerprint(out: &[ph2d_vec_scene::VecPath]) -> Vec<[f64; 2]> {
    out.iter()
        .flat_map(|p| p.verts.iter().map(|v| v.anchor))
        .collect()
}

/// Um blend sobre DUAS formas quaisquer (não só retângulos). Devolve `(sim, scene, map, sources)`.
fn blend_two(
    a: ph2d_vec_scene::VecPath,
    b: ph2d_vec_scene::VecPath,
    steps: u32,
) -> (SimWorld, VecScene, VecEntityMap, [VecPathId; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ia = scene.push_path(a);
    let ib = scene.push_path(b);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let xf = crate::vec_transform::build(&sim, &map);
    let (spine, blend) = create(&mut scene, &xf, &[ia, ib], steps).expect("create");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(attach(&mut sim, &map, spine, &blend));
    (sim, scene, map, [ia, ib])
}

/// O fingerprint do overlay de um blend, com a fonte `idx` girada `rot` rad.
fn overlay_of(
    mut sim: SimWorld,
    mut scene: VecScene,
    map: &VecEntityMap,
    rotate: Option<(VecPathId, f32)>,
) -> Vec<[f64; 2]> {
    if let Some((id, r)) = rotate {
        let e = Entity::from_bits(map[&id]);
        sim.world_mut()
            .get_mut::<Transform>(e)
            .expect("Transform")
            .rotation = r;
    }
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, map);
    recook(
        &mut sim,
        &mut scene,
        map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    fingerprint(&out)
}

/// O overlay depois de rodar o recook com as fontes (retângulos) giradas `(rot_a, rot_b)` rad.
fn overlay_with_rotation(rot_a: f32, rot_b: f32) -> Vec<[f64; 2]> {
    let (mut sim, mut scene, map, _s, src) = scene_with_blend(2, 3);
    for (id, r) in [(src[0], rot_a), (src[1], rot_b)] {
        if r != 0.0 {
            let e = Entity::from_bits(map[&id]);
            sim.world_mut()
                .get_mut::<Transform>(e)
                .expect("Transform")
                .rotation = r;
        }
    }
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    fingerprint(&out)
}

/// **O TESTE.** Move-se uma fonte; a transição se refaz.
///
/// É a feature inteira: os passos são função pura das fontes cozidas no MUNDO, então arrastar uma
/// forma (o gizmo, ADR-0111) recoza o overlay — sem re-clicar "Blend". A última fonte (o topo da
/// pilha) anda o delta EXATO; e o overlay como um todo muda.
#[test]
fn moving_a_source_reflows_the_blend() {
    let (mut sim, mut scene, map, _spine, sources) = scene_with_blend(2, 5);
    let mut out = Vec::new();

    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    // 2 fontes, 1 elo: 5 passos + a fonte de cima = 6 no overlay.
    assert_eq!(out.len(), 6, "5 passos + a fonte de cima");
    let before = fingerprint(&out);
    let b_before = centroid(out.last().expect("a fonte de cima"));

    // Move a 2ª fonte (B) por d — o gizmo de sprite faria exatamente isto.
    let d = [3.0_f32, 2.0_f32];
    let eb = Entity::from_bits(map[&sources[1]]);
    sim.world_mut()
        .get_mut::<Transform>(eb)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(d[0], d[1]);

    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    let b_after = centroid(out.last().expect("a fonte de cima"));

    assert_ne!(
        fingerprint(&out),
        before,
        "o overlay tinha de mudar ao mover B"
    );
    // A fonte de cima (B) andou o delta EXATO.
    assert!(
        (b_after[0] - b_before[0] - f64::from(d[0])).abs() < 1e-6
            && (b_after[1] - b_before[1] - f64::from(d[1])).abs() < 1e-6,
        "a fonte B tinha de andar {d:?}: {b_before:?} -> {b_after:?}"
    );
}

/// **A simetria da rotação.** Girar QUALQUER ponta refaz os passos — não só a primeira.
///
/// O "a última forma não influencia" do smoke do Enio era o CÍRCULO: um círculo é rotacionalmente
/// simétrico, então girá-lo não muda geometria nenhuma. Aqui as duas pontas são quadrados
/// (assimétricos a 0,4 rad ≈ 23°), e as DUAS têm de influenciar — senão haveria um bug de fato.
#[test]
fn rotating_either_endpoint_reflows_the_blend() {
    let base = overlay_with_rotation(0.0, 0.0);
    assert_ne!(
        overlay_with_rotation(0.4, 0.0),
        base,
        "girar a 1ª fonte tem de refazer os passos"
    );
    assert_ne!(
        overlay_with_rotation(0.0, 0.4),
        base,
        "girar a ÚLTIMA fonte TAMBÉM (o círculo do smoke é simétrico — daí não influenciar)"
    );
}

/// **A ponta LISA também reflui — quando NÃO é um círculo.** É o caso exato do smoke: a última
/// forma era um CÍRCULO (elipse 2×2), rotacionalmente simétrico, então girá-lo não muda geometria
/// nenhuma (o "não influencia" do Enio — correto, não é bug). O smoke agora usa uma elipse
/// NÃO-circular na ponta, e girá-la refaz o blend — o reflow chega ao lado liso.
#[test]
fn rotating_a_smooth_noncircular_endpoint_reflows_the_blend() {
    let star = cook(
        ShapeKind::Star,
        [-4.0, -1.0],
        [-2.0, 1.0],
        &[5.0, 0.45, 0.0],
    );
    let ellipse = cook(ShapeKind::Ellipse, [2.0, -0.65], [4.0, 0.65], &[]); // 2×1.3, orientada
    let (s0, sc0, m0, _) = blend_two(star.clone(), ellipse.clone(), 3);
    let base = overlay_of(s0, sc0, &m0, None);
    let (s1, sc1, m1, src1) = blend_two(star, ellipse, 3);
    let rot = overlay_of(s1, sc1, &m1, Some((src1[1], 0.5)));
    assert_ne!(
        base, rot,
        "girar a elipse (não-circular) tem de refazer o blend"
    );
}

/// **A pilha de z do Illustrator.** A última fonte é desenhada por ÚLTIMO — POR CIMA do último
/// passo (o defeito do smoke: o círculo ficava enterrado sob o último passo). O último item do
/// overlay é EXATAMENTE a última fonte assada no mundo. Gate mutation-testável: se o recook voltar
/// a empilhar só os passos, o último item vira um passo ≠ a fonte, e isto fica vermelho.
#[test]
fn the_last_source_is_drawn_on_top() {
    let (mut sim, mut scene, map, _spine, sources) = scene_with_blend(3, 3);
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    let last_source = world(&scene, &xf, *sources.last().expect("fonte"));
    assert_eq!(
        out.last().map(centroid),
        last_source.as_ref().map(centroid),
        "o último item desenhado tem de ser a última fonte (ela fica no topo)"
    );
    assert_eq!(
        out.last().map(|p| p.verts.len()),
        last_source.as_ref().map(|p| p.verts.len()),
        "…e ser a forma da fonte, não um passo interpolado"
    );
}

/// A cadeia é **pairwise**: N fontes ⇒ (N−1) elos, e cada elo contribui `steps` passos + 1 fonte
/// de cima. É o Blend multi-forma do Illustrator, a capacidade nova do ADR-0122.
#[test]
fn chain_is_pairwise_across_sources() {
    for (n, steps) in [(2, 5), (3, 4), (5, 2)] {
        let want = (n - 1) * (steps as usize + 1);
        let (mut sim, mut scene, map, _s, _src) = scene_with_blend(n, steps);
        let mut out = Vec::new();
        let xf = crate::vec_transform::build(&sim, &map);
        recook(
            &mut sim,
            &mut scene,
            &map,
            &xf,
            &mut BlendSpines::new(),
            &mut out,
        );
        assert_eq!(
            out.len(),
            want,
            "{n} fontes × ({steps} passos + 1 fonte)/elo = {want}"
        );
    }
}

/// O blend vive na IDENTIDADE: o recook devolve o `Transform` da entidade dele à identidade,
/// mesmo que alguém (o gizmo) o tenha mexido. É o que o torna não-arrastável — o que se move são
/// as fontes.
#[test]
fn the_blend_object_lives_at_identity() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let es = Entity::from_bits(map[&spine]);
    // Alguém arrastou o blend (o gizmo). O recook tem de desfazer.
    sim.world_mut()
        .get_mut::<Transform>(es)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(5.0, 5.0);

    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        Some(Transform::IDENTITY),
        "o recook devolve o blend à identidade (a geometria dele é MUNDO)"
    );
}

/// O `settle_origins` PULA o blend. O spine tem geometria de MUNDO (centros das fontes, longe de
/// 0), então sem o pulo o `settle` o centraria e a transição sairia deslocada. Gate
/// mutation-testável: tire o `VecBlend.is_none()` do `settle` e este fica vermelho.
#[test]
fn settle_skips_the_blend_object() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let es = Entity::from_bits(map[&spine]);
    let before = sim.world().get::<Transform>(es).copied();

    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        before,
        "o settle não pode assentar o blend (ele vive na identidade)"
    );
    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        Some(Transform::IDENTITY),
    );
}

/// Um elo morto (fonte apagada) é PULADO — a cadeia não quebra. E sem 2 fontes vivas, o overlay
/// some (o spine fica vazio, nada é desenhado).
#[test]
fn a_dead_source_is_skipped_and_below_two_the_blend_vanishes() {
    let (mut sim, mut scene, map, _spine, sources) = scene_with_blend(3, 4);
    let mut out = Vec::new();

    // Apaga a fonte do MEIO — restam 2, a cadeia vira 1 elo (4 passos + 1 fonte = 5 no overlay).
    scene.remove_path(sources[1]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    assert_eq!(out.len(), 5, "3→2 fontes vivas: 1 elo, 4 passos + 1 fonte");

    // Apaga mais uma — resta 1, não há transição.
    scene.remove_path(sources[0]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    assert!(out.is_empty(), "menos de 2 fontes vivas: overlay vazio");
}

/// **A criação pelo painel: a seleção FECHADA, em z, capada em 5.** É o que o botão "Blend" liga.
/// Formas abertas (sem interior para interpolar) caem; a ordem é a de z (a cadeia do "Make" do
/// Illustrator), não a de clique.
#[test]
fn selected_closed_in_z_drops_open_sorts_by_z_and_caps_at_five() {
    let mut scene = VecScene::new();
    // 6 fechadas em z crescente + 1 ABERTA.
    let closed: Vec<VecPathId> = (0..6)
        .map(|i| {
            let x = i as f64 * 3.0;
            scene.push_path(rectangle([x - 1.0, -1.0], [x + 1.0, 1.0]))
        })
        .collect();
    let open = scene.push_path(ph2d_vec_scene::line([0.0, 5.0], [1.0, 5.0]));

    let mut pen = ph2d_vec_edit::PenTool::default();
    // Seleciona TUDO fora de ordem, com a aberta no meio.
    pen.select_many(&[
        closed[3], open, closed[0], closed[5], closed[1], closed[4], closed[2],
    ]);

    let got = selected_closed_in_z(&scene, &pen);
    assert_eq!(
        got.as_slice(),
        &closed[..MAX_BLEND_SOURCES],
        "fechadas em z, a aberta fora, capado em {MAX_BLEND_SOURCES}"
    );
}

/// **O slider Steps retuna o blend que a seleção TOCA, ao vivo — pela LINHA ou por uma FORMA.**
/// Nada tocado ⇒ não faz nada (o valor é só o de criação futura). Idempotente.
///
/// A FORMA é o caso que importa (Enio 2026-07-15): no modo Select a linha nem é selecionável, então
/// se só o spine contasse o slider ficaria inerte justo no modo em que se mexe nas formas.
#[test]
fn set_selected_steps_retunes_the_blend_touched_by_the_selection() {
    let (mut sim, _scene, map, spine, src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    let steps_of = |sim: &SimWorld| sim.world().get::<VecBlend>(e).expect("blend").steps;

    let mut pen = ph2d_vec_edit::PenTool::default();
    assert!(
        !set_selected_steps(&mut sim, &map, &pen, 10),
        "nada selecionado, nada é retunado"
    );

    // (a) Pela LINHA (o modo Node seleciona o spine).
    pen.select_many(&[spine]);
    assert!(
        set_selected_steps(&mut sim, &map, &pen, 10),
        "o spine selecionado retuna"
    );
    assert_eq!(steps_of(&sim), 10);
    assert!(
        !set_selected_steps(&mut sim, &map, &pen, 10),
        "o MESMO valor não marca a entidade suja (idempotente)"
    );

    // (b) Por uma FORMA-fonte (o modo Select — a linha não é selecionável lá).
    pen.select_many(&[src[1]]);
    assert!(
        set_selected_steps(&mut sim, &map, &pen, 7),
        "uma FORMA do blend selecionada também retuna"
    );
    assert_eq!(steps_of(&sim), 7);

    // (c) Uma forma que NÃO é do blend não retuna nada.
    let mut other_scene = _scene;
    let outsider = other_scene.push_path(rectangle([99.0, 99.0], [100.0, 100.0]));
    pen.select_many(&[outsider]);
    assert!(
        !set_selected_steps(&mut sim, &map, &pen, 3),
        "uma forma fora do blend não o retuna"
    );
    assert_eq!(steps_of(&sim), 7, "os passos ficaram como estavam");
}

/// **Modo Node: o spine aparece elevado ao topo do overlay — e a cena segue invisível.** Na cena o
/// spine é sempre INVISÍVEL (o `recook` mantém o traço em `None`, para não virar fantasma no Select);
/// `elevate_spines` acrescenta um clone TRAÇADO no fim do overlay — desenhado por último, acima de
/// tudo. O sibling de PRESENÇA (o clone tem traço VISÍVEL) é essencial: sem ele, "não está na cena"
/// ficaria verde com um spine que não é desenhado em lugar nenhum.
#[test]
fn in_node_mode_the_spine_is_lifted_onto_the_overlay_top_and_the_scene_stays_invisible() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    // O spine é INVISÍVEL na cena (nem no Select ele aparece — é Node-only).
    let spine_verts = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .clone();
    assert!(
        scene
            .paths()
            .iter()
            .find(|p| p.id == spine)
            .expect("spine")
            .stroke
            .is_none(),
        "o spine é invisível na cena (Select não o mostra)"
    );

    let before = out.len();
    crate::blend_live::elevate_spines(&sim, &mut scene, &map, &mut out);

    // A cena segue invisível; o topo do overlay é o spine, TRAÇADO (visível no Node).
    assert!(
        scene
            .paths()
            .iter()
            .find(|p| p.id == spine)
            .expect("spine")
            .stroke
            .is_none(),
        "a cena segue invisível"
    );
    assert_eq!(out.len(), before + 1, "o spine foi acrescentado ao overlay");
    let top = out.last().expect("topo");
    assert!(
        top.stroke.is_some(),
        "o spine elevado é desenhado (traço visível no Node)"
    );
    assert!(!top.closed, "o spine é um path ABERTO");
    let top_verts: Vec<_> = top.verts.iter().map(|v| v.anchor).collect();
    let want: Vec<_> = spine_verts.iter().map(|v| v.anchor).collect();
    assert_eq!(
        top_verts, want,
        "o spine de topo tem a geometria do spine da cena"
    );
}

/// **No modo Select o spine é INVISÍVEL na cena** (Enio 2026-07-15) — mantê-lo traçado o mostrava
/// como um "fantasma" com drift ao mover as formas. O `recook` zera o traço todo frame, então nem a
/// criação (que nasce sem traço) nem um frame anterior o deixam aparecer.
#[test]
fn the_spine_is_invisible_in_the_scene_in_select_mode() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let mut out = Vec::new();
    let mut spines = BlendSpines::new();
    let xf = crate::vec_transform::build(&sim, &map);

    for _ in 0..3 {
        recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
        assert!(
            scene
                .paths()
                .iter()
                .find(|p| p.id == spine)
                .expect("spine")
                .stroke
                .is_none(),
            "o spine fica invisível na cena todo frame (sem fantasma no Select)"
        );
    }
}

/// **Pick Shapes: a prévia realça as escolhas E as costura na ORDEM DE CLIQUE** (ADR-0122 C2b) —
/// não na de z. Três retângulos em x = 0, 4, 8; escolhidos fora de ordem (8, 0, 4). A prévia tem um
/// contorno por escolha (sem fill, com traço) + uma polilinha aberta pelos centros nessa mesma
/// ordem. Se a prévia usasse a ordem de z, os x sairiam 0, 4, 8 — o oráculo distingue as duas.
#[test]
fn pick_preview_outlines_the_picks_and_threads_them_in_click_order() {
    let (sim, scene, map, _spine, src) = scene_with_blend(3, 3); // retângulos em x = 0, 4, 8
    let picks = vec![src[2], src[0], src[1]]; // ordem de CLIQUE: 8, 0, 4
    let xf = crate::vec_transform::build(&sim, &map);
    let preview = crate::blend_live::pick_preview(&scene, &xf, &picks);

    assert_eq!(preview.len(), 4, "3 contornos + a polilinha");
    for outline in &preview[..3] {
        assert!(
            outline.fill.is_none() && outline.stroke.is_some(),
            "cada escolha vira um CONTORNO realçado (sem fill)"
        );
    }
    let line = preview.last().expect("polilinha");
    assert!(
        !line.closed && line.stroke.is_some(),
        "a costura é uma polilinha ABERTA e traçada"
    );
    let xs: Vec<f64> = line.verts.iter().map(|v| v.anchor[0]).collect();
    assert_eq!(xs.len(), 3, "um vértice por escolha");
    assert!(
        (xs[0] - 8.0).abs() < 1e-6 && (xs[1] - 0.0).abs() < 1e-6 && (xs[2] - 4.0).abs() < 1e-6,
        "a polilinha segue a ORDEM DE CLIQUE (8, 0, 4), não a de z: {xs:?}"
    );
}
