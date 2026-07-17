//! Testes do **SPINE editável** do [`crate::blend_live`] (ADR-0128 Fase C2) — irmão de
//! `blend_live_tests.rs`, separado pelo teto de 600 LOC. Reusa `scene_with_blend`/`centroid` de lá
//! (`super::tests`, `pub(super)`).
//!
//! O que provam: o spine editado puxa os passos pela curva (arco); o não-editado fica na reta; a
//! edição AUTORA o spine (e trava o auto-regen); as ÂNCORAS seguem as fontes e arrastar uma âncora
//! (ponta OU meio da cadeia) MOVE a forma dela; o Reset volta ao automático.

use super::tests::{blend_two, centroid, scene_with_blend};
use super::*;

/// Uma **rosquinha** de raio `r_out` com buraco `r_in`, centrada em `c` — como a booleana a monta
/// (contorno de fora primário, buraco em `subpaths`, `EvenOdd`).
fn donut(c: [f64; 2], r_out: f64, r_in: f64) -> ph2d_vec_scene::VecPath {
    let ring = |r: f64| {
        ph2d_vec_scene::cook(
            ph2d_vec_scene::ShapeKind::Ellipse,
            [c[0] - r, c[1] - r],
            [c[0] + r, c[1] + r],
            &[],
        )
    };
    ph2d_vec_scene::VecPath {
        verts: ring(r_out).verts,
        closed: true,
        subpaths: vec![ph2d_vec_scene::Contour::new_closed(ring(r_in).verts)],
        fill_rule: ph2d_vec_scene::FillRule::EvenOdd,
        ..ph2d_vec_scene::VecPath::default()
    }
}

/// **O BURACO FLUI JUNTO COM O PASSO.**
///
/// O deslocamento do spine é aplicado ao passo INTEIRO, não ao contorno de fora dele. Um laço só
/// sobre `verts` mandava a parede para a curva e **deixava o buraco para trás**, na posição do
/// lerp: o passo lá em cima ficava sólido, e um buraco órfão pairava lá embaixo, dentro de nada.
///
/// Não é hipótese — era o código, e ele não tinha gate nenhum: o motor nunca produzia um passo com
/// buraco, então o defeito era inalcançável e ficou dormente até a rosquinha chegar aqui.
/// [[feedback_two_doors_to_the_same_question_diverge]]
#[test]
fn an_authored_spine_flows_the_hole_with_the_step() {
    let (mut sim, mut scene, map, _src) = blend_two(
        donut([0.0, 0.0], 1.0, 0.5),
        donut([4.0, 0.0], 1.0, 0.5),
        1, // um passo, no meio
    );
    let spine = *map
        .keys()
        .find(|id| scene.paths().iter().any(|p| p.id == **id && !p.closed))
        .expect("o spine é o único path ABERTO da cena");
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]);

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

    let step = &out[0];
    let at = centroid(step); // o centro do contorno de FORA — para onde a parede foi
    assert!(at[1] > 2.0, "o passo nem subiu para a curva: {at:?}");
    assert!(
        !ph2d_vec_scene::contains_point(step, at),
        "o passo subiu para o spine SÓLIDO: o buraco ficou para trás, na posição do lerp"
    );
}

/// Bota o spine `id` com os vértices `pts` (o que a edição no modo Node faria).
fn set_spine(scene: &mut VecScene, id: VecPathId, pts: &[[f64; 2]]) {
    let p = scene.path_mut(id).expect("spine");
    p.verts = pts.iter().map(|&q| VecVertex::corner(q)).collect();
    p.closed = false;
}

/// **O spine EDITADO puxa os passos para a curva** (o coração do pedido do Enio). As fontes estão
/// em (0,0) e (4,0); um spine com pico em (2,3) leva o passo do meio (fração de arco ½) para o
/// topo. Prova o flow por comprimento de arco no caminho real do recook.
#[test]
fn an_authored_bent_spine_flows_the_steps_onto_the_curve() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 1); // 1 passo (no meio)
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]); // pico no meio

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

    // out = [passo, fonte_de_cima]. O passo (lerp em (2,0)) subiu para ~(2,3).
    let step = centroid(&out[0]);
    assert!(
        (step[0] - 2.0).abs() < 0.5 && step[1] > 2.0,
        "o passo tinha de subir para a curva do spine: {step:?}"
    );
}

/// **O spine NÃO-editado deixa os passos na reta do lerp** — o caminho automático é byte-idêntico
/// à Fase B (as fontes estão em y=0, então todo passo fica em y=0; um flow espúrio o tiraria).
#[test]
fn an_unedited_spine_leaves_the_steps_on_the_lerp() {
    let (mut sim, mut scene, map, _spine, _src) = scene_with_blend(2, 5);
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
    for step in &out {
        let c = centroid(step);
        assert!(
            c[1].abs() < 1e-9,
            "não-autorado: passo na reta (y=0): {c:?}"
        );
    }
}

/// **Editar o spine o AUTORA — e o recook para de sobrescrevê-lo.** É a detecção (spine atual ≠
/// último auto): frame 1 escreve o auto e o memoriza; a mão adiciona um ponto INTERIOR; frame 2
/// detecta, marca `spine_authored` e o interior SOBREVIVE (o auto teria voltado a 2 vértices). Edita
/// o INTERIOR, não uma ponta — as pontas são pinadas aos centros (ver o gate de pinagem).
#[test]
fn editing_the_spine_authors_it_and_stops_the_auto_regen() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    let mut spines = BlendSpines::new();
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);

    // Frame 1: automático. O recook escreve o auto (2 vértices) e o memoriza.
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(
        !sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "ainda automático"
    );

    // A mão edita o spine (modo Node): ADICIONA um ponto interior e o sobe.
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 5.0], [4.0, 0.0]]);

    // Frame 2: detecta a edição → autora, e o INTERIOR sobrevive (auto teria voltado a 2 vértices).
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(
        sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "a edição tinha de autorar o spine"
    );
    let p = scene.paths().iter().find(|p| p.id == spine).expect("spine");
    assert_eq!(
        p.verts.len(),
        3,
        "o ponto interior não foi varrido pelo auto-regen"
    );
    assert!(
        p.verts[1].anchor[1] > 4.0,
        "o interior editado sobrevive: {:?}",
        p.verts[1].anchor
    );
}

/// **As PONTAS do spine seguem as fontes** — só o interior é editável (o Illustrator). O artista
/// tenta arrastar as pontas para longe; o recook as devolve aos centros das fontes; o interior fica.
#[test]
fn the_spine_endpoints_are_pinned_to_the_sources() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 9.0], [2.0, 3.0], [4.0, 9.0]]); // pontas afastadas

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

    let v = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .clone();
    assert!(
        v[0].anchor[1].abs() < 1e-9 && v[2].anchor[1].abs() < 1e-9,
        "as pontas voltaram aos centros (y=0): {:?} / {:?}",
        v[0].anchor,
        v[2].anchor
    );
    assert!(
        (v[1].anchor[1] - 3.0).abs() < 1e-9,
        "o interior fica: {:?}",
        v[1].anchor
    );
}

/// **Reset Spine volta ao automático — e NÃO re-autora.** É o caminho completo (ADR-0128 C2b): o
/// spine é automático (o recook o memoriza), depois autorado e curvado, depois resetado. A prova de
/// que o `spines.remove` é necessário: sem apagar a memória do auto, a detecção do recook seguinte
/// compararia o spine BENT ainda na cena com o auto memorizado (diferentes) e o RE-autoraria na
/// hora — o reset não pegaria. Por isso o teste passa pelo AUTO antes (senão a memória estaria
/// vazia e a mutação não apareceria).
#[test]
fn reset_spine_returns_to_the_automatic_straight_line_and_does_not_reauthor() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    let mut spines = BlendSpines::new();
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);

    // Frame 1: automático — o recook memoriza a reta em `spines` (é o que o reset precisa apagar).
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(spines.contains_key(&spine), "o auto foi memorizado");

    // A mão autora e curva o spine (modo Node); o recook authored NÃO atualiza `spines`.
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 5.0], [4.0, 0.0]]);
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);

    // Reset: limpa o flag E a memória do auto.
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&[spine]);
    assert!(
        reset_spine(&mut sim, &map, &pen, &mut spines),
        "resetou o spine"
    );
    assert!(
        !sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "o flag foi limpo"
    );

    // Frame seguinte: o ramo automático reescreve a RETA e NÃO re-autora (a memória sumiu).
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(
        !sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "o recook NÃO re-autora após o reset (a memória do auto foi apagada)"
    );
    let p = scene.paths().iter().find(|p| p.id == spine).expect("spine");
    assert_eq!(p.verts.len(), 2, "voltou à reta pelos 2 centros");
    for v in &p.verts {
        assert!(v.anchor[1].abs() < 1e-9, "reta em y=0: {:?}", v.anchor);
    }
}

/// Uma ponta fixada **segue a fonte** quando ela se move (a curva vai junto).
#[test]
fn a_pinned_spine_endpoint_follows_its_source() {
    let (mut sim, mut scene, map, spine, sources) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]);

    let e1 = Entity::from_bits(map[&sources[1]]);
    sim.world_mut()
        .get_mut::<Transform>(e1)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(0.0, 5.0);

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

    let last = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .last()
        .expect("ponta")
        .anchor;
    assert!(
        (last[1] - 5.0).abs() < 1e-6,
        "a ponta seguiu a fonte para y=5: {last:?}"
    );
}

/// **Arrastar uma ponta do spine MOVE a forma-fonte dela** (ADR-0128 C2b, ajuste do Enio) — o
/// inverso da pinagem. A fonte 1 está em (4,0); o artista arrasta a última âncora do spine para
/// (4,5) no modo Node, e a fonte SEGUE (seu centro vira (4,5)). É o que faz editar a curva no Node
/// equivaler a mover a forma no Select.
#[test]
fn dragging_a_spine_endpoint_moves_its_source() {
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(2, 3);
    // Auto spine = [(0,0), (4,0)]. A mão arrasta a última ponta para (4,5).
    set_spine(&mut scene, spine, &[[0.0, 0.0], [4.0, 5.0]]);
    let mut spines = BlendSpines::new();
    let xf = crate::vec_transform::build(&sim, &map);
    drag_spine_anchors_move_sources(&mut sim, &scene, &map, &xf, &mut spines);

    // A fonte 1 seguiu: seu centro é agora (4,5).
    let xf = crate::vec_transform::build(&sim, &map);
    let c = center_of(&scene, &xf, src[1]).expect("centro");
    assert!(
        (c[0] - 4.0).abs() < 1e-6 && (c[1] - 5.0).abs() < 1e-6,
        "a fonte seguiu a ponta arrastada: {c:?}"
    );
}

/// **Arrastar a âncora de uma forma do MEIO move a forma do MEIO** (ajuste do Enio: cadeias de 3+).
/// Três fontes em x = 0, 4, 8; o auto spine tem um vértice por fonte. O artista arrasta o vértice do
/// MEIO (índice 1, da fonte central) para (4,5), e a fonte central SEGUE — não só as pontas.
#[test]
fn dragging_a_middle_anchor_moves_the_middle_source() {
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(3, 3);
    // Auto spine = [(0,0),(4,0),(8,0)]. Arrasta o vértice do MEIO para (4,5).
    set_spine(&mut scene, spine, &[[0.0, 0.0], [4.0, 5.0], [8.0, 0.0]]);
    let mut spines = BlendSpines::new();
    let xf = crate::vec_transform::build(&sim, &map);
    drag_spine_anchors_move_sources(&mut sim, &scene, &map, &xf, &mut spines);

    // A fonte do MEIO (src[1]) seguiu; as das pontas (0 e 2) ficaram no lugar.
    let xf = crate::vec_transform::build(&sim, &map);
    let mid = center_of(&scene, &xf, src[1]).expect("centro do meio");
    assert!(
        (mid[0] - 4.0).abs() < 1e-6 && (mid[1] - 5.0).abs() < 1e-6,
        "a fonte do MEIO seguiu a âncora arrastada: {mid:?}"
    );
    let c0 = center_of(&scene, &xf, src[0]).expect("centro 0");
    let c2 = center_of(&scene, &xf, src[2]).expect("centro 2");
    assert!(
        c0[1].abs() < 1e-9 && c2[1].abs() < 1e-9,
        "as pontas não se moveram: {c0:?} / {c2:?}"
    );
}

/// **Mover a ponta NÃO autora o spine — e não há salto** (mover a forma ≠ curvar a curva). Passa
/// pelo auto primeiro (memoriza), arrasta a ponta, move a fonte + atualiza a memória do auto, e o
/// recook seguinte deixa o spine AUTOMÁTICO (a ponta ficou onde foi arrastada, sobre o novo centro).
/// Mutation-testado: sem atualizar a memória do auto (`spines`) no `drag_spine_anchors_move_sources`, a
/// detecção confundiria o movimento da forma com uma edição de curva e autoraria.
#[test]
fn dragging_an_endpoint_moves_the_source_without_authoring_the_spine() {
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    let mut spines = BlendSpines::new();
    let mut out = Vec::new();

    // Frame 1: auto — memoriza o auto.
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);

    // A mão arrasta a última ponta (modo Node): (4,0) → (4,5).
    set_spine(&mut scene, spine, &[[0.0, 0.0], [4.0, 5.0]]);
    let xf = crate::vec_transform::build(&sim, &map);
    drag_spine_anchors_move_sources(&mut sim, &scene, &map, &xf, &mut spines);

    // Frame 2: a fonte seguiu, o spine segue AUTOMÁTICO, e a ponta ficou onde foi arrastada.
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    let c = center_of(&scene, &xf, src[1]).expect("centro");
    assert!(
        (c[0] - 4.0).abs() < 1e-6 && (c[1] - 5.0).abs() < 1e-6,
        "a fonte seguiu: {c:?}"
    );
    assert!(
        !sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "mover a ponta NÃO autora o spine (mover a forma ≠ curvar a curva)"
    );
    let last = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .last()
        .expect("ponta")
        .anchor;
    assert!(
        (last[1] - 5.0).abs() < 1e-6,
        "a ponta ficou onde foi arrastada (sem salto de volta): {last:?}"
    );
}

/// Um spine autorado com um ponto de dobra LIVRE no meio (o artista criou um ponto além das duas
/// formas), e as `n` fontes movidas por `deltas[i]`. Devolve a âncora do ponto livre depois do
/// recook — é ela que tem de acompanhar (ou não).
fn free_bend_after_moving(deltas: &[[f32; 2]]) -> [f64; 2] {
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    // 3 vértices para 2 fontes: o do meio é LIVRE (`anchor_source_pairs` liga só a 1ª e a última).
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]);
    // **Dois frames com a MESMA memória** — é o que o produto faz, e é o que dá ao 2º frame um
    // "antes" com que comparar os centros. Um frame só nunca vê movimento nenhum.
    let mut mem = BlendSpines::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut mem, &mut Vec::new());
    // O artista arrasta as formas.
    for (id, d) in src.iter().zip(deltas) {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(Entity::from_bits(map[id]))
            .expect("Transform");
        t.translation.x += d[0];
        t.translation.y += d[1];
    }
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut mem, &mut Vec::new());
    scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts[1]
        .anchor
}

/// **Arrastar TODAS as formas leva a curva INTEIRA** (Enio 2026-07-16) — inclusive os pontos de
/// dobra que o artista criou além delas. Eles não pertencem a fonte nenhuma, então nada os movia: a
/// multi-seleção arrastava as formas e deixava a curva para trás, deformando a transição, quando o
/// que o artista fez foi mover o conjunto de lugar.
#[test]
fn dragging_every_source_together_carries_the_free_bend_points() {
    let moved = free_bend_after_moving(&[[5.0, -2.0], [5.0, -2.0]]);
    let want = [2.0 + 5.0, 3.0 - 2.0];
    assert!(
        (moved[0] - want[0]).abs() < 1e-9 && (moved[1] - want[1]).abs() < 1e-9,
        "o ponto livre tinha de andar o mesmo delta das formas: {moved:?} != {want:?}"
    );
}

/// …e mover UMA fonte só **não** leva os pontos livres: aí as formas se moveram uma em relação à
/// outra, cada âncora vai para o seu centro e a curva se DEFORMA entre elas — que é o que o artista
/// pediu ao mover uma só. É o outro lado da mesma decisão, e sem este gate "translada sempre"
/// passaria.
#[test]
fn moving_a_single_source_leaves_the_free_bend_points_alone() {
    let moved = free_bend_after_moving(&[[5.0, -2.0], [0.0, 0.0]]);
    assert!(
        (moved[0] - 2.0).abs() < 1e-9 && (moved[1] - 3.0).abs() < 1e-9,
        "o ponto livre nao podia se mexer: {moved:?}"
    );
}

/// **Em repouso o spine não anda um bit.** Nada se moveu ⇒ nada a transladar. A captura do undo é
/// tirada no fim do frame e o diff registra QUALQUER diferença como ação do usuário, então um passe
/// por-frame que ande um ulp vira um passo espúrio a cada frame (BUGS #15 — o "undo só faz uma
/// etapa"). Este gate é a rede sob o passe INTEIRO, não sob o early-return do `rigid_move`: aquele é
/// contrato, e nenhum mutante o distingue (transladar por zero já é exato).
#[test]
fn at_rest_the_spine_does_not_move_by_a_single_bit() {
    let before = free_bend_after_moving(&[[0.0, 0.0], [0.0, 0.0]]);
    assert_eq!(before, [2.0, 3.0], "repouso tem de ser bit-identico");
}
